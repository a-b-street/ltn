use geo::{Area, BooleanOps, Buffer, MultiPolygon, Point, Polygon, PreparedGeometry, Relate};
use i_overlay::mesh::style::{LineJoin, OutlineStyle};
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
                    let overlap_ratio = overlap.unsigned_area() / area_meters;
                    simd += overlap_ratio * population_zone.population_zone.imd_percentile as f64
                }
            }

            // Counting incidents in a given neighbourhood, we want to include incidents that happen
            // *on* the perimeter road, not just the interior.
            //
            // Our road LineString's are center-lines, so to count incidents *on* the perimeter,
            // conceptually we need a buffer at least half a road-width beyond our perimeter.
            //
            // To make the analysis reasonable, we only provide one uniform buffer amount.
            // We can't reasonably buffer each raod segment based on the "actual" road width of that
            // segment.
            //
            // So how wide should we buffer?
            //
            // Extending beyond the perimeter road should include relatively fewer "extra"
            // incidents, where as clipping off part of the perimeter road itself (by buffering too small)
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
