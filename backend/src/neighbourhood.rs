use std::collections::HashSet;

use anyhow::Result;
use geo::{Contains, EuclideanDistance, Intersects, Polygon};
use geojson::{Feature, GeoJson, Geometry};

use crate::{IntersectionID, MapModel, RoadID};

pub struct Neighbourhood {
    interior_roads: HashSet<RoadID>,
    crosses: HashSet<RoadID>,
    border_intersections: HashSet<IntersectionID>,
}

impl Neighbourhood {
    pub fn new(map: &MapModel, boundary: Polygon) -> Result<Self> {
        let mut interior_roads = HashSet::new();
        let mut crosses = HashSet::new();
        for r in &map.roads {
            if boundary.contains(&r.linestring) {
                interior_roads.insert(r.id);
            } else if boundary.intersects(&r.linestring) {
                crosses.insert(r.id);
            }
        }

        let mut border_intersections = HashSet::new();
        for i in &map.intersections {
            // 0 if it's within...
            let dist = i.point.euclidean_distance(&boundary);
            if dist > 0.0 && dist < 5.0 {
                border_intersections.insert(i.id);
            }
        }

        if interior_roads.is_empty() {
            bail!("No roads inside the boundary");
        }

        Ok(Self {
            interior_roads,
            crosses,
            border_intersections,
        })
    }

    pub fn to_gj(&self, map: &MapModel) -> GeoJson {
        let mut features = Vec::new();

        for r in &self.interior_roads {
            let mut f = map.roads[r.0].to_gj(&map.mercator);
            f.set_property("kind", "interior_road");
            features.push(f);
        }
        for r in &self.crosses {
            let mut f = map.roads[r.0].to_gj(&map.mercator);
            f.set_property("kind", "crosses");
            features.push(f);
        }
        for i in &self.border_intersections {
            let mut f = Feature::from(Geometry::from(
                &map.mercator.to_wgs84(&map.intersections[i.0].point),
            ));
            f.set_property("kind", "border_intersection");
            features.push(f);
        }

        GeoJson::from(features)
    }
}
