mod slice_nearest_boundary;
pub use slice_nearest_boundary::SliceNearestFrechetBoundary;

use geo::{
    BooleanOps, BoundingRect, Contains, Coord, Distance, Euclidean, Intersects, Length, Line,
    LineInterpolatePoint, LineIntersection, LineLocatePoint, LineString, Point, Polygon, Rect,
    Validation,
};
use rstar::AABB;
use utils::LineSplit;

/// Looks for the first place ls2 crosses ls1. Returns the percent_along ls1 of that point.
pub fn linestring_intersection(ls1: &LineString, ls2: &LineString) -> Option<f64> {
    if !ls1.intersects(ls2) {
        return None;
    }
    // TODO Urgh very brute force
    // TODO Could use https://docs.rs/geo/latest/geo/algorithm/sweep/struct.Intersections.html, but
    // not sure about the order, so we'd do line_locate_point for everything and take the min
    for line1 in ls1.lines() {
        for line2 in ls2.lines() {
            if let Some(LineIntersection::SinglePoint { intersection, .. }) =
                geo::algorithm::line_intersection::line_intersection(line1, line2)
            {
                return ls1.line_locate_point(&intersection.into());
            }
        }
    }
    // TODO Didn't find it...
    None
}

/// Returns the fraction along ls1 every place ls1 and ls2 intersect, sorted.
fn all_intersections(ls1: &LineString, ls2: &LineString) -> Vec<f64> {
    let mut fractions = Vec::new();

    for line1 in ls1.lines() {
        for line2 in ls2.lines() {
            if let Some(LineIntersection::SinglePoint { intersection, .. }) =
                geo::algorithm::line_intersection::line_intersection(line1, line2)
            {
                if let Some(fraction) = ls1.line_locate_point(&intersection.into()) {
                    fractions.push(fraction);
                }
            }
        }
    }

    // May not be sorted, because multiple line2's might cross a line1 in any order
    fractions.sort_by_key(|x| (*x * 10e9) as usize);

    fractions
}

/// Clips a linestring to the interior of a polygon's exterior. (Holes aren't
/// handled.)
///
/// This is a non-crashing version of BooleanOps::clip. It's NOT heavily tested!
pub fn clip_linestring_to_polygon(linestring: &LineString, polygon: &Polygon) -> Vec<LineString> {
    let mut fractions = all_intersections(linestring, polygon.exterior());
    // Make sure to visit the start and end
    fractions.insert(0, 0.0);
    fractions.push(1.0);
    // In case there was an intersection at an endpoint
    fractions.dedup();

    let mut results = Vec::new();
    if let Some(splits) = linestring.line_split_many(&fractions) {
        for split in splits {
            let Some(split) = split else {
                continue;
            };
            // Is this piece inside the polygon or not? Check the midpoint
            if let Some(midpt) = split.line_interpolate_point(0.5) {
                if polygon.contains(&midpt) {
                    results.push(split);
                }
            }
        }
    }
    results
}

pub fn buffer_aabb(aabb: AABB<Point>, buffer_meters: f64) -> AABB<Point> {
    AABB::from_corners(
        Point::new(
            aabb.lower().x() - buffer_meters,
            aabb.lower().y() - buffer_meters,
        ),
        Point::new(
            aabb.upper().x() + buffer_meters,
            aabb.upper().y() + buffer_meters,
        ),
    )
}

// TODO What in the generics is going on here...
pub fn aabb<G: BoundingRect<f64, Output = Option<Rect<f64>>>>(geom: &G) -> AABB<Point> {
    let bbox: Rect = geom.bounding_rect().unwrap().into();
    AABB::from_corners(
        Point::new(bbox.min().x, bbox.min().y),
        Point::new(bbox.max().x, bbox.max().y),
    )
}

pub fn angle_of_line(line: Line) -> f64 {
    (line.dy()).atan2(line.dx()).to_degrees()
}

/// North is 0°
/// East is 90°
/// South  is 180°
/// West is 270°
pub fn euclidean_bearing(origin: Coord, destination: Coord) -> f64 {
    (angle_of_line(Line::new(origin, destination)) + 450.0) % 360.0
}

/// The bearing of the first segment of `linestring` starting from `endpoint`.
///
/// precondition: `endpoint` must be either the first or last point in `linestring`
/// precondition: `linestring` must have at least 2 coordinates
pub fn bearing_from_endpoint(endpoint: Point, linestring: &LineString) -> f64 {
    assert!(
        linestring.0.len() >= 2,
        "zero length roads should be filtered out"
    );
    let next_coord = if endpoint.0 == linestring.0[0] {
        linestring.0[1]
    } else if endpoint.0 == linestring.0[linestring.0.len() - 1] {
        linestring.0[linestring.0.len() - 2]
    } else {
        // I'm assuming this won't happen, but maybe it's possible,
        // e.g. to different rounding schemes.
        debug_assert!(false, "road does not terminate at intersection");
        linestring.0[1]
    };

    euclidean_bearing(endpoint.0, next_coord)
}

pub fn angle_of_pt_on_line(linestring: &LineString, pt: Coord) -> f64 {
    let line = linestring
        .lines()
        .min_by_key(|line| (Euclidean.distance(line, pt) * 10e9) as usize)
        .unwrap();
    angle_of_line(line)
}

/// Constrain an angle between [0, 180]. Used for rotating modal filter icons visually
pub fn limit_angle(a1: f64) -> f64 {
    // Normalize to [0, 360]
    let a2 = if a1 < 0.0 { a1 + 360.0 } else { a1 };
    // Don't draw things upside down
    if a2 > 180.0 {
        a2 - 180.0
    } else {
        a2
    }
}

pub fn euclidean_destination(pt: Point, angle_degs: f64, dist_away_m: f64) -> Point {
    let (sin, cos) = angle_degs.to_radians().sin_cos();
    Point::new(pt.x() + dist_away_m * cos, pt.y() + dist_away_m * sin)
}

fn euclidean_destination_coord(pt: Coord, angle_degs: f64, dist_away_m: f64) -> Coord {
    euclidean_destination(pt.into(), angle_degs, dist_away_m).into()
}

/// Attempts to make the input polygon valid by union-ing it with itself.
///
/// It hasn't been thoroughly tested.
pub fn make_polygon_valid(polygon: &Polygon) -> Polygon {
    let mut valid_multipolygon = polygon.union(polygon);

    // I don't think we should get more than one piece back for any sane input, but we'll see...
    debug_assert!(
        valid_multipolygon.0.len() == 1,
        "MultiPolygon not handle yet"
    );

    let Some(valid_polygon) = valid_multipolygon.0.pop() else {
        debug_assert!(false, "empty valid polygon not handled yet");
        return polygon.clone();
    };

    debug_assert!(valid_polygon.is_valid());
    valid_polygon
}

// If the line is too short for the thickness, give up
pub fn make_arrow(line: Line, thickness: f64, double_ended: bool) -> Option<Polygon> {
    let head_size = thickness * 2.0;
    let triangle_height = head_size / 2.0_f64.sqrt();
    let angle = angle_of_line(line);
    let length = Euclidean.length(&line);

    if length < triangle_height * 3.0 {
        return None;
    }

    let mut pts = Vec::new();

    let start_trimmed = euclidean_destination_coord(line.start, angle, triangle_height);
    let end_trimmed = euclidean_destination_coord(line.start, angle, length - triangle_height);

    if double_ended {
        pts.push(line.start);
        pts.push(euclidean_destination_coord(
            start_trimmed,
            angle + 90.0,
            thickness * 1.5,
        ));
        pts.push(euclidean_destination_coord(
            start_trimmed,
            angle + 90.0,
            thickness * 0.5,
        ));
    } else {
        pts.push(euclidean_destination_coord(
            line.start,
            angle + 90.0,
            thickness * 0.5,
        ));
    }
    pts.push(euclidean_destination_coord(
        end_trimmed,
        angle + 90.0,
        thickness * 0.5,
    ));
    pts.push(euclidean_destination_coord(
        end_trimmed,
        angle + 90.0,
        thickness * 1.5,
    ));

    pts.push(line.end);

    pts.push(euclidean_destination_coord(
        end_trimmed,
        angle - 90.0,
        thickness * 1.5,
    ));
    pts.push(euclidean_destination_coord(
        end_trimmed,
        angle - 90.0,
        thickness * 0.5,
    ));
    if double_ended {
        pts.push(euclidean_destination_coord(
            start_trimmed,
            angle - 90.0,
            thickness * 0.5,
        ));
        pts.push(euclidean_destination_coord(
            start_trimmed,
            angle - 90.0,
            thickness * 1.5,
        ));
    } else {
        pts.push(euclidean_destination_coord(
            line.start,
            angle - 90.0,
            thickness * 0.5,
        ));
    }

    pts.push(pts[0]);

    Some(Polygon::new(LineString::new(pts), Vec::new()))
}

pub fn thicken_line(line: Line, thickness: f64) -> Polygon {
    let angle = angle_of_line(line);
    Polygon::new(
        LineString::new(vec![
            euclidean_destination_coord(line.start, angle - 90.0, thickness * 0.5),
            euclidean_destination_coord(line.end, angle - 90.0, thickness * 0.5),
            euclidean_destination_coord(line.end, angle + 90.0, thickness * 0.5),
            euclidean_destination_coord(line.start, angle + 90.0, thickness * 0.5),
            euclidean_destination_coord(line.start, angle - 90.0, thickness * 0.5),
        ]),
        Vec::new(),
    )
}

/// Create a polygon covering the world, minus a hole for the input polygon. Assumes the input is
/// in WGS84 and has no holes itself.
pub fn invert_polygon(wgs84_polygon: Polygon) -> Polygon {
    Polygon::new(
        LineString::from(vec![
            (180.0, 90.0),
            (-180.0, 90.0),
            (-180.0, -90.0),
            (180.0, -90.0),
            (180.0, 90.0),
        ]),
        vec![wgs84_polygon.into_inner().0],
    )
}

/// The "diagonal line" is an equal angular distance from a and b.
/// The diagonal bearing is the bearing of this "diagonal line".
///
/// That is, given the bearing of a and b, returns the bearing of line c.
///
/// ```ignore
///       a    b
///        \  /
///      ∂° \/ ∂°
///    ------------ c
/// ```
pub fn diagonal_bearing(bearing_a: f64, bearing_b: f64) -> f64 {
    let angle_between = bearing_b - bearing_a;
    (angle_between / 2.0 + 90.0 + bearing_a) % 360.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use geo::wkt;

    #[test]
    fn test_line_bearing() {
        let p1 = Point::new(0.0, 0.0);

        // Due South in our projection
        assert_relative_eq!(90.0, angle_of_line(Line::new(p1, Point::new(0.0, 1.0))));
        // East
        assert_relative_eq!(0.0, angle_of_line(Line::new(p1, Point::new(1.0, 0.0))));
        // North
        assert_relative_eq!(-90.0, angle_of_line(Line::new(p1, Point::new(0.0, -1.0))));
        // West
        assert_relative_eq!(180.0, angle_of_line(Line::new(p1, Point::new(-1.0, 0.0))));
    }

    #[test]
    fn test_bearing_from_endpoint() {
        let p1 = Point::new(0.0, 0.0);

        // p1 is start point

        // North
        assert_relative_eq!(
            0.,
            bearing_from_endpoint(p1, &wkt!(LINESTRING(0. 0.,0. -1.)))
        );
        // East
        assert_relative_eq!(
            90.,
            bearing_from_endpoint(p1, &wkt!(LINESTRING(0. 0.,1. 0.)))
        );
        // South
        assert_relative_eq!(
            180.,
            bearing_from_endpoint(p1, &wkt!(LINESTRING(0. 0.,0. 1.)))
        );
        // West
        assert_relative_eq!(
            270.,
            bearing_from_endpoint(p1, &wkt!(LINESTRING(0. 0.,-1. 0.)))
        );
        // Northwest
        assert_relative_eq!(
            315.,
            bearing_from_endpoint(p1, &wkt!(LINESTRING(0. 0.,-1. -1.)))
        );

        // Flipped - p1 is now the end point, not the start point

        // North
        assert_relative_eq!(
            0.,
            bearing_from_endpoint(p1, &wkt!(LINESTRING(0. -1.,0. 0.)))
        );
        // East
        assert_relative_eq!(
            90.,
            bearing_from_endpoint(p1, &wkt!(LINESTRING(1. 0.,0. 0.)))
        );
        // South
        assert_relative_eq!(
            180.,
            bearing_from_endpoint(p1, &wkt!(LINESTRING(0. 1., 0. 0.)))
        );
        // West
        assert_relative_eq!(
            270.,
            bearing_from_endpoint(p1, &wkt!(LINESTRING(-1. 0.,0. 0.)))
        );
    }

    #[test]
    fn test_diagonal_angle() {
        assert_eq!(135.0, diagonal_bearing(0., 90.));
        assert_eq!(270.0, diagonal_bearing(135., 225.));
        assert_eq!(270.0, diagonal_bearing(300., 60.));
    }
}
