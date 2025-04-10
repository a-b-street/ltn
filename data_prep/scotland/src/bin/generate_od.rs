use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use geo::{Intersects, MultiPolygon};
use serde::Deserialize;
use utils::Mercator;

use backend::od::{DemandModel, ZoneID};
use data_prep::StudyArea;

/// Generate travel demand (OD) model, and write to file
fn main() -> Result<()> {
    let study_areas = StudyArea::read_all_from_file()?;

    let input_zones: Vec<Zone> = geojson::de::deserialize_feature_collection_str_to_vec(
        &std::fs::read_to_string("zones.geojson")?,
    )?;
    let zones: BTreeMap<String, Zone> = input_zones
        .into_iter()
        .map(|zone| (zone.name.clone(), zone))
        .collect();
    println!("Read {} zones", zones.len());

    let desire_lines = read_desire_lines("od.csv")?;
    println!("Read {} desire lines", desire_lines.len());

    for study_area in study_areas {
        let subset_zones = find_matching_zones(study_area.geometry, &zones);

        let mut subset_desire_lines = Vec::new();
        for (zone1, zone2, count) in &desire_lines {
            if let (Some(from), Some(to)) = (subset_zones.get(zone1), subset_zones.get(zone2)) {
                subset_desire_lines.push((*from, *to, *count));
            }
        }
        let demand = DemandModel {
            zones: subset_zones
                .into_keys()
                .map(|name| backend::od::Zone {
                    name: name.clone(),
                    geometry: zones[&name].geometry.clone(),
                })
                .collect(),
            desire_lines: subset_desire_lines,
            cached_zone_roads: vec![],
        };
        let path = format!("demand/demand_{}_{}.bin", study_area.kind, study_area.name);
        println!(
            "Writing {path} with {} matching zones and {} desire lines",
            demand.zones.len(),
            demand.desire_lines.len()
        );
        std::fs::write(path, bincode::serialize(&demand)?)?;
    }

    Ok(())
}

#[derive(Deserialize)]
struct Zone {
    #[serde(deserialize_with = "geojson::de::deserialize_geometry")]
    geometry: MultiPolygon,
    name: String,
}

fn read_desire_lines(path: &str) -> Result<Vec<(String, String, usize)>> {
    let mut out = Vec::new();
    for rec in csv::Reader::from_reader(std::fs::File::open(path)?).deserialize() {
        let row: DesireLineRow = rec?;
        out.push((
            row.geo_code1,
            row.geo_code2,
            row.car_driver + row.car_passenger,
        ));
    }
    Ok(out)
}

#[derive(Deserialize)]
struct DesireLineRow {
    geo_code1: String,
    geo_code2: String,
    car_driver: usize,
    car_passenger: usize,
}

/// Returns a mapping from original zone name to sequential IDs
fn find_matching_zones(
    boundary_wgs84: MultiPolygon,
    zones: &BTreeMap<String, Zone>,
) -> BTreeMap<String, ZoneID> {
    let mut matches = BTreeSet::new();
    let mercator = Mercator::from(boundary_wgs84.clone()).unwrap();
    let boundary_mercator = mercator.to_mercator(&boundary_wgs84);

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
