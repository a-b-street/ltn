use std::collections::HashMap;

use geo::{BoundingRect, Contains, MultiPolygon, Point};
use nanorand::{Rng, WyRand};
use serde::{Deserialize, Serialize};

use crate::{IntersectionID, MapModel};

/// Origin/destination demand data, representing driving trips between two zones.
#[derive(Serialize, Deserialize)]
pub struct DemandModel {
    pub zones: HashMap<String, Zone>,
    // (zone1, zone2, count), with count being the number of trips between the two zones
    pub desire_lines: Vec<(String, String, usize)>,
}

impl DemandModel {
    pub fn make_requests(mut self, map: &MapModel) -> Vec<(IntersectionID, IntersectionID, usize)> {
        info!(
            "Making requests from {} zones and {} desire lines",
            self.zones.len(),
            self.desire_lines.len()
        );

        // Turn all of the zones into Mercator. Don't do this when originally building and
        // serializing them, because that process might not use exactly the same Mercator object.
        for zone in self.zones.values_mut() {
            map.mercator.to_mercator_in_place(&mut zone.geometry);
            let bbox = zone.geometry.bounding_rect().unwrap();
            zone.x1 = (bbox.min().x * 100.0) as i64;
            zone.y1 = (bbox.min().y * 100.0) as i64;
            zone.x2 = (bbox.max().x * 100.0) as i64;
            zone.y2 = (bbox.max().y * 100.0) as i64;
        }

        // TODO Plumb through UI
        // To speed up the impact calculation, how many specific requests per (zone1, zone2)? If
        // true, just do one, but weight it by count.
        let fast_sample = true;

        let mut rng = WyRand::new_seed(42);
        let mut requests = Vec::new();

        for (zone1, zone2, raw_count) in &self.desire_lines {
            let (iterations, trip_count) = if fast_sample {
                (1, *raw_count)
            } else {
                (*raw_count, 1)
            };

            for _ in 0..iterations {
                let pt1 = self.zones[zone1].random_point(&mut rng);
                let pt2 = self.zones[zone2].random_point(&mut rng);
                if let (Some(i1), Some(i2)) = (
                    map.closest_intersection
                        .nearest_neighbor(&pt1)
                        .map(|obj| obj.data),
                    map.closest_intersection
                        .nearest_neighbor(&pt2)
                        .map(|obj| obj.data),
                ) {
                    if i1 != i2 {
                        requests.push((i1, i2, trip_count));
                    }
                }
            }
        }
        requests
    }
}

#[derive(Serialize, Deserialize)]
pub struct Zone {
    // WGS84 when built originally and serialized, then Mercator right before being used
    pub geometry: MultiPolygon,
    // The bbox, rounded to centimeters, for generate_range to work. Only calculated right before
    // use.
    #[serde(skip_deserializing, skip_serializing)]
    pub x1: i64,
    #[serde(skip_deserializing, skip_serializing)]
    pub y1: i64,
    #[serde(skip_deserializing, skip_serializing)]
    pub x2: i64,
    #[serde(skip_deserializing, skip_serializing)]
    pub y2: i64,
}

impl Zone {
    fn random_point(&self, rng: &mut WyRand) -> Point {
        loop {
            let x = (rng.generate_range(self.x1..=self.x2) as f64) / 100.0;
            let y = (rng.generate_range(self.y1..=self.y2) as f64) / 100.0;
            let pt = Point::new(x, y);
            if self.geometry.contains(&pt) {
                return pt;
            }
        }
    }
}

/// Deterministically produce a bunch of OD pairs, just as a fallback when there's no real data
pub fn synthetic_od_requests(map: &MapModel) -> Vec<(IntersectionID, IntersectionID, usize)> {
    let num_requests = 1_000;

    let mut rng = WyRand::new_seed(42);
    let mut requests = Vec::new();
    for _ in 0..num_requests {
        let i1 = IntersectionID(rng.generate_range(0..map.intersections.len()));
        let i2 = IntersectionID(rng.generate_range(0..map.intersections.len()));
        requests.push((i1, i2, 1));
    }
    requests
}
