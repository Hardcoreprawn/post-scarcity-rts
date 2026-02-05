//! Procedural map generation with terrain, resources, and obstacles.
//!
//! Generates balanced maps with:
//! - Symmetric terrain layouts
//! - Resource nodes with fair distribution
//! - Obstacles and chokepoints
//! - NavGrid-compatible terrain data

use serde::{Deserialize, Serialize};

use crate::math::{Fixed, Vec2Fixed};
use crate::pathfinding::CellType;

/// Map configuration for procedural generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapConfig {
    /// Map width in cells.
    pub width: u32,
    /// Map height in cells.
    pub height: u32,
    /// Cell size in world units (for coordinate conversion).
    pub cell_size: u32,
    /// Symmetry mode for fair gameplay.
    pub symmetry: SymmetryMode,
    /// Obstacle density (0.0 = open, 1.0 = very dense).
    pub obstacle_density: f32,
    /// Resource density multiplier.
    pub resource_density: f32,
    /// Random seed for deterministic generation.
    pub seed: u64,
}

impl Default for MapConfig {
    fn default() -> Self {
        Self {
            width: 64,
            height: 64,
            cell_size: 8,
            symmetry: SymmetryMode::Rotational,
            obstacle_density: 0.15,
            resource_density: 1.0,
            seed: 12345,
        }
    }
}

impl MapConfig {
    /// Create a small map (512x512 world units).
    #[must_use]
    pub fn small() -> Self {
        Self {
            width: 64,
            height: 64,
            cell_size: 8,
            ..Default::default()
        }
    }

    /// Create a medium map (768x768 world units).
    #[must_use]
    pub fn medium() -> Self {
        Self {
            width: 96,
            height: 96,
            cell_size: 8,
            ..Default::default()
        }
    }

    /// Create a large map (1024x1024 world units).
    #[must_use]
    pub fn large() -> Self {
        Self {
            width: 128,
            height: 128,
            cell_size: 8,
            ..Default::default()
        }
    }

    /// Set the random seed.
    #[must_use]
    pub const fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Set obstacle density.
    #[must_use]
    pub fn with_obstacle_density(mut self, density: f32) -> Self {
        self.obstacle_density = density.clamp(0.0, 1.0);
        self
    }
}

/// Symmetry mode for map generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SymmetryMode {
    /// No symmetry (for PvE or asymmetric scenarios).
    None,
    /// Horizontal symmetry (left-right mirror).
    Horizontal,
    /// Vertical symmetry (top-bottom mirror).
    Vertical,
    /// 180-degree rotational symmetry (standard for 1v1).
    #[default]
    Rotational,
    /// 4-way symmetry (for 4-player maps).
    FourWay,
}

/// Generated terrain cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TerrainCell {
    /// Cell type for pathfinding.
    pub cell_type: CellType,
    /// Height level (0-3, for visual variation).
    pub height: u8,
    /// Whether this is a spawn-safe zone (no obstacles near spawn).
    pub spawn_safe: bool,
}

impl TerrainCell {
    /// Create a walkable cell.
    #[must_use]
    pub const fn walkable() -> Self {
        Self {
            cell_type: CellType::Walkable,
            height: 0,
            spawn_safe: false,
        }
    }

    /// Create a blocked cell.
    #[must_use]
    pub const fn blocked() -> Self {
        Self {
            cell_type: CellType::Blocked,
            height: 0,
            spawn_safe: false,
        }
    }

    /// Create a slow terrain cell.
    #[must_use]
    pub const fn slow() -> Self {
        Self {
            cell_type: CellType::SlowTerrain,
            height: 0,
            spawn_safe: false,
        }
    }

    /// Mark as spawn-safe zone.
    #[must_use]
    pub const fn with_spawn_safe(mut self) -> Self {
        self.spawn_safe = true;
        self
    }

    /// Set height level.
    #[must_use]
    pub const fn with_height(mut self, height: u8) -> Self {
        self.height = height;
        self
    }
}

/// A resource node placement on the generated map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePlacement {
    /// Position in world coordinates.
    pub position: Vec2Fixed,
    /// Resource amount.
    pub amount: i64,
    /// Whether this is a permanent (base) or temporary (expansion) node.
    pub permanent: bool,
}

impl ResourcePlacement {
    /// Create a new resource placement.
    #[must_use]
    pub const fn new(position: Vec2Fixed, amount: i64, permanent: bool) -> Self {
        Self {
            position,
            amount,
            permanent,
        }
    }
}

/// A spawn point on the generated map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnPoint {
    /// Position in world coordinates.
    pub position: Vec2Fixed,
    /// Nearby base resources.
    pub base_resources: Vec<Vec2Fixed>,
}

/// Generated map data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedMap {
    /// Map configuration used.
    pub config: MapConfig,
    /// Terrain grid (row-major order).
    pub terrain: Vec<TerrainCell>,
    /// Resource node placements.
    pub resources: Vec<ResourcePlacement>,
    /// Spawn points.
    pub spawn_points: Vec<SpawnPoint>,
}

impl GeneratedMap {
    /// Get terrain cell at grid coordinates.
    #[must_use]
    pub fn get_cell(&self, x: u32, y: u32) -> Option<&TerrainCell> {
        if x < self.config.width && y < self.config.height {
            let idx = (y * self.config.width + x) as usize;
            self.terrain.get(idx)
        } else {
            None
        }
    }

    /// Get mutable terrain cell at grid coordinates.
    pub fn get_cell_mut(&mut self, x: u32, y: u32) -> Option<&mut TerrainCell> {
        if x < self.config.width && y < self.config.height {
            let idx = (y * self.config.width + x) as usize;
            self.terrain.get_mut(idx)
        } else {
            None
        }
    }

    /// Convert world coordinates to grid coordinates.
    #[must_use]
    pub fn world_to_grid(&self, world_pos: Vec2Fixed) -> (u32, u32) {
        let cell_size = Fixed::from_num(self.config.cell_size);
        let x = (world_pos.x / cell_size).to_num::<i32>().max(0) as u32;
        let y = (world_pos.y / cell_size).to_num::<i32>().max(0) as u32;
        (x.min(self.config.width - 1), y.min(self.config.height - 1))
    }

    /// Convert grid coordinates to world coordinates (center of cell).
    #[must_use]
    pub fn grid_to_world(&self, x: u32, y: u32) -> Vec2Fixed {
        let cell_size = Fixed::from_num(self.config.cell_size);
        let half = cell_size / Fixed::from_num(2);
        Vec2Fixed::new(
            Fixed::from_num(x) * cell_size + half,
            Fixed::from_num(y) * cell_size + half,
        )
    }

    /// Get the NavGrid-compatible cell types.
    #[must_use]
    pub fn as_cell_types(&self) -> Vec<CellType> {
        self.terrain.iter().map(|c| c.cell_type).collect()
    }

    /// Map width in world units.
    #[must_use]
    pub const fn world_width(&self) -> u32 {
        self.config.width * self.config.cell_size
    }

    /// Map height in world units.
    #[must_use]
    pub const fn world_height(&self) -> u32 {
        self.config.height * self.config.cell_size
    }
}

/// Simple deterministic RNG for map generation.
struct MapRng {
    state: u64,
}

impl MapRng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(0x9E3779B97F4A7C15),
        }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(0x5DEECE66D).wrapping_add(11);
        self.state
    }

    fn next_f32(&mut self) -> f32 {
        (self.next() % 10000) as f32 / 10000.0
    }

    fn next_range(&mut self, min: i32, max: i32) -> i32 {
        let range = (max - min) as u64;
        if range == 0 {
            return min;
        }
        min + (self.next() % range) as i32
    }
}

/// Generate a map with the given configuration.
#[must_use]
pub fn generate_map(config: MapConfig) -> GeneratedMap {
    let mut rng = MapRng::new(config.seed);
    let total_cells = (config.width * config.height) as usize;

    // Initialize all cells as walkable
    let mut terrain: Vec<TerrainCell> = vec![TerrainCell::walkable(); total_cells];

    // Generate spawn points first (to create safe zones)
    let spawn_points = generate_spawn_points(&config, &mut rng);

    // Mark spawn-safe zones
    let safe_radius = 8u32; // cells
    for spawn in &spawn_points {
        let (sx, sy) = world_to_grid_simple(&config, spawn.position);
        for dy in 0..=(safe_radius * 2) {
            for dx in 0..=(safe_radius * 2) {
                let nx = sx as i32 + dx as i32 - safe_radius as i32;
                let ny = sy as i32 + dy as i32 - safe_radius as i32;
                if nx >= 0 && ny >= 0 && (nx as u32) < config.width && (ny as u32) < config.height {
                    let idx = (ny as u32 * config.width + nx as u32) as usize;
                    terrain[idx].spawn_safe = true;
                }
            }
        }
    }

    // Generate obstacles
    generate_obstacles(&config, &mut terrain, &mut rng);

    // Apply symmetry to terrain
    apply_symmetry(&config, &mut terrain);

    // Generate resources
    let resources = generate_resources(&config, &spawn_points, &mut rng);

    GeneratedMap {
        config,
        terrain,
        resources,
        spawn_points,
    }
}

fn world_to_grid_simple(config: &MapConfig, pos: Vec2Fixed) -> (u32, u32) {
    let cell_size = Fixed::from_num(config.cell_size);
    let x = (pos.x / cell_size).to_num::<i32>().max(0) as u32;
    let y = (pos.y / cell_size).to_num::<i32>().max(0) as u32;
    (x.min(config.width - 1), y.min(config.height - 1))
}

fn generate_spawn_points(config: &MapConfig, rng: &mut MapRng) -> Vec<SpawnPoint> {
    let world_w = (config.width * config.cell_size) as i32;
    let world_h = (config.height * config.cell_size) as i32;
    let padding = (config.cell_size * 6) as i32;

    match config.symmetry {
        SymmetryMode::None => {
            // Random spawns with minimum distance
            let mut spawns = Vec::new();
            for _ in 0..2 {
                let x = rng.next_range(padding, world_w - padding);
                let y = rng.next_range(padding, world_h - padding);
                spawns.push(SpawnPoint {
                    position: Vec2Fixed::new(Fixed::from_num(x), Fixed::from_num(y)),
                    base_resources: Vec::new(),
                });
            }
            spawns
        }
        SymmetryMode::Horizontal => {
            let x1 = padding;
            let x2 = world_w - padding;
            let y = world_h / 2 + rng.next_range(-padding, padding);
            vec![
                SpawnPoint {
                    position: Vec2Fixed::new(Fixed::from_num(x1), Fixed::from_num(y)),
                    base_resources: Vec::new(),
                },
                SpawnPoint {
                    position: Vec2Fixed::new(Fixed::from_num(x2), Fixed::from_num(y)),
                    base_resources: Vec::new(),
                },
            ]
        }
        SymmetryMode::Vertical => {
            let x = world_w / 2 + rng.next_range(-padding, padding);
            let y1 = padding;
            let y2 = world_h - padding;
            vec![
                SpawnPoint {
                    position: Vec2Fixed::new(Fixed::from_num(x), Fixed::from_num(y1)),
                    base_resources: Vec::new(),
                },
                SpawnPoint {
                    position: Vec2Fixed::new(Fixed::from_num(x), Fixed::from_num(y2)),
                    base_resources: Vec::new(),
                },
            ]
        }
        SymmetryMode::Rotational => {
            // Corners with jitter
            let jitter = rng.next_range(-padding / 2, padding / 2);
            vec![
                SpawnPoint {
                    position: Vec2Fixed::new(
                        Fixed::from_num(padding + jitter),
                        Fixed::from_num(padding + jitter),
                    ),
                    base_resources: Vec::new(),
                },
                SpawnPoint {
                    position: Vec2Fixed::new(
                        Fixed::from_num(world_w - padding - jitter),
                        Fixed::from_num(world_h - padding - jitter),
                    ),
                    base_resources: Vec::new(),
                },
            ]
        }
        SymmetryMode::FourWay => {
            let offset = padding + rng.next_range(0, padding / 2);
            vec![
                SpawnPoint {
                    position: Vec2Fixed::new(Fixed::from_num(offset), Fixed::from_num(offset)),
                    base_resources: Vec::new(),
                },
                SpawnPoint {
                    position: Vec2Fixed::new(
                        Fixed::from_num(world_w - offset),
                        Fixed::from_num(offset),
                    ),
                    base_resources: Vec::new(),
                },
                SpawnPoint {
                    position: Vec2Fixed::new(
                        Fixed::from_num(offset),
                        Fixed::from_num(world_h - offset),
                    ),
                    base_resources: Vec::new(),
                },
                SpawnPoint {
                    position: Vec2Fixed::new(
                        Fixed::from_num(world_w - offset),
                        Fixed::from_num(world_h - offset),
                    ),
                    base_resources: Vec::new(),
                },
            ]
        }
    }
}

fn generate_obstacles(config: &MapConfig, terrain: &mut [TerrainCell], rng: &mut MapRng) {
    let target_obstacles = (config.width * config.height) as f32 * config.obstacle_density * 0.1;
    let num_features = target_obstacles.round() as u32;

    for _ in 0..num_features {
        // Random position
        let x = rng.next_range(2, config.width as i32 - 2) as u32;
        let y = rng.next_range(2, config.height as i32 - 2) as u32;
        let idx = (y * config.width + x) as usize;

        // Skip spawn-safe zones
        if terrain[idx].spawn_safe {
            continue;
        }

        // Random feature type
        let feature_type = rng.next() % 3;
        match feature_type {
            0 => {
                // Rock cluster (blocked cells)
                let size = rng.next_range(1, 4) as u32;
                for dy in 0..size {
                    for dx in 0..size {
                        let nx = x + dx;
                        let ny = y + dy;
                        if nx < config.width && ny < config.height {
                            let nidx = (ny * config.width + nx) as usize;
                            if !terrain[nidx].spawn_safe {
                                terrain[nidx].cell_type = CellType::Blocked;
                                terrain[nidx].height = rng.next_range(1, 4) as u8;
                            }
                        }
                    }
                }
            }
            1 => {
                // Rough terrain patch
                let size = rng.next_range(2, 5) as u32;
                for dy in 0..size {
                    for dx in 0..size {
                        let nx = x + dx;
                        let ny = y + dy;
                        if nx < config.width && ny < config.height {
                            let nidx = (ny * config.width + nx) as usize;
                            if !terrain[nidx].spawn_safe
                                && terrain[nidx].cell_type != CellType::Blocked
                            {
                                terrain[nidx].cell_type = CellType::SlowTerrain;
                            }
                        }
                    }
                }
            }
            _ => {
                // Chokepoint (horizontal or vertical wall with gap)
                let horizontal = rng.next() % 2 == 0;
                let length = rng.next_range(4, 8) as u32;
                let gap_pos = rng.next_range(1, length as i32 - 1) as u32;

                for i in 0..length {
                    if i == gap_pos || i == gap_pos + 1 {
                        continue; // Leave gap
                    }
                    let (nx, ny) = if horizontal { (x + i, y) } else { (x, y + i) };
                    if nx < config.width && ny < config.height {
                        let nidx = (ny * config.width + nx) as usize;
                        if !terrain[nidx].spawn_safe {
                            terrain[nidx].cell_type = CellType::Blocked;
                        }
                    }
                }
            }
        }
    }
}

fn apply_symmetry(config: &MapConfig, terrain: &mut [TerrainCell]) {
    let w = config.width;
    let h = config.height;

    match config.symmetry {
        SymmetryMode::None => {}
        SymmetryMode::Horizontal => {
            // Mirror left to right
            for y in 0..h {
                for x in 0..(w / 2) {
                    let mirror_x = w - 1 - x;
                    let src_idx = (y * w + x) as usize;
                    let dst_idx = (y * w + mirror_x) as usize;
                    terrain[dst_idx] = terrain[src_idx];
                }
            }
        }
        SymmetryMode::Vertical => {
            // Mirror top to bottom
            for y in 0..(h / 2) {
                for x in 0..w {
                    let mirror_y = h - 1 - y;
                    let src_idx = (y * w + x) as usize;
                    let dst_idx = (mirror_y * w + x) as usize;
                    terrain[dst_idx] = terrain[src_idx];
                }
            }
        }
        SymmetryMode::Rotational => {
            // 180-degree rotation
            for y in 0..(h / 2) {
                for x in 0..w {
                    let rot_x = w - 1 - x;
                    let rot_y = h - 1 - y;
                    let src_idx = (y * w + x) as usize;
                    let dst_idx = (rot_y * w + rot_x) as usize;
                    terrain[dst_idx] = terrain[src_idx];
                }
            }
        }
        SymmetryMode::FourWay => {
            // First horizontal, then vertical
            for y in 0..h {
                for x in 0..(w / 2) {
                    let mirror_x = w - 1 - x;
                    let src_idx = (y * w + x) as usize;
                    let dst_idx = (y * w + mirror_x) as usize;
                    terrain[dst_idx] = terrain[src_idx];
                }
            }
            for y in 0..(h / 2) {
                for x in 0..w {
                    let mirror_y = h - 1 - y;
                    let src_idx = (y * w + x) as usize;
                    let dst_idx = (mirror_y * w + x) as usize;
                    terrain[dst_idx] = terrain[src_idx];
                }
            }
        }
    }
}

fn generate_resources(
    config: &MapConfig,
    spawn_points: &[SpawnPoint],
    rng: &mut MapRng,
) -> Vec<ResourcePlacement> {
    let mut resources = Vec::new();
    let world_w = (config.width * config.cell_size) as i32;
    let world_h = (config.height * config.cell_size) as i32;
    let center = Vec2Fixed::new(Fixed::from_num(world_w / 2), Fixed::from_num(world_h / 2));

    // Base resources near each spawn (permanent, always safe income)
    for spawn in spawn_points {
        let base_offset = Fixed::from_num(80);
        let offsets = [
            Vec2Fixed::new(base_offset, Fixed::ZERO),
            Vec2Fixed::new(-base_offset, Fixed::ZERO),
            Vec2Fixed::new(Fixed::ZERO, base_offset),
        ];

        for offset in offsets {
            resources.push(ResourcePlacement::new(
                spawn.position + offset,
                5000,
                true, // permanent
            ));
        }
    }

    // Contested center resources (high value)
    let center_count = (3.0 * config.resource_density).round() as i32;
    for i in 0..center_count {
        let angle = (i as f32 / center_count as f32) * std::f32::consts::TAU;
        let radius = 40.0 + rng.next_f32() * 30.0;
        let offset = Vec2Fixed::new(
            Fixed::from_num(angle.cos() * radius),
            Fixed::from_num(angle.sin() * radius),
        );
        resources.push(ResourcePlacement::new(
            center + offset,
            10000,
            false, // temporary
        ));
    }

    // Expansion resources (between spawn and center)
    let expansion_count = (4.0 * config.resource_density).round() as i32;
    for spawn in spawn_points {
        let to_center = center - spawn.position;
        for i in 1..=expansion_count {
            let t = i as f32 / (expansion_count as f32 + 1.0);
            let base_pos = spawn.position
                + Vec2Fixed::new(
                    Fixed::from_num(to_center.x.to_num::<f32>() * t),
                    Fixed::from_num(to_center.y.to_num::<f32>() * t),
                );
            // Add some jitter
            let jitter = Vec2Fixed::new(
                Fixed::from_num(rng.next_range(-40, 41)),
                Fixed::from_num(rng.next_range(-40, 41)),
            );
            resources.push(ResourcePlacement::new(
                base_pos + jitter,
                (3000.0 * config.resource_density) as i64,
                false,
            ));
        }
    }

    resources
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = MapConfig::default();
        assert_eq!(config.width, 64);
        assert_eq!(config.height, 64);
        assert_eq!(config.symmetry, SymmetryMode::Rotational);
    }

    #[test]
    fn test_generate_small_map() {
        let config = MapConfig::small().with_seed(12345);
        let map = generate_map(config);

        assert_eq!(map.terrain.len(), 64 * 64);
        assert!(!map.spawn_points.is_empty());
        assert!(!map.resources.is_empty());
    }

    #[test]
    fn test_generate_medium_map() {
        let config = MapConfig::medium().with_seed(67890);
        let map = generate_map(config);

        assert_eq!(map.terrain.len(), 96 * 96);
        assert_eq!(map.world_width(), 768);
        assert_eq!(map.world_height(), 768);
    }

    #[test]
    fn test_generate_large_map() {
        let config = MapConfig::large().with_seed(11111);
        let map = generate_map(config);

        assert_eq!(map.terrain.len(), 128 * 128);
        assert_eq!(map.world_width(), 1024);
        assert_eq!(map.world_height(), 1024);
    }

    #[test]
    fn test_symmetry_rotational() {
        let config = MapConfig::small().with_seed(99999);
        let map = generate_map(config);

        // Check that spawn points are rotationally symmetric
        assert_eq!(map.spawn_points.len(), 2);
        let p1 = map.spawn_points[0].position;
        let p2 = map.spawn_points[1].position;
        let world_w = Fixed::from_num(map.world_width());
        let world_h = Fixed::from_num(map.world_height());

        // p2 should be roughly at (width - p1.x, height - p1.y)
        let expected_x = world_w - p1.x;
        let expected_y = world_h - p1.y;
        let tolerance = Fixed::from_num(32); // Allow some variance

        assert!((p2.x - expected_x).abs() < tolerance);
        assert!((p2.y - expected_y).abs() < tolerance);
    }

    #[test]
    fn test_spawn_safe_zones() {
        let config = MapConfig::small().with_seed(12345);
        let map = generate_map(config);

        // Cells near spawn should be spawn-safe
        for spawn in &map.spawn_points {
            let (gx, gy) = map.world_to_grid(spawn.position);
            if let Some(cell) = map.get_cell(gx, gy) {
                assert!(cell.spawn_safe, "Spawn cell should be spawn-safe");
                assert_eq!(
                    cell.cell_type,
                    CellType::Walkable,
                    "Spawn cell should be walkable"
                );
            }
        }
    }

    #[test]
    fn test_resources_near_spawns() {
        let config = MapConfig::small().with_seed(12345);
        let map = generate_map(config);

        // Each spawn should have nearby permanent resources
        // Using squared distance to avoid sqrt (150^2 = 22500)
        let threshold_sq = Fixed::from_num(22500);
        for spawn in &map.spawn_points {
            let nearby = map
                .resources
                .iter()
                .filter(|r| {
                    let dist_sq = r.position.distance_squared(spawn.position);
                    dist_sq < threshold_sq && r.permanent
                })
                .count();
            assert!(
                nearby >= 2,
                "Spawn {} should have at least 2 nearby permanent resources",
                nearby
            );
        }
    }

    #[test]
    fn test_contested_center_resources() {
        let config = MapConfig::small().with_seed(12345);
        let map = generate_map(config);

        let center = Vec2Fixed::new(
            Fixed::from_num(map.world_width() / 2),
            Fixed::from_num(map.world_height() / 2),
        );

        // Using squared distance to avoid sqrt (100^2 = 10000)
        let threshold_sq = Fixed::from_num(10000);
        let center_resources = map
            .resources
            .iter()
            .filter(|r| {
                let dist_sq = r.position.distance_squared(center);
                dist_sq < threshold_sq && !r.permanent
            })
            .count();

        assert!(
            center_resources >= 2,
            "Should have contested resources near center"
        );
    }

    #[test]
    fn test_obstacle_density() {
        let open_config = MapConfig::small()
            .with_seed(12345)
            .with_obstacle_density(0.0);
        let open_map = generate_map(open_config);

        // Count non-spawn-safe blocked cells
        let open_blocked = open_map
            .terrain
            .iter()
            .filter(|c| c.cell_type == CellType::Blocked && !c.spawn_safe)
            .count();

        let dense_config = MapConfig::small()
            .with_seed(12345)
            .with_obstacle_density(0.5);
        let dense_map = generate_map(dense_config);

        let dense_blocked = dense_map
            .terrain
            .iter()
            .filter(|c| c.cell_type == CellType::Blocked && !c.spawn_safe)
            .count();

        // Dense should have more obstacles (though symmetry affects this)
        assert!(dense_blocked >= open_blocked);
    }

    #[test]
    fn test_determinism() {
        let config1 = MapConfig::small().with_seed(42);
        let config2 = MapConfig::small().with_seed(42);

        let map1 = generate_map(config1);
        let map2 = generate_map(config2);

        // Same seed should produce identical maps
        assert_eq!(map1.terrain.len(), map2.terrain.len());
        for (c1, c2) in map1.terrain.iter().zip(map2.terrain.iter()) {
            assert_eq!(c1.cell_type, c2.cell_type);
        }
        assert_eq!(map1.resources.len(), map2.resources.len());
        assert_eq!(map1.spawn_points.len(), map2.spawn_points.len());
    }

    #[test]
    fn test_different_seeds() {
        let map1 = generate_map(MapConfig::small().with_seed(1));
        let map2 = generate_map(MapConfig::small().with_seed(2));

        // Different seeds should produce different spawn positions
        let p1 = map1.spawn_points[0].position;
        let p2 = map2.spawn_points[0].position;
        assert!(p1 != p2 || map1.spawn_points[1].position != map2.spawn_points[1].position);
    }

    #[test]
    fn test_coordinate_conversion() {
        let config = MapConfig::small();
        let map = generate_map(config);

        // World (0,0) should map to grid (0,0)
        let pos = Vec2Fixed::new(Fixed::from_num(0), Fixed::from_num(0));
        let (gx, gy) = map.world_to_grid(pos);
        assert_eq!((gx, gy), (0, 0));

        // Grid (0,0) center is at (4, 4) with cell_size=8
        let world_pos = map.grid_to_world(0, 0);
        assert_eq!(world_pos.x, Fixed::from_num(4));
        assert_eq!(world_pos.y, Fixed::from_num(4));
    }

    #[test]
    fn test_as_cell_types() {
        let map = generate_map(MapConfig::small().with_seed(12345));
        let cells = map.as_cell_types();

        assert_eq!(cells.len(), map.terrain.len());
        // All should be valid cell types
        for cell in cells {
            assert!(matches!(
                cell,
                CellType::Walkable | CellType::Blocked | CellType::SlowTerrain
            ));
        }
    }
}
