use std::collections::{BTreeMap, BTreeSet, HashMap};

use anyhow::Result;
use geo::line_measures::InterpolatableLine;
use geo::{Coord, Euclidean, MultiPolygon};
use osm_reader::{NodeID, WayID};
use petgraph::graphmap::UnGraphMap;
use rstar::{primitives::GeomWithData, RTree};
use utils::osm2graph::{EdgeID, Graph};

use crate::boundary_stats::ContextData;
use crate::{
    impact::Impact, od::DemandModel, FilterKind, Intersection, IntersectionID, MapModel, Road,
    RoadID, Router, TravelFlow,
};

mod dog_leg;
mod parse;

pub fn create_from_osm(
    input_bytes: &[u8],
    boundary_wgs84: MultiPolygon,
    demand: Option<DemandModel>,
    mut serialized_context_data: Option<ContextData>,
) -> Result<MapModel> {
    let mut osm = parse::Osm::default();
    let mut graph = Graph::new(input_bytes, parse::is_road, &mut osm)?;
    remove_disconnected_components(&mut graph);
    info!("Collapsing dog-leg intersections");
    dog_leg::collapse_dog_legs(&mut graph);
    graph.compact_ids();

    if let Some(ref mut serialized_context_data) = serialized_context_data {
        serialized_context_data.pois.extend(osm.pois);
    }

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
            speed_mph: parse::parse_maxspeed_mph(&e.osm_tags),
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
    let mut is_main_road = BTreeMap::new();
    for r in &roads {
        travel_flows.insert(r.id, TravelFlow::from_osm(&r.tags));
        is_main_road.insert(r.id, r.is_severance());
    }

    let num_intersections = intersections.len();
    let mut map = MapModel {
        roads,
        intersections,
        bus_routes_on_roads: osm.bus_routes_on_roads,
        mercator: graph.mercator,
        boundary_wgs84,
        project_details: None,
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
        is_main_road,

        impact: Some(Impact::default()),
        demand: None,

        undo_stack: Vec::new(),
        redo_stack: Vec::new(),
        reclassifications_in_progress: BTreeSet::new(),
        boundaries: BTreeMap::new(),
        serialized_context_data,
        context_data: None,
    };
    if let Some(mut demand) = demand {
        info!("Load demand data");
        demand.finish_loading(&map);
        map.demand = Some(demand);
    }

    let graph = GraphSubset {
        node_to_edge: graph.node_to_edge,
        node_to_pt: graph.node_to_pt,
    };

    info!("Applying existing filters");
    apply_existing_filters(&mut map, osm.barrier_nodes, &graph);
    info!("Applying existing turn restrictions");
    apply_turn_restrictions(&mut map, osm.turn_restrictions);

    info!("Creating the router");
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
    map.redo_stack.clear();
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
