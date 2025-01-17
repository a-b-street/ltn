use std::collections::HashMap;

use geo::{Contains, Coord};
use geojson::{Feature, FeatureCollection};
use nanorand::{Rng, WyRand};

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
            requests: synthetic_od_requests(map),
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
fn synthetic_od_requests(map: &MapModel) -> Vec<(IntersectionID, IntersectionID)> {
    let num_requests = 1_000;

    let mut rng = WyRand::new_seed(42);
    let mut requests = Vec::new();
    for _ in 0..num_requests {
        let i1 = IntersectionID(rng.generate_range(0..map.intersections.len()));
        let i2 = IntersectionID(rng.generate_range(0..map.intersections.len()));
        requests.push((i1, i2));
    }
    requests
}
