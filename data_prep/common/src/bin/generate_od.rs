use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;

use anyhow::{bail, Result};
use argh::FromArgs;
use geo::{Intersects, MultiPolygon};
use serde::Deserialize;
use utils::Mercator;

use backend::od::{DemandModel, ZoneID};
use common_data_prep::StudyArea;

#[derive(FromArgs)]
/// Generate travel demand (OD) model
struct Args {
    /// path to study area boundaries.geojson, with a `kind` and `name` property
    #[argh(option)]
    study_area_boundaries: String,

    /// path to a zones.geojson, with a `name` property
    #[argh(option)]
    od_zones: String,

    /// path to an od.csv, with fields `zone1`, `zone2`, `count`
    #[argh(option)]
    od_csv: String,

    /// path to the directory where gzipped output will be written, as
    /// `{study_area.kind}_{study_area.name}.bin.gz`
    #[argh(option)]
    out_dir: String,
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();

    let study_areas = StudyArea::read_all_from_file(&args.study_area_boundaries)?;

    let input_zones: Vec<Zone> = geojson::de::deserialize_feature_collection_str_to_vec(
        &fs_err::read_to_string(&args.od_zones)?,
    )?;
    let zones: BTreeMap<String, Zone> = input_zones
        .into_iter()
        .map(|zone| (zone.name.clone(), zone))
        .collect();
    println!("Read {} zones", zones.len());

    let desire_lines = read_desire_lines(&args.od_csv)?;
    println!("Read {} desire lines", desire_lines.len());

    for study_area in study_areas {
        let subset_zones = find_matching_zones(study_area.geometry, &zones);

        let mut subset_desire_lines = Vec::new();
        for row in &desire_lines {
            if let (Some(from), Some(to)) =
                (subset_zones.get(&row.zone1), subset_zones.get(&row.zone2))
            {
                subset_desire_lines.push((*from, *to, row.count));
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
        let path = format!(
            "{}/{}_{}.bin",
            args.out_dir, study_area.kind, study_area.name
        );
        println!(
            "Writing {path} with {} matching zones and {} desire lines",
            demand.zones.len(),
            demand.desire_lines.len()
        );
        fs_err::write(&path, bincode::serialize(&demand)?)?;

        println!("Running: gzip {path}");
        if !Command::new("gzip").arg(&path).status()?.success() {
            bail!("`gzip {path}` failed");
        }
    }

    Ok(())
}

#[derive(Deserialize)]
struct Zone {
    #[serde(deserialize_with = "geojson::de::deserialize_geometry")]
    geometry: MultiPolygon,
    name: String,
}

fn read_desire_lines(path: &str) -> Result<Vec<DesireLineRow>> {
    let mut out = Vec::new();
    for rec in csv::Reader::from_reader(fs_err::File::open(path)?).deserialize() {
        let row: DesireLineRow = rec?;
        out.push(row);
    }
    Ok(out)
}

#[derive(Deserialize)]
struct DesireLineRow {
    zone1: String,
    zone2: String,
    count: usize,
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
