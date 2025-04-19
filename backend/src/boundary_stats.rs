use geo::{Area, BooleanOps, Buffer, MultiPolygon, Point, Polygon, PreparedGeometry, Relate};
use i_overlay::mesh::style::{LineJoin, OutlineStyle};
use serde::{Deserialize, Serialize};
use utils::Mercator;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct BoundaryStats {
    pub area_km2: f64,
    // floating point / partial people seems silly, but because we accumulate them in an area proportional manner, any rounding
    // can have weird effects, especially for very small blocks.
    pub population: f64,
    pub simd: f64,
    pub number_stats19_collisions: u32,
    pub number_pois: u32,
    pub total_households: u32,
    pub households_with_cars_or_vans: u32,
    pub combined_score: f64,
}

impl BoundaryStats {
    pub fn new(polygon: &Polygon, context_data: Option<&PreparedContextData>) -> Self {
        // Use unsigned area to ignore polygon orientation.
        let area_meters = polygon.unsigned_area();

        let mut simd = 0.0;
        let mut number_stats19_collisions = 0;
        let mut number_pois = 0;
        let mut total_households = 0.0;
        let mut households_with_cars_or_vans = 0.0;
        let mut population = 0.0;
        if let Some(context_data) = context_data {
            let prepared_polygon = PreparedGeometry::from(polygon);

            // TODO: review this methodology
            // area proportional accumulation:
            // If  3/4 the boundary is in a zone with simd 100:   0.75 * 100 = 75
            // and 1/4 the boundary is in a zone with  simd 40:  +0.25 *  40 = 10
            //                  than the computed simd will be:          sum = 85
            // TODO: maybe also introduce an overall RTree to get intersection candidates quickly
            for population_zone in &context_data.population_zones {
                let de9im = population_zone.prepared_geometry.relate(&prepared_polygon);
                if de9im.is_intersects() {
                    let overlap_area = if de9im.is_within() {
                        // population zone is entirely inside the boundary.
                        population_zone.population_zone.geometry.unsigned_area()
                    } else if de9im.is_contains() {
                        // boundary is entirely inside the population zone
                        area_meters
                    } else {
                        // only partial overlap
                        let overlap =
                            polygon.intersection(&population_zone.population_zone.geometry);
                        overlap.unsigned_area()
                    };

                    let ratio_in_boundary = overlap_area / area_meters;
                    simd +=
                        ratio_in_boundary * population_zone.population_zone.imd_percentile as f64;

                    // PERF cache this area calculation on the prepared population zone
                    let ratio_in_population_zone =
                        overlap_area / population_zone.prepared_geometry.geometry().unsigned_area();
                    population += ratio_in_population_zone
                        * population_zone.population_zone.population as f64;

                    total_households += ratio_in_population_zone
                        * population_zone.population_zone.total_households as f64;
                    households_with_cars_or_vans += ratio_in_population_zone
                        * population_zone.population_zone.households_with_cars_or_vans as f64;
                }
            }

            // When counting stats19 collisions in a given neighbourhood, we want to include
            // collisions that happen *on* the perimeter road, not just the interior. Similarly,
            // POIs roughly on the other side of the perimeter road are still relevant. An LTN is
            // likely to receive some kind of attention to the perimeter road itself too.
            //
            // Our road LineString's are center-lines, so to count incidents *on* the perimeter,
            // conceptually we need a buffer at least half a road-width beyond our perimeter.
            //
            // To make the analysis reasonable, we only provide one uniform buffer amount.
            // We can't reasonably buffer each road segment based on the "actual" road width of that
            // segment.
            //
            // So how wide should we buffer?
            //
            // Extending beyond the perimeter road should include relatively fewer "extra"
            // incidents, whereas clipping off part of the perimeter road itself (by buffering too small)
            // will more likely clip off relatively more incidents.
            //
            // Erring towards "a bit too big" will give more accurate results than
            // "a bit too small".
            //
            // Conclusion: To count incidents on the perimeter, we should buffer a bit more than 1/2
            // the expected road width.
            let buffer_meters = 10.0;
            let style = OutlineStyle::new(buffer_meters).line_join(LineJoin::Bevel);
            let buffered_polygon = polygon.buffer_with_style(style);
            let prepared_buffered_polygon = PreparedGeometry::from(&buffered_polygon);
            for pt in &context_data.stats19_collisions {
                if prepared_buffered_polygon.relate(pt).is_contains() {
                    number_stats19_collisions += 1;
                }
            }
            for poi in &context_data.pois {
                if prepared_buffered_polygon.relate(&poi.point).is_contains() {
                    number_pois += 1;
                }
            }
        }

        let mut stats = Self {
            area_km2: area_meters / 1_000_000.0,
            population,
            simd,
            number_stats19_collisions,
            number_pois,
            total_households: total_households.round() as u32,
            households_with_cars_or_vans: households_with_cars_or_vans.round() as u32,
            combined_score: 0.0,
        };

        if let Some(context_data) = context_data {
            stats.combined_score = calculate_combined_score(context_data, &stats);
        }

        stats
    }
}

/// Contextual data for a study area, used for calculating statistics about a neighbourhood
/// boundary. This is directly serialized and deserialized.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContextData {
    pub settlements: MultiPolygon,
    pub population_zones: Vec<PopulationZone>,
    pub stats19_collisions: Vec<Point>,
    pub pois: Vec<POI>,
    pub metric_buckets: MetricBuckets,
}

/// Precalculated buckets to classify metrics
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MetricBuckets {
    pub population_density: [usize; 6],
    pub collision_density: [usize; 6],
    pub poi_density: [usize; 6],
}

impl ContextData {
    pub fn into_prepared(mut self, mercator: &Mercator) -> PreparedContextData {
        for population_zone in &mut self.population_zones {
            mercator.to_mercator_in_place(&mut population_zone.geometry);
        }
        for stats19_collision in &mut self.stats19_collisions {
            mercator.to_mercator_in_place(stats19_collision);
        }
        for poi in &mut self.pois {
            mercator.to_mercator_in_place(&mut poi.point);
        }
        mercator.to_mercator_in_place(&mut self.settlements);

        let population_zones = self
            .population_zones
            .into_iter()
            .map(|population_zone| {
                PreparedPopulationZone {
                    // PERF: remove this clone, and use new prepared_geometry generics to get
                    // at inner geometry, move the other (non-geoemtry) fields from population_zone
                    // directly onto PreparedPopulationZones
                    prepared_geometry: PreparedGeometry::from(population_zone.geometry.clone()),
                    population_zone,
                }
            })
            .collect();

        PreparedContextData {
            settlements: self.settlements.into(),
            population_zones,
            stats19_collisions: self.stats19_collisions,
            pois: self.pois,
            metric_buckets: self.metric_buckets,
        }
    }
}

/// After deserializing `ContextData`, make it faster for `BoundaryStats` to query.
pub struct PreparedContextData {
    pub settlements: PreparedGeometry<'static, MultiPolygon>,
    pub population_zones: Vec<PreparedPopulationZone>,
    pub stats19_collisions: Vec<Point>,
    pub pois: Vec<POI>,
    pub metric_buckets: MetricBuckets,
}

/// Note when we deserialize this entity it will be in WGS84, but we should immediately
/// project it to our map mercator.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PopulationZone {
    pub geometry: MultiPolygon,

    // "id": "S01006506",
    // (unused)

    // "imd_rank": 4691,
    // (unused)

    // "imd_percentile": 68,
    pub imd_percentile: u8,

    // "population": 894,
    pub population: u32,

    // "area": 4388802.1221970674
    // (unused - though maybe we would find it helpful for pre-computing density or to save the cost of calculating area live)
    pub total_households: u32,
    pub households_with_cars_or_vans: u32,
}

pub struct PreparedPopulationZone {
    pub population_zone: PopulationZone,
    pub prepared_geometry: PreparedGeometry<'static, MultiPolygon>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct POI {
    pub point: Point,
    pub kind: POIKind,
    pub name: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum POIKind {
    CommunityCenter,
    GP,
    Grocery,
    Hospital,
    School,
    Recreation,
}

fn calculate_combined_score(context: &PreparedContextData, stats: &BoundaryStats) -> f64 {
    // Turn each metric into 1 (least important) to 5 (most important)
    let population = find_bucket(
        stats.population / stats.area_km2,
        &context.metric_buckets.population_density,
    );
    let collisions = find_bucket(
        (stats.number_stats19_collisions as f64) / stats.area_km2,
        &context.metric_buckets.collision_density,
    );
    let pois = find_bucket(
        (stats.number_pois as f64) / stats.area_km2,
        &context.metric_buckets.poi_density,
    );

    // SIMD is [1, 100], with the [1, 20] group being the most important
    let simd = find_bucket(100.0 - stats.simd, &[0, 20, 40, 60, 80, 100]);

    let percent_with_cars = if stats.total_households > 0 {
        100.0 * (stats.households_with_cars_or_vans as f64) / (stats.total_households as f64)
    } else {
        0.0
    };
    // Lowest percentage of cars is the most important
    let car_ownership = find_bucket(100.0 - percent_with_cars, &[0, 20, 40, 60, 80, 100]);

    // Average
    (population + collisions + pois + simd + car_ownership) / 5.0
}

fn find_bucket(value: f64, buckets: &[usize; 6]) -> f64 {
    let value = value as usize;

    // First value is always 0
    if value < buckets[1] {
        1.0
    } else if value < buckets[2] {
        2.0
    } else if value < buckets[3] {
        3.0
    } else if value < buckets[4] {
        4.0
    } else {
        // buckets[5] may not be the true upper limit. The buckets are calculated from
        // PopulationZones, but BoundaryStats could be built from smaller areas that achieve even
        // higher density.
        5.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    use geo::{wkt, Rect};

    #[test]
    fn aggregation() {
        //
        //            (6, 5)
        // (0, 5) +------+----+ (10, 5)
        //        |aaaaaa|bbbb|
        //        |aaaaaa|bbbb|
        // (0, 3) +------+----+ (10, 3)
        //        |acacac|bcbc|
        //        |acacac|bcbc|
        //        |acacac|bcbc|
        // (0, 0) +------+----| (10, 0)
        //            (6, 0)

        // adjacent population zones
        let a = wkt!(MULTIPOLYGON(((0. 0.,6. 0.,6. 5.,0. 5.))));
        let b = wkt!(MULTIPOLYGON(((6. 0.,10. 0.,10. 5.,6. 5.))));

        // neighbourhood boundary covered by parts of `a` and `b`
        let mut c = wkt!(POLYGON((0. 0.,10. 0.,10. 3.,0. 3.)));

        let zone_1 = PopulationZone {
            geometry: a,
            imd_percentile: 12,
            population: 100,
            total_households: 10,
            households_with_cars_or_vans: 9,
        };

        let zone_2 = PopulationZone {
            geometry: b,
            imd_percentile: 60,
            population: 10000,
            total_households: 990,
            households_with_cars_or_vans: 99,
        };

        let mercator = Mercator::from(Rect::new((0., 0.), (10., 10.))).unwrap();
        let context_data = ContextData {
            settlements: wkt!(MULTIPOLYGON EMPTY),
            population_zones: vec![zone_1, zone_2],
            stats19_collisions: vec![],
            pois: vec![],
            metric_buckets: MetricBuckets::default(),
        }
        .into_prepared(&mercator);

        mercator.to_mercator_in_place(&mut c);

        // [backend/src/boundary_stats.rs:176:9] a.unsigned_area() = 30.0
        // [backend/src/boundary_stats.rs:177:9] b.unsigned_area() = 20.0
        // [backend/src/boundary_stats.rs:178:9] c.unsigned_area() = 30.0
        // [backend/src/boundary_stats.rs:180:9] a.intersection(&c).unsigned_area() = 18.0
        // [backend/src/boundary_stats.rs:181:9] b.intersection(&c).unsigned_area() = 12.0

        let boundary_stats = BoundaryStats::new(&c, Some(&context_data));
        assert_relative_eq!(boundary_stats.area_km2, 370930., epsilon = 1.0);
        assert_relative_eq!(boundary_stats.population, 60. + 6000., epsilon = 1e-5);
        assert_relative_eq!(boundary_stats.simd, 7.2 + 24., epsilon = 1e-5);
        assert_eq!(boundary_stats.number_stats19_collisions, 0);
        assert_eq!(boundary_stats.number_pois, 0);
        assert_eq!(boundary_stats.total_households, 600);
        assert_eq!(boundary_stats.households_with_cars_or_vans, 65);
    }

    #[test]
    fn test_find_bucket() {
        let buckets = [0, 1_100, 2_600, 3_900, 5_450, 12_000];
        assert_eq!(1.0, find_bucket(50.0, &buckets));
        // Right on the boundary
        assert_eq!(2.0, find_bucket(1_100.0, &buckets));
        assert_eq!(3.0, find_bucket(3_000.0, &buckets));
        assert_eq!(5.0, find_bucket(10_000.0, &buckets));
        assert_eq!(5.0, find_bucket(10_000.0, &buckets));
        // Even higher than the last threshold
        assert_eq!(5.0, find_bucket(13_000.0, &buckets));
    }
}
