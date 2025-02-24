use geo::{Area, BooleanOps, MultiPolygon, Point, Polygon, PreparedGeometry, Relate};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct BoundaryStats {
    pub area_km2: f64,
    pub simd: f64,
    pub number_stats19_collisions: usize,
}

impl BoundaryStats {
    pub fn new(polygon: &Polygon, context_data: Option<&PreparedContextData>) -> Self {
        // Use unsigned area to ignore polygon orientation.
        let area_meters = polygon.unsigned_area();

        let mut simd = 0.0;
        let mut number_stats19_collisions = 0;
        if let Some(context_data) = context_data {
            // TODO: review this methodology
            // area proportional accumulation:
            // If  3/4 the boundary is in a zone with simd 100:   0.75 * 100 = 75
            // and 1/4 the boundary is in a zone with  simd 40:  +0.25 *  40 = 10
            //                  than the computed simd will be:          sum = 85
            // TODO: maybe also introduce an overall RTree to get intersection candidates quickly
            for population_zone in &context_data.population_zones {
                if population_zone
                    .prepared_geometry
                    .relate(polygon)
                    .is_intersects()
                {
                    let overlap = polygon.intersection(&population_zone.population_zone.geometry);
                    let overlap_ratio = overlap.unsigned_area() / area_meters;
                    simd += overlap_ratio * population_zone.population_zone.imd_percentile as f64
                }
            }

            // TODO Maybe buffer the polygon
            let polygon_prepared = PreparedGeometry::from(polygon.clone());
            for pt in &context_data.stats19_collisions {
                if polygon_prepared.relate(pt).is_contains() {
                    number_stats19_collisions += 1;
                }
            }
        }

        Self {
            area_km2: area_meters / 1_000_000.0,
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

/// After deserializing `ContextData`, make it faster for `BoundaryStats` to query.
pub struct PreparedContextData {
    pub population_zones: Vec<PreparedPopulationZone>,
    pub stats19_collisions: Vec<Point>,
}

/// Note when we deserialize this entity it will be in WGS84, but we should immedately
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
    pub prepared_geometry: PreparedGeometry<'static>,
}
