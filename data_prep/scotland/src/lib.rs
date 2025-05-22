use anyhow::Result;
use geo::{MultiPolygon, PreparedGeometry};
use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
pub struct PopulationZoneInput {
    #[serde(deserialize_with = "deserialize_prepared_multipolygon")]
    pub geometry: PreparedGeometry<'static, MultiPolygon>,

    // "id": "S01006506",
    pub id: String,

    // "imd_rank": 4691,
    // (unused)

    // "imd_percentile": 68,
    pub imd_percentile: u8,

    // "population": 894,
    pub population: u32,

    // "area": 4388802.1221970674
    pub area: f64,
}

pub fn deserialize_prepared_multipolygon<'de, D>(
    deserializer: D,
) -> std::result::Result<PreparedGeometry<'static, MultiPolygon>, D::Error>
where
    D: Deserializer<'de>,
{
    let multi_polygon: MultiPolygon = geojson::de::deserialize_geometry(deserializer)?;
    Ok(PreparedGeometry::from(multi_polygon))
}

impl PopulationZoneInput {
    pub fn read_all_from_file() -> Result<Vec<Self>> {
        let population_zones = geojson::de::deserialize_feature_collection_str_to_vec(
            &fs_err::read_to_string("tmp/population.geojson")?,
        )?;
        println!("Read {} population zones", population_zones.len());
        Ok(population_zones)
    }

    pub fn area_km2(&self) -> f64 {
        self.area / 1000.0 / 1000.0
    }
}
