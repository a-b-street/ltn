#[macro_use]
extern crate log;

use std::fmt;
use std::sync::Once;

use geo::{LineString, Point};
use geojson::{Feature, GeoJson, Geometry};
use wasm_bindgen::prelude::*;

mod mercator;
mod scrape;
mod tags;

static START: Once = Once::new();

#[wasm_bindgen]
pub struct MapModel {
    roads: Vec<Road>,
    intersections: Vec<Intersection>,
    // All geometry stored in worldspace, including rtrees
    mercator: mercator::Mercator,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct RoadID(pub usize);
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
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

    fn find_edge(&self, i1: IntersectionID, i2: IntersectionID) -> &Road {
        // TODO Store lookup table
        for r in &self.intersections[i1.0].roads {
            let road = &self.roads[r.0];
            if road.src_i == i2 || road.dst_i == i2 {
                return road;
            }
        }
        panic!("no road from {i1} to {i2} or vice versa");
    }
}

impl Road {
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

fn err_to_js<E: std::fmt::Display>(err: E) -> JsValue {
    JsValue::from_str(&err.to_string())
}
