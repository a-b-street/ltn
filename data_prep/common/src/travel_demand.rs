use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use geo::{Intersects, MultiPolygon};
use serde::Deserialize;
use utils::Mercator;

use crate::StudyArea;
use backend::od::{DemandModel, ZoneID};

pub struct BuildDemandModels {
    zones: BTreeMap<String, Zone>,
    desire_lines: Vec<DesireLineRow>,
}

impl BuildDemandModels {
    pub fn new(od_zones_path: &str, od_csv_path: &str) -> Result<Self> {
        let input_zones: Vec<Zone> = geojson::de::deserialize_feature_collection_str_to_vec(
            &fs_err::read_to_string(od_zones_path)?,
        )?;
        let zones: BTreeMap<String, Zone> = input_zones
            .into_iter()
            .map(|zone| (zone.name.clone(), zone))
            .collect();
        println!("Read {} zones", zones.len());

        let mut desire_lines = Vec::new();
        for rec in csv::Reader::from_reader(fs_err::File::open(od_csv_path)?).deserialize() {
            let row: DesireLineRow = rec?;
            desire_lines.push(row);
        }
        println!("Read {} desire lines", desire_lines.len());

        Ok(Self {
            zones,
            desire_lines,
        })
    }

    pub fn build_for_area(&self, study_area: &StudyArea) -> DemandModel {
        let subset_zones = find_matching_zones(&study_area.geometry, &self.zones);

        let mut subset_desire_lines = Vec::new();
        for row in &self.desire_lines {
            if let (Some(from), Some(to)) =
                (subset_zones.get(&row.zone1), subset_zones.get(&row.zone2))
            {
                subset_desire_lines.push((*from, *to, row.count));
            }
        }
        DemandModel {
            zones: subset_zones
                .into_keys()
                .map(|name| backend::od::Zone {
                    name: name.clone(),
                    geometry: self.zones[&name].geometry.clone(),
                })
                .collect(),
            desire_lines: subset_desire_lines,
            cached_zone_roads: vec![],
        }
    }
}

#[derive(Deserialize)]
struct Zone {
    #[serde(deserialize_with = "geojson::de::deserialize_geometry")]
    geometry: MultiPolygon,
    name: String,
}

#[derive(Deserialize)]
struct DesireLineRow {
    zone1: String,
    zone2: String,
    count: usize,
}

/// Returns a mapping from original zone name to sequential IDs
fn find_matching_zones(
    boundary_wgs84: &MultiPolygon,
    zones: &BTreeMap<String, Zone>,
) -> BTreeMap<String, ZoneID> {
    let mut matches = BTreeSet::new();
    let mercator = Mercator::from(boundary_wgs84.clone()).unwrap();
    let boundary_mercator = mercator.to_mercator(boundary_wgs84);

    for zone in zones.values() {
        let zone_mercator = mercator.to_mercator(&zone.geometry);
        if boundary_mercator.intersects(&zone_mercator) {
            matches.insert(zone.name.clone());
        }
    }

    matches
        .into_iter()
        .enumerate()
        .map(|(idx, name)| (name, ZoneID(idx)))
        .collect()
}
