use std::collections::{HashMap, HashSet};

use anyhow::Result;
use geo::{Contains, EuclideanDistance, Intersects, Polygon};
use geojson::{Feature, GeoJson, Geometry};

use crate::{IntersectionID, MapModel, RoadID};

pub struct Neighbourhood {
    interior_roads: HashSet<RoadID>,
    crosses: HashSet<RoadID>,
    border_intersections: HashMap<IntersectionID, f64>,
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

        let mut border_intersections = HashMap::new();
        for i in &map.intersections {
            // Check distance to the polygon's linestring, rather than the polygon itself. Points
            // contained within a polygon and eight on the linestring both count as 0.
            let dist = i.point.euclidean_distance(boundary.exterior());
            // Allow a small tolerance
            if dist < 0.1 {
                border_intersections.insert(i.id, dist);
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
        for (i, dist) in &self.border_intersections {
            let mut f = Feature::from(Geometry::from(
                &map.mercator.to_wgs84(&map.intersections[i.0].point),
            ));
            f.set_property("kind", "border_intersection");
            f.set_property("dist", *dist);
            features.push(f);
        }

        GeoJson::from(features)
    }
}
