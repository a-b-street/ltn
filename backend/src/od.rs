use std::collections::BTreeMap;
use geo::{MultiPolygon, PreparedGeometry, Relate};
use geojson::GeoJson;
use nanorand::{Rng, WyRand};
use serde::{Deserialize, Serialize};

use crate::{MapModel, RoadID};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ZoneID(pub usize);

/// Origin/destination demand data, representing driving trips between two zones.
#[derive(Serialize, Deserialize)]
pub struct DemandModel {
    pub zones: Vec<Zone>,
    #[serde(skip)]
    pub cached_zone_roads: Vec<Vec<RoadID>>,
    // (zone1, zone2, count), with count being the number of trips from zone1 to zone2
    pub desire_lines: Vec<(ZoneID, ZoneID, usize)>,
}

impl DemandModel {
    /// Turn all of the zones into Mercator. Don't do this when originally building and serializing
    /// them, because that process might not use exactly the same Mercator object.
    ///
    /// Also, calculate all the roads in each zone.
    pub fn finish_loading(&mut self, map: &MapModel) {
        self.cached_zone_roads = vec![vec![]; self.zones.len()];

        let mut prepared_zones = vec![];
        for zone in &mut self.zones {
            map.mercator.to_mercator_in_place(&mut zone.geometry);
            prepared_zones.push(PreparedGeometry::from(&zone.geometry));
        }

        for road in &map.roads {
            for (zone_idx, zone) in prepared_zones.iter().enumerate() {
                if zone.relate(&road.linestring).is_intersects() {
                    self.cached_zone_roads[zone_idx].push(road.id);
                }
            }
        }
    }

    pub fn make_requests(&self, fast_sample: bool) -> BTreeMap<(RoadID, RoadID), usize> {
        info!(
            "Making requests from {} zones and {} desire lines, sampling = {fast_sample}",
            self.zones.len(),
            self.desire_lines.len()
        );

        let mut rng = WyRand::new_seed(42);
        let mut requests = BTreeMap::new();

        fn choose(slice: &[RoadID], rng: &mut WyRand) -> Option<RoadID> {
            if slice.is_empty() {
                return None;
            }
            let idx = rng.generate_range(0..slice.len());
            Some(slice[idx])
        }

        // One sample will be made for this many trips when `fast_sample` is enabled.
        // Increasing this will speed up fast_sample mode, but give less accurate results.
        // Some OD pairs with less than this many trips might be skipped completely.
        let trips_per_sampled_request = 10;
        let mut accumulated_trip_count = 0;
        for (zone1, zone2, trip_count) in &self.desire_lines {
            accumulated_trip_count += *trip_count;

            let (request_count, request_weight) = if fast_sample {
                if accumulated_trip_count < trips_per_sampled_request {
                    continue;
                }
                let accumulated_requests = accumulated_trip_count / trips_per_sampled_request;
                accumulated_trip_count -= accumulated_requests * trips_per_sampled_request;
                (accumulated_requests, trips_per_sampled_request)
            } else {
                (*trip_count, 1)
            };

            for _ in 0..request_count {
                let Some(r1) = choose(&self.cached_zone_roads[zone1.0], &mut rng) else {
                    continue;
                };
                let Some(r2) = choose(&self.cached_zone_roads[zone2.0], &mut rng) else {
                    continue;
                };
                if r1 != r2 {
                    *requests.entry((r1, r2)).or_insert(0) += request_weight;
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
