use crate::map_model::ProjectDetails;
use crate::neighbourhood::NeighbourhoodBoundary;
use crate::{MapModel, Neighbourhood};
use anyhow::{Context, Result};
use geojson::{Feature, FeatureCollection};
use std::io::{Cursor, Read};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct NeighbourhoodFixture {
    pub study_area_name: &'static str,
    pub neighbourhood_name: &'static str,
    pub savefile_name: &'static str,
    pub is_cnt: bool,
}

impl NeighbourhoodFixture {
    pub const BRISTOL_WEST: Self = Self {
        study_area_name: "bristol",
        neighbourhood_name: "west",
        savefile_name: "bristol_west",
        is_cnt: false,
    };
    pub const BRISTOL_EAST: Self = Self {
        study_area_name: "bristol",
        neighbourhood_name: "east",
        savefile_name: "bristol_east",
        is_cnt: false,
    };
    pub const STRASBOURG: Self = Self {
        study_area_name: "strasbourg",
        neighbourhood_name: "Schilik velorue",
        savefile_name: "strasbourg",
        is_cnt: false,
    };
    pub const DUNDEE: Self = Self {
        study_area_name: "LAD_Dundee City",
        neighbourhood_name: "Hilltown",
        savefile_name: "dundee",
        is_cnt: true,
    };
    pub const INVERNESS: Self = Self {
        study_area_name: "LAD_Highland",
        neighbourhood_name: "Longman",
        savefile_name: "inverness",
        is_cnt: true,
    };
}

// Currently, `db_schema_version` is only used by the frontend for import/export.
// To the backend it's opaque, so the value here currently has no consequence.
// In theory, we could one day do our schema migrations in rust.
pub const TEST_DB_SCHEMA_VERSION: u32 = 3;

impl NeighbourhoodFixture {
    pub fn map_model(&self) -> Result<MapModel> {
        self.map_model_builder()?()
    }

    pub fn neighbourhood_map(&self) -> Result<(Neighbourhood, MapModel)> {
        let (boundary, map) = self.neighbourhood_params()?;

        // Uncomment if you want to re-save the savefiles
        // std::fs::write(self.savefile_path(), map.to_savefile().to_string())?;
        let neighbourhood = Neighbourhood::new(&map, boundary)?;
        Ok((neighbourhood, map))
    }

    pub fn map_model_builder(&self) -> Result<Box<dyn Fn() -> Result<MapModel>>> {
        let project_details = ProjectDetails {
            project_name: self.savefile_name.to_string(),
            study_area_name: Some(self.study_area_name.to_string()),
            app_focus: if self.is_cnt {
                "cnt".to_string()
            } else {
                "global".to_string()
            },
            db_schema_version: TEST_DB_SCHEMA_VERSION,
        };

        // CNT files are pre-built with everything already
        if self.is_cnt {
            let path = format!("../web/public/cnt/maps_v3/{}.bin.gz", self.study_area_name);
            let input_bytes = std::fs::read(&path).context(format!("unable to read '{path}'"))?;
            let mut gunzipped = Vec::new();
            let mut decoder = flate2::read::GzDecoder::new(Cursor::new(input_bytes));
            decoder
                .read_to_end(&mut gunzipped)
                .expect("unable to gunzip pbf");

            return Ok(Box::new(move || {
                let mut map: MapModel = bincode::deserialize(&gunzipped)?;
                map.finish_loading(project_details.clone());
                Ok(map)
            }));
        }

        let pbf_path = format!("../web/public/severance_pbfs/v2/{}.osm.pbf", self.study_area_name);
        let pbf_bytes = std::fs::read(&pbf_path).context(format!("unable to read '{pbf_path}'"))?;

        let boundary_path = format!("../web/public/boundaries/{}.geojson", self.study_area_name);
        let boundary: Feature = std::fs::read_to_string(&boundary_path)?.parse()?;
        // Non-CNT boundaries are polygons
        let geometry: geo::Polygon = boundary
            .geometry
            .expect("missing geometry in test fixture")
            .try_into()?;
        let multi_polygon = geo::MultiPolygon(vec![geometry]);

        Ok(Box::new(move || {
            let demand = None;
            let context_data = None;
            let mut map = MapModel::create_serialized(
                &pbf_bytes,
                multi_polygon.clone(),
                demand,
                context_data,
            )?;
            map.finish_loading(project_details.clone());
            Ok(map)
        }))
    }

    pub fn bench_sample_size(&self) -> usize {
        if self.study_area_name == Self::STRASBOURG.study_area_name
            || self.study_area_name == Self::INVERNESS.study_area_name
        {
            // Some study areas are big and slow, so we bench them fewer times
            10
        } else {
            100
        }
    }

    pub fn savefile(&self) -> Result<FeatureCollection> {
        let savefile: FeatureCollection = std::fs::read_to_string(self.savefile_path())?.parse()?;
        Ok(savefile)
    }

    fn savefile_path(&self) -> String {
        format!(
            "../tests/{savefile_name}.geojson",
            savefile_name = self.savefile_name
        )
    }

    pub fn neighbourhood_params(&self) -> Result<(NeighbourhoodBoundary, MapModel)> {
        let mut map = self.map_model()?;
        map.load_savefile(self.savefile()?)?;
        Ok((map.boundaries[self.neighbourhood_name].clone(), map))
    }
}
