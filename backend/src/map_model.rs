use crate::boundary_stats::{ContextData, PreparedContextData};
use crate::geo_helpers::{
    aabb, angle_of_pt_on_line, bearing_from_endpoint, buffer_aabb, diagonal_bearing,
    invert_multi_polygon, limit_angle, linestring_intersection,
};
use crate::impact::Impact;
use crate::neighbourhood::{NeighbourhoodBoundary, NeighbourhoodDefinition};
use crate::route::RouterInput;
use crate::{od::DemandModel, Neighbourhood, Router};
use anyhow::Result;
use geo::{
    line_measures::InterpolatableLine, Closest, ClosestPoint, Coord, Distance, Euclidean, Length,
    LineLocatePoint, LineString, MultiPolygon, Point, Polygon, Simplify,
};
use geojson::{Feature, FeatureCollection, GeoJson, Geometry, JsonValue};
use rstar::{primitives::GeomWithData, RTree, AABB};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;
use utils::{osm2graph, Mercator, Tags};

pub struct MapModel {
    pub roads: Vec<Road>,
    pub intersections: Vec<Intersection>,
    pub bus_routes_on_roads: HashMap<osm_reader::WayID, Vec<String>>,
    // All geometry stored in worldspace, including rtrees
    pub mercator: Mercator,
    pub project_details: ProjectDetails,
    pub boundary_wgs84: MultiPolygon,
    pub closest_road: RTree<GeomWithData<LineString, RoadID>>,
    pub closest_intersection: RTree<GeomWithData<Point, IntersectionID>>,

    // Only those acting as severances; above or belowground don't count
    pub railways: Vec<LineString>,
    pub waterways: Vec<LineString>,

    // TODO Wasteful, can share some
    pub router_before: Router,
    // Calculated lazily. Changes with edits and main_road_penalty.
    pub router_after: Option<Router>,
    // Calculated lazily. No edits, just main_road_penalty.
    pub router_before_with_penalty: Option<Router>,

    // Just from the basemap, existing filters
    pub original_modal_filters: BTreeMap<RoadID, ModalFilter>,
    pub modal_filters: BTreeMap<RoadID, ModalFilter>,
    pub diagonal_filters: BTreeMap<IntersectionID, DiagonalFilter>,

    /// Indexed by IntersectionID. For each intersection, a list of (from, to) roads that are not
    /// allowed. May be redundant with the road TravelFlow.
    pub turn_restrictions: Vec<Vec<(RoadID, RoadID)>>,
    pub original_turn_restrictions: Vec<Vec<(RoadID, RoadID)>>,

    // Every road is filled out
    pub travel_flows: BTreeMap<RoadID, TravelFlow>,
    pub is_main_road: BTreeMap<RoadID, bool>,

    // Not optional, but wrapped for the borrow checker
    pub impact: Option<Impact>,
    pub demand: Option<DemandModel>,

    // TODO Keep edits / state here or not?
    pub undo_stack: Vec<Command>,
    pub redo_stack: Vec<Command>,
    pub boundaries: BTreeMap<String, NeighbourhoodBoundary>,

    pub context_data: Option<PreparedContextData>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
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

/// A segment of a road network - no intersections happen *within* a `Road`.
/// An osm Way is divided into potentially multiple `Road`s
#[derive(Clone)]
pub struct Road {
    pub id: RoadID,
    pub src_i: IntersectionID,
    pub dst_i: IntersectionID,
    pub way: osm_reader::WayID,
    pub linestring: LineString,
    pub tags: Tags,
    pub speed_mph: usize,
}

impl fmt::Debug for Road {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(&format!(
            "r({id}): i({src})-->i({dst})",
            id = self.id.0,
            src = self.src_i.0,
            dst = self.dst_i.0
        ))
        .field("way", &self.way.to_string())
        .field("linestring", &self.linestring)
        .field("tags", &self.tags)
        .field("speed_mph", &self.speed_mph)
        .finish()
    }
}

impl Road {
    /// By the road's existing OSM classification, is it a main road?
    pub fn is_severance(&self) -> bool {
        // PERF: is_any/has_any should take a const slice, not an owned vec... though maybe
        // the compiler is smart enough to optimize this.
        self.tags.is_any(
            "highway",
            vec![
                "motorway",
                "motorway_link",
                "trunk",
                "trunk_link",
                "primary",
                "primary_link",
                "secondary",
                "secondary_link",
                "tertiary",
                "tertiary_link",
            ],
        )
    }
}

/// Connection between `Road` (segments).
#[derive(Debug, Clone)]
pub struct Intersection {
    pub id: IntersectionID,
    pub node: osm_reader::NodeID,
    pub point: Point,
    // Ordered clockwise from North
    pub roads: Vec<RoadID>,
}

impl Intersection {
    pub(crate) fn from_graph(mut value: osm2graph::Intersection, roads: &[Road]) -> Self {
        // Sort intersection roads clockwise, starting from North
        value.edges.sort_by_cached_key(|road_id| {
            let road = &roads[road_id.0];
            let bearing = bearing_from_endpoint(value.point, &road.linestring);
            // work around that f64 is not Ord
            debug_assert!(
                bearing.is_finite(),
                "Assuming bearing output is always 0...360, this shouldn't happen"
            );
            (bearing * 1e6) as i64
        });

        Intersection {
            id: IntersectionID(value.id.0),
            point: value.point,
            node: value.osm_node,
            roads: value.edges.into_iter().map(|e| RoadID(e.0)).collect(),
        }
    }

    pub fn allowed_movements_from<'a>(
        &'a self,
        from_r: RoadID,
        router_input: &'a impl RouterInput,
    ) -> impl Iterator<Item = (RoadID, Direction)> + 'a {
        debug_assert!(
            self.roads.contains(&from_r),
            "{from_r:?} is not connected to intersection {self:?}"
        );
        let from_road = router_input.get_r(from_r);
        self.roads.iter().filter_map(move |road_id| {
            let to_road = router_input.get_r(*road_id);
            if to_road.id == from_road.id {
                return None;
            }
            if router_input.has_modal_filter(to_road.id) {
                return None;
            }
            if router_input
                .turn_restrictions(self.id)
                .contains(&(from_r, to_road.id))
            {
                return None;
            }
            if let Some(diagonal_filter) = router_input.diagonal_filter(self.id) {
                if !diagonal_filter.allows_movement(&(from_road.id, to_road.id)) {
                    return None;
                }
            }
            let travel_flow = router_input.travel_flow(to_road.id);
            if self.id == to_road.src_i && travel_flow.flows_forwards() {
                Some((to_road.id, Direction::Forwards))
            } else if self.id == to_road.dst_i && travel_flow.flows_backwards() {
                Some((to_road.id, Direction::Backwards))
            } else {
                None
            }
        })
    }

    pub fn allowed_movements(&self, router_input: &impl RouterInput) -> Vec<(RoadID, RoadID)> {
        let mut movements = vec![];
        for from_r in &self.roads {
            for (to_r, _) in self.allowed_movements_from(*from_r, router_input) {
                movements.push((*from_r, to_r))
            }
        }
        movements
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDetails {
    pub app_focus: String,
    pub study_area_name: Option<String>,
    pub project_name: String,
    pub db_schema_version: u32,
}

impl MapModel {
    /// Call with bytes of an osm.pbf or osm.xml string
    pub fn new(
        input_bytes: &[u8],
        boundary_wgs84: MultiPolygon,
        project_details: ProjectDetails,
        demand: Option<DemandModel>,
        context_data: Option<ContextData>,
    ) -> Result<MapModel> {
        crate::create::create_from_osm(
            input_bytes,
            boundary_wgs84,
            project_details,
            demand,
            context_data,
        )
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
        candidate_roads: Option<Vec<RoadID>>,
        kind: FilterKind,
    ) {
        let cmd = self.do_edit(self.add_modal_filter_cmd(pt, candidate_roads, kind));
        self.undo_stack.push(cmd);
        self.redo_stack.clear();
        self.after_edited();
    }

    fn add_modal_filter_cmd(
        &self,
        pt: Coord,
        candidate_roads: Option<Vec<RoadID>>,
        mut kind: FilterKind,
    ) -> Command {
        let (r, percent_along) = self.closest_point_on_road(pt, candidate_roads).unwrap();
        if self.get_bus_routes_on_road(r).is_some() && kind != FilterKind::BusGate {
            info!("Using a BusGate instead of {kind:?} for a road");
            kind = FilterKind::BusGate;
        }
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
        candidate_roads: Option<Vec<RoadID>>,
    ) -> Option<(RoadID, f64)> {
        // If candidate_roads is not specified, search around the point with a generous buffer
        let roads = candidate_roads.unwrap_or_else(|| {
            let bbox = buffer_aabb(AABB::from_point(click_pt.into()), 50.0);
            self.closest_road
                .locate_in_envelope_intersecting(&bbox)
                .map(|r| r.data)
                .collect()
        });

        roads
            .into_iter()
            .filter_map(|r| {
                let road = self.get_r(r);
                if let Some(hit_pt) = match road.linestring.closest_point(&click_pt.into()) {
                    Closest::Intersection(pt) => Some(pt),
                    Closest::SinglePoint(pt) => Some(pt),
                    Closest::Indeterminate => None,
                } {
                    let score = Euclidean.distance(click_pt, hit_pt.0);
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
                let diff1 = Euclidean.distance(
                    r.linestring.points().next().unwrap(),
                    linestring.points().next().unwrap(),
                );
                let diff2 = Euclidean.distance(
                    r.linestring.points().last().unwrap(),
                    linestring.points().last().unwrap(),
                );
                ((diff1 + diff2) * 100.0) as usize
            })
            .unwrap()
            .id
    }

    fn after_edited(&mut self) {
        self.router_after = None;
        self.impact.as_mut().unwrap().invalidate_after_edits();
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
                let mut use_kind = kind;
                if self.get_bus_routes_on_road(*r).is_some() && kind != FilterKind::BusGate {
                    info!("Using a BusGate instead of {kind:?} for a road");
                    use_kind = FilterKind::BusGate;
                }

                edits.push(Command::SetModalFilter(
                    *r,
                    Some(ModalFilter {
                        percent_along,
                        kind: use_kind,
                    }),
                ));
            }
        }
        let cmd = self.do_edit(Command::Multiple(edits));
        self.undo_stack.push(cmd);
        self.redo_stack.clear();
        self.after_edited();
    }

    pub fn delete_modal_filter(&mut self, r: RoadID) {
        let cmd = self.do_edit(Command::SetModalFilter(r, None));
        self.undo_stack.push(cmd);
        self.redo_stack.clear();
        self.after_edited();
    }

    pub fn add_diagonal_filter(&mut self, i: IntersectionID) {
        let intersection = self.get_i(i);
        let diagonal_filter = DiagonalFilter::new(intersection, false, self);
        let cmd = Command::SetDiagonalFilter(i, Some(diagonal_filter));
        let undo_cmd = self.do_edit(cmd);
        self.undo_stack.push(undo_cmd);
        self.redo_stack.clear();
        self.after_edited();
    }

    pub fn rotate_diagonal_filter(&mut self, i: IntersectionID) {
        let intersection = self.get_i(i);
        let diagonal_filter = DiagonalFilter::new(intersection, true, self);
        let cmd = Command::SetDiagonalFilter(i, Some(diagonal_filter));
        let undo_cmd = self.do_edit(cmd);
        self.undo_stack.push(undo_cmd);
        self.redo_stack.clear();
        self.after_edited();
    }

    pub fn delete_diagonal_filter(&mut self, i: IntersectionID) {
        let cmd = Command::SetDiagonalFilter(i, None);
        let undo_cmd = self.do_edit(cmd);
        self.undo_stack.push(undo_cmd);
        self.redo_stack.clear();
        self.after_edited();
    }

    pub fn add_turn_restriction(&mut self, from: RoadID, to: RoadID) -> Result<()> {
        let from = self.get_r(from);
        let to = self.get_r(to);
        let i = if from.src_i == to.src_i || from.src_i == to.dst_i {
            from.src_i
        } else if from.dst_i == to.src_i || from.dst_i == to.dst_i {
            from.dst_i
        } else {
            bail!("{} and {} don't share an intersection", from.id, to.id);
        };

        let mut restrictions = self.turn_restrictions[i.0].clone();
        if restrictions.contains(&(from.id, to.id)) {
            // The frontend should never do this, but just be idempotent
            return Ok(());
        }
        restrictions.push((from.id, to.id));

        let cmd = Command::SetTurnRestrictions(i, restrictions);
        let undo_cmd = self.do_edit(cmd);
        self.undo_stack.push(undo_cmd);
        self.redo_stack.clear();
        self.after_edited();
        Ok(())
    }

    pub fn delete_turn_restriction(
        &mut self,
        i: IntersectionID,
        from: RoadID,
        to: RoadID,
    ) -> Result<()> {
        let mut restrictions = self.turn_restrictions[i.0].clone();
        restrictions.retain(|(a, b)| (*a, *b) != (from, to));

        let cmd = Command::SetTurnRestrictions(i, restrictions);
        let undo_cmd = self.do_edit(cmd);
        self.undo_stack.push(undo_cmd);
        self.redo_stack.clear();
        self.after_edited();
        Ok(())
    }

    pub fn toggle_travel_flow(&mut self, r: RoadID) {
        let dir = match self.travel_flows[&r] {
            TravelFlow::FORWARDS => TravelFlow::BACKWARDS,
            TravelFlow::BACKWARDS => TravelFlow::BothWays,
            TravelFlow::BothWays => TravelFlow::FORWARDS,
        };
        let cmd = self.do_edit(Command::SetTravelFlow(r, dir));
        self.undo_stack.push(cmd);
        self.redo_stack.clear();
        self.after_edited();
    }

    pub fn toggle_main_road(&mut self, r: RoadID) {
        let is_main_road = !self.is_main_road[&r];
        let cmd = self.do_edit(Command::SetMainRoad(r, is_main_road));
        self.undo_stack.push(cmd);
        self.redo_stack.clear();
        self.after_edited();
    }

    fn roads_along_line(
        &self,
        neighbourhood: &Neighbourhood,
        line_string: LineString,
    ) -> Vec<RoadID> {
        let line_string = line_string.simplify(&0.5);
        let bbox = aabb(&line_string);
        let buffered_bbox = buffer_aabb(bbox, 20.);
        let road_iter = self
            .closest_road
            .locate_in_envelope_intersecting(&buffered_bbox)
            .filter(|r| {
                neighbourhood.interior_roads.contains(&r.data)
                    || neighbourhood.main_roads.contains(&r.data)
            })
            .map(|road_node| self.get_r(road_node.data));

        crate::geo_helpers::roads_along_line(road_iter, &line_string)
    }

    pub fn reclassify_roads_along_line(
        &mut self,
        neighbourhood: &Neighbourhood,
        line_string: LineString,
        is_main_road: bool,
        add_to_undo_stack: bool,
    ) {
        let roads_along = self.roads_along_line(neighbourhood, line_string);
        let cmds = roads_along
            .iter()
            .map(|r| Command::SetMainRoad(*r, is_main_road))
            .collect();
        let cmd = Command::Multiple(cmds);

        let undo_cmd = self.do_edit(cmd);
        if add_to_undo_stack {
            if let Some(prev_cmd) = self.undo_stack.first() {
                if prev_cmd == &undo_cmd {
                    // Don't add no-op undo commands.
                    // e.g. when re-classifying an area the same as it was already classified.
                    //
                    // I originally though we could bail out earlier, by seeing if any of
                    // `roads_along` had changed, but then that would prevent us from *ever*
                    // recording the undo action in the case of our incremental updates (see `add_to_undo_stack`)
                    return;
                }
            }
            self.undo_stack.push(undo_cmd);
            self.redo_stack.clear();
        }
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
            Command::SetDiagonalFilter(i, filter) => {
                let prev = self.diagonal_filters.get(&i).cloned();
                if let Some(filter) = filter {
                    info!("added filter to {i:?}: {filter:?}");
                    self.diagonal_filters.insert(i, filter);
                } else {
                    let filter = self.diagonal_filters.remove(&i);
                    info!("removed filter from {i:?}: {filter:?}");
                }
                Command::SetDiagonalFilter(i, prev)
            }
            Command::SetTravelFlow(r, dir) => {
                info!("changed travel flow of {r} to {}", dir.to_string());
                let prev = self.travel_flows[&r];
                self.travel_flows.insert(r, dir);
                Command::SetTravelFlow(r, prev)
            }
            Command::SetMainRoad(r, is_main_road) => {
                info!("changed {r} to now be a main road = {is_main_road}");
                self.is_main_road.insert(r, is_main_road);
                Command::SetMainRoad(r, !is_main_road)
            }
            Command::SetTurnRestrictions(i, mut restrictions) => {
                std::mem::swap(&mut self.turn_restrictions[i.0], &mut restrictions);
                Command::SetTurnRestrictions(i, restrictions)
            }
            Command::Multiple(list) => {
                let undo_list = list.into_iter().map(|cmd| self.do_edit(cmd)).collect();
                Command::Multiple(undo_list)
            }
        }
    }

    /// Returns the command that was reverted.
    pub fn undo(&mut self) -> Option<Command> {
        // The UI shouldn't call this when the stack is empty, but when holding down the redo key,
        // it doesn't update fast enough
        let cmd = self.undo_stack.pop()?;
        let redo_cmd = self.do_edit(cmd.clone());
        self.redo_stack.push(redo_cmd);
        self.after_edited();
        Some(cmd)
    }

    /// Returns the command that was applied.
    pub fn redo(&mut self) -> Option<Command> {
        let cmd = self.redo_stack.pop()?;
        let undo_cmd = self.do_edit(cmd.clone());
        self.undo_stack.push(undo_cmd);
        self.after_edited();
        Some(cmd)
    }

    // NOTE: this method is used both for saving and for serializing to the frontend,
    // but for ModalFilters and DiagonalFilters we need different information in each case. It might be good
    // to split up this functionality
    pub fn filters_to_gj(&self) -> FeatureCollection {
        let mut features = Vec::new();
        for (r, filter) in &self.modal_filters {
            let road = self.get_r(*r);
            let pt = road
                .linestring
                .point_at_ratio_from_start(&Euclidean, filter.percent_along)
                .unwrap();
            let angle = limit_angle(angle_of_pt_on_line(&road.linestring, pt.into()) + 90.0);
            let mut f = self.mercator.to_wgs84_gj(&pt);
            f.set_property("filter_kind", filter.kind.to_string());
            f.set_property("road", r.0);
            f.set_property("angle", angle);
            f.set_property("edited", Some(filter) != self.original_modal_filters.get(r));
            features.push(f);
        }
        for (i, filter) in &self.diagonal_filters {
            let intersection = self.get_i(*i);
            let mut f = self.mercator.to_wgs84_gj(&intersection.point);
            f.set_property("filter_kind", FilterKind::DiagonalFilter.to_string());
            f.set_property("intersection_id", i.0);
            f.set_property("filter", filter);
            // part of being a "filter"
            f.set_property("edited", true);
            features.push(f);
        }
        FeatureCollection {
            features,
            bbox: None,
            foreign_members: None,
        }
    }

    /// Because ids like RoadID and IntersectionID aren't guaranteed to be stable across loads,
    /// we use more permanent markers like GPS points to map to features.
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
                .point_at_ratio_from_start(&Euclidean, filter.percent_along)
                .unwrap();
            let mut f = self.mercator.to_wgs84_gj(&pt);
            f.set_property("kind", "deleted_existing_modal_filter");
            gj.features.push(f);
        }

        // Any travel flow or main road edits
        for r in &self.roads {
            if self.travel_flows[&r.id] != TravelFlow::from_osm(&r.tags) {
                let mut f = self.mercator.to_wgs84_gj(&r.linestring);
                f.set_property("kind", "travel_flow");
                f.set_property("travel_flow", self.travel_flows[&r.id].to_string());
                gj.features.push(f);
            }

            if self.is_main_road[&r.id] != r.is_severance() {
                let mut f = self.mercator.to_wgs84_gj(&r.linestring);
                f.set_property("kind", "main_road");
                f.set_property("is_main_road", self.is_main_road[&r.id]);
                gj.features.push(f);
            }
        }

        // Edited turn restrictions only
        for (idx, turn_restrictions) in self.turn_restrictions.iter().enumerate() {
            for (from, to) in turn_restrictions {
                if self.original_turn_restrictions[idx].contains(&(*from, *to)) {
                    continue;
                }

                let intersection = self.get_i(IntersectionID(idx));
                let mut f = self.mercator.to_wgs84_gj(&intersection.point);
                f.set_property("kind", "turn_restriction");
                // Identify the two roads by their absolute bearings
                let (abs_bearing_1, abs_bearing_2) =
                    intersection.bearing_of_roads(self.get_r(*from), self.get_r(*to));
                f.set_property("bearing1", abs_bearing_1.round());
                f.set_property("bearing2", abs_bearing_2.round());
                gj.features.push(f);
            }
        }

        // Look for any basemap turn restrictions that were deleted
        for (idx, turn_restrictions) in self.original_turn_restrictions.iter().enumerate() {
            for (from, to) in turn_restrictions {
                if self.turn_restrictions[idx].contains(&(*from, *to)) {
                    continue;
                }

                let intersection = self.get_i(IntersectionID(idx));
                let mut f = self.mercator.to_wgs84_gj(&intersection.point);
                f.set_property("kind", "deleted_existing_turn_restriction");
                let (abs_bearing_1, abs_bearing_2) =
                    intersection.bearing_of_roads(self.get_r(*from), self.get_r(*to));
                f.set_property("bearing1", abs_bearing_1.round());
                f.set_property("bearing2", abs_bearing_2.round());
                gj.features.push(f);
            }
        }

        for neighbourhood_boundary in self.boundaries.values() {
            // we don't save the derived "stats" just the boundary definition
            gj.features
                .push(neighbourhood_boundary.definition.to_feature(self))
        }

        let mut f = Feature::from(Geometry::from(&self.boundary_wgs84));
        f.set_property("kind", "study_area_boundary");
        gj.features.push(f);

        gj.foreign_members = Some(
            // The features are elements within the study area, we store properties of the
            // project itself as foreign members.
            serde_json::json!(&self.project_details)
                .as_object()
                .unwrap()
                .to_owned(),
        );
        gj
    }

    pub fn load_savefile(&mut self, gj: FeatureCollection) -> Result<()> {
        // Clear previous state
        self.boundaries.clear();
        self.modal_filters = self.original_modal_filters.clone();
        self.turn_restrictions = self.original_turn_restrictions.clone();
        for (r, dir) in &mut self.travel_flows {
            *dir = TravelFlow::from_osm(&self.roads[r.0].tags);
        }
        for (r, is_main_road) in &mut self.is_main_road {
            *is_main_road = self.roads[r.0].is_severance();
        }
        self.undo_stack.clear();
        self.redo_stack.clear();

        // Filters could be defined for multiple neighbourhoods, not just the one
        // in the savefile
        let mut cmds = Vec::new();

        for f in gj.features {
            match f
                .property("kind")
                .expect("savefile feature missing `kind`")
                .as_str()
                .unwrap()
            {
                "modal_filter" => {
                    let kind = FilterKind::from_string(get_str_prop(&f, "filter_kind")?)?;
                    let gj_pt: Point = f.geometry.as_ref().unwrap().try_into()?;
                    match kind {
                        FilterKind::DiagonalFilter => {
                            let i = {
                                let pt = self.mercator.pt_to_mercator(gj_pt.into());
                                self.closest_intersection
                                    .nearest_neighbor(&Point(pt))
                                    .expect("intersection near saved editable intersection")
                                    .data
                            };
                            let intersection = self.get_i(i);
                            let filter = f.property("filter").unwrap().as_object().unwrap();
                            let is_rotated = filter
                                .get("is_rotated")
                                .expect("missing is_rotated")
                                .as_bool()
                                .expect("expected a bool");

                            let diagonal_filter =
                                DiagonalFilter::new(intersection, is_rotated, self);
                            self.diagonal_filters
                                .insert(intersection.id, diagonal_filter);
                        }
                        _ => {
                            cmds.push(self.add_modal_filter_cmd(
                                self.mercator.pt_to_mercator(gj_pt.into()),
                                None,
                                kind,
                            ));
                        }
                    }
                }
                "deleted_existing_modal_filter" => {
                    let gj_pt: Point = f.geometry.unwrap().try_into()?;
                    let pt = self.mercator.pt_to_mercator(gj_pt.into());
                    // TODO Better error handling if we don't match
                    let (r, _) = self.closest_point_on_road(pt, None).unwrap();
                    cmds.push(Command::SetModalFilter(r, None));
                }
                "travel_flow" => {
                    let dir = TravelFlow::from_string(get_str_prop(&f, "travel_flow")?)?;
                    let mut linestring: LineString = f.geometry.unwrap().try_into()?;
                    self.mercator.to_mercator_in_place(&mut linestring);
                    let r = self.most_similar_linestring(&linestring);
                    cmds.push(Command::SetTravelFlow(r, dir));
                }
                "main_road" => {
                    let is_main_road = get_bool_prop(&f, "is_main_road")?;
                    let mut linestring: LineString = f.geometry.unwrap().try_into()?;
                    self.mercator.to_mercator_in_place(&mut linestring);
                    let r = self.most_similar_linestring(&linestring);
                    cmds.push(Command::SetMainRoad(r, is_main_road));
                }
                "turn_restriction" => {
                    let bearing1 = get_f64_prop(&f, "bearing1")?;
                    let bearing2 = get_f64_prop(&f, "bearing2")?;
                    let gj_pt: Point = f.geometry.unwrap().try_into()?;
                    let pt = self.mercator.pt_to_mercator(gj_pt.into());
                    let (i, from, to) =
                        self.find_turn_restriction(pt.into(), bearing1, bearing2)?;
                    self.turn_restrictions[i.0].push((from, to));
                }
                "deleted_existing_turn_restriction" => {
                    let bearing1 = get_f64_prop(&f, "bearing1")?;
                    let bearing2 = get_f64_prop(&f, "bearing2")?;
                    let gj_pt: Point = f.geometry.unwrap().try_into()?;
                    let pt = self.mercator.pt_to_mercator(gj_pt.into());
                    let (i, from, to) =
                        self.find_turn_restriction(pt.into(), bearing1, bearing2)?;
                    self.turn_restrictions[i.0].retain(|(a, b)| (*a, *b) != (from, to));
                }
                "boundary" => {
                    let name = get_str_prop(&f, "name")?.to_string();
                    if self.boundaries.contains_key(&name) {
                        bail!("Multiple boundaries named {name} in savefile");
                    }
                    let neighbourhood_definition = NeighbourhoodDefinition::from_feature(f, self)?;
                    let neighbourhood_boundary = NeighbourhoodBoundary::new(
                        neighbourhood_definition,
                        self.context_data.as_ref(),
                    );
                    self.boundaries.insert(name, neighbourhood_boundary);
                }
                "study_area_boundary" => {
                    // TODO Detect if it's close enough to boundary_polygon? Overwrite?
                }
                x => bail!("Unknown kind in savefile: {x}"),
            }
        }

        // Keep the undo stack empty. A user shouldn't be able to undo and clear the whole
        // savefile.
        self.do_edit(Command::Multiple(cmds));
        self.after_edited();

        Ok(())
    }

    pub fn router_input_before(&self) -> impl RouterInput + use<'_> {
        struct RouterInputBefore<'a> {
            map: &'a MapModel,
        }
        impl RouterInput for RouterInputBefore<'_> {
            fn roads_iter(&self) -> impl Iterator<Item = &Road> {
                self.map.roads.iter()
            }

            fn get_r(&self, r: RoadID) -> &Road {
                self.map.get_r(r)
            }

            fn get_i(&self, i: IntersectionID) -> &Intersection {
                self.map.get_i(i)
            }

            fn modal_filter(&self, r: RoadID) -> Option<&ModalFilter> {
                self.map.original_modal_filters.get(&r)
            }

            fn travel_flow(&self, r: RoadID) -> TravelFlow {
                let road = self.get_r(r);
                TravelFlow::from_osm(&road.tags)
            }

            fn diagonal_filter(&self, _i: IntersectionID) -> Option<&DiagonalFilter> {
                // We don't import pre-existing diagonal filters from OSM
                // As there isn't a well known tagging / topological structure that we can easily identify.
                None
            }

            fn turn_restrictions(&self, i: IntersectionID) -> &Vec<(RoadID, RoadID)> {
                &self.map.original_turn_restrictions[i.0]
            }
        }

        RouterInputBefore { map: self }
    }

    pub fn router_input_after(&self) -> impl RouterInput + use<'_> {
        struct RouterInputAfter<'a> {
            map: &'a MapModel,
        }
        impl RouterInput for RouterInputAfter<'_> {
            fn roads_iter(&self) -> impl Iterator<Item = &Road> {
                self.map.roads.iter()
            }

            fn get_r(&self, r: RoadID) -> &Road {
                self.map.get_r(r)
            }

            fn get_i(&self, i: IntersectionID) -> &Intersection {
                self.map.get_i(i)
            }

            fn modal_filter(&self, r: RoadID) -> Option<&ModalFilter> {
                self.map.modal_filters.get(&r)
            }

            fn travel_flow(&self, r: RoadID) -> TravelFlow {
                self.map.travel_flows[&r]
            }

            fn diagonal_filter(&self, i: IntersectionID) -> Option<&DiagonalFilter> {
                self.map.diagonal_filters.get(&i)
            }

            fn turn_restrictions(&self, i: IntersectionID) -> &Vec<(RoadID, RoadID)> {
                &self.map.turn_restrictions[i.0]
            }
        }
        RouterInputAfter { map: self }
    }

    // Lazily builds the router if needed.
    pub fn rebuild_router(&mut self, main_road_penalty: f64) {
        if self
            .router_before_with_penalty
            .as_ref()
            .map(|r| r.main_road_penalty != main_road_penalty)
            .unwrap_or(true)
        {
            let router_before_with_penalty =
                Router::new(&self.router_input_before(), main_road_penalty);
            self.router_before_with_penalty = Some(router_before_with_penalty);
        }

        if self
            .router_after
            .as_ref()
            .map(|r| r.main_road_penalty != main_road_penalty)
            .unwrap_or(true)
        {
            let router_after = Router::new(&self.router_input_after(), main_road_penalty);
            self.router_after = Some(router_after);
        }
    }

    pub fn compare_route(&mut self, pt1: Coord, pt2: Coord, main_road_penalty: f64) -> GeoJson {
        self.rebuild_router(main_road_penalty);

        let mut features = Vec::new();
        if let Some(route) = self
            .router_before_with_penalty
            .as_ref()
            .unwrap()
            .route(self, pt1, pt2)
        {
            let (distance, time) = route.get_distance_and_time(self);
            let mut f = self.mercator.to_wgs84_gj(&route.to_linestring(self));
            f.set_property("kind", "before");
            f.set_property("distance", distance);
            f.set_property("time", time);
            features.push(f);
        }
        if let Some(route) = self.router_after.as_ref().unwrap().route(self, pt1, pt2) {
            let (distance, time) = route.get_distance_and_time(self);
            let mut f = self.mercator.to_wgs84_gj(&route.to_linestring(self));
            f.set_property("kind", "after");
            f.set_property("distance", distance);
            f.set_property("time", time);
            features.push(f);
        }
        GeoJson::from(features)
    }

    pub fn impact_to_one_destination(
        &mut self,
        pt2: Coord,
        from: Vec<RoadID>,
    ) -> FeatureCollection {
        // main_road_penalty doesn't seem relevant for this question
        self.rebuild_router(1.0);

        // From every road, calculate the route before and after to the one destination
        let mut features = Vec::new();
        let mut highest_time_ratio: f64 = 1.0;
        for r in from {
            let road = self.get_r(r);
            let pt1 = road
                .linestring
                .point_at_ratio_from_start(&Euclidean, 0.5)
                .unwrap()
                .into();

            // TODO How to handle missing one or both routes missing?
            if let (Some(before), Some(after)) = (
                self.router_before_with_penalty
                    .as_ref()
                    .unwrap()
                    .route(self, pt1, pt2),
                self.router_after.as_ref().unwrap().route(self, pt1, pt2),
            ) {
                let from_pt = self.mercator.pt_to_wgs84(pt1);
                let (distance_before, time_before) = before.get_distance_and_time(self);
                let (distance_after, time_after) = after.get_distance_and_time(self);

                let mut f = self.mercator.to_wgs84_gj(&road.linestring);
                f.set_property("distance_before", distance_before);
                f.set_property("distance_after", distance_after);
                f.set_property("time_before", time_before);
                f.set_property("time_after", time_after);
                f.set_property("pt1_x", from_pt.x);
                f.set_property("pt1_y", from_pt.y);
                features.push(f);

                highest_time_ratio = highest_time_ratio.max(time_after / time_before);
            }
        }

        FeatureCollection {
            features,
            bbox: None,
            foreign_members: Some(
                serde_json::json!({
                    "highest_time_ratio": highest_time_ratio,
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        }
    }

    /// Return a polygon covering the world, minus a hole for the study area boundary, in WGS84
    pub fn invert_study_area_boundary(&self) -> Polygon {
        invert_multi_polygon(self.boundary_wgs84.clone())
    }

    /// What're the names of bus routes along a road?
    pub fn get_bus_routes_on_road(&self, r: RoadID) -> Option<&Vec<String>> {
        let way = self.get_r(r).way;
        self.bus_routes_on_roads.get(&way)
    }
}

impl Road {
    // How long does it take for a car following the speed limit to cross this road?
    pub fn cost_seconds(&self) -> f64 {
        let meters = Euclidean.length(&self.linestring);
        let meters_per_second = (self.speed_mph as f64) * 0.44704;
        meters / meters_per_second
    }

    pub fn to_gj(&self, mercator: &Mercator) -> Feature {
        let mut f = mercator.to_wgs84_gj(&self.linestring);
        f.set_property("id", self.id.0);
        f.set_property("speed_mph", self.speed_mph);
        // TODO Debug only, reconsider
        f.set_property("way", self.way.to_string());
        for (k, v) in &self.tags.0 {
            f.set_property(k, v.to_string());
        }
        f
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModalFilter {
    pub kind: FilterKind,
    pub percent_along: f64,
}

/// A DiagonalFilter is placed at a 4-way intersection, and prevents traffic from going "straight"
/// through the intersection. Traffic must turn.
///
/// The DiagonalFilter can be placed in one of two rotations to determine which way traffic is forced
/// to turn.
///
/// Note: When all the roads at the intersection are 1-way roads, there is only one reasonable
/// orientation for the diagonal filter, the other orientation would effectively block the intersection.
/// We could choose to enforce "reasonable" filtering in the UI, or keep the interface consistent
/// and leave it up to the user to manually ensure the filter is orientated reasonably.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DiagonalFilter {
    /// Travel within these roads are allowed, but not to the other group.
    pub group_a: Vec<RoadID>,
    /// Travel within these roads are allowed, but not to the other group.
    pub group_b: Vec<RoadID>,
    /// The topological orientation of the filter - it determines how `intersection.roads` are split
    /// into `group_a` and `group_b`.
    pub is_rotated: bool,
    /// How many degrees to rotate a vertical line to split `group_a` from `group_b`
    pub angle: f32,
}

impl DiagonalFilter {
    /// Precondition: Intersection must be a 4-way intersection
    fn new(intersection: &Intersection, is_rotated: bool, map_model: &MapModel) -> DiagonalFilter {
        debug_assert_eq!(
            intersection.roads.len(),
            4,
            "diagonal filters only support 4-way intersections"
        );

        let split_offset = if is_rotated { 1 } else { 0 };

        let group_a: Vec<RoadID> = (0..2)
            .map(|offset| intersection.roads[(offset + split_offset) % intersection.roads.len()])
            .collect();

        let group_b: Vec<RoadID> = (2..4)
            .map(|offset| intersection.roads[(offset + split_offset) % intersection.roads.len()])
            .collect();

        let road_1 = map_model.get_r(group_a[0]);
        let road_2 = map_model.get_r(group_a[1]);

        let bearing_1 = bearing_from_endpoint(intersection.point, &road_1.linestring);
        let bearing_2 = bearing_from_endpoint(intersection.point, &road_2.linestring);
        let angle = diagonal_bearing(bearing_1, bearing_2) as f32;
        DiagonalFilter {
            group_a,
            group_b,
            is_rotated,
            angle,
        }
    }

    // `movement`: (from, to)
    pub fn allows_movement(&self, movement: &(RoadID, RoadID)) -> bool {
        let (from, to) = movement;

        debug_assert!(self.group_a.contains(from) || self.group_b.contains(from));
        debug_assert!(self.group_a.contains(to) || self.group_b.contains(to));

        (self.group_a.contains(from) && self.group_a.contains(to))
            || (self.group_b.contains(from) && self.group_b.contains(to))
    }
}

impl From<&DiagonalFilter> for JsonValue {
    fn from(value: &DiagonalFilter) -> Self {
        serde_json::to_value(value).expect("valid JSON fields")
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FilterKind {
    WalkCycleOnly,
    NoEntry,
    BusGate,
    SchoolStreet,
    DiagonalFilter,
}

// TODO strum?
impl FilterKind {
    pub fn to_string(self) -> &'static str {
        match self {
            Self::WalkCycleOnly => "walk_cycle_only",
            Self::NoEntry => "no_entry",
            Self::BusGate => "bus_gate",
            Self::SchoolStreet => "school_street",
            Self::DiagonalFilter => "diagonal_filter",
        }
    }

    pub fn from_string(x: &str) -> Result<Self> {
        match x {
            "walk_cycle_only" => Ok(Self::WalkCycleOnly),
            "no_entry" => Ok(Self::NoEntry),
            "bus_gate" => Ok(Self::BusGate),
            "school_street" => Ok(Self::SchoolStreet),
            "diagonal_filter" => Ok(Self::DiagonalFilter),
            _ => bail!("Invalid FilterKind: {x}"),
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, PartialOrd, Ord, Eq, Serialize, Deserialize)]
pub enum TravelFlow {
    BothWays,
    OneWay(Direction),
}
impl TravelFlow {
    pub const FORWARDS: TravelFlow = TravelFlow::OneWay(Direction::Forwards);
    pub const BACKWARDS: TravelFlow = TravelFlow::OneWay(Direction::Backwards);

    pub fn flows_forwards(self) -> bool {
        matches!(self, TravelFlow::FORWARDS | TravelFlow::BothWays)
    }
    pub fn flows_backwards(self) -> bool {
        matches!(self, TravelFlow::BACKWARDS | TravelFlow::BothWays)
    }
}

#[derive(Clone, Debug, Copy, PartialEq, PartialOrd, Ord, Eq, Serialize, Deserialize)]
pub enum Direction {
    Forwards,
    Backwards,
}

impl TravelFlow {
    pub fn from_osm(tags: &Tags) -> Self {
        // TODO Improve this
        if tags.is("oneway", "yes") {
            Self::FORWARDS
        } else if tags.is("oneway", "-1") {
            Self::BACKWARDS
        } else {
            // https://wiki.openstreetmap.org/wiki/Key:oneway#Implied_oneway_restriction
            if tags.is("highway", "motorway") || tags.is("junction", "roundabout") {
                return Self::FORWARDS;
            }

            Self::BothWays
        }
    }

    pub fn to_string(self) -> &'static str {
        match self {
            Self::FORWARDS => "forwards",
            Self::BACKWARDS => "backwards",
            Self::BothWays => "both",
        }
    }

    pub fn from_string(x: &str) -> Result<Self> {
        match x {
            "forwards" => Ok(Self::FORWARDS),
            "backwards" => Ok(Self::BACKWARDS),
            "both" => Ok(Self::BothWays),
            _ => bail!("Invalid Direction: {x}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    SetModalFilter(RoadID, Option<ModalFilter>),
    SetDiagonalFilter(IntersectionID, Option<DiagonalFilter>),
    SetTravelFlow(RoadID, TravelFlow),
    SetMainRoad(RoadID, bool),
    SetTurnRestrictions(IntersectionID, Vec<(RoadID, RoadID)>),
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

fn get_f64_prop<'a>(f: &'a Feature, key: &str) -> Result<f64> {
    let Some(value) = f.property(key) else {
        bail!("Feature doesn't have a {key} property");
    };
    let Some(n) = value.as_f64() else {
        bail!("Feature's {key} property isn't a f64");
    };
    Ok(n)
}

fn get_bool_prop<'a>(f: &'a Feature, key: &str) -> Result<bool> {
    let Some(value) = f.property(key) else {
        bail!("Feature doesn't have a {key} property");
    };
    let Some(x) = value.as_bool() else {
        bail!("Feature's {key} property isn't a boolean");
    };
    Ok(x)
}
