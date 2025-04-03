use std::cell::RefCell;
use std::collections::HashMap;

use fast_paths::{FastGraph, InputGraph, PathCalculator};
use geo::{Coord, Euclidean, Length, LineLocatePoint, LineString};
use itertools::Itertools;
use rstar::{primitives::GeomWithData, RTree};
use utils::{LineSplit, NodeMap};

use crate::map_model::{DiagonalFilter, Direction};
use crate::{
    Intersection, IntersectionID, MapModel, ModalFilter, Position, Road, RoadID, TravelFlow,
};

// For vehicles only
pub struct Router {
    ch: FastGraph,
    path_calculator: RefCell<PathCalculator>,
    node_map: NodeMap<(RoadID, Direction)>,
    pub main_road_penalty: f64,
}

#[derive(Debug, Clone)]
pub struct Route {
    pub steps: Vec<(RoadID, Direction)>,
    pub start: Position,
    pub end: Position,
}

/// Routable input can represent the entire map, or a neighborhood within a map.
///
/// Most of the methods delegate to the entire map (e.g. `get_i`), but `roads_iter` allows you to
/// specify a subset of roads, so we can build a routing graph focused within a single neighborhood.
pub trait RouterInput {
    fn roads_iter(&self) -> impl Iterator<Item = &Road>;
    fn closest_road(&self) -> &RTree<GeomWithData<LineString, RoadID>>;

    fn get_r(&self, r: RoadID) -> &Road;
    fn get_i(&self, i: IntersectionID) -> &Intersection;
    fn modal_filter(&self, r: RoadID) -> Option<&ModalFilter>;
    fn has_modal_filter(&self, r: RoadID) -> bool {
        self.modal_filter(r).is_some()
    }
    fn travel_flow(&self, r: RoadID) -> TravelFlow;
    fn diagonal_filter(&self, i: IntersectionID) -> Option<&DiagonalFilter>;
    fn turn_restrictions(&self, i: IntersectionID) -> &Vec<(RoadID, RoadID)>;

    fn snap_to_road(&self, pt: Coord) -> Position {
        let r = self
            .closest_road()
            .nearest_neighbor(&pt.into())
            .unwrap()
            .data;
        let road = self.get_r(r);
        let percent_along = road.linestring.line_locate_point(&pt.into()).unwrap();
        Position {
            road: road.id,
            percent_along,
        }
    }
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

            let penalty = if road.is_severance() {
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

    pub fn route_from_points(
        &self,
        router_input: &impl RouterInput,
        pt1: Coord,
        pt2: Coord,
    ) -> Option<Route> {
        self.route_from_positions(
            router_input,
            router_input.snap_to_road(pt1),
            router_input.snap_to_road(pt2),
        )
    }

    /// Routes from the middle of `start` to the middle of `end`.
    pub fn route_from_roads(
        &self,
        router_input: &impl RouterInput,
        start: RoadID,
        end: RoadID,
    ) -> Option<Route> {
        self.route_from_positions(
            router_input,
            Position {
                road: start,
                percent_along: 0.5,
            },
            Position {
                road: end,
                percent_along: 0.5,
            },
        )
    }

    pub fn route_from_positions(
        &self,
        router_input: &impl RouterInput,
        start: Position,
        end: Position,
    ) -> Option<Route> {
        if start == end {
            return None;
        }

        if start.road == end.road {
            return Some(Route {
                start,
                end,
                steps: vec![(
                    start.road,
                    Direction::forwards(start.percent_along < end.percent_along),
                )],
            });
        }

        // Nodes in the contraction hierarchy are (road, direction) pairs. The start and end of the
        // search can happen in two directions.
        let mut start_nodes = vec![];
        let mut end_nodes = vec![];
        for (nodes, position, is_start) in [
            (&mut start_nodes, start, true),
            (&mut end_nodes, end, false),
        ] {
            for direction in [Direction::Forwards, Direction::Backwards] {
                if let Some(node) = self.node_map.get((position.road, direction)) {
                    // Calculate the cost of the first or last road, which usually doesn't use the
                    // entire length of the road.
                    // Note this extra cost gets double-counted -- the contraction hierachy edge
                    // weights count crossing the full start and end road. But this is OK; the sum
                    // cost is only used to pick the shortest path.
                    let road = router_input.get_r(position.road);
                    let percent_of_length = if (direction == Direction::Forwards) == is_start {
                        // From the position to the end (dst_i) of this road
                        1.0 - position.percent_along
                    } else {
                        // From the position to the start (src_i) of this road
                        position.percent_along
                    };
                    let extra_cost = self.cost_for_road(road, percent_of_length);

                    nodes.push((node, extra_cost));
                }
            }

            // If the start or end road has a modal filter, then it won't be in the contraction
            // hierarchy. Depending on the filter position, we can only travel in one direction.
            // (If the position is EXACTLY on the filter position, arbitrarily pick one side)
            if let Some(filter) = router_input.modal_filter(position.road) {
                assert!(nodes.is_empty(), "a road with a filter is in the CH");
                let road = router_input.get_r(position.road);
                let (i, percent_of_length) = if position.percent_along <= filter.percent_along {
                    (road.src_i, position.percent_along)
                } else {
                    (road.dst_i, 1.0 - position.percent_along)
                };
                let extra_cost = self.cost_for_road(road, percent_of_length);

                if is_start {
                    for outgoing_road in router_input
                        .get_i(i)
                        .allowed_movements_from(position.road, router_input)
                    {
                        if let Some(node) = self.node_map.get(outgoing_road) {
                            nodes.push((node, extra_cost));
                        }
                    }
                } else {
                    for incoming_road in router_input
                        .get_i(i)
                        .allowed_movements_to(position.road, router_input)
                    {
                        if let Some(node) = self.node_map.get(incoming_road) {
                            nodes.push((node, extra_cost));
                        }
                    }
                }
            }
        }

        if start_nodes.is_empty() || end_nodes.is_empty() {
            return None;
        }

        let shortest_path = self
            .path_calculator
            .borrow_mut()
            .calc_path_multiple_sources_and_targets(&self.ch, start_nodes, end_nodes)?;

        let mut steps = Vec::new();
        for node in shortest_path.get_nodes() {
            let (road, direction) = self.node_map.translate_id(*node);
            steps.push((road, direction));
        }

        // If we started or ended on a filtered road, we need to insert the step for that
        if start.road != steps[0].0 {
            let filter = router_input
                .modal_filter(start.road)
                .expect("start road must have a filter");
            steps.insert(
                0,
                (
                    start.road,
                    Direction::forwards(start.percent_along > filter.percent_along),
                ),
            );
        }
        if end.road != steps.last().unwrap().0 {
            let filter = router_input
                .modal_filter(end.road)
                .expect("end road must have a filter");
            steps.push((
                end.road,
                Direction::forwards(end.percent_along <= filter.percent_along),
            ));
        }

        Some(Route { steps, start, end })
    }

    /// Produce routes for all the requests and count how many routes cross each road
    pub fn od_to_counts(
        &self,
        router_input: &impl RouterInput,
        requests: &Vec<(RoadID, RoadID, usize)>,
    ) -> HashMap<RoadID, usize> {
        let mut results = HashMap::new();
        for (r1, r2, count) in requests {
            if let Some(route) = self.route_from_roads(router_input, *r1, *r2) {
                for (r, _) in route.steps {
                    *results.entry(r).or_insert(0) += *count;
                }
            }
        }
        results
    }

    fn cost_for_road(&self, road: &Road, percent_of_length: f64) -> usize {
        let penalty = if road.is_severance() {
            self.main_road_penalty
        } else {
            1.0
        };
        (penalty * percent_of_length * road.cost_seconds() * 100.0) as usize
    }
}

impl Route {
    pub fn to_linestring(&self, map: &MapModel) -> LineString {
        let mut pts = Vec::new();

        for (pos, (road, direction)) in self.steps.iter().with_position() {
            pts.extend(self.slice_road_step(&map.get_r(*road).linestring, *direction, pos));
        }

        pts.dedup();
        LineString::new(pts)
    }

    /// Includes the start and end road, even if they're only partially crossed
    pub fn crosses_road(&self, road: RoadID) -> bool {
        self.steps.iter().any(|(r, _)| *r == road)
    }

    /// Returns (meters, seconds)
    pub fn get_distance_and_time(&self, map: &MapModel) -> (f64, f64) {
        let mut distance = 0.0;
        let mut time = 0.0;
        for (pos, (r, dir)) in self.steps.iter().with_position() {
            let road = &map.roads[r.0];

            let percent_of_length = match pos {
                itertools::Position::Only => {
                    (self.start.percent_along - self.end.percent_along).abs()
                }
                itertools::Position::First => {
                    if *dir == Direction::Forwards {
                        1.0 - self.start.percent_along
                    } else {
                        self.start.percent_along
                    }
                }
                itertools::Position::Middle => 1.0,
                itertools::Position::Last => {
                    if *dir == Direction::Forwards {
                        self.end.percent_along
                    } else {
                        1.0 - self.end.percent_along
                    }
                }
            };
            distance += percent_of_length * Euclidean.length(&road.linestring);
            time += percent_of_length * road.cost_seconds();
        }
        (distance, time)
    }

    /// Returns the points to glue together in order for one step of the route
    fn slice_road_step(
        &self,
        linestring: &LineString,
        dir: Direction,
        pos: itertools::Position,
    ) -> Vec<Coord> {
        let mut pts = match pos {
            itertools::Position::First => {
                let (a, b) = if dir == Direction::Forwards {
                    (self.start.percent_along, 1.0)
                } else {
                    (0.0, self.start.percent_along)
                };
                linestring
                    .line_split_twice(a, b)
                    .unwrap()
                    .into_second()
                    .map(|ls| ls.0)
                    // If start.percent_along is exactly 0 or 1, depending on the direction, then
                    // there are no actual points here
                    .unwrap_or_else(Vec::new)
            }
            itertools::Position::Last => {
                let (a, b) = if dir == Direction::Forwards {
                    (0.0, self.end.percent_along)
                } else {
                    (self.end.percent_along, 1.0)
                };
                linestring
                    .line_split_twice(a, b)
                    .unwrap()
                    .into_second()
                    .map(|ls| ls.0)
                    .unwrap_or_else(Vec::new)
            }
            itertools::Position::Middle => linestring.0.clone(),
            itertools::Position::Only => linestring
                .line_split_twice(self.start.percent_along, self.end.percent_along)
                .unwrap()
                .into_second()
                .map(|ls| ls.0)
                .unwrap_or_else(Vec::new),
        };
        if dir == Direction::Backwards {
            pts.reverse();
        }
        pts
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
        let Route { steps, .. } = map
            .router_before
            .route_from_roads(&map.router_input_before(), r(3), r(2))
            .unwrap();
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
        assert!(router_after
            .route_from_roads(&map.router_input_after(), r(2), r(3))
            .is_none());
        assert!(router_after
            .route_from_roads(&map.router_input_after(), r(3), r(2))
            .is_none());
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
        let Route {
            steps: valid_path, ..
        } = map
            .router_before
            .route_from_roads(&map.router_input_before(), r(0), r(3))
            .unwrap();
        assert_eq!(
            valid_path,
            vec![(r(0), Direction::Forwards), (r(3), Direction::Forwards)]
        );
        // attempting illegal left turn
        let invalid_path =
            map.router_before
                .route_from_roads(&map.router_input_before(), r(0), r(2));
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
            ..
        } = map
            .router_after
            .as_ref()
            .unwrap()
            .route_from_roads(&map.router_input_after(), r(3), r(1))
            .unwrap();
        assert_eq!(
            right_turn_path,
            vec![(r(3), Direction::Backwards), (r(1), Direction::Forwards)]
        );
        let left_turn_path = map.router_after.as_ref().unwrap().route_from_roads(
            &map.router_input_after(),
            r(3),
            r(0),
        );
        assert!(left_turn_path.is_none());
    }
}
