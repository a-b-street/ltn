use anyhow::Result;
use geo::{Euclidean, Length, Line, LineInterpolatePoint, Point, Polygon};
use geojson::GeoJson;
use itertools::Itertools;

use crate::{
    geo_helpers::{euclidean_bearing, make_arrow, thicken_line},
    Intersection, IntersectionID, MapModel, Road, RoadID,
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

                    let (abs_bearing_1, abs_bearing_2) = i.bearing_of_roads(from, to);
                    let kind = classify_relative_bearing(abs_bearing_1, abs_bearing_2);

                    // Place at the end of the 'from' road
                    let pt = from.pt_near_intersection(offset, i.id);

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
                    f.set_property("intersection", i.id.0);
                    f.set_property("from_road", from.id.0);
                    f.set_property("to_road", to.id.0);
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
            for to_r in &intersection.roads {
                if *to_r == from.id {
                    continue;
                }
                // If there's already a TR between these two, skip it.
                if self.turn_restrictions[i.0].contains(&(from.id, *to_r)) {
                    continue;
                }

                let to = self.get_r(*to_r);

                let (abs_bearing_1, abs_bearing_2) = intersection.bearing_of_roads(from, to);
                let kind = classify_relative_bearing(abs_bearing_1, abs_bearing_2);

                let mut f = self.mercator.to_wgs84_gj(&to.linestring);
                f.set_property("road", to_r.0);
                f.set_property("kind", kind);
                if let Some(name) = to.tags.get("name") {
                    f.set_property("name", name.clone());
                }
                features.push(f);
            }
        }
        GeoJson::from(features)
    }

    /// Find (intersection, from, to) to represent a turn restriction
    pub fn find_turn_restriction(
        &self,
        pt: Point,
        bearing1: f64,
        bearing2: f64,
    ) -> Result<(IntersectionID, RoadID, RoadID)> {
        let intersection = match self.closest_intersection.nearest_neighbor(&pt) {
            Some(obj) => self.get_i(obj.data),
            None => bail!("No intersection near point"),
        };
        // For every road attached to this intersection, calculate its absolute bearing as a (from,
        // to) road
        let bearings: Vec<(RoadID, f64, f64)> = intersection
            .roads
            .iter()
            .map(|r| {
                let road = self.get_r(*r);
                let road_pt = self.get_r(*r).pt_near_intersection(0, intersection.id);
                (
                    road.id,
                    euclidean_bearing(road_pt.into(), intersection.point.into()),
                    euclidean_bearing(intersection.point.into(), road_pt.into()),
                )
            })
            .collect();

        // Find the best match for each bearing
        let (from, _, _) = bearings
            .iter()
            .min_by_key(|(_, b1, _)| smallest_rotation(bearing1, *b1) as usize)
            .unwrap();
        let (to, _, _) = bearings
            .iter()
            .min_by_key(|(_, _, b2)| smallest_rotation(bearing2, *b2) as usize)
            .unwrap();
        if from == to {
            bail!("Bearings {bearing1} and {bearing2} both matched to the same road");
        }
        Ok((intersection.id, *from, *to))
    }
}

fn render_arrow(i: IntersectionID, offset1: usize, road1: &Road, road2: &Road) -> Polygon {
    let line = Line::new(
        road1.pt_near_intersection(offset1, i),
        road2.pt_near_intersection(0, i),
    );
    let thickness = 2.0;
    let double_ended = false;
    make_arrow(line, thickness, double_ended).unwrap_or_else(|| thicken_line(line, thickness))
}

impl Intersection {
    /// Returns the absolute bearing of (road1 pointing to the intersection, road2 pointing away
    /// from the intersection)
    pub fn bearing_of_roads(&self, road1: &Road, road2: &Road) -> (f64, f64) {
        // TODO Calculate the two absolute bearings in an easier way? Why rely on
        // pt_near_intersection?
        (
            euclidean_bearing(
                road1.pt_near_intersection(0, self.id).into(),
                self.point.into(),
            ),
            euclidean_bearing(
                self.point.into(),
                road2.pt_near_intersection(0, self.id).into(),
            ),
        )
    }
}

impl Road {
    fn pt_near_intersection(&self, offset: usize, i: IntersectionID) -> Point {
        // If the road is long enough, offset from the intersection this much
        let distance_away = 5.0 * (1 + offset) as f64;
        let len = Euclidean.length(&self.linestring);

        if len > distance_away {
            let pct = if self.src_i == i {
                distance_away / len
            } else {
                1.0 - (distance_away / len)
            };
            return self.linestring.line_interpolate_point(pct).unwrap();
        }

        // If not, just take the other endpoint
        let pct = if self.src_i == i { 1.0 } else { 0.0 };
        self.linestring.line_interpolate_point(pct).unwrap()
    }
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

// Given two angles in [0, 360), return the smallest rotation between them
fn smallest_rotation(b1: f64, b2: f64) -> f64 {
    let rot1 = (b1 - b2).abs();
    let rot2 = 360.0 - rot1;
    rot1.min(rot2)
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

    #[test]
    fn test_smallest_rotation() {
        assert_eq!(0., smallest_rotation(30., 30.));
        assert_eq!(10., smallest_rotation(30., 40.));
        assert_eq!(10., smallest_rotation(40., 30.));
        assert_eq!(10., smallest_rotation(5., 355.));
        assert_eq!(10., smallest_rotation(355., 5.));
    }
}
