use geo::{Line, LineInterpolatePoint, Polygon};
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

                let polygon = render_arrow(i, road1, road2);
                features.push(self.mercator.to_wgs84_gj(&polygon));
            }
        }

        GeoJson::from(features)
    }
}

fn render_arrow(i: IntersectionID, road1: &Road, road2: &Road) -> Polygon {
    let pt1 = road1
        .linestring
        .line_interpolate_point(if road1.src_i == i { 0.2 } else { 0.8 })
        .unwrap();
    let pt2 = road2
        .linestring
        .line_interpolate_point(if road2.src_i == i { 0.2 } else { 0.8 })
        .unwrap();

    let thickness = 2.0;
    let double_ended = false;
    let line = Line::new(pt1, pt2);

    make_arrow(line, thickness, double_ended).unwrap_or_else(|| thicken_line(line, thickness))
}
