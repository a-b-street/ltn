use anyhow::Result;
use backend::boundary_stats::{ContextData, POIKind, PopulationZone, POI};
use data_prep::{PopulationZoneInput, StudyArea};
use geo::{Point, Relate};
use serde::Deserialize;
use std::time::Instant;

fn main() -> Result<()> {
    println!("Generating prioritization code");
    let start = Instant::now();

    let study_areas = StudyArea::read_all_prepared_from_file()?;
    println!("Time since start {:?}", start.elapsed());
    let population_zone_inputs = PopulationZoneInput::read_all_from_file()?;
    println!("Time since start {:?}", start.elapsed());
    let stats19_collisions = Stats19Input::read_all_from_file()?;
    let pois = InputPOI::read_all_from_files()?;

    for study_area in &study_areas {
        let study_area_start = Instant::now();
        println!("Starting {}", study_area.1.name);
        let mut context_data = ContextData {
            population_zones: Vec::new(),
            stats19_collisions: Vec::new(),
            pois: Vec::new(),
        };

        for population_zone_input in &population_zone_inputs {
            if study_area
                .0
                .relate(&population_zone_input.geometry)
                .is_intersects()
            {
                context_data.population_zones.push(PopulationZone {
                    geometry: population_zone_input.geometry.geometry().clone(),
                    imd_percentile: population_zone_input.imd_percentile,
                    population: population_zone_input.population,
                });
            }
        }

        for pt in &stats19_collisions {
            if study_area.0.relate(&pt.geometry).is_contains() {
                context_data.stats19_collisions.push(pt.geometry);
            }
        }

        for poi in &pois {
            if study_area.0.relate(&poi.point).is_contains() {
                context_data.pois.push(poi.clone());
            }
        }

        std::fs::create_dir_all("prioritization")?;
        let path = format!(
            "prioritization/context_{}_{}.bin",
            study_area.1.kind, study_area.1.name
        );
        std::fs::write(&path, bincode::serialize(&context_data)?)?;
        println!(
            "Wrote {} population zones to {} (took {:?})",
            context_data.population_zones.len(),
            path,
            study_area_start.elapsed()
        );
    }

    println!("Time since start {:?}", start.elapsed());
    Ok(())
}

// Ignore all properties
#[derive(Deserialize)]
struct Stats19Input {
    #[serde(deserialize_with = "geojson::de::deserialize_geometry")]
    geometry: Point,
}

impl Stats19Input {
    fn read_all_from_file() -> Result<Vec<Self>> {
        Ok(geojson::de::deserialize_feature_collection_str_to_vec(
            &fs_err::read_to_string("atip-data-prep/stats19/tmp/stats19_clipped.geojson")?,
        )?)
    }
}

#[derive(Deserialize)]
struct InputPOI {
    #[serde(deserialize_with = "geojson::de::deserialize_geometry")]
    pub geometry: Point,
    pub name: Option<String>,
}

impl InputPOI {
    fn read_all_from_files() -> Result<Vec<POI>> {
        let mut pois = Vec::new();
        for (path, kind) in [
            ("tmp/gp_practices.geojson", POIKind::GP),
            ("tmp/hospitals.geojson", POIKind::Hospital),
            ("tmp/schools.geojson", POIKind::School),
        ] {
            let input: Vec<InputPOI> = geojson::de::deserialize_feature_collection_str_to_vec(
                &fs_err::read_to_string(path)?,
            )?;
            for poi in input {
                pois.push(POI {
                    kind,
                    point: poi.geometry,
                    name: poi.name,
                });
            }
        }
        Ok(pois)
    }
}
