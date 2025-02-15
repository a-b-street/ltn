use geo::{Euclidean, Length, Line, LineInterpolatePoint, Point, Polygon};
use geojson::GeoJson;

use crate::{
    geo_helpers::{euclidean_bearing, make_arrow, thicken_line},
    IntersectionID, MapModel, Road,
};

impl MapModel {
    pub fn get_movements(&self, i: IntersectionID) -> GeoJson {
        let mut features = Vec::new();

        let intersection = self.get_i(i);
        for (r1, r2) in intersection.allowed_movements(&self.router_input_after()) {
            let road1 = self.get_r(r1);
            let road2 = self.get_r(r2);
            let polygon = render_arrow(i, road1, road2);
            features.push(self.mercator.to_wgs84_gj(&polygon));
        }

        GeoJson::from(features)
    }

    pub fn turn_restrictions_to_gj(&self) -> GeoJson {
        let mut features = Vec::new();

        for i in &self.intersections {
            for (from, to) in &i.turn_restrictions {
                // TODO Skip if it's redundant with the one-ways

                // TODO Group by road first and offset them
                let line = movement_line(i.id, self.get_r(*from), self.get_r(*to));

                let mut f = self.mercator.to_wgs84_gj(&Point::from(line.start));
                f.set_property("angle", euclidean_bearing(line.start, line.end));
                // Editing isn't possible yet
                f.set_property("edited", false);
                features.push(f);
            }
        }

        GeoJson::from(features)
    }
}

fn movement_line(i: IntersectionID, road1: &Road, road2: &Road) -> Line {
    Line::new(
        pt_near_intersection(i, road1),
        pt_near_intersection(i, road2),
    )
}

fn render_arrow(i: IntersectionID, road1: &Road, road2: &Road) -> Polygon {
    let line = movement_line(i, road1, road2);
    let thickness = 2.0;
    let double_ended = false;
    make_arrow(line, thickness, double_ended).unwrap_or_else(|| thicken_line(line, thickness))
}

fn pt_near_intersection(i: IntersectionID, road: &Road) -> Point {
    // If the road is long enough, offset from the intersection this much
    let distance_away = 10.0;
    let len = Euclidean.length(&road.linestring);

    if len > distance_away {
        let pct = if road.src_i == i {
            distance_away / len
        } else {
            1.0 - (distance_away / len)
        };
        return road.linestring.line_interpolate_point(pct).unwrap();
    }

    // If not, just take the other endpoint
    let pct = if road.src_i == i { 1.0 } else { 0.0 };
    road.linestring.line_interpolate_point(pct).unwrap()
}
