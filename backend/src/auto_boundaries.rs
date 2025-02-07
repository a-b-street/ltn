use geo::{Area, Coord, Intersects, LineString, Polygon};
use geojson::FeatureCollection;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::float::slice::FloatSlice;

use crate::MapModel;

impl MapModel {
    pub fn render_auto_boundaries(&self) -> FeatureCollection {
        let mut features = Vec::new();
        let mut severances = Vec::new();
        let mut road_severances = Vec::new();

        for road in &self.roads {
            if road.tags.is_any(
                "highway",
                vec![
                    "motorway",
                    "motorway_link",
                    "trunk",
                    "trunk_link",
                    "primary",
                    "primary_link",
                    "secondary",
                    "secondary_link",
                    "tertiary",
                    "tertiary_link",
                ],
            ) {
                let mut f = self.mercator.to_wgs84_gj(&road.linestring);
                // TODO Important to distinguish, or just debugging?
                f.set_property("kind", "road severance");
                features.push(f);

                severances.push(road.linestring.clone());
                road_severances.push(road.linestring.clone());
            }
        }

        for linestring in &self.railways {
            let mut f = self.mercator.to_wgs84_gj(linestring);
            f.set_property("kind", "railway severance");
            features.push(f);

            severances.push(linestring.clone());
        }

        for linestring in &self.waterways {
            let mut f = self.mercator.to_wgs84_gj(linestring);
            f.set_property("kind", "waterway severance");
            features.push(f);

            severances.push(linestring.clone());
        }

        let boundary_mercator = self.mercator.to_mercator(&self.boundary_wgs84);
        for polygon in boundary_mercator
            .into_iter()
            .flat_map(|boundary_polygon| split_polygon(boundary_polygon, &severances))
        {
            // TODO This is expensive; could this info somehow be retained?
            let touches_big_road = boundary_touches_any(&polygon, &road_severances);
            let touches_railway = boundary_touches_any(&polygon, &self.railways);
            let touches_waterway = boundary_touches_any(&polygon, &self.waterways);

            let mut f = self.mercator.to_wgs84_gj(&polygon);
            f.set_property("kind", "area");
            f.set_property("touches_big_road", touches_big_road);
            f.set_property("touches_railway", touches_railway);
            f.set_property("touches_waterway", touches_waterway);
            // Convert from m^2 to km^2. Use unsigned area to ignore polygon orientation.
            f.set_property("area_km2", polygon.unsigned_area() / 1_000_000.0);
            features.push(f);
        }

        FeatureCollection {
            features,
            bbox: None,
            foreign_members: None,
        }
    }
}

// TODO Revisit some of this; conversions are now in geo
fn split_polygon(polygon: Polygon, linestrings: &Vec<LineString>) -> Vec<Polygon> {
    let mut shape = to_i_overlay_contour(polygon.exterior());

    // geo Polygon's are explicitly closed LineStrings, but i_overlay Polygon's are not.
    shape.pop();

    let splitters: Vec<_> = linestrings.iter().map(to_i_overlay_contour).collect();
    let shapes = shape.slice_by(&splitters, FillRule::NonZero);
    shapes.into_iter().map(to_geo_polygon).collect()
}

fn to_geo_polygon(rings: Vec<Vec<[f64; 2]>>) -> Polygon {
    let mut interiors: Vec<LineString> = rings.into_iter().map(to_geo_linestring).collect();
    let exterior = interiors.remove(0);
    Polygon::new(exterior, interiors)
}

fn to_geo_linestring(pts: Vec<[f64; 2]>) -> LineString {
    LineString(
        pts.into_iter()
            .map(|pt| Coord { x: pt[0], y: pt[1] })
            .collect(),
    )
}

fn to_i_overlay_contour(line_string: &LineString) -> Vec<[f64; 2]> {
    line_string.coords().map(|c| [c.x, c.y]).collect()
}

fn boundary_touches_any(polygon: &Polygon, linestrings: &Vec<LineString>) -> bool {
    // TODO At least consider an rtree to prune!
    linestrings
        .iter()
        .any(|ls| ls.intersects(polygon.exterior()))
}
