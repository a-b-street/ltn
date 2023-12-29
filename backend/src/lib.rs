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
use self::map_model::{Intersection, IntersectionID, MapModel, Road, RoadID};
use self::neighbourhood::Neighbourhood;
use self::render_cells::RenderCells;
use self::shortcuts::Shortcuts;

mod cells;
mod common;
mod map_model;
mod neighbourhood;
mod render_cells;
mod scrape;
mod shortcuts;

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
    pub fn new(input_bytes: &[u8]) -> Result<LTN, JsValue> {
        // Panics shouldn't happen, but if they do, console.log them.
        console_error_panic_hook::set_once();
        START.call_once(|| {
            console_log::init_with_level(log::Level::Info).unwrap();
        });

        let map = MapModel::new(input_bytes).map_err(err_to_js)?;
        Ok(LTN {
            map,
            neighbourhood: None,
        })
    }

    /// Returns a GeoJSON string. Just shows the full network
    #[wasm_bindgen()]
    pub fn render(&mut self) -> Result<String, JsValue> {
        let mut features = Vec::new();

        for r in &self.map.roads {
            features.push(r.to_gj(&self.map.mercator));
        }

        let gj = GeoJson::from(features);
        let out = serde_json::to_string(&gj).map_err(err_to_js)?;
        Ok(out)
    }

    #[wasm_bindgen(js_name = toRouteSnapper)]
    pub fn to_route_snapper(&self) -> Vec<u8> {
        use route_snapper_graph::{Edge, NodeID, RouteSnapperMap};

        let mut nodes = Vec::new();
        for i in &self.map.intersections {
            nodes.push(self.map.mercator.to_wgs84(&i.point).into());
        }

        let mut edges = Vec::new();
        for r in &self.map.roads {
            edges.push(Edge {
                node1: NodeID(r.src_i.0 as u32),
                node2: NodeID(r.dst_i.0 as u32),
                geometry: self.map.mercator.to_wgs84(&r.linestring),
                // Isn't serialized, doesn't matter
                length_meters: 0.0,
                name: r.tags.get("name").cloned(),
            });
        }

        let graph = RouteSnapperMap { nodes, edges };
        let bytes = bincode::serialize(&graph).unwrap();
        bytes
    }

    /// Takes boundary GJ polygon, returns GJ with more details
    #[wasm_bindgen(js_name = analyzeNeighbourhood)]
    pub fn analyze_neighbourhood(&mut self, input: JsValue) -> Result<String, JsValue> {
        let boundary_gj: Feature = serde_wasm_bindgen::from_value(input)?;
        let mut boundary_geo: Polygon = boundary_gj.try_into().map_err(err_to_js)?;
        self.map.mercator.to_mercator_in_place(&mut boundary_geo);

        self.neighbourhood = Some(Neighbourhood::new(&self.map, boundary_geo).map_err(err_to_js)?);
        self.render_neighbourhood()
    }

    #[wasm_bindgen(js_name = renderNeighbourhood)]
    pub fn render_neighbourhood(&self) -> Result<String, JsValue> {
        Ok(
            serde_json::to_string(&self.neighbourhood.as_ref().unwrap().to_gj(&self.map))
                .map_err(err_to_js)?,
        )
    }

    #[wasm_bindgen(js_name = unsetNeighbourhood)]
    pub fn unset_neighbourhood(&mut self) {
        self.neighbourhood = None;
    }

    /// Takes a LngLat, returns the same GJ as analyze_neighbourhood
    #[wasm_bindgen(js_name = addModalFilter)]
    pub fn add_modal_filter(&mut self, input: JsValue) -> Result<String, JsValue> {
        let pos: LngLat = serde_wasm_bindgen::from_value(input)?;
        self.map.add_modal_filter(
            self.map.mercator.pt_to_mercator(Coord {
                x: pos.lng,
                y: pos.lat,
            }),
            &self.neighbourhood.as_ref().unwrap().interior_roads,
        );
        self.render_neighbourhood()
    }

    /// Takes a LineString feature
    #[wasm_bindgen(js_name = addManyModalFilters)]
    pub fn add_many_modal_filters(&mut self, input: JsValue) -> Result<String, JsValue> {
        let gj: Feature = serde_wasm_bindgen::from_value(input)?;
        let mut linestring: LineString = gj.try_into().map_err(err_to_js)?;
        self.map.mercator.to_mercator_in_place(&mut linestring);

        self.map.add_many_modal_filters(
            linestring,
            &self.neighbourhood.as_ref().unwrap().interior_roads,
        );
        self.render_neighbourhood()
    }

    #[wasm_bindgen(js_name = deleteModalFilter)]
    pub fn delete_modal_filter(&mut self, road: usize) -> Result<String, JsValue> {
        self.map.delete_modal_filter(RoadID(road));
        self.render_neighbourhood()
    }

    pub fn undo(&mut self) -> Result<String, JsValue> {
        self.map.undo();
        self.render_neighbourhood()
    }
    pub fn redo(&mut self) -> Result<String, JsValue> {
        self.map.redo();
        self.render_neighbourhood()
    }

    #[wasm_bindgen(js_name = getShortcutsCrossingRoad)]
    pub fn get_shortcuts_crossing_road(&self, road: usize) -> Result<String, JsValue> {
        Ok(serde_json::to_string(&GeoJson::from(
            Shortcuts::new(&self.map, self.neighbourhood.as_ref().unwrap())
                .subset(RoadID(road))
                .into_iter()
                .map(|path| {
                    Feature::from(Geometry::from(
                        &self.map.mercator.to_wgs84(&path.geometry(&self.map)),
                    ))
                })
                .collect::<Vec<_>>(),
        ))
        .map_err(err_to_js)?)
    }

    /// GJ with modal filters and optionally the neighbourhood boundary
    #[wasm_bindgen(js_name = toSavefile)]
    pub fn to_savefile(&self) -> Result<String, JsValue> {
        // TODO Trim coordinates... in mercator?
        Ok(
            serde_json::to_string(&self.map.to_savefile(self.neighbourhood.as_ref()))
                .map_err(err_to_js)?,
        )
    }

    /// Doesn't return anything; the caller has to figure out what to render
    #[wasm_bindgen(js_name = loadSavefile)]
    pub fn load_savefile(&mut self, input: JsValue) -> Result<(), JsValue> {
        let gj: FeatureCollection = serde_wasm_bindgen::from_value(input)?;
        let boundary = self.map.load_savefile(gj).map_err(err_to_js)?;

        self.neighbourhood = None;
        if let Some(boundary) = boundary {
            self.neighbourhood = Some(Neighbourhood::new(&self.map, boundary).map_err(err_to_js)?);
        }

        Ok(())
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
