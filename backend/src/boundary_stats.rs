use geo::{Area, BooleanOps, Buffer, MultiPolygon, Point, Polygon, PreparedGeometry, Relate};
use i_overlay::mesh::style::{LineJoin, OutlineStyle};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct BoundaryStats {
    pub area_km2: f64,
    // floating point / partial people seems silly, but because we accumulate them in an area proportional manner, any rounding
    // can have weird effects, especially for very small blocks.
    pub population: f64,
    pub simd: f64,
    pub number_stats19_collisions: u32,
}

impl BoundaryStats {
    pub fn new(polygon: &Polygon, context_data: Option<&PreparedContextData>) -> Self {
        // Use unsigned area to ignore polygon orientation.
        let area_meters = polygon.unsigned_area();

        let mut simd = 0.0;
        let mut number_stats19_collisions = 0;
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
                if population_zone
                    .prepared_geometry
                    .relate(&prepared_polygon)
                    .is_intersects()
                {
                    let overlap = polygon.intersection(&population_zone.population_zone.geometry);
                    let overlap_area = overlap.unsigned_area();
                    let ratio_in_boundary = overlap_area / area_meters;
                    simd +=
                        ratio_in_boundary * population_zone.population_zone.imd_percentile as f64;

                    // PERF cache this area calculation on the prepared population zone
                    let ratio_in_population_zone =
                        overlap_area / population_zone.prepared_geometry.geometry().unsigned_area();
                    population += ratio_in_population_zone
                        * population_zone.population_zone.population as f64;
                }
            }

            // Counting incidents in a given neighbourhood, we want to include incidents that happen
            // *on* the perimeter road, not just the interior.
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
        }

        Self {
            area_km2: area_meters / 1_000_000.0,
            population,
            simd,
            number_stats19_collisions,
        }
    }
}

/// Contextual data for a study area, used for calculating statistics about a neighbourhood
/// boundary. This is directly serialized and deserialized.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContextData {
    pub population_zones: Vec<PopulationZone>,
    pub stats19_collisions: Vec<Point>,
}

impl ContextData {
    pub fn into_prepared(self) -> PreparedContextData {
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
            population_zones,
            stats19_collisions: self.stats19_collisions,
        }
    }
}

/// After deserializing `ContextData`, make it faster for `BoundaryStats` to query.
pub struct PreparedContextData {
    pub population_zones: Vec<PreparedPopulationZone>,
    pub stats19_collisions: Vec<Point>,
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
}

pub struct PreparedPopulationZone {
    pub population_zone: PopulationZone,
    pub prepared_geometry: PreparedGeometry<'static, MultiPolygon>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    use geo::wkt;

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
        let c = wkt!(POLYGON((0. 0.,10. 0.,10. 3.,0. 3.)));

        let zone_1 = PopulationZone {
            geometry: a,
            imd_percentile: 12,
            population: 100,
        };

        let zone_2 = PopulationZone {
            geometry: b,
            imd_percentile: 60,
            population: 10000,
        };

        let context_data = ContextData {
            population_zones: vec![zone_1, zone_2],
            stats19_collisions: vec![],
        }
        .into_prepared();

        // [backend/src/boundary_stats.rs:176:9] a.unsigned_area() = 30.0
        // [backend/src/boundary_stats.rs:177:9] b.unsigned_area() = 20.0
        // [backend/src/boundary_stats.rs:178:9] c.unsigned_area() = 30.0
        // [backend/src/boundary_stats.rs:180:9] a.intersection(&c).unsigned_area() = 18.0
        // [backend/src/boundary_stats.rs:181:9] b.intersection(&c).unsigned_area() = 12.0

        let boundary_stats = BoundaryStats::new(&c, Some(&context_data));
        assert_relative_eq!(boundary_stats.area_km2, 3e-5);
        assert_relative_eq!(boundary_stats.population, 60. + 6000.);
        assert_relative_eq!(boundary_stats.simd, 7.2 + 24.);
        assert_eq!(boundary_stats.number_stats19_collisions, 0);
    }
}
