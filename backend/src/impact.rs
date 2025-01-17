use std::collections::HashMap;

use geo::{Contains, Coord};
use geojson::{Feature, FeatureCollection};

use crate::{IntersectionID, MapModel, RoadID};

// TODO Rename / explain what this does
pub struct Impact {
    requests: Vec<(IntersectionID, IntersectionID)>,

    // TODO Can use Vec for perf
    baseline_counts: HashMap<RoadID, usize>,
    after_edits_counts: HashMap<RoadID, usize>,
}

impl Impact {
    /// Calculates `requests` only
    pub fn new(map: &MapModel) -> Self {
        Self {
            requests: make_requests(map, synthetic_od_requests(map)),
            baseline_counts: HashMap::new(),
            after_edits_counts: HashMap::new(),
        }
    }

    pub fn invalidate_after_edits(&mut self) {
        self.after_edits_counts.clear();
    }

    /// Returns a feature per road, with `before` and `after` counts, and a `max_count` foreign
    /// member
    pub fn recalculate(&mut self, map: &MapModel) -> FeatureCollection {
        if self.baseline_counts.is_empty() {
            info!("Calculating baseline impacts first");
            self.baseline_counts = map
                .router_original
                .as_ref()
                .unwrap()
                .od_to_counts(map, &self.requests);
        }

        if self.after_edits_counts.is_empty() {
            info!("Calculating impacts after edits");
            self.after_edits_counts = map
                .router_current
                .as_ref()
                .expect("need to rebuild_router")
                .od_to_counts(map, &self.requests);
        }

        let mut features = Vec::new();
        let mut max_count = 0;
        for road in &map.roads {
            let before = self.baseline_counts.get(&road.id).cloned().unwrap_or(0);
            let after = self.after_edits_counts.get(&road.id).cloned().unwrap_or(0);
            max_count = max_count.max(before.max(after));
            // Don't show unchanged roads
            if before != after && (before > 0 || after > 0) {
                let mut f = map.mercator.to_wgs84_gj(&road.linestring);
                f.set_property("id", road.id.0);
                f.set_property("before", before);
                f.set_property("after", after);
                features.push(f);
            }
        }

        FeatureCollection {
            features,
            bbox: None,
            foreign_members: Some(
                serde_json::json!({
                    "max_count": max_count,
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        }
    }

    pub fn get_impacts_on_road(&self, map: &MapModel, road: RoadID) -> Vec<(Feature, Feature)> {
        let mut changed_paths = Vec::new();

        let router_before = map.router_original.as_ref().unwrap();
        let router_after = map.router_current.as_ref().unwrap();

        // TODO We could remember the indices of requests that have changes
        for (i1, i2) in &self.requests {
            let Some(route1) = router_before.route_from_intersections(map, *i1, *i2) else {
                continue;
            };
            let Some(route2) = router_after.route_from_intersections(map, *i1, *i2) else {
                continue;
            };
            if route1.crosses_road(road) != route2.crosses_road(road) {
                let mut f1 = map.mercator.to_wgs84_gj(&route1.to_linestring(map));
                f1.set_property("kind", "before");
                let mut f2 = map.mercator.to_wgs84_gj(&route2.to_linestring(map));
                f2.set_property("kind", "after");
                changed_paths.push((f1, f2));
            }
        }

        changed_paths
    }
}

/// Deterministically produce a bunch of OD pairs, just as a fallback when there's no real data
fn synthetic_od_requests(map: &MapModel) -> Vec<(Coord, Coord)> {
    // TODO Or just directly use intersections and save the step of using closest_intersection?

    let step_size_meters = 10;
    let boundary = map.mercator.to_mercator(&map.boundary_wgs84);

    let mut pts = Vec::new();
    for x in (0..map.mercator.width as usize).step_by(step_size_meters) {
        for y in (0..map.mercator.height as usize).step_by(step_size_meters) {
            let pt = Coord {
                x: x as f64,
                y: y as f64,
            };
            if boundary.contains(&pt) {
                pts.push(pt);
            }
        }
    }

    // Jumble them up without pulling in dependencies on RNGs
    pts.sort_by_key(|pt| (((pt.x + pt.y) * 1000.0) as usize) % 100);

    pts.windows(2).map(|pair| (pair[0], pair[1])).collect()
}

fn make_requests(
    map: &MapModel,
    pts: Vec<(Coord, Coord)>,
) -> Vec<(IntersectionID, IntersectionID)> {
    let mut requests = Vec::new();
    for (pt1, pt2) in pts {
        if let (Some(a), Some(b)) = (
            map.closest_intersection.nearest_neighbor(&pt1.into()),
            map.closest_intersection.nearest_neighbor(&pt2.into()),
        ) {
            requests.push((a.data, b.data));
        }
    }
    requests
}
