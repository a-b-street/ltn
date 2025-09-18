use std::collections::HashMap;

use crate::map_model::Direction;
use crate::route::Router;
use crate::{Cell, MapModel, Neighbourhood, RoadID};
use geo::{Euclidean, Length, LineString};
use geojson::Feature;

pub struct Shortcuts {
    pub paths: Vec<Path>,
    pub count_per_road: HashMap<RoadID, usize>,
}

pub struct Path {
    // TODO: dedupe - make this `Route`
    steps: Vec<(RoadID, Direction)>,
    directness: f64,
    weight: usize,
}

impl Shortcuts {
    pub fn new(map: &MapModel, neighbourhood: &Neighbourhood, cells: &Vec<Cell>) -> Self {
        let router_input_after = neighbourhood.shortcuts_router_input_after(map);
        let router_input_before = neighbourhood.shortcuts_router_input_before(map);
        // Heavily penalize using main roads, so more shortcuts use local roads.
        let router_after = Router::new(&router_input_after, 2.0);

        let mut paths = Vec::new();
        let mut count_per_road = HashMap::new();

        // A shortcut is contained within a cell, because cells can't cross modal filters or main
        // roads, and neither can shortcuts. Only looking for candidate shortcuts in each cell is
        // much faster than using all pairs of the neighbourhood's border intersections, because
        // all the pairs belonging to different cells will need to go along or cross a main road to
        // achieve a route.
        for cell in cells {
            for start_i in &cell.border_intersections {
                let start_intersection = map.get_i(*start_i);
                for start_r in &start_intersection.roads {
                    // It's not a "shortcut" unless it starts outside the interior and cuts through
                    // the interior.
                    if !neighbourhood.main_roads.contains(start_r) {
                        continue;
                    }
                    for end_i in &cell.border_intersections {
                        if start_i == end_i {
                            continue;
                        }
                        let weight =
                            if map.is_major_junction(*start_i) && map.is_major_junction(*end_i) {
                                50
                            } else {
                                1
                            };
                        let end_intersection = map.get_i(*end_i);
                        'next_road: for end_r in &end_intersection.roads {
                            // It's not a "shortcut" unless it ends outside the interior after
                            // cutting through.
                            if !neighbourhood.main_roads.contains(end_r) {
                                continue;
                            }

                            // TODO We could get a little more precision by starting from the
                            // correct end of the road, but it doesn't matter
                            let Some(route) = router_after.route_from_roads(
                                &router_input_after,
                                *start_r,
                                *end_r,
                            ) else {
                                continue;
                            };

                            let mut shortcut_length = 0.0;

                            let interior_steps = {
                                let mut steps = route.steps.iter();
                                let first_step = steps.next().expect("route can't be empty");
                                let first_road = map.get_r(first_step.0);
                                shortcut_length += Euclidean.length(&first_road.linestring);
                                let mut steps_reversed = steps.rev();
                                let Some(final_step) = steps_reversed.next() else {
                                    continue;
                                };
                                let final_road = map.get_r(final_step.0);
                                shortcut_length += Euclidean.length(&final_road.linestring);

                                // re-reverse back to the original ordering, but without first and final steps
                                steps_reversed.rev()
                            };

                            let mut shortcut_roads = Vec::new();
                            for (r, _direction) in interior_steps {
                                // For the purpose of counting unique shortcuts, only the first and
                                // final steps should be on main roads.
                                //
                                // If we've left the interior, only the portion inside the interior
                                // counts as distinct plausible shortcut.
                                //
                                // If a route leaves and re-enters the interior through another
                                // border intersection, then that route will be counted as two
                                // distinct shortcuts.
                                if neighbourhood.main_roads.contains(&r) {
                                    break 'next_road;
                                }

                                shortcut_roads.push(*r);
                            }

                            // Only increase count_per_road after verifying above that the shortcut
                            // is indeed valid and doesn't cross any main roads
                            for r in shortcut_roads {
                                let road = map.get_r(r);
                                *count_per_road.entry(road.id).or_insert(0) += weight;
                                shortcut_length += Euclidean.length(&road.linestring);
                            }

                            // How long is the shortest route through the original router, using this
                            // neighbourhood or not?
                            let direct_length = match map.router_before.route_from_roads(
                                &router_input_before,
                                *start_r,
                                *end_r,
                            ) {
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
                                weight,
                            });
                        }
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
        f.set_property("weight", self.weight);
        f
    }
}
