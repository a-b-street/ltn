use std::collections::{BTreeMap, BTreeSet};

use geo::{Euclidean, Length};

use crate::{IntersectionID, MapModel, Neighbourhood, Road, RoadID};

/// A partitioning of the interior of a neighbourhood based on driving connectivity
pub struct Cell {
    /// Most roads are fully in one cell. Roads with modal filters on them are sometimes split
    /// between two cells, and the PercentInterval indicates the split.
    pub roads: BTreeMap<RoadID, PercentInterval>,
    /// Intersections where this cell touches the boundary of the neighbourhood.
    pub border_intersections: BTreeSet<IntersectionID>,
    /// The cell only contains service roads and can be visually de-emphasized
    pub unimportant: bool,
}

impl Cell {
    /// A cell is disconnected if it's not connected to a perimeter road.
    pub fn is_disconnected(&self) -> bool {
        self.border_intersections.is_empty()
    }

    /// Find all of the disconnected of reachable areas, bounded by border intersections. This is
    /// with respect to driving.
    pub fn find_all(map: &MapModel, neighbourhood: &Neighbourhood) -> Vec<Cell> {
        let mut cells = Vec::new();
        let mut visited = BTreeSet::new();

        for start in &neighbourhood.interior_roads {
            if visited.contains(start) || map.modal_filters.contains_key(start) {
                continue;
            }
            let start = *start;
            let road = map.get_r(start);
            // There are non-private roads connected only to private roads, like
            // https://www.openstreetmap.org/way/725759378 and
            // https://www.openstreetmap.org/way/27890699. Also skip these, to avoid creating a
            // disconnected cell.
            let connected_to_public_road = [road.src_i, road.dst_i]
                .into_iter()
                .flat_map(|i| &map.get_i(i).roads)
                .any(|r| *r != start && !is_private(map.get_r(*r)));
            if !connected_to_public_road {
                continue;
            }

            let cell = floodfill(map, start, neighbourhood);
            visited.extend(cell.roads.keys().cloned());

            cells.push(cell);
        }

        // Filtered roads right along the perimeter have a tiny cell
        for (r, filter) in &map.modal_filters {
            let road = map.get_r(*r);
            if neighbourhood.border_intersections.contains(&road.src_i) {
                let mut cell = Cell {
                    roads: BTreeMap::new(),
                    border_intersections: BTreeSet::from([road.src_i]),
                    unimportant: false,
                };
                cell.roads.insert(
                    road.id,
                    PercentInterval {
                        start: 0.0,
                        end: filter.percent_along,
                    },
                );
                cells.push(cell);
            }
            if neighbourhood.border_intersections.contains(&road.dst_i) {
                let mut cell = Cell {
                    roads: BTreeMap::new(),
                    border_intersections: BTreeSet::from([road.dst_i]),
                    unimportant: false,
                };
                cell.roads.insert(
                    road.id,
                    PercentInterval {
                        start: filter.percent_along,
                        end: 1.0,
                    },
                );
                cells.push(cell);
            }
        }

        cells
    }
}

/// An interval of percentages along a road's length, with start < end.
pub struct PercentInterval {
    pub start: f64,
    pub end: f64,
}

fn floodfill(map: &MapModel, start: RoadID, neighbourhood: &Neighbourhood) -> Cell {
    let mut visited_roads: BTreeMap<RoadID, PercentInterval> = BTreeMap::new();
    let mut cell_borders = BTreeSet::new();
    // We don't need a priority queue
    let mut queue = vec![start];

    // The caller should handle this case
    assert!(!map.modal_filters.contains_key(&start));

    while !queue.is_empty() {
        let current = map.get_r(queue.pop().unwrap());
        if visited_roads.contains_key(&current.id) {
            continue;
        }
        visited_roads.insert(
            current.id,
            PercentInterval {
                start: 0.0,
                end: 1.0,
            },
        );

        for i in [current.src_i, current.dst_i] {
            // It's possible for one border intersection to have two roads in the interior of the
            // neighbourhood. Don't consider a turn between those roads through this intersection as
            // counting as connectivity -- we're right at the boundary road, so it's like leaving
            // and re-entering the neighbourhood.
            if neighbourhood.border_intersections.contains(&i) {
                cell_borders.insert(i);
                continue;
            }

            for next in &map.get_i(i).roads {
                let next_road = map.get_r(*next);
                if let Some(ref filter) = map.diagonal_filters.get(&i) {
                    if !filter.allows_movement(&(current.id, *next)) {
                        continue;
                    }
                }
                if let Some(ref filter) = map.modal_filters.get(next) {
                    // Which ends of the filtered road have we reached?
                    let mut visited_start = next_road.src_i == i;
                    let mut visited_end = next_road.dst_i == i;
                    // We may have visited previously from the other side.
                    if let Some(interval) = visited_roads.get(next) {
                        if interval.start == 0.0 {
                            visited_start = true;
                        }
                        if interval.end == Euclidean.length(&next_road.linestring) {
                            visited_end = true;
                        }
                    }
                    visited_roads.insert(
                        *next,
                        PercentInterval {
                            start: if visited_start {
                                0.0
                            } else {
                                filter.percent_along
                            },
                            end: if visited_end {
                                1.0
                            } else {
                                filter.percent_along
                            },
                        },
                    );
                    continue;
                }

                // TODO This happens near weird geometry. This is OK, but should someday root-cause it.
                if !neighbourhood.interior_roads.contains(next) {
                    continue;
                }

                queue.push(*next);
            }
        }
    }

    let unimportant = if map.hide_unimportant_cells {
        visited_roads
            .keys()
            .all(|r| map.get_r(*r).tags.is("highway", "service"))
    } else {
        false
    };

    Cell {
        roads: visited_roads,
        border_intersections: cell_borders,
        unimportant,
    }
}

fn is_private(_road: &Road) -> bool {
    false
}
