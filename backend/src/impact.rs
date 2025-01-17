use std::collections::HashMap;

use geo::{Contains, Coord};
use geojson::GeoJson;

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

    /// Returns a feature per road, with `before` and `after` counts
    pub fn recalculate(&mut self, map: &MapModel) -> GeoJson {
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
        for road in &map.roads {
            let before = self.baseline_counts.get(&road.id).cloned().unwrap_or(0);
            let after = self.after_edits_counts.get(&road.id).cloned().unwrap_or(0);
            if before > 0 || after > 0 {
                let mut f = map.mercator.to_wgs84_gj(&road.linestring);
                f.set_property("before", before);
                f.set_property("after", after);
                features.push(f);
            }
        }
        GeoJson::from(features)
    }
}

/// Deterministically produce a bunch of OD pairs, just as a fallback when there's no real data
fn synthetic_od_requests(map: &MapModel) -> Vec<(Coord, Coord)> {
    // TODO Or just directly use intersections and save the step of using closest_intersection?

    let step_size_meters = 50;
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
