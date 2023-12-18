use std::collections::HashSet;

use anyhow::Result;
use geo::{Contains, Polygon};
use geojson::GeoJson;

use crate::{MapModel, RoadID};

pub struct Neighbourhood {
    interior_roads: HashSet<RoadID>,
}

impl Neighbourhood {
    pub fn new(map: &MapModel, boundary: Polygon) -> Result<Self> {
        let mut interior_roads = HashSet::new();
        for r in &map.roads {
            if boundary.contains(&r.linestring) {
                interior_roads.insert(r.id);
            }
        }

        if interior_roads.is_empty() {
            bail!("No roads inside the boundary");
        }

        Ok(Self { interior_roads })
    }

    pub fn to_gj(&self, map: &MapModel) -> GeoJson {
        let mut features = Vec::new();

        for r in &self.interior_roads {
            let mut f = map.roads[r.0].to_gj(&map.mercator);
            f.set_property("kind", "interior_road");
            features.push(f);
        }

        GeoJson::from(features)
    }
}
