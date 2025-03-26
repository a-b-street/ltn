use crate::geo_helpers::slice_nearest_boundary::SliceNearEndpoints;
use crate::{Road, RoadID};
use geo::line_measures::{Densifiable, LengthMeasurable};
use geo::{Euclidean, HausdorffDistance, LineString};
use std::collections::BTreeSet;

pub fn roads_along_line<'a>(
    roads: impl Iterator<Item = &'a Road>,
    line_string: &LineString,
) -> Vec<RoadID> {
    let mut road_likelihoods = vec![];

    for road in roads {
        // Higher score means higher likelihood that the road lies along the drawn line.
        let score = {
            // Hausdorff distance measures a type of linestring similarity:
            //   1. Get the set of shortest distances between all points on the two linestrings.
            //   2. The minimum of those shortest distances would be the classic "Distance" calculation between linestrings,
            //      though it's only computed between nodes.
            //   3. Whereas the longest of those shortest distances is the "Hausdorff Distance".

            // The line we draw will usually be much longer than the individual road segments we're
            // matching against. If we were to consider the entire drawn line, most of it would be
            // very far (in hausdorff terms) from any individual segment.
            // So we trim the drawn line back near the endpoints of the segment we're considering.
            //
            // Well matched roads, lying along the drawn line, will tend to be about as long as
            // the trimmed drawn line.
            // Whereas perpendicular roads tend to have a very short trimmed drawn line, and should
            // be punished.
            let trimmed_line_string = line_string.slice_near_endpoints(&road.linestring);
            let trimmed_line_length = trimmed_line_string.length(&Euclidean);
            let road_length = road.linestring.length(&Euclidean);
            let trimming_ratio = ((road_length - trimmed_line_length) / trimmed_line_length).abs();
            let trimming_score = 1.0 / (1.0 + trimming_ratio);

            // Hausdorff distance only computes distance between nodes, so we need to make sure our
            // road is thoroughly densified.
            // Densifying the drawn line_string will have minimal impact on Hausdorff distance,
            // so we skip it.
            let densified_road = road.linestring.densify(&Euclidean, 1.0);

            let hausdorff_distance = trimmed_line_string.hausdorff_distance(&densified_road);

            // Hausdorff distance is bounded by the length of the road, so taking raw Hausdorff
            // distance naively could unfairly benefit short segments.
            //
            // e.g. Imagine we're tracing along a 100 meter main road.
            // Our drawn line could reasonably deviate up to 10 meters from the road we're tracing,
            // giving us a Hausdorff distance of 10 meters.
            //
            // Now imagine that main road has a bunch of intersecting perpendicular 10 meter drive ways.
            // Those driveways all have a Hausdorff distance of 10 meters.
            //
            // So: compute the delta of `road_length - hausdorff` to penalize short roads
            let hausdorff_delta = road_length - hausdorff_distance;

            // By itself, that delta implies, given a long enough road, Hausdorff distance becomes
            // irrelevant. To account for that we also divide by the road length.
            let hausdorff_score = hausdorff_delta / road_length;

            hausdorff_score + trimming_score
        };

        road_likelihoods.push((road, score));
    }

    // We might want to tweak this if it's too strict/loose
    // We could also parameterize it, based on zoom. When zoomed in you can be more precise.
    let threshold = 1.5;

    let mut along = vec![];
    let mut not_along = vec![];

    let mut along_i = BTreeSet::new();
    for (road, score) in road_likelihoods {
        if score >= threshold {
            along.push((road, score));
            along_i.insert(road.src_i);
            along_i.insert(road.dst_i);
        } else {
            not_along.push((road, score));
        }
    }

    // Try to add in any missed connections
    //
    // It's easy to miss short connections between segments if your line
    // happens to deviate a little when crossing the connection.
    //
    // So give a bonus to anything connected to our existing `along` lines.
    let mut new_connections = vec![];
    loop {
        for (idx, (road, mut score)) in not_along.iter().enumerate() {
            let connected_bonus = threshold * 0.3;
            if along_i.contains(&road.src_i) {
                score += connected_bonus;
            }
            if along_i.contains(&road.dst_i) {
                score += connected_bonus;
            }
            if score >= threshold {
                new_connections.push(idx);
            }
        }

        // Adding a connection might recursively enable new connections,
        // so repeat until no new connections are added.
        if new_connections.is_empty() {
            break;
        } else {
            while let Some(last_idx) = new_connections.pop() {
                let road_score = not_along.remove(last_idx);
                along.push(road_score);
            }
        }
    }

    if along.is_empty() {
        // No good matches, but we know the user was trying to draw *something*,
        // Make our best guess so they can make progress.
        // This is especially helpful when doing cleanup to get any final little leftover segments.
        if let Some(closest) = not_along.iter().max_by(|(_r1, score1), (_r2, score2)| {
            score1
                .partial_cmp(score2)
                .unwrap_or(std::cmp::Ordering::Equal)
        }) {
            vec![closest.0.id]
        } else {
            vec![]
        }
    } else {
        along.into_iter().map(|(road, _)| road.id).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::geo_helpers::roads_along_line;
    use crate::{IntersectionID, Road, RoadID};
    use geo::{wkt, LineString};
    use osm_reader::WayID;
    use utils::Tags;

    fn road(id: usize, linestring: LineString) -> Road {
        // For this test, we don't consider connectedness,
        // so we give every intersection a unique ID
        let src_i = IntersectionID(id + 1000);
        let dst_i = IntersectionID(id + 2000);
        Road {
            id: RoadID(id),
            src_i,
            dst_i,
            way: WayID(123),
            linestring,
            tags: Tags(Default::default()),
            speed_mph: 0,
        }
    }

    #[test]
    fn exclude_short_perpendicular_roads() {
        // Geometries lifted from a Bristol neighbourhood.
        // Real world geometries lifted from a Bristol neighbourhood.
        //
        // If you need to update this test, it'd be reasonable to just grab another set from the
        // debug console which have the same characteristics, there's nothing particularly sacred about these.
        let line_string = wkt!(LINESTRING(2714.9251154723443 1405.9543323582727,2710.295719658016 1401.3262811170637,2700.5739885454377 1384.202528156889,2690.852257358002 1360.1368108719553,2687.1487407360883 1346.252795091471,2683.908163672953 1318.4848774612738,2682.519344996617 1266.1890450431092,2681.1305262454234 1251.379703664963,2677.4270096235095 1238.421565400088,2670.019976306795 1224.537882543963,2670.019976306795 1218.5216317629931,2674.649372119153 1206.9519387870262,2674.649372119153 1205.1007903605253,2673.26055336796 1203.7124294657174,2672.3346742504023 1200.9357088517531));

        // EXCLUDE: Cooperage Road - Short stub, perpendicular to line.
        let road_741 = road(
            741,
            wkt!(LINESTRING(2670.734872548158 1258.917339879581,2661.2878993451413 1263.287306533261)),
        );
        // EXCLUDE: no name - Short stub, perpendicular to line.
        let road_3360 = road(
            3360,
            wkt!(LINESTRING(2673.2023138249424 1207.600810351793,2658.1134945564545 1210.136058181511)),
        );
        // EXCLUDE: no name - Short stub, perpendicular to line, and farther from line
        let road_3361 = road(
            3361,
            wkt!(LINESTRING(2658.1134945564545 1210.136058181511,2648.118971182499 1212.7157840430527)),
        );
        // INCLUDE: Netham Road. It's short but somewhat parallel to the road direction.
        let road_665 = road(
            665,
            wkt!(LINESTRING(2673.2023138249424 1207.600810351793,2673.243899913871 1212.0263745454492)),
        );
        // INCLUDE: Netham Road. It's medium short but mostly parallel to the road direction.
        let road_911 = road(
            911,
            wkt!(LINESTRING(2676.5292009397203 1252.3234716221045,2680.285810973464 1258.961817912194,2682.3928394794866 1262.6868530993036,2685.075142215785 1269.7699797107846)),
        );
        // EXCLUDE: Grindell Road - Short stub, perpendicular to line.
        let road_2083 = road(
            2083,
            wkt!(LINESTRING(2683.2176302433622 1208.4014149293491,2673.243899913871 1212.0263745454492)),
        );
        // INCLUDE: Netham Road - long and mostly parallel to the road. Should be included.
        let road_912 = road(
            912,
            wkt!(LINESTRING(2685.075142215785 1269.7699797107846,2685.768243698034 1271.593579026593,2686.0801393650263 1281.1452364180823,2684.964245978623 1297.3241205922984,2682.2126330941205 1316.4385548842208,2681.4710145081012 1321.4756920191976,2682.0740127976524 1327.4802263516601,2692.1586393643365 1363.0848910421403,2697.0311427845145 1380.2867699546082,2702.0907836048996 1392.6294238601815,2707.8573879371565 1400.1684503004171,2723.9304113104217 1411.8550532328468)),
        );

        let roads = vec![
            road_741, road_3360, road_3361, road_665, road_911, road_2083, road_912,
        ];

        let expected = vec![RoadID(665), RoadID(911), RoadID(912)];
        let actual = roads_along_line(roads.iter(), &line_string);
        assert_eq!(expected, actual);
    }

    #[test]
    fn exclude_long_parallel_roads_that_are_far_away() {
        // Real world geometries lifted from a Bristol neighbourhood.
        //
        // If you need to update this test, it'd be reasonable to just grab another set from the
        // debug console which have the same characteristics, there's nothing particularly sacred about these.
        let line_string = wkt!(LINESTRING(2541.7682765144386 1161.2486674255972,2551.3101163852075 1175.1897364336735,2563.053919295652 1189.130843735762,2576.632691413377 1199.0363906121681,2579.9356359975145 1202.3382438848537,2590.9454512451393 1215.9125520617677,2595.7163711549147 1223.6169052903697,2600.120297232296 1228.7531472832736,2608.1941617774264 1232.7887696266173,2608.9281494422153 1241.960650485197,2610.396124815132 1247.0969110451822,2618.8369831434093 1254.80131160641,2625.809866144078 1262.5057238577883));

        let roads = vec![
            road(
                743,
                wkt!(LINESTRING(2631.3805703862904 1275.5854824070113,2599.2067995804837 1289.6627795639567,2589.946963777706 1290.0074843127586,2587.146833789434 1288.0504509006907,2582.911983732924 1285.0926617666776,2521.621019658025 1183.6938681018958,2515.2306239917225 1179.490694068935)),
            ),
            road(
                1124,
                wkt!(LINESTRING(2631.3805703862904 1275.5854824070113,2612.1262112095264 1240.3811200045193,2606.671502544262 1230.406921307945,2530.8392693718747 1150.8023633684263)),
            ),
        ];
        let expected = vec![RoadID(1124)];
        let actual = roads_along_line(roads.iter(), &line_string);
        assert_eq!(expected, actual);
    }
}
