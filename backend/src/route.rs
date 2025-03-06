use std::cell::RefCell;
use std::collections::HashMap;

use fast_paths::{FastGraph, InputGraph, PathCalculator};
use geo::{Coord, Euclidean, Length, LineString};
use utils::NodeMap;

use crate::map_model::{DiagonalFilter, Direction};
use crate::{Intersection, IntersectionID, MapModel, ModalFilter, Road, RoadID, TravelFlow};

// For vehicles only
pub struct Router {
    ch: FastGraph,
    path_calculator: RefCell<PathCalculator>,
    node_map: NodeMap<(RoadID, Direction)>,
    pub main_road_penalty: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    pub steps: Vec<(RoadID, Direction)>,
}

/// Routable input can represent the entire map, or a neighborhood within a map.
///
/// Most of the methods delegate to the entire map (e.g. `get_i`), but `roads_iter` allows you to
/// specify a subset of roads, so we can build a routing graph focused within a single neighborhood.
pub trait RouterInput {
    fn roads_iter(&self) -> impl Iterator<Item = &Road>;

    fn get_r(&self, r: RoadID) -> &Road;
    fn get_i(&self, i: IntersectionID) -> &Intersection;
    fn modal_filter(&self, r: RoadID) -> Option<&ModalFilter>;
    fn has_modal_filter(&self, r: RoadID) -> bool {
        self.modal_filter(r).is_some()
    }
    fn travel_flow(&self, r: RoadID) -> TravelFlow;
    fn diagonal_filter(&self, i: IntersectionID) -> Option<&DiagonalFilter>;
    fn turn_restrictions(&self, i: IntersectionID) -> &Vec<(RoadID, RoadID)>;
}

impl Router {
    pub fn empty() -> Self {
        let mut input_graph = InputGraph::new();
        let node_map = NodeMap::new();
        input_graph.freeze();
        let ch = fast_paths::prepare(&input_graph);
        let path_calculator = RefCell::new(fast_paths::create_calculator(&ch));

        Self {
            ch,
            path_calculator,
            node_map,
            main_road_penalty: 1.0,
        }
    }

    pub fn new(router_input: &impl RouterInput, main_road_penalty: f64) -> Self {
        let mut input_graph = InputGraph::new();
        let mut node_map = NodeMap::new();

        for road in router_input.roads_iter() {
            if router_input.has_modal_filter(road.id) {
                continue;
            }
            // Loops can't be part of a shortest path
            if road.src_i == road.dst_i {
                continue;
            }

            let penalty = if road.tags.is_any(
                "highway",
                vec![
                    "motorway",
                    "motorway_link",
                    "trunk",
                    "trunk_link",
                    "primary",
                    "primary_link",
                    "secondary",
                    "secondary_link",
                    "tertiary",
                    "tertiary_link",
                ],
            ) {
                main_road_penalty
            } else {
                1.0
            };
            let cost = (penalty * road.cost_seconds() * 100.0) as usize;

            let mut link_through_intersection =
                |intersection: &Intersection, direction: Direction| {
                    // a given NodeId might refer to a different physical feature across rebuilds of
                    // the routing graph - do not assume they are stable.
                    let from = node_map.get_or_insert((road.id, direction));
                    for outgoing_road in intersection.allowed_movements_from(road.id, router_input)
                    {
                        let to = node_map.get_or_insert(outgoing_road);
                        input_graph.add_edge(from, to, cost);
                    }
                };

            let travel_flow = router_input.travel_flow(road.id);
            if travel_flow.flows_forwards() {
                link_through_intersection(router_input.get_i(road.dst_i), Direction::Forwards);
            }
            if travel_flow.flows_backwards() {
                link_through_intersection(router_input.get_i(road.src_i), Direction::Backwards);
            }
        }
        input_graph.freeze();
        let ch = fast_paths::prepare(&input_graph);
        let path_calculator = RefCell::new(fast_paths::create_calculator(&ch));

        Self {
            ch,
            path_calculator,
            node_map,
            main_road_penalty,
        }
    }

    pub fn route(&self, map: &MapModel, pt1: Coord, pt2: Coord) -> Option<Route> {
        let start = map.closest_road.nearest_neighbor(&pt1.into()).unwrap().data;
        let end = map.closest_road.nearest_neighbor(&pt2.into()).unwrap().data;
        self.route_from_roads(start, end)
    }

    pub fn route_from_roads(&self, start: RoadID, end: RoadID) -> Option<Route> {
        if start == end {
            return None;
        }

        let mut starts = vec![];
        let mut ends = vec![];
        for direction in [Direction::Forwards, Direction::Backwards] {
            // We consider all start/end pairs equally.
            let extra_weight = 0;
            if let Some(start) = self.node_map.get((start, direction)) {
                starts.push((start, extra_weight));
            };
            if let Some(end) = self.node_map.get((end, direction)) {
                ends.push((end, extra_weight));
            };
        }
        if starts.is_empty() || ends.is_empty() {
            return None;
        }
        let shortest_path = self
            .path_calculator
            .borrow_mut()
            .calc_path_multiple_sources_and_targets(&self.ch, starts, ends)?;
        let steps: Vec<_> = shortest_path
            .get_nodes()
            .iter()
            .map(|n| self.node_map.translate_id(*n))
            .collect();
        Some(Route { steps })
    }

    /// Produce routes for all the requests and count how many routes cross each road
    pub fn od_to_counts(&self, requests: &Vec<(RoadID, RoadID, usize)>) -> HashMap<RoadID, usize> {
        let mut results = HashMap::new();
        for (r1, r2, count) in requests {
            if let Some(route) = self.route_from_roads(*r1, *r2) {
                for (r, _) in route.steps {
                    *results.entry(r).or_insert(0) += *count;
                }
            }
        }
        results
    }
}

impl Route {
    pub fn to_linestring(&self, map: &MapModel) -> LineString {
        let mut pts = Vec::new();
        for (r, direction) in &self.steps {
            let road = &map.roads[r.0];
            if *direction == Direction::Forwards {
                pts.extend(road.linestring.0.clone());
            } else {
                let mut rev = road.linestring.0.clone();
                rev.reverse();
                pts.extend(rev);
            }
        }
        pts.dedup();
        LineString::new(pts)
    }

    pub fn crosses_road(&self, road: RoadID) -> bool {
        self.steps.iter().any(|(r, _)| *r == road)
    }

    /// Returns (meters, seconds)
    pub fn get_distance_and_time(&self, map: &MapModel) -> (f64, f64) {
        let mut distance = 0.0;
        let mut time = 0.0;
        for (r, _) in &self.steps {
            let road = &map.roads[r.0];
            distance += Euclidean.length(&road.linestring);
            time += road.cost_seconds();
        }
        (distance, time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{osm_tests::load_osm_xml, FilterKind};

    fn r(road_id: usize) -> RoadID {
        RoadID(road_id)
    }

    #[test]
    fn basic_route() {
        // Topology: `IntersectionId(1)` is the center
        //
        // All the roads are 2-way. The arrows indicate the `road.src -> road.dst`
        //
        // ```notrust
        //              i2
        //               ↑
        //              r1
        //               |
        // i3 -- r2 --> i1 -- r3 --> i4
        //               ↑
        //              r0
        //               |
        //              i0
        // ```
        let mut map = load_osm_xml("simple_four_way_intersection");
        let Route { steps } = map.router_before.route_from_roads(r(3), r(2)).unwrap();
        assert_eq!(
            steps,
            vec![(r(3), Direction::Backwards), (r(2), Direction::Backwards)]
        );

        // Filter r2
        map.add_modal_filter(
            map.get_r(r(2)).linestring.0[0],
            Some(vec![r(2)]),
            FilterKind::NoEntry,
        );
        map.rebuild_router(1.0);
        let router_after = map.router_after.as_ref().unwrap();
        // A route starting or ending there should fail
        assert!(router_after.route_from_roads(r(2), r(3)).is_none());
        assert!(router_after.route_from_roads(r(3), r(2)).is_none());
    }

    #[test]
    fn oneway_route() {
        // Arrows represent *both* road.src_i->road.dst_i and (in this cae) also direction of travel.
        //
        // ```notrust
        //              i2
        //               ↑
        //              r1
        //               |
        // i3 -- r2 --> i1 -- r3 --> i4
        //               ↑
        //              r0
        //               |
        //              i0
        // ```
        let map = load_osm_xml("two_crossing_one_ways");
        let Route { steps: valid_path } = map.router_before.route_from_roads(r(0), r(3)).unwrap();
        assert_eq!(
            valid_path,
            vec![(r(0), Direction::Forwards), (r(3), Direction::Forwards)]
        );
        // attempting illegal left turn
        let invalid_path = map.router_before.route_from_roads(r(0), r(2));
        assert!(invalid_path.is_none());
    }

    #[test]
    fn no_left_turns_route() {
        // Roads are two-way, arrows indicate road.src_i -> road.dst_i
        // ```notrust
        //              i2
        //               ↑
        //              r1
        //               |
        // i3 -- r2 --> i1 -- r3 --> i4
        //               ↑
        //              r0
        //               |
        //              i0
        // ```
        let mut map = load_osm_xml("simple_four_way_intersection");

        map.turn_restrictions[1] = vec![(r(1), r(3)), (r(3), r(0)), (r(0), r(2)), (r(2), r(1))];
        map.rebuild_router(1.1);

        let Route {
            steps: right_turn_path,
        } = map
            .router_after
            .as_ref()
            .unwrap()
            .route_from_roads(r(3), r(1))
            .unwrap();
        assert_eq!(
            right_turn_path,
            vec![(r(3), Direction::Backwards), (r(1), Direction::Forwards)]
        );
        let left_turn_path = map
            .router_after
            .as_ref()
            .unwrap()
            .route_from_roads(r(3), r(0));
        assert!(left_turn_path.is_none());
    }
}
