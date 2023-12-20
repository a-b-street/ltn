#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Once;

use geo::{EuclideanLength, LineString, Point, Polygon};
use geojson::{Feature, GeoJson, Geometry};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use self::cells::Cell;
use self::neighbourhood::Neighbourhood;
use self::render_cells::RenderCells;
use self::shortcuts::Shortcuts;

mod cells;
mod mercator;
mod neighbourhood;
mod node_map;
mod render_cells;
mod scrape;
mod shortcuts;
mod tags;

static START: Once = Once::new();

#[wasm_bindgen]
pub struct MapModel {
    roads: Vec<Road>,
    intersections: Vec<Intersection>,
    // All geometry stored in worldspace, including rtrees
    mercator: mercator::Mercator,

    // TODO Split stuff
    modal_filters: BTreeMap<RoadID, ModalFilter>,
}

impl MapModel {
    fn get_r(&self, r: RoadID) -> &Road {
        &self.roads[r.0]
    }

    fn get_i(&self, i: IntersectionID) -> &Intersection {
        &self.intersections[i.0]
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct RoadID(pub usize);
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct IntersectionID(pub usize);

impl fmt::Display for RoadID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Road #{}", self.0)
    }
}

impl fmt::Display for IntersectionID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Intersection #{}", self.0)
    }
}

pub struct Road {
    id: RoadID,
    src_i: IntersectionID,
    dst_i: IntersectionID,
    way: osm_reader::WayID,
    node1: osm_reader::NodeID,
    node2: osm_reader::NodeID,
    linestring: LineString,
    tags: tags::Tags,
}

pub struct Intersection {
    id: IntersectionID,
    node: osm_reader::NodeID,
    point: Point,
    roads: Vec<RoadID>,
}

#[wasm_bindgen]
impl MapModel {
    /// Call with bytes of an osm.pbf or osm.xml string
    #[wasm_bindgen(constructor)]
    pub fn new(input_bytes: &[u8]) -> Result<MapModel, JsValue> {
        // Panics shouldn't happen, but if they do, console.log them.
        console_error_panic_hook::set_once();
        START.call_once(|| {
            console_log::init_with_level(log::Level::Info).unwrap();
        });

        scrape::scrape_osm(input_bytes).map_err(err_to_js)
    }

    /// Returns a GeoJSON string. Just shows the full network
    #[wasm_bindgen()]
    pub fn render(&mut self) -> Result<String, JsValue> {
        let mut features = Vec::new();

        for r in &self.roads {
            features.push(r.to_gj(&self.mercator));
        }

        let gj = GeoJson::from(features);
        let out = serde_json::to_string(&gj).map_err(err_to_js)?;
        Ok(out)
    }

    #[wasm_bindgen(js_name = toRouteSnapper)]
    pub fn to_route_snapper(&self) -> Vec<u8> {
        use route_snapper_graph::{Edge, NodeID, RouteSnapperMap};

        let mut nodes = Vec::new();
        for i in &self.intersections {
            nodes.push(self.mercator.to_wgs84(&i.point).into());
        }

        let mut edges = Vec::new();
        for r in &self.roads {
            edges.push(Edge {
                node1: NodeID(r.src_i.0 as u32),
                node2: NodeID(r.dst_i.0 as u32),
                geometry: self.mercator.to_wgs84(&r.linestring),
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
    pub fn analyze_neighbourhood(&self, input: JsValue) -> Result<String, JsValue> {
        let boundary_gj: Feature = serde_wasm_bindgen::from_value(input)?;
        let mut boundary_geo: Polygon = boundary_gj.try_into().map_err(err_to_js)?;
        self.mercator.to_mercator_in_place(&mut boundary_geo);

        let neighbourhood = Neighbourhood::new(self, boundary_geo).map_err(err_to_js)?;
        Ok(serde_json::to_string(&neighbourhood.to_gj(self)).map_err(err_to_js)?)
    }

    #[wasm_bindgen(js_name = addModalFilter)]
    pub fn add_modal_filter(&self, input: JsValue) -> Result<(), JsValue> {
        let pos: LngLat = serde_wasm_bindgen::from_value(input)?;
        info!("add to {}, {}", pos.lng, pos.lat);
        Ok(())
    }

    fn find_edge(&self, i1: IntersectionID, i2: IntersectionID) -> &Road {
        // TODO Store lookup table
        for r in &self.get_i(i1).roads {
            let road = self.get_r(*r);
            if road.src_i == i2 || road.dst_i == i2 {
                return road;
            }
        }
        panic!("no road from {i1} to {i2} or vice versa");
    }
}

impl Road {
    fn length(&self) -> f64 {
        self.linestring.euclidean_length()
    }

    fn to_gj(&self, mercator: &mercator::Mercator) -> Feature {
        let mut f = Feature::from(Geometry::from(&mercator.to_wgs84(&self.linestring)));
        f.set_property("id", self.id.0);
        f.set_property("way", self.way.to_string());
        f.set_property("node1", self.node1.to_string());
        f.set_property("node2", self.node2.to_string());
        for (k, v) in &self.tags.0 {
            f.set_property(k, v.to_string());
        }
        f
    }
}

#[derive(Deserialize)]
struct LngLat {
    lng: f64,
    lat: f64,
}

pub struct ModalFilter {
    pub distance: f64,
}

fn err_to_js<E: std::fmt::Display>(err: E) -> JsValue {
    JsValue::from_str(&err.to_string())
}
