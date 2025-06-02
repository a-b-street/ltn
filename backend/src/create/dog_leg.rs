use geo::{Euclidean, InterpolatableLine, Length};
use utils::osm2graph::{EdgeID, Graph, Intersection, IntersectionID};

use crate::geo_helpers::bearing_from_endpoint;

/// Dog-leg intersections are "nearly" 4-way intersections, where a short edge separates two 3-way
/// intersections. Collapse them into a regular 4-way intersection so diagonal filters can be used.
pub fn collapse_dog_legs(graph: &mut Graph) {
    let mut dog_legs = Vec::new();
    for e in graph.edges.keys() {
        if let Some((side1, side2)) = detect_dog_leg(graph, *e) {
            dog_legs.push((*e, side1, side2));
        }
    }
    info!("Collapsing {} dog-legs", dog_legs.len());

    for (collapse_e, side1, side2) in dog_legs {
        fix_dog_leg(graph, collapse_e, side1, side2);
    }

    // Remove deleted edges from node_to_edge, used later for applying existing modal filters.
    graph
        .node_to_edge
        .retain(|_, e| graph.edges.contains_key(e));
}

/// If this edge is the short edge causing a dog-leg intersection, then return the two
/// perpendicular "side roads".
fn detect_dog_leg(graph: &Graph, e: EdgeID) -> Option<(EdgeID, EdgeID)> {
    let edge = &graph.edges[&e];
    if Euclidean.length(&edge.linestring) > 5.0 {
        return None;
    }
    if !edge.osm_tags.has("name") {
        return None;
    }
    let src_i = &graph.intersections[&edge.src];
    let dst_i = &graph.intersections[&edge.dst];
    let mut src_edges = src_i.edges.clone();
    let mut dst_edges = dst_i.edges.clone();
    if src_edges.len() != 3 || dst_edges.len() != 3 {
        return None;
    }

    // Find the two "side roads" with a different name than the short edge
    // (TODO We could use angle to be safer than name, but name works fine in all known cases so
    // far)
    src_edges.retain(|x| graph.edges[x].osm_tags.get("name") != edge.osm_tags.get("name"));
    dst_edges.retain(|x| graph.edges[x].osm_tags.get("name") != edge.osm_tags.get("name"));

    if src_edges.len() != 1 || dst_edges.len() != 1 {
        return None;
    }

    let src_edge = &graph.edges[&src_edges[0]];
    let dst_edge = &graph.edges[&dst_edges[0]];

    // Check the bearing of the two side roads, both pointing "away" from the short edge
    let b1 = bearing_from_endpoint(src_i.point, &src_edge.linestring);
    let b2 = bearing_from_endpoint(dst_i.point, &dst_edge.linestring);

    // If these both point about the same way, they're on the "same side" of the short edge.
    // Not a dog leg.
    if (b1 - b2).abs() < 30.0 {
        return None;
    }

    Some((src_edges[0], dst_edges[0]))
}

fn fix_dog_leg(graph: &mut Graph, collapse_e: EdgeID, side1: EdgeID, side2: EdgeID) {
    let (src, dst, midpt) = {
        let edge = &graph.edges[&collapse_e];
        let midpt = edge
            .linestring
            .point_at_ratio_from_start(&Euclidean, 0.5)
            .unwrap();
        (edge.src, edge.dst, midpt)
    };

    // Create a new intersection at the middle of the short edge
    let new_intersection = new_intersection_id(graph);
    graph.intersections.insert(
        new_intersection,
        Intersection {
            id: new_intersection,
            edges: Vec::new(),
            point: midpt,
            // OSM IDs are only used for debugging. Rather than change the representation here to
            // allow for "synthetic" intersections without an OSM node, just borrow one of the
            // endpoints.
            osm_node: graph.intersections[&src].osm_node,
        },
    );

    // Remove the edge
    graph.edges.remove(&collapse_e).unwrap();

    // Remove the two old intersections, reconnecting the edges
    for i in [src, dst] {
        let intersection = graph.intersections.remove(&i).unwrap();

        for e in intersection.edges {
            if e == collapse_e {
                continue;
            }
            graph
                .intersections
                .get_mut(&new_intersection)
                .unwrap()
                .edges
                .push(e);
            let fix_edge = graph.edges.get_mut(&e).unwrap();

            // First reconnect the edge
            if fix_edge.src == i {
                fix_edge.src = new_intersection;
            } else {
                fix_edge.dst = new_intersection;
            }

            // Then geometry, which depends if this edge is a side road or not
            if e == side1 || e == side2 {
                // Trim off the first or last meter, then connect to the new intersection
                if fix_edge.src == new_intersection {
                    if let Some(trim_pt) = fix_edge
                        .linestring
                        .point_at_distance_from_start(&Euclidean, 1.0)
                    {
                        fix_edge.linestring.0[0] = trim_pt.into();
                        fix_edge.linestring.0.insert(0, midpt.into());
                    }
                } else {
                    if let Some(trim_pt) = fix_edge
                        .linestring
                        .point_at_distance_from_end(&Euclidean, 1.0)
                    {
                        fix_edge.linestring.0.pop();
                        fix_edge.linestring.0.push(trim_pt.into());
                        fix_edge.linestring.0.push(midpt.into());
                    }
                }
            } else {
                // Extend the main roads up to the new point
                if fix_edge.src == new_intersection {
                    fix_edge.linestring.0.insert(0, midpt.into());
                } else {
                    fix_edge.linestring.0.push(midpt.into());
                }
            }
        }
    }
}

fn new_intersection_id(graph: &Graph) -> IntersectionID {
    IntersectionID(graph.intersections.keys().max().unwrap().0 + 1)
}
