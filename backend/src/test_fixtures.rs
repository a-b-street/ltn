use crate::boundary_stats::ContextData;
use crate::map_model::ProjectDetails;
use crate::neighbourhood::NeighbourhoodBoundary;
use crate::od::DemandModel;
use crate::{MapModel, Neighbourhood};
use anyhow::{Context, Result};
use geojson::{Feature, FeatureCollection};

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

    fn pbf_path(&self) -> String {
        if self.is_cnt {
            format!("../web/public/cnt_osm/{}.osm.pbf", self.study_area_name)
        } else {
            format!("../web/public/severance_pbfs/{}.pbf", self.study_area_name)
        }
    }

    fn boundary_path(&self) -> String {
        if self.is_cnt {
            format!(
                "../web/public/cnt_boundaries/{}.geojson",
                self.study_area_name
            )
        } else {
            format!("../web/public/boundaries/{}.geojson", self.study_area_name)
        }
    }

    fn context_data_path(&self) -> Option<String> {
        if self.is_cnt {
            Some(format!(
                "../web/public/cnt_prioritization/context_{}.bin",
                self.study_area_name
            ))
        } else {
            None
        }
    }

    fn context_data(&self) -> Option<ContextData> {
        let path = self.context_data_path()?;
        let context_data_bytes =
            std::fs::read(&path).expect(&format!("unable to read context_data: {path}"));
        Some(bincode::deserialize(&context_data_bytes).expect("unable to deserialize context_data"))
    }

    fn demand_data(&self) -> Option<DemandModel> {
        if !self.is_cnt {
            return None;
        }
        let path = format!(
            "../web/public/cnt_demand/demand_{}.bin",
            self.study_area_name
        );
        let bytes = std::fs::read(&path).expect(&format!("unable to read demand_data: {path}"));
        Some(bincode::deserialize(&bytes).expect("unable to deserialize demand_data"))
    }

    pub fn map_model_builder(&self) -> Result<impl Fn() -> Result<MapModel> + use<'_>> {
        let study_area_name = &self.study_area_name;

        let input_bytes = std::fs::read(&self.pbf_path())
            .context(format!("unable to read '{}'", self.pbf_path()))?;

        let boundary: Feature = std::fs::read_to_string(&self.boundary_path())?.parse()?;
        let geometry: geo::Geometry = boundary
            .geometry
            .expect("missing geometry in test fixture")
            .try_into()?;
        let multi_polygon = match geometry {
            // CNT boundaries are MultiPolygons.
            geo::Geometry::MultiPolygon(g) => g,
            // Historically other boundaries were polygons.
            geo::Geometry::Polygon(single_polygon) => single_polygon.into(),
            other => bail!("unexpected geometry type {other:?}"),
        };
        Ok(move || {
            let demand = self.demand_data();
            let context_data = self.context_data();
            let project_details = ProjectDetails {
                project_name: self.savefile_name.to_string(),
                study_area_name: Some(study_area_name.to_string()),
                app_focus: "global".to_string(),
                db_schema_version: TEST_DB_SCHEMA_VERSION,
            };
            MapModel::new(
                &input_bytes,
                multi_polygon.clone(),
                project_details,
                demand,
                context_data,
            )
        })
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
