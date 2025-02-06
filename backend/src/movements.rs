use geo::{Euclidean, Length, Line, LineInterpolatePoint, Point, Polygon};
use geojson::GeoJson;

use crate::{
    geo_helpers::{make_arrow, thicken_line},
    Direction, IntersectionID, MapModel, Road,
};

impl MapModel {
    pub fn get_movements(&self, i: IntersectionID) -> GeoJson {
        let mut features = Vec::new();

        let intersection = self.get_i(i);
        for r1 in &intersection.roads {
            for r2 in &intersection.roads {
                // TODO Handle u-turns at dead-ends
                if r1 == r2 {
                    continue;
                }
                let road1 = self.get_r(*r1);
                let road2 = self.get_r(*r2);

                // If road1 is one-way, can we go towards i?
                let ok1 = match self.directions[r1] {
                    Direction::BothWays => true,
                    Direction::Forwards => road1.dst_i == i,
                    Direction::Backwards => road1.src_i == i,
                };
                // If road2 is one-way, can we go away from i?
                let ok2 = match self.directions[r2] {
                    Direction::BothWays => true,
                    Direction::Forwards => road2.src_i == i,
                    Direction::Backwards => road2.dst_i == i,
                };
                if !ok1 || !ok2 {
                    continue;
                }

                // Is there a turn restriction between this pair?
                if intersection.turn_restrictions.contains(&(*r1, *r2)) {
                    continue;
                }

                if let Some(diagonal_filter) = self.diagonal_filters.get(&i) {
                    if !diagonal_filter.allows_movement(&(*r1, *r2)) {
                        continue;
                    }
                }

                let polygon = render_arrow(i, road1, road2);
                features.push(self.mercator.to_wgs84_gj(&polygon));
            }
        }

        GeoJson::from(features)
    }
}

fn render_arrow(i: IntersectionID, road1: &Road, road2: &Road) -> Polygon {
    let line = Line::new(
        pt_near_intersection(i, road1),
        pt_near_intersection(i, road2),
    );
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
