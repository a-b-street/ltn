use std::collections::HashMap;

use geojson::{Feature, FeatureCollection};

use crate::{od, MapModel, RoadID};

// TODO Rename?
/// Besides just studying the impact on shortcuts within one neighbourhood boundary, the user can
/// see how traffic changes across roads in the whole map. This works by finding the best route
/// before and after changes for every origin/destination "OD" pairs, then counting routes per
/// road.
pub struct Impact {
    // (r1, r2, count) -- `count` identical trips from `r1` to `r2`
    requests: Vec<(RoadID, RoadID, usize)>,

    // TODO Can use Vec for perf
    counts_before: HashMap<RoadID, usize>,
    counts_after: HashMap<RoadID, usize>,
}

impl Impact {
    /// Calculates `requests` only
    pub fn new(map: &MapModel, demand: Option<&od::DemandModel>) -> Self {
        Self {
            requests: match demand {
                Some(demand) => demand.make_requests(map),
                None => od::synthetic_od_requests(map),
            },
            counts_before: HashMap::new(),
            counts_after: HashMap::new(),
        }
    }

    pub fn invalidate_after_edits(&mut self) {
        self.counts_after.clear();
    }

    /// Returns a feature per road, with `before` and `after` counts, and a `max_count` foreign
    /// member
    pub fn recalculate(&mut self, map: &MapModel) -> FeatureCollection {
        if self.counts_before.is_empty() {
            info!("Calculating impacts before edits");
            self.counts_before = map.router_before.od_to_counts(&self.requests);
        }

        if self.counts_after.is_empty() {
            info!("Calculating impacts after edits");
            self.counts_after = map
                .router_after
                .as_ref()
                .expect("need to rebuild_router")
                .od_to_counts(&self.requests);
        }

        let mut features = Vec::new();
        let mut max_count = 0;
        for road in &map.roads {
            let before = self.counts_before.get(&road.id).cloned().unwrap_or(0);
            let after = self.counts_after.get(&road.id).cloned().unwrap_or(0);
            // Don't show unchanged roads, but to scale the absolute counts, do look at the max
            // count seen anywhere
            max_count = max_count.max(before.max(after));
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

    pub fn get_impacts_on_road(
        &self,
        map: &MapModel,
        road: RoadID,
    ) -> Vec<(Option<Feature>, Option<Feature>)> {
        let mut changed_paths = Vec::new();

        let router_after = map.router_after.as_ref().unwrap();

        // TODO We could remember the indices of requests that have changes
        for (r1, r2, _) in &self.requests {
            let route1 = map.router_before.route_from_roads(*r1, *r2);
            let route2 = router_after.route_from_roads(*r1, *r2);
            let crosses1 = route1
                .as_ref()
                .map(|route| route.crosses_road(road))
                .unwrap_or(false);
            let crosses2 = route2
                .as_ref()
                .map(|route| route.crosses_road(road))
                .unwrap_or(false);

            if crosses1 != crosses2 {
                let f1 = route1.map(|route| {
                    let mut f = map.mercator.to_wgs84_gj(&route.to_linestring(map));
                    f.set_property("kind", "before");
                    f
                });
                let f2 = route2.map(|route| {
                    let mut f = map.mercator.to_wgs84_gj(&route.to_linestring(map));
                    f.set_property("kind", "after");
                    f
                });
                changed_paths.push((f1, f2));
            }
        }

        changed_paths
    }
}
