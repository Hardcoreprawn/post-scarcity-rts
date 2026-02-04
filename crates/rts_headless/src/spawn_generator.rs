//! Dynamic spawn position generator.
//!
//! Generates randomized but balanced spawn positions for factions,
//! ensuring fair distances and strategic variety.

use crate::scenario::Scenario;

/// Spawn configuration options.
#[derive(Debug, Clone)]
pub struct SpawnConfig {
    /// Minimum distance between faction HQs.
    pub min_base_distance: f32,
    /// Maximum distance between faction HQs.
    pub max_base_distance: f32,
    /// Minimum distance from map edge.
    pub edge_padding: i32,
    /// Spawn pattern to use.
    pub pattern: SpawnPattern,
    /// Enable mirrored spawns (symmetric).
    pub mirrored: bool,
    /// Randomness factor (0.0 = fixed, 1.0 = full random).
    pub randomness: f32,
}

impl Default for SpawnConfig {
    fn default() -> Self {
        Self {
            min_base_distance: 300.0,
            max_base_distance: 450.0,
            edge_padding: 48,
            pattern: SpawnPattern::Corners,
            mirrored: true,
            randomness: 0.5,
        }
    }
}

/// Predefined spawn patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnPattern {
    /// Spawn at opposite corners (classic).
    Corners,
    /// Spawn on opposite sides (horizontal).
    Horizontal,
    /// Spawn on opposite sides (vertical).
    Vertical,
    /// Spawn at random positions with distance constraints.
    Random,
    /// Circular arena with central resources.
    Arena,
    /// Cross pattern (4 spawn points).
    Cross,
}

/// Simple deterministic RNG for spawns.
#[derive(Debug, Clone)]
pub struct SpawnRng {
    state: u64,
}

impl SpawnRng {
    /// Create new RNG from seed.
    pub fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(0x9E3779B97F4A7C15),
        }
    }

    /// Get next random value.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(0x5DEECE66D).wrapping_add(11);
        self.state
    }

    /// Get random float in 0.0..1.0.
    pub fn next_f32(&mut self) -> f32 {
        (self.next() % 10000) as f32 / 10000.0
    }

    /// Get random int in range [min, max).
    pub fn next_range(&mut self, min: i32, max: i32) -> i32 {
        let range = (max - min) as u64;
        if range == 0 {
            return min;
        }
        min + (self.next() % range) as i32
    }
}

/// Generate spawn positions for factions.
pub fn generate_spawns(
    map_size: (u32, u32),
    num_factions: u32,
    seed: u64,
    config: &SpawnConfig,
) -> Vec<(i32, i32)> {
    let mut rng = SpawnRng::new(seed);
    let (map_w, map_h) = (map_size.0 as i32, map_size.1 as i32);
    let padding = config.edge_padding;

    match config.pattern {
        SpawnPattern::Corners => generate_corner_spawns(map_w, map_h, padding, &mut rng, config),
        SpawnPattern::Horizontal => {
            generate_horizontal_spawns(map_w, map_h, padding, &mut rng, config)
        }
        SpawnPattern::Vertical => generate_vertical_spawns(map_w, map_h, padding, &mut rng, config),
        SpawnPattern::Random => {
            generate_random_spawns(map_w, map_h, padding, num_factions, &mut rng, config)
        }
        SpawnPattern::Arena => generate_arena_spawns(map_w, map_h, padding, &mut rng, config),
        SpawnPattern::Cross => generate_cross_spawns(map_w, map_h, padding, &mut rng, config),
    }
}

fn generate_corner_spawns(
    map_w: i32,
    map_h: i32,
    padding: i32,
    rng: &mut SpawnRng,
    config: &SpawnConfig,
) -> Vec<(i32, i32)> {
    let corners = [
        (padding, padding),                 // Top-left
        (map_w - padding, padding),         // Top-right
        (padding, map_h - padding),         // Bottom-left
        (map_w - padding, map_h - padding), // Bottom-right
    ];

    // Select two opposite corners
    let corner_pair = (rng.next() % 2) as usize;
    let (c1, c2) = if corner_pair == 0 {
        (corners[0], corners[3]) // TL-BR diagonal
    } else {
        (corners[1], corners[2]) // TR-BL diagonal
    };

    // Add randomness
    let jitter = (config.randomness * 50.0) as i32;
    let spawn1 = (
        c1.0 + rng.next_range(-jitter, jitter + 1),
        c1.1 + rng.next_range(-jitter, jitter + 1),
    );
    let spawn2 = (
        c2.0 + rng.next_range(-jitter, jitter + 1),
        c2.1 + rng.next_range(-jitter, jitter + 1),
    );

    vec![
        clamp_spawn(spawn1, map_w, map_h, padding),
        clamp_spawn(spawn2, map_w, map_h, padding),
    ]
}

fn generate_horizontal_spawns(
    map_w: i32,
    map_h: i32,
    padding: i32,
    rng: &mut SpawnRng,
    config: &SpawnConfig,
) -> Vec<(i32, i32)> {
    let center_y = map_h / 2;
    let jitter_y = (config.randomness * (map_h as f32 / 4.0)) as i32;

    let y1 = center_y + rng.next_range(-jitter_y, jitter_y + 1);
    let y2 = if config.mirrored {
        map_h - y1 // Mirror around center
    } else {
        center_y + rng.next_range(-jitter_y, jitter_y + 1)
    };

    vec![
        clamp_spawn((padding, y1), map_w, map_h, padding),
        clamp_spawn((map_w - padding, y2), map_w, map_h, padding),
    ]
}

fn generate_vertical_spawns(
    map_w: i32,
    map_h: i32,
    padding: i32,
    rng: &mut SpawnRng,
    config: &SpawnConfig,
) -> Vec<(i32, i32)> {
    let center_x = map_w / 2;
    let jitter_x = (config.randomness * (map_w as f32 / 4.0)) as i32;

    let x1 = center_x + rng.next_range(-jitter_x, jitter_x + 1);
    let x2 = if config.mirrored {
        map_w - x1
    } else {
        center_x + rng.next_range(-jitter_x, jitter_x + 1)
    };

    vec![
        clamp_spawn((x1, padding), map_w, map_h, padding),
        clamp_spawn((x2, map_h - padding), map_w, map_h, padding),
    ]
}

fn generate_random_spawns(
    map_w: i32,
    map_h: i32,
    padding: i32,
    num_factions: u32,
    rng: &mut SpawnRng,
    config: &SpawnConfig,
) -> Vec<(i32, i32)> {
    let mut spawns = Vec::new();
    let min_dist_sq = config.min_base_distance * config.min_base_distance;

    for _ in 0..num_factions {
        // Try to find a valid spawn position
        for _ in 0..100 {
            let x = rng.next_range(padding, map_w - padding);
            let y = rng.next_range(padding, map_h - padding);

            // Check distance from other spawns
            let mut valid = true;
            for &(sx, sy) in &spawns {
                let dx = (x - sx) as f32;
                let dy = (y - sy) as f32;
                if dx * dx + dy * dy < min_dist_sq {
                    valid = false;
                    break;
                }
            }

            if valid {
                spawns.push((x, y));
                break;
            }
        }
    }

    // Fallback: if we couldn't find valid spawns, use horizontal
    if spawns.len() < num_factions as usize {
        return generate_horizontal_spawns(map_w, map_h, padding, rng, config);
    }

    spawns
}

fn generate_arena_spawns(
    map_w: i32,
    map_h: i32,
    padding: i32,
    rng: &mut SpawnRng,
    config: &SpawnConfig,
) -> Vec<(i32, i32)> {
    let center = (map_w / 2, map_h / 2);
    let radius = ((map_w.min(map_h) / 2) - padding) as f32;

    // Spawn at opposite points on circle
    let base_angle = rng.next_f32() * std::f32::consts::PI;
    let jitter = config.randomness * 0.3;

    let angle1 = base_angle + rng.next_f32() * jitter - jitter / 2.0;
    let angle2 = angle1 + std::f32::consts::PI; // Opposite side

    let spawn1 = (
        center.0 + (radius * angle1.cos()) as i32,
        center.1 + (radius * angle1.sin()) as i32,
    );
    let spawn2 = (
        center.0 + (radius * angle2.cos()) as i32,
        center.1 + (radius * angle2.sin()) as i32,
    );

    vec![
        clamp_spawn(spawn1, map_w, map_h, padding),
        clamp_spawn(spawn2, map_w, map_h, padding),
    ]
}

fn generate_cross_spawns(
    map_w: i32,
    map_h: i32,
    padding: i32,
    rng: &mut SpawnRng,
    config: &SpawnConfig,
) -> Vec<(i32, i32)> {
    let cross_points = [
        (map_w / 2, padding),         // Top
        (map_w - padding, map_h / 2), // Right
        (map_w / 2, map_h - padding), // Bottom
        (padding, map_h / 2),         // Left
    ];

    // Pick two opposite points
    let pair = (rng.next() % 2) as usize;
    let (c1, c2) = if pair == 0 {
        (cross_points[0], cross_points[2]) // Top-Bottom
    } else {
        (cross_points[1], cross_points[3]) // Right-Left
    };

    let jitter = (config.randomness * 30.0) as i32;
    let spawn1 = (
        c1.0 + rng.next_range(-jitter, jitter + 1),
        c1.1 + rng.next_range(-jitter, jitter + 1),
    );
    let spawn2 = (
        c2.0 + rng.next_range(-jitter, jitter + 1),
        c2.1 + rng.next_range(-jitter, jitter + 1),
    );

    vec![
        clamp_spawn(spawn1, map_w, map_h, padding),
        clamp_spawn(spawn2, map_w, map_h, padding),
    ]
}

fn clamp_spawn(pos: (i32, i32), map_w: i32, map_h: i32, padding: i32) -> (i32, i32) {
    (
        pos.0.clamp(padding, map_w - padding),
        pos.1.clamp(padding, map_h - padding),
    )
}

/// Generate a complete randomized scenario based on seed.
pub fn generate_dynamic_scenario(
    seed: u64,
    base_scenario: &Scenario,
    spawn_config: &SpawnConfig,
) -> Scenario {
    let mut scenario = base_scenario.clone();
    let mut rng = SpawnRng::new(seed);

    // Randomly pick spawn pattern based on seed
    let patterns = [
        SpawnPattern::Corners,
        SpawnPattern::Horizontal,
        SpawnPattern::Vertical,
        SpawnPattern::Arena,
        SpawnPattern::Cross,
    ];
    let pattern_idx = (rng.next() % patterns.len() as u64) as usize;
    let mut config = spawn_config.clone();
    config.pattern = patterns[pattern_idx];

    // Generate spawn positions
    let spawns = generate_spawns(
        scenario.map_size,
        scenario.factions.len() as u32,
        seed,
        &config,
    );

    // Update faction spawn positions
    for (i, faction) in scenario.factions.iter_mut().enumerate() {
        if i < spawns.len() {
            let (x, y) = spawns[i];
            faction.spawn_position = (x, y);

            // Update building positions relative to spawn
            for building in faction.starting_buildings.iter_mut() {
                let offset_x = building.position.0 - faction.spawn_position.0;
                let offset_y = building.position.1 - faction.spawn_position.1;
                building.position = (x + offset_x, y + offset_y);
            }

            // Update unit positions relative to spawn
            for unit in faction.starting_units.iter_mut() {
                let offset_x = unit.position.0 - faction.spawn_position.0;
                let offset_y = unit.position.1 - faction.spawn_position.1;
                unit.position = (x + offset_x, y + offset_y);
            }
        }
    }

    // Vary starting resources slightly
    let resource_variance = (config.randomness * 200.0) as i64;
    for faction in scenario.factions.iter_mut() {
        let variance =
            rng.next_range(-resource_variance as i32, resource_variance as i32 + 1) as i64;
        faction.starting_resources = (faction.starting_resources + variance).max(500);
    }

    scenario.name = format!("{} (seed:{})", scenario.name, seed);
    scenario
}

/// Visual balance metrics for spawn positions.
#[derive(Debug, Clone)]
pub struct SpawnBalanceMetrics {
    /// Distance between bases.
    pub base_distance: f32,
    /// Average distance to center resources.
    pub avg_resource_distance: f32,
    /// Distance difference to center (fairness).
    pub center_distance_fairness: f32,
    /// Spawn pattern used.
    pub pattern: SpawnPattern,
    /// Overall balance score (0-100).
    pub balance_score: u32,
}

impl SpawnBalanceMetrics {
    /// Evaluate spawn balance for a scenario.
    pub fn evaluate(scenario: &Scenario) -> Self {
        let spawns: Vec<(f32, f32)> = scenario
            .factions
            .iter()
            .map(|f| (f.spawn_position.0 as f32, f.spawn_position.1 as f32))
            .collect();

        if spawns.len() < 2 {
            return Self {
                base_distance: 0.0,
                avg_resource_distance: 0.0,
                center_distance_fairness: 0.0,
                pattern: SpawnPattern::Horizontal,
                balance_score: 50,
            };
        }

        // Calculate base distance
        let dx = spawns[0].0 - spawns[1].0;
        let dy = spawns[0].1 - spawns[1].1;
        let base_distance = (dx * dx + dy * dy).sqrt();

        // Calculate distance to map center
        let center = (
            scenario.map_size.0 as f32 / 2.0,
            scenario.map_size.1 as f32 / 2.0,
        );
        let dist1 = ((spawns[0].0 - center.0).powi(2) + (spawns[0].1 - center.1).powi(2)).sqrt();
        let dist2 = ((spawns[1].0 - center.0).powi(2) + (spawns[1].1 - center.1).powi(2)).sqrt();
        let center_distance_fairness = 1.0 - (dist1 - dist2).abs() / dist1.max(dist2).max(1.0);

        // Calculate average resource distance
        let mut total_resource_dist = 0.0;
        let mut resource_count = 0;
        for ore in &scenario.initial_resources.ore_nodes {
            let (ox, oy) = (ore.position.0 as f32, ore.position.1 as f32);
            for spawn in &spawns {
                let d = ((spawn.0 - ox).powi(2) + (spawn.1 - oy).powi(2)).sqrt();
                total_resource_dist += d;
                resource_count += 1;
            }
        }
        let avg_resource_distance = if resource_count > 0 {
            total_resource_dist / resource_count as f32
        } else {
            0.0
        };

        // Determine pattern from positions
        let pattern = if dx.abs() > dy.abs() * 3.0 {
            SpawnPattern::Horizontal
        } else if dy.abs() > dx.abs() * 3.0 {
            SpawnPattern::Vertical
        } else {
            SpawnPattern::Corners
        };

        // Calculate overall balance score
        let mut score = 100u32;

        // Penalize if bases too close or too far
        if base_distance < 200.0 {
            score = score.saturating_sub(30);
        } else if base_distance > 600.0 {
            score = score.saturating_sub(15);
        }

        // Penalize unfair center distance
        if center_distance_fairness < 0.8 {
            score = score.saturating_sub(((1.0 - center_distance_fairness) * 50.0) as u32);
        }

        Self {
            base_distance,
            avg_resource_distance,
            center_distance_fairness,
            pattern,
            balance_score: score,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rng_determinism() {
        let mut rng1 = SpawnRng::new(12345);
        let mut rng2 = SpawnRng::new(12345);

        for _ in 0..100 {
            assert_eq!(rng1.next(), rng2.next());
        }
    }

    #[test]
    fn test_corner_spawns() {
        let spawns = generate_spawns((512, 512), 2, 42, &SpawnConfig::default());
        assert_eq!(spawns.len(), 2);

        // Check distance between spawns is reasonable
        let dx = (spawns[0].0 - spawns[1].0) as f32;
        let dy = (spawns[0].1 - spawns[1].1) as f32;
        let dist = (dx * dx + dy * dy).sqrt();
        assert!(dist > 200.0, "Spawns too close: {}", dist);
    }

    #[test]
    fn test_horizontal_spawns() {
        let config = SpawnConfig {
            pattern: SpawnPattern::Horizontal,
            ..Default::default()
        };
        let spawns = generate_spawns((512, 512), 2, 42, &config);
        assert_eq!(spawns.len(), 2);

        // Horizontal: first spawn should be on left, second on right
        assert!(spawns[0].0 < 256);
        assert!(spawns[1].0 > 256);
    }

    #[test]
    fn test_arena_spawns() {
        let config = SpawnConfig {
            pattern: SpawnPattern::Arena,
            ..Default::default()
        };
        let spawns = generate_spawns((512, 512), 2, 42, &config);
        assert_eq!(spawns.len(), 2);

        // Arena: spawns should be roughly opposite each other around center
        let center = (256.0, 256.0);
        let v1 = (spawns[0].0 as f32 - center.0, spawns[0].1 as f32 - center.1);
        let v2 = (spawns[1].0 as f32 - center.0, spawns[1].1 as f32 - center.1);

        // Dot product should be negative (opposite directions)
        let dot = v1.0 * v2.0 + v1.1 * v2.1;
        assert!(dot < 0.0, "Spawns not opposite: dot={}", dot);
    }

    #[test]
    fn test_different_seeds_different_spawns() {
        let spawns1 = generate_spawns((512, 512), 2, 1, &SpawnConfig::default());
        let spawns2 = generate_spawns((512, 512), 2, 999, &SpawnConfig::default());

        // At least one coordinate should differ
        assert!(spawns1[0] != spawns2[0] || spawns1[1] != spawns2[1]);
    }

    #[test]
    fn test_spawn_balance_metrics() {
        let scenario = Scenario::skirmish_1v1();
        let metrics = SpawnBalanceMetrics::evaluate(&scenario);

        assert!(metrics.base_distance > 200.0);
        assert!(metrics.balance_score >= 70);
    }
}
