#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

use std::sync::Once;

use geo::{Coord, LineString, Polygon};
use geojson::{Feature, FeatureCollection, GeoJson, Geometry};
use serde::Deserialize;
use wasm_bindgen::prelude::*;

use self::cells::Cell;
use self::common::*;
use self::map_model::{
    Direction, FilterKind, Intersection, IntersectionID, MapModel, ModalFilter, Road, RoadID,
};
use self::neighbourhood::Neighbourhood;
use self::render_cells::RenderCells;
use self::route::Router;
use self::shortcuts::Shortcuts;

mod cells;
mod common;
mod map_model;
mod neighbourhood;
mod render_cells;
mod route;
mod route_snapper;
mod scrape;
mod shortcuts;
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
    pub fn new(input_bytes: &[u8], study_area_name: Option<String>) -> Result<LTN, JsValue> {
        // Panics shouldn't happen, but if they do, console.log them.
        console_error_panic_hook::set_once();
        START.call_once(|| {
            console_log::init_with_level(log::Level::Info).unwrap();
        });

        let map = MapModel::new(input_bytes, study_area_name).map_err(err_to_js)?;
        Ok(LTN {
            map,
            neighbourhood: None,
        })
    }

    #[wasm_bindgen(js_name = getInvertedBoundary)]
    pub fn get_inverted_boundary(&self) -> Result<String, JsValue> {
        let f = Feature::from(Geometry::from(&self.map.invert_boundary()));
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

    #[wasm_bindgen(js_name = renderNeighbourhood)]
    pub fn render_neighbourhood(&self) -> Result<String, JsValue> {
        Ok(
            serde_json::to_string(&self.neighbourhood.as_ref().unwrap().to_gj(&self.map))
                .map_err(err_to_js)?,
        )
    }

    /// Takes a name and boundary GJ polygon
    #[wasm_bindgen(js_name = setNeighbourhoodBoundary)]
    pub fn set_neighbourhood_boundary(
        &mut self,
        name: String,
        input: JsValue,
    ) -> Result<(), JsValue> {
        let mut boundary_gj: Feature = serde_wasm_bindgen::from_value(input)?;
        boundary_gj.set_property("kind", "boundary");
        boundary_gj.set_property("name", name.clone());
        self.map.boundaries.insert(name, boundary_gj);
        Ok(())
    }

    #[wasm_bindgen(js_name = deleteNeighbourhoodBoundary)]
    pub fn delete_neighbourhood_boundary(&mut self, name: String) {
        self.map.boundaries.remove(&name);
    }

    #[wasm_bindgen(js_name = setCurrentNeighbourhood)]
    pub fn set_current_neighbourhood(&mut self, name: String) -> Result<(), JsValue> {
        let boundary_gj = self.map.boundaries.get(&name).cloned().unwrap();
        let mut boundary_geo: Polygon = boundary_gj.try_into().map_err(err_to_js)?;
        self.map.mercator.to_mercator_in_place(&mut boundary_geo);

        self.neighbourhood =
            Some(Neighbourhood::new(&self.map, name, boundary_geo).map_err(err_to_js)?);
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
            &self.neighbourhood.as_ref().unwrap().interior_roads,
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

    #[wasm_bindgen(js_name = toggleDirection)]
    pub fn toggle_direction(&mut self, road: usize) {
        self.map.toggle_direction(RoadID(road));
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
    pub fn compare_route(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) -> Result<String, JsValue> {
        let pt1 = self.map.mercator.pt_to_mercator(Coord { x: x1, y: y1 });
        let pt2 = self.map.mercator.pt_to_mercator(Coord { x: x2, y: y2 });
        Ok(serde_json::to_string(&self.map.compare_route(pt1, pt2)).map_err(err_to_js)?)
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
