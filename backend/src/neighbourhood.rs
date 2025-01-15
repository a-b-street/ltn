use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use geo::{Area, Distance, Euclidean, Length, Line, LineString, Polygon, PreparedGeometry, Relate};
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
    // Just debug, has no actual use
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
        let t1 = Instant::now();
        let bbox = buffer_aabb(aabb(&boundary_polygon), 50.0);

        let prepared_boundary = PreparedGeometry::from(&boundary_polygon);

        let mut interior_roads = BTreeSet::new();
        let mut crosses = BTreeMap::new();
        for obj in map.closest_road.locate_in_envelope_intersecting(&bbox) {
            let r = &map.roads[obj.data.0];

            match line_in_polygon(&r.linestring, &boundary_polygon, &prepared_boundary) {
                LineInPolygon::Inside => {
                    interior_roads.insert(r.id);
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

        // Convert from m^2 to km^2. Use unsigned area to ignore polygon orientation.
        let boundary_area_km2 = boundary_polygon.unsigned_area() / 1_000_000.0;
        let t3 = Instant::now();

        if true {
            info!("Neighbourhood set up, total {:?}. Finding roads took {:?}, intersections took {:?}", t3 - t1, t2 - t1, t3 - t2);
        }

        let mut n = Self {
            interior_roads,
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
                .chain(self.crosses.keys())
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
    Crosses { percent: f64 },
    Outside,
}

fn line_in_polygon(
    linestring: &LineString,
    polygon: &Polygon,
    prepared_polygon: &PreparedGeometry,
) -> LineInPolygon {
    // TODO Reconsider rewriting all of this logic based on clip_linestring_to_polygon

    let matrix = prepared_polygon.relate(linestring);

    if matrix.is_within() {
        return LineInPolygon::Inside;
    }

    if !matrix.is_intersects() {
        return LineInPolygon::Outside;
    }

    // Clip the linestring to the polygon
    // Multiple segments generally don't happen, but might right on a boundary
    let mut sum = 0.0;
    for clipped in clip_linestring_to_polygon(linestring, polygon) {
        sum += clipped.length::<Euclidean>();
    }

    // How much of the clipped linestring is inside the boundary? If it's nearly 1, then this
    // road is interior. Round to make diffs less noisy.
    let percent = (sum / linestring.length::<Euclidean>() * 10e3).round() / 10e3;
    if percent > 0.99 {
        LineInPolygon::Inside
    } else {
        LineInPolygon::Crosses { percent }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod line_in_polygon {
        use super::*;
        use geo::wkt;

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
        fn inside_but_almost_touching_boundary() {
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
    }
}
