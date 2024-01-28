use std::collections::{HashMap, HashSet};

use anyhow::Result;
use geo::{
    BooleanOps, Contains, EuclideanDistance, EuclideanLength, Intersects, MultiLineString, Polygon,
};
use geojson::{Feature, FeatureCollection, Geometry};
use web_time::Instant;

use crate::render_cells::Color;
use crate::{Cell, IntersectionID, MapModel, RenderCells, RoadID, Shortcuts};

pub struct Neighbourhood {
    // Immutable once created
    pub interior_roads: HashSet<RoadID>,
    crosses: HashMap<RoadID, f64>,
    pub border_intersections: HashSet<IntersectionID>,
    name: String,
    pub boundary_polygon: Polygon,

    // Updated after mutations
    derived: Option<DerivedNeighbourhoodState>,
}

struct DerivedNeighbourhoodState {
    render_cells: RenderCells,
    shortcuts: Shortcuts,
}

impl Neighbourhood {
    pub fn new(map: &MapModel, name: String, boundary_polygon: Polygon) -> Result<Self> {
        let mut interior_roads = HashSet::new();
        let mut crosses = HashMap::new();
        for r in &map.roads {
            if boundary_polygon.contains(&r.linestring) {
                interior_roads.insert(r.id);
            } else if boundary_polygon.intersects(&r.linestring) {
                // Clip the linestring to the polygon
                let invert = false;
                let clipped =
                    boundary_polygon.clip(&MultiLineString::from(r.linestring.clone()), invert);
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
            let dist = i.point.euclidean_distance(boundary_polygon.exterior());
            // Allow a small tolerance
            if dist < 0.1 {
                border_intersections.insert(i.id);
            }
        }

        if interior_roads.is_empty() {
            bail!("No roads inside the boundary");
        }

        let mut n = Self {
            interior_roads,
            crosses,
            border_intersections,
            name,
            boundary_polygon,
            derived: None,
        };
        n.after_edit(map);
        Ok(n)
    }

    pub fn after_edit(&mut self, map: &MapModel) {
        let t1 = Instant::now();
        let cells = Cell::find_all(map, self);
        let t2 = Instant::now();
        let render_cells = RenderCells::new(map, self, &cells);
        let t3 = Instant::now();
        let shortcuts = Shortcuts::new(map, self);
        let t4 = Instant::now();
        self.derived = Some(DerivedNeighbourhoodState {
            render_cells,
            shortcuts,
        });
        if false {
            info!("Neighbourhood edited. Finding cells took {:?}, rendering cells took {:?}, finding shortcuts took {:?}", t2 - t1, t3 - t2, t4 - t3);
        }
    }

    pub fn to_gj(&self, map: &MapModel) -> FeatureCollection {
        let mut features = Vec::new();

        let derived = self.derived.as_ref().unwrap();

        // Just one boundary
        features.push(map.boundaries.get(&self.name).cloned().unwrap());

        for r in &self.interior_roads {
            let mut f = map.get_r(*r).to_gj(&map.mercator);
            f.set_property("kind", "interior_road");
            f.set_property(
                "shortcuts",
                derived
                    .shortcuts
                    .count_per_road
                    .get(r)
                    .cloned()
                    .unwrap_or(0),
            );
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

        for (polygons, color) in derived
            .render_cells
            .polygons_per_cell
            .iter()
            .zip(derived.render_cells.colors.iter())
        {
            let mut f = Feature::from(Geometry::from(&map.mercator.to_wgs84(polygons)));
            f.set_property("kind", "cell");
            match color {
                Color::Disconnected => f.set_property("cell_color", "disconnected"),
                Color::Cell(idx) => f.set_property("cell_color", *idx),
            }
            features.push(f);
        }

        FeatureCollection {
            features,
            bbox: None,
            foreign_members: Some(
                serde_json::json!({
                    "undo_length": map.undo_stack.len(),
                    "redo_length": map.redo_queue.len(),
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        }
    }
}
