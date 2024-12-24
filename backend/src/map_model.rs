use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use anyhow::Result;
use geo::{
    Closest, ClosestPoint, Coord, Euclidean, Length, Line, LineInterpolatePoint, LineLocatePoint,
    LineString, Point, Polygon,
};
use geojson::{Feature, FeatureCollection, GeoJson, Geometry};
use rstar::{primitives::GeomWithData, RTree, AABB};
use serde::Serialize;
use utils::{Mercator, Tags};

use crate::geo_helpers::{angle_of_pt_on_line, buffer_aabb, limit_angle, linestring_intersection};
use crate::Router;

pub struct MapModel {
    pub roads: Vec<Road>,
    pub intersections: Vec<Intersection>,
    // All geometry stored in worldspace, including rtrees
    pub mercator: Mercator,
    pub study_area_name: Option<String>,
    pub boundary_wgs84: Polygon,
    pub closest_road: RTree<GeomWithData<LineString, RoadID>>,
    pub closest_intersection: RTree<GeomWithData<Point, IntersectionID>>,

    // Only those acting as severances; above or belowground don't count
    pub railways: Vec<LineString>,
    pub waterways: Vec<LineString>,

    // TODO Wasteful, can share some
    // This is guaranteed to exist, only Option during MapModel::new internals
    pub router_original: Option<Router>,
    // Calculated lazily. Changes with edits and main_road_penalty.
    pub router_current: Option<Router>,
    // Calculated lazily. No edits, just main_road_penalty.
    pub router_original_with_penalty: Option<Router>,

    // Just from the basemap, existing filters
    pub original_modal_filters: BTreeMap<RoadID, ModalFilter>,
    pub modal_filters: BTreeMap<RoadID, ModalFilter>,

    // Every road is filled out
    pub directions: BTreeMap<RoadID, Direction>,

    // TODO Keep edits / state here or not?
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
    #[allow(unused)]
    pub node: osm_reader::NodeID,
    pub point: Point,
    pub roads: Vec<RoadID>,
}

impl MapModel {
    /// Call with bytes of an osm.pbf or osm.xml string
    pub fn new(
        input_bytes: &[u8],
        boundary_wgs84: Polygon,
        study_area_name: Option<String>,
    ) -> Result<MapModel> {
        crate::scrape::scrape_osm(input_bytes, boundary_wgs84, study_area_name)
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
        pt: Coord,
        candidate_roads: Option<&BTreeSet<RoadID>>,
        kind: FilterKind,
    ) {
        let cmd = self.do_edit(self.add_modal_filter_cmd(pt, candidate_roads, kind));
        self.undo_stack.push(cmd);
        self.redo_queue.clear();
        self.after_edited();
    }

    fn add_modal_filter_cmd(
        &self,
        pt: Coord,
        candidate_roads: Option<&BTreeSet<RoadID>>,
        kind: FilterKind,
    ) -> Command {
        let (r, percent_along) = self.closest_point_on_road(pt, candidate_roads).unwrap();
        Command::SetModalFilter(
            r,
            Some(ModalFilter {
                percent_along,
                kind,
            }),
        )
    }

    fn closest_point_on_road(
        &self,
        click_pt: Coord,
        candidate_roads: Option<&BTreeSet<RoadID>>,
    ) -> Option<(RoadID, f64)> {
        // If candidate_roads is not specified, search around the point with a generous buffer
        let roads: Vec<RoadID> = if let Some(set) = candidate_roads {
            set.iter().cloned().collect()
        } else {
            let bbox = buffer_aabb(AABB::from_point(click_pt.into()), 50.0);
            self.closest_road
                .locate_in_envelope_intersecting(&bbox)
                .map(|r| r.data)
                .collect()
        };

        roads
            .into_iter()
            .filter_map(|r| {
                let road = self.get_r(r);
                if let Some(hit_pt) = match road.linestring.closest_point(&click_pt.into()) {
                    Closest::Intersection(pt) => Some(pt),
                    Closest::SinglePoint(pt) => Some(pt),
                    Closest::Indeterminate => None,
                } {
                    let score = Line::new(click_pt, hit_pt.into()).length::<Euclidean>();
                    let percent_along = road.linestring.line_locate_point(&hit_pt).unwrap();
                    Some(((score * 100.0) as usize, road.id, percent_along))
                } else {
                    None
                }
            })
            .min_by_key(|pair| pair.0)
            .map(|pair| (pair.1, pair.2))
    }

    fn most_similar_linestring(&self, linestring: &LineString) -> RoadID {
        // TODO Detect many possible cases of OSM data changing. Could at least compare the length
        // of the candidate. Decide how to handle possible splits/merges.
        self.roads
            .iter()
            .min_by_key(|r| {
                let diff1 = Line::new(
                    r.linestring.points().next().unwrap(),
                    linestring.points().next().unwrap(),
                )
                .length::<Euclidean>();
                let diff2 = Line::new(
                    r.linestring.points().last().unwrap(),
                    linestring.points().last().unwrap(),
                )
                .length::<Euclidean>();
                ((diff1 + diff2) * 100.0) as usize
            })
            .unwrap()
            .id
    }

    fn after_edited(&mut self) {
        // Invalidate it
        self.router_current = None;
    }

    pub fn add_many_modal_filters(
        &mut self,
        along_line: LineString,
        candidate_roads: &BTreeSet<RoadID>,
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

    pub fn toggle_direction(&mut self, r: RoadID) {
        let dir = match self.directions[&r] {
            Direction::Forwards => Direction::Backwards,
            Direction::Backwards => Direction::BothWays,
            Direction::BothWays => Direction::Forwards,
        };
        let cmd = self.do_edit(Command::SetDirection(r, dir));
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
            Command::SetDirection(r, dir) => {
                info!("changed direction of {r} to {}", dir.to_string());
                let prev = self.directions[&r];
                self.directions.insert(r, dir);
                Command::SetDirection(r, prev)
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
            let road = self.get_r(*r);
            let pt = road
                .linestring
                .line_interpolate_point(filter.percent_along)
                .unwrap();
            let angle = limit_angle(angle_of_pt_on_line(&road.linestring, pt.into()) + 90.0);
            let mut f = self.mercator.to_wgs84_gj(&pt);
            f.set_property("filter_kind", filter.kind.to_string());
            f.set_property("road", r.0);
            f.set_property("angle", angle);
            f.set_property("edited", Some(filter) != self.original_modal_filters.get(r));
            features.push(f);
        }
        FeatureCollection {
            features,
            bbox: None,
            foreign_members: None,
        }
    }

    pub fn to_savefile(&self) -> FeatureCollection {
        // Edited filters only
        let mut gj = self.filters_to_gj();
        gj.features
            .retain(|f| f.property("edited").unwrap().as_bool().unwrap());
        for f in &mut gj.features {
            f.set_property("kind", "modal_filter");
            f.remove_property("road");
        }

        // Look for any basemap filters that were deleted entirely
        for (r, filter) in &self.original_modal_filters {
            if self.modal_filters.contains_key(r) {
                continue;
            }
            let pt = self
                .get_r(*r)
                .linestring
                .line_interpolate_point(filter.percent_along)
                .unwrap();
            let mut f = self.mercator.to_wgs84_gj(&pt);
            f.set_property("kind", "deleted_existing_modal_filter");
            gj.features.push(f);
        }

        // Any direction edits
        for r in &self.roads {
            if self.directions[&r.id] != Direction::from_osm(&r.tags) {
                let mut f = self.mercator.to_wgs84_gj(&r.linestring);
                f.set_property("kind", "direction");
                f.set_property("direction", self.directions[&r.id].to_string());
                gj.features.push(f);
            }
        }

        gj.features.extend(self.boundaries.values().cloned());

        let mut f = Feature::from(Geometry::from(&self.boundary_wgs84));
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
        self.modal_filters = self.original_modal_filters.clone();
        for (r, dir) in &mut self.directions {
            *dir = Direction::from_osm(&self.roads[r.0].tags);
        }
        self.undo_stack.clear();
        self.redo_queue.clear();

        // Filters could be defined for multiple neighbourhoods, not just the one
        // in the savefile
        let mut cmds = Vec::new();

        for f in gj.features {
            match f.property("kind").unwrap().as_str().unwrap() {
                "modal_filter" => {
                    let kind = FilterKind::from_string(get_str_prop(&f, "filter_kind")?)?;
                    let gj_pt: Point = f.geometry.unwrap().try_into()?;
                    cmds.push(self.add_modal_filter_cmd(
                        self.mercator.pt_to_mercator(gj_pt.into()),
                        None,
                        kind,
                    ));
                }
                "deleted_existing_modal_filter" => {
                    let gj_pt: Point = f.geometry.unwrap().try_into()?;
                    let pt = self.mercator.pt_to_mercator(gj_pt.into());
                    // TODO Better error handling if we don't match
                    let (r, _) = self.closest_point_on_road(pt, None).unwrap();
                    cmds.push(Command::SetModalFilter(r, None));
                }
                "direction" => {
                    let dir = Direction::from_string(get_str_prop(&f, "direction")?)?;
                    let mut linestring: LineString = f.geometry.unwrap().try_into()?;
                    self.mercator.to_mercator_in_place(&mut linestring);
                    let r = self.most_similar_linestring(&linestring);
                    cmds.push(Command::SetDirection(r, dir));
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
                x => bail!("Unknown kind in savefile: {x}"),
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
    pub fn compare_route(&mut self, pt1: Coord, pt2: Coord, main_road_penalty: f64) -> GeoJson {
        if self
            .router_current
            .as_ref()
            .map(|r| r.main_road_penalty != main_road_penalty)
            .unwrap_or(true)
        {
            self.router_current = Some(Router::new(
                &self.roads,
                &self.modal_filters,
                &self.directions,
                main_road_penalty,
            ));
        }
        if self
            .router_original_with_penalty
            .as_ref()
            .map(|r| r.main_road_penalty != main_road_penalty)
            .unwrap_or(true)
        {
            self.router_original_with_penalty = Some(Router::new(
                &self.roads,
                &self.original_modal_filters,
                &self.original_directions(),
                main_road_penalty,
            ));
        }

        let mut features = Vec::new();
        if let Some(linestring) = self
            .router_original_with_penalty
            .as_ref()
            .unwrap()
            .route(self, pt1, pt2)
        {
            let mut f = self.mercator.to_wgs84_gj(&linestring);
            f.set_property("kind", "before");
            features.push(f);
        }
        if let Some(linestring) = self.router_current.as_ref().unwrap().route(self, pt1, pt2) {
            let mut f = self.mercator.to_wgs84_gj(&linestring);
            f.set_property("kind", "after");
            features.push(f);
        }
        GeoJson::from(features)
    }

    /// Return a polygon covering the world, minus a hole for the boundary, in WGS84
    pub fn invert_boundary(&self) -> Polygon {
        Polygon::new(
            LineString::from(vec![
                (180.0, 90.0),
                (-180.0, 90.0),
                (-180.0, -90.0),
                (180.0, -90.0),
                (180.0, 90.0),
            ]),
            vec![self.boundary_wgs84.exterior().clone()],
        )
    }

    fn original_directions(&self) -> BTreeMap<RoadID, Direction> {
        let mut directions = BTreeMap::new();
        for r in &self.roads {
            directions.insert(r.id, Direction::from_osm(&r.tags));
        }
        directions
    }
}

impl Road {
    pub fn length(&self) -> f64 {
        self.linestring.length::<Euclidean>()
    }

    pub fn to_gj(&self, mercator: &Mercator) -> Feature {
        let mut f = mercator.to_wgs84_gj(&self.linestring);
        // TODO Most of this is debug only
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

#[derive(Clone, PartialEq)]
pub struct ModalFilter {
    pub kind: FilterKind,
    pub percent_along: f64,
}

#[derive(Clone, Copy, PartialEq)]
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
            Self::WalkCycleOnly => "walk_cycle_only",
            Self::NoEntry => "no_entry",
            Self::BusGate => "bus_gate",
            Self::SchoolStreet => "school_street",
        }
    }

    pub fn from_string(x: &str) -> Result<Self> {
        match x {
            "walk_cycle_only" => Ok(Self::WalkCycleOnly),
            "no_entry" => Ok(Self::NoEntry),
            "bus_gate" => Ok(Self::BusGate),
            "school_street" => Ok(Self::SchoolStreet),
            _ => bail!("Invalid FilterKind: {x}"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Direction {
    Forwards,
    Backwards,
    BothWays,
}

impl Direction {
    pub fn from_osm(tags: &Tags) -> Self {
        // TODO Improve this
        if tags.is("oneway", "yes") {
            Self::Forwards
        } else if tags.is("oneway", "-1") {
            Self::Backwards
        } else {
            Self::BothWays
        }
    }

    // TODO strum?
    pub fn to_string(self) -> &'static str {
        match self {
            Self::Forwards => "forwards",
            Self::Backwards => "backwards",
            Self::BothWays => "both",
        }
    }

    pub fn from_string(x: &str) -> Result<Self> {
        match x {
            "forwards" => Ok(Self::Forwards),
            "backwards" => Ok(Self::Backwards),
            "both" => Ok(Self::BothWays),
            _ => bail!("Invalid Direction: {x}"),
        }
    }
}

pub enum Command {
    SetModalFilter(RoadID, Option<ModalFilter>),
    SetDirection(RoadID, Direction),
    Multiple(Vec<Command>),
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
