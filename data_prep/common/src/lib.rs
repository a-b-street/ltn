use anyhow::Result;
use geo::{MultiPolygon, PreparedGeometry};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct StudyArea {
    #[serde(deserialize_with = "geojson::de::deserialize_geometry")]
    pub geometry: MultiPolygon,
    pub name: String,
    pub kind: String,
}

impl StudyArea {
    pub fn read_all_from_file(path: &str) -> Result<Vec<Self>> {
        let study_areas =
            geojson::de::deserialize_feature_collection_str_to_vec(&fs_err::read_to_string(path)?)?;
        println!("Read {} study area boundaries", study_areas.len());
        Ok(study_areas)
    }

    pub fn read_all_prepared_from_file(
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
