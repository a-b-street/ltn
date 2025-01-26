use anyhow::Result;
use geo::MultiPolygon;
use serde::Deserialize;

use backend::od::Zone;

fn main() -> Result<()> {
    let study_areas: Vec<StudyArea> = geojson::de::deserialize_feature_collection_str_to_vec(
        &std::fs::read_to_string("boundaries.geojson")?,
    )?;
    println!("Read {} study area boundaries", study_areas.len());

    let zones: Vec<Zone> = geojson::de::deserialize_feature_collection_str_to_vec(
        &std::fs::read_to_string("zones.geojson")?,
    )?;
    println!("Read {} zones", zones.len());

    let desire_lines = read_desire_lines("od.csv")?;
    println!("Read {} desire lines", desire_lines.len());

    Ok(())
}

#[derive(Deserialize)]
struct StudyArea {
    #[serde(deserialize_with = "geojson::de::deserialize_geometry")]
    geometry: MultiPolygon,
    name: String,
    kind: String,
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
