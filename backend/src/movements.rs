use geo::{Line, LineInterpolatePoint};
use geojson::GeoJson;

use crate::{
    geo_helpers::{make_arrow, thicken_line},
    Direction, IntersectionID, MapModel,
};

impl MapModel {
    pub fn get_movements(&self, i: IntersectionID) -> GeoJson {
        let mut features = Vec::new();

        for r1 in &self.get_i(i).roads {
            for r2 in &self.get_i(i).roads {
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

                let polygon = make_arrow(line, thickness, double_ended)
                    .unwrap_or_else(|| thicken_line(line, thickness));
                features.push(self.mercator.to_wgs84_gj(&polygon));
            }
        }

        GeoJson::from(features)
    }
}
