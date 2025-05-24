use std::process::Command;

use anyhow::{bail, Result};
use argh::FromArgs;
use geo::{MultiPolygon, PreparedGeometry};
use serde::Deserialize;

use backend::MapModel;

mod travel_demand;

#[derive(FromArgs)]
/// Generate MapModel files with travel demand (OD) data
struct Args {
    /// path to study area boundaries.geojson, with a `kind` and `name` property
    #[argh(option)]
    study_area_boundaries: String,

    /// path to the directory where osm.pbf input files exist, in the form
    /// `{study_area.kind}_{study_area.name}.osm.pbf`
    #[argh(option)]
    osm_input_dir: String,

    /// path to a zones.geojson used for travel demand data, with a `name` property
    #[argh(option)]
    od_zones: String,

    /// path to an od.csv, with fields `zone1`, `zone2`, `count`, with `zone1` and `zone2` matching
    /// to `name` in `od_zones`
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
    let demand_models = travel_demand::BuildDemandModels::new(&args.od_zones, &args.od_csv)?;

    for study_area in study_areas {
        let demand = demand_models.build_for_area(&study_area);
        let context_data = None;
        let map = MapModel::create_serialized(
            &fs_err::read(format!(
                "{}/{}_{}.osm.pbf",
                args.osm_input_dir, study_area.kind, study_area.name
            ))?,
            study_area.geometry,
            Some(demand),
            context_data,
        )?;

        let path = format!(
            "{}/{}_{}.bin",
            args.out_dir, study_area.kind, study_area.name
        );
        println!("Writing {path}");
        fs_err::write(&path, bincode::serialize(&map)?)?;

        println!("Running: gzip {path}");
        if !Command::new("gzip").arg(&path).status()?.success() {
            bail!("`gzip {path}` failed");
        }
    }

    Ok(())
}

#[derive(Deserialize)]
struct StudyArea {
    #[serde(deserialize_with = "geojson::de::deserialize_geometry")]
    geometry: MultiPolygon,
    name: String,
    kind: String,
}

impl StudyArea {
    fn read_all_from_file(path: &str) -> Result<Vec<Self>> {
        let study_areas =
            geojson::de::deserialize_feature_collection_str_to_vec(&fs_err::read_to_string(path)?)?;
        println!("Read {} study area boundaries", study_areas.len());
        Ok(study_areas)
    }

    fn read_all_prepared_from_file(
        path: &str,
    ) -> Result<Vec<(PreparedGeometry<'static, MultiPolygon>, Self)>> {
        Ok(Self::read_all_from_file(path)?
            .into_iter()
            .map(|study_area| {
                (
                    PreparedGeometry::from(study_area.geometry.clone()),
                    study_area,
                )
            })
            .collect())
    }
}
