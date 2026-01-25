//! Building placement and construction system.
//!
//! Handles placement validation, construction progress, and NavGrid integration
//! for buildings in the game world.
//!
//! All calculations use fixed-point math for deterministic simulation.

use serde::{Deserialize, Serialize};

use crate::components::{EntityId, Position};
use crate::math::{fixed_serde, Fixed, Vec2Fixed};
use crate::pathfinding::{CellType, NavGrid};
use crate::production::Building;

// ============================================================================
// Placement Grid
// ============================================================================

/// State of a cell in the placement grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlacementCell {
    /// Cell is empty and available for building.
    Empty,
    /// Cell is occupied by a building entity.
    Occupied(EntityId),
    /// Cell is blocked by terrain or resources.
    Blocked,
}

impl Default for PlacementCell {
    fn default() -> Self {
        Self::Empty
    }
}

/// Grid for tracking building placement.
///
/// Separate from NavGrid to allow independent tracking of building
/// footprints vs navigation obstacles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementGrid {
    /// Grid width in cells.
    width: u32,
    /// Grid height in cells.
    height: u32,
    /// Cell data stored in row-major order.
    cells: Vec<PlacementCell>,
    /// Size of each cell in world units.
    #[serde(with = "fixed_serde")]
    cell_size: Fixed,
}

impl PlacementGrid {
    /// Create a new placement grid with all cells empty.
    ///
    /// # Panics
    ///
    /// Panics if `width` or `height` is zero, or if `cell_size` is not positive.
    #[must_use]
    pub fn new(width: u32, height: u32, cell_size: Fixed) -> Self {
        assert!(width > 0, "PlacementGrid width must be positive");
        assert!(height > 0, "PlacementGrid height must be positive");
        assert!(
            cell_size > Fixed::ZERO,
            "PlacementGrid cell_size must be positive"
        );

        let cell_count = (width as usize) * (height as usize);
        Self {
            width,
            height,
            cells: vec![PlacementCell::Empty; cell_count],
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

    /// Get cell state at coordinates.
    /// Returns `None` if out of bounds.
    #[must_use]
    pub fn get_cell(&self, x: u32, y: u32) -> Option<PlacementCell> {
        if self.in_bounds(x, y) {
            Some(self.cells[self.coords_to_index(x, y)])
        } else {
            None
        }
    }

    /// Set cell state at coordinates.
    /// Returns `false` if out of bounds.
    pub fn set_cell(&mut self, x: u32, y: u32, cell: PlacementCell) -> bool {
        if self.in_bounds(x, y) {
            let index = self.coords_to_index(x, y);
            self.cells[index] = cell;
            true
        } else {
            false
        }
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

        // Check bounds
        if x >= 0 && x < self.width as i64 && y >= 0 && y < self.height as i64 {
            Some((x as u32, y as u32))
        } else {
            None
        }
    }

    /// Convert grid coordinates to world position (corner of cell).
    #[must_use]
    pub fn grid_to_world(&self, x: u32, y: u32) -> Vec2Fixed {
        Vec2Fixed::new(
            Fixed::from_num(x) * self.cell_size,
            Fixed::from_num(y) * self.cell_size,
        )
    }

    /// Snap a world position to grid alignment.
    #[must_use]
    pub fn snap_to_grid(&self, pos: Vec2Fixed) -> Vec2Fixed {
        let x = (pos.x / self.cell_size).floor() * self.cell_size;
        let y = (pos.y / self.cell_size).floor() * self.cell_size;
        Vec2Fixed::new(x, y)
    }

    /// Check if a cell is available for placement.
    #[must_use]
    pub fn is_available(&self, x: u32, y: u32) -> bool {
        matches!(self.get_cell(x, y), Some(PlacementCell::Empty))
    }

    /// Mark cells as occupied by a building.
    ///
    /// Returns `false` if any cell is out of bounds.
    pub fn occupy_cells(
        &mut self,
        start_x: u32,
        start_y: u32,
        footprint: &BuildingFootprint,
        entity_id: EntityId,
    ) -> bool {
        // First verify all cells are in bounds
        for dy in 0..footprint.height {
            for dx in 0..footprint.width {
                if !self.in_bounds(start_x + dx, start_y + dy) {
                    return false;
                }
            }
        }

        // Mark all cells as occupied
        for dy in 0..footprint.height {
            for dx in 0..footprint.width {
                self.set_cell(
                    start_x + dx,
                    start_y + dy,
                    PlacementCell::Occupied(entity_id),
                );
            }
        }

        true
    }

    /// Clear cells occupied by a building.
    pub fn clear_cells(&mut self, start_x: u32, start_y: u32, footprint: &BuildingFootprint) {
        for dy in 0..footprint.height {
            for dx in 0..footprint.width {
                let x = start_x + dx;
                let y = start_y + dy;
                if self.in_bounds(x, y) {
                    self.set_cell(x, y, PlacementCell::Empty);
                }
            }
        }
    }
}

// ============================================================================
// Building Footprint
// ============================================================================

/// Defines the size of a building in grid cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BuildingFootprint {
    /// Width in cells.
    pub width: u32,
    /// Height in cells.
    pub height: u32,
}

impl BuildingFootprint {
    /// Create a new building footprint.
    #[must_use]
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Create a square footprint.
    #[must_use]
    pub const fn square(size: u32) -> Self {
        Self {
            width: size,
            height: size,
        }
    }

    /// Get the total number of cells this footprint covers.
    #[must_use]
    pub const fn cell_count(&self) -> u32 {
        self.width * self.height
    }
}

impl Default for BuildingFootprint {
    fn default() -> Self {
        Self::new(1, 1)
    }
}

// ============================================================================
// Placement Validation
// ============================================================================

/// Minimum distance in cells from resource nodes.
pub const MIN_RESOURCE_DISTANCE: u32 = 2;

/// Result of placement validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlacementResult {
    /// Placement is valid.
    Valid,
    /// One or more cells are blocked.
    Blocked {
        /// List of blocked cell coordinates.
        cells: Vec<(u32, u32)>,
    },
    /// Building would be placed outside grid bounds.
    OutOfBounds,
    /// Building is too close to a resource node.
    TooCloseToResource,
}

impl PlacementResult {
    /// Check if placement is valid.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        matches!(self, PlacementResult::Valid)
    }
}

/// Check if a building can be placed at the given position.
///
/// This function validates:
/// - All cells are within grid bounds
/// - All cells are empty (not blocked or occupied)
///
/// # Arguments
///
/// * `grid` - The placement grid to check against
/// * `position` - World position for the building (will be snapped to grid)
/// * `footprint` - Size of the building
///
/// # Returns
///
/// A `PlacementResult` indicating whether placement is valid or why it failed.
#[must_use]
pub fn can_place_building(
    grid: &PlacementGrid,
    position: Vec2Fixed,
    footprint: &BuildingFootprint,
) -> PlacementResult {
    // Convert world position to grid coordinates
    let Some((start_x, start_y)) = grid.world_to_grid(position) else {
        return PlacementResult::OutOfBounds;
    };

    // Check if entire footprint is in bounds
    if start_x + footprint.width > grid.width() || start_y + footprint.height > grid.height() {
        return PlacementResult::OutOfBounds;
    }

    // Check each cell in the footprint
    let mut blocked_cells = Vec::new();

    for dy in 0..footprint.height {
        for dx in 0..footprint.width {
            let x = start_x + dx;
            let y = start_y + dy;

            match grid.get_cell(x, y) {
                Some(PlacementCell::Empty) => {
                    // Cell is available
                }
                Some(PlacementCell::Occupied(_)) | Some(PlacementCell::Blocked) => {
                    blocked_cells.push((x, y));
                }
                None => {
                    return PlacementResult::OutOfBounds;
                }
            }
        }
    }

    if blocked_cells.is_empty() {
        PlacementResult::Valid
    } else {
        PlacementResult::Blocked {
            cells: blocked_cells,
        }
    }
}

/// Check if a building can be placed, also checking distance from resources.
///
/// This is a more thorough check that includes resource proximity validation.
#[must_use]
pub fn can_place_building_with_resource_check(
    grid: &PlacementGrid,
    position: Vec2Fixed,
    footprint: &BuildingFootprint,
    resource_positions: &[(u32, u32)],
) -> PlacementResult {
    // First do basic placement check
    let basic_result = can_place_building(grid, position, footprint);
    if !basic_result.is_valid() {
        return basic_result;
    }

    // Get grid coordinates
    let Some((start_x, start_y)) = grid.world_to_grid(position) else {
        return PlacementResult::OutOfBounds;
    };

    // Check distance from each resource
    for &(res_x, res_y) in resource_positions {
        for dy in 0..footprint.height {
            for dx in 0..footprint.width {
                let x = start_x + dx;
                let y = start_y + dy;

                // Calculate Chebyshev (chessboard) distance
                let dist_x = x.abs_diff(res_x);
                let dist_y = y.abs_diff(res_y);
                let distance = dist_x.max(dist_y);

                if distance < MIN_RESOURCE_DISTANCE {
                    return PlacementResult::TooCloseToResource;
                }
            }
        }
    }

    PlacementResult::Valid
}

// ============================================================================
// Construction System
// ============================================================================

/// Events generated by the construction system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstructionEvent {
    /// Construction has progressed.
    ConstructionProgress {
        /// The building entity.
        building: EntityId,
        /// Current progress in ticks.
        progress: u32,
        /// Total construction time in ticks.
        total: u32,
    },
    /// Construction has completed.
    ConstructionComplete {
        /// The building entity.
        building: EntityId,
    },
}

/// Advance construction for all buildings under construction.
///
/// # Arguments
///
/// * `buildings` - Tuples of (entity_id, building_component, position)
/// * `tick` - Current simulation tick
///
/// # Returns
///
/// A vector of construction events that occurred this tick.
pub fn construction_system(
    buildings: &mut [(EntityId, &mut Building, &Position)],
    _tick: u64,
) -> Vec<ConstructionEvent> {
    let mut events = Vec::new();

    for (entity_id, building, _position) in buildings.iter_mut() {
        // Skip already constructed buildings
        if building.is_constructed {
            continue;
        }

        // Advance construction
        let just_completed = building.tick_construction();

        if just_completed {
            events.push(ConstructionEvent::ConstructionComplete {
                building: *entity_id,
            });
        } else if !building.is_constructed {
            // Emit progress event
            events.push(ConstructionEvent::ConstructionProgress {
                building: *entity_id,
                progress: building.construction_progress,
                total: building.construction_total,
            });
        }
    }

    events
}

// ============================================================================
// Placement Preview (Ghost Building)
// ============================================================================

/// Data for rendering a building placement preview.
///
/// This is used by the UI to show where a building will be placed
/// and whether the placement is valid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacementPreview {
    /// Snapped world position for the building.
    pub position: Vec2Fixed,
    /// Building footprint.
    pub footprint: BuildingFootprint,
    /// Whether the current placement is valid.
    pub is_valid: bool,
    /// Cells that are blocked (if any).
    pub blocked_cells: Vec<(u32, u32)>,
}

impl PlacementPreview {
    /// Create a new placement preview.
    ///
    /// Automatically validates the placement and populates blocked cells.
    #[must_use]
    pub fn new(grid: &PlacementGrid, position: Vec2Fixed, footprint: BuildingFootprint) -> Self {
        let snapped_position = grid.snap_to_grid(position);
        let result = can_place_building(grid, snapped_position, &footprint);

        let (is_valid, blocked_cells) = match result {
            PlacementResult::Valid => (true, Vec::new()),
            PlacementResult::Blocked { cells } => (false, cells),
            PlacementResult::OutOfBounds | PlacementResult::TooCloseToResource => {
                (false, Vec::new())
            }
        };

        Self {
            position: snapped_position,
            footprint,
            is_valid,
            blocked_cells,
        }
    }

    /// Get all cells covered by this preview.
    #[must_use]
    pub fn get_footprint_cells(&self, grid: &PlacementGrid) -> Vec<(u32, u32)> {
        let mut cells = Vec::new();

        if let Some((start_x, start_y)) = grid.world_to_grid(self.position) {
            for dy in 0..self.footprint.height {
                for dx in 0..self.footprint.width {
                    cells.push((start_x + dx, start_y + dy));
                }
            }
        }

        cells
    }
}

// ============================================================================
// NavGrid Integration
// ============================================================================

/// Mark cells in the NavGrid as blocked when a building is placed.
///
/// Call this after successfully placing a building to update pathfinding.
pub fn mark_building_in_navgrid(
    nav_grid: &mut NavGrid,
    placement_grid: &PlacementGrid,
    position: Vec2Fixed,
    footprint: &BuildingFootprint,
) {
    if let Some((start_x, start_y)) = placement_grid.world_to_grid(position) {
        for dy in 0..footprint.height {
            for dx in 0..footprint.width {
                let x = start_x + dx;
                let y = start_y + dy;
                nav_grid.set_cell(x, y, CellType::Blocked);
            }
        }
    }
}

/// Clear cells in the NavGrid when a building is destroyed.
///
/// Call this when a building is removed to restore pathfinding.
pub fn clear_building_from_navgrid(
    nav_grid: &mut NavGrid,
    placement_grid: &PlacementGrid,
    position: Vec2Fixed,
    footprint: &BuildingFootprint,
) {
    if let Some((start_x, start_y)) = placement_grid.world_to_grid(position) {
        for dy in 0..footprint.height {
            for dx in 0..footprint.width {
                let x = start_x + dx;
                let y = start_y + dy;
                nav_grid.set_cell(x, y, CellType::Walkable);
            }
        }
    }
}

/// Place a building, updating both placement grid and nav grid.
///
/// This is a convenience function that handles:
/// 1. Validating placement
/// 2. Marking cells as occupied in PlacementGrid
/// 3. Blocking cells in NavGrid
///
/// # Returns
///
/// `true` if placement succeeded, `false` otherwise.
pub fn place_building(
    placement_grid: &mut PlacementGrid,
    nav_grid: &mut NavGrid,
    position: Vec2Fixed,
    footprint: &BuildingFootprint,
    entity_id: EntityId,
) -> bool {
    // Validate placement first
    if !can_place_building(placement_grid, position, footprint).is_valid() {
        return false;
    }

    let Some((start_x, start_y)) = placement_grid.world_to_grid(position) else {
        return false;
    };

    // Update placement grid
    if !placement_grid.occupy_cells(start_x, start_y, footprint, entity_id) {
        return false;
    }

    // Update nav grid
    mark_building_in_navgrid(nav_grid, placement_grid, position, footprint);

    true
}

/// Remove a building, updating both placement grid and nav grid.
///
/// This is a convenience function that handles:
/// 1. Clearing cells in PlacementGrid
/// 2. Unblocking cells in NavGrid
pub fn remove_building(
    placement_grid: &mut PlacementGrid,
    nav_grid: &mut NavGrid,
    position: Vec2Fixed,
    footprint: &BuildingFootprint,
) {
    if let Some((start_x, start_y)) = placement_grid.world_to_grid(position) {
        // Clear placement grid
        placement_grid.clear_cells(start_x, start_y, footprint);

        // Clear nav grid
        clear_building_from_navgrid(nav_grid, placement_grid, position, footprint);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::production::BuildingTypeId;

    fn fixed(n: i32) -> Fixed {
        Fixed::from_num(n)
    }

    fn vec2(x: i32, y: i32) -> Vec2Fixed {
        Vec2Fixed::new(fixed(x), fixed(y))
    }

    // ------------------------------------------------------------------------
    // PlacementGrid Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_placement_grid_creation() {
        let grid = PlacementGrid::new(10, 10, fixed(1));
        assert_eq!(grid.width(), 10);
        assert_eq!(grid.height(), 10);
        assert_eq!(grid.cell_size(), fixed(1));
    }

    #[test]
    fn test_placement_grid_default_empty() {
        let grid = PlacementGrid::new(5, 5, fixed(1));
        for y in 0..5 {
            for x in 0..5 {
                assert!(grid.is_available(x, y));
            }
        }
    }

    #[test]
    fn test_placement_grid_set_get_cell() {
        let mut grid = PlacementGrid::new(10, 10, fixed(1));

        // Set to blocked
        assert!(grid.set_cell(3, 4, PlacementCell::Blocked));
        assert_eq!(grid.get_cell(3, 4), Some(PlacementCell::Blocked));
        assert!(!grid.is_available(3, 4));

        // Set to occupied
        assert!(grid.set_cell(5, 5, PlacementCell::Occupied(42)));
        assert_eq!(grid.get_cell(5, 5), Some(PlacementCell::Occupied(42)));
        assert!(!grid.is_available(5, 5));

        // Out of bounds
        assert!(!grid.set_cell(10, 10, PlacementCell::Blocked));
        assert_eq!(grid.get_cell(10, 10), None);
    }

    #[test]
    fn test_placement_grid_world_to_grid() {
        let grid = PlacementGrid::new(10, 10, fixed(2));

        // Position (1, 1) should be in cell (0, 0)
        assert_eq!(grid.world_to_grid(vec2(1, 1)), Some((0, 0)));

        // Position (4, 4) should be in cell (2, 2)
        assert_eq!(grid.world_to_grid(vec2(4, 4)), Some((2, 2)));

        // Negative positions
        assert_eq!(grid.world_to_grid(vec2(-1, 0)), None);

        // Out of bounds
        assert_eq!(grid.world_to_grid(vec2(20, 20)), None);
    }

    #[test]
    fn test_placement_grid_snap_to_grid() {
        let grid = PlacementGrid::new(10, 10, fixed(2));

        let snapped = grid.snap_to_grid(vec2(3, 5));
        assert_eq!(snapped.x, fixed(2));
        assert_eq!(snapped.y, fixed(4));
    }

    #[test]
    fn test_placement_grid_occupy_cells() {
        let mut grid = PlacementGrid::new(10, 10, fixed(1));
        let footprint = BuildingFootprint::new(2, 3);

        assert!(grid.occupy_cells(2, 2, &footprint, 100));

        // Verify all cells are occupied
        for dy in 0..3 {
            for dx in 0..2 {
                assert_eq!(
                    grid.get_cell(2 + dx, 2 + dy),
                    Some(PlacementCell::Occupied(100))
                );
            }
        }

        // Verify adjacent cells are still empty
        assert!(grid.is_available(1, 2));
        assert!(grid.is_available(4, 2));
    }

    #[test]
    fn test_placement_grid_occupy_cells_out_of_bounds() {
        let mut grid = PlacementGrid::new(10, 10, fixed(1));
        let footprint = BuildingFootprint::new(3, 3);

        // Try to place at edge where it would overflow
        assert!(!grid.occupy_cells(8, 8, &footprint, 100));

        // Verify no cells were modified
        for y in 0..10 {
            for x in 0..10 {
                assert!(grid.is_available(x, y));
            }
        }
    }

    #[test]
    fn test_placement_grid_clear_cells() {
        let mut grid = PlacementGrid::new(10, 10, fixed(1));
        let footprint = BuildingFootprint::new(2, 2);

        grid.occupy_cells(3, 3, &footprint, 50);
        grid.clear_cells(3, 3, &footprint);

        // All cells should be empty again
        for dy in 0..2 {
            for dx in 0..2 {
                assert!(grid.is_available(3 + dx, 3 + dy));
            }
        }
    }

    // ------------------------------------------------------------------------
    // BuildingFootprint Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_building_footprint() {
        let footprint = BuildingFootprint::new(3, 4);
        assert_eq!(footprint.width, 3);
        assert_eq!(footprint.height, 4);
        assert_eq!(footprint.cell_count(), 12);
    }

    #[test]
    fn test_building_footprint_square() {
        let footprint = BuildingFootprint::square(5);
        assert_eq!(footprint.width, 5);
        assert_eq!(footprint.height, 5);
        assert_eq!(footprint.cell_count(), 25);
    }

    // ------------------------------------------------------------------------
    // Placement Validation Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_can_place_building_valid() {
        let grid = PlacementGrid::new(10, 10, fixed(1));
        let footprint = BuildingFootprint::new(2, 2);

        let result = can_place_building(&grid, vec2(3, 3), &footprint);
        assert!(result.is_valid());
    }

    #[test]
    fn test_can_place_building_out_of_bounds() {
        let grid = PlacementGrid::new(10, 10, fixed(1));
        let footprint = BuildingFootprint::new(3, 3);

        // Negative position
        let result = can_place_building(&grid, vec2(-1, 0), &footprint);
        assert!(matches!(result, PlacementResult::OutOfBounds));

        // Position that would overflow grid
        let result = can_place_building(&grid, vec2(8, 8), &footprint);
        assert!(matches!(result, PlacementResult::OutOfBounds));
    }

    #[test]
    fn test_can_place_building_blocked() {
        let mut grid = PlacementGrid::new(10, 10, fixed(1));
        let footprint = BuildingFootprint::new(3, 3);

        // Block a cell
        grid.set_cell(4, 4, PlacementCell::Blocked);

        let result = can_place_building(&grid, vec2(3, 3), &footprint);
        match result {
            PlacementResult::Blocked { cells } => {
                assert!(cells.contains(&(4, 4)));
            }
            _ => panic!("Expected Blocked result"),
        }
    }

    #[test]
    fn test_can_place_building_occupied() {
        let mut grid = PlacementGrid::new(10, 10, fixed(1));
        let footprint = BuildingFootprint::new(2, 2);

        // Occupy a cell
        grid.set_cell(5, 5, PlacementCell::Occupied(99));

        let result = can_place_building(&grid, vec2(5, 5), &footprint);
        match result {
            PlacementResult::Blocked { cells } => {
                assert!(cells.contains(&(5, 5)));
            }
            _ => panic!("Expected Blocked result"),
        }
    }

    #[test]
    fn test_can_place_building_with_resource_check() {
        let grid = PlacementGrid::new(10, 10, fixed(1));
        let footprint = BuildingFootprint::new(2, 2);
        let resources = vec![(3, 3)];

        // Too close to resource
        let result =
            can_place_building_with_resource_check(&grid, vec2(2, 2), &footprint, &resources);
        assert!(matches!(result, PlacementResult::TooCloseToResource));

        // Far enough from resource
        let result =
            can_place_building_with_resource_check(&grid, vec2(7, 7), &footprint, &resources);
        assert!(result.is_valid());
    }

    // ------------------------------------------------------------------------
    // Construction System Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_construction_system_progress() {
        let mut building = Building::new(BuildingTypeId::new(1), 100);
        let position = Position::new(vec2(0, 0));

        let mut buildings = vec![(1u64, &mut building, &position)];

        // First tick
        let events = construction_system(&mut buildings, 1);
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            ConstructionEvent::ConstructionProgress {
                building: 1,
                progress: 1,
                total: 100
            }
        ));
    }

    #[test]
    fn test_construction_system_completion() {
        let mut building = Building::new(BuildingTypeId::new(1), 3);
        let position = Position::new(vec2(0, 0));

        // Advance to just before completion
        building.construction_progress = 2;

        let mut buildings = vec![(1u64, &mut building, &position)];

        // Completing tick
        let events = construction_system(&mut buildings, 3);

        assert!(events
            .iter()
            .any(|e| matches!(e, ConstructionEvent::ConstructionComplete { building: 1 })));
    }

    #[test]
    fn test_construction_system_skips_completed() {
        let mut building = Building::constructed(BuildingTypeId::new(1));
        let position = Position::new(vec2(0, 0));

        let mut buildings = vec![(1u64, &mut building, &position)];

        let events = construction_system(&mut buildings, 1);
        assert!(events.is_empty());
    }

    // ------------------------------------------------------------------------
    // PlacementPreview Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_placement_preview_valid() {
        let grid = PlacementGrid::new(10, 10, fixed(1));
        let footprint = BuildingFootprint::new(2, 2);

        let preview = PlacementPreview::new(&grid, vec2(3, 3), footprint);

        assert!(preview.is_valid);
        assert!(preview.blocked_cells.is_empty());
        assert_eq!(preview.position, vec2(3, 3));
    }

    #[test]
    fn test_placement_preview_invalid() {
        let mut grid = PlacementGrid::new(10, 10, fixed(1));
        let footprint = BuildingFootprint::new(2, 2);

        grid.set_cell(4, 4, PlacementCell::Blocked);

        let preview = PlacementPreview::new(&grid, vec2(3, 3), footprint);

        assert!(!preview.is_valid);
        assert!(preview.blocked_cells.contains(&(4, 4)));
    }

    #[test]
    fn test_placement_preview_snapping() {
        let grid = PlacementGrid::new(10, 10, fixed(2));
        let footprint = BuildingFootprint::new(1, 1);

        // Position (3, 3) should snap to (2, 2)
        let preview = PlacementPreview::new(&grid, vec2(3, 3), footprint);

        assert_eq!(preview.position.x, fixed(2));
        assert_eq!(preview.position.y, fixed(2));
    }

    #[test]
    fn test_placement_preview_get_footprint_cells() {
        let grid = PlacementGrid::new(10, 10, fixed(1));
        let footprint = BuildingFootprint::new(2, 3);

        let preview = PlacementPreview::new(&grid, vec2(2, 2), footprint);
        let cells = preview.get_footprint_cells(&grid);

        assert_eq!(cells.len(), 6);
        assert!(cells.contains(&(2, 2)));
        assert!(cells.contains(&(3, 2)));
        assert!(cells.contains(&(2, 3)));
        assert!(cells.contains(&(3, 3)));
        assert!(cells.contains(&(2, 4)));
        assert!(cells.contains(&(3, 4)));
    }

    // ------------------------------------------------------------------------
    // NavGrid Integration Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_mark_building_in_navgrid() {
        let mut nav_grid = NavGrid::new(10, 10, fixed(1));
        let placement_grid = PlacementGrid::new(10, 10, fixed(1));
        let footprint = BuildingFootprint::new(2, 2);

        mark_building_in_navgrid(&mut nav_grid, &placement_grid, vec2(3, 3), &footprint);

        // Verify cells are blocked in nav grid
        for dy in 0..2 {
            for dx in 0..2 {
                assert!(!nav_grid.is_walkable(3 + dx, 3 + dy));
            }
        }

        // Adjacent cells should still be walkable
        assert!(nav_grid.is_walkable(2, 3));
        assert!(nav_grid.is_walkable(5, 3));
    }

    #[test]
    fn test_clear_building_from_navgrid() {
        let mut nav_grid = NavGrid::new(10, 10, fixed(1));
        let placement_grid = PlacementGrid::new(10, 10, fixed(1));
        let footprint = BuildingFootprint::new(2, 2);

        // First block the cells
        mark_building_in_navgrid(&mut nav_grid, &placement_grid, vec2(3, 3), &footprint);

        // Then clear them
        clear_building_from_navgrid(&mut nav_grid, &placement_grid, vec2(3, 3), &footprint);

        // All cells should be walkable again
        for dy in 0..2 {
            for dx in 0..2 {
                assert!(nav_grid.is_walkable(3 + dx, 3 + dy));
            }
        }
    }

    #[test]
    fn test_place_building_integration() {
        let mut placement_grid = PlacementGrid::new(10, 10, fixed(1));
        let mut nav_grid = NavGrid::new(10, 10, fixed(1));
        let footprint = BuildingFootprint::new(2, 2);

        // Place building
        let success = place_building(
            &mut placement_grid,
            &mut nav_grid,
            vec2(4, 4),
            &footprint,
            123,
        );

        assert!(success);

        // Verify placement grid updated
        for dy in 0..2 {
            for dx in 0..2 {
                assert_eq!(
                    placement_grid.get_cell(4 + dx, 4 + dy),
                    Some(PlacementCell::Occupied(123))
                );
            }
        }

        // Verify nav grid updated
        for dy in 0..2 {
            for dx in 0..2 {
                assert!(!nav_grid.is_walkable(4 + dx, 4 + dy));
            }
        }
    }

    #[test]
    fn test_place_building_fails_on_blocked() {
        let mut placement_grid = PlacementGrid::new(10, 10, fixed(1));
        let mut nav_grid = NavGrid::new(10, 10, fixed(1));
        let footprint = BuildingFootprint::new(2, 2);

        // Block a cell
        placement_grid.set_cell(5, 5, PlacementCell::Blocked);

        // Try to place building over blocked cell
        let success = place_building(
            &mut placement_grid,
            &mut nav_grid,
            vec2(4, 4),
            &footprint,
            123,
        );

        assert!(!success);
    }

    #[test]
    fn test_remove_building_integration() {
        let mut placement_grid = PlacementGrid::new(10, 10, fixed(1));
        let mut nav_grid = NavGrid::new(10, 10, fixed(1));
        let footprint = BuildingFootprint::new(2, 2);

        // Place and then remove building
        place_building(
            &mut placement_grid,
            &mut nav_grid,
            vec2(4, 4),
            &footprint,
            123,
        );

        remove_building(&mut placement_grid, &mut nav_grid, vec2(4, 4), &footprint);

        // Verify placement grid cleared
        for dy in 0..2 {
            for dx in 0..2 {
                assert!(placement_grid.is_available(4 + dx, 4 + dy));
            }
        }

        // Verify nav grid cleared
        for dy in 0..2 {
            for dx in 0..2 {
                assert!(nav_grid.is_walkable(4 + dx, 4 + dy));
            }
        }
    }

    // ------------------------------------------------------------------------
    // Determinism Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_placement_validation_determinism() {
        let mut grid1 = PlacementGrid::new(20, 20, fixed(1));
        let mut grid2 = PlacementGrid::new(20, 20, fixed(1));

        // Set up identical blocked cells
        for i in 5..15 {
            grid1.set_cell(10, i, PlacementCell::Blocked);
            grid2.set_cell(10, i, PlacementCell::Blocked);
        }

        let footprint = BuildingFootprint::new(3, 3);

        // Test multiple positions
        for x in 0..15 {
            for y in 0..15 {
                let result1 = can_place_building(&grid1, vec2(x, y), &footprint);
                let result2 = can_place_building(&grid2, vec2(x, y), &footprint);

                assert_eq!(
                    result1.is_valid(),
                    result2.is_valid(),
                    "Determinism failed at ({x}, {y})"
                );
            }
        }
    }

    #[test]
    fn test_construction_determinism() {
        // Create two identical buildings
        let mut building1 = Building::new(BuildingTypeId::new(1), 50);
        let mut building2 = Building::new(BuildingTypeId::new(1), 50);
        let position = Position::new(vec2(0, 0));

        // Run construction on both
        for tick in 1..=50 {
            let mut b1 = vec![(1u64, &mut building1, &position)];
            let mut b2 = vec![(1u64, &mut building2, &position)];

            let events1 = construction_system(&mut b1, tick);
            let events2 = construction_system(&mut b2, tick);

            // Events should be identical
            assert_eq!(events1.len(), events2.len());

            // Progress should be identical
            assert_eq!(
                building1.construction_progress,
                building2.construction_progress
            );
            assert_eq!(building1.is_constructed, building2.is_constructed);
        }
    }
}
