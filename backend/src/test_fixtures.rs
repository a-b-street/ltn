use crate::neighbourhood::NeighbourhoodBoundary;
use crate::{MapModel, Neighbourhood};
use anyhow::Result;
use geo::{MultiPolygon, Polygon};
use geojson::{Feature, FeatureCollection};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct NeighbourhoodFixture {
    pub study_area_name: &'static str,
    pub neighbourhood_name: &'static str,
    pub savefile_name: &'static str,
}

impl NeighbourhoodFixture {
    pub const BRISTOL_WEST: Self = Self {
        study_area_name: "bristol",
        neighbourhood_name: "west",
        savefile_name: "bristol_west",
    };
    pub const BRISTOL_EAST: Self = Self {
        study_area_name: "bristol",
        neighbourhood_name: "east",
        savefile_name: "bristol_east",
    };
    pub const STRASBOURG: Self = Self {
        study_area_name: "strasbourg",
        neighbourhood_name: "Schilik velorue",
        savefile_name: "strasbourg",
    };
}

impl NeighbourhoodFixture {
    pub fn map_model(&self) -> Result<MapModel> {
        self.map_model_builder()?()
    }

    pub fn neighbourhood_map(&self) -> Result<(Neighbourhood, MapModel)> {
        let (neighborhood_stats, map) = self.neighbourhood_params()?;

        // Uncomment if you want to re-save the savefiles
        // std::fs::write(self.savefile_path(), map.to_savefile().to_string())?;

        let edit_perimeter_roads = false;
        let neighbourhood = Neighbourhood::new(&map, neighborhood_stats, edit_perimeter_roads)?;
        Ok((neighbourhood, map))
    }

    pub fn map_model_builder(&self) -> Result<impl Fn() -> Result<MapModel> + use<'_>> {
        let study_area_name = &self.study_area_name;

        let pbf_path = format!("../web/public/severance_pbfs/{study_area_name}.pbf");
        let input_bytes = std::fs::read(&pbf_path)?;

        let boundary_path = format!("../web/public/boundaries/{study_area_name}.geojson");
        let boundary: Feature = std::fs::read_to_string(&boundary_path)?.parse()?;
        // All test study area boundaries are polygons for now
        let polygon: Polygon = boundary.try_into()?;
        let multi_polygon: MultiPolygon = polygon.into();

        Ok(move || {
            let demand = None;
            MapModel::new(
                &input_bytes,
                multi_polygon.clone(),
                Some(study_area_name.to_string()),
                demand,
            )
        })
    }

    pub fn savefile(&self) -> Result<FeatureCollection> {
        let savefile: FeatureCollection = std::fs::read_to_string(self.savefile_path())?.parse()?;
        Ok(savefile)
    }

    pub fn savefile_path(&self) -> String {
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
