use std::collections::{BTreeMap, HashSet};
use std::fmt;

use anyhow::Result;
use geo::{
    Closest, ClosestPoint, Coord, EuclideanLength, Line, LineLocatePoint, LineString, Point,
};
use geojson::{Feature, Geometry};
use serde::Serialize;

use crate::{Mercator, Tags};

pub struct MapModel {
    pub roads: Vec<Road>,
    pub intersections: Vec<Intersection>,
    // All geometry stored in worldspace, including rtrees
    pub mercator: Mercator,

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
        // TODO prune with rtree?
        let (_, r, percent_along) = candidate_roads
            .iter()
            .map(|r| {
                let road = self.get_r(*r);
                let hit_pt = match road.linestring.closest_point(&click_pt.into()) {
                    Closest::Intersection(pt) => pt,
                    Closest::SinglePoint(pt) => pt,
                    Closest::Indeterminate => unreachable!(),
                };
                let score = Line::new(click_pt, hit_pt.into()).euclidean_length();
                let percent_along = road.linestring.line_locate_point(&hit_pt).unwrap();
                ((score * 100.0) as usize, road.id, percent_along)
            })
            .min_by_key(|pair| pair.0)
            .unwrap();

        let cmd = self.do_edit(Command::SetModalFilter(
            r,
            Some(ModalFilter { percent_along }),
        ));
        self.undo_stack.push(cmd);
        self.redo_queue.clear();
    }

    pub fn delete_modal_filter(&mut self, r: RoadID) {
        let cmd = self.do_edit(Command::SetModalFilter(r, None));
        self.undo_stack.push(cmd);
        self.redo_queue.clear();
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
        }
    }

    pub fn undo(&mut self) {
        let cmd = self.undo_stack.pop().unwrap();
        let cmd = self.do_edit(cmd);
        self.redo_queue.push(cmd);
    }

    pub fn redo(&mut self) {
        let cmd = self.redo_queue.remove(0);
        let cmd = self.do_edit(cmd);
        self.undo_stack.push(cmd);
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
}
