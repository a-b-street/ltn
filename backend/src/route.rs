use std::collections::{BTreeMap, HashMap};

use fast_paths::{FastGraph, InputGraph};
use geo::{Coord, LineString};
use utils::NodeMap;

use crate::{Direction, IntersectionID, MapModel, ModalFilter, Road, RoadID};

// For vehicles only
#[derive(Clone)]
pub struct Router {
    ch: FastGraph,
    node_map: NodeMap<IntersectionID>,
    pub main_road_penalty: f64,
}

impl Router {
    pub fn new(
        roads: &Vec<Road>,
        modal_filters: &BTreeMap<RoadID, ModalFilter>,
        directions: &BTreeMap<RoadID, Direction>,
        main_road_penalty: f64,
    ) -> Self {
        let mut input_graph = InputGraph::new();
        let mut node_map = NodeMap::new();

        for road in roads {
            if modal_filters.contains_key(&road.id) {
                continue;
            }
            // Loops can't be part of a shortest path
            if road.src_i == road.dst_i {
                continue;
            }

            let i1 = node_map.get_or_insert(road.src_i);
            let i2 = node_map.get_or_insert(road.dst_i);
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
            let cost = (penalty * road.length() * 100.0) as usize;
            match directions[&road.id] {
                Direction::Forwards => {
                    input_graph.add_edge(i1, i2, cost);
                }
                Direction::Backwards => {
                    input_graph.add_edge(i2, i1, cost);
                }
                Direction::BothWays => {
                    input_graph.add_edge(i1, i2, cost);
                    input_graph.add_edge(i2, i1, cost);
                }
            }
        }
        input_graph.freeze();
        let ch = fast_paths::prepare(&input_graph);

        Self {
            ch,
            node_map,
            main_road_penalty,
        }
    }

    pub fn route(&self, map: &MapModel, pt1: Coord, pt2: Coord) -> Option<LineString> {
        // TODO Find the closest neighbor in the node_map!
        let start = self.node_map.get(
            map.closest_intersection
                .nearest_neighbor(&pt1.into())
                .unwrap()
                .data,
        )?;
        let end = self.node_map.get(
            map.closest_intersection
                .nearest_neighbor(&pt2.into())
                .unwrap()
                .data,
        )?;
        if start == end {
            return None;
        }

        // TODO Reuse
        let mut path_calc = fast_paths::create_calculator(&self.ch);
        let path = path_calc.calc_path(&self.ch, start, end)?;

        let mut pts = Vec::new();
        for pair in path.get_nodes().windows(2) {
            let i1 = self.node_map.translate_id(pair[0]);
            let i2 = self.node_map.translate_id(pair[1]);
            let road = map.find_edge(i1, i2);

            if road.src_i == i1 {
                pts.extend(road.linestring.0.clone());
            } else {
                let mut rev = road.linestring.0.clone();
                rev.reverse();
                pts.extend(rev);
            }
        }
        pts.dedup();
        Some(LineString::new(pts))
    }

    /// Produce routes for all the requests and count how many routes cross each road
    pub fn od_to_counts(
        &self,
        map: &MapModel,
        requests: &Vec<(IntersectionID, IntersectionID)>,
    ) -> HashMap<RoadID, usize> {
        // TODO Reuse
        let mut path_calc = fast_paths::create_calculator(&self.ch);

        let mut results = HashMap::new();
        for (req_i1, req_i2) in requests {
            if let (Some(start), Some(end)) =
                (self.node_map.get(*req_i1), self.node_map.get(*req_i2))
            {
                if let Some(path) = path_calc.calc_path(&self.ch, start, end) {
                    for pair in path.get_nodes().windows(2) {
                        let i1 = self.node_map.translate_id(pair[0]);
                        let i2 = self.node_map.translate_id(pair[1]);
                        let road = map.find_edge(i1, i2);
                        *results.entry(road.id).or_insert(0) += 1;
                    }
                }
            }
        }
        results
    }
}
