use std::collections::{BTreeMap, HashSet};
use std::fmt;

use anyhow::Result;
use geo::{
    Closest, ClosestPoint, Coord, EuclideanLength, Intersects, Line, LineInterpolatePoint,
    LineIntersection, LineLocatePoint, LineString, Point, Polygon,
};
use geojson::{Feature, FeatureCollection, GeoJson, Geometry};
use serde::Serialize;

use crate::{Mercator, Neighbourhood, Router, Tags};

pub struct MapModel {
    pub roads: Vec<Road>,
    pub intersections: Vec<Intersection>,
    // All geometry stored in worldspace, including rtrees
    pub mercator: Mercator,
    // TODO Wasteful, can share some
    pub router_original: Router,
    pub router_current: Router,

    // TODO Keep edits / state here or not?
    pub modal_filters: BTreeMap<RoadID, ModalFilter>,
    pub undo_stack: Vec<Command>,
    pub redo_queue: Vec<Command>,
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
    pub id: RoadID,
    pub src_i: IntersectionID,
    pub dst_i: IntersectionID,
    pub way: osm_reader::WayID,
    pub node1: osm_reader::NodeID,
    pub node2: osm_reader::NodeID,
    pub linestring: LineString,
    pub tags: Tags,
}

pub struct Intersection {
    pub id: IntersectionID,
    pub node: osm_reader::NodeID,
    pub point: Point,
    pub roads: Vec<RoadID>,
}

impl MapModel {
    /// Call with bytes of an osm.pbf or osm.xml string
    pub fn new(input_bytes: &[u8]) -> Result<MapModel> {
        crate::scrape::scrape_osm(input_bytes)
    }

    pub fn get_r(&self, r: RoadID) -> &Road {
        &self.roads[r.0]
    }

    pub fn get_i(&self, i: IntersectionID) -> &Intersection {
        &self.intersections[i.0]
    }

    pub fn find_edge(&self, i1: IntersectionID, i2: IntersectionID) -> &Road {
        // TODO Store lookup table
        for r in &self.get_i(i1).roads {
            let road = self.get_r(*r);
            if road.src_i == i2 || road.dst_i == i2 {
                return road;
            }
        }
        panic!("no road from {i1} to {i2} or vice versa");
    }

    pub fn add_modal_filter(&mut self, click_pt: Coord, candidate_roads: &HashSet<RoadID>) {
        let cmd = self.do_edit(self.add_modal_filter_cmd(click_pt, candidate_roads));
        self.undo_stack.push(cmd);
        self.redo_queue.clear();
        self.after_edited();
    }

    fn add_modal_filter_cmd(&self, click_pt: Coord, candidate_roads: &HashSet<RoadID>) -> Command {
        // TODO prune with rtree?
        let (_, r, percent_along) = candidate_roads
            .iter()
            .filter_map(|r| {
                let road = self.get_r(*r);
                if let Some(hit_pt) = match road.linestring.closest_point(&click_pt.into()) {
                    Closest::Intersection(pt) => Some(pt),
                    Closest::SinglePoint(pt) => Some(pt),
                    Closest::Indeterminate => None,
                } {
                    let score = Line::new(click_pt, hit_pt.into()).euclidean_length();
                    let percent_along = road.linestring.line_locate_point(&hit_pt).unwrap();
                    Some(((score * 100.0) as usize, road.id, percent_along))
                } else {
                    None
                }
            })
            .min_by_key(|pair| pair.0)
            .unwrap();
        Command::SetModalFilter(r, Some(ModalFilter { percent_along }))
    }

    fn after_edited(&mut self) {
        self.router_current = Router::new(&self.roads, &self.intersections, &self.modal_filters);
    }

    pub fn add_many_modal_filters(
        &mut self,
        along_line: LineString,
        candidate_roads: &HashSet<RoadID>,
    ) {
        let mut edits = Vec::new();
        for r in candidate_roads {
            let road = self.get_r(*r);
            if let Some(percent_along) = linestring_intersection(&road.linestring, &along_line) {
                edits.push(Command::SetModalFilter(
                    *r,
                    Some(ModalFilter { percent_along }),
                ));
            }
        }
        let cmd = self.do_edit(Command::Multiple(edits));
        self.undo_stack.push(cmd);
        self.redo_queue.clear();
        self.after_edited();
    }

    pub fn delete_modal_filter(&mut self, r: RoadID) {
        let cmd = self.do_edit(Command::SetModalFilter(r, None));
        self.undo_stack.push(cmd);
        self.redo_queue.clear();
        self.after_edited();
    }

    // Returns the command to undo this one
    fn do_edit(&mut self, cmd: Command) -> Command {
        match cmd {
            Command::SetModalFilter(r, filter) => {
                let prev = self.modal_filters.get(&r).cloned();
                if let Some(filter) = filter {
                    info!("added a filter to {r} at {}%", filter.percent_along);
                    self.modal_filters.insert(r, filter);
                } else {
                    info!("deleted a filter from {r}");
                    self.modal_filters.remove(&r);
                }
                Command::SetModalFilter(r, prev)
            }
            Command::Multiple(list) => {
                let undo_list = list.into_iter().map(|cmd| self.do_edit(cmd)).collect();
                Command::Multiple(undo_list)
            }
        }
    }

    pub fn undo(&mut self) {
        // The UI shouldn't call this when the stack is empty, but when holding down the redo key,
        // it doesn't update fast enough
        if let Some(cmd) = self.undo_stack.pop() {
            let cmd = self.do_edit(cmd);
            self.redo_queue.push(cmd);
            self.after_edited();
        }
    }

    pub fn redo(&mut self) {
        if self.redo_queue.is_empty() {
            return;
        }
        let cmd = self.redo_queue.remove(0);
        let cmd = self.do_edit(cmd);
        self.undo_stack.push(cmd);
        self.after_edited();
    }

    pub fn to_savefile(&self, neighbourhood: Option<&Neighbourhood>) -> GeoJson {
        let mut features = Vec::new();

        // A point per modal filter
        // (When we detect existing, maybe need to instead compact edits)
        for (r, modal_filter) in &self.modal_filters {
            let pt = self
                .get_r(*r)
                .linestring
                .line_interpolate_point(modal_filter.percent_along)
                .unwrap();
            // TODO Maybe make the WASM API always do the mercator stuff
            let mut f = Feature::from(Geometry::from(&self.mercator.to_wgs84(&pt)));
            f.set_property("kind", "modal_filter");
            features.push(f);
        }

        if let Some(neighbourhood) = neighbourhood {
            let mut f = Feature::from(Geometry::from(
                &self.mercator.to_wgs84(&neighbourhood.boundary_polygon),
            ));
            f.set_property("kind", "boundary");
            features.push(f);
        }

        GeoJson::from(features)
    }

    /// Returns the optional boundary polygon
    pub fn load_savefile(&mut self, gj: FeatureCollection) -> Result<Option<Polygon>> {
        // Clear previous state
        self.modal_filters.clear();
        self.undo_stack.clear();
        self.redo_queue.clear();

        // Filters could be defined for multiple neighbourhoods, not just the one
        // in the savefile
        let all_roads: HashSet<RoadID> = self.roads.iter().map(|r| r.id).collect();
        let mut boundary = None;
        let mut cmds = Vec::new();

        for f in gj.features {
            match f.property("kind").unwrap().as_str().unwrap() {
                "modal_filter" => {
                    let gj_pt: Point = f.geometry.unwrap().try_into()?;
                    cmds.push(
                        self.add_modal_filter_cmd(
                            self.mercator.pt_to_mercator(gj_pt.into()),
                            &all_roads,
                        ),
                    );
                }
                "boundary" => {
                    if boundary.is_some() {
                        bail!("Multiple boundaries in savefile");
                    }
                    let mut polygon: Polygon = f.geometry.unwrap().value.try_into()?;
                    self.mercator.to_mercator_in_place(&mut polygon);
                    boundary = Some(polygon);
                }
                x => bail!("Unknown kind in savefile {x}"),
            }
        }

        if !cmds.is_empty() {
            let cmd = self.do_edit(Command::Multiple(cmds));
            self.undo_stack.push(cmd);
            self.redo_queue.clear();
        }
        self.after_edited();

        Ok(boundary)
    }

    pub fn compare_route(&self, pt1: Coord, pt2: Coord) -> GeoJson {
        let mut features = Vec::new();
        if let Some(linestring) = self.router_original.route(self, pt1, pt2) {
            let mut f = Feature::from(Geometry::from(&self.mercator.to_wgs84(&linestring)));
            f.set_property("kind", "before");
            features.push(f);
        }
        if let Some(linestring) = self.router_current.route(self, pt1, pt2) {
            let mut f = Feature::from(Geometry::from(&self.mercator.to_wgs84(&linestring)));
            f.set_property("kind", "after");
            features.push(f);
        }
        GeoJson::from(features)
    }
}

impl Road {
    pub fn length(&self) -> f64 {
        self.linestring.euclidean_length()
    }

    pub fn to_gj(&self, mercator: &Mercator) -> Feature {
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

#[derive(Clone)]
pub struct ModalFilter {
    pub percent_along: f64,
}

pub enum Command {
    SetModalFilter(RoadID, Option<ModalFilter>),
    Multiple(Vec<Command>),
}

// Looks for the first place ls2 crosses ls1. Returns the percent_along ls1 of that point.
fn linestring_intersection(ls1: &LineString, ls2: &LineString) -> Option<f64> {
    if !ls1.intersects(ls2) {
        return None;
    }
    // TODO Urgh very brute force
    for line1 in ls1.lines() {
        for line2 in ls2.lines() {
            if let Some(LineIntersection::SinglePoint { intersection, .. }) =
                geo::algorithm::line_intersection::line_intersection(line1, line2)
            {
                return ls1.line_locate_point(&intersection.into());
            }
        }
    }
    // TODO Didn't find it...
    None
}
