use std::collections::HashMap;

use geo::{Euclidean, Length, LineString};
use geojson::Feature;

use crate::map_model::Direction;
use crate::route::Router;
use crate::{MapModel, Neighbourhood, RoadID};

pub struct Shortcuts {
    pub paths: Vec<Path>,
    pub count_per_road: HashMap<RoadID, usize>,
}

pub struct Path {
    // TODO: dedupe - make this `Route`
    steps: Vec<(RoadID, Direction)>,
    directness: f64,
}

impl Shortcuts {
    pub fn new(map: &MapModel, neighbourhood: &Neighbourhood) -> Self {
        let router_input = neighbourhood.router_input(map);
        let router = Router::new(&router_input, 1.0);
        let mut paths = Vec::new();
        let mut count_per_road = HashMap::new();
        for start_i in &neighbourhood.border_intersections {
            let start_intersection = map.get_i(*start_i);
            for start_r in &start_intersection.roads {
                // It's not a "shortcut" unless it cuts through the interior.
                if !neighbourhood.interior_roads.contains(start_r) {
                    continue;
                }
                for end_i in &neighbourhood.border_intersections {
                    if start_i == end_i {
                        continue;
                    }
                    let end_intersection = map.get_i(*end_i);
                    for end_r in &end_intersection.roads {
                        // It's not a "shortcut" unless it cuts through the interior.
                        if !neighbourhood.interior_roads.contains(end_r) {
                            continue;
                        }
                        let Some(route) = router.route_from_roads(*start_r, *end_r) else {
                            continue;
                        };
                        let mut shortcut_length = 0.0;
                        for (r, _direction) in &route.steps {
                            let road = map.get_r(*r);
                            *count_per_road.entry(road.id).or_insert(0) += 1;
                            shortcut_length += Euclidean.length(&road.linestring);
                        }

                        // How long is the shortest route through the original router, using this
                        // neighbourhood or not?
                        let direct_length = match map
                            .router_before
                            .route_from_roads(*start_r, *end_r)
                        {
                            Some(route) => Euclidean.length(&route.to_linestring(map)),
                            None => {
                                warn!("Found a shortcut from {start_r} to {end_r}, but not a route using the whole map");
                                shortcut_length
                            }
                        };
                        let directness = shortcut_length / direct_length;
                        paths.push(Path {
                            steps: route.steps,
                            directness,
                        });
                    }
                }
            }
        }

        paths.sort_by_key(|path| (path.directness * 100.0) as usize);

        Self {
            paths,
            count_per_road,
        }
    }

    pub fn subset(&self, crosses: RoadID) -> Vec<&Path> {
        self.paths
            .iter()
            .filter(|path| path.steps.iter().any(|(r, _)| *r == crosses))
            .collect()
    }
}

impl Path {
    pub fn to_gj(&self, map: &MapModel) -> Feature {
        let mut pts = Vec::new();
        for (r, direction) in &self.steps {
            let road = map.get_r(*r);
            if *direction == Direction::Forwards {
                pts.extend(road.linestring.0.clone());
            } else {
                // PERF: reverse iter to avoid clone
                let mut rev = road.linestring.0.clone();
                rev.reverse();
                pts.extend(rev);
            }
        }
        let linestring = LineString::new(pts);

        let length = Euclidean.length(&linestring);
        let mut f = map.mercator.to_wgs84_gj(&linestring);
        f.set_property("directness", self.directness);
        f.set_property("length_meters", length);
        f
    }
}
