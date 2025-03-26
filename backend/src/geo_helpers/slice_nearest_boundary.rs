use geo::{
    coord, line_measures::FrechetDistance, Closest, ClosestPoint, Coord, Distance, Euclidean,
    HasDimensions, LineString, Point, Polygon,
};
use std::cmp::Ordering;

pub trait SliceNearEndpoints {
    type SliceType;

    /// returns points on `self` nearest to the beginning and end of `closest_to`, and the segment
    /// index on `self` where each of those coords lies.
    ///
    /// returns (start, end)
    fn coords_near_endpoints(&self, closest_to: &LineString) -> ((usize, Coord), (usize, Coord));

    /// Splits `self` at the two points nearest to `closest_to.start` and `closest_to.end`
    ///
    /// In the likely event that the closest points on the exterior are not pre-existing vertices,
    /// new coords will be interpolated into the existing segments.
    fn slice_near_endpoints(&self, closest_to: &LineString) -> Self::SliceType;
}

pub trait SliceNearestFrechetBoundary {
    /// Returns the subset of self.exterior() closest to `closest_to` and its frechet distance.
    ///
    /// i.e. returns one of the LineStrings from `split_boundary_nearest_endpoints`, whichever
    /// one has the smallest frechet distance.
    ///
    /// All points in the output will be *topologically* within `self`, however the first and
    /// final points of the output may not appear explicitly in `self`, in which case they
    /// represent splitting the existing segments at the point nearest `closest_to`.
    fn slice_nearest_frechet_boundary(&self, closest_to: &LineString) -> (LineString, f64);
}

impl SliceNearestFrechetBoundary for Polygon {
    fn slice_nearest_frechet_boundary(&self, closest_to: &LineString) -> (LineString, f64) {
        // We snip the polygon's exterior into two parts at the points nearest
        // `closest_to.first` and `closest_to.last`.
        //
        // Of the two parts, the one with the lowest frechet_distance represents the best
        // candidate for its corresponding boundary.
        let (forwards_half, backwards_half) = self.slice_near_endpoints(closest_to);
        let forwards_frechet = Euclidean.frechet_distance(&forwards_half, closest_to);

        // The second half of the polygon begins where the first half ends, so we
        // need to reverse `closest_to` to get an accurate (minimal) distance measure
        let mut backwards_closest_to = closest_to.clone();
        backwards_closest_to.0.reverse();
        let backwards_frechet = Euclidean.frechet_distance(&backwards_half, &backwards_closest_to);

        if forwards_frechet < backwards_frechet {
            (forwards_half, forwards_frechet)
        } else {
            (backwards_half, backwards_frechet)
        }
    }
}

impl SliceNearEndpoints for Polygon {
    fn coords_near_endpoints(&self, closest_to: &LineString) -> ((usize, Coord), (usize, Coord)) {
        self.exterior().coords_near_endpoints(closest_to)
    }

    // A closed ring can be split into two equally plausible (complementing) halves.
    type SliceType = (LineString, LineString);
    fn slice_near_endpoints(&self, closest_to: &LineString) -> (LineString, LineString) {
        let exterior = self.exterior();

        let (
            (segment_idx_closest_to_first, coord_closest_to_first),
            (segment_idx_closest_to_last, coord_closest_to_last),
        ) = self.coords_near_endpoints(closest_to);

        let assemble = |starting_segment_idx: usize,
                        ending_segment_idx: usize,
                        starting_coord,
                        ending_coord| {
            let mut coords = match starting_segment_idx.cmp(&ending_segment_idx) {
                Ordering::Less => {
                    let mut coords = exterior.0[starting_segment_idx..=ending_segment_idx].to_vec();
                    coords[0] = starting_coord;
                    coords.push(ending_coord);
                    coords
                }
                Ordering::Equal => {
                    // can we consolidate this?
                    vec![starting_coord, ending_coord]
                }
                Ordering::Greater => {
                    // This means we "wrap around" the polygon, so we have to stitch together both halves
                    let head = &exterior.0[starting_segment_idx..];
                    let tail = &exterior.0[0..ending_segment_idx];
                    let mut coords = head.to_vec();
                    coords[0] = starting_coord;
                    coords.extend_from_slice(tail);
                    coords.push(ending_coord);
                    coords
                }
            };
            coords.dedup();
            LineString::new(coords)
        };

        let front_half = assemble(
            segment_idx_closest_to_first,
            segment_idx_closest_to_last,
            coord_closest_to_first,
            coord_closest_to_last,
        );
        let back_half = assemble(
            segment_idx_closest_to_last,
            segment_idx_closest_to_first,
            coord_closest_to_last,
            coord_closest_to_first,
        );
        (front_half, back_half)
    }
}

impl SliceNearEndpoints for LineString {
    fn coords_near_endpoints(&self, closest_to: &LineString) -> ((usize, Coord), (usize, Coord)) {
        // Not sure if this will ever be an issue in practice
        assert!(!closest_to.is_empty(), "we don't yet handle empty input");
        assert!(!self.0.is_empty(), "we don't yet handle empty input");

        let first_coord = *closest_to.0.first().expect("non-empty linestring");
        let final_coord = *closest_to.0.last().expect("non-empty linestring");

        let mut distance_to_first = f64::MAX;
        let mut segment_idx_closest_to_first = 0;
        let mut coord_closest_to_first = coord!(x: 0., y: 0.);

        let mut distance_to_end = f64::MAX;
        let mut segment_idx_closest_to_end = 0;
        let mut coord_closest_to_end = coord!(x: 0., y: 0.);
        for (segment_idx, segment) in self.lines().enumerate() {
            let new_start_distance = Euclidean.distance(&segment, first_coord);
            if new_start_distance < distance_to_first {
                distance_to_first = new_start_distance;
                segment_idx_closest_to_first = segment_idx;
                coord_closest_to_first = match segment.closest_point(&Point(first_coord)) {
                    Closest::SinglePoint(p) => p.0,
                    Closest::Intersection(p) => p.0,
                    Closest::Indeterminate => {
                        // I don't think this should happen, but let's try to lumber on if it does.
                        // And assert so we know that we have to think harder about this.
                        debug_assert!(false, "Only happens with empty or invalid geometries");
                        continue;
                    }
                };
            }

            let new_end_distance = Euclidean.distance(&segment, final_coord);
            if new_end_distance < distance_to_end {
                distance_to_end = new_end_distance;
                segment_idx_closest_to_end = segment_idx;
                coord_closest_to_end = match segment.closest_point(&Point(final_coord)) {
                    Closest::SinglePoint(p) => p.0,
                    Closest::Intersection(p) => p.0,
                    Closest::Indeterminate => {
                        // I don't think this should happen, but let's try to lumber on if it does.
                        // And assert so we know that we have to think harder about this.
                        debug_assert!(false, "Only happens with empty or invalid geometries");
                        continue;
                    }
                };
            }
        }
        (
            (segment_idx_closest_to_first, coord_closest_to_first),
            (segment_idx_closest_to_end, coord_closest_to_end),
        )
    }

    type SliceType = LineString;
    fn slice_near_endpoints(&self, closest_to: &LineString) -> LineString {
        let (
            (segment_idx_closest_to_first, coord_closest_to_first),
            (segment_idx_closest_to_last, coord_closest_to_last),
        ) = self.coords_near_endpoints(closest_to);

        let mut coords = match segment_idx_closest_to_first.cmp(&segment_idx_closest_to_last) {
            Ordering::Less => {
                let mut coords =
                    self.0[segment_idx_closest_to_first..=segment_idx_closest_to_last].to_vec();
                coords[0] = coord_closest_to_first;
                coords.push(coord_closest_to_last);
                coords
            }
            Ordering::Equal => {
                // can we consolidate this?
                vec![coord_closest_to_first, coord_closest_to_last]
            }
            Ordering::Greater => {
                let mut coords =
                    self.0[segment_idx_closest_to_last..=segment_idx_closest_to_first].to_vec();
                coords[0] = coord_closest_to_last;
                coords.push(coord_closest_to_first);
                coords
            }
        };
        coords.dedup();
        LineString::new(coords)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use geo::wkt;

    #[test]
    fn simple_segment_in_rect() {
        // You can use JTSTestBuilder or another app to inspect and analyze WKT geometries.
        // JTSTestBuilder.jar is available to download with every JTS release: https://github.com/locationtech/jts/releases
        // WKT is also the output Debug format for geo-types geometries (as of 0.7.16+).
        let boundary = wkt!(POLYGON((0. 0.,0. 10.,10. 10.,10. 0.,0. 0.)));
        let (closest, _) =
            boundary.slice_nearest_frechet_boundary(&wkt!(LINESTRING(0.1 0.2,0.1 9.8)));
        assert_relative_eq!(wkt!(LINESTRING(0.0 9.8,0.0 0.2)), closest);
    }

    #[test]
    fn simple_segment_reversed_in_rect() {
        let boundary = wkt!(POLYGON((0. 0.,0. 10.,10. 10.,10. 0.,0. 0.)));
        let (closest, _) =
            boundary.slice_nearest_frechet_boundary(&wkt!(LINESTRING(0.1 9.8,0.1 0.2)));
        assert_relative_eq!(wkt!(LINESTRING(0. 0.2,0. 9.8)), closest);
    }

    #[test]
    fn simple_segment_in_reversed_rect() {
        let boundary = wkt!(POLYGON((0. 0.,10. 0.,10. 10.,0. 10.,0. 0.)));
        let (closest, _) =
            boundary.slice_nearest_frechet_boundary(&wkt!(LINESTRING(0.1 0.2,0.1 9.8)));
        assert_relative_eq!(
            wkt!(LINESTRING(0. 9.8, 0. 0.2)),
            closest,
            max_relative = 1e-14
        );
    }

    #[test]
    fn simple_segment_reveresed_in_reversed_rect() {
        let boundary = wkt!(POLYGON((0. 0.,10. 0.,10. 10.,0. 10.,0. 0.)));
        let (closest, _) =
            boundary.slice_nearest_frechet_boundary(&wkt!(LINESTRING(0.1 9.8,0.1 0.2)));
        assert_relative_eq!(
            wkt!(LINESTRING(0. 0.2, 0. 9.8)),
            closest,
            max_relative = 1e-14
        );
    }

    #[test]
    fn around_corner_in_rect() {
        let boundary = wkt!(POLYGON((0. 0.,0. 10.,10. 10.,10. 0.,0. 0.)));
        let (closest, _) =
            boundary.slice_nearest_frechet_boundary(&wkt!(LINESTRING(0.1 0.2,0.1 9.8,5.0 9.8)));
        assert_relative_eq!(wkt!(LINESTRING(0.0 0.2,0. 10.,5. 10.)), closest);
    }

    #[test]
    fn wrapping_around_initial_point() {
        let boundary = wkt!(POLYGON((0. 0.,0. 10.,10. 10.,10. 0.,0. 0.)));
        let (closest, _) =
            boundary.slice_nearest_frechet_boundary(&wkt!(LINESTRING(5. 0.2,0.1 0.2,0.1 5.)));
        assert_relative_eq!(wkt!(LINESTRING(5. 0.,0. 0.,0. 5.)), closest);
    }

    #[test]
    fn almost_full_circuit() {
        let boundary = wkt!(POLYGON((0. 0.,0. 10.,10. 10.,10. 0.,0. 0.)));
        let (closest, _) = boundary
            .slice_nearest_frechet_boundary(&wkt!(LINESTRING(0.1 0.2,0.1 9.8,9.9 9.8,9.9 0.2)));
        assert_relative_eq!(
            wkt!(LINESTRING(0. 0.2,0. 10.,10. 10.,10. 0.2)),
            closest,
            max_relative = 1e-14
        );
    }

    #[test]
    fn almost_full_circuit_wrapping_around_initial_point() {
        let boundary = wkt!(POLYGON((0. 0.,0. 10.,10. 10.,10. 0.,0. 0.)));
        let (closest, _) = boundary
            .slice_nearest_frechet_boundary(&wkt!(LINESTRING(9.9 9.8,9.9 0.2,0.1 0.2,0.1 9.8)));
        assert_relative_eq!(
            wkt!(LINESTRING(10. 9.8,10. 0.,0. 0.,0. 9.8)),
            closest,
            max_relative = 1e-14
        );
    }

    #[test]
    fn fix_wrapping_wrong_way() {
        // This example simplified from real-world data
        // The algorithm sometimes picks the *complement* of the border we want

        let boundary = wkt!(POLYGON ((1495.030280264711 1166.3140770613454, 1495.1276851599255 1178.667850475072, 1495.3224949514188 1205.7438525123148, 1495.6671584281842 1719.4428841752342, 1878.3934715810285 1719.365047618951, 1872.9612754760572 1361.6727135240535, 1495.030280264711 1166.3140770613454)));
        // A segment of harvard ave east: https://www.openstreetmap.org/way/256916775#map=18/47.648036/-122.322121
        let line_string = wkt!(LINESTRING(1495.3224949514188 1205.7438525123148,1495.1276851599255 1178.667850475072));
        let (closest, _) = boundary.slice_nearest_frechet_boundary(&line_string);

        assert_relative_eq!(
            wkt!(LINESTRING(1495.1276851599255 1178.667850475072,1495.3224949514188 1205.7438525123148)),
            closest,
            max_relative = 1e-14
        );
    }
}
