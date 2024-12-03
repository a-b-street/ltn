use geo::{Area, Coord, LineString, Polygon};
use geojson::FeatureCollection;
use i_float::f64_point::F64Point;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::f64::string::F64StringOverlay;
use i_overlay::string::rule::StringRule;

use crate::MapModel;

impl MapModel {
    pub fn render_auto_boundaries(&self) -> FeatureCollection {
        let mut features = Vec::new();
        let mut severances = Vec::new();

        for road in &self.roads {
            if road.tags.is_any(
                "highway",
                vec![
                    "motorway",
                    "motorway_link",
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

        for polygon in split_polygon(self.mercator.to_mercator(&self.boundary_wgs84), severances) {
            let mut f = self.mercator.to_wgs84_gj(&polygon);
            f.set_property("kind", "area");
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

fn split_polygon(polygon: Polygon, linestrings: Vec<LineString>) -> Vec<Polygon> {
    let mut overlay = F64StringOverlay::new();
    overlay.add_shape_path(polygon.exterior().coords().map(to_pt).collect());
    for ls in linestrings {
        overlay.add_string_lines(
            ls.lines()
                .map(|l| [to_pt(&l.start), to_pt(&l.end)])
                .collect(),
        );
    }

    let graph = overlay.into_graph(FillRule::NonZero);
    let shapes = graph.extract_shapes(StringRule::Slice);

    shapes.into_iter().map(to_geo_polygon).collect()
}

fn to_pt(pt: &Coord) -> F64Point {
    F64Point::new(pt.x, pt.y)
}

fn to_geo_polygon(rings: Vec<Vec<F64Point>>) -> Polygon {
    let mut interiors: Vec<LineString> = rings.into_iter().map(to_geo_linestring).collect();
    let exterior = interiors.remove(0);
    Polygon::new(exterior, interiors)
}

fn to_geo_linestring(pts: Vec<F64Point>) -> LineString {
    LineString(
        pts.into_iter()
            .map(|pt| Coord { x: pt.x, y: pt.y })
            .collect(),
    )
}
