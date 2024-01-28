use std::collections::{BTreeMap, HashSet};
use std::fmt;

use anyhow::Result;
use geo::{
    Closest, ClosestPoint, Coord, EuclideanLength, Intersects, Line, LineInterpolatePoint,
    LineIntersection, LineLocatePoint, LineString, Point, Polygon,
};
use geojson::{Feature, FeatureCollection, GeoJson, Geometry};
use serde::Serialize;

use crate::{Mercator, Router, Tags};

pub struct MapModel {
    pub roads: Vec<Road>,
    pub intersections: Vec<Intersection>,
    // All geometry stored in worldspace, including rtrees
    pub mercator: Mercator,
    pub study_area_name: Option<String>,
    pub boundary_polygon: Polygon,

    // TODO Wasteful, can share some
    pub router_original: Router,
    // Calculated lazily
    pub router_current: Option<Router>,

    // TODO Keep edits / state here or not?
    pub modal_filters: BTreeMap<RoadID, ModalFilter>,
    pub undo_stack: Vec<Command>,
    pub redo_queue: Vec<Command>,
    // Stores boundary polygons in WGS84, with ALL of their GeoJSON props.
    pub boundaries: BTreeMap<String, Feature>,
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
    pub fn new(input_bytes: &[u8], study_area_name: Option<String>) -> Result<MapModel> {
        crate::scrape::scrape_osm(input_bytes, study_area_name)
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

    pub fn add_modal_filter(
        &mut self,
        click_pt: Coord,
        candidate_roads: &HashSet<RoadID>,
        kind: FilterKind,
    ) {
        let cmd = self.do_edit(self.add_modal_filter_cmd(click_pt, candidate_roads, kind));
        self.undo_stack.push(cmd);
        self.redo_queue.clear();
        self.after_edited();
    }

    fn add_modal_filter_cmd(
        &self,
        click_pt: Coord,
        candidate_roads: &HashSet<RoadID>,
        kind: FilterKind,
    ) -> Command {
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
        Command::SetModalFilter(
            r,
            Some(ModalFilter {
                percent_along,
                kind,
            }),
        )
    }

    fn after_edited(&mut self) {
        // Invalidate it
        self.router_current = None;
    }

    pub fn add_many_modal_filters(
        &mut self,
        along_line: LineString,
        candidate_roads: &HashSet<RoadID>,
        kind: FilterKind,
    ) {
        let mut edits = Vec::new();
        for r in candidate_roads {
            let road = self.get_r(*r);
            if let Some(percent_along) = linestring_intersection(&road.linestring, &along_line) {
                edits.push(Command::SetModalFilter(
                    *r,
                    Some(ModalFilter {
                        percent_along,
                        kind,
                    }),
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

    pub fn filters_to_gj(&self) -> FeatureCollection {
        let mut features = Vec::new();
        for (r, filter) in &self.modal_filters {
            let pt = self
                .get_r(*r)
                .linestring
                .line_interpolate_point(filter.percent_along)
                .unwrap();
            let mut f = Feature::from(Geometry::from(&self.mercator.to_wgs84(&pt)));
            f.set_property("filter_kind", filter.kind.to_string());
            f.set_property("road", r.0);
            features.push(f);
        }
        FeatureCollection {
            features,
            bbox: None,
            foreign_members: None,
        }
    }

    pub fn to_savefile(&self) -> FeatureCollection {
        let mut gj = self.filters_to_gj();
        // TODO When we detect existing filters, maybe need to instead compact edits
        for f in &mut gj.features {
            f.set_property("kind", "modal_filter");
            f.remove_property("road");
        }
        gj.features.extend(self.boundaries.values().cloned());

        let mut f = Feature::from(Geometry::from(
            &self.mercator.to_wgs84(&self.boundary_polygon),
        ));
        f.set_property("kind", "study_area_boundary");
        gj.features.push(f);

        gj.foreign_members = Some(
            serde_json::json!({
                "study_area_name": self.study_area_name,
            })
            .as_object()
            .unwrap()
            .clone(),
        );

        gj
    }

    pub fn load_savefile(&mut self, gj: FeatureCollection) -> Result<()> {
        // Clear previous state
        self.boundaries.clear();
        self.modal_filters.clear();
        self.undo_stack.clear();
        self.redo_queue.clear();

        // Filters could be defined for multiple neighbourhoods, not just the one
        // in the savefile
        let all_roads: HashSet<RoadID> = self.roads.iter().map(|r| r.id).collect();
        let mut cmds = Vec::new();

        for f in gj.features {
            match f.property("kind").unwrap().as_str().unwrap() {
                "modal_filter" => {
                    let kind = FilterKind::from_string(get_str_prop(&f, "filter_kind")?)?;
                    let gj_pt: Point = f.geometry.unwrap().try_into()?;
                    cmds.push(self.add_modal_filter_cmd(
                        self.mercator.pt_to_mercator(gj_pt.into()),
                        &all_roads,
                        kind,
                    ));
                }
                "boundary" => {
                    let name = get_str_prop(&f, "name")?;
                    if self.boundaries.contains_key(name) {
                        bail!("Multiple boundaries named {name} in savefile");
                    }
                    self.boundaries.insert(name.to_string(), f);
                }
                "study_area_boundary" => {
                    // TODO Detect if it's close enough to boundary_polygon? Overwrite?
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

        Ok(())
    }

    // Lazily builds the router if needed.
    pub fn compare_route(&mut self, pt1: Coord, pt2: Coord) -> GeoJson {
        if self.router_current.is_none() {
            self.router_current = Some(Router::new(
                &self.roads,
                &self.intersections,
                &self.modal_filters,
            ));
        }

        let mut features = Vec::new();
        if let Some(linestring) = self.router_original.route(self, pt1, pt2) {
            let mut f = Feature::from(Geometry::from(&self.mercator.to_wgs84(&linestring)));
            f.set_property("kind", "before");
            features.push(f);
        }
        if let Some(linestring) = self.router_current.as_ref().unwrap().route(self, pt1, pt2) {
            let mut f = Feature::from(Geometry::from(&self.mercator.to_wgs84(&linestring)));
            f.set_property("kind", "after");
            features.push(f);
        }
        GeoJson::from(features)
    }

    /// Return a polygon covering the world, minus a hole for the boundary, in WGS84
    pub fn invert_boundary(&self) -> Polygon {
        let (boundary, _) = self.mercator.to_wgs84(&self.boundary_polygon).into_inner();
        Polygon::new(
            LineString::from(vec![
                (180.0, 90.0),
                (-180.0, 90.0),
                (-180.0, -90.0),
                (180.0, -90.0),
                (180.0, 90.0),
            ]),
            vec![boundary],
        )
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
    pub kind: FilterKind,
    pub percent_along: f64,
}

#[derive(Clone, Copy)]
pub enum FilterKind {
    WalkCycleOnly,
    NoEntry,
    BusGate,
    SchoolStreet,
}

// TODO strum?
impl FilterKind {
    pub fn to_string(self) -> &'static str {
        match self {
            FilterKind::WalkCycleOnly => "walk_cycle_only",
            FilterKind::NoEntry => "no_entry",
            FilterKind::BusGate => "bus_gate",
            FilterKind::SchoolStreet => "school_street",
        }
    }

    pub fn from_string(x: &str) -> Result<Self> {
        match x {
            "walk_cycle_only" => Ok(FilterKind::WalkCycleOnly),
            "no_entry" => Ok(FilterKind::NoEntry),
            "bus_gate" => Ok(FilterKind::BusGate),
            "school_street" => Ok(FilterKind::SchoolStreet),
            _ => bail!("Invalid FilterKind: {x}"),
        }
    }
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
    // TODO Could use https://docs.rs/geo/latest/geo/algorithm/sweep/struct.Intersections.html, but
    // not sure about the order, so we'd do line_locate_point for everything and take the min
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

fn get_str_prop<'a>(f: &'a Feature, key: &str) -> Result<&'a str> {
    let Some(value) = f.property(key) else {
        bail!("Feature doesn't have a {key} property");
    };
    let Some(string) = value.as_str() else {
        bail!("Feature's {key} property isn't a string");
    };
    Ok(string)
}
