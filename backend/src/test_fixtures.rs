use crate::MapModel;
use anyhow::Result;
use geo::{MultiPolygon, Polygon};
use geojson::{Feature, FeatureCollection};

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

    pub fn map_model_builder(&self) -> Result<impl Fn() -> Result<MapModel> + use<'_>> {
        let study_area_name = &self.study_area_name;

        let pbf_path = format!("../web/public/osm/{study_area_name}.pbf");
        let input_bytes = std::fs::read(&pbf_path)?;

        let boundary_path = format!("../web/public/boundaries/{study_area_name}.geojson");
        let boundary: Feature = std::fs::read_to_string(&boundary_path)?.parse()?;
        // All test study area boundaries are polygons for now
        let polygon: Polygon = boundary.try_into()?;
        let multi_polygon: MultiPolygon = polygon.into();

        Ok(move || {
            MapModel::new(
                &input_bytes,
                multi_polygon.clone(),
                Some(study_area_name.to_string()),
            )
        })
    }

    pub fn savefile(&self) -> Result<FeatureCollection> {
        let savefile: FeatureCollection = std::fs::read_to_string(format!(
            "../tests/{savefile_name}.geojson",
            savefile_name = self.savefile_name
        ))?
        .parse()?;
        Ok(savefile)
    }

    pub fn boundary_geo(&self, map: &MapModel) -> Result<Polygon> {
        // set_current_neighbourhood equivalent
        let boundary_gj = map
            .boundaries
            .get(self.neighbourhood_name)
            .cloned()
            .unwrap();
        let mut boundary_geo: Polygon = boundary_gj.try_into()?;
        map.mercator.to_mercator_in_place(&mut boundary_geo);
        Ok(boundary_geo)
    }

    pub fn neighbourhood_params(&self) -> Result<(MapModel, Polygon)> {
        let mut map = self.map_model()?;
        map.load_savefile(self.savefile()?)?;
        let boundary_geo = self.boundary_geo(&map)?;
        Ok((map, boundary_geo))
    }
}
