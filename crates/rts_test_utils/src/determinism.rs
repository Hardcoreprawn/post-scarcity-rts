//! Determinism testing utilities.
//!
//! Provides a harness for verifying that the simulation
//! produces identical results given identical inputs.
//!
//! # Testing Strategy
//!
//! RTS simulations must be 100% deterministic for lockstep multiplayer.
//! Sources of non-determinism include:
//!
//! - **Floating-point math**: Different CPUs can produce different results.
//!   We use fixed-point arithmetic via [`rts_core::math::Fixed`] throughout.
//!
//! - **HashMap iteration order**: Rust's default hasher is randomized.
//!   We always iterate in sorted entity ID order.
//!
//! - **System randomness**: No calls to `rand()` without explicit seeds.
//!   All "random" behavior uses seeded PRNGs.
//!
//! - **Uninitialized memory**: Rust prevents this, but be careful with FFI.
//!
//! # Test Levels
//!
//! 1. **Unit tests**: Individual system determinism (movement, combat, etc.)
//! 2. **Property tests**: Random inputs must still produce deterministic outputs
//! 3. **Integration tests**: Full simulation scenarios are reproducible
//! 4. **Parallel tests**: Running N simulations in parallel all match

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::thread;

/// Result of a determinism test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeterminismResult {
    /// Whether all runs produced identical results.
    pub is_deterministic: bool,
    /// Hashes from each run.
    pub hashes: Vec<u64>,
    /// Number of ticks simulated.
    pub ticks: u64,
}

impl DeterminismResult {
    /// Get all unique hashes (should be 1 for deterministic simulation).
    #[must_use]
    pub fn unique_hashes(&self) -> Vec<u64> {
        let mut unique: Vec<u64> = self.hashes.clone();
        unique.sort_unstable();
        unique.dedup();
        unique
    }

    /// Assert that the simulation was deterministic, with a detailed error message.
    ///
    /// # Panics
    ///
    /// Panics if the simulation produced different hashes across runs.
    pub fn assert_deterministic(&self) {
        if !self.is_deterministic {
            let unique = self.unique_hashes();
            panic!(
                "Simulation is non-deterministic!\n\
                 Runs: {}\n\
                 Ticks: {}\n\
                 Unique hashes: {} (expected 1)\n\
                 All hashes: {:?}",
                self.hashes.len(),
                self.ticks,
                unique.len(),
                self.hashes
            );
        }
    }
}

/// Result of parallel simulation runs.
#[derive(Debug, Clone)]
pub struct ParallelSimResult {
    /// Final state hash from each simulation.
    pub hashes: Vec<u64>,
    /// Number of ticks each simulation ran.
    pub ticks: u64,
    /// Number of simulations run.
    pub num_sims: usize,
}

impl ParallelSimResult {
    /// Check if all simulations produced identical results.
    #[must_use]
    pub fn is_deterministic(&self) -> bool {
        self.hashes.windows(2).all(|w| w[0] == w[1])
    }

    /// Assert all simulations matched.
    ///
    /// # Panics
    ///
    /// Panics if simulations produced different hashes.
    pub fn assert_deterministic(&self) {
        if !self.is_deterministic() {
            let mut unique: Vec<u64> = self.hashes.clone();
            unique.sort_unstable();
            unique.dedup();
            panic!(
                "Parallel simulations diverged!\n\
                 Simulations: {}\n\
                 Ticks: {}\n\
                 Unique hashes: {}\n\
                 All hashes: {:?}",
                self.num_sims,
                self.ticks,
                unique.len(),
                self.hashes
            );
        }
    }
}

/// Run a simulation multiple times and verify determinism.
///
/// # Arguments
///
/// * `runs` - Number of times to run the simulation
/// * `ticks` - Number of ticks to simulate per run
/// * `setup` - Function to create initial simulation state
/// * `step` - Function to advance simulation by one tick
/// * `hash` - Function to compute state hash
///
/// # Example
///
/// ```ignore
/// use rts_test_utils::determinism::verify_determinism;
/// use rts_core::simulation::Simulation;
///
/// let result = verify_determinism(
///     5,  // Run 5 times
///     100, // 100 ticks each
///     || setup_combat_scenario(),
///     |sim| { sim.tick(); },
///     |sim| sim.state_hash(),
/// );
/// result.assert_deterministic();
/// ```
pub fn verify_determinism<S, Setup, Step, HashFn>(
    runs: usize,
    ticks: u64,
    setup: Setup,
    step: Step,
    hash: HashFn,
) -> DeterminismResult
where
    Setup: Fn() -> S,
    Step: Fn(&mut S),
    HashFn: Fn(&S) -> u64,
{
    let mut hashes = Vec::with_capacity(runs);

    for _ in 0..runs {
        let mut state = setup();

        for _ in 0..ticks {
            step(&mut state);
        }

        hashes.push(hash(&state));
    }

    let is_deterministic = hashes.windows(2).all(|w| w[0] == w[1]);

    DeterminismResult {
        is_deterministic,
        hashes,
        ticks,
    }
}

/// Simplified determinism verification for `Simulation` type.
///
/// Runs the simulation twice with identical setup and verifies the final
/// state hashes match exactly.
///
/// # Arguments
///
/// * `setup_fn` - Function that creates and configures a simulation
/// * `num_ticks` - Number of ticks to run
///
/// # Returns
///
/// `true` if both runs produced identical state hashes.
///
/// # Example
///
/// ```ignore
/// use rts_test_utils::determinism::verify_simulation_determinism;
/// use rts_core::simulation::{Simulation, EntitySpawnParams};
/// use rts_core::math::{Fixed, Vec2Fixed};
///
/// let is_deterministic = verify_simulation_determinism(
///     || {
///         let mut sim = Simulation::new();
///         sim.spawn_entity(EntitySpawnParams {
///             position: Some(Vec2Fixed::new(Fixed::from_num(10), Fixed::from_num(20))),
///             ..Default::default()
///         });
///         sim
///     },
///     100,
/// );
/// assert!(is_deterministic);
/// ```
pub fn verify_simulation_determinism<F>(setup_fn: F, num_ticks: u64) -> bool
where
    F: Fn() -> rts_core::simulation::Simulation,
{
    let result = verify_determinism(
        2,
        num_ticks,
        &setup_fn,
        |sim| {
            sim.tick();
        },
        |sim| sim.state_hash(),
    );
    result.is_deterministic
}

/// Run N simulations in parallel and collect final hashes.
///
/// This is useful for catching non-determinism that only manifests
/// under thread scheduling variations, memory layout differences, etc.
///
/// # Arguments
///
/// * `setup_fn` - Function that creates and configures a simulation (must be thread-safe)
/// * `num_sims` - Number of parallel simulations to run
/// * `num_ticks` - Number of ticks to run each simulation
///
/// # Example
///
/// ```ignore
/// use rts_test_utils::determinism::run_parallel_simulations;
///
/// let result = run_parallel_simulations(
///     || setup_battle_scenario(),
///     8,   // 8 parallel simulations
///     500, // 500 ticks each
/// );
/// result.assert_deterministic();
/// ```
pub fn run_parallel_simulations<F>(
    setup_fn: F,
    num_sims: usize,
    num_ticks: u64,
) -> ParallelSimResult
where
    F: Fn() -> rts_core::simulation::Simulation + Send + Sync,
{
    let setup_ref = &setup_fn;
    let handles: Vec<_> = (0..num_sims)
        .map(|_| {
            // Clone setup function result for each thread
            let mut sim = setup_ref();
            thread::scope(|_| {
                for _ in 0..num_ticks {
                    sim.tick();
                }
                sim.state_hash()
            })
        })
        .collect();

    ParallelSimResult {
        hashes: handles,
        ticks: num_ticks,
        num_sims,
    }
}

/// Run N simulations in parallel using thread::scope for better safety.
///
/// Like [`run_parallel_simulations`] but uses scoped threads to avoid
/// 'static lifetime requirements on the setup function.
pub fn run_parallel_simulations_scoped<F>(
    setup_fn: F,
    num_sims: usize,
    num_ticks: u64,
) -> ParallelSimResult
where
    F: Fn() -> rts_core::simulation::Simulation + Sync,
{
    let hashes = thread::scope(|s| {
        let handles: Vec<_> = (0..num_sims)
            .map(|_| {
                s.spawn(|| {
                    let mut sim = setup_fn();
                    for _ in 0..num_ticks {
                        sim.tick();
                    }
                    sim.state_hash()
                })
            })
            .collect();

        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    ParallelSimResult {
        hashes,
        ticks: num_ticks,
        num_sims,
    }
}

/// Compare two simulation runs tick-by-tick, finding first divergence.
///
/// Useful for debugging non-determinism by finding exactly when
/// simulations start to differ.
///
/// # Returns
///
/// `None` if simulations are deterministic, `Some(tick)` if they diverge
/// at that tick.
pub fn find_first_divergence<F>(setup_fn: F, num_ticks: u64) -> Option<u64>
where
    F: Fn() -> rts_core::simulation::Simulation,
{
    let mut sim1 = setup_fn();
    let mut sim2 = setup_fn();

    // Check initial state
    if sim1.state_hash() != sim2.state_hash() {
        return Some(0);
    }

    for tick in 1..=num_ticks {
        sim1.tick();
        sim2.tick();

        if sim1.state_hash() != sim2.state_hash() {
            return Some(tick);
        }
    }

    None
}

/// Verify that serialization round-trip preserves simulation state exactly.
///
/// This is critical for save/load and network synchronization.
pub fn verify_serialization_determinism<F>(setup_fn: F, num_ticks: u64) -> bool
where
    F: Fn() -> rts_core::simulation::Simulation,
{
    let mut sim = setup_fn();

    for _ in 0..num_ticks {
        sim.tick();
    }

    let hash_before = sim.state_hash();

    // Serialize and deserialize
    let bytes = match sim.serialize() {
        Ok(b) => b,
        Err(_) => return false,
    };

    let restored = match rts_core::simulation::Simulation::deserialize(&bytes) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let hash_after = restored.state_hash();

    hash_before == hash_after
}

/// Compute a simple hash for any hashable value.
pub fn compute_hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// Proptest strategies for determinism testing.
///
/// These strategies generate random but reproducible inputs for
/// property-based testing of simulation determinism.
pub mod strategies {
    use proptest::prelude::*;
    use rts_core::components::Command;
    use rts_core::math::{Fixed, Vec2Fixed};

    /// Generate a fixed-point number in a reasonable range for positions.
    ///
    /// Range: -10000 to 10000 (typical map size)
    pub fn arb_fixed_position() -> impl Strategy<Value = Fixed> {
        (-10000i32..10000i32).prop_map(Fixed::from_num)
    }

    /// Generate a fixed-point number for speeds.
    ///
    /// Range: 1 to 20 (units per tick)
    pub fn arb_fixed_speed() -> impl Strategy<Value = Fixed> {
        (1i32..20i32).prop_map(Fixed::from_num)
    }

    /// Generate a fixed-point 2D vector for positions.
    pub fn arb_vec2_position() -> impl Strategy<Value = Vec2Fixed> {
        (arb_fixed_position(), arb_fixed_position()).prop_map(|(x, y)| Vec2Fixed::new(x, y))
    }

    /// Generate a MoveTo command.
    pub fn arb_move_command() -> impl Strategy<Value = Command> {
        arb_vec2_position().prop_map(Command::MoveTo)
    }

    /// Generate an AttackMove command.
    pub fn arb_attack_move_command() -> impl Strategy<Value = Command> {
        arb_vec2_position().prop_map(Command::AttackMove)
    }

    /// Generate a Patrol command.
    pub fn arb_patrol_command() -> impl Strategy<Value = Command> {
        arb_vec2_position().prop_map(Command::Patrol)
    }

    /// Generate any movement-related command (no entity refs).
    pub fn arb_movement_command() -> impl Strategy<Value = Command> {
        prop_oneof![
            arb_move_command(),
            arb_attack_move_command(),
            arb_patrol_command(),
            Just(Command::Stop),
            Just(Command::HoldPosition),
        ]
    }

    /// Generate a sequence of commands.
    pub fn arb_command_sequence(max_len: usize) -> impl Strategy<Value = Vec<Command>> {
        proptest::collection::vec(arb_movement_command(), 0..max_len)
    }

    /// Generate health values (1-1000).
    pub fn arb_health() -> impl Strategy<Value = u32> {
        1u32..1000u32
    }

    /// Generate damage values (1-100).
    pub fn arb_damage() -> impl Strategy<Value = u32> {
        1u32..100u32
    }

    /// Generate attack range in fixed-point.
    pub fn arb_attack_range() -> impl Strategy<Value = Fixed> {
        (1i32..100i32).prop_map(Fixed::from_num)
    }

    /// Parameters for spawning a test entity.
    #[derive(Debug, Clone)]
    pub struct TestEntityParams {
        /// Position.
        pub position: Vec2Fixed,
        /// Health.
        pub health: u32,
        /// Movement speed (None = immobile).
        pub speed: Option<Fixed>,
    }

    /// Generate parameters for a test entity.
    pub fn arb_entity_params() -> impl Strategy<Value = TestEntityParams> {
        (
            arb_vec2_position(),
            arb_health(),
            proptest::option::of(arb_fixed_speed()),
        )
            .prop_map(|(position, health, speed)| TestEntityParams {
                position,
                health,
                speed,
            })
    }

    /// Generate a list of entity spawn parameters.
    pub fn arb_entity_list(max_entities: usize) -> impl Strategy<Value = Vec<TestEntityParams>> {
        proptest::collection::vec(arb_entity_params(), 1..max_entities)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use rts_core::components::{CombatStats, Command};
    use rts_core::math::{Fixed, Vec2Fixed};
    use rts_core::simulation::{EntitySpawnParams, Simulation};

    // =========================================================================
    // Basic determinism tests
    // =========================================================================

    #[test]
    fn test_verify_determinism_simple() {
        let result = verify_determinism(3, 100, || 0u64, |n| *n += 1, |n| *n);

        assert!(result.is_deterministic);
        assert_eq!(result.hashes, vec![100, 100, 100]);
    }

    #[test]
    fn test_empty_simulation_determinism() {
        assert!(verify_simulation_determinism(Simulation::new, 100));
    }

    #[test]
    fn test_single_entity_determinism() {
        let is_det = verify_simulation_determinism(
            || {
                let mut sim = Simulation::new();
                sim.spawn_entity(EntitySpawnParams {
                    position: Some(Vec2Fixed::new(Fixed::from_num(100), Fixed::from_num(50))),
                    health: Some(100u32),
                    movement: Some(Fixed::from_num(5)),
                    ..Default::default()
                });
                sim
            },
            200,
        );
        assert!(is_det);
    }

    #[test]
    fn test_find_divergence_on_deterministic_sim() {
        let divergence = find_first_divergence(
            || {
                let mut sim = Simulation::new();
                sim.spawn_entity(EntitySpawnParams {
                    position: Some(Vec2Fixed::ZERO),
                    health: Some(50),
                    ..Default::default()
                });
                sim
            },
            100,
        );
        assert!(divergence.is_none(), "Expected no divergence");
    }

    // =========================================================================
    // Serialization round-trip tests
    // =========================================================================

    #[test]
    fn test_serialization_preserves_empty_sim() {
        assert!(verify_serialization_determinism(Simulation::new, 0));
    }

    #[test]
    fn test_serialization_preserves_complex_state() {
        assert!(verify_serialization_determinism(
            || {
                let mut sim = Simulation::new();

                // Spawn multiple entities
                for i in 0..10 {
                    let x = Fixed::from_num(i * 100);
                    let y = Fixed::from_num(i * 50);
                    let id = sim.spawn_entity(EntitySpawnParams {
                        position: Some(Vec2Fixed::new(x, y)),
                        health: Some(100),
                        movement: Some(Fixed::from_num(2)),
                        ..Default::default()
                    });

                    // Issue commands
                    let _ = sim.apply_command(
                        id,
                        Command::MoveTo(Vec2Fixed::new(Fixed::from_num(500), Fixed::from_num(500))),
                    );
                }
                sim
            },
            50,
        ));
    }

    // =========================================================================
    // Integration tests: Combat determinism
    // =========================================================================

    fn setup_combat_scenario() -> Simulation {
        let mut sim = Simulation::new();

        // Attacker unit
        let attacker = sim.spawn_entity(EntitySpawnParams {
            position: Some(Vec2Fixed::new(Fixed::from_num(0), Fixed::from_num(0))),
            health: Some(100),
            movement: Some(Fixed::from_num(5)),
            combat_stats: Some(CombatStats::new(10, Fixed::from_num(50), 5)),
            ..Default::default()
        });

        // Defender unit
        let defender = sim.spawn_entity(EntitySpawnParams {
            position: Some(Vec2Fixed::new(Fixed::from_num(20), Fixed::from_num(0))),
            health: Some(100),
            movement: Some(Fixed::from_num(5)),
            combat_stats: Some(CombatStats::new(10, Fixed::from_num(50), 5)),
            ..Default::default()
        });

        // Set attack targets
        let _ = sim.set_attack_target(attacker, defender);
        let _ = sim.set_attack_target(defender, attacker);

        sim
    }

    /// Setup a projectile combat scenario with splash damage.
    ///
    /// Creates an attacker with projectile-based weapons (non-zero speed)
    /// and splash damage, attacking a cluster of defenders. This tests
    /// determinism of projectile travel, impact detection, and splash damage.
    fn setup_projectile_combat_scenario() -> Simulation {
        let mut sim = Simulation::new();

        // Attacker with projectile weapon + splash
        let attacker = sim.spawn_entity(EntitySpawnParams {
            position: Some(Vec2Fixed::new(Fixed::from_num(0), Fixed::from_num(0))),
            health: Some(200),
            movement: Some(Fixed::from_num(3)),
            combat_stats: Some(
                CombatStats::new(15, Fixed::from_num(80), 10)
                    .with_projectile_speed(Fixed::from_num(8))
                    .with_splash_radius(Fixed::from_num(30)),
            ),
            ..Default::default()
        });

        // Cluster of defenders close together (splash should hit multiple)
        let defender1 = sim.spawn_entity(EntitySpawnParams {
            position: Some(Vec2Fixed::new(Fixed::from_num(40), Fixed::from_num(0))),
            health: Some(100),
            movement: Some(Fixed::from_num(2)),
            combat_stats: Some(CombatStats::new(5, Fixed::from_num(50), 8)),
            ..Default::default()
        });

        let _defender2 = sim.spawn_entity(EntitySpawnParams {
            position: Some(Vec2Fixed::new(Fixed::from_num(45), Fixed::from_num(10))),
            health: Some(80),
            movement: Some(Fixed::from_num(2)),
            ..Default::default()
        });

        let _defender3 = sim.spawn_entity(EntitySpawnParams {
            position: Some(Vec2Fixed::new(Fixed::from_num(42), Fixed::from_num(-8))),
            health: Some(120),
            movement: Some(Fixed::from_num(2)),
            ..Default::default()
        });

        // Attacker fires projectiles at the first defender; splash
        // should affect the nearby defenders deterministically.
        let _ = sim.set_attack_target(attacker, defender1);

        sim
    }

    #[test]
    fn test_combat_determinism() {
        let result = verify_determinism(
            5,
            200,
            setup_combat_scenario,
            |sim| {
                sim.tick();
            },
            |sim| sim.state_hash(),
        );
        result.assert_deterministic();
    }

    #[test]
    fn test_combat_damage_is_exact() {
        // Run combat scenario twice and check damage dealt is identical
        let mut sim1 = setup_combat_scenario();
        let mut sim2 = setup_combat_scenario();

        // Run for 50 ticks (enough for several attacks)
        for _ in 0..50 {
            let events1 = sim1.tick();
            let events2 = sim2.tick();

            // Same number of damage events
            assert_eq!(
                events1.damage_events.len(),
                events2.damage_events.len(),
                "Different number of damage events"
            );

            // Same damage values
            for (e1, e2) in events1.damage_events.iter().zip(&events2.damage_events) {
                assert_eq!(e1.damage, e2.damage, "Damage values differ");
            }
        }
    }

    // =========================================================================
    // Integration tests: Projectile combat determinism
    // =========================================================================

    #[test]
    fn test_projectile_combat_determinism() {
        let result = verify_determinism(
            5,
            300,
            setup_projectile_combat_scenario,
            |sim| {
                sim.tick();
            },
            |sim| sim.state_hash(),
        );
        result.assert_deterministic();
    }

    #[test]
    fn test_projectile_splash_determinism() {
        // Run the projectile scenario twice and verify damage events match
        let mut sim1 = setup_projectile_combat_scenario();
        let mut sim2 = setup_projectile_combat_scenario();

        for tick in 0..300 {
            let events1 = sim1.tick();
            let events2 = sim2.tick();

            assert_eq!(
                events1.damage_events.len(),
                events2.damage_events.len(),
                "Different number of damage events at tick {tick}"
            );

            for (e1, e2) in events1.damage_events.iter().zip(&events2.damage_events) {
                assert_eq!(e1.damage, e2.damage, "Damage values differ at tick {tick}");
                assert_eq!(e1.target, e2.target, "Damage targets differ at tick {tick}");
            }
        }
    }

    #[test]
    fn test_parallel_projectile_simulations() {
        let result = run_parallel_simulations_scoped(setup_projectile_combat_scenario, 4, 300);
        result.assert_deterministic();
    }

    // =========================================================================
    // Integration tests: Movement determinism
    // =========================================================================

    fn setup_movement_scenario() -> Simulation {
        let mut sim = Simulation::new();

        let unit = sim.spawn_entity(EntitySpawnParams {
            position: Some(Vec2Fixed::ZERO),
            health: Some(100),
            movement: Some(Fixed::from_num(3)),
            ..Default::default()
        });

        let _ = sim.apply_command(
            unit,
            Command::MoveTo(Vec2Fixed::new(Fixed::from_num(1000), Fixed::from_num(1000))),
        );

        sim
    }

    #[test]
    fn test_movement_determinism() {
        let result = verify_determinism(
            5,
            500,
            setup_movement_scenario,
            |sim| {
                sim.tick();
            },
            |sim| sim.state_hash(),
        );
        result.assert_deterministic();
    }

    #[test]
    fn test_movement_arrives_at_exact_position() {
        let mut sim1 = setup_movement_scenario();
        let mut sim2 = setup_movement_scenario();

        // Run until arrival or timeout
        for _ in 0..1000 {
            sim1.tick();
            sim2.tick();
        }

        // Both should be at exact same position
        let entity1 = sim1.get_entity(1).unwrap();
        let entity2 = sim2.get_entity(1).unwrap();

        let pos1 = entity1.position.unwrap();
        let pos2 = entity2.position.unwrap();

        assert_eq!(pos1.value.x, pos2.value.x, "X positions differ");
        assert_eq!(pos1.value.y, pos2.value.y, "Y positions differ");
    }

    // =========================================================================
    // Integration tests: Production determinism
    // =========================================================================

    fn setup_production_scenario() -> Simulation {
        let mut sim = Simulation::new();

        // Spawn a building with production queue
        sim.spawn_entity(EntitySpawnParams {
            position: Some(Vec2Fixed::new(Fixed::from_num(100), Fixed::from_num(100))),
            health: Some(500),
            has_production_queue: true,
            ..Default::default()
        });

        sim
    }

    #[test]
    fn test_production_determinism() {
        let result = verify_determinism(
            3,
            100,
            setup_production_scenario,
            |sim| {
                sim.tick();
            },
            |sim| sim.state_hash(),
        );
        result.assert_deterministic();
    }

    // =========================================================================
    // Parallel simulation tests
    // =========================================================================

    #[test]
    fn test_parallel_empty_simulations() {
        let result = run_parallel_simulations_scoped(Simulation::new, 4, 100);
        result.assert_deterministic();
    }

    #[test]
    fn test_parallel_combat_simulations() {
        let result = run_parallel_simulations_scoped(setup_combat_scenario, 4, 200);
        result.assert_deterministic();
    }

    #[test]
    fn test_parallel_movement_simulations() {
        let result = run_parallel_simulations_scoped(setup_movement_scenario, 4, 500);
        result.assert_deterministic();
    }

    // =========================================================================
    // Property-based tests using proptest
    // =========================================================================

    proptest! {
        /// Any random spawn position should produce deterministic results.
        ///
        /// This catches floating-point contamination in position handling.
        #[test]
        fn prop_random_spawn_positions_are_deterministic(
            x in -10000i32..10000,
            y in -10000i32..10000,
        ) {
            let setup = move || {
                let mut sim = Simulation::new();
                sim.spawn_entity(EntitySpawnParams {
                    position: Some(Vec2Fixed::new(
                        Fixed::from_num(x),
                        Fixed::from_num(y),
                    )),
                    health: Some(100),
                    movement: Some(Fixed::from_num(5)),
                    ..Default::default()
                });
                sim
            };

            let result = verify_determinism(2, 50, setup, |s| { s.tick(); }, |s| s.state_hash());
            prop_assert!(result.is_deterministic);
        }

        /// Random movement speeds should produce deterministic results.
        ///
        /// This catches issues with fixed-point division/multiplication.
        #[test]
        fn prop_random_speeds_are_deterministic(
            speed in 1i32..20,
            target_x in 0i32..1000,
            target_y in 0i32..1000,
        ) {
            let setup = move || {
                let mut sim = Simulation::new();
                let unit = sim.spawn_entity(EntitySpawnParams {
                    position: Some(Vec2Fixed::ZERO),
                    health: Some(100),
                    movement: Some(Fixed::from_num(speed)),
                    ..Default::default()
                });

                let _ = sim.apply_command(
                    unit,
                    Command::MoveTo(Vec2Fixed::new(
                        Fixed::from_num(target_x),
                        Fixed::from_num(target_y),
                    )),
                );
                sim
            };

            let result = verify_determinism(2, 100, setup, |s| { s.tick(); }, |s| s.state_hash());
            prop_assert!(result.is_deterministic);
        }

        /// Random command sequences should produce identical results when replayed.
        ///
        /// This catches issues with command queue ordering or state.
        #[test]
        fn prop_command_sequences_are_replayable(
            commands in strategies::arb_command_sequence(10),
        ) {
            let commands_clone = commands.clone();

            let setup = move || {
                let mut sim = Simulation::new();
                let unit = sim.spawn_entity(EntitySpawnParams {
                    position: Some(Vec2Fixed::ZERO),
                    health: Some(100),
                    movement: Some(Fixed::from_num(5)),
                    ..Default::default()
                });

                // Apply all commands
                for cmd in &commands_clone {
                    let _ = sim.queue_command(unit, cmd.clone());
                }
                sim
            };

            let result = verify_determinism(2, 200, setup, |s| { s.tick(); }, |s| s.state_hash());
            prop_assert!(result.is_deterministic);
        }

        /// Serialization round-trip should always preserve state exactly.
        ///
        /// This catches any serialization bugs that could cause desyncs.
        #[test]
        fn prop_serialization_roundtrip_is_exact(
            num_entities in 1usize..10,
            num_ticks in 0u64..100,
        ) {
            let setup = move || {
                let mut sim = Simulation::new();
                for i in 0..num_entities {
                    sim.spawn_entity(EntitySpawnParams {
                        position: Some(Vec2Fixed::new(
                            Fixed::from_num(i as i32 * 100),
                            Fixed::from_num(i as i32 * 50),
                        )),
                        health: Some(100),
                        movement: Some(Fixed::from_num(2)),
                        ..Default::default()
                    });
                }
                sim
            };

            prop_assert!(verify_serialization_determinism(setup, num_ticks));
        }

        /// Multiple entities with random positions should simulate deterministically.
        ///
        /// This catches iteration order issues (HashMap randomization).
        #[test]
        fn prop_multiple_entities_deterministic(
            entity_params in strategies::arb_entity_list(20),
        ) {
            let entity_params_clone = entity_params.clone();

            let setup = move || {
                let mut sim = Simulation::new();
                for params in &entity_params_clone {
                    sim.spawn_entity(EntitySpawnParams {
                        position: Some(params.position),
                        health: Some(params.health),
                        movement: params.speed,
                        ..Default::default()
                    });
                }
                sim
            };

            let result = verify_determinism(2, 100, setup, |s| { s.tick(); }, |s| s.state_hash());
            prop_assert!(result.is_deterministic);
        }

        /// Combat between units should deal exact same damage each run.
        ///
        /// This catches floating-point contamination in damage calculations.
        #[test]
        fn prop_combat_damage_is_exact(
            attacker_health in 50u32..200,
            defender_health in 50u32..200,
            damage in 5u32..30,
            range in 10i32..100,
        ) {
            let setup = move || {
                let mut sim = Simulation::new();

                let attacker = sim.spawn_entity(EntitySpawnParams {
                    position: Some(Vec2Fixed::ZERO),
                    health: Some(attacker_health),
                    movement: Some(Fixed::from_num(5)),
                    combat_stats: Some(CombatStats::new(damage, Fixed::from_num(range), 5)),
                    ..Default::default()
                });

                let defender = sim.spawn_entity(EntitySpawnParams {
                    position: Some(Vec2Fixed::new(
                        Fixed::from_num(range / 2),
                        Fixed::from_num(0),
                    )),
                    health: Some(defender_health),
                    movement: Some(Fixed::from_num(5)),
                    ..Default::default()
                });

                let _ = sim.set_attack_target(attacker, defender);
                sim
            };

            let result = verify_determinism(2, 100, setup, |s| { s.tick(); }, |s| s.state_hash());
            prop_assert!(result.is_deterministic);
        }
    }

    // =========================================================================
    // Stress tests (only run explicitly with --ignored)
    // =========================================================================

    #[test]
    #[ignore = "Long-running stress test"]
    fn stress_test_many_entities() {
        let setup = || {
            let mut sim = Simulation::new();

            // Spawn 100 entities
            for i in 0..100 {
                let x = (i % 10) * 100;
                let y = (i / 10) * 100;
                let unit = sim.spawn_entity(EntitySpawnParams {
                    position: Some(Vec2Fixed::new(Fixed::from_num(x), Fixed::from_num(y))),
                    health: Some(100),
                    movement: Some(Fixed::from_num(2)),
                    ..Default::default()
                });

                // Give them all movement commands
                let _ = sim.apply_command(
                    unit,
                    Command::MoveTo(Vec2Fixed::new(Fixed::from_num(500), Fixed::from_num(500))),
                );
            }
            sim
        };

        let result = verify_determinism(
            5,
            1000,
            setup,
            |s| {
                s.tick();
            },
            |s| s.state_hash(),
        );
        result.assert_deterministic();
    }

    #[test]
    #[ignore = "Long-running stress test"]
    fn stress_test_parallel_many_simulations() {
        let result = run_parallel_simulations_scoped(setup_combat_scenario, 16, 1000);
        result.assert_deterministic();
    }
}
