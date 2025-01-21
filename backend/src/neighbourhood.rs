use std::collections::{BTreeMap, BTreeSet};

use crate::geo_helpers::SliceNearestFrechetBoundary;
use anyhow::Result;
use geo::{
    Area, BooleanOps, Distance, Euclidean, Length, Line, LineString, Polygon, PreparedGeometry,
    Relate, Validation,
};
use geojson::{Feature, FeatureCollection, Geometry};
use web_time::Instant;

use crate::geo_helpers::{
    aabb, angle_of_line, buffer_aabb, clip_linestring_to_polygon, euclidean_destination,
    invert_polygon, make_arrow,
};
use crate::{Cell, Direction, IntersectionID, MapModel, RenderCells, RoadID, Shortcuts};

pub struct Neighbourhood {
    // Immutable once created
    pub interior_roads: BTreeSet<RoadID>,
    // Immutable once created
    pub perimeter_roads: BTreeSet<RoadID>,
    // Immutable once created
    crosses: BTreeMap<RoadID, f64>,
    pub border_intersections: BTreeSet<IntersectionID>,
    name: String,
    // Mercator
    pub boundary_polygon: Polygon,
    boundary_area_km2: f64,
    /// If true, shortcuts across perimeter roads will be calculated, and the user can edit these
    /// roads.
    pub edit_perimeter_roads: bool,

    // Updated after mutations
    derived: Option<DerivedNeighbourhoodState>,
}

struct DerivedNeighbourhoodState {
    render_cells: RenderCells,
    shortcuts: Shortcuts,
}

impl Neighbourhood {
    pub fn new(
        map: &MapModel,
        name: String,
        boundary_polygon: Polygon,
        edit_perimeter_roads: bool,
    ) -> Result<Self> {
        // make boundary_polygon valid - later topology checks require it, notably the "is perimeter" check.
        let boundary_polygon = {
            let mut valid = boundary_polygon.union(&boundary_polygon);
            // I don't think this should happen with any sane input, but we'll see...
            debug_assert!(valid.0.len() == 1, "multipolygons not handle yet");
            valid.0.pop().expect("empty valid polygon not handled yet")
        };
        debug_assert!(boundary_polygon.is_valid());

        let t1 = Instant::now();
        let bbox = buffer_aabb(aabb(&boundary_polygon), 50.0);

        let prepared_boundary = PreparedGeometry::from(&boundary_polygon);

        let mut interior_roads = BTreeSet::new();
        let mut perimeter_roads = BTreeSet::new();
        let mut crosses = BTreeMap::new();
        debug!("boundary_polygon: {boundary_polygon:?}",);
        for obj in map.closest_road.locate_in_envelope_intersecting(&bbox) {
            let r = &map.roads[obj.data.0];
            let result = line_in_polygon(&r.linestring, &boundary_polygon, &prepared_boundary);
            debug!(
                "linestring {road_id}: {linestring:?}, way: {way_id}, result: {result:?}",
                road_id = obj.data,
                linestring = r.linestring,
                way_id = r.way
            );
            match result {
                LineInPolygon::Inside => {
                    interior_roads.insert(r.id);
                }
                LineInPolygon::Perimeter => {
                    perimeter_roads.insert(r.id);
                }
                LineInPolygon::Crosses { percent } => {
                    // It's either something close to a perimeter road, or a weird case like
                    // https://www.openstreetmap.org/way/15778470 that's a bridge or tunnel
                    // crossing the boundary without touching it. For those cases, what do we want
                    // to do with them -- still consider them borders, yeah, because it's a way in
                    // or out.
                    crosses.insert(r.id, percent);
                }
                LineInPolygon::Outside => {}
            }
        }

        let t2 = Instant::now();
        let mut border_intersections = BTreeSet::new();
        for obj in map
            .closest_intersection
            .locate_in_envelope_intersecting(&bbox)
        {
            // Check distance to the polygon's linestring, rather than the polygon itself. Points
            // contained within a polygon and right on the linestring both count as 0.
            let dist = Euclidean::distance(obj.geom(), boundary_polygon.exterior());
            // Allow a small tolerance
            if dist < 0.1 {
                border_intersections.insert(obj.data);
            }
        }

        if interior_roads.is_empty() {
            bail!("No roads inside the boundary");
        }

        if perimeter_roads.is_empty() {
            // REVIEW: Is this actually a problem?
            bail!("No perimeter roads");
        }

        // Convert from m^2 to km^2. Use unsigned area to ignore polygon orientation.
        let boundary_area_km2 = boundary_polygon.unsigned_area() / 1_000_000.0;
        let t3 = Instant::now();

        if true {
            info!("Neighbourhood set up, total {:?}. Finding roads took {:?}, intersections took {:?}", t3 - t1, t2 - t1, t3 - t2);
        }

        let mut n = Self {
            interior_roads,
            perimeter_roads,
            crosses,
            border_intersections,
            name,
            boundary_polygon,
            boundary_area_km2,
            edit_perimeter_roads,
            derived: None,
        };
        n.after_edit(map);
        Ok(n)
    }

    pub fn after_edit(&mut self, map: &MapModel) {
        let t1 = Instant::now();
        let cells = Cell::find_all(map, self);
        let t2 = Instant::now();
        let render_cells = RenderCells::new(map, self, &cells);
        let t3 = Instant::now();
        let shortcuts = Shortcuts::new(map, self);
        let t4 = Instant::now();
        self.derived = Some(DerivedNeighbourhoodState {
            render_cells,
            shortcuts,
        });
        if true {
            info!("Neighbourhood edited, total {:?}. Finding cells took {:?}, rendering cells took {:?}, finding shortcuts took {:?}", t4 - t1, t2 - t1, t3 - t2, t4 - t3);
        }
    }

    pub fn editable_roads(&self) -> Vec<RoadID> {
        if self.edit_perimeter_roads {
            self.interior_roads
                .iter()
                .chain(self.perimeter_roads.iter())
                .cloned()
                .collect()
        } else {
            self.interior_roads.iter().cloned().collect()
        }
    }

    pub fn to_gj(&self, map: &MapModel) -> FeatureCollection {
        let mut features = Vec::new();

        let derived = self.derived.as_ref().unwrap();

        // Invert the boundary
        {
            let mut boundary_feature = map.boundaries.get(&self.name).cloned().unwrap();
            let p: Polygon = boundary_feature.clone().try_into().unwrap();
            boundary_feature.geometry = Some(Geometry::from(&invert_polygon(p)));
            features.push(boundary_feature);
        }

        for r in self.editable_roads() {
            let road = map.get_r(r);
            let mut f = road.to_gj(&map.mercator);
            f.set_property("kind", "interior_road");
            f.set_property(
                "shortcuts",
                derived
                    .shortcuts
                    .count_per_road
                    .get(&r)
                    .cloned()
                    .unwrap_or(0),
            );
            f.set_property("direction", map.directions[&r].to_string());
            f.set_property(
                "direction_edited",
                map.directions[&r] != Direction::from_osm(&road.tags),
            );
            f.set_property(
                "edited",
                map.directions[&r] != Direction::from_osm(&road.tags)
                    || map.modal_filters.get(&r) != map.original_modal_filters.get(&r),
            );
            f.set_property("road", r.0);
            if let Some(color) = derived.render_cells.colors_per_road.get(&r) {
                f.set_property("cell_color", *color);
            }

            features.push(f);
        }

        // Only for debugging
        for (r, pct) in &self.crosses {
            let mut f = map.get_r(*r).to_gj(&map.mercator);
            f.set_property("kind", "crosses");
            f.set_property("pct", *pct);
            features.push(f);
        }
        for i in &self.border_intersections {
            let mut f = map.mercator.to_wgs84_gj(&map.get_i(*i).point);
            f.set_property("kind", "border_intersection");
            features.push(f);

            features.extend(self.border_arrow(*i, map));
        }

        for (polygons, color) in derived
            .render_cells
            .polygons_per_cell
            .iter()
            .zip(derived.render_cells.colors.iter())
        {
            let mut f = map.mercator.to_wgs84_gj(polygons);
            f.set_property("kind", "cell");
            f.set_property("cell_color", *color);
            features.push(f);
        }

        FeatureCollection {
            features,
            bbox: None,
            foreign_members: Some(
                serde_json::json!({
                    "undo_length": map.undo_stack.len(),
                    "redo_length": map.redo_queue.len(),
                    "area_km2": self.boundary_area_km2,
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        }
    }

    fn border_arrow(&self, i: IntersectionID, map: &MapModel) -> Vec<Feature> {
        let derived = self.derived.as_ref().unwrap();
        let mut features = Vec::new();
        let intersection = map.get_i(i);
        for r in &intersection.roads {
            // Most borders only have one road in the interior of the neighbourhood. Draw an arrow
            // for each of those. If there happen to be multiple interior roads for one border, the
            // arrows will overlap each other -- but that happens anyway with borders close
            // together at certain angles.
            if !self.interior_roads.contains(r) {
                continue;
            }

            // Design choice: when we have a filter right at the entrance of a neighbourhood, it
            // creates its own little cell allowing access to just the very beginning of the
            // road. Let's not draw anything for that.
            if map.modal_filters.contains_key(r) {
                continue;
            }

            // Find the angle pointing into the neighbourhood
            let road = map.get_r(*r);
            let angle_in = if road.src_i == i {
                angle_of_line(road.linestring.lines().next().unwrap())
            } else {
                angle_of_line(road.linestring.lines().last().unwrap()) + 180.0
            };

            let center = intersection.point;
            let pt_farther = euclidean_destination(center, angle_in + 180.0, 40.0);
            let pt_closer = euclidean_destination(center, angle_in + 180.0, 10.0);

            // Point out of the neighbourhood
            let mut line = Line::new(pt_closer, pt_farther);
            // If the road is one-way and points in, then flip it
            if (map.directions[r] == Direction::Forwards && road.src_i == i)
                || (map.directions[r] == Direction::Backwards && road.dst_i == i)
            {
                std::mem::swap(&mut line.start, &mut line.end);
            }

            let thickness = 6.0;
            let double_ended = map.directions[r] == Direction::BothWays;
            if let Some(polygon) = make_arrow(line, thickness, double_ended) {
                let mut f = map.mercator.to_wgs84_gj(&polygon);
                f.set_property("kind", "border_arrow");
                f.set_property("cell_color", derived.render_cells.colors_per_border[&i]);
                features.push(f);
            }
        }
        features
    }
}

#[derive(Debug, Clone, PartialEq)]
enum LineInPolygon {
    Inside,
    Perimeter,
    Crosses { percent: f64 },
    Outside,
}

// NOTE: polygon must be `valid` (no spikes!) to get reasonable results
fn line_in_polygon(
    linestring: &LineString,
    polygon: &Polygon,
    prepared_polygon: &PreparedGeometry,
) -> LineInPolygon {
    // TODO Reconsider rewriting all of this logic based on clip_linestring_to_polygon

    let perimeter_likelihood = perimeter_likelihood(&linestring, polygon);
    // dbg!(&linestring);
    debug!("linestring: {linestring:?}");
    // dbg!(&perimeter_likelihood);
    debug!("perimeter_likelihood: {perimeter_likelihood:?}");
    if perimeter_likelihood > 1e-3 {
        return LineInPolygon::Perimeter;
    }

    let matrix = prepared_polygon.relate(linestring);
    if matrix.is_within() {
        return LineInPolygon::Inside;
    }

    if !matrix.is_intersects() {
        return LineInPolygon::Outside;
    }

    // Clip the linestring to the polygon
    // Multiple segments generally don't happen, but might right on a boundary
    let mut length_in_polygon = 0.0;
    // TODO: update this to use i_overlay impl
    for clipped in clip_linestring_to_polygon(linestring, polygon) {
        let length = clipped.length::<Euclidean>();
        length_in_polygon += length
    }
    // How much of the clipped linestring is inside the boundary? If it's nearly 1, then this
    // road is interior. Round to make diffs less noisy.
    let percent = (length_in_polygon / linestring.length::<Euclidean>() * 10e3).round() / 10e3;
    info!("percent: {percent}");
    if percent > 0.99 {
        LineInPolygon::Inside
    } else if percent < 0.01 {
        LineInPolygon::Outside
    } else {
        LineInPolygon::Crosses { percent }
    }
}

/// A high value means the line_string is "mostly close" to the boundary's exterior.
///
/// Essentially, we take the area between the line_string and the boundary, and divide it by the line_string's length.
///
/// So the more of the line_string that is "far" from the boundary, the lower this metric goes.
fn perimeter_likelihood(line_string: &LineString, boundary: &Polygon) -> f64 {
    let (_nearby_boundary, frechet_distance) = boundary.slice_nearest_frechet_boundary(line_string);
    let metric = (-frechet_distance).exp();
    debug!("perimeter_likelihood: {metric}");
    metric
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo::wkt;

    mod perimeter_metric {
        use super::*;
        use approx::assert_relative_eq;

        #[test]
        fn line_segment_on_exterior() {
            let boundary = wkt!(POLYGON((0. 0.,0. 10.,10. 10.,10. 0.,0. 0.)));
            let line = wkt!(LINESTRING(0.0 0.2,0.0 9.8));
            assert_relative_eq!(1.0, perimeter_likelihood(&line, &boundary));
        }

        #[test]
        fn line_segment_close_to_exterior() {
            let boundary = wkt!(POLYGON((0. 0.,0. 10.,10. 10.,10. 0.,0. 0.)));

            {
                let line = wkt!(LINESTRING(0.1 0.2,0.1 9.8));
                assert_relative_eq!(0.9048374180359595, perimeter_likelihood(&line, &boundary));
            }

            {
                // Length of the line, so long as it's all the same distance from the perimeter,
                // should not affect the metric
                let shorter_line = wkt!(LINESTRING(0.1 5.,0.1 9.8));
                assert_relative_eq!(
                    0.9048374180359595,
                    perimeter_likelihood(&shorter_line, &boundary)
                );
            }

            {
                let farther_line = wkt!(LINESTRING(0.2 5.,0.2 9.8));
                assert_relative_eq!(
                    0.7536383164437652,
                    perimeter_likelihood(&farther_line, &boundary)
                );
            }

            {
                let much_farther_line = wkt!(LINESTRING(1.2 5.,1.2 9.8));
                assert_relative_eq!(
                    0.29624972759029666,
                    perimeter_likelihood(&much_farther_line, &boundary)
                );
            }
        }

        #[test]
        fn almost_circuit() {
            let boundary = wkt!(POLYGON((0. 0.,0. 10.,10. 10.,10. 0.,0. 0.)));
            let line = wkt!(LINESTRING(0.1 0.2,0.1 9.8,9.9 9.8,9.9 0.2));
            assert_relative_eq!(
                0.7996294886770354,
                perimeter_likelihood(&line, &boundary),
                max_relative = 1e-14
            );
        }

        #[test]
        fn almost_full_circuit_wrapping_around_initial_point() {
            let boundary = wkt!(POLYGON((0. 0.,0. 10.,10. 10.,10. 0.,0. 0.)));
            let line = wkt!(LINESTRING(9.9 9.8,9.9 0.2,0.1 0.2,0.1 9.8));
            assert_relative_eq!(
                0.7996294886770354,
                perimeter_likelihood(&line, &boundary),
                max_relative = 1e-14
            );
        }
    }

    mod line_in_polygon {
        use super::*;

        // A small neighbourhood in Bristol
        fn bristol_neighborhood() -> Polygon {
            wkt!(POLYGON((1604.987378374668 710.1584994193722,1613.075872672461 741.9602923664602,1626.6675927392578 795.3895284188816,1633.7926759767377 825.5233951619371,1641.0286554513798 856.1354007500536,1647.6547056216293 884.156560969265,1654.9322711851996 912.6002624927714,1660.7612546508792 932.4819428385802,1662.0573544226795 941.3997882732328,1661.96032021517 946.8483472043511,1660.9830471252042 956.766948361739,1677.6036206694394 961.5705758278668,1686.08025179727 964.9064282343516,1697.1144273946143 967.630707700306,1745.2988424402636 924.1534313289417,1783.0104940891906 896.2212271741664,1803.7203663786565 880.4760038131888,1864.4221941936573 823.0771033966548,1884.4458960157158 806.4423193936846,1911.1926822155328 789.2404404820068,1926.8844997735414 774.1957461259958,1941.3772517672899 755.2147459303324,1947.1369250847356 743.1611992327944,1950.2004336362604 735.3219460763703,1953.3887004545697 724.5582623096029,1956.9443110584832 704.7544185200773,1959.2731320388307 679.5576133394093,1960.569231810631 665.0577748765891,1961.4356086634346 655.5505955169228,1962.1980202939183 643.7750365200568,1961.893055641737 626.0616602388577,1960.6731970329822 602.0991204489842,1959.6196827799708 589.7231080189508,1959.633544809593 583.7964102419814,1953.1599769654342 583.6963346701818,1941.654492360165 582.2063205950221,1925.505227823885 579.8045068615629,1873.6820299964354 570.2194909456139,1859.50117366971 568.4626086779349,1814.9070243020913 562.5581499172722,1805.0996383283436 561.9132184522819,1795.832871510724 561.7909038641756,1785.0828675210964 563.0585277786399,1766.5909199748157 570.2862079937436,1749.0415904443846 582.2174401031752,1741.9165072069045 588.8113083606518,1730.9793658171002 600.1087285124568,1723.9721098315936 610.0384491772079,1709.7565984307512 632.3219432564756,1701.2175881694975 643.5192878356911,1696.2619125714637 649.7128538043894,1693.2261280792138 653.3600524360058,1686.412940508757 659.1977941485393,1680.0571999165713 664.8909822566598,1654.5025482662033 682.7711511577877,1629.294447356975 696.715014219264,1624.331840744099 699.4615327007346,1604.987378374668 710.1584994193722)))
        }

        #[test]
        fn line_well_inside_boundary() {
            // Road #2944, Part of https://www.openstreetmap.org/way/655580962
            let line_string = wkt!(LINESTRING(1791.9930892990628 778.865939495864,1756.1042945484414 814.0369433731066,1716.3965106306534 778.6880273669917));

            let boundary = bristol_neighborhood();
            let prepared_polygon = PreparedGeometry::from(&boundary);
            let result = line_in_polygon(&line_string, &boundary, &prepared_polygon);
            assert_eq!(result, LineInPolygon::Inside);
        }

        #[test]
        fn line_well_outside_boundary() {
            let line_string = wkt!(LINESTRING(1942.4792831240406 995.0958925173832,1959.030546520042 997.5533037908187));

            let boundary = bristol_neighborhood();
            let prepared_polygon = PreparedGeometry::from(&boundary);
            let result = line_in_polygon(&line_string, &boundary, &prepared_polygon);
            assert_eq!(result, LineInPolygon::Outside);
        }

        #[test]
        fn inside_and_perpendicular_to_boundary_almost_touching_boundary() {
            let line_string = wkt!(LINESTRING(1641.0286554513798 856.1354007500536,1646.2130545385496 850.8091564062511,1660.3939108652753 836.2314813879377));

            let boundary = bristol_neighborhood();
            let prepared_polygon = PreparedGeometry::from(&boundary);
            let result = line_in_polygon(&line_string, &boundary, &prepared_polygon);
            assert_eq!(result, LineInPolygon::Inside);
        }

        // A neighborhood in Seattle
        fn portage_bay_neighborhood() -> Polygon {
            wkt!(POLYGON((1494.8879192632764 1156.4065954129007,1495.030280264711 1166.3140770613454,1495.1276851599255 1178.667850475072,1495.3224949514188 1205.7438525123148,1496.109226801229 1253.1574347241208,1496.2965439078196 1305.5414370219842,1496.5812659106891 1334.163050674363,1496.7386122808641 1343.8815006865716,1496.78356838602 1350.3864128804018,1496.761090333442 1354.5117503570798,1496.858495229721 1361.7505500795464,1497.420446550558 1456.043978118121,1497.6826905008495 1491.0593088835872,1497.6377343946288 1499.8214812056735,1497.5703002358305 1507.5384197739918,1495.8619682196775 1624.7602733560345,1495.8544755358398 1671.918106883475,1495.7570706395607 1688.4083372820328,1495.6671584281842 1719.4428841752342,1495.794534060879 1749.8102605871381,1497.278085549123 1769.2805191360155,1511.9637467430978 1769.3805947078151,1542.901040145765 1769.6029848685107,1553.6605347755903 1769.6474629003337,1567.90412759652 1769.725299456617,1581.1437007235688 1769.7920165047467,1590.4870780239487 1769.8364945373598,1599.583196742777 1769.8809725691826,1604.0713146277628 1769.9032115854895,1614.561072623459 1769.958809125466,1632.0789684763247 1770.0366456817487,1648.48045437029 1770.1144822372416,1657.9062511982088 1770.0255261735956,1681.8303921124464 1769.8253750292065,1711.096816920279 1769.2916386441689,1719.391218420557 1769.4250727404283,1724.0891314657758 1770.3479919060906,1730.8999814788747 1772.6163715425002,1739.4116708232234 1778.665383907576,1751.4748925185404 1788.6173435886337,1765.815890235749 1802.1275458368718,1781.8502346003709 1817.261196256529,1822.0110223545612 1811.034271763371,1906.8656718541479 1798.15788147276,1939.294009428897 1792.8761151615709,1917.0632151631369 1766.7452713062971,1900.661729268107 1747.0748616130306,1883.7207701058826 1726.6038473422077,1880.9859403422056 1723.379190015676,1878.3934715810285 1719.365047618951,1874.6396367547297 1711.5035554462202,1873.463285322998 1706.0438770069486,1873.1935486888685 1700.4730034869342,1873.485763375576 1633.5891627270407,1873.4333145855173 1624.6046002442586,1873.4932560594139 1617.2657249484123,1873.2385047940243 1553.417509878815,1873.0661730561737 1507.2493125651663,1873.0287096348557 1500.4330541469942,1873.0362023186935 1492.938505739372,1872.9687681598953 1370.9019051830478,1872.9687681598953 1362.640110721539,1872.9612754760572 1361.6727135240535,1871.1630312485281 1355.8349718115203,1869.3647870199336 1350.9535077891094,1867.1919085781583 1345.071288044753,1853.068198706085 1322.0094284042352,1844.661406940788 1308.2323579634783,1844.0994556199514 1307.309438797816,1842.9305968709923 1305.3968834175714,1837.2136787642164 1296.0342576615276,1814.1437038575227 1258.1945718583077,1812.7425718968175 1255.9150727137448,1808.2919174331498 1248.6762729904879,1804.118492287449 1241.8155365397026,1791.028772845673 1220.399364086911,1790.1970848907072 1219.031664599857,1783.2438738761737 1207.6564078925592,1782.487112763844 1206.4110229936116,1770.0117934340276 1185.8732916746592,1768.8279493173934 1184.0385728506974,1768.4608077869848 1183.349163353094,1751.9769023654808 1156.8736147498087,1748.230560224085 1150.946916972839,1743.8698179717937 1143.6302806940894,1677.0350741720133 1032.3573639045326,1674.974585994192 1028.9770334654347,1673.9331028789925 1029.3106187060832,1672.8466636575724 1029.2883796897768,1671.820165911113 1028.9325554328216,1670.9585072186667 1028.2765044596779,1670.3441071077716 1027.3758243095322,1670.0518924199994 1026.3305905557634,1670.1043412100576 1025.2408787693819,1670.493960793044 1024.2290035392832,1671.1832877465754 1023.3839209291136,1671.6703122247757 1023.0948137210783,1667.9089847157045 1016.8122916871537,1663.7355595700037 1009.9626747453119,1623.2001376015673 942.7452487439798,1617.8203902866546 933.849642325634,1612.717872289709 937.8415457052624,1608.4020861436386 941.1551590962304,1560.6062531052212 1030.9451863848656,1559.5048285161258 1032.9355783213932,1556.425335275683 1038.8845151138794,1518.6696991760793 1110.927807596856,1506.2243505826789 1134.8236303393896,1499.638281098554 1147.4442719456351,1494.8879192632764 1156.4065954129007)))
        }

        // See https://github.com/a-b-street/ltn/issues/67 - gwinn should be editable, but it's not considered
        #[test]
        fn missing_east_gwinn_in_portage_bay() {
            // https://www.openstreetmap.org/way/6475490
            let line_string = wkt!(LINESTRING(1496.109226801229 1253.1574347241208,1505.2428089413754 1253.1685542322741,1798.2891839156541 1253.3687053766632,1800.4021208835336 1252.3901886702342,1808.2919174331498 1248.6762729904879));

            let boundary = portage_bay_neighborhood();
            let prepared_polygon = PreparedGeometry::from(&boundary);
            let result = line_in_polygon(&line_string, &boundary, &prepared_polygon);

            // This is considered *crossing* not Inside, which is why it's not showing up as editable.
            assert_eq!(result, LineInPolygon::Inside);
        }

        #[test]
        fn part_of_harvard() {
            // way: https://www.openstreetmap.org/way/256916775
            let line_string = wkt!(LINESTRING(1495.3224949514188 1205.7438525123148,1495.1276851599255 1178.667850475072));

            let boundary = wkt!(POLYGON ((1495.030280264711 1166.3140770613454, 1495.1276851599255 1178.667850475072, 1495.3224949514188 1205.7438525123148, 1495.6671584281842 1719.4428841752342, 1878.3934715810285 1719.365047618951, 1872.9612754760572 1361.6727135240535, 1495.030280264711 1166.3140770613454)));

            let prepared_polygon = PreparedGeometry::from(&boundary);
            let result = line_in_polygon(&line_string, &boundary, &prepared_polygon);

            assert_eq!(result, LineInPolygon::Perimeter);
        }

        #[test]
        fn missing_segment_in_invalid_columbus() {
            // https://www.openstreetmap.org/way/1312473650 should be Perimeter but was Outside
            let line_string = wkt!(LINESTRING(750.1572646763966 1244.5286964975037,738.6251692907384 1246.0631886052765,712.705655565978 1249.5213556006574));
            let boundary = wkt!(POLYGON((555.3645758729823 964.2614967693613,556.7283122085399 973.6018835090985,558.1772820648426 983.6094407301326,562.524191633751 1013.7433074731879,563.8026944485632 1022.4165237316282,566.1039995147408 1037.4278595631795,569.5985738737743 1060.8900214924752,575.6501538624323 1101.9210060985572,577.0138901979898 1111.4837829982,578.4628600542925 1121.2689500593285,588.0090144007727 1185.984486754612,597.6404022704206 1251.478389012725,599.7712402938965 1265.8225543621893,625.3412965816635 1262.153116715056,663.0145128467466 1256.593362703195,698.6421246078877 1251.3671939319825,700.9434296740653 1251.1448037720768,712.705655565978 1249.4768775680443,738.6166459379372 1246.0298300808167,750.1231712676139 1244.4730989575273,752.5097098545369 1260.9299708324152,754.8962484414598 1276.6084771452634,762.0558642022282 1323.3104108431594,770.6644498192995 1380.6870722441763,772.1134196756022 1390.472239304515,773.3919224904147 1399.5902358835565,764.9538039148338 1416.9366683996468,763.3343670170407 1420.161325726969,756.5156853404642 1426.4994453000797,748.2480338063738 1429.7241026266115,743.6454236752297 1430.168882948003,748.2480338063738 1429.7241026266115,756.5156853404642 1426.4994453000797,763.3343670170407 1420.161325726969,764.9538039148338 1416.9366683996468,773.3919224904147 1399.5902358835565,772.1134196756022 1390.472239304515,770.6644498192995 1380.6870722441763,762.0558642022282 1323.3104108431594,754.8962484414598 1276.6084771452634,752.5097098545369 1260.9299708324152,750.1231712676139 1244.4730989575273,761.0330619508629 1242.805172754285,824.1058674622226 1232.686420453298,887.5196070565631 1223.0124484729122,898.0885636556199 1221.4557173496228,907.890418565547 1220.010181306286,971.6450922440796 1210.558599486596,960.0533333936577 1131.054117119667,964.4854764833111 1130.38694663837,995.1695440291161 1125.8279483484544,993.7205741728133 1115.8203911274202,987.9246947476023 1075.1229917619867,987.9246947476023 1070.452798392118,988.0951617890927 1069.5632377501256,989.7145986880973 1064.5594591396084,990.0555327710782 1060.1116559304355,990.2259998137799 1057.887754325849,989.5441316453954 1053.99592651802,1006.5908358380477 1051.4384396719952,1019.6315645447726 1049.5481233080573,1025.5979110126852 1044.6555397782831,1033.2689278991365 1043.321198814899,1031.9904250843242 1034.425592396553,1026.5354797433054 997.953606080387,1073.0729821876955 990.7259258644933,1113.8146052056632 984.3878062913824,1120.1218857577683 983.387050569437,1126.173465745215 982.4974899274443,1127.5372020807724 982.2750997667487,1134.6115843207958 981.1631489648506,1213.7935252929763 968.820495058487,1207.060077137145 923.6752924838806,1200.1561619386123 877.0845538659374,1198.7924256042659 867.7441671262003,1206.122508406525 866.5210212443492,1240.5568508748104 859.5157311891513,1239.278348059998 850.5089296900626,1230.073127796499 787.9060995186228,1227.1751880838933 768.224570317203,1224.106781328586 747.0975050732366,1278.6562347436195 738.7578740554447,1277.633432492254 731.4189987603884,1276.354929677442 721.8562218599554,1287.4352874021813 719.521125175416,1324.1709349366442 710.0695433557262,1366.4467613328713 699.6172058133006,1377.1009514526734 696.8373288073701,1383.2377649620767 695.2805976840807,1391.9315840998931 692.7231108388457,1447.6743068077826 679.7132864519751,1471.6249261977562 673.59755703877,1503.8431971206094 665.369121101721,1507.7639390857912 664.3683653797757,1514.92355484656 662.4780490150474,1609.7884636746237 638.1263264445492,1603.9925842494129 614.7753595952063,1601.3503451002541 606.9917039787586,1597.6000701777737 599.4304385230064,1592.0598913147985 591.9803681472074,1582.5989704890635 584.1967125307594,1577.3997257102803 580.6384699635792,1549.5283643563355 561.1793309228551,1498.558718821081 527.820806852478,1486.455558844976 546.0568000105611,1481.1710805454475 554.6188211890485,1478.3583743535871 559.956185040214,1468.215585359468 575.7458864330149,1402.6710077416515 572.1876438658346,1396.8751283152294 571.8540586251861,1383.7491660877592 570.7421078224979,1382.129729189966 597.6513172390214,1380.595525812918 626.3396479395299,1379.1465559566152 653.3600524360058,1371.0493714652266 654.5831983186472,1326.216539439375 661.0325129725009,1317.096552696621 662.3668539350947,1268.5986792702454 669.4833390702456,1267.8315775811159 664.5907555396811,1161.0339758184377 680.4916520132251,1122.6788913861815 686.1626011050391,1108.530126906135 688.2753076296726,1104.9503190263563 688.8312830310169,1056.1967450365337 696.0589632461204,1038.63863971941 698.6164500913553,961.1613691657683 710.0695433557262,948.9729756689181 711.8486646389213,941.7281263874044 712.9606154416097,934.0571095009532 714.0725662435077,894.0825881696925 720.0771005767601,883.2579310084 721.6338317000498,866.0407597742574 724.191318545285,848.3974209351773 726.7488053905199,834.5895905393228 728.8615119151534,825.2139032343331 730.1958528777473,820.1851254970405 696.3925484867689,809.4457018564932 697.9492796100585,746.3728963463449 707.2896663497957,741.8555197347349 707.9568368310925,683.1296237934947 716.6300530895328,672.2197331102458 718.2979792927752,661.7360100319344 719.8547104160646,652.1898556842428 721.1890513786586,660.6279742598238 785.4598077533406,662.3326446783622 795.9121452957662,646.5644433005223 798.2472419803054,618.2669143428515 802.4726550295727,619.8863512406448 813.2585778118565,621.3353210969475 823.0437448729851,616.8179444853374 823.7109153542821,619.7158841991543 843.281249474959,622.1024227848659 859.7381213498469,624.3184943302982 877.3069440266331,591.5035887610173 882.1995275571974,569.7690409152648 885.5353799636821,558.7739167112705 887.0921110869716,544.2842181482429 889.3160126915581,553.7451389751891 953.141988745639,555.3645758729823 964.2614967693613)));

            {
                let prepared_boundary = PreparedGeometry::from(&boundary);
                let result = line_in_polygon(&line_string, &boundary, &prepared_boundary);

                // NOTE: This isn't *desirable* but documents how giving an invalid polygon can produce
                // invalid results. Specifically, an invalid "spike" near the line_string can break our "perimeter" check.
                assert_eq!(result, LineInPolygon::Outside);
            }

            {
                use geo::BooleanOps;
                let mut valid_boundary = boundary.union(&boundary);
                let prepared_valid_boundary = PreparedGeometry::from(&boundary);
                assert_eq!(valid_boundary.0.len(), 1);
                let valid_boundary = valid_boundary.0.pop().unwrap();
                let result =
                    line_in_polygon(&line_string, &valid_boundary, &prepared_valid_boundary);
                // With a valid boundary polygon, we get the result we expect
                assert_eq!(result, LineInPolygon::Perimeter);
            }
        }
    }
}
