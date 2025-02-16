use geo::{Euclidean, Length, Line, LineInterpolatePoint, Point, Polygon};
use geojson::GeoJson;

use crate::{
    geo_helpers::{angle_of_pt_on_line, euclidean_bearing, limit_angle, make_arrow, thicken_line},
    IntersectionID, MapModel, Road, RoadID,
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

                let from = self.get_r(*from);
                let to = self.get_r(*to);
                let line = movement_line(i.id, from, to);
                // Place at the end of the 'from' road
                let pt = line.start;

                // Rotate the icon based on the 'from' road's angle only, but make sure that road
                // points at the intersection
                let mut road_pointing_at_i = from.linestring.clone();
                if from.src_i == i.id {
                    road_pointing_at_i.0.reverse();
                }
                // TODO Why +90 and +180? Kind of dunno, just making it look correct
                let icon_angle =
                    limit_angle(angle_of_pt_on_line(&road_pointing_at_i, pt.into()) + 90.0) + 180.0;

                // Use the angle between the two roads to determine the type of turn
                let bearing = euclidean_bearing(line.start, line.end);
                let kind = classify_bearing(bearing);

                // Render the polygon arrow showing this restriction more clearly
                let arrow = render_arrow(i.id, from, to);
                let arrow_geometry = self.mercator.to_wgs84_gj(&arrow).geometry.take().unwrap();

                let mut f = self.mercator.to_wgs84_gj(&Point::from(pt));
                f.set_property("kind", kind);
                f.set_property("icon_angle", icon_angle);
                // TODO Temporarily, to debug
                f.set_property("bearing", bearing);
                f.set_property("arrow", serde_json::to_value(arrow_geometry).unwrap());
                // Editing isn't possible yet
                f.set_property("edited", false);
                features.push(f);
            }
        }

        GeoJson::from(features)
    }

    pub fn get_turn_restriction_targets(&self, from: RoadID) -> GeoJson {
        let from = self.get_r(from);
        let mut features = Vec::new();
        // TODO Account for one-ways
        for i in [from.src_i, from.dst_i] {
            let intersection = self.get_i(i);
            for r in &intersection.roads {
                if *r == from.id {
                    continue;
                }
                // If there's already a TR between these two, skip it.
                if intersection.turn_restrictions.contains(&(from.id, *r)) {
                    continue;
                }

                let to = self.get_r(*r);
                let mut f = self.mercator.to_wgs84_gj(&to.linestring);
                f.set_property("road", r.0);
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

// TODO Unit test this? Or just draw it
fn classify_bearing(bearing: f64) -> &'static str {
    // Remember 0 is north (straight), 90 is east (right), 270 is west (left)
    let threshold = 30.0;

    if bearing <= 0.0 + threshold || bearing >= 360.0 - threshold {
        "straight"
    } else if bearing > 0.0 + threshold && bearing <= 90.0 + threshold {
        "right"
    } else if bearing < 360.0 - threshold && bearing >= 270.0 - threshold {
        "left"
    } else {
        "u"
    }
}
