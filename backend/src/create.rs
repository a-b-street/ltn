use std::collections::{BTreeMap, BTreeSet, HashMap};

use anyhow::Result;
use geo::line_measures::InterpolatableLine;
use geo::{Coord, Euclidean, LineString, MultiPolygon};
use osm_reader::{NodeID, OsmID, RelationID, WayID};
use petgraph::graphmap::UnGraphMap;
use rstar::{primitives::GeomWithData, RTree};
use utils::{
    osm2graph::{EdgeID, Graph, OsmReader},
    Tags,
};

use crate::boundary_stats::{ContextData, POIKind, POI};
use crate::{
    impact::Impact, od::DemandModel, FilterKind, Intersection, IntersectionID, MapModel, Road,
    RoadID, Router, TravelFlow,
};

#[derive(Default)]
struct Osm {
    bus_routes_on_roads: HashMap<WayID, Vec<String>>,
    railways: Vec<LineString>,
    waterways: Vec<LineString>,
    barrier_nodes: BTreeSet<NodeID>,
    // Only represent one case of restricted turns (from, to) on a particular node
    turn_restrictions: HashMap<NodeID, Vec<(WayID, WayID)>>,
    pois: Vec<POI>,
}

impl OsmReader for Osm {
    fn node(&mut self, id: NodeID, pt: Coord, tags: Tags) {
        // Tuning these by hand for a few known areas.
        // https://wiki.openstreetmap.org/wiki/Key:barrier is proper reference.
        if let Some(kind) = tags.get("barrier") {
            // Bristol has many gates that don't seem as relevant
            if kind != "gate" {
                self.barrier_nodes.insert(id);
            }
        }

        self.pois.extend(get_poi(&tags, pt));
    }

    fn way(
        &mut self,
        _: WayID,
        node_ids: &Vec<NodeID>,
        node_mapping: &HashMap<NodeID, Coord>,
        tags: &Tags,
    ) {
        if node_ids.len() < 2 {
            return;
        }
        if tags.has("railway") && (!tags.has("layer") || tags.is("layer", "0")) {
            self.railways.push(LineString(
                node_ids.into_iter().map(|n| node_mapping[&n]).collect(),
            ));
        } else if tags.is_any("natural", vec!["water", "coastline"]) || tags.is("waterway", "dock")
        {
            // If the entire area is inside the study area, the LineString will be closed. If
            // it intersects the study area, then it might not be.
            self.waterways.push(LineString(
                node_ids.into_iter().map(|n| node_mapping[&n]).collect(),
            ));
        }

        self.pois.extend(get_poi(
            &tags,
            node_mapping[node_ids.into_iter().next().unwrap()],
        ));
    }

    fn relation(&mut self, _: RelationID, members: &Vec<(String, OsmID)>, tags: &Tags) {
        if tags.is("type", "route") && tags.is("route", "bus") {
            if let Some(name) = tags.get("name") {
                for (role, member) in members {
                    if let OsmID::Way(w) = member {
                        if role.is_empty() {
                            self.bus_routes_on_roads
                                .entry(*w)
                                .or_insert_with(Vec::new)
                                .push(name.to_string());
                        }
                    }
                }
            }
        }

        // https://wiki.openstreetmap.org/wiki/Relation:restriction describes many cases. Only handle
        // the simplest: a banned turn involving exactly 2 ways and 1 node.
        if tags.is("type", "restriction")
            && tags.is_any(
                "restriction",
                vec![
                    "no_right_turn",
                    "no_left_turn",
                    "no_u_turn",
                    "no_straight_on",
                ],
            )
        {
            let mut from = None;
            let mut via = None;
            let mut to = None;
            for (role, member) in members {
                match member {
                    OsmID::Way(w) => {
                        if role == "from" && from.is_none() {
                            from = Some(*w);
                        } else if role == "to" && to.is_none() {
                            to = Some(*w);
                        } else {
                            // Some other case, bail out
                            return;
                        }
                    }
                    OsmID::Node(n) => {
                        if role == "via" && via.is_none() {
                            via = Some(*n);
                        } else {
                            return;
                        }
                    }
                    OsmID::Relation(_) => {
                        return;
                    }
                }
            }

            if let (Some(from), Some(via), Some(to)) = (from, via, to) {
                self.turn_restrictions
                    .entry(via)
                    .or_insert_with(Vec::new)
                    .push((from, to));
            }
        }
    }
}

pub fn create_from_osm(
    input_bytes: &[u8],
    boundary_wgs84: MultiPolygon,
    study_area_name: Option<String>,
    demand: Option<DemandModel>,
    context_data_wgs84: Option<ContextData>,
) -> Result<MapModel> {
    let mut osm = Osm::default();
    let mut graph = Graph::new(input_bytes, is_road, &mut osm)?;
    remove_disconnected_components(&mut graph);
    graph.compact_ids();

    let context_data = context_data_wgs84.map(|mut context_data_wgs84| {
        context_data_wgs84.pois.extend(osm.pois);

        for population_zone in &mut context_data_wgs84.population_zones {
            graph
                .mercator
                .to_mercator_in_place(&mut population_zone.geometry);
        }
        for stats19_collision in &mut context_data_wgs84.stats19_collisions {
            graph.mercator.to_mercator_in_place(stats19_collision);
        }
        for poi in &mut context_data_wgs84.pois {
            graph.mercator.to_mercator_in_place(&mut poi.point);
        }
        context_data_wgs84.into_prepared()
    });

    // Add in a bit
    let roads: Vec<Road> = graph
        .edges
        .into_values()
        .map(|e| Road {
            id: RoadID(e.id.0),
            src_i: IntersectionID(e.src.0),
            dst_i: IntersectionID(e.dst.0),
            way: e.osm_way,
            linestring: e.linestring,
            speed_mph: parse_maxspeed_mph(&e.osm_tags),
            tags: e.osm_tags,
        })
        .collect();

    // Copy all the fields
    let intersections: Vec<Intersection> = graph
        .intersections
        .into_values()
        .map(|i| Intersection::from_graph(i, &roads))
        .collect();

    for ls in &mut osm.railways {
        graph.mercator.to_mercator_in_place(ls);
    }
    for ls in &mut osm.waterways {
        graph.mercator.to_mercator_in_place(ls);
    }

    info!("Building RTrees");
    let closest_road = RTree::bulk_load(
        roads
            .iter()
            .map(|r| GeomWithData::new(r.linestring.clone(), r.id))
            .collect(),
    );
    let closest_intersection = RTree::bulk_load(
        intersections
            .iter()
            .map(|i| GeomWithData::new(i.point, i.id))
            .collect(),
    );

    info!("Finalizing the map model");

    let mut travel_flows = BTreeMap::new();
    for r in &roads {
        travel_flows.insert(r.id, TravelFlow::from_osm(&r.tags));
    }

    let num_intersections = intersections.len();
    let mut map = MapModel {
        roads,
        intersections,
        bus_routes_on_roads: osm.bus_routes_on_roads,
        mercator: graph.mercator,
        boundary_wgs84,
        study_area_name,
        closest_road,
        closest_intersection,

        railways: osm.railways,
        waterways: osm.waterways,

        router_before: Router::empty(),
        router_after: None,
        router_before_with_penalty: None,

        original_modal_filters: BTreeMap::new(),
        modal_filters: BTreeMap::new(),
        diagonal_filters: BTreeMap::new(),

        turn_restrictions: std::iter::repeat_with(Vec::new)
            .take(num_intersections)
            .collect(),
        original_turn_restrictions: std::iter::repeat_with(Vec::new)
            .take(num_intersections)
            .collect(),

        travel_flows,

        impact: None,
        demand: None,

        undo_stack: Vec::new(),
        redo_queue: Vec::new(),
        boundaries: BTreeMap::new(),
        context_data,
    };
    if let Some(mut demand) = demand {
        demand.finish_loading(&map.mercator);
        map.impact = Some(Impact::new(&map, Some(&demand)));
        map.demand = Some(demand);
    } else {
        map.impact = Some(Impact::new(&map, None));
    }

    let graph = GraphSubset {
        node_to_edge: graph.node_to_edge,
        node_to_pt: graph.node_to_pt,
    };

    apply_existing_filters(&mut map, osm.barrier_nodes, &graph);
    apply_turn_restrictions(&mut map, osm.turn_restrictions);

    let router_before = Router::new(&map.router_input_before(), 1.0);
    map.router_before = router_before;

    Ok(map)
}

// Handles a partial borrow of Graph
struct GraphSubset {
    node_to_edge: HashMap<NodeID, EdgeID>,
    node_to_pt: HashMap<NodeID, Coord>,
}

fn apply_existing_filters(
    map: &mut MapModel,
    barrier_nodes: BTreeSet<NodeID>,
    graph: &GraphSubset,
) {
    // TODO Batch some or all of these initial edits?

    // Apply barriers on any surviving edges. RoadID and osm2graph::EdgeID are the same.
    for node in barrier_nodes {
        // If there's no surviving edge, then it was a barrier on something we don't consider a
        // road or on a road that was removed
        let Some(edge) = graph.node_to_edge.get(&node) else {
            continue;
        };
        let pt = map.mercator.pt_to_mercator(graph.node_to_pt[&node]);
        // TODO What FilterKind?
        map.add_modal_filter(pt, Some(vec![RoadID(edge.0)]), FilterKind::NoEntry);
    }

    // Look for roads tagged with access restrictions
    let pedestrian_roads: BTreeSet<RoadID> = map
        .roads
        .iter()
        .filter(|r| r.tags.is("highway", "pedestrian"))
        .map(|r| r.id)
        .collect();
    let bus_roads: BTreeSet<RoadID> = map
        .roads
        .iter()
        .filter(|r| {
            (r.tags.is("access", "no") || r.tags.is("motor_vehicle", "no"))
                && r.tags.is("bus", "yes")
        })
        .map(|r| r.id)
        .collect();
    for (roads, filter) in [
        (&pedestrian_roads, FilterKind::WalkCycleOnly),
        (&bus_roads, FilterKind::BusGate),
    ] {
        for r in roads.iter().cloned() {
            // TODO Should road-level filters override point barriers or not?
            // https://www.openstreetmap.org/way/448813838
            if map.modal_filters.contains_key(&r) {
                continue;
            }

            let (src_i, dst_i) = {
                let road = map.get_r(r);
                (road.src_i, road.dst_i)
            };
            // On each end of this road, is there a connecting unfiltered road?
            let src_unfiltered = map.get_i(src_i).roads.iter().any(|x| {
                *x != r
                    && !pedestrian_roads.contains(x)
                    && !bus_roads.contains(x)
                    && !map.modal_filters.contains_key(x)
            });
            let dst_unfiltered = map.get_i(dst_i).roads.iter().any(|x| {
                *x != r
                    && !pedestrian_roads.contains(x)
                    && !bus_roads.contains(x)
                    && !map.modal_filters.contains_key(x)
            });

            let percent = if src_unfiltered && dst_unfiltered {
                0.5
            } else if src_unfiltered {
                0.1
            } else if dst_unfiltered {
                0.9
            } else {
                // This is nestled between intersections withall filtered roads, so don't put
                // another point filter here
                continue;
            };

            // TODO Form commands directly?
            let pt = map
                .get_r(r)
                .linestring
                .point_at_ratio_from_start(&Euclidean, percent)
                .unwrap();
            map.add_modal_filter(pt.into(), Some(vec![r]), filter);
        }
    }

    // The commands above populate the existing modal filters and edit history. Undo that.
    map.original_modal_filters = map.modal_filters.clone();
    map.undo_stack.clear();
    map.redo_queue.clear();
}

fn apply_turn_restrictions(
    map: &mut MapModel,
    mut turn_restrictions: HashMap<NodeID, Vec<(WayID, WayID)>>,
) {
    for intersection in &map.intersections {
        if let Some(list) = turn_restrictions.remove(&intersection.node) {
            for (from_way, to_way) in list {
                // One OSM way turns into multiple Roads. The restriction only makes sense on the
                // Road connected to this intersection. So search only this intersection's roads.
                let mut from = None;
                let mut to = None;
                for r in &intersection.roads {
                    let way = map.roads[r.0].way;
                    if way == from_way {
                        from = Some(*r);
                    } else if way == to_way {
                        to = Some(*r);
                    }
                }

                if let (Some(from), Some(to)) = (from, to) {
                    // Set this directly; don't bother with Command and then fixing the undo/redo
                    // queues
                    map.original_turn_restrictions[intersection.id.0].push((from, to));
                }
            }
        }
    }

    map.turn_restrictions = map.original_turn_restrictions.clone();
}

fn is_road(tags: &Tags) -> bool {
    if !tags.has("highway") || tags.is("area", "yes") {
        return false;
    }
    if tags.is_any(
        "highway",
        vec![
            "cycleway",
            "footway",
            "steps",
            "path",
            "track",
            "corridor",
            "proposed",
            "construction",
        ],
    ) {
        return false;
    }
    true
}

// This uses OSM data when directly tagged, but otherwise makes some assumptions specific to
// Scotland.
// TODO Look at muv or https://github.com/westnordost/osm-legal-default-speeds for something more
// rigorous
fn parse_maxspeed_mph(tags: &Tags) -> usize {
    if let Some(maxspeed) = tags.get("maxspeed") {
        if let Ok(kmph) = maxspeed.parse::<f64>() {
            return (kmph * 0.621371).round() as usize;
        }
        if let Some(mph) = maxspeed
            .strip_suffix(" mph")
            .and_then(|x| x.parse::<f64>().ok())
        {
            return mph.round() as usize;
        }
    }

    // TODO Check these against osmactive
    match tags.get("highway").unwrap().as_str() {
        "motorway" | "motorway_link" => 70,
        "trunk" | "trunk_link" => 60,
        "primary" | "primary_link" => 40,
        "secondary" | "secondary_link" | "tertiary" | "tertiary_link" => 30,
        "residential" | "service" | "unclassified" => 20,
        "living_street" => 15,
        "pedestrian" => 10,
        // Should look into these
        _ => 10,
    }
}

// TODO Consider upstreaming to osm2graph
fn remove_disconnected_components(graph: &mut Graph) {
    let mut scc_graph: UnGraphMap<utils::osm2graph::IntersectionID, EdgeID> = UnGraphMap::new();
    for edge in graph.edges.values() {
        scc_graph.add_edge(edge.src, edge.dst, edge.id);
    }

    let mut components: Vec<BTreeSet<EdgeID>> = Vec::new();
    for nodes in petgraph::algo::kosaraju_scc(&scc_graph) {
        components.push(nodes_to_edges(graph, nodes));
    }
    components.sort_by_key(|scc| scc.len());
    components.reverse();

    let mut remove_edges = BTreeSet::new();
    // Keep only the largest component
    for scc in components.into_iter().skip(1) {
        info!("Removing component with only {} roads", scc.len());
        remove_edges.extend(scc);
    }

    info!("Removing {} disconnected roads", remove_edges.len());
    graph.remove_edges(remove_edges);
}

// Note this only works for connected components of nodes!
fn nodes_to_edges(graph: &Graph, nodes: Vec<utils::osm2graph::IntersectionID>) -> BTreeSet<EdgeID> {
    let mut edges = BTreeSet::new();
    for i in nodes {
        edges.extend(graph.intersections[&i].edges.clone());
    }
    edges
}

fn get_poi(tags: &Tags, point: Coord) -> Option<POI> {
    if tags.is_any("shop", vec!["convenience", "grocery", "supermarket"]) {
        return Some(POI {
            point: point.into(),
            kind: POIKind::Grocery,
            name: tags.get("name").cloned(),
        });
    }

    if tags.is_any("amenity", vec!["community_centre", "library"]) {
        return Some(POI {
            point: point.into(),
            kind: POIKind::CommunityCenter,
            name: tags.get("name").cloned(),
        });
    }

    None
}
