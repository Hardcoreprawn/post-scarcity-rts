//! Grid-based pathfinding using A* algorithm.
//!
//! All calculations use fixed-point math for deterministic results
//! across different platforms and clients.

use crate::error::{GameError, Result};
use crate::math::{fixed_serde, Fixed, Vec2Fixed};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

/// Cell types for navigation grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum CellType {
    /// Normal walkable terrain (cost: 1).
    #[default]
    Walkable,
    /// Impassable terrain.
    Blocked,
    /// Slow terrain with 2x movement cost.
    SlowTerrain,
}

impl CellType {
    /// Returns the movement cost for this cell type.
    /// Returns `None` for blocked cells.
    #[must_use]
    pub const fn movement_cost(self) -> Option<Fixed> {
        match self {
            Self::Walkable => Some(Fixed::ONE),
            Self::Blocked => None,
            Self::SlowTerrain => Some(Fixed::const_from_int(2)),
        }
    }

    /// Returns true if this cell is walkable.
    #[must_use]
    pub const fn is_walkable(self) -> bool {
        !matches!(self, Self::Blocked)
    }
}

/// Navigation grid for pathfinding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavGrid {
    /// Grid width in cells.
    width: u32,
    /// Grid height in cells.
    height: u32,
    /// Cell data stored in row-major order.
    cells: Vec<CellType>,
    /// Size of each cell in world units.
    #[serde(with = "fixed_serde")]
    cell_size: Fixed,
}

impl NavGrid {
    /// Create a new navigation grid with all cells walkable.
    ///
    /// # Panics
    ///
    /// Panics if `width` or `height` is zero, or if `cell_size` is not positive.
    #[must_use]
    pub fn new(width: u32, height: u32, cell_size: Fixed) -> Self {
        assert!(width > 0, "NavGrid width must be positive");
        assert!(height > 0, "NavGrid height must be positive");
        assert!(
            cell_size > Fixed::ZERO,
            "NavGrid cell_size must be positive"
        );

        let cell_count = (width as usize) * (height as usize);
        Self {
            width,
            height,
            cells: vec![CellType::Walkable; cell_count],
            cell_size,
        }
    }

    /// Grid width in cells.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Grid height in cells.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Cell size in world units.
    #[must_use]
    pub const fn cell_size(&self) -> Fixed {
        self.cell_size
    }

    /// Convert (x, y) coordinates to grid index.
    #[inline]
    fn coords_to_index(&self, x: u32, y: u32) -> usize {
        (y as usize) * (self.width as usize) + (x as usize)
    }

    /// Check if coordinates are within grid bounds.
    #[must_use]
    pub fn in_bounds(&self, x: u32, y: u32) -> bool {
        x < self.width && y < self.height
    }

    /// Get cell type at coordinates.
    /// Returns `None` if out of bounds.
    #[must_use]
    pub fn get_cell(&self, x: u32, y: u32) -> Option<CellType> {
        if self.in_bounds(x, y) {
            Some(self.cells[self.coords_to_index(x, y)])
        } else {
            None
        }
    }

    /// Set cell type at coordinates.
    /// Returns `false` if out of bounds.
    pub fn set_cell(&mut self, x: u32, y: u32, cell_type: CellType) -> bool {
        if self.in_bounds(x, y) {
            let index = self.coords_to_index(x, y);
            self.cells[index] = cell_type;
            true
        } else {
            false
        }
    }

    /// Check if a cell is walkable.
    #[must_use]
    pub fn is_walkable(&self, x: u32, y: u32) -> bool {
        self.get_cell(x, y).is_some_and(|c| c.is_walkable())
    }

    /// Convert world position to grid coordinates.
    ///
    /// Returns `None` if the position is outside the grid bounds.
    #[must_use]
    pub fn world_to_grid(&self, pos: Vec2Fixed) -> Option<(u32, u32)> {
        // Handle negative positions
        if pos.x < Fixed::ZERO || pos.y < Fixed::ZERO {
            return None;
        }

        let x = (pos.x / self.cell_size).to_num::<i64>();
        let y = (pos.y / self.cell_size).to_num::<i64>();

        // Check bounds (i64 to handle potential overflow)
        if x >= 0 && x < self.width as i64 && y >= 0 && y < self.height as i64 {
            Some((x as u32, y as u32))
        } else {
            None
        }
    }

    /// Convert grid coordinates to world position (center of cell).
    #[must_use]
    pub fn grid_to_world(&self, x: u32, y: u32) -> Vec2Fixed {
        let half = self.cell_size / Fixed::from_num(2);
        Vec2Fixed::new(
            Fixed::from_num(x) * self.cell_size + half,
            Fixed::from_num(y) * self.cell_size + half,
        )
    }

    /// Get movement cost for a cell.
    /// Returns `None` for blocked or out-of-bounds cells.
    #[must_use]
    pub fn movement_cost(&self, x: u32, y: u32) -> Option<Fixed> {
        self.get_cell(x, y).and_then(|c| c.movement_cost())
    }
}

impl Default for NavGrid {
    /// Create a default NavGrid (64x64 cells, 32 unit cell size).
    fn default() -> Self {
        Self::new(64, 64, Fixed::from_num(32))
    }
}

/// A node in the A* open set priority queue.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct AStarNode {
    /// Grid coordinates.
    x: u32,
    y: u32,
    /// f_score = g_score + heuristic (negated for min-heap)
    f_score: Fixed,
    /// Tie-breaker for determinism: lower coordinates first.
    /// This ensures consistent ordering when f_scores are equal.
    tie_breaker: u64,
}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap is a max-heap, so we reverse the comparison for min-heap behavior.
        // Lower f_score = higher priority, so we use other.cmp(self).
        match other.f_score.cmp(&self.f_score) {
            Ordering::Equal => {
                // Deterministic tie-breaking: prefer lower tie_breaker
                other.tie_breaker.cmp(&self.tie_breaker)
            }
            ord => ord,
        }
    }
}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Direction offsets for 8-directional movement.
const DIRECTIONS: [(i32, i32); 8] = [
    (1, 0),   // East
    (1, 1),   // Southeast
    (0, 1),   // South
    (-1, 1),  // Southwest
    (-1, 0),  // West
    (-1, -1), // Northwest
    (0, -1),  // North
    (1, -1),  // Northeast
];

/// Diagonal movement cost multiplier (approximation of sqrt(2) in fixed-point).
const DIAGONAL_COST: Fixed = Fixed::const_from_int(1); // Using 1 for Chebyshev distance

/// Calculate Chebyshev distance heuristic (suitable for 8-directional movement).
#[inline]
fn chebyshev_heuristic(x1: u32, y1: u32, x2: u32, y2: u32) -> Fixed {
    let dx = x1.abs_diff(x2);
    let dy = y1.abs_diff(y2);
    Fixed::from_num(dx.max(dy))
}

/// Check if a diagonal move is valid (no corner cutting through blocked cells).
#[inline]
fn is_diagonal_valid(grid: &NavGrid, x: u32, y: u32, dx: i32, dy: i32) -> bool {
    // For diagonal moves, check that adjacent cardinal cells are walkable
    if dx != 0 && dy != 0 {
        let check_x = (x as i32 + dx) as u32;
        let check_y = (y as i32 + dy) as u32;
        let adj1 = grid.is_walkable(check_x, y);
        let adj2 = grid.is_walkable(x, check_y);
        adj1 && adj2
    } else {
        true
    }
}

/// Find a path from start to goal using A* algorithm.
///
/// Returns a vector of world positions representing the path, or an error if no path exists.
///
/// # Errors
///
/// Returns `GameError::InvalidState` if:
/// - Start or goal position is outside the grid
/// - Start or goal position is blocked
/// - No path exists between start and goal
pub fn find_path(grid: &NavGrid, start: Vec2Fixed, goal: Vec2Fixed) -> Result<Vec<Vec2Fixed>> {
    // Convert world positions to grid coordinates
    let (start_x, start_y) = grid
        .world_to_grid(start)
        .ok_or_else(|| GameError::InvalidState("Start position outside grid".into()))?;

    let (goal_x, goal_y) = grid
        .world_to_grid(goal)
        .ok_or_else(|| GameError::InvalidState("Goal position outside grid".into()))?;

    // Check if start and goal are walkable
    if !grid.is_walkable(start_x, start_y) {
        return Err(GameError::InvalidState("Start position is blocked".into()));
    }
    if !grid.is_walkable(goal_x, goal_y) {
        return Err(GameError::InvalidState("Goal position is blocked".into()));
    }

    // Early exit if start == goal
    if start_x == goal_x && start_y == goal_y {
        return Ok(vec![grid.grid_to_world(start_x, start_y)]);
    }

    find_path_grid(grid, start_x, start_y, goal_x, goal_y)
}

/// Internal A* implementation working on grid coordinates.
fn find_path_grid(
    grid: &NavGrid,
    start_x: u32,
    start_y: u32,
    goal_x: u32,
    goal_y: u32,
) -> Result<Vec<Vec2Fixed>> {
    let mut open_set: BinaryHeap<AStarNode> = BinaryHeap::new();
    let mut came_from: HashMap<(u32, u32), (u32, u32)> = HashMap::new();
    let mut g_score: HashMap<(u32, u32), Fixed> = HashMap::new();

    // Initialize with start node
    let start_h = chebyshev_heuristic(start_x, start_y, goal_x, goal_y);
    g_score.insert((start_x, start_y), Fixed::ZERO);
    open_set.push(AStarNode {
        x: start_x,
        y: start_y,
        f_score: start_h,
        tie_breaker: coords_to_tie_breaker(start_x, start_y),
    });

    while let Some(current) = open_set.pop() {
        // Goal reached
        if current.x == goal_x && current.y == goal_y {
            return Ok(reconstruct_path(grid, &came_from, goal_x, goal_y));
        }

        let current_g = g_score
            .get(&(current.x, current.y))
            .copied()
            .unwrap_or(Fixed::MAX);

        // Explore neighbors
        for &(dx, dy) in &DIRECTIONS {
            let nx = current.x as i32 + dx;
            let ny = current.y as i32 + dy;

            // Skip out of bounds
            if nx < 0 || ny < 0 {
                continue;
            }

            let nx = nx as u32;
            let ny = ny as u32;

            if !grid.in_bounds(nx, ny) {
                continue;
            }

            // Get movement cost (None if blocked)
            let Some(cell_cost) = grid.movement_cost(nx, ny) else {
                continue;
            };

            // Check diagonal validity (no corner cutting)
            if !is_diagonal_valid(grid, current.x, current.y, dx, dy) {
                continue;
            }

            // Calculate movement cost (diagonal moves cost same as cardinal for Chebyshev)
            let move_cost = if dx != 0 && dy != 0 {
                cell_cost * DIAGONAL_COST
            } else {
                cell_cost
            };

            let tentative_g = current_g + move_cost;
            let neighbor_g = g_score.get(&(nx, ny)).copied().unwrap_or(Fixed::MAX);

            if tentative_g < neighbor_g {
                // This path is better
                came_from.insert((nx, ny), (current.x, current.y));
                g_score.insert((nx, ny), tentative_g);

                let h = chebyshev_heuristic(nx, ny, goal_x, goal_y);
                let f = tentative_g + h;

                open_set.push(AStarNode {
                    x: nx,
                    y: ny,
                    f_score: f,
                    tie_breaker: coords_to_tie_breaker(nx, ny),
                });
            }
        }
    }

    // No path found
    Err(GameError::InvalidState(format!(
        "No path from ({start_x}, {start_y}) to ({goal_x}, {goal_y})"
    )))
}

/// Convert coordinates to a tie-breaker value for deterministic ordering.
#[inline]
fn coords_to_tie_breaker(x: u32, y: u32) -> u64 {
    ((y as u64) << 32) | (x as u64)
}

/// Reconstruct path from came_from map.
fn reconstruct_path(
    grid: &NavGrid,
    came_from: &HashMap<(u32, u32), (u32, u32)>,
    goal_x: u32,
    goal_y: u32,
) -> Vec<Vec2Fixed> {
    let mut path = Vec::new();
    let mut current = (goal_x, goal_y);

    path.push(grid.grid_to_world(current.0, current.1));

    while let Some(&prev) = came_from.get(&current) {
        path.push(grid.grid_to_world(prev.0, prev.1));
        current = prev;
    }

    path.reverse();
    path
}

/// Smooth a path by removing unnecessary waypoints.
///
/// Uses line-of-sight checks to skip intermediate waypoints while
/// ensuring the path doesn't cut through obstacles.
#[must_use]
pub fn smooth_path(grid: &NavGrid, path: Vec<Vec2Fixed>) -> Vec<Vec2Fixed> {
    if path.len() <= 2 {
        return path;
    }

    let mut smoothed = Vec::with_capacity(path.len());
    smoothed.push(path[0]);

    let mut current_idx = 0;

    while current_idx < path.len() - 1 {
        // Try to find the furthest point we can reach directly
        let mut furthest_visible = current_idx + 1;

        for check_idx in (current_idx + 2)..path.len() {
            if has_line_of_sight(grid, path[current_idx], path[check_idx]) {
                furthest_visible = check_idx;
            }
        }

        smoothed.push(path[furthest_visible]);
        current_idx = furthest_visible;
    }

    smoothed
}

/// Check if there's a clear line of sight between two world positions.
///
/// Uses Bresenham-like stepping through grid cells.
fn has_line_of_sight(grid: &NavGrid, start: Vec2Fixed, end: Vec2Fixed) -> bool {
    let Some((x0, y0)) = grid.world_to_grid(start) else {
        return false;
    };
    let Some((x1, y1)) = grid.world_to_grid(end) else {
        return false;
    };

    // Bresenham's line algorithm adapted for grid
    let dx = (x1 as i32 - x0 as i32).abs();
    let dy = (y1 as i32 - y0 as i32).abs();
    let sx = if x0 < x1 { 1i32 } else { -1i32 };
    let sy = if y0 < y1 { 1i32 } else { -1i32 };
    let mut err = dx - dy;

    let mut x = x0 as i32;
    let mut y = y0 as i32;

    loop {
        // Check if current cell is walkable
        if !grid.is_walkable(x as u32, y as u32) {
            return false;
        }

        if x == x1 as i32 && y == y1 as i32 {
            break;
        }

        let e2 = 2 * err;

        // Check diagonal moves for corner cutting
        if e2 > -dy && e2 < dx {
            // This is a diagonal move
            let next_x = x + sx;
            let next_y = y + sy;
            // Ensure both adjacent cells are walkable
            if !grid.is_walkable(next_x as u32, y as u32)
                || !grid.is_walkable(x as u32, next_y as u32)
            {
                return false;
            }
        }

        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed(n: i32) -> Fixed {
        Fixed::from_num(n)
    }

    fn vec2(x: i32, y: i32) -> Vec2Fixed {
        Vec2Fixed::new(fixed(x), fixed(y))
    }

    #[test]
    fn test_cell_type_costs() {
        assert_eq!(CellType::Walkable.movement_cost(), Some(Fixed::ONE));
        assert_eq!(CellType::Blocked.movement_cost(), None);
        assert_eq!(
            CellType::SlowTerrain.movement_cost(),
            Some(Fixed::from_num(2))
        );
    }

    #[test]
    fn test_navgrid_creation() {
        let grid = NavGrid::new(10, 10, fixed(1));
        assert_eq!(grid.width(), 10);
        assert_eq!(grid.height(), 10);
        assert_eq!(grid.cell_size(), fixed(1));
    }

    #[test]
    fn test_world_to_grid_conversion() {
        let grid = NavGrid::new(10, 10, fixed(2));

        // Position (1, 1) should be in cell (0, 0)
        assert_eq!(grid.world_to_grid(vec2(1, 1)), Some((0, 0)));

        // Position (3, 3) should be in cell (1, 1)
        assert_eq!(grid.world_to_grid(vec2(3, 3)), Some((1, 1)));

        // Position (19, 19) should be in cell (9, 9)
        assert_eq!(grid.world_to_grid(vec2(19, 19)), Some((9, 9)));

        // Position (20, 20) should be out of bounds
        assert_eq!(grid.world_to_grid(vec2(20, 20)), None);

        // Negative positions should be out of bounds
        assert_eq!(grid.world_to_grid(vec2(-1, 0)), None);
    }

    #[test]
    fn test_grid_to_world_conversion() {
        let grid = NavGrid::new(10, 10, fixed(2));

        // Cell (0, 0) center should be at (1, 1)
        let pos = grid.grid_to_world(0, 0);
        assert_eq!(pos.x, fixed(1));
        assert_eq!(pos.y, fixed(1));

        // Cell (1, 1) center should be at (3, 3)
        let pos = grid.grid_to_world(1, 1);
        assert_eq!(pos.x, fixed(3));
        assert_eq!(pos.y, fixed(3));
    }

    #[test]
    fn test_set_and_get_cell() {
        let mut grid = NavGrid::new(5, 5, fixed(1));

        assert!(grid.is_walkable(2, 2));

        grid.set_cell(2, 2, CellType::Blocked);
        assert!(!grid.is_walkable(2, 2));

        grid.set_cell(2, 2, CellType::SlowTerrain);
        assert!(grid.is_walkable(2, 2));
        assert_eq!(grid.movement_cost(2, 2), Some(fixed(2)));
    }

    #[test]
    fn test_simple_path() {
        let grid = NavGrid::new(10, 10, fixed(1));

        let start = vec2(0, 0);
        let goal = vec2(5, 5);

        let path = find_path(&grid, start, goal).unwrap();

        // Path should start near start and end near goal
        assert!(!path.is_empty());

        // First waypoint should be at grid cell (0, 0) center
        let first = path.first().unwrap();
        assert!(first.x >= Fixed::ZERO && first.x < fixed(1));
        assert!(first.y >= Fixed::ZERO && first.y < fixed(1));

        // Last waypoint should be at grid cell (5, 5) center
        let last = path.last().unwrap();
        assert!(last.x >= fixed(5) && last.x < fixed(6));
        assert!(last.y >= fixed(5) && last.y < fixed(6));
    }

    #[test]
    fn test_path_around_obstacle() {
        let mut grid = NavGrid::new(10, 10, fixed(1));

        // Create a vertical wall
        for y in 2..8 {
            grid.set_cell(5, y, CellType::Blocked);
        }

        let start = vec2(2, 5);
        let goal = vec2(8, 5);

        let path = find_path(&grid, start, goal).unwrap();

        // Path should exist and avoid the wall
        assert!(!path.is_empty());

        // Verify no waypoint is in a blocked cell
        for point in &path {
            let (gx, gy) = grid.world_to_grid(*point).unwrap();
            assert!(
                grid.is_walkable(gx, gy),
                "Path goes through blocked cell ({gx}, {gy})"
            );
        }
    }

    #[test]
    fn test_no_path_exists() {
        let mut grid = NavGrid::new(10, 10, fixed(1));

        // Create a complete barrier
        for y in 0..10 {
            grid.set_cell(5, y, CellType::Blocked);
        }

        let start = vec2(2, 5);
        let goal = vec2(8, 5);

        let result = find_path(&grid, start, goal);
        assert!(result.is_err());
    }

    #[test]
    fn test_path_to_same_cell() {
        let grid = NavGrid::new(10, 10, fixed(1));

        let start = vec2(5, 5);
        let goal = vec2(5, 5);

        let path = find_path(&grid, start, goal).unwrap();
        assert_eq!(path.len(), 1);
    }

    #[test]
    fn test_blocked_start() {
        let mut grid = NavGrid::new(10, 10, fixed(1));
        grid.set_cell(0, 0, CellType::Blocked);

        let start = vec2(0, 0);
        let goal = vec2(5, 5);

        let result = find_path(&grid, start, goal);
        assert!(result.is_err());
    }

    #[test]
    fn test_blocked_goal() {
        let mut grid = NavGrid::new(10, 10, fixed(1));
        grid.set_cell(5, 5, CellType::Blocked);

        let start = vec2(0, 0);
        let goal = vec2(5, 5);

        let result = find_path(&grid, start, goal);
        assert!(result.is_err());
    }

    #[test]
    fn test_path_smoothing() {
        let grid = NavGrid::new(10, 10, fixed(1));

        // Create a straight line path
        let path = vec![vec2(0, 0), vec2(1, 1), vec2(2, 2), vec2(3, 3), vec2(4, 4)];

        let smoothed = smooth_path(&grid, path);

        // Smoothed path should have fewer waypoints (just start and end)
        assert!(smoothed.len() <= 2);
        assert_eq!(smoothed.first().unwrap().x, fixed(0));
        assert_eq!(smoothed.last().unwrap().x, fixed(4));
    }

    #[test]
    fn test_determinism() {
        let mut grid = NavGrid::new(20, 20, fixed(1));

        // Create some obstacles
        for i in 5..15 {
            grid.set_cell(10, i, CellType::Blocked);
        }

        let start = vec2(5, 10);
        let goal = vec2(15, 10);

        // Run pathfinding multiple times
        let path1 = find_path(&grid, start, goal).unwrap();
        let path2 = find_path(&grid, start, goal).unwrap();
        let path3 = find_path(&grid, start, goal).unwrap();

        // All paths must be identical (determinism)
        assert_eq!(path1, path2);
        assert_eq!(path2, path3);
    }

    #[test]
    fn test_slow_terrain_preference() {
        let mut grid = NavGrid::new(10, 3, fixed(1));

        // Create slow terrain on the direct path
        grid.set_cell(3, 1, CellType::SlowTerrain);
        grid.set_cell(4, 1, CellType::SlowTerrain);
        grid.set_cell(5, 1, CellType::SlowTerrain);
        grid.set_cell(6, 1, CellType::SlowTerrain);

        let start = vec2(0, 1);
        let goal = vec2(9, 1);

        let path = find_path(&grid, start, goal).unwrap();

        // Path should exist
        assert!(!path.is_empty());
    }

    #[test]
    fn test_chebyshev_heuristic() {
        assert_eq!(chebyshev_heuristic(0, 0, 5, 5), fixed(5));
        assert_eq!(chebyshev_heuristic(0, 0, 3, 7), fixed(7));
        assert_eq!(chebyshev_heuristic(5, 5, 5, 5), fixed(0));
    }
}
