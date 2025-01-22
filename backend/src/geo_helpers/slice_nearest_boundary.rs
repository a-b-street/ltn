use geo::orient::Direction;
use geo::{
    Closest, ClosestPoint, Distance, Euclidean, FrechetDistance, LineString, Point, Polygon,
};
use std::cmp::Ordering;

pub trait SliceNearestFrechetBoundary {
    /// Returns the subset of self closest to `closest_to`.
    ///
    /// All points in the output will be *topologically* within `self`, however the first and
    /// final points of the output may not appear explicitly in `self`, in which case they
    /// represent splitting the existing segments at the point nearest `closest_to`.
    fn slice_nearest_frechet_boundary(&self, closest_to: &LineString) -> (LineString, f64);
    fn _slice_boundary_nearest_endpoint(&self, closest_to: &LineString) -> LineString;
}

impl SliceNearestFrechetBoundary for Polygon {
    fn slice_nearest_frechet_boundary(&self, closest_to: &LineString) -> (LineString, f64) {
        // We snip the polygon's exterior into two parts at the points nearest
        // `closest_to.first` and `closest_to.last`.
        //
        // Of the two parts, the one with the lowest frechet_distance represents the best
        // candidate for it's corresponding boundary.
        let forwards_closest_to = closest_to;
        let forwards = self._slice_boundary_nearest_endpoint(forwards_closest_to);
        let forwards_frechet = forwards.frechet_distance(closest_to);

        let mut backwards_closest_to = closest_to.clone();
        backwards_closest_to.0.reverse();
        let mut backwards = self._slice_boundary_nearest_endpoint(&backwards_closest_to);
        let backwards_frechet = backwards.frechet_distance(&backwards_closest_to);

        if forwards_frechet < backwards_frechet {
            (forwards, forwards_frechet)
        } else {
            backwards.0.reverse();
            (backwards, backwards_frechet)
        }
    }

    /// Note: this is only based on the start/end points, so depending on winding you might get a
    /// weird result.
    fn _slice_boundary_nearest_endpoint(&self, closest_to: &LineString) -> LineString {
        use geo::{coord, HasDimensions};

        // Not sure if this will ever be an issue in practice
        assert!(!closest_to.is_empty(), "we don't yet handle empty input");
        assert!(
            !self.exterior().is_empty(),
            "we don't yet handle empty input"
        );

        let exterior = self.exterior();

        let first_coord = *closest_to.0.first().expect("non-empty linestring");
        let final_coord = *closest_to.0.last().expect("non-empty linestring");

        let mut distance_to_first = f64::MAX;
        let mut segment_idx_closest_to_first = 0;
        let mut coord_closest_to_first = coord!(x: 0., y: 0.);

        let mut distance_to_final = f64::MAX;
        let mut segment_idx_closest_to_final = 0;
        let mut coord_closest_to_final = coord!(x: 0., y: 0.);
        for (segment_idx, segment) in exterior.lines().enumerate() {
            let new_first_distance = Euclidean::distance(&segment, first_coord);
            if new_first_distance < distance_to_first {
                distance_to_first = new_first_distance;
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

            let new_final_distance = Euclidean::distance(&segment, final_coord);
            if new_final_distance < distance_to_final {
                distance_to_final = new_final_distance;
                segment_idx_closest_to_final = segment_idx;
                coord_closest_to_final = match segment.closest_point(&Point(final_coord)) {
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

        let mut coords = match segment_idx_closest_to_first.cmp(&segment_idx_closest_to_final) {
            Ordering::Less => {
                let mut coords = exterior.0
                    [segment_idx_closest_to_first + 1..=segment_idx_closest_to_final]
                    .to_vec();
                coords.insert(0, coord_closest_to_first);
                coords.push(coord_closest_to_final);
                coords
            }
            Ordering::Equal => {
                vec![coord_closest_to_first, coord_closest_to_final]
            }
            Ordering::Greater => {
                // This means we "wrap around" the polygon, so we have to stitch together both halves
                let head = &exterior.0[segment_idx_closest_to_first + 1..];
                let mut coords = head.to_vec();
                let tail = &exterior.0[0..segment_idx_closest_to_final];
                coords.extend_from_slice(&tail);
                coords.insert(0, coord_closest_to_first);
                coords.push(coord_closest_to_final);
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
        let boundary = wkt!(POLYGON((0. 0.,0. 10.,10. 10.,10. 0.,0. 0.)));
        let (closest, _) =
            boundary.slice_nearest_frechet_boundary(&wkt!(LINESTRING(0.1 0.2,0.1 9.8)));
        assert_relative_eq!(wkt!(LINESTRING(0. 0.2,0. 9.8)), closest);
    }

    #[test]
    fn simple_segment_reversed_in_rect() {
        let boundary = wkt!(POLYGON((0. 0.,0. 10.,10. 10.,10. 0.,0. 0.)));
        let (closest, _) =
            boundary.slice_nearest_frechet_boundary(&wkt!(LINESTRING(0.1 9.8,0.1 0.2)));
        assert_relative_eq!(wkt!(LINESTRING(0. 9.8,0. 0.2)), closest);
    }

    #[test]
    fn simple_segment_in_reversed_rect() {
        let boundary = wkt!(POLYGON((0. 0.,10. 0.,10. 10.,0. 10.,0. 0.)));
        let (closest, _) =
            boundary.slice_nearest_frechet_boundary(&wkt!(LINESTRING(0.1 0.2,0.1 9.8)));
        assert_relative_eq!(
            wkt!(LINESTRING(0. 0.2,0. 9.8)),
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
            wkt!(LINESTRING(0. 9.8,0. 0.2)),
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

        // dbg!(line_string.length::<Euclidean>());
        // dbg!(closest.length::<Euclidean>());
        assert_relative_eq!(
            wkt!(LINESTRING(1495.3224949514188 1205.7438525123148,1495.1276851599255 1178.667850475072)),
            closest,
            max_relative = 1e-14
        );
    }
}
