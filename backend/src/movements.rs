use geo::{Euclidean, Length, Line, LineInterpolatePoint, Point, Polygon};
use geojson::GeoJson;
use itertools::Itertools;

use crate::{
    geo_helpers::{euclidean_bearing, make_arrow, thicken_line},
    IntersectionID, MapModel, Road, RoadID,
};

impl MapModel {
    pub fn get_movements(&self, i: IntersectionID) -> GeoJson {
        let mut features = Vec::new();

        let intersection = self.get_i(i);
        for (r1, r2) in intersection.allowed_movements(&self.router_input_after()) {
            let road1 = self.get_r(r1);
            let road2 = self.get_r(r2);
            let polygon = render_arrow(i, 0, road1, road2);
            features.push(self.mercator.to_wgs84_gj(&polygon));
        }

        GeoJson::from(features)
    }

    pub fn turn_restrictions_to_gj(&self) -> GeoJson {
        let mut features = Vec::new();

        for i in &self.intersections {
            for (tr_from, tr_all_to) in self.turn_restrictions[i.id.0]
                .iter()
                .cloned()
                .into_group_map()
            {
                let from = self.get_r(tr_from);

                // TODO Order the TR kinds consistently?
                for (offset, to) in tr_all_to.into_iter().enumerate() {
                    // TODO Skip if it's redundant with the one-ways

                    let to = self.get_r(to);

                    // TODO Calculate the two absolute bearings in an easier way? Why rely on
                    // pt_near_intersection?
                    let abs_bearing_1 = euclidean_bearing(
                        pt_near_intersection(0, i.id, from).into(),
                        i.point.into(),
                    );
                    let abs_bearing_2 =
                        euclidean_bearing(i.point.into(), pt_near_intersection(0, i.id, to).into());
                    let kind = classify_relative_bearing(abs_bearing_1, abs_bearing_2);

                    // Place at the end of the 'from' road
                    let pt = pt_near_intersection(offset, i.id, from);

                    // Rotate the icon based on the 'from' road's angle only, but make sure that road
                    // points at the intersection
                    let mut road_pointing_at_i = from.linestring.clone();
                    if from.src_i == i.id {
                        road_pointing_at_i.0.reverse();
                    }

                    let from_geometry = self
                        .mercator
                        .to_wgs84_gj(&from.linestring)
                        .geometry
                        .take()
                        .unwrap();
                    let to_geometry = self
                        .mercator
                        .to_wgs84_gj(&to.linestring)
                        .geometry
                        .take()
                        .unwrap();

                    let mut f = self.mercator.to_wgs84_gj(&Point::from(pt));
                    f.set_property("kind", kind);
                    // Use abs_bearing_1 to rotate the angle on the screen. The icons are "oriented"
                    // north, aka 0 means no rotation needed.
                    f.set_property("icon_angle", abs_bearing_1);
                    f.set_property(
                        "from_geometry",
                        serde_json::to_value(from_geometry).unwrap(),
                    );
                    f.set_property("to_geometry", serde_json::to_value(to_geometry).unwrap());
                    f.set_property(
                        "edited",
                        !self.original_turn_restrictions[i.id.0].contains(&(from.id, to.id)),
                    );
                    features.push(f);
                }
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
                if self.turn_restrictions[i.0].contains(&(from.id, *r)) {
                    continue;
                }

                let to = self.get_r(*r);
                let mut f = self.mercator.to_wgs84_gj(&to.linestring);
                f.set_property("road", r.0);
                if let Some(name) = to.tags.get("name") {
                    f.set_property("name", name.clone());
                }
                features.push(f);
            }
        }
        GeoJson::from(features)
    }
}

fn render_arrow(i: IntersectionID, offset1: usize, road1: &Road, road2: &Road) -> Polygon {
    let line = Line::new(
        pt_near_intersection(offset1, i, road1),
        pt_near_intersection(0, i, road2),
    );
    let thickness = 2.0;
    let double_ended = false;
    make_arrow(line, thickness, double_ended).unwrap_or_else(|| thicken_line(line, thickness))
}

fn pt_near_intersection(offset: usize, i: IntersectionID, road: &Road) -> Point {
    // If the road is long enough, offset from the intersection this much
    let distance_away = 5.0 * (1 + offset) as f64;
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

/// ```ignore
///
///             0 = straight
///
///             315     45
///                \   /
///                 \ /
/// 270 = left       *    90 = right
///                 /|\
///                / | \
///              225 |  135
///                  |
/// u_right_to_left  |  u_left_to_right
///                  180
/// ```
fn classify_relative_bearing(abs_bearing1: f64, abs_bearing2: f64) -> &'static str {
    let unnormalized_diff = abs_bearing2 - abs_bearing1;
    // Normalize to [0, 360]
    let diff = if unnormalized_diff < 0.0 {
        unnormalized_diff + 360.0
    } else {
        unnormalized_diff
    };

    if diff <= 45. {
        "straight"
    } else if diff <= 135. {
        "right"
    } else if diff <= 180. {
        "u_left_to_right"
    } else if diff <= 225. {
        "u_right_to_left"
    } else if diff <= 315. {
        "left"
    } else {
        "straight"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_relative_bearing() {
        assert_eq!("straight", classify_relative_bearing(0., 0.));
        assert_eq!("straight", classify_relative_bearing(30., 350.));
        assert_eq!("straight", classify_relative_bearing(350., 30.));
        assert_eq!("straight", classify_relative_bearing(180., 180.));
        assert_eq!("straight", classify_relative_bearing(180., 190.));
        assert_eq!("straight", classify_relative_bearing(180., 170.));

        assert_eq!("left", classify_relative_bearing(0., 270.));
        assert_eq!("left", classify_relative_bearing(180., 90.));

        assert_eq!("right", classify_relative_bearing(0., 90.));
        assert_eq!("right", classify_relative_bearing(180., 270.));
    }
}
