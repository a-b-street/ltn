use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::Result;
use geo::{
    BooleanOps, Contains, EuclideanDistance, EuclideanLength, Intersects, MultiLineString, Polygon,
};
use geojson::{Feature, GeoJson, Geometry};

use crate::{Cell, IntersectionID, MapModel, RoadID, Shortcuts};

pub struct Neighbourhood {
    pub interior_roads: HashSet<RoadID>,
    crosses: HashMap<RoadID, f64>,
    pub border_intersections: HashSet<IntersectionID>,
}

impl Neighbourhood {
    pub fn new(map: &MapModel, boundary: Polygon) -> Result<Self> {
        let mut interior_roads = HashSet::new();
        let mut crosses = HashMap::new();
        for r in &map.roads {
            if boundary.contains(&r.linestring) {
                interior_roads.insert(r.id);
            } else if boundary.intersects(&r.linestring) {
                // Clip the linestring to the polygon
                let invert = false;
                let clipped = boundary.clip(&MultiLineString::from(r.linestring.clone()), invert);
                // How much of the clipped linestring is inside the boundary? If it's nearly 1,
                // then this road is interior.
                let pct = clipped.euclidean_length() / r.linestring.euclidean_length();
                if pct > 0.99 {
                    interior_roads.insert(r.id);
                } else {
                    // It's either something close to a perimeter road, or a weird case like
                    // https://www.openstreetmap.org/way/15778470 that's a bridge or tunnel
                    // crossing the boundary without touching it. For those cases, what do we want
                    // to do with them -- still consider them borders, yeah, because it's a way in
                    // or out.
                    crosses.insert(r.id, pct);
                }
            }
        }

        let mut border_intersections = HashSet::new();
        for i in &map.intersections {
            // Check distance to the polygon's linestring, rather than the polygon itself. Points
            // contained within a polygon and eight on the linestring both count as 0.
            let dist = i.point.euclidean_distance(boundary.exterior());
            // Allow a small tolerance
            if dist < 0.1 {
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

        // TODO Decide how/where state lives
        let modal_filters = BTreeMap::new();
        let cells = Cell::find_all(map, self, &modal_filters);
        let shortcuts = Shortcuts::new(map, self);

        // TODO Temporary rendering
        let mut road_to_cell = HashMap::new();
        for (idx, cell) in cells.iter().enumerate() {
            for (r, _) in &cell.roads {
                road_to_cell.insert(*r, idx);
            }
        }

        for r in &self.interior_roads {
            let mut f = map.get_r(*r).to_gj(&map.mercator);
            f.set_property("kind", "interior_road");
            f.set_property(
                "shortcuts",
                shortcuts.count_per_road.get(r).cloned().unwrap_or(0),
            );
            if let Some(cell) = road_to_cell.get(r) {
                f.set_property("cell", *cell);
            } else {
                error!("A road has no cell");
            }
            features.push(f);
        }
        for (r, pct) in &self.crosses {
            let mut f = map.get_r(*r).to_gj(&map.mercator);
            f.set_property("kind", "crosses");
            f.set_property("pct", *pct);
            features.push(f);
        }
        for i in &self.border_intersections {
            let mut f = Feature::from(Geometry::from(&map.mercator.to_wgs84(&map.get_i(*i).point)));
            f.set_property("kind", "border_intersection");
            features.push(f);
        }

        GeoJson::from(features)
    }
}
