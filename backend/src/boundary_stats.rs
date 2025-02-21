use geo::{Area, Polygon};
use nanorand::{RandomGen, WyRand};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct BoundaryStats {
    pub area_km2: f64,
    pub simd: f64,
}

impl BoundaryStats {
    pub fn new(polygon: &Polygon) -> Self {
        // Convert from m^2 to km^2. Use unsigned area to ignore polygon orientation.
        let area_km2 = polygon.unsigned_area() / 1_000_000.0;
        // TODO: SIMD
        let mut rng = WyRand::new_seed((area_km2 * 1000000.0) as u64);
        let simd = f64::random(&mut rng) * 100.0;
        Self { area_km2, simd }
    }
}
