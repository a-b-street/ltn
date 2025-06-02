use std::collections::{BTreeSet, HashMap};

use geo::{Coord, LineString};
use osm_reader::{NodeID, OsmID, RelationID, WayID};
use utils::{osm2graph::OsmReader, Tags};

use crate::boundary_stats::{POIKind, POI};

#[derive(Default)]
pub struct Osm {
    pub bus_routes_on_roads: HashMap<WayID, Vec<String>>,
    pub railways: Vec<LineString>,
    pub waterways: Vec<LineString>,
    pub barrier_nodes: BTreeSet<NodeID>,
    // Only represent one case of restricted turns (from, to) on a particular node
    pub turn_restrictions: HashMap<NodeID, Vec<(WayID, WayID)>>,
    pub pois: Vec<POI>,
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
            // TODO If a POI is tagged on the building itself, ideally we'd use its centroid. But
            // an arbitrary point on the boundary is good enough.
            node_mapping[&node_ids[0]],
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

pub fn is_road(tags: &Tags) -> bool {
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
pub fn parse_maxspeed_mph(tags: &Tags) -> usize {
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
