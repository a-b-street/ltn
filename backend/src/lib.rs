#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

use std::sync::Once;

use self::boundary_stats::ContextData;
use self::cells::Cell;
pub use self::map_model::{
    FilterKind, Intersection, IntersectionID, MapModel, ModalFilter, Road, RoadID, TravelFlow,
};
pub use self::neighbourhood::{Neighbourhood, NeighbourhoodBoundary, NeighbourhoodDefinition};
use self::render_cells::RenderCells;
pub use self::route::Router;
pub use self::shortcuts::Shortcuts;
use geo::{Coord, LineString};
use geojson::{Feature, FeatureCollection, GeoJson, Geometry};
use serde::Deserialize;
use wasm_bindgen::prelude::*;

mod auto_boundaries;
pub mod boundary_stats;
mod cells;
mod create;
mod geo_helpers;
mod impact;
mod map_model;
mod movements;
mod neighbourhood;
pub mod od;
#[cfg(test)]
mod osm_tests;
mod render_cells;
mod route;
mod route_snapper;
mod shortcuts;
// TODO: We could hide this behind a feature flag - it's used by both tests and benches
pub mod test_fixtures;
#[cfg(test)]
mod tests;

static START: Once = Once::new();

#[wasm_bindgen]
pub struct LTN {
    map: MapModel,
    // TODO Stateful, synced with the UI. Weird?
    neighbourhood: Option<Neighbourhood>,
}

#[wasm_bindgen]
impl LTN {
    /// Call with bytes of an osm.pbf or osm.xml string
    #[wasm_bindgen(constructor)]
    pub fn new(
        input_bytes: &[u8],
        // Option doesn't work; the caller should just pass in 0 bytes to mean empty
        demand_bytes: &[u8],
        context_data_bytes: &[u8],
        boundary_input: JsValue,
        study_area_name: Option<String>,
    ) -> Result<LTN, JsValue> {
        // Panics shouldn't happen, but if they do, console.log them.
        console_error_panic_hook::set_once();
        START.call_once(|| {
            console_log::init_with_level(log::Level::Info).unwrap();
        });

        let boundary: Feature = serde_wasm_bindgen::from_value(boundary_input)?;
        let boundary_geom: geo::Geometry = boundary.try_into().map_err(err_to_js)?;
        let multi_polygon = match boundary_geom {
            geo::Geometry::Polygon(p) => p.into(),
            geo::Geometry::MultiPolygon(mp) => mp,
            _ => {
                return Err(JsValue::from_str("unexpected boundary geometry type"));
            }
        };

        let mut demand = None;
        if demand_bytes.len() > 0 {
            demand = Some(bincode::deserialize(demand_bytes).map_err(err_to_js)?);
        }

        let context_data: Option<ContextData> = if context_data_bytes.len() > 0 {
            Some(bincode::deserialize(context_data_bytes).map_err(err_to_js)?)
        } else {
            None
        };

        let map = MapModel::new(
            input_bytes,
            multi_polygon,
            study_area_name,
            demand,
            context_data,
        )
        .map_err(err_to_js)?;
        Ok(LTN {
            map,
            neighbourhood: None,
        })
    }

    #[wasm_bindgen(js_name = getInvertedBoundary)]
    pub fn get_inverted_boundary(&self) -> Result<String, JsValue> {
        let f = Feature::from(Geometry::from(&self.map.invert_study_area_boundary()));
        let out = serde_json::to_string(&f).map_err(err_to_js)?;
        Ok(out)
    }

    #[wasm_bindgen(js_name = getBounds)]
    pub fn get_bounds(&self) -> Vec<f64> {
        let b = &self.map.mercator.wgs84_bounds;
        vec![b.min().x, b.min().y, b.max().x, b.max().y]
    }

    #[wasm_bindgen(js_name = toRouteSnapper)]
    pub fn to_route_snapper(&self) -> Vec<u8> {
        let graph = self.map.to_route_snapper_graph();
        bincode::serialize(&graph).unwrap()
    }

    #[wasm_bindgen(js_name = toRouteSnapperGj)]
    pub fn to_route_snapper_gj(&self) -> Result<String, JsValue> {
        let graph = self.map.to_route_snapper_graph();

        let mut features = Vec::new();
        for (idx, edge) in graph.edges.iter().enumerate() {
            let mut f = Feature::from(Geometry::from(&edge.geometry));
            f.set_property("edge_id", idx);
            f.set_property("node1", edge.node1.0);
            f.set_property("node2", edge.node2.0);
            f.set_property("length_meters", edge.length_meters);
            f.set_property("name", edge.name.clone());
            features.push(f);
        }
        for (idx, pt) in graph.nodes.iter().enumerate() {
            let mut f = Feature::from(Geometry::from(&geo::Point::from(*pt)));
            f.set_property("node_id", idx);
            features.push(f);
        }
        let gj =
            geojson::GeoJson::from(features.into_iter().collect::<geojson::FeatureCollection>());
        Ok(serde_json::to_string(&gj).map_err(err_to_js)?)
    }

    #[wasm_bindgen(js_name = renderModalFilters)]
    pub fn render_modal_filters(&self) -> Result<String, JsValue> {
        Ok(serde_json::to_string(&self.map.filters_to_gj()).map_err(err_to_js)?)
    }

    #[wasm_bindgen(js_name = renderTurnRestrictions)]
    pub fn render_turn_restrictions(&self) -> Result<String, JsValue> {
        Ok(serde_json::to_string(&self.map.turn_restrictions_to_gj()).map_err(err_to_js)?)
    }

    #[wasm_bindgen(js_name = renderNeighbourhood)]
    pub fn render_neighbourhood(&self) -> Result<String, JsValue> {
        Ok(
            serde_json::to_string(&self.neighbourhood.as_ref().unwrap().to_gj(&self.map))
                .map_err(err_to_js)?,
        )
    }

    #[wasm_bindgen(js_name = renderAutoBoundaries)]
    pub fn render_auto_boundaries(&self) -> Result<String, JsValue> {
        Ok(serde_json::to_string(&self.map.render_auto_boundaries()).map_err(err_to_js)?)
    }

    /// `input`: GeoJson Feature w/ Polygon Geometry
    #[wasm_bindgen(js_name = setNeighbourhoodBoundary)]
    pub fn set_neighbourhood_boundary(
        &mut self,
        name: String,
        neighborhood_feature: JsValue,
    ) -> Result<(), JsValue> {
        let mut feature: Feature = serde_wasm_bindgen::from_value(neighborhood_feature)?;
        feature.set_property("name", name.clone());
        let neighbourhood_definition =
            NeighbourhoodDefinition::from_feature(feature, &self.map).map_err(err_to_js)?;
        let boundary =
            NeighbourhoodBoundary::new(neighbourhood_definition, self.map.context_data.as_ref());
        self.map.boundaries.insert(name, boundary);
        Ok(())
    }

    #[wasm_bindgen(js_name = deleteNeighbourhoodBoundary)]
    pub fn delete_neighbourhood_boundary(&mut self, name: String) {
        self.map.boundaries.remove(&name);
    }

    #[wasm_bindgen(js_name = renameNeighbourhoodBoundary)]
    pub fn rename_neighbourhood_boundary(&mut self, old_name: String, new_name: String) {
        let mut boundary = self.map.boundaries.remove(&old_name).unwrap();
        boundary.definition.name = new_name.clone();
        self.map.boundaries.insert(new_name, boundary);
    }

    #[wasm_bindgen(js_name = setCurrentNeighbourhood)]
    pub fn set_current_neighbourhood(
        &mut self,
        name: String,
        edit_perimeter_roads: bool,
    ) -> Result<(), JsValue> {
        let boundary = self.map.boundaries.get(&name).unwrap();

        // Are we still editing the same neighbourhood, just switching edit_perimeter_roads?
        let editing_same = self
            .neighbourhood
            .as_ref()
            .map(|n| n.name() == name)
            .unwrap_or(false);
        self.neighbourhood = Some(
            Neighbourhood::new(&self.map, boundary.clone(), edit_perimeter_roads)
                .map_err(err_to_js)?,
        );

        // Undoing edits in another neighbourhood doesn't make sense
        if !editing_same {
            self.map.undo_stack.clear();
            self.map.redo_queue.clear();
        }

        Ok(())
    }

    /// Takes a LngLat
    #[wasm_bindgen(js_name = addModalFilter)]
    pub fn add_modal_filter(&mut self, input: JsValue, kind: String) -> Result<(), JsValue> {
        let pos: LngLat = serde_wasm_bindgen::from_value(input)?;
        self.map.add_modal_filter(
            self.map.mercator.pt_to_mercator(Coord {
                x: pos.lng,
                y: pos.lat,
            }),
            Some(self.neighbourhood.as_ref().unwrap().editable_roads()),
            FilterKind::from_string(&kind).unwrap(),
        );
        self.after_edit();
        Ok(())
    }

    /// Takes a LineString feature
    #[wasm_bindgen(js_name = addManyModalFilters)]
    pub fn add_many_modal_filters(&mut self, input: JsValue, kind: String) -> Result<(), JsValue> {
        let gj: Feature = serde_wasm_bindgen::from_value(input)?;
        let mut linestring: LineString = gj.try_into().map_err(err_to_js)?;
        self.map.mercator.to_mercator_in_place(&mut linestring);

        self.map.add_many_modal_filters(
            linestring,
            &self.neighbourhood.as_ref().unwrap().interior_roads,
            FilterKind::from_string(&kind).unwrap(),
        );
        self.after_edit();
        Ok(())
    }

    #[wasm_bindgen(js_name = deleteModalFilter)]
    pub fn delete_modal_filter(&mut self, road: usize) {
        self.map.delete_modal_filter(RoadID(road));
        self.after_edit();
    }

    /// Takes an IntersectionID
    #[wasm_bindgen(js_name = addDiagonalFilter)]
    pub fn add_diagonal_filter(&mut self, intersection_id: usize) -> Result<(), JsValue> {
        self.map
            .add_diagonal_filter(IntersectionID(intersection_id));
        self.after_edit();
        Ok(())
    }

    /// Takes an IntersectionID
    #[wasm_bindgen(js_name = rotateDiagonalFilter)]
    pub fn rotate_diagonal_filter(&mut self, intersection_id: usize) -> Result<(), JsValue> {
        self.map
            .rotate_diagonal_filter(IntersectionID(intersection_id));
        self.after_edit();
        Ok(())
    }

    /// Takes an IntersectionID
    #[wasm_bindgen(js_name = deleteDiagonalFilter)]
    pub fn delete_diagonal_filter(&mut self, intersection_id: usize) -> Result<(), JsValue> {
        self.map
            .delete_diagonal_filter(IntersectionID(intersection_id));
        self.after_edit();
        Ok(())
    }

    #[wasm_bindgen(js_name = addTurnRestriction)]
    pub fn add_turn_restriction(&mut self, from: usize, to: usize) -> Result<(), JsValue> {
        self.map
            .add_turn_restriction(RoadID(from), RoadID(to))
            .map_err(err_to_js)?;
        self.after_edit();
        Ok(())
    }

    #[wasm_bindgen(js_name = deleteTurnRestriction)]
    pub fn delete_turn_restriction(
        &mut self,
        intersection: usize,
        from: usize,
        to: usize,
    ) -> Result<(), JsValue> {
        self.map
            .delete_turn_restriction(IntersectionID(intersection), RoadID(from), RoadID(to))
            .map_err(err_to_js)?;
        self.after_edit();
        Ok(())
    }

    #[wasm_bindgen(js_name = getTurnRestrictionTargets)]
    pub fn get_turn_restriction_targets_wasm(&self, road: usize) -> Result<String, JsValue> {
        Ok(
            serde_json::to_string(&self.map.get_turn_restriction_targets(RoadID(road)))
                .map_err(err_to_js)?,
        )
    }

    #[wasm_bindgen(js_name = toggleTravelFlow)]
    pub fn toggle_travel_flow(&mut self, road: usize) {
        self.map.toggle_travel_flow(RoadID(road));
        self.after_edit();
    }

    pub fn undo(&mut self) {
        self.map.undo();
        self.after_edit();
    }
    pub fn redo(&mut self) {
        self.map.redo();
        self.after_edit();
    }

    #[wasm_bindgen(js_name = getShortcutsCrossingRoad)]
    pub fn get_shortcuts_crossing_road(&self, road: usize) -> Result<String, JsValue> {
        Ok(serde_json::to_string(&GeoJson::from(
            Shortcuts::new(&self.map, self.neighbourhood.as_ref().unwrap())
                .subset(RoadID(road))
                .into_iter()
                .map(|path| path.to_gj(&self.map))
                .collect::<Vec<_>>(),
        ))
        .map_err(err_to_js)?)
    }

    #[wasm_bindgen(js_name = getAllShortcuts)]
    pub fn get_all_shortcuts(&self) -> Result<String, JsValue> {
        Ok(serde_json::to_string(&GeoJson::from(
            Shortcuts::new(&self.map, self.neighbourhood.as_ref().unwrap())
                .paths
                .into_iter()
                .map(|path| path.to_gj(&self.map))
                .collect::<Vec<_>>(),
        ))
        .map_err(err_to_js)?)
    }

    /// GJ with modal filters and named boundaries. This is meant for savefiles, so existing
    /// filters aren't included (and deletions of existing are included)
    #[wasm_bindgen(js_name = toSavefile)]
    pub fn to_savefile(&self) -> Result<String, JsValue> {
        // TODO Trim coordinates... in mercator?
        Ok(serde_json::to_string(&self.map.to_savefile()).map_err(err_to_js)?)
    }

    #[wasm_bindgen(js_name = loadSavefile)]
    pub fn load_savefile(&mut self, input: JsValue) -> Result<(), JsValue> {
        let gj: FeatureCollection = serde_wasm_bindgen::from_value(input)?;
        self.map.load_savefile(gj).map_err(err_to_js)?;
        self.neighbourhood = None;
        Ok(())
    }

    /// Returns GJ with two LineStrings, before and after
    #[wasm_bindgen(js_name = compareRoute)]
    pub fn compare_route(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        main_road_penalty: f64,
    ) -> Result<String, JsValue> {
        let pt1 = self.map.mercator.pt_to_mercator(Coord { x: x1, y: y1 });
        let pt2 = self.map.mercator.pt_to_mercator(Coord { x: x2, y: y2 });
        Ok(
            serde_json::to_string(&self.map.compare_route(pt1, pt2, main_road_penalty))
                .map_err(err_to_js)?,
        )
    }

    /// Returns GJ with a LineString per interior road
    #[wasm_bindgen(js_name = impactToOneDestination)]
    pub fn impact_to_one_destination(&mut self, x: f64, y: f64) -> Result<String, JsValue> {
        let pt = self.map.mercator.pt_to_mercator(Coord { x, y });
        Ok(serde_json::to_string(
            &self.map.impact_to_one_destination(
                pt,
                self.neighbourhood.as_ref().unwrap().editable_roads(),
            ),
        )
        .map_err(err_to_js)?)
    }

    /// Returns GJ with a LineString per road, with before/after counts
    #[wasm_bindgen(js_name = predictImpact)]
    pub fn predict_impact(&mut self) -> Result<String, JsValue> {
        self.map.rebuild_router(1.0);
        let mut impact = self.map.impact.take().unwrap();
        let out = impact.recalculate(&self.map);
        self.map.impact = Some(impact);
        Ok(serde_json::to_string(&out).map_err(err_to_js)?)
    }

    /// Returns a JSON blob [{before, after}], with before and after being LineStrings
    #[wasm_bindgen(js_name = getImpactsOnRoad)]
    pub fn get_impacts_on_road(&self, road: usize) -> Result<String, JsValue> {
        // Shouldn't need to recalculate impact
        Ok(serde_json::to_string(
            &self
                .map
                .impact
                .as_ref()
                .unwrap()
                .get_impacts_on_road(&self.map, RoadID(road)),
        )
        .map_err(err_to_js)?)
    }

    #[wasm_bindgen(js_name = getAllNeighbourhoods)]
    pub fn get_all_neighbourhoods(&self) -> Result<String, JsValue> {
        let features = self
            .map
            .boundaries
            .values()
            .map(|neighbourhood| neighbourhood.to_feature(&self.map));
        let fc = FeatureCollection::from_iter(features);
        Ok(serde_json::to_string(&fc).map_err(err_to_js)?)
    }

    #[wasm_bindgen(js_name = getAllIntersections)]
    pub fn get_all_intersections(&self) -> Result<String, JsValue> {
        Ok(serde_json::to_string(&GeoJson::from(
            self.map
                .intersections
                .iter()
                .map(|i| {
                    let mut f = self.map.mercator.to_wgs84_gj(&i.point);
                    f.set_property(
                        "has_turn_restrictions",
                        !self.map.turn_restrictions[i.id.00].is_empty(),
                    );
                    f.set_property("intersection_id", i.id.0);
                    f.set_property("osm", i.node.to_string());
                    f
                })
                .collect::<Vec<_>>(),
        ))
        .map_err(err_to_js)?)
    }

    #[wasm_bindgen(js_name = getMovements)]
    pub fn get_movements(&self, intersection: usize) -> Result<String, JsValue> {
        Ok(
            serde_json::to_string(&self.map.get_movements(IntersectionID(intersection)))
                .map_err(err_to_js)?,
        )
    }

    #[wasm_bindgen(js_name = getDemandModel)]
    pub fn get_demand_model(&self) -> Result<String, JsValue> {
        let Some(ref demand) = self.map.demand else {
            return Err(JsValue::from_str("no demand model"));
        };
        Ok(serde_json::to_string(&demand.to_gj(&self.map)).map_err(err_to_js)?)
    }

    // TODO This is also internal to MapModel. But not sure who should own Neighbourhood or how to
    // plumb, so duplicting here.
    fn after_edit(&mut self) {
        if let Some(ref mut n) = self.neighbourhood {
            n.after_edit(&self.map);
        }
    }
}

#[derive(Deserialize)]
struct LngLat {
    lng: f64,
    lat: f64,
}

fn err_to_js<E: std::fmt::Display>(err: E) -> JsValue {
    JsValue::from_str(&err.to_string())
}
