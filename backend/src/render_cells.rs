use std::collections::{HashMap, HashSet, VecDeque};

use geo::{BooleanOps, BoundingRect, Coord, Densify, Euclidean, LineString, MultiPolygon, Rect};
use serde::{Serialize, Serializer};
use utils::{Grid, LineSplit};

use crate::{Cell, IntersectionID, MapModel, Neighbourhood, RoadID};

const NUM_COLORS: usize = 10;
const RESOLUTION_M: f64 = 10.0;

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Disconnected,
    Pedestrianized,
    Cell(usize),
}

// Somewhat redundant with Into<JSonValue> impl, but this impl lets you serialize it
// within a struct declaratively.
impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Color::Disconnected => "disconnected".serialize(serializer),
            Color::Pedestrianized => "pedestrianized".serialize(serializer),
            Color::Cell(idx) => idx.serialize(serializer),
        }
    }
}

// Somewhat redundant with `impl Serialize for Color`, but this impl let's you serialize
// programmatically.
impl Into<geojson::JsonValue> for Color {
    fn into(self) -> geojson::JsonValue {
        match self {
            Color::Disconnected => "disconnected".into(),
            Color::Pedestrianized => "pedestrianized".into(),
            Color::Cell(idx) => idx.into(),
        }
    }
}

pub struct RenderCells {
    /// Rarely, this might be empty if the area is very small
    pub polygons_per_cell: Vec<MultiPolygon>,
    /// Colors per cell, such that adjacent cells are colored differently
    pub colors: Vec<Color>,
    /// Each border belongs to a cell; put its color here
    pub colors_per_border: HashMap<IntersectionID, Color>,
    /// Each road belongs to one or more cells; arbitrarily pick one of those cells and put its
    /// color here
    pub colors_per_road: HashMap<RoadID, Color>,
}

impl RenderCells {
    /// Partition a neighbourhood's boundary polygon based on the cells. This discretizes space into
    /// a grid, and then extracts a polygon from the raster. The results don't look perfect, but
    /// it's fast.
    pub fn new(map: &MapModel, neighbourhood: &Neighbourhood, cells: &Vec<Cell>) -> RenderCells {
        let boundary_polygon = neighbourhood.boundary_polygon().clone();
        // Make a 2D grid covering the polygon. Each tile in the grid contains a cell index, which
        // will become a color by the end. None means no cell is assigned yet.
        let bounds = <Option<Rect>>::from(boundary_polygon.bounding_rect()).unwrap();
        let mut grid: Grid<Option<usize>> = Grid::new(
            (bounds.width() / RESOLUTION_M).ceil() as usize,
            (bounds.height() / RESOLUTION_M).ceil() as usize,
            None,
        );

        // Initially fill out the grid based on the roads in each cell
        let mut warn_leak = true;
        for (cell_idx, cell) in cells.iter().enumerate() {
            for (r, interval) in &cell.roads {
                let road = map.get_r(*r);
                let slice = slice_linestring(&road.linestring, interval.start, interval.end);
                // Walk along the center line
                for pt in Euclidean.densify(&slice, RESOLUTION_M / 2.0).0 {
                    let grid_idx = grid.idx(
                        ((pt.x - bounds.min().x) / RESOLUTION_M) as usize,
                        ((pt.y - bounds.min().y) / RESOLUTION_M) as usize,
                    );
                    // Due to tunnels/bridges, sometimes a road belongs to a neighbourhood, but
                    // leaks outside the neighbourhood's boundary. Avoid crashing. The real fix
                    // is to better define boundaries in the face of z-order changes.
                    //
                    // Example is https://www.openstreetmap.org/way/87298633
                    if grid_idx >= grid.data.len() {
                        if warn_leak {
                            warn!(
                                "{} leaks outside its neighbourhood's boundary polygon, near {:?}",
                                road.id, pt
                            );
                            // In some neighbourhoods, there are so many warnings that logging
                            // causes noticeable slowdown!
                            warn_leak = false;
                        }
                        continue;
                    }

                    // If roads from two different cells are close enough to clobber
                    // originally, oh well?
                    grid.data[grid_idx] = Some(cell_idx);
                }
            }
        }
        // Also mark the boundary polygon, so we can prevent the diffusion from "leaking" outside
        // the area. The grid covers the rectangular bounds of the polygon. Rather than make an
        // enum with 3 cases, just assign a new index to mean "boundary."
        let boundary_marker = cells.len();
        for pt in Euclidean
            .densify(boundary_polygon.exterior(), RESOLUTION_M / 2.0)
            .0
        {
            // TODO Refactor helpers to transform between map-space and the grid tiles. Possibly
            // Grid should know about this.
            let grid_idx = grid.idx(
                ((pt.x - bounds.min().x) / RESOLUTION_M) as usize,
                ((pt.y - bounds.min().y) / RESOLUTION_M) as usize,
            );
            grid.data[grid_idx] = Some(boundary_marker);
        }

        let adjacencies = diffusion(&mut grid, boundary_marker);
        let mut cell_colors = color_cells(cells.len(), adjacencies);

        // Color some special cells
        for (idx, cell) in cells.iter().enumerate() {
            // If there's only one cell, it's not disconnected -- there are likely no main roads
            if cell.is_disconnected() && cells.len() > 1 {
                // Communicate pedestrianized differently
                if cell.roads.keys().all(|r| {
                    map.get_r(*r)
                        .tags
                        .is_any("highway", vec!["pedestrian", "service"])
                }) {
                    cell_colors[idx] = Color::Pedestrianized;
                } else {
                    cell_colors[idx] = Color::Disconnected;
                }
            }
        }

        if true {
            finalize(
                grid,
                cells,
                cell_colors,
                bounds,
                &MultiPolygon::new(vec![boundary_polygon]),
            )
        } else {
            debug_grid(grid, cells, cell_colors, bounds)
        }
    }
}

fn finalize(
    main_grid: Grid<Option<usize>>,
    cells: &Vec<Cell>,
    cell_colors: Vec<Color>,
    bounds: Rect,
    boundary: &MultiPolygon,
) -> RenderCells {
    let mut result = RenderCells {
        polygons_per_cell: Vec::new(),
        colors: Vec::new(),
        colors_per_border: HashMap::new(),
        colors_per_road: HashMap::new(),
    };

    for (idx, color) in cell_colors.into_iter().enumerate() {
        // contour will find where the grid is >= a threshold value. The main grid has one
        // number per cell, so we can't directly use it -- the area >= some cell index is
        // meaningless. Per cell, make a new grid that just has that cell.
        //
        // TODO Try isobands
        let grid: Grid<f64> = Grid {
            width: main_grid.width,
            height: main_grid.height,
            data: main_grid
                .data
                .iter()
                .map(
                    |maybe_cell| {
                        if maybe_cell == &Some(idx) {
                            1.0
                        } else {
                            0.0
                        }
                    },
                )
                .collect(),
        };

        let smooth = false;
        let contour_builder =
            contour::ContourBuilder::new(grid.width as u32, grid.height as u32, smooth)
                .x_origin(bounds.min().x)
                .y_origin(bounds.min().y)
                .x_step(RESOLUTION_M)
                .y_step(RESOLUTION_M);
        let thresholds = vec![1.0];

        let mut cell_polygons = Vec::new();
        for contour in contour_builder.contours(&grid.data, &thresholds).unwrap() {
            let (multipolygon, _) = contour.into_inner();
            cell_polygons.push(multipolygon);
        }
        assert_eq!(cell_polygons.len(), 1);

        // Sometimes one cell "leaks" out of the neighbourhood boundary. Not sure why. But we can
        // just clip the result.
        let cell_polygon = cell_polygons.remove(0);
        result
            .polygons_per_cell
            .push(cell_polygon.intersection(boundary));
        result.colors.push(color);

        for i in &cells[idx].border_intersections {
            result.colors_per_border.insert(*i, color);
        }
        // For roads crossing cells, arbitrarily overwrite
        for r in cells[idx].roads.keys() {
            result.colors_per_road.insert(*r, color);
        }
    }

    result
}

fn debug_grid(
    grid: Grid<Option<usize>>,
    cells: &Vec<Cell>,
    cell_colors: Vec<Color>,
    bounds: Rect,
) -> RenderCells {
    let mut result = RenderCells {
        polygons_per_cell: Vec::new(),
        colors: Vec::new(),
        colors_per_border: HashMap::new(),
        colors_per_road: HashMap::new(),
    };

    for (idx, color) in cell_colors.into_iter().enumerate() {
        let mut squares = Vec::new();
        for x in 0..grid.width {
            for y in 0..grid.height {
                if grid.data[grid.idx(x, y)] == Some(idx) {
                    squares.push(
                        Rect::new(
                            Coord {
                                x: (x as f64) * RESOLUTION_M + bounds.min().x,
                                y: (y as f64) * RESOLUTION_M + bounds.min().y,
                            },
                            Coord {
                                x: ((x + 1) as f64) * RESOLUTION_M + bounds.min().x,
                                y: ((y + 1) as f64) * RESOLUTION_M + bounds.min().y,
                            },
                        )
                        .to_polygon(),
                    );
                }
            }
        }

        result.polygons_per_cell.push(MultiPolygon::new(squares));
        result.colors.push(color);

        for i in &cells[idx].border_intersections {
            result.colors_per_border.insert(*i, color);
        }
        // For roads crossing cells, arbitrarily overwrite
        for r in cells[idx].roads.keys() {
            result.colors_per_road.insert(*r, color);
        }
    }

    result
}

/// Returns a set of adjacent indices. The pairs are symmetric -- (x, y) and (y, x) will both be
/// populated. Adjacency with boundary_marker doesn't count.
fn diffusion(grid: &mut Grid<Option<usize>>, boundary_marker: usize) -> HashSet<(usize, usize)> {
    // Grid indices to propagate
    let mut queue: VecDeque<usize> = VecDeque::new();

    // Initially seed the queue with all colored tiles
    for (idx, value) in grid.data.iter().enumerate() {
        if let Some(x) = value {
            // Don't expand the boundary tiles
            if *x != boundary_marker {
                queue.push_back(idx);
            }
        }
    }

    let mut adjacencies = HashSet::new();

    while !queue.is_empty() {
        let current_idx = queue.pop_front().unwrap();
        let current_color = grid.data[current_idx].unwrap();
        let (current_x, current_y) = grid.xy(current_idx);
        // Don't flood to diagonal neighbors. That would usually result in "leaking" out past the
        // boundary tiles when the boundary polygon isn't axis-aligned.
        // TODO But this still does "leak" out sometimes -- the cell covering 22nd/Lynn, for
        // example.
        for (next_x, next_y) in grid.orthogonal_neighbors(current_x, current_y) {
            let next_idx = grid.idx(next_x, next_y);
            if let Some(prev_color) = grid.data[next_idx] {
                // If the color doesn't match our current_color, we've found the border between two
                // cells.
                if current_color != prev_color
                    && current_color != boundary_marker
                    && prev_color != boundary_marker
                {
                    adjacencies.insert((current_color, prev_color));
                    adjacencies.insert((prev_color, current_color));
                }
                // If a color has been assigned, don't flood any further.
            } else {
                grid.data[next_idx] = Some(current_color);
                queue.push_back(next_idx);
            }
        }
    }

    adjacencies
}

fn color_cells(num_cells: usize, adjacencies: HashSet<(usize, usize)>) -> Vec<Color> {
    // This is the same greedy logic as Perimeter::calculate_coloring
    let mut assigned_colors = Vec::new();
    for this_idx in 0..num_cells {
        let mut available_colors: Vec<bool> = std::iter::repeat(true).take(NUM_COLORS).collect();
        // Find all neighbors
        for other_idx in 0..num_cells {
            if adjacencies.contains(&(this_idx, other_idx)) {
                // We assign colors in order, so any neighbor index smaller than us has been
                // chosen
                if other_idx < this_idx {
                    available_colors[assigned_colors[other_idx]] = false;
                }
            }
        }

        // If there are multiple colors available, prefer one that hasn't been used anywhere yet.
        // Cells far apart shouldn't seem related to the user.
        let mut choice = None;
        let mut backup = None;
        for (idx, available) in available_colors.into_iter().enumerate() {
            if !available {
                continue;
            }
            if assigned_colors.iter().any(|x| *x == idx) {
                if backup.is_none() {
                    backup = Some(idx);
                }
            } else {
                choice = Some(idx);
                break;
            }
        }
        assigned_colors.push(
            choice
                .or(backup)
                .unwrap_or_else(|| assigned_colors.len() % NUM_COLORS),
        );
    }
    assigned_colors
        .into_iter()
        .map(|idx| Color::Cell(idx))
        .collect()
}

// Return the linestring in an interval, or the whole thing if something breaks
fn slice_linestring(linestring: &LineString, start: f64, end: f64) -> LineString {
    linestring
        .line_split_twice(start, end)
        .and_then(|result| result.into_second())
        .unwrap_or_else(|| linestring.clone())
}
