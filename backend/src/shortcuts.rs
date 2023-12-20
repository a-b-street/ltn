use std::collections::HashMap;

use fast_paths::InputGraph;

use crate::node_map::NodeMap;
use crate::{MapModel, Neighbourhood, RoadID};

pub struct Shortcuts {
    pub count_per_road: HashMap<RoadID, usize>,
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

        let mut count_per_road = HashMap::new();
        for start in &neighbourhood.border_intersections {
            for end in &neighbourhood.border_intersections {
                if let (Some(i1), Some(i2)) = (node_map.get(*start), node_map.get(*end)) {
                    if let Some(path) = path_calc.calc_path(&ch, i1, i2) {
                        for pair in path.get_nodes().windows(2) {
                            let i1 = node_map.translate_id(pair[0]);
                            let i2 = node_map.translate_id(pair[1]);
                            let road = map.find_edge(i1, i2);
                            *count_per_road.entry(road.id).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        Self { count_per_road }
    }
}
