use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use geo::{
    Area, Distance, Euclidean, Length, LineInterpolatePoint, LineLocatePoint, LineString, Point,
    Polygon, PreparedGeometry, Relate,
};
use geojson::FeatureCollection;
use web_time::Instant;

use crate::geo_helpers::{aabb, buffer_aabb, clip_linestring_to_polygon};
use crate::render_cells::Color;
use crate::{Cell, Direction, IntersectionID, MapModel, RenderCells, RoadID, Shortcuts};

pub struct Neighbourhood {
    // Immutable once created
    pub interior_roads: BTreeSet<RoadID>,
    // Just debug, has no actual use
    crosses: BTreeMap<RoadID, f64>,
    pub border_intersections: BTreeSet<IntersectionID>,
    name: String,
    // Mercator
    pub boundary_polygon: Polygon,
    boundary_area_km2: f64,
    /// If true, shortcuts across perimeter roads will be calculated, and the user can edit these
    /// roads.
    pub edit_perimeter_roads: bool,

    // Updated after mutations
    derived: Option<DerivedNeighbourhoodState>,
}

struct DerivedNeighbourhoodState {
    render_cells: RenderCells,
    shortcuts: Shortcuts,
}

impl Neighbourhood {
    pub fn new(
        map: &MapModel,
        name: String,
        boundary_polygon: Polygon,
        edit_perimeter_roads: bool,
    ) -> Result<Self> {
        let t1 = Instant::now();
        let bbox = buffer_aabb(aabb(&boundary_polygon), 50.0);

        let prepared_boundary = PreparedGeometry::from(boundary_polygon.clone());

        let mut interior_roads = BTreeSet::new();
        let mut crosses = BTreeMap::new();
        for obj in map.closest_road.locate_in_envelope_intersecting(&bbox) {
            let r = &map.roads[obj.data.0];

            match line_in_polygon(&r.linestring, &boundary_polygon, &prepared_boundary) {
                LineInPolygon::Inside => {
                    interior_roads.insert(r.id);
                }
                LineInPolygon::Crosses { percent } => {
                    // It's either something close to a perimeter road, or a weird case like
                    // https://www.openstreetmap.org/way/15778470 that's a bridge or tunnel
                    // crossing the boundary without touching it. For those cases, what do we want
                    // to do with them -- still consider them borders, yeah, because it's a way in
                    // or out.
                    crosses.insert(r.id, percent);
                }
                LineInPolygon::Outside => {}
            }
        }

        let t2 = Instant::now();
        let mut border_intersections = BTreeSet::new();
        for obj in map
            .closest_intersection
            .locate_in_envelope_intersecting(&bbox)
        {
            // Check distance to the polygon's linestring, rather than the polygon itself. Points
            // contained within a polygon and right on the linestring both count as 0.
            let dist = Euclidean::distance(obj.geom(), boundary_polygon.exterior());
            // Allow a small tolerance
            if dist < 0.1 {
                border_intersections.insert(obj.data);
            }
        }

        if interior_roads.is_empty() {
            bail!("No roads inside the boundary");
        }

        // Convert from m^2 to km^2. Use unsigned area to ignore polygon orientation.
        let boundary_area_km2 = boundary_polygon.unsigned_area() / 1_000_000.0;
        let t3 = Instant::now();

        if true {
            info!("Neighbourhood set up, total {:?}. Finding roads took {:?}, intersections took {:?}", t3 - t1, t2 - t1, t3 - t2);
        }

        let mut n = Self {
            interior_roads,
            crosses,
            border_intersections,
            name,
            boundary_polygon,
            boundary_area_km2,
            edit_perimeter_roads,
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
        if true {
            info!("Neighbourhood edited, total {:?}. Finding cells took {:?}, rendering cells took {:?}, finding shortcuts took {:?}", t4 - t1, t2 - t1, t3 - t2, t4 - t3);
        }
    }

    pub fn editable_roads(&self) -> Vec<RoadID> {
        if self.edit_perimeter_roads {
            self.interior_roads
                .iter()
                .chain(self.crosses.keys())
                .cloned()
                .collect()
        } else {
            self.interior_roads.iter().cloned().collect()
        }
    }

    pub fn to_gj(&self, map: &MapModel) -> FeatureCollection {
        let mut features = Vec::new();

        let derived = self.derived.as_ref().unwrap();

        // Just one boundary
        features.push(map.boundaries.get(&self.name).cloned().unwrap());

        for r in self.editable_roads() {
            let road = map.get_r(r);
            let mut f = road.to_gj(&map.mercator);
            f.set_property("kind", "interior_road");
            f.set_property(
                "shortcuts",
                derived
                    .shortcuts
                    .count_per_road
                    .get(&r)
                    .cloned()
                    .unwrap_or(0),
            );
            f.set_property("direction", map.directions[&r].to_string());
            f.set_property(
                "direction_edited",
                map.directions[&r] != Direction::from_osm(&road.tags),
            );
            f.set_property("road", r.0);
            features.push(f);
        }

        // Only for debugging
        for (r, pct) in &self.crosses {
            let mut f = map.get_r(*r).to_gj(&map.mercator);
            f.set_property("kind", "crosses");
            f.set_property("pct", *pct);
            features.push(f);
        }
        for i in &self.border_intersections {
            let mut f = map.mercator.to_wgs84_gj(&map.get_i(*i).point);
            f.set_property("kind", "border_intersection");
            features.push(f);
        }

        for (polygons, color) in derived
            .render_cells
            .polygons_per_cell
            .iter()
            .zip(derived.render_cells.colors.iter())
        {
            let mut f = map.mercator.to_wgs84_gj(polygons);
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
                    "area_km2": self.boundary_area_km2,
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        }
    }
}

enum LineInPolygon {
    Inside,
    Crosses { percent: f64 },
    Outside,
}

fn line_in_polygon(
    linestring: &LineString,
    polygon: &Polygon,
    prepared_polygon: &PreparedGeometry,
) -> LineInPolygon {
    // TODO Reconsider rewriting all of this logic based on clip_linestring_to_polygon

    let matrix = prepared_polygon.relate(linestring);

    if matrix.is_within() {
        return double_check_line_in_polygon(linestring, polygon);
    }

    if !matrix.is_intersects() {
        return LineInPolygon::Outside;
    }

    // Clip the linestring to the polygon
    // Multiple segments generally don't happen, but might right on a boundary
    let mut sum = 0.0;
    for clipped in clip_linestring_to_polygon(linestring, polygon) {
        sum += clipped.length::<Euclidean>();
    }

    // How much of the clipped linestring is inside the boundary? If it's nearly 1, then this
    // road is interior. Round to make diffs less noisy.
    let percent = (sum / linestring.length::<Euclidean>() * 10e3).round() / 10e3;
    if percent <= 0.99 {
        return LineInPolygon::Crosses { percent };
    }
    double_check_line_in_polygon(linestring, polygon)
}

fn double_check_line_in_polygon(linestring: &LineString, polygon: &Polygon) -> LineInPolygon {
    // It looks like the line is inside, but there are false positives right along the polygon's
    // exterior. So find the distance between the linestring endpoints and the closest point on the
    // exterior. If both of those distances are too small, then it's probably right on the polygon.
    let ls_pt1 = linestring.points().next().unwrap();
    let ls_pt2 = linestring.points().last().unwrap();
    let polygon_pt1 = closest_point(polygon.exterior(), ls_pt1);
    let polygon_pt2 = closest_point(polygon.exterior(), ls_pt2);

    if Euclidean::distance(ls_pt1, polygon_pt1) < 0.1
        && Euclidean::distance(ls_pt2, polygon_pt2) < 0.1
    {
        return LineInPolygon::Crosses { percent: 1.0 };
    }

    LineInPolygon::Inside
}

fn closest_point(exterior: &LineString, pt: Point) -> Point {
    let fraction = exterior.line_locate_point(&pt).unwrap();
    exterior.line_interpolate_point(fraction).unwrap()
}
