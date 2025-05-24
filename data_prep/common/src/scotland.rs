use crate::StudyArea;
use anyhow::{Context, Result};
use backend::boundary_stats::{ContextData, MetricBuckets, POIKind, PopulationZone, POI};
use geo::{MultiPolygon, Point, PreparedGeometry, Relate};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::time::Instant;

pub struct BuildContextData {
    car_ownership_data_zones: HashMap<String, CarOwnershipDataZone>,
    population_zone_inputs: Vec<PopulationZoneInput>,
    stats19_collisions_input: Vec<Stats19Input>,
    pois_input: Vec<POI>,
    all_settlements: HashMap<String, SettlementsInput>,
}

impl BuildContextData {
    pub fn new() -> Result<Self> {
        let start = Instant::now();
        let car_ownership_data_zones = CarOwnershipDataZone::read_all_from_file_as_map()?;

        println!("Time since start {:?}", start.elapsed());
        let population_zone_inputs = PopulationZoneInput::read_all_from_file()?;
        println!("Time since start {:?}", start.elapsed());
        let stats19_collisions_input = Stats19Input::read_all_from_file()?;
        let pois_input = InputPOI::read_all_from_files()?;

        let all_settlements = SettlementsInput::read_all_from_file()?;

        Ok(Self {
            car_ownership_data_zones,
            population_zone_inputs,
            stats19_collisions_input,
            pois_input,
            all_settlements,
        })
    }

    pub fn build_for_area(&self, study_area: &StudyArea) -> Result<ContextData> {
        let prepared_study_area = PreparedGeometry::from(study_area.geometry.clone());

        let settlements = self
            .all_settlements
            .get(&study_area.name)
            .unwrap()
            .geometry
            .clone();
        let mut population_zones = Vec::new();
        let mut stats19_collisions = Vec::new();
        let mut pois = Vec::new();

        // Store the area per population zone temporarily, to calculate buckets
        for population_zone_input in &self.population_zone_inputs {
            if prepared_study_area
                .relate(&population_zone_input.geometry)
                .is_intersects()
            {
                let car_ownership = &self.car_ownership_data_zones[&population_zone_input.id];
                population_zones.push(PopulationZone {
                    geometry: population_zone_input.geometry.geometry().clone(),
                    imd_percentile: population_zone_input.imd_percentile,
                    population: population_zone_input.population,
                    total_households: car_ownership.total_households,
                    households_with_cars_or_vans: car_ownership.households_with_cars_or_vans(),
                });
            }
        }

        for pt in &self.stats19_collisions_input {
            if prepared_study_area.relate(&pt.geometry).is_contains() {
                stats19_collisions.push(pt.geometry);
            }
        }

        for poi in &self.pois_input {
            if prepared_study_area.relate(&poi.point).is_contains() {
                pois.push(poi.clone());
            }
        }

        let metric_buckets =
            calculate_metric_buckets(&self.population_zone_inputs, &stats19_collisions, &pois)?;
        Ok(ContextData {
            settlements,
            population_zones,
            stats19_collisions,
            pois,
            metric_buckets,
        })
    }
}

#[derive(Deserialize)]
struct PopulationZoneInput {
    #[serde(deserialize_with = "deserialize_prepared_multipolygon")]
    geometry: PreparedGeometry<'static, MultiPolygon>,

    // "id": "S01006506",
    id: String,

    // "imd_rank": 4691,
    // (unused)

    // "imd_percentile": 68,
    imd_percentile: u8,

    // "population": 894,
    population: u32,

    // "area": 4388802.1221970674
    area: f64,
}

fn deserialize_prepared_multipolygon<'de, D>(
    deserializer: D,
) -> std::result::Result<PreparedGeometry<'static, MultiPolygon>, D::Error>
where
    D: Deserializer<'de>,
{
    let multi_polygon: MultiPolygon = geojson::de::deserialize_geometry(deserializer)?;
    Ok(PreparedGeometry::from(multi_polygon))
}

impl PopulationZoneInput {
    fn read_all_from_file() -> Result<Vec<Self>> {
        let population_zones = geojson::de::deserialize_feature_collection_str_to_vec(
            &fs_err::read_to_string("tmp/population.geojson")?,
        )?;
        println!("Read {} population zones", population_zones.len());
        Ok(population_zones)
    }

    fn area_km2(&self) -> f64 {
        self.area / 1000.0 / 1000.0
    }
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
    geometry: Point,
    name: Option<String>,
}

impl InputPOI {
    fn read_all_from_files() -> Result<Vec<POI>> {
        let mut pois = Vec::new();
        for (path, kind) in [
            ("tmp/gp_practices.geojson", POIKind::GP),
            ("tmp/hospitals.geojson", POIKind::Hospital),
            ("tmp/schools.geojson", POIKind::School),
            (
                "../../web/public/cnt/layers/recreation.geojson",
                POIKind::Recreation,
            ),
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
    population_zone_inputs: &[PopulationZoneInput],
    stats19_collisions: &[Point],
    pois: &[POI],
) -> Result<MetricBuckets> {
    let population_density = make_buckets(
        &population_zone_inputs
            .iter()
            .map(|zone| zone.population as f64 / zone.area_km2())
            .collect(),
    )?;

    let mut collisions_per_zone = Vec::new();
    for zone in population_zone_inputs {
        let mut count = 0;
        for pt in stats19_collisions {
            if zone.geometry.relate(pt).is_contains() {
                count += 1;
            }
        }
        collisions_per_zone.push(count);
    }

    let collision_density = make_buckets(
        &collisions_per_zone
            .into_iter()
            .zip(population_zone_inputs.iter())
            .map(|(collisions, population_zone)| collisions as f64 / population_zone.area_km2())
            .collect(),
    )?;

    let mut pois_per_zone = Vec::new();
    for zone in population_zone_inputs {
        let mut count = 0;
        for poi in pois {
            if zone.geometry.relate(&poi.point).is_contains() {
                count += 1;
            }
        }
        pois_per_zone.push(count);
    }

    let poi_density = make_buckets(
        &pois_per_zone
            .into_iter()
            .zip(population_zone_inputs.iter())
            .map(|(pois, population_zone)| pois as f64 / population_zone.area_km2())
            .collect(),
    )?;

    Ok(MetricBuckets {
        population_density,
        collision_density,
        poi_density,
    })
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
