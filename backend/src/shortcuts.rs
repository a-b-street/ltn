use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::{MapModel, Neighbourhood, RoadID};

pub struct Shortcuts {
    pub count_per_road: HashMap<RoadID, usize>,
}

impl Shortcuts {
    pub fn new(map: &MapModel, neighbourhood: &Neighbourhood) -> Self {
        let mut graph = DiGraphMap::new();
        for r in &neighbourhood.interior_roads {
            let road = map.get_r(*r);
            graph.add_edge(road.src_i, road.dst_i, (road.id, road.length()));
            // TODO Look at one-way for driving
            graph.add_edge(road.dst_i, road.src_i, (road.id, road.length()));
        }

        let mut count_per_road = HashMap::new();
        for start in &neighbourhood.border_intersections {
            for end in &neighbourhood.border_intersections {
                if let Some((_, path)) = petgraph::algo::astar(
                    &graph,
                    *start,
                    |i| i == *end,
                    |(_, _, (_, dist))| *dist,
                    |_| 0.0,
                ) {
                    for pair in path.windows(2) {
                        let (r, _) = *graph.edge_weight(pair[0], pair[1]).unwrap();
                        *count_per_road.entry(r).or_insert(0) += 1;
                    }
                }
            }
        }

        Self { count_per_road }
    }
}
