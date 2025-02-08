use geo::{BoundingRect, Contains, MultiPolygon, Point};
use geojson::GeoJson;
use nanorand::{Rng, WyRand};
use serde::{Deserialize, Serialize};
use utils::Mercator;

use crate::{MapModel, RoadID};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ZoneID(pub usize);

/// Origin/destination demand data, representing driving trips between two zones.
#[derive(Serialize, Deserialize)]
pub struct DemandModel {
    pub zones: Vec<Zone>,
    // (zone1, zone2, count), with count being the number of trips from zone1 to zone2
    pub desire_lines: Vec<(ZoneID, ZoneID, usize)>,
}

impl DemandModel {
    /// Turn all of the zones into Mercator. Don't do this when originally building and serializing
    /// them, because that process might not use exactly the same Mercator object.
    pub fn finish_loading(&mut self, mercator: &Mercator) {
        for zone in &mut self.zones {
            mercator.to_mercator_in_place(&mut zone.geometry);
            let bbox = zone.geometry.bounding_rect().unwrap();
            zone.x1 = (bbox.min().x * 100.0) as i64;
            zone.y1 = (bbox.min().y * 100.0) as i64;
            zone.x2 = (bbox.max().x * 100.0) as i64;
            zone.y2 = (bbox.max().y * 100.0) as i64;
        }
    }

    pub fn make_requests(&self, map: &MapModel) -> Vec<(RoadID, RoadID, usize)> {
        info!(
            "Making requests from {} zones and {} desire lines",
            self.zones.len(),
            self.desire_lines.len()
        );

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
                let pt1 = self.zones[zone1.0].random_point(&mut rng);
                let pt2 = self.zones[zone2.0].random_point(&mut rng);
                if let (Some(i1), Some(i2)) = (
                    map.closest_road.nearest_neighbor(&pt1).map(|obj| obj.data),
                    map.closest_road.nearest_neighbor(&pt2).map(|obj| obj.data),
                ) {
                    if i1 != i2 {
                        requests.push((i1, i2, trip_count));
                    }
                }
            }
        }
        requests
    }

    pub fn to_gj(&self, map: &MapModel) -> GeoJson {
        // Per (from, to) pair, how many trips?
        let mut from: Vec<Vec<usize>> =
            std::iter::repeat_with(|| std::iter::repeat(0).take(self.zones.len()).collect())
                .take(self.zones.len())
                .collect();
        // Per (to, from) pair, how many trips?
        let mut to: Vec<Vec<usize>> =
            std::iter::repeat_with(|| std::iter::repeat(0).take(self.zones.len()).collect())
                .take(self.zones.len())
                .collect();

        for (zone1, zone2, count) in &self.desire_lines {
            from[zone1.0][zone2.0] += count;
            to[zone2.0][zone1.0] += count;
        }

        let mut features = Vec::new();
        for (idx, zone) in self.zones.iter().enumerate() {
            let mut f = map.mercator.to_wgs84_gj(&zone.geometry);
            f.set_property("name", zone.name.clone());
            f.set_property("counts_from", std::mem::take(&mut from[idx]));
            f.set_property("counts_to", std::mem::take(&mut to[idx]));
            features.push(f);
        }
        GeoJson::from(features)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Zone {
    /// An original opaque string ID, from different data sources.
    pub name: String,
    /// WGS84 when built originally and serialized, then Mercator right before being used
    pub geometry: MultiPolygon,
    /// The bbox, rounded to centimeters, for generate_range to work. Only calculated right before
    /// use.
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
pub fn synthetic_od_requests(map: &MapModel) -> Vec<(RoadID, RoadID, usize)> {
    let num_requests = 1_000;

    let mut rng = WyRand::new_seed(42);
    let mut requests = Vec::new();
    while requests.len() != num_requests {
        let r1 = RoadID(rng.generate_range(0..map.roads.len()));
        let r2 = RoadID(rng.generate_range(0..map.roads.len()));
        if r1 != r2 {
            requests.push((r1, r2, 1));
        }
    }
    requests
}
