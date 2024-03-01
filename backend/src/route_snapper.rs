use std::collections::BTreeMap;

use geo::{Coord, LineIntersection, LineLocatePoint, LineSplit, Point};
use route_snapper_graph::{Edge, NodeID, RouteSnapperMap};

use crate::{MapModel, RoadID};

impl MapModel {
    pub fn to_route_snapper_graph(&self) -> route_snapper_graph::RouteSnapperMap {
        let mut nodes = Vec::new();
        for i in &self.intersections {
            nodes.push(self.mercator.to_wgs84(&i.point).into());
        }

        let mut edges = Vec::new();
        for r in &self.roads {
            edges.push(Edge {
                node1: NodeID(r.src_i.0 as u32),
                node2: NodeID(r.dst_i.0 as u32),
                geometry: self.mercator.to_wgs84(&r.linestring),
                name: r.tags.get("name").cloned(),

                // Isn't serialized, doesn't matter
                length_meters: 0.0,
                forward_cost: None,
                backward_cost: None,
            });
        }

        // TODO This should be a method on RouteSnapperMap, but we'll have to project to mercator
        // and back
        let mut all_lines = Vec::new();
        for r in &self.roads {
            for line in r.linestring.lines() {
                all_lines.push(LineWithData { line, id: r.id });
            }
        }

        // Make new nodes for these split points, and figure out where to split roads
        let mut pt_to_node_id: BTreeMap<(isize, isize), NodeID> = BTreeMap::new();
        for i in &self.intersections {
            pt_to_node_id.insert(hashify_point(i.point.into()), NodeID(i.id.0 as u32));
        }
        let mut split_roads_at: BTreeMap<RoadID, Vec<f64>> = BTreeMap::new();
        for (r1, r2, cross) in geo::sweep::Intersections::<_>::from_iter(all_lines) {
            if let LineIntersection::SinglePoint {
                intersection,
                is_proper,
            } = cross
            {
                // Intersections are expected constantly at endpoints, so ignore those
                if is_proper {
                    pt_to_node_id.insert(hashify_point(intersection), NodeID(nodes.len() as u32));
                    nodes.push(self.mercator.to_wgs84(&Point::from(intersection)).into());

                    let r1_dist = self
                        .get_r(r1.id)
                        .linestring
                        .line_locate_point(&intersection.into())
                        .unwrap();
                    let r2_dist = self
                        .get_r(r2.id)
                        .linestring
                        .line_locate_point(&intersection.into())
                        .unwrap();
                    split_roads_at
                        .entry(r1.id)
                        .or_insert_with(Vec::new)
                        .push(r1_dist);
                    split_roads_at
                        .entry(r2.id)
                        .or_insert_with(Vec::new)
                        .push(r2_dist);
                }
            }
        }

        let mut remove_old_roads = Vec::new();
        for (r, fractions) in split_roads_at {
            for split_ls in self
                .get_r(r)
                .linestring
                .line_split_many(&fractions)
                .unwrap()
            {
                let Some(split_ls) = split_ls else {
                    // Sometimes the split points are too close together
                    continue;
                };
                // Make a new edge
                edges.push(Edge {
                    node1: pt_to_node_id[&hashify_point(split_ls.0[0])],
                    node2: pt_to_node_id[&hashify_point(*split_ls.0.last().unwrap())],
                    geometry: self.mercator.to_wgs84(&split_ls),
                    name: self.get_r(r).tags.get("name").cloned(),

                    // Isn't serialized, doesn't matter
                    length_meters: 0.0,
                    forward_cost: None,
                    backward_cost: None,
                });
            }

            remove_old_roads.push(r);
        }

        // Remove the old edge with the full road. The index into edges matches the RoadID, but
        // we'll change indices as we modify stuff, so carefully do it backwards
        remove_old_roads.reverse();
        for r in remove_old_roads {
            edges.remove(r.0);
        }

        RouteSnapperMap {
            nodes,
            edges,
            override_forward_costs: Vec::new(),
            override_backward_costs: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
struct LineWithData {
    line: geo::Line,
    id: RoadID,
}

impl geo::sweep::Cross for LineWithData {
    type Scalar = f64;

    fn line(&self) -> geo::sweep::LineOrPoint<Self::Scalar> {
        self.line.line()
    }
}

fn hashify_point(pt: Coord) -> (isize, isize) {
    // cm resolution
    ((pt.x * 100.0) as isize, (pt.y * 100.0) as isize)
}
