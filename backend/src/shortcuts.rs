use std::collections::HashMap;

use fast_paths::InputGraph;
use geo::LineString;

use crate::{IntersectionID, MapModel, Neighbourhood, NodeMap, RoadID};

pub struct Shortcuts {
    pub paths: Vec<Path>,
    pub count_per_road: HashMap<RoadID, usize>,
}

pub struct Path {
    steps: Vec<(RoadID, IntersectionID, IntersectionID)>,
}

impl Shortcuts {
    pub fn new(map: &MapModel, neighbourhood: &Neighbourhood) -> Self {
        let mut input_graph = InputGraph::new();
        let mut node_map = NodeMap::new();

        for r in &neighbourhood.interior_roads {
            if map.modal_filters.contains_key(r) {
                continue;
            }

            let road = map.get_r(*r);
            let i1 = node_map.get_or_insert(road.src_i);
            let i2 = node_map.get_or_insert(road.dst_i);
            let cost = (road.length() * 100.0) as usize;
            // TODO Look at one-way for driving
            input_graph.add_edge(i1, i2, cost);
            input_graph.add_edge(i2, i1, cost);
        }
        input_graph.freeze();
        let ch = fast_paths::prepare(&input_graph);
        let mut path_calc = fast_paths::create_calculator(&ch);

        let mut paths = Vec::new();
        let mut count_per_road = HashMap::new();
        for start in &neighbourhood.border_intersections {
            for end in &neighbourhood.border_intersections {
                if let (Some(i1), Some(i2)) = (node_map.get(*start), node_map.get(*end)) {
                    if let Some(path) = path_calc.calc_path(&ch, i1, i2) {
                        let mut steps = Vec::new();
                        for pair in path.get_nodes().windows(2) {
                            let i1 = node_map.translate_id(pair[0]);
                            let i2 = node_map.translate_id(pair[1]);
                            let road = map.find_edge(i1, i2);
                            steps.push((road.id, i1, i2));
                            *count_per_road.entry(road.id).or_insert(0) += 1;
                        }
                        paths.push(Path { steps });
                    }
                }
            }
        }

        Self {
            paths,
            count_per_road,
        }
    }

    pub fn subset(&self, crosses: RoadID) -> Vec<&Path> {
        self.paths
            .iter()
            .filter(|path| path.steps.iter().any(|(r, _, _)| *r == crosses))
            .collect()
    }
}

impl Path {
    pub fn geometry(&self, map: &MapModel) -> LineString {
        let mut pts = Vec::new();
        for (r, i1, i2) in &self.steps {
            let road = map.get_r(*r);
            if *i1 == road.src_i {
                pts.extend(road.linestring.0.clone());
            } else {
                let mut rev = road.linestring.0.clone();
                rev.reverse();
                pts.extend(rev);
            }
        }
        LineString::new(pts)
    }
}
