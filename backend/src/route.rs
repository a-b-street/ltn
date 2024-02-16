use std::collections::BTreeMap;

use fast_paths::{FastGraph, InputGraph};
use geo::{Coord, LineString};
use rstar::primitives::GeomWithData;
use rstar::RTree;

use crate::{
    Direction, Intersection, IntersectionID, MapModel, ModalFilter, NodeMap, Road, RoadID,
};

#[derive(Clone)]
pub struct Router {
    ch: FastGraph,
    node_map: NodeMap<IntersectionID>,
    closest_intersection: RTree<IntersectionLocation>,
}

// fast_paths ID representing the IntersectionID as the data
type IntersectionLocation = GeomWithData<[f64; 2], usize>;

impl Router {
    pub fn new(
        roads: &Vec<Road>,
        intersections: &Vec<Intersection>,
        modal_filters: &BTreeMap<RoadID, ModalFilter>,
        directions: &BTreeMap<RoadID, Direction>,
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
            let cost = (road.length() * 100.0) as usize;
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

        let mut points = Vec::new();
        for i in intersections {
            if let Some(node) = node_map.get(i.id) {
                points.push(IntersectionLocation::new(i.point.into(), node));
            }
        }
        let closest_intersection = RTree::bulk_load(points);

        Self {
            ch,
            node_map,
            closest_intersection,
        }
    }

    pub fn route(&self, map: &MapModel, pt1: Coord, pt2: Coord) -> Option<LineString> {
        let start = self
            .closest_intersection
            .nearest_neighbor(&[pt1.x, pt1.y])
            .unwrap()
            .data;
        let end = self
            .closest_intersection
            .nearest_neighbor(&[pt2.x, pt2.y])
            .unwrap()
            .data;
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
}
