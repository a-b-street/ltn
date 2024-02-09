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
    FilterKind, Intersection, IntersectionID, MapModel, ModalFilter, Road, RoadID,
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

    fn to_route_snapper_graph(&self) -> route_snapper_graph::RouteSnapperMap {
        use geo::{LineIntersection, LineLocatePoint, LineSplit, Point};
        use route_snapper_graph::{Edge, NodeID, RouteSnapperMap};
        use std::collections::BTreeMap;

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

        // TODO This should be a method on RouteSnapperMap, but we'll have to project to mercator
        // and back
        let mut all_lines = Vec::new();
        for r in &self.map.roads {
            for line in r.linestring.lines() {
                all_lines.push(LineWithData { line, id: r.id });
            }
        }

        // Make new nodes for these split points, and figure out where to split roads
        let mut pt_to_node_id: BTreeMap<(isize, isize), NodeID> = BTreeMap::new();
        for i in &self.map.intersections {
            pt_to_node_id.insert(hashify_point(i.point.into()), NodeID(i.id.0 as u32));
        }
        let mut split_roads_at: BTreeMap<RoadID, Vec<f64>> = BTreeMap::new();
        for (r1, r2, cross) in geo::sweep::Intersections::<_>::from_iter(all_lines) {
            if let LineIntersection::SinglePoint {
                intersection,
                is_proper,
            } = cross
            {
                // Intersections are expected constantly at endpoints, so ignore those
                if is_proper {
                    pt_to_node_id.insert(hashify_point(intersection), NodeID(nodes.len() as u32));
                    nodes.push(
                        self.map
                            .mercator
                            .to_wgs84(&Point::from(intersection))
                            .into(),
                    );

                    let r1_dist = self
                        .map
                        .get_r(r1.id)
                        .linestring
                        .line_locate_point(&intersection.into())
                        .unwrap();
                    let r2_dist = self
                        .map
                        .get_r(r2.id)
                        .linestring
                        .line_locate_point(&intersection.into())
                        .unwrap();
                    split_roads_at
                        .entry(r1.id)
                        .or_insert_with(Vec::new)
                        .push(r1_dist);
                    split_roads_at
                        .entry(r2.id)
                        .or_insert_with(Vec::new)
                        .push(r2_dist);
                }
            }
        }

        let mut remove_old_roads = Vec::new();
        for (r, fractions) in split_roads_at {
            for split_ls in self
                .map
                .get_r(r)
                .linestring
                .line_split_many(&fractions)
                .unwrap()
            {
                let Some(split_ls) = split_ls else {
                    // Sometimes the split points are too close together
                    continue;
                };
                // Make a new edge
                edges.push(Edge {
                    node1: pt_to_node_id[&hashify_point(split_ls.0[0])],
                    node2: pt_to_node_id[&hashify_point(*split_ls.0.last().unwrap())],
                    geometry: self.map.mercator.to_wgs84(&split_ls),
                    // Isn't serialized, doesn't matter
                    length_meters: 0.0,
                    name: self.map.get_r(r).tags.get("name").cloned(),
                });
            }

            remove_old_roads.push(r);
        }

        // Remove the old edge with the full road. The index into edges matches the RoadID, but
        // we'll change indices as we modify stuff, so carefully do it backwards
        remove_old_roads.reverse();
        for r in remove_old_roads {
            edges.remove(r.0);
        }

        RouteSnapperMap { nodes, edges }
    }

    #[wasm_bindgen(js_name = toRouteSnapper)]
    pub fn to_route_snapper(&self) -> Vec<u8> {
        let graph = self.to_route_snapper_graph();
        bincode::serialize(&graph).unwrap()
    }

    #[wasm_bindgen(js_name = toRouteSnapperGj)]
    pub fn to_route_snapper_gj(&self) -> Result<String, JsValue> {
        let graph = self.to_route_snapper_graph();

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

    /// GJ with modal filters and named boundaries
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

#[derive(Clone, Debug)]
struct LineWithData {
    line: geo::Line,
    id: RoadID,
}

impl geo::sweep::Cross for LineWithData {
    type Scalar = f64;

    fn line(&self) -> geo::sweep::LineOrPoint<Self::Scalar> {
        self.line.line()
    }
}

fn hashify_point(pt: Coord) -> (isize, isize) {
    // cm resolution
    ((pt.x * 100.0) as isize, (pt.y * 100.0) as isize)
}
