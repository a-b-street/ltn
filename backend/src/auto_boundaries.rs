use crate::boundary_stats::BoundaryStats;
use crate::geo_helpers::buffer_polygon;
use crate::{MapModel, NeighbourhoodBoundary, NeighbourhoodDefinition};
use anyhow::Result;
use geo::{Area, BoundingRect, Coord, LineString, MultiPolygon, Polygon};
use geojson::{Feature, FeatureCollection};
use i_overlay::core::fill_rule::FillRule;
use i_overlay::float::slice::FloatSlice;
use rstar::RTree;
use serde::{Deserialize, Serialize};

impl MapModel {
    pub fn generated_boundaries(&self) -> FeatureCollection {
        let mut features = Vec::new();
        let severances = self
            .roads
            .iter()
            .filter_map(|r| {
                if r.is_severance() {
                    Some(&r.linestring)
                } else {
                    None
                }
            })
            .chain(self.railways.iter())
            .chain(self.waterways.iter());

        let boundary_mercator = self.mercator.to_mercator(&self.boundary_wgs84);
        let severance_rtree = RTree::bulk_load(severances.cloned().collect());
        let multiple_boundary_polygons = boundary_mercator.0.len() > 1;

        for polygon in boundary_mercator.into_iter().flat_map(|boundary_polygon| {
            if multiple_boundary_polygons {
                let area_km_2 = boundary_polygon.unsigned_area() / 1000. / 1000.;
                if area_km_2 < 16.0 {
                    info!("skipping small island: {area_km_2} km^2");
                    return vec![];
                } else {
                    info!("Continuing with larger land mass: {area_km_2} km^2");
                }
            }

            let Some(envelope) = boundary_polygon.bounding_rect().map(|rect| {
                rstar::AABB::from_corners(geo::Point(rect.min()), geo::Point(rect.max()))
            }) else {
                debug_assert!(false, "boundary polygon was unexpectedly empty");
                return vec![];
            };
            let severance_candidates = severance_rtree.locate_in_envelope_intersecting(&envelope);
            split_polygon(boundary_polygon, severance_candidates)
        }) {
            let area_km_2 = polygon.unsigned_area() / 1000. / 1000.;

            // Discard small areas.
            //
            // We might want to tweak this threshold.
            //
            // In general, it's better to err on the side of a "too low", threshold, the downside of which is primarily the visual distraction of tiny irrelevant sliver areas.
            // Whereas having this number "too high" will potentially preclude more areas someone wants to choose.
            // .0025km (50m x 50m)
            let min_area_km_2 = 0.0025;
            if area_km_2 < min_area_km_2 {
                continue;
            }
            let boundary_stats = BoundaryStats::new(&polygon, self.context_data.as_ref());
            let generated_boundary = GeneratedBoundary {
                geometry: polygon,
                boundary_stats,
            };

            features.push(generated_boundary.to_feature(self));
        }

        FeatureCollection {
            features,
            bbox: None,
            foreign_members: None,
        }
    }

    pub fn generate_merged_boundary(
        &self,
        boundaries_to_merge: Vec<Polygon>,
    ) -> Result<NeighbourhoodBoundary> {
        let original_boundaries = MultiPolygon(boundaries_to_merge);

        // Merged boundaries must be adjacent, but it's important to allow a little slop,
        // because our severance-based boundary generation can insert tiny slivers between
        // neighbourhoods.
        let adjacency_tolerance = 15.0;
        // Note that buffering geometries will union them if the buffering results in overlap,
        // no need for an explicit `unary_union` step.
        let polygon = buffer_polygon(&original_boundaries, adjacency_tolerance)?;
        let (exterior, _interiors) = polygon.into_inner();
        let solid = Polygon::new(exterior, vec![]);
        let definition = NeighbourhoodDefinition {
            geometry: solid,
            name: "".to_string(),
            waypoints: None,
        };

        Ok(NeighbourhoodBoundary::new(
            definition,
            self.context_data.as_ref(),
        ))
    }
}

/// The static data that defines where exactly a neighbourhood is.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct GeneratedBoundary {
    /// `geometry` is always Mercator.
    /// We convert it to/from wgs84 only when serializizing/deserializing GeoJSON.
    #[serde(
        serialize_with = "geojson::ser::serialize_geometry",
        deserialize_with = "geojson::de::deserialize_geometry"
    )]
    pub geometry: Polygon,
    #[serde(flatten)]
    pub boundary_stats: BoundaryStats,
}

impl GeneratedBoundary {
    pub fn to_feature(&self, map: &MapModel) -> Feature {
        let mut projected = self.clone();
        map.mercator.to_wgs84_in_place(&mut projected.geometry);
        geojson::ser::to_feature(projected).expect("should have no unserializable fields")
    }
}

// TODO Revisit some of this; conversions are now in geo
fn split_polygon<'a>(
    polygon: Polygon,
    severances: impl Iterator<Item = &'a LineString>,
) -> Vec<Polygon> {
    let mut shape = to_i_overlay_contour(polygon.exterior());

    // geo Polygon's are explicitly closed LineStrings, but i_overlay Polygon's are not.
    shape.pop();

    let splitters: Vec<_> = severances.map(to_i_overlay_contour).collect();
    let shapes = shape.slice_by(&splitters, FillRule::NonZero);

    shapes
        .into_iter()
        .map(|rings| {
            let exterior = rings.into_iter().next().expect("shapes must be non-empty");
            let exterior_line_string = to_geo_linestring(exterior);
            // We ignore any interiors
            Polygon::new(exterior_line_string, vec![])
        })
        .collect()
}

fn to_geo_linestring(pts: Vec<[f64; 2]>) -> LineString {
    LineString(
        pts.into_iter()
            .map(|pt| Coord { x: pt[0], y: pt[1] })
            .collect(),
    )
}

fn to_i_overlay_contour(line_string: &LineString) -> Vec<[f64; 2]> {
    line_string.coords().map(|c| [c.x, c.y]).collect()
}
