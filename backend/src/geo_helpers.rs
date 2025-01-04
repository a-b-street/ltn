use geo::{
    BoundingRect, Contains, Coord, Distance, Euclidean, Intersects, Line, LineInterpolatePoint,
    LineIntersection, LineLocatePoint, LineString, Point, Polygon, Rect,
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

pub fn angle_of_pt_on_line(linestring: &LineString, pt: Coord) -> f64 {
    let line = linestring
        .lines()
        .min_by_key(|line| (Euclidean::distance(line, pt) * 10e9) as usize)
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
    Point::new(
        pt.x() + dist_away_m * cos,
        pt.y() + dist_away_m * sin,
    )
}
