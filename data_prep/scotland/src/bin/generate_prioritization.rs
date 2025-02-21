use anyhow::Result;
use backend::boundary_stats::PopulationZone;
use data_prep::StudyArea;
use geo::{MultiPolygon, PreparedGeometry, Relate};
use serde::Deserialize;
use std::time::Instant;

fn main() -> Result<()> {
    println!("Generating prioritization code");
    let start = Instant::now();

    let study_areas = StudyArea::read_all_prepared_from_file()?;
    println!("Time since start {:?}", start.elapsed());
    let population_zone_inputs = PopulationZoneInput::read_all_prepared_from_file()?;
    println!("Time since start {:?}", start.elapsed());

    for study_area in &study_areas {
        let study_area_start = Instant::now();
        println!("Starting {}", study_area.1.name);
        let mut study_area_population_zones = vec![];
        for population_zone_input in &population_zone_inputs {
            if study_area
                .0
                .relate(&population_zone_input.0)
                .is_intersects()
            {
                let population_zone = PopulationZone {
                    geometry: population_zone_input.1.geometry.clone(),
                    imd_percentile: population_zone_input.1.imd_percentile,
                    population: population_zone_input.1.population,
                };
                study_area_population_zones.push(population_zone);
            }
        }

        std::fs::create_dir_all("prioritization")?;
        let path = format!(
            "prioritization/population_{}_{}.bin",
            study_area.1.kind, study_area.1.name
        );
        std::fs::write(&path, bincode::serialize(&study_area_population_zones)?)?;
        println!(
            "Wrote {} population zones to {} (took {:?})",
            study_area_population_zones.len(),
            path,
            study_area_start.elapsed()
        );
    }

    println!("Time since start {:?}", start.elapsed());
    Ok(())
}

#[derive(Clone, Debug, Deserialize)]
pub struct PopulationZoneInput {
    #[serde(deserialize_with = "geojson::de::deserialize_geometry")]
    pub geometry: MultiPolygon,

    // "id": "S01006506",
    // (unused)

    // "imd_rank": 4691,
    // (unused)

    // "imd_percentile": 68,
    pub imd_percentile: u8,

    // "population": 894,
    pub population: u32,
    // "area": 4388802.1221970674
    // (unused - though maybe we would find it helpful for pre-computing density or to save the cost of calculating area live)
}

impl PopulationZoneInput {
    fn read_all_from_file() -> Result<Vec<Self>> {
        let population_zones = geojson::de::deserialize_feature_collection_str_to_vec(
            // REVIEW: I think it's OK to just explode with useful error messages for the data_prep stuff, right?
            &std::fs::read_to_string("tmp/population.geojson")
                .expect("missing population.geojson - see get_reference_layers.sh"),
        )?;
        println!("Read {} population zones", population_zones.len());
        Ok(population_zones)
    }

    fn read_all_prepared_from_file() -> Result<Vec<(PreparedGeometry<'static>, Self)>> {
        let iter = Self::read_all_from_file()?
            .into_iter()
            .map(|population_zone| {
                (
                    PreparedGeometry::from(population_zone.geometry.clone()),
                    population_zone,
                )
            });
        Ok(iter.collect())
    }
}
