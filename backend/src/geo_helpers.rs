use geo::{
    Contains, Intersects, LineInterpolatePoint, LineIntersection, LineLocatePoint, LineSplit,
    LineString, Polygon,
};

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
