use std::collections::{BTreeMap, BTreeSet, HashMap};

use anyhow::Result;
use geo::{ConvexHull, Coord, Geometry, GeometryCollection, LineString, Point};
use osm_reader::{Element, NodeID, WayID};

use crate::{
    FilterKind, Intersection, IntersectionID, MapModel, Mercator, Road, RoadID, Router, Tags,
};

struct Way {
    id: WayID,
    node_ids: Vec<NodeID>,
    tags: Tags,
}

pub fn scrape_osm(input_bytes: &[u8], study_area_name: Option<String>) -> Result<MapModel> {
    let mut node_mapping = HashMap::new();
    let mut highways = Vec::new();
    let mut all_barriers: BTreeSet<NodeID> = BTreeSet::new();
    osm_reader::parse(input_bytes, |elem| match elem {
        Element::Node { id, lon, lat, tags } => {
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
                    highways.push(Way { id, node_ids, tags });
                }
            }
        }
        Element::Relation { .. } => {}
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

    let (mut roads, mut intersections) = split_edges(&node_mapping, highways);

    // TODO expensive
    let mut collection: GeometryCollection = roads
        .iter()
        .map(|r| Geometry::LineString(r.linestring.clone()))
        .chain(
            intersections
                .iter()
                .map(|i| Geometry::Point(i.point.clone())),
        )
        .collect::<Vec<_>>()
        .into();
    let mercator = Mercator::from(collection.clone()).unwrap();
    for r in &mut roads {
        mercator.to_mercator_in_place(&mut r.linestring);
    }
    for i in &mut intersections {
        mercator.to_mercator_in_place(&mut i.point);
    }
    for coord in &mut barrier_pts {
        *coord = mercator.pt_to_mercator(*coord);
    }

    mercator.to_mercator_in_place(&mut collection);
    let boundary_polygon = collection.convex_hull();

    let modal_filters = BTreeMap::new();
    // TODO Do this latr
    let router_original = Router::new(&roads, &intersections, &modal_filters);

    let mut map = MapModel {
        roads,
        intersections,
        mercator,
        boundary_polygon,
        study_area_name,

        router_original,
        router_current: None,

        modal_filters,
        undo_stack: Vec::new(),
        redo_queue: Vec::new(),
        boundaries: BTreeMap::new(),
    };

    // Apply barriers (only those that're exactly on one of the roads)
    let all_roads: BTreeSet<RoadID> = map.roads.iter().map(|r| r.id).collect();
    for pt in barrier_pts {
        // TODO What kind?
        map.add_modal_filter(pt, &all_roads, FilterKind::NoEntry);
    }

    Ok(map)
}

fn split_edges(
    node_mapping: &HashMap<NodeID, Coord>,
    ways: Vec<Way>,
) -> (Vec<Road>, Vec<Intersection>) {
    // Count how many ways reference each node
    let mut node_counter: HashMap<NodeID, usize> = HashMap::new();
    for way in &ways {
        for node in &way.node_ids {
            *node_counter.entry(*node).or_insert(0) += 1;
        }
    }

    // Split each way into edges
    let mut node_to_intersection: HashMap<NodeID, IntersectionID> = HashMap::new();
    let mut intersections = Vec::new();
    let mut roads = Vec::new();
    for way in ways {
        let mut node1 = way.node_ids[0];
        let mut pts = Vec::new();

        let num_nodes = way.node_ids.len();
        for (idx, node) in way.node_ids.into_iter().enumerate() {
            pts.push(node_mapping[&node]);
            // Edges start/end at intersections between two ways. The endpoints of the way also
            // count as intersections.
            let is_endpoint =
                idx == 0 || idx == num_nodes - 1 || *node_counter.get(&node).unwrap() > 1;
            if is_endpoint && pts.len() > 1 {
                let road_id = RoadID(roads.len());

                let mut i_ids = Vec::new();
                for (n, point) in [(node1, pts[0]), (node, *pts.last().unwrap())] {
                    let intersection = if let Some(i) = node_to_intersection.get(&n) {
                        &mut intersections[i.0]
                    } else {
                        let i = IntersectionID(intersections.len());
                        intersections.push(Intersection {
                            id: i,
                            node: n,
                            point: Point(point),
                            roads: Vec::new(),
                        });
                        node_to_intersection.insert(n, i);
                        &mut intersections[i.0]
                    };

                    intersection.roads.push(road_id);
                    i_ids.push(intersection.id);
                }

                roads.push(Road {
                    id: road_id,
                    src_i: i_ids[0],
                    dst_i: i_ids[1],
                    way: way.id,
                    node1,
                    node2: node,
                    linestring: LineString::new(std::mem::take(&mut pts)),
                    tags: way.tags.clone(),
                });

                // Start the next edge
                node1 = node;
                pts.push(node_mapping[&node]);
            }
        }
    }

    (roads, intersections)
}

fn is_road(tags: &Tags) -> bool {
    if !tags.has("highway") || tags.is("area", "yes") {
        return false;
    }
    if tags.is_any(
        "highway",
        vec![
            "cycleway", "footway", "steps", "path", "track", "corridor", "proposed",
        ],
    ) {
        return false;
    }
    true
}
