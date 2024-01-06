use geo::{BoundingRect, Coord, HaversineLength, LineString, MapCoords, MapCoordsInPlace, Rect};

/// Projects WGS84 points onto a Euclidean plane, using a Mercator projection. The top-left is (0,
/// 0) and grows to the right and down (screen-drawing order, not Cartesian), with units of meters.
/// The accuracy of this weakens for larger areas.
pub struct Mercator {
    pub wgs84_bounds: Rect,
    width: f64,
    height: f64,
}

#[allow(dead_code)]
impl Mercator {
    // TODO The API is kind of annoying, or wasteful. Do builder style.
    /// Create a boundary covering some geometry
    pub fn from<T: BoundingRect<f64>>(geometry: T) -> Option<Self> {
        let wgs84_bounds = geometry.bounding_rect().into()?;
        let width = LineString::from(vec![
            (wgs84_bounds.min().x, wgs84_bounds.min().y),
            (wgs84_bounds.max().x, wgs84_bounds.min().y),
        ])
        .haversine_length();
        let height = LineString::from(vec![
            (wgs84_bounds.min().x, wgs84_bounds.min().y),
            (wgs84_bounds.min().x, wgs84_bounds.max().y),
        ])
        .haversine_length();
        Some(Self {
            wgs84_bounds,
            width,
            height,
        })
    }

    pub fn pt_to_mercator(&self, pt: Coord) -> Coord {
        let x = self.width * (pt.x - self.wgs84_bounds.min().x) / self.wgs84_bounds.width();
        // Invert y, so that the northernmost latitude is 0
        let y = self.height
            - self.height * (pt.y - self.wgs84_bounds.min().y) / self.wgs84_bounds.height();
        Coord { x, y }
    }

    pub fn pt_to_wgs84(&self, pt: Coord) -> Coord {
        let x = self.wgs84_bounds.min().x + (pt.x / self.width * self.wgs84_bounds.width());
        let y = self.wgs84_bounds.min().y
            + (self.wgs84_bounds.height() * (self.height - pt.y) / self.height);
        Coord { x, y }
    }

    pub fn to_mercator<G: MapCoords<f64, f64, Output = G>>(&self, geom: &G) -> G {
        geom.map_coords(|pt| self.pt_to_mercator(pt))
    }

    pub fn to_wgs84<G: MapCoords<f64, f64, Output = G>>(&self, geom: &G) -> G {
        geom.map_coords(|pt| self.pt_to_wgs84(pt))
    }

    pub fn to_mercator_in_place<G: MapCoordsInPlace<f64>>(&self, geom: &mut G) {
        geom.map_coords_in_place(|pt| self.pt_to_mercator(pt));
    }

    pub fn to_wgs84_in_place<G: MapCoordsInPlace<f64>>(&self, geom: &mut G) {
        geom.map_coords_in_place(|pt| self.pt_to_wgs84(pt));
    }
}
