use std::collections::{BTreeMap, BTreeSet, HashMap};

use anyhow::Result;
use geo::{Coord, LineString, Polygon};
use osm_reader::{Element, NodeID};
use rstar::{primitives::GeomWithData, RTree};
use utils::Tags;

use crate::{Direction, FilterKind, Intersection, IntersectionID, MapModel, Road, RoadID, Router};

pub fn scrape_osm(
    input_bytes: &[u8],
    boundary_wgs84: Polygon,
    study_area_name: Option<String>,
) -> Result<MapModel> {
    info!("Parsing {} bytes of OSM data", input_bytes.len());
    // This doesn't use osm2graph's helper, because it needs to scrape more things from OSM
    let mut node_mapping = HashMap::new();
    let mut highways = Vec::new();
    let mut railways = Vec::new();
    let mut waterways = Vec::new();
    let mut all_barriers: BTreeSet<NodeID> = BTreeSet::new();
    osm_reader::parse(input_bytes, |elem| match elem {
        Element::Node {
            id, lon, lat, tags, ..
        } => {
            let pt = Coord { x: lon, y: lat };
            node_mapping.insert(id, pt);

            // Tuning these by hand for a few known areas.
            // https://wiki.openstreetmap.org/wiki/Key:barrier is proper reference.
            if let Some(kind) = tags.get("barrier") {
                // Bristol has many gates that don't seem as relevant
                if kind != "gate" {
                    all_barriers.insert(id);
                }
            }
        }
        Element::Way {
            id,
            mut node_ids,
            tags,
            ..
        } => {
            let tags = tags.into();
            if is_road(&tags) {
                // TODO This sometimes happens from Overpass?
                let num = node_ids.len();
                node_ids.retain(|n| node_mapping.contains_key(n));
                if node_ids.len() != num {
                    warn!("{id} refers to nodes outside the imported area");
                }
                if node_ids.len() >= 2 {
                    highways.push(utils::osm2graph::Way { id, node_ids, tags });
                }
            } else if tags.has("railway") && (!tags.has("layer") || tags.is("layer", "0")) {
                node_ids.retain(|n| node_mapping.contains_key(n));
                if node_ids.len() >= 2 {
                    railways.push(LineString(
                        node_ids.into_iter().map(|n| node_mapping[&n]).collect(),
                    ));
                }
            } else if tags.is_any("natural", vec!["water", "coastline"])
                || tags.is("waterway", "dock")
            {
                // If the entire area is inside the study area, the LineString will be closed. If
                // it intersects the study area, then it might not be.
                node_ids.retain(|n| node_mapping.contains_key(n));
                if node_ids.len() >= 2 {
                    waterways.push(LineString(
                        node_ids.into_iter().map(|n| node_mapping[&n]).collect(),
                    ));
                }
            }
        }
        Element::Relation { .. } => {}
        Element::Bounds { .. } => {}
    })?;

    // There'll be many barrier nodes on non-driveable paths we don't consider roads. Filter for
    // just those on things we consider roads.
    let mut barrier_pts = Vec::new();
    for way in &highways {
        for node in &way.node_ids {
            if all_barriers.contains(node) {
                barrier_pts.push(node_mapping[node]);
            }
        }
    }

    info!("Splitting {} ways into edges", highways.len());
    let graph = utils::osm2graph::Graph::from_scraped_osm(node_mapping, highways);
    // Copy all the fields
    let intersections: Vec<Intersection> = graph
        .intersections
        .into_iter()
        .map(|i| Intersection {
            id: IntersectionID(i.id.0),
            point: i.point,
            node: i.osm_node,
            roads: i.edges.into_iter().map(|e| RoadID(e.0)).collect(),
        })
        .collect();

    // Add in a bit
    let roads: Vec<Road> = graph
        .edges
        .into_iter()
        .map(|e| Road {
            id: RoadID(e.id.0),
            src_i: IntersectionID(e.src.0),
            dst_i: IntersectionID(e.dst.0),
            way: e.osm_way,
            node1: e.osm_node1,
            node2: e.osm_node2,
            linestring: e.linestring,
            tags: e.osm_tags,
        })
        .collect();
    for coord in &mut barrier_pts {
        *coord = graph.mercator.pt_to_mercator(*coord);
    }

    for ls in &mut railways {
        graph.mercator.to_mercator_in_place(ls);
    }
    for ls in &mut waterways {
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

    let mut directions = BTreeMap::new();
    for r in &roads {
        directions.insert(r.id, Direction::from_osm(&r.tags));
    }

    let mut map = MapModel {
        roads,
        intersections,
        mercator: graph.mercator,
        boundary_wgs84,
        study_area_name,
        closest_road,
        closest_intersection,

        railways,
        waterways,

        router_original: None,
        router_current: None,
        router_original_with_penalty: None,

        original_modal_filters: BTreeMap::new(),
        modal_filters: BTreeMap::new(),

        directions,

        undo_stack: Vec::new(),
        redo_queue: Vec::new(),
        boundaries: BTreeMap::new(),
    };

    // Apply barriers (only those that're exactly on one of the roads)
    for pt in barrier_pts {
        // TODO What kind?
        map.add_modal_filter(pt, None, FilterKind::NoEntry);
    }
    // The commands above populate the existing modal filters and edit history. Undo that.
    map.original_modal_filters = map.modal_filters.clone();
    map.undo_stack.clear();
    map.redo_queue.clear();

    let main_road_penalty = 1.0;
    map.router_original = Some(Router::new(
        &map.roads,
        &map.modal_filters,
        &map.directions,
        main_road_penalty,
    ));

    Ok(map)
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
