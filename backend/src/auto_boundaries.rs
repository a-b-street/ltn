use crate::boundary_stats::BoundaryStats;
use crate::geo_helpers::buffer_polygon;
use crate::{MapModel, NeighbourhoodBoundary, NeighbourhoodDefinition};
use anyhow::Result;
use geo::{Coord, Intersects, LineString, MultiPolygon, Polygon};
use geojson::{Feature, FeatureCollection};
use i_overlay::core::fill_rule::FillRule;
use i_overlay::float::slice::FloatSlice;
use serde::{Deserialize, Serialize};

impl MapModel {
    pub fn generated_boundaries(&self) -> FeatureCollection {
        let mut features = Vec::new();
        let mut severances = Vec::new();
        let mut road_severances = Vec::new();

        for road in &self.roads {
            if road.is_severance() {
                severances.push(road.linestring.clone());
                road_severances.push(road.linestring.clone());
            }
        }

        for linestring in &self.railways {
            severances.push(linestring.clone());
        }

        for linestring in &self.waterways {
            severances.push(linestring.clone());
        }

        let boundary_mercator = self.mercator.to_mercator(&self.boundary_wgs84);
        for polygon in boundary_mercator
            .into_iter()
            .flat_map(|boundary_polygon| split_polygon(boundary_polygon, &severances))
        {
            // TODO This is expensive; could this info somehow be retained?
            let touches_big_road = boundary_touches_any(&polygon, &road_severances);
            let touches_railway = boundary_touches_any(&polygon, &self.railways);
            let touches_waterway = boundary_touches_any(&polygon, &self.waterways);

            let boundary_stats = BoundaryStats::new(&polygon, self.context_data.as_ref());
            let generated_boundary = GeneratedBoundary {
                geometry: polygon,
                touches_big_road,
                touches_railway,
                touches_waterway,
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
    pub touches_big_road: bool,
    pub touches_railway: bool,
    pub touches_waterway: bool,
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
fn split_polygon(polygon: Polygon, linestrings: &Vec<LineString>) -> Vec<Polygon> {
    let mut shape = to_i_overlay_contour(polygon.exterior());

    // geo Polygon's are explicitly closed LineStrings, but i_overlay Polygon's are not.
    shape.pop();

    let splitters: Vec<_> = linestrings.iter().map(to_i_overlay_contour).collect();
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

fn boundary_touches_any(polygon: &Polygon, linestrings: &Vec<LineString>) -> bool {
    // TODO At least consider an rtree to prune!
    linestrings
        .iter()
        .any(|ls| ls.intersects(polygon.exterior()))
}
