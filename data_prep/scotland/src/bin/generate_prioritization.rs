use anyhow::{Context, Result};
use backend::boundary_stats::{ContextData, MetricBuckets, POIKind, PopulationZone, POI};
use data_prep::{PopulationZoneInput, StudyArea};
use geo::{MultiPolygon, Point, PreparedGeometry, Relate};
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Instant;

fn main() -> Result<()> {
    println!("Generating prioritization code");
    let start = Instant::now();

    let car_ownership_data_zones = CarOwnershipDataZone::read_all_from_file_as_map()?;

    let study_areas = StudyArea::read_all_prepared_from_file()?;
    println!("Time since start {:?}", start.elapsed());
    let population_zone_inputs = PopulationZoneInput::read_all_from_file()?;
    println!("Time since start {:?}", start.elapsed());
    let stats19_collisions = Stats19Input::read_all_from_file()?;
    let pois = InputPOI::read_all_from_files()?;

    let mut all_settlements = SettlementsInput::read_all_from_file()?;

    for (idx, study_area) in study_areas.iter().enumerate() {
        let study_area_start = Instant::now();
        println!(
            "Starting {} ({} / {})",
            study_area.1.name,
            idx + 1,
            study_areas.len()
        );
        let mut context_data = ContextData {
            settlements: all_settlements.remove(&study_area.1.name).unwrap().geometry,
            population_zones: Vec::new(),
            stats19_collisions: Vec::new(),
            pois: Vec::new(),
            metric_buckets: MetricBuckets::default(),
        };

        // Store the area per population zone temporarily, to calculate buckets
        let mut population_zone_area_km2 = Vec::new();
        for population_zone_input in &population_zone_inputs {
            if study_area
                .0
                .relate(&population_zone_input.geometry)
                .is_intersects()
            {
                let car_ownership = &car_ownership_data_zones[&population_zone_input.id];
                context_data.population_zones.push(PopulationZone {
                    geometry: population_zone_input.geometry.geometry().clone(),
                    imd_percentile: population_zone_input.imd_percentile,
                    population: population_zone_input.population,
                    total_households: car_ownership.total_households,
                    households_with_cars_or_vans: car_ownership.households_with_cars_or_vans(),
                });

                population_zone_area_km2.push(population_zone_input.area / 1000.0 / 1000.0);
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

        calculate_metric_buckets(&mut context_data, population_zone_area_km2)?;
        // TODO do not commit
        //println!("for {}, got {:?}", study_area.1.name, context_data.metric_buckets.population_density);

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

    println!("All done!");
    println!("Now run:\n  cp prioritization/* ../../web/public/cnt_prioritization");

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
            ("out_layers/recreation.geojson", POIKind::Recreation),
        ] {
            let input: Vec<InputPOI> = geojson::de::deserialize_feature_collection_str_to_vec(
                &fs_err::read_to_string(path).context(format!("opening {path}"))?,
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

/// The number of households with certain numbers of cars, bucketed by datazone
///
/// CSV data rows looks like:
/// "S01006506",491,79,246,130,22,14,
///
/// Unfortunately there are some weird front-matter/back-matter in the csv, you'll have to remove it manually. See `get_reference_layers.sh` for instructions
///
/// Header row itself has a weird first column:
/// "Number of cars or vans","All occupied households","Number of cars or vans in household: No cars or vans","Number of cars or vans in household: One car or van","Number of cars or vans in household: Two cars or vans","Number of cars or vans in household: Three cars or vans","Number of cars or vans in household: Four or more cars or vans",
///
/// In particular, the first column is a datazone id, not "Number of cars or vans". Looking at this data in the census WebUI, I think "number of cars or vans" is the title of the table. It is not, as far as I can tell, a column of the total number of cars.
#[derive(Deserialize, Debug)]
struct CarOwnershipDataZone {
    /// Census data zone
    #[serde(rename = "Number of cars or vans")]
    data_zone_id: String,

    /// Total number of households
    #[serde(rename = "All occupied households")]
    total_households: u32,

    /// Households with no cars/vans
    #[serde(rename = "Number of cars or vans in household: No cars or vans")]
    cars_0: u32,

    /// Households with 1 car/van
    #[serde(rename = "Number of cars or vans in household: One car or van")]
    cars_1: u32,

    /// Households with 2 cars/vans
    #[serde(rename = "Number of cars or vans in household: Two cars or vans")]
    cars_2: u32,

    /// Households with 3 cars/vans
    #[serde(rename = "Number of cars or vans in household: Three cars or vans")]
    cars_3: u32,

    /// Households with 4 or more cars/vans
    #[serde(rename = "Number of cars or vans in household: Four or more cars or vans")]
    cars_4_or_more: u32,
}

impl CarOwnershipDataZone {
    fn read_all_from_file() -> Result<Vec<Self>> {
        let input_path = "input/car_ownership_data_zones.csv";
        let mut output = vec![];
        for rec in csv::Reader::from_path(input_path)
            .context(format!("opening {input_path}"))?
            .deserialize()
        {
            let rec: Self = rec?;
            debug_assert_eq!(
                rec.total_households,
                rec.cars_0 + rec.cars_1 + rec.cars_2 + rec.cars_3 + rec.cars_4_or_more,
                "data inconsistency: total_households is not equal to the sum of categories"
            );
            output.push(rec);
        }
        Ok(output)
    }
    /// data_zone_id -> CarOwnershipDataZone
    fn read_all_from_file_as_map() -> Result<HashMap<String, Self>> {
        let mut map = HashMap::new();
        for record in Self::read_all_from_file()?.into_iter() {
            map.insert(record.data_zone_id.clone(), record);
        }
        Ok(map)
    }

    fn households_with_cars_or_vans(&self) -> u32 {
        self.total_households - self.cars_0
    }
}

#[derive(Deserialize)]
struct SettlementsInput {
    #[serde(deserialize_with = "geojson::de::deserialize_geometry")]
    geometry: MultiPolygon,
    #[serde(rename = "name")]
    study_area_name: String,
}

impl SettlementsInput {
    fn read_all_from_file() -> Result<HashMap<String, Self>> {
        let path = "tmp/lad_settlements.geojson";
        let input: Vec<SettlementsInput> =
            geojson::de::deserialize_feature_collection_str_to_vec(&fs_err::read_to_string(path)?)?;
        Ok(input
            .into_iter()
            .map(|s| (s.study_area_name.clone(), s))
            .collect())
    }
}

fn calculate_metric_buckets(
    data: &mut ContextData,
    population_zone_area_km2: Vec<f64>,
) -> Result<()> {
    data.metric_buckets.population_density = make_buckets(
        &data
            .population_zones
            .iter()
            .enumerate()
            .map(|(idx, zone)| zone.population as f64 / population_zone_area_km2[idx])
            .collect(),
    )?;

    let population_zones = data
        .population_zones
        .iter()
        .map(|zone| PreparedGeometry::from(&zone.geometry))
        .collect::<Vec<_>>();

    let mut collions_per_zone = Vec::new();
    for zone in &population_zones {
        let mut count = 0;
        for pt in &data.stats19_collisions {
            if zone.relate(pt).is_contains() {
                count += 1;
            }
        }
        collions_per_zone.push(count);
    }
    data.metric_buckets.collision_density = make_buckets(
        &collions_per_zone
            .into_iter()
            .enumerate()
            .map(|(idx, collisions)| collisions as f64 / population_zone_area_km2[idx])
            .collect(),
    )?;

    let mut pois_per_zone = Vec::new();
    for zone in &population_zones {
        let mut count = 0;
        for poi in &data.pois {
            if zone.relate(&poi.point).is_contains() {
                count += 1;
            }
        }
        pois_per_zone.push(count);
    }
    data.metric_buckets.poi_density = make_buckets(
        &pois_per_zone
            .into_iter()
            .enumerate()
            .map(|(idx, pois)| pois as f64 / population_zone_area_km2[idx])
            .collect(),
    )?;

    Ok(())
}

// Use ckmeans to find the upper limit of 5 buckets
fn make_buckets(values: &Vec<f64>) -> Result<[usize; 6]> {
    let breaks = ckmeans::roundbreaks(&values, 6)?;
    assert_eq!(breaks.len(), 5);
    Ok([
        // Always start with a leading 0, for convenience on the maplibre end
        0,
        breaks[0] as usize,
        breaks[1] as usize,
        breaks[2] as usize,
        breaks[3] as usize,
        breaks[4] as usize,
    ])
}
