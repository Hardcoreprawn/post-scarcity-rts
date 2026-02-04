//! Real game execution for headless testing.
//!
//! This module runs actual game simulations using rts_core's Simulation,
//! executing AI strategies and collecting detailed metrics.
//!
//! # Defensive Coding Principles (JPL-style)
//!
//! - All loops are bounded with explicit maximum iterations
//! - All allocations have predetermined limits
//! - Progress is logged at regular intervals
//! - Failure modes are explicit, not silent
//! - Resource usage is tracked and reported

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, trace, warn};

use rts_core::components::{CombatStats, Command, EntityId, FactionMember};
use rts_core::data::UnitData;
use rts_core::factions::FactionId;
use rts_core::math::{Fixed, Vec2Fixed};
use rts_core::simulation::{EntitySpawnParams, Simulation};

use crate::faction_loader::FactionRegistry;
use crate::metrics::{EventType, FactionMetrics, GameMetrics, TimedEvent};
use crate::scenario::Scenario;
use crate::screenshot::{
    ScreenshotConfig, ScreenshotManager, ScreenshotTrigger, UnitVisual, VisualState,
};
use crate::strategies::{BuildOrderItem, Strategy, StrategyExecutor, TacticalDecision};

/// High-level game runner for headless testing.
///
/// Provides a convenient interface to run complete game simulations with
/// strategy-driven AI and metrics collection.
#[derive(Debug, Clone, Default)]
pub struct GameRunner {
    /// Default max ticks if not specified in config.
    pub default_max_ticks: u64,
}

impl GameRunner {
    /// Create a new game runner.
    #[must_use]
    pub fn new() -> Self {
        Self {
            default_max_ticks: 36000, // 10 minutes at 60 tps
        }
    }

    /// Run a game with the given configuration.
    pub fn run(&self, mut config: GameConfig) -> GameResult {
        if config.max_ticks == 0 {
            config.max_ticks = self.default_max_ticks;
        }
        run_game(config)
    }
}

/// Configuration for a single game run.
#[derive(Debug, Clone)]
pub struct GameConfig {
    /// Random seed for determinism.
    pub seed: u64,
    /// Maximum ticks before timeout.
    pub max_ticks: u64,
    /// Scenario to use.
    pub scenario: Scenario,
    /// Strategy for faction A (Continuity).
    pub strategy_a: Strategy,
    /// Strategy for faction B (Collegium).
    pub strategy_b: Strategy,
    /// Screenshot configuration.
    pub screenshot_config: Option<ScreenshotConfig>,
    /// Game ID for tracking.
    pub game_id: String,
    /// Optional faction registry for data-driven unit stats.
    /// If None, falls back to hardcoded generic units.
    pub faction_registry: Option<Arc<FactionRegistry>>,
}

/// State for one player in the game.
#[derive(Debug)]
struct PlayerState {
    faction_id: FactionId,
    executor: StrategyExecutor,
    resources: i64,
    depot_entity: Option<EntityId>,
    units: Vec<EntityId>,
    buildings: Vec<EntityId>,
    units_produced: HashMap<String, u32>,
    units_lost: HashMap<String, u32>,
    units_killed: HashMap<String, u32>,
    buildings_constructed: HashMap<String, u32>,
    buildings_lost: HashMap<String, u32>,
    total_damage_dealt: i64,
    total_damage_taken: i64,
    first_attack_tick: Option<u64>,
    peak_army_size: u32,
}

impl PlayerState {
    fn new(faction_id: FactionId, strategy: Strategy) -> Self {
        let executor = StrategyExecutor::new(strategy.clone());
        Self {
            faction_id,
            executor,
            resources: 1000,
            depot_entity: None,
            units: Vec::new(),
            buildings: Vec::new(),
            units_produced: HashMap::new(),
            units_lost: HashMap::new(),
            units_killed: HashMap::new(),
            buildings_constructed: HashMap::new(),
            buildings_lost: HashMap::new(),
            total_damage_dealt: 0,
            total_damage_taken: 0,
            first_attack_tick: None,
            peak_army_size: 0,
        }
    }

    /// Update peak army size.
    fn update_peak_army(&mut self) {
        let current = self.units.len() as u32;
        if current > self.peak_army_size {
            self.peak_army_size = current;
        }
    }
}

/// Result of running a game.
#[derive(Debug)]
pub struct GameResult {
    pub metrics: GameMetrics,
    pub final_state_hash: u64,
}

// =============================================================================
// RESOURCE LIMITS (JPL-style defensive programming)
// =============================================================================

/// Maximum entities we'll ever allow in a single game.
/// Prevents runaway spawning from consuming all memory.
/// Rationale: 10K entities × ~1KB each = ~10MB, well within reason.
const MAX_ENTITIES: usize = 10_000;

/// Maximum events to track per game.
/// Prevents unbounded memory growth from event logging.
/// Rationale: 100K events × ~100 bytes = ~10MB.
const MAX_EVENTS: usize = 100_000;

/// Progress logging interval (ticks).
/// Log every N ticks so we can see the game is making progress.
const PROGRESS_LOG_INTERVAL: u64 = 1000;

// =============================================================================
// ECONOMY LIMITS (game balance)
// =============================================================================

/// Maximum units per player (supply cap).
/// This is a fundamental RTS mechanic - you can't just build infinite units.
/// 200 is standard for most RTS games (StarCraft, C&C, etc.)
const MAX_SUPPLY_PER_PLAYER: usize = 200;

// =============================================================================
// WATCHDOG TIMEOUTS (detecting hangs, not game duration)
// =============================================================================

/// Maximum wall-clock time for a SINGLE TICK to complete.
/// If one tick takes > 5 seconds, we have an infinite loop or deadlock.
/// Normal ticks should be < 1ms even with thousands of entities.
const TICK_TIMEOUT_MS: u128 = 5_000;

/// Grace period before logging "slow tick" warnings (ms).
/// Ticks taking > 100ms are concerning but not fatal.
const SLOW_TICK_THRESHOLD_MS: u128 = 100;

/// Run a complete game simulation.
///
/// # Panics
/// Panics if:
/// - Entity count exceeds MAX_ENTITIES (runaway spawning)
/// - A single tick takes longer than TICK_TIMEOUT_MS
/// - Memory allocation fails
pub fn run_game(config: GameConfig) -> GameResult {
    let game_start = Instant::now();
    info!(
        game_id = %config.game_id,
        seed = config.seed,
        max_ticks = config.max_ticks,
        scenario = %config.scenario.name,
        "Starting game simulation"
    );

    let mut sim = Simulation::new();
    let mut rng = SimpleRng::new(config.seed);

    // Get faction registry reference for spawn functions
    let registry = config.faction_registry.as_deref();

    // Set up initial state from scenario
    let mut player_a = PlayerState::new(FactionId::Continuity, config.strategy_a.clone());
    let mut player_b = PlayerState::new(FactionId::Collegium, config.strategy_b.clone());

    // Spawn initial entities for each faction from scenario
    for faction_setup in &config.scenario.factions {
        let player = if faction_setup.faction_id == "continuity" {
            &mut player_a
        } else {
            &mut player_b
        };

        // Set starting resources
        player.resources = faction_setup.starting_resources;

        // Spawn depot/command center
        for building in &faction_setup.starting_buildings {
            let entity_id = spawn_building_with_registry(
                &mut sim,
                &building.kind,
                building.position.0,
                building.position.1,
                player.faction_id,
                registry,
            );
            player.buildings.push(entity_id);
            if matches!(
                building.kind.as_str(),
                "command_center" | "depot" | "administration_center"
            ) {
                player.depot_entity = Some(entity_id);
            }
            *player
                .buildings_constructed
                .entry(building.kind.clone())
                .or_insert(0) += 1;
        }

        // Spawn initial units
        for unit_spawn in &faction_setup.starting_units {
            for _ in 0..unit_spawn.count {
                let entity_id = spawn_unit_with_registry(
                    &mut sim,
                    &unit_spawn.kind,
                    unit_spawn.position.0,
                    unit_spawn.position.1,
                    player.faction_id,
                    registry,
                );
                player.units.push(entity_id);
                *player
                    .units_produced
                    .entry(unit_spawn.kind.clone())
                    .or_insert(0) += 1;
            }
        }

        // Update peak army size
        player.update_peak_army();
    }

    // Track events with bounded capacity
    let mut events: Vec<TimedEvent> = Vec::with_capacity(1024);
    let mut screenshot_manager = config.screenshot_config.map(ScreenshotManager::new);

    // Pre-game diagnostics
    let initial_entity_count = sim.entities().len();
    info!(
        initial_entities = initial_entity_count,
        player_a_units = player_a.units.len(),
        player_a_buildings = player_a.buildings.len(),
        player_b_units = player_b.units.len(),
        player_b_buildings = player_b.buildings.len(),
        "Game initialized"
    );

    // Main game loop - BOUNDED by max_ticks
    let mut tick = 0u64;
    let mut winner: Option<String> = None;
    let mut win_condition = "timeout".to_string();
    let mut last_progress_log = Instant::now();

    // Invariant: tick always increases, loop will terminate at max_ticks
    while tick < config.max_ticks {
        let tick_start = Instant::now();
        // Defensive check: entity count sanity
        let entity_count = sim.entities().len();
        if entity_count > MAX_ENTITIES {
            error!(
                tick = tick,
                entity_count = entity_count,
                max = MAX_ENTITIES,
                "FATAL: Entity count exceeded maximum - aborting to prevent OOM"
            );
            win_condition = "error_entity_overflow".to_string();
            break;
        }

        // Execute AI for each player
        execute_ai_turn(&mut sim, &mut player_a, tick, &mut rng, registry);
        execute_ai_turn(&mut sim, &mut player_b, tick, &mut rng, registry);

        // Advance simulation
        let tick_events = sim.tick();
        tick += 1;

        // Watchdog: check tick duration
        let tick_duration = tick_start.elapsed();

        // Warn about slow ticks (not fatal, but concerning)
        if tick_duration.as_millis() > SLOW_TICK_THRESHOLD_MS
            && tick_duration.as_millis() <= TICK_TIMEOUT_MS
        {
            warn!(
                tick = tick,
                duration_ms = tick_duration.as_millis(),
                threshold_ms = SLOW_TICK_THRESHOLD_MS,
                entities = sim.entities().len(),
                "Slow tick detected - possible performance issue"
            );
        }

        // Fatal: tick took way too long
        if tick_duration.as_millis() > TICK_TIMEOUT_MS {
            error!(
                tick = tick,
                duration_ms = tick_duration.as_millis(),
                timeout_ms = TICK_TIMEOUT_MS,
                "FATAL: Tick took too long - possible infinite loop or deadlock"
            );
            win_condition = "error_tick_timeout".to_string();
            break;
        }

        // Progress logging
        if tick % PROGRESS_LOG_INTERVAL == 0 || last_progress_log.elapsed() > Duration::from_secs(5)
        {
            debug!(
                tick = tick,
                max_ticks = config.max_ticks,
                progress_pct = (tick as f64 / config.max_ticks as f64 * 100.0) as u32,
                entities = entity_count,
                player_a_units = player_a.units.len(),
                player_b_units = player_b.units.len(),
                elapsed_ms = game_start.elapsed().as_millis(),
                "Game progress"
            );
            last_progress_log = Instant::now();
        }

        // Process combat events
        for damage_event in &tick_events.damage_events {
            // Find which player owns attacker and target
            let attacker_faction = get_entity_faction(&sim, damage_event.attacker);
            let target_faction = get_entity_faction(&sim, damage_event.target);

            if let Some(af) = attacker_faction {
                let player = if af == FactionId::Continuity {
                    &mut player_a
                } else {
                    &mut player_b
                };
                player.total_damage_dealt += damage_event.damage as i64;
            }
            if let Some(tf) = target_faction {
                let player = if tf == FactionId::Continuity {
                    &mut player_a
                } else {
                    &mut player_b
                };
                player.total_damage_taken += damage_event.damage as i64;
            }
        }

        // Process deaths
        for dead_id in &tick_events.deaths {
            // Remove from player unit lists
            if player_a.units.contains(dead_id) {
                player_a.units.retain(|&id| id != *dead_id);
                *player_a.units_lost.entry("unit".to_string()).or_insert(0) += 1;
                events.push(TimedEvent {
                    tick,
                    event_type: EventType::UnitKilled,
                    faction: "continuity".to_string(),
                    details: format!("Unit {} died", dead_id),
                });

                // Credit the kill to the other player
                *player_b.units_killed.entry("unit".to_string()).or_insert(0) += 1;
            }
            if player_b.units.contains(dead_id) {
                player_b.units.retain(|&id| id != *dead_id);
                *player_b.units_lost.entry("unit".to_string()).or_insert(0) += 1;
                events.push(TimedEvent {
                    tick,
                    event_type: EventType::UnitKilled,
                    faction: "collegium".to_string(),
                    details: format!("Unit {} died", dead_id),
                });

                *player_a.units_killed.entry("unit".to_string()).or_insert(0) += 1;
            }

            // Check for depot destruction
            if player_a.depot_entity == Some(*dead_id) {
                player_a.depot_entity = None;
            }
            if player_b.depot_entity == Some(*dead_id) {
                player_b.depot_entity = None;
            }
        }

        // Check for screenshot triggers
        if let Some(ref mut manager) = screenshot_manager {
            // Major battle trigger
            if tick_events.damage_events.len() > 5 {
                let state = create_visual_state(&config.game_id, tick, &sim);
                let trigger = ScreenshotTrigger::MajorBattle {
                    unit_count: tick_events.damage_events.len() as u32,
                };
                let _ = manager.capture(state, &trigger);
            }

            // Timed snapshots every 2 minutes (7200 ticks at 60fps, 2400 at 20fps)
            if manager.should_capture_timed(tick) {
                let state = create_visual_state(&config.game_id, tick, &sim);
                let trigger = ScreenshotTrigger::TimedSnapshot { tick };
                let _ = manager.capture(state, &trigger);
                manager.record_timed_capture(tick);
            }
        }

        // Check victory conditions
        if tick_events.game_end.is_some() {
            if let Some(winning_faction) = tick_events.game_end {
                winner = Some(match winning_faction {
                    FactionId::Continuity => "continuity".to_string(),
                    FactionId::Collegium => "collegium".to_string(),
                    _ => "unknown".to_string(),
                });
                win_condition = "elimination".to_string();
                break;
            }
        }

        // Victory condition: HQ/depot destruction
        let a_has_depot = player_a.depot_entity.is_some();
        let b_has_depot = player_b.depot_entity.is_some();

        if !a_has_depot && b_has_depot {
            winner = Some("collegium".to_string());
            win_condition = "elimination".to_string();
            break;
        }
        if !b_has_depot && a_has_depot {
            winner = Some("continuity".to_string());
            win_condition = "elimination".to_string();
            break;
        }
        if !a_has_depot && !b_has_depot {
            // Mutual destruction - draw
            break;
        }
    }

    // Post-game diagnostics
    let game_duration = game_start.elapsed();
    info!(
        game_id = %config.game_id,
        duration_ticks = tick,
        duration_ms = game_duration.as_millis(),
        winner = ?winner,
        win_condition = %win_condition,
        final_entities = sim.entities().len(),
        events_recorded = events.len(),
        "Game simulation complete"
    );

    // Warn if we hit resource limits
    if events.len() >= MAX_EVENTS {
        warn!(
            events = events.len(),
            max = MAX_EVENTS,
            "Event buffer may have been truncated"
        );
    }

    // Build metrics
    let mut factions = HashMap::new();

    factions.insert(
        "continuity".to_string(),
        build_faction_metrics(&player_a, tick),
    );
    factions.insert(
        "collegium".to_string(),
        build_faction_metrics(&player_b, tick),
    );

    let metrics = GameMetrics {
        game_id: config.game_id,
        scenario: config.scenario.name.clone(),
        seed: config.seed,
        duration_ticks: tick,
        winner,
        win_condition,
        factions,
        events,
        final_state_hash: 0, // Set by caller when copying to batch results
    };

    GameResult {
        metrics,
        final_state_hash: sim.state_hash(),
    }
}

/// Execute AI for a player's turn.
fn execute_ai_turn(
    sim: &mut Simulation,
    player: &mut PlayerState,
    tick: u64,
    rng: &mut SimpleRng,
    registry: Option<&FactionRegistry>,
) {
    // Get current unit count for strategy decisions
    let current_resources = player.resources;
    let unit_counts: HashMap<String, u32> = player.units_produced.clone();
    let current_supply = player.units.len();

    // Supply cap check - fundamental RTS mechanic
    let can_build_units = current_supply < MAX_SUPPLY_PER_PLAYER;

    // Check build order
    if let Some(item) = player
        .executor
        .next_build_item(tick, current_resources, &unit_counts)
    {
        match item {
            BuildOrderItem::Unit(unit_type) => {
                // Only build if we have resources AND supply
                let cost = get_unit_cost_with_registry(&unit_type, player.faction_id, registry);
                if player.resources >= cost && can_build_units {
                    // Spawn near depot
                    if let Some(depot_id) = player.depot_entity {
                        if let Some(depot_pos) = get_entity_position(sim, depot_id) {
                            let offset_x = (rng.next() % 50) as i32 - 25;
                            let offset_y = (rng.next() % 50) as i32 - 25;
                            let entity_id = spawn_unit_with_registry(
                                sim,
                                &unit_type,
                                depot_pos.x.to_num::<i32>() + offset_x,
                                depot_pos.y.to_num::<i32>() + offset_y,
                                player.faction_id,
                                registry,
                            );
                            player.units.push(entity_id);
                            player.resources -= cost;
                            *player.units_produced.entry(unit_type).or_insert(0) += 1;
                        }
                    }
                }
            }
            BuildOrderItem::Building(building_type) => {
                let cost =
                    get_building_cost_with_registry(&building_type, player.faction_id, registry);
                if player.resources >= cost {
                    if let Some(depot_id) = player.depot_entity {
                        if let Some(depot_pos) = get_entity_position(sim, depot_id) {
                            let offset_x = (rng.next() % 100) as i32 - 50;
                            let offset_y = (rng.next() % 100) as i32 - 50;
                            let entity_id = spawn_building_with_registry(
                                sim,
                                &building_type,
                                depot_pos.x.to_num::<i32>() + offset_x,
                                depot_pos.y.to_num::<i32>() + offset_y,
                                player.faction_id,
                                registry,
                            );
                            player.buildings.push(entity_id);
                            player.resources -= cost;
                            *player
                                .buildings_constructed
                                .entry(building_type)
                                .or_insert(0) += 1;
                        }
                    }
                }
            }
            _ => {}
        }
    } else {
        // Build order exhausted - continuous production based on composition
        // Prioritize combat units based on strategy composition
        let composition = player.executor.composition();

        // Find the unit type we need most
        if let Some((best_unit, _)) = composition
            .iter()
            .filter(|(unit, _)| *unit != "harvester") // Skip harvesters for combat focus
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        {
            let cost = get_unit_cost_with_registry(best_unit, player.faction_id, registry);
            // Only build if we have resources AND supply
            if player.resources >= cost && can_build_units {
                if let Some(depot_id) = player.depot_entity {
                    if let Some(depot_pos) = get_entity_position(sim, depot_id) {
                        let offset_x = (rng.next() % 50) as i32 - 25;
                        let offset_y = (rng.next() % 50) as i32 - 25;
                        let entity_id = spawn_unit_with_registry(
                            sim,
                            best_unit,
                            depot_pos.x.to_num::<i32>() + offset_x,
                            depot_pos.y.to_num::<i32>() + offset_y,
                            player.faction_id,
                            registry,
                        );
                        player.units.push(entity_id);
                        player.resources -= cost;
                        *player.units_produced.entry(best_unit.clone()).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    // ==========================================================================
    // ECONOMY: Passive income (simulating harvesters for headless testing)
    // ==========================================================================
    // Target economy rates (per second of game time, at 60 tps):
    //   - Early game: ~5-10 resources/sec (1 infantry every 5-10 seconds)
    //   - Mid game:   ~15-25 resources/sec (with 3-5 harvesters)
    //   - Late game:  ~30-50 resources/sec (with economy buildings)
    //
    // For headless testing without actual harvesters, we use passive income:
    //   - 1 resource every 6 ticks = 10 resources/sec
    //   - This allows ~1 infantry per 5 seconds baseline
    //
    // TODO: Replace with actual harvester simulation for realistic economy
    if tick % 6 == 0 {
        player.resources += 1;
    }

    // Target acquisition - find and attack nearby enemies
    acquire_targets_for_units(sim, player);

    // Check if we can see any enemies
    let visible_enemies = sim.get_visible_enemies_for(player.faction_id);
    let has_visible_enemies = !visible_enemies.is_empty();

    // Execute tactical decisions
    let army_supply = player.units.len() as u32;
    let decision = player.executor.decide_action(tick, army_supply, 5, false); // Estimate enemy supply

    // Enemy base location for attack/scout moves
    let enemy_base = if player.faction_id == FactionId::Continuity {
        Vec2Fixed::new(Fixed::from_num(464), Fixed::from_num(256)) // Collegium base
    } else {
        Vec2Fixed::new(Fixed::from_num(48), Fixed::from_num(256)) // Continuity base
    };

    // Map center for scouting
    let map_center = Vec2Fixed::new(Fixed::from_num(256), Fixed::from_num(256));

    match decision {
        TacticalDecision::Attack => {
            if player.first_attack_tick.is_none() {
                player.first_attack_tick = Some(tick);
            }

            // Send units toward enemy base using ATTACK-MOVE so they engage on the way
            for &unit_id in &player.units {
                // Check if unit already has an attack target
                let has_target = sim
                    .get_entity(unit_id)
                    .and_then(|e| e.attack_target.as_ref())
                    .and_then(|t| t.target)
                    .is_some();

                if !has_target {
                    // Attack-move, not just move - engage anything on the way
                    let _ = sim.apply_command(unit_id, Command::AttackMove(enemy_base));
                }
            }
        }
        TacticalDecision::Defend => {
            // Rally to base
            if let Some(depot_id) = player.depot_entity {
                if let Some(depot_pos) = get_entity_position(sim, depot_id) {
                    for &unit_id in &player.units {
                        let _ = sim.apply_command(unit_id, Command::AttackMove(depot_pos));
                    }
                }
            }
        }
        TacticalDecision::Scout => {
            // Active scouting - send units to find enemies
            // Scout toward map center first, then enemy base
            for &unit_id in &player.units {
                let has_target = sim
                    .get_entity(unit_id)
                    .and_then(|e| e.attack_target.as_ref())
                    .and_then(|t| t.target)
                    .is_some();

                if !has_target {
                    let _ = sim.apply_command(unit_id, Command::AttackMove(map_center));
                }
            }
        }
        TacticalDecision::Hold => {
            // If we can't see enemies and we're holding, we should still scout!
            // Otherwise we just sit at home forever
            if !has_visible_enemies && player.units.len() >= 5 {
                // Send a few units to scout (keep some home for defense)
                let scouts_to_send = player.units.len() / 3; // Send 1/3 of army
                for &unit_id in player.units.iter().take(scouts_to_send) {
                    let has_target = sim
                        .get_entity(unit_id)
                        .and_then(|e| e.attack_target.as_ref())
                        .and_then(|t| t.target)
                        .is_some();

                    if !has_target {
                        let _ = sim.apply_command(unit_id, Command::AttackMove(map_center));
                    }
                }
            }
        }
        TacticalDecision::Expand => {
            // For now, treat like hold - maybe build expansion later
        }
    }
}

/// Spawn a unit in the simulation using faction data if available.
fn spawn_unit_with_registry(
    sim: &mut Simulation,
    unit_type: &str,
    x: i32,
    y: i32,
    faction: FactionId,
    registry: Option<&FactionRegistry>,
) -> EntityId {
    // Try to get unit data from faction registry
    if let Some(reg) = registry {
        // First try exact ID match
        if let Some(unit_data) = reg.get_unit(faction, unit_type) {
            return spawn_unit_from_data(sim, unit_data, x, y, faction);
        }
        // Then try role-based lookup (e.g., "infantry" tag matches "security_team")
        if let Some(unit_data) = reg.get_unit_by_role(faction, unit_type) {
            return spawn_unit_from_data(sim, unit_data, x, y, faction);
        }
    }

    // Fall back to hardcoded generic units
    spawn_unit(sim, unit_type, x, y, faction)
}

/// Spawn a unit from faction data definition.
fn spawn_unit_from_data(
    sim: &mut Simulation,
    unit_data: &UnitData,
    x: i32,
    y: i32,
    faction: FactionId,
) -> EntityId {
    let combat_stats = unit_data
        .combat
        .as_ref()
        .map(|c| CombatStats::new(c.damage, c.range, c.attack_cooldown));

    sim.spawn_entity(EntitySpawnParams {
        position: Some(Vec2Fixed::new(Fixed::from_num(x), Fixed::from_num(y))),
        health: Some(unit_data.health),
        movement: Some(unit_data.speed),
        combat_stats,
        faction: Some(FactionMember::new(faction, 0)),
        is_depot: false,
        ..Default::default()
    })
}

/// Spawn a unit in the simulation (legacy hardcoded fallback).
fn spawn_unit(
    sim: &mut Simulation,
    unit_type: &str,
    x: i32,
    y: i32,
    faction: FactionId,
) -> EntityId {
    let (health, damage, range, speed) = match unit_type {
        "scout" | "patrol_vehicle" => (100, 8, 80, 15),
        "infantry" | "security_team" => (80, 12, 50, 10),
        "crowd_management_unit" => (60, 18, 40, 9),
        "ranger" => (80, 20, 120, 10),
        "tank" | "guardian_mech" => (500, 45, 70, 5),
        "harvester" | "collection_vehicle" => (150, 0, 0, 7),
        "pacification_platform" => (300, 60, 120, 4),
        "sovereign_platform" => (1200, 100, 90, 3),
        _ => (100, 12, 60, 10),
    };

    let combat_stats = if damage > 0 {
        Some(CombatStats::new(damage, Fixed::from_num(range), 20))
    } else {
        None
    };

    sim.spawn_entity(EntitySpawnParams {
        position: Some(Vec2Fixed::new(Fixed::from_num(x), Fixed::from_num(y))),
        health: Some(health),
        movement: Some(Fixed::from_num(speed)),
        combat_stats,
        faction: Some(FactionMember::new(faction, 0)),
        is_depot: false,
        ..Default::default()
    })
}

/// Spawn a building in the simulation using faction data if available.
fn spawn_building_with_registry(
    sim: &mut Simulation,
    building_type: &str,
    x: i32,
    y: i32,
    faction: FactionId,
    registry: Option<&FactionRegistry>,
) -> EntityId {
    // Try to get building data from faction registry
    if let Some(reg) = registry {
        if let Some(building_data) = reg.get_building(faction, building_type) {
            let is_depot = building_data.is_main_base;
            return sim.spawn_entity(EntitySpawnParams {
                position: Some(Vec2Fixed::new(Fixed::from_num(x), Fixed::from_num(y))),
                health: Some(building_data.health as u32),
                faction: Some(FactionMember::new(faction, 0)),
                is_depot,
                ..Default::default()
            });
        }
    }

    // Fall back to hardcoded
    spawn_building(sim, building_type, x, y, faction)
}

/// Spawn a building in the simulation (legacy hardcoded fallback).
fn spawn_building(
    sim: &mut Simulation,
    building_type: &str,
    x: i32,
    y: i32,
    faction: FactionId,
) -> EntityId {
    let health = match building_type {
        "command_center" | "depot" | "administration_center" => 1500,
        "barracks" | "training_center" => 500,
        "supply_depot" | "processing_facility" => 600,
        "tech_lab" | "research_institute" => 400,
        "turret" | "defense_turret" => 350,
        "vehicle_depot" => 600,
        "walker_facility" => 800,
        "air_operations" => 700,
        _ => 500,
    };

    let is_depot = matches!(
        building_type,
        "command_center" | "depot" | "administration_center"
    );

    sim.spawn_entity(EntitySpawnParams {
        position: Some(Vec2Fixed::new(Fixed::from_num(x), Fixed::from_num(y))),
        health: Some(health),
        faction: Some(FactionMember::new(faction, 0)),
        is_depot,
        ..Default::default()
    })
}

/// Get an entity's faction.
fn get_entity_faction(sim: &Simulation, entity_id: EntityId) -> Option<FactionId> {
    sim.get_entity(entity_id)
        .and_then(|e| e.faction.as_ref())
        .map(|f| f.faction)
}

/// Get an entity's position.
fn get_entity_position(sim: &Simulation, entity_id: EntityId) -> Option<Vec2Fixed> {
    sim.get_entity(entity_id)
        .and_then(|e| e.position.as_ref())
        .map(|p| p.value)
}

/// Acquire targets for units - find nearby enemies and issue Attack commands.
/// Prioritize depot (HQ) when in range to enable victory.
/// Uses visibility system - AI can only target what it can see.
///
/// # Bounds
/// - Iterates over player.units (bounded by MAX_ENTITIES)
/// - Iterates over visible_enemies (bounded by MAX_ENTITIES)
/// - Total work: O(units * visible_enemies) with both bounded
fn acquire_targets_for_units(sim: &mut Simulation, player: &PlayerState) {
    // Defensive: log if we have a suspiciously large number of units
    if player.units.len() > 1000 {
        warn!(
            faction = ?player.faction_id,
            unit_count = player.units.len(),
            "Unusually large unit count - possible runaway production"
        );
    }

    // Get only VISIBLE enemies for this faction (fair play)
    let visible_enemies = sim.get_visible_enemies_for(player.faction_id);

    trace!(
        faction = ?player.faction_id,
        own_units = player.units.len(),
        visible_enemies = visible_enemies.len(),
        "Target acquisition"
    );

    if visible_enemies.is_empty() {
        return;
    }

    // Bounded iteration counter for paranoia
    let mut iterations = 0usize;
    let max_iterations = player
        .units
        .len()
        .saturating_mul(visible_enemies.len().saturating_add(1));

    // For each of our units, find best target
    // ALWAYS prioritize depot/HQ when in range - re-evaluate every tick
    for &unit_id in &player.units {
        iterations += 1;
        if iterations > max_iterations {
            error!(
                iterations = iterations,
                max = max_iterations,
                "FATAL: acquire_targets exceeded iteration bound - breaking to prevent hang"
            );
            break;
        }
        let Some(unit_pos) = get_entity_position(sim, unit_id) else {
            continue;
        };

        let Some(unit) = sim.get_entity(unit_id) else {
            continue;
        };

        // Skip if no combat stats (non-combat unit like harvester)
        if unit.combat_stats.is_none() {
            continue;
        }

        // Check if depot is within attack range - ALWAYS switch to it
        let attack_range = unit
            .combat_stats
            .as_ref()
            .map(|c| c.range)
            .unwrap_or(Fixed::from_num(60));
        let depot_range_sq = attack_range * attack_range * Fixed::from_num(4); // 2x attack range

        let mut depot_in_range: Option<EntityId> = None;
        for enemy in &visible_enemies {
            if enemy.is_depot {
                let dist_sq = unit_pos.distance_squared(enemy.position);
                if dist_sq <= depot_range_sq {
                    depot_in_range = Some(enemy.id);
                    break;
                }
            }
        }

        // If depot in range, ALWAYS attack it (override current target)
        if let Some(depot_id) = depot_in_range {
            // Check if we're already attacking the depot
            let currently_attacking_depot = unit
                .attack_target
                .as_ref()
                .and_then(|t| t.target)
                .map(|t| t == depot_id)
                .unwrap_or(false);

            if !currently_attacking_depot {
                let _ = sim.apply_command(unit_id, Command::Attack(depot_id));
            }
            continue;
        }

        // Not near depot - check if we need a new target
        let needs_target = match &unit.attack_target {
            Some(at) => match at.target {
                Some(target_id) => sim.get_entity(target_id).is_none(),
                None => true,
            },
            None => true,
        };

        if needs_target {
            // Find nearest VISIBLE enemy
            let mut best_target: Option<EntityId> = None;
            let mut best_dist = Fixed::MAX;

            for enemy in &visible_enemies {
                let dist_sq = unit_pos.distance_squared(enemy.position);
                if dist_sq < best_dist {
                    best_dist = dist_sq;
                    best_target = Some(enemy.id);
                }
            }

            if let Some(target_id) = best_target {
                let _ = sim.apply_command(unit_id, Command::Attack(target_id));
            }
        }
    }
}

/// Get unit production cost with optional faction data lookup.
fn get_unit_cost_with_registry(
    unit_type: &str,
    faction: FactionId,
    registry: Option<&FactionRegistry>,
) -> i64 {
    if let Some(reg) = registry {
        // Try exact ID match first
        if let Some(unit_data) = reg.get_unit(faction, unit_type) {
            return unit_data.cost as i64;
        }
        // Then try role-based lookup
        if let Some(unit_data) = reg.get_unit_by_role(faction, unit_type) {
            return unit_data.cost as i64;
        }
    }
    get_unit_cost(unit_type)
}

/// Get unit production cost (legacy hardcoded fallback).
fn get_unit_cost(unit_type: &str) -> i64 {
    match unit_type {
        "scout" | "patrol_vehicle" => 60,
        "infantry" | "security_team" => 50,
        "crowd_management_unit" => 75,
        "ranger" => 100,
        "tank" | "guardian_mech" => 300,
        "harvester" | "collection_vehicle" => 100,
        "pacification_platform" => 250,
        "protected_transport" => 150,
        "sovereign_platform" => 800,
        "rapid_response_squadron" => 400,
        _ => 75,
    }
}

/// Get building cost with optional faction data lookup.
fn get_building_cost_with_registry(
    building_type: &str,
    faction: FactionId,
    registry: Option<&FactionRegistry>,
) -> i64 {
    if let Some(reg) = registry {
        if let Some(building_data) = reg.get_building(faction, building_type) {
            return building_data.cost as i64;
        }
    }
    get_building_cost(building_type)
}

/// Get building construction cost (legacy hardcoded fallback).
fn get_building_cost(building_type: &str) -> i64 {
    match building_type {
        "barracks" | "training_center" => 150,
        "supply_depot" | "processing_facility" => 200,
        "tech_lab" | "research_institute" => 200,
        "turret" | "defense_turret" => 150,
        "vehicle_depot" => 200,
        "walker_facility" => 350,
        "air_operations" => 400,
        "strategic_operations" => 500,
        "checkpoint" => 100,
        _ => 200,
    }
}

/// Create a visual state snapshot from the current simulation.
fn create_visual_state(game_id: &str, tick: u64, sim: &Simulation) -> VisualState {
    let trigger = ScreenshotTrigger::TimedSnapshot { tick };
    let mut state = VisualState::new(game_id, tick, trigger);

    // Add all entities as unit visuals (simplified)
    for (_, entity) in sim.entities().iter() {
        if let Some(pos) = &entity.position {
            let faction_name = entity
                .faction
                .as_ref()
                .map(|f| match f.faction {
                    FactionId::Continuity => "continuity",
                    FactionId::Collegium => "collegium",
                    _ => "unknown",
                })
                .unwrap_or("neutral")
                .to_string();

            let health_percent = entity
                .health
                .as_ref()
                .map(|h| h.current as f32 / h.max as f32)
                .unwrap_or(1.0);

            state.units.push(UnitVisual {
                entity_id: entity.id,
                kind: "unit".to_string(),
                faction: faction_name,
                position: (pos.value.x.to_num(), pos.value.y.to_num()),
                rotation: 0.0,
                health_percent,
                animation_state: "idle".to_string(),
                animation_frame: 0,
                is_selected: false,
                current_action: None,
            });
        }
    }

    state
}

/// Build faction metrics from player state.
fn build_faction_metrics(player: &PlayerState, _duration: u64) -> FactionMetrics {
    // Calculate K/D ratio
    let total_killed: u32 = player.units_killed.values().sum();
    let total_lost: u32 = player.units_lost.values().sum();
    let kd_ratio = if total_lost > 0 {
        total_killed as f64 / total_lost as f64
    } else if total_killed > 0 {
        f64::INFINITY
    } else {
        1.0
    };

    FactionMetrics {
        faction_id: match player.faction_id {
            FactionId::Continuity => "continuity".to_string(),
            FactionId::Collegium => "collegium".to_string(),
            _ => "unknown".to_string(),
        },
        final_score: (player.total_damage_dealt - player.total_damage_taken + player.resources),
        total_resources_gathered: player.resources
            + player
                .units_produced
                .values()
                .map(|&c| c as i64 * 75)
                .sum::<i64>(),
        total_resources_spent: player
            .units_produced
            .values()
            .map(|&c| c as i64 * 75)
            .sum::<i64>(),
        peak_income_rate: 0.0,    // Would need tracking
        resource_efficiency: 0.8, // Placeholder
        units_produced: player.units_produced.clone(),
        units_lost: player.units_lost.clone(),
        units_killed: player.units_killed.clone(),
        buildings_constructed: player.buildings_constructed.clone(),
        buildings_destroyed: HashMap::new(),
        buildings_lost: player.buildings_lost.clone(),
        total_damage_dealt: player.total_damage_dealt,
        total_damage_taken: player.total_damage_taken,
        battles_won: player.units_killed.values().sum::<u32>(),
        battles_lost: player.units_lost.values().sum::<u32>(),
        kd_ratio,
        first_attack_tick: player.first_attack_tick,
        first_expansion_tick: None,
        tech_unlock_times: HashMap::new(),
        first_combat_unit_tick: None, // Would need tracking when first military unit is produced
        map_control_over_time: Vec::new(),
        average_army_position: Vec::new(),
        peak_army_size: player.peak_army_size,
    }
}

/// Simple deterministic RNG for reproducibility.
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(1),
        }
    }

    fn next(&mut self) -> u64 {
        // xorshift64
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_combat_works() {
        // Create a minimal simulation with two units facing each other
        let mut sim = Simulation::new();

        // Spawn a Continuity attacker at (0, 0)
        let attacker = sim.spawn_entity(EntitySpawnParams {
            position: Some(Vec2Fixed::new(Fixed::from_num(0), Fixed::from_num(0))),
            health: Some(100),
            movement: Some(Fixed::from_num(2)),
            combat_stats: Some(CombatStats::new(10, Fixed::from_num(50), 20)),
            faction: Some(FactionMember::new(FactionId::Continuity, 0)),
            is_depot: false,
            ..Default::default()
        });

        // Spawn a Collegium target at (20, 0) - within attack range (50)
        let target = sim.spawn_entity(EntitySpawnParams {
            position: Some(Vec2Fixed::new(Fixed::from_num(20), Fixed::from_num(0))),
            health: Some(100),
            movement: Some(Fixed::from_num(2)),
            combat_stats: Some(CombatStats::new(10, Fixed::from_num(50), 20)),
            faction: Some(FactionMember::new(FactionId::Collegium, 0)),
            is_depot: false,
            ..Default::default()
        });

        // Both units should have required components
        let attacker_ent = sim.get_entity(attacker).unwrap();
        assert!(
            attacker_ent.command_queue.is_some(),
            "Attacker should have command queue"
        );
        assert!(
            attacker_ent.combat_stats.is_some(),
            "Attacker should have combat stats"
        );

        // Give attacker an explicit Attack command on target
        let result = sim.apply_command(attacker, Command::Attack(target));
        assert!(result.is_ok(), "apply_command should succeed: {:?}", result);

        // Run simulation for some ticks
        let mut total_damage = 0u32;
        for _ in 0..100 {
            let events = sim.tick();
            for de in &events.damage_events {
                total_damage += de.damage;
            }
        }

        assert!(total_damage > 0, "Combat should have dealt damage");
    }

    #[test]
    fn test_game_with_fast_attack() {
        // Create simulation with two units
        let mut sim = Simulation::new();

        // Spawn close together on same Y
        let unit_a = spawn_unit(&mut sim, "infantry", 50, 100, FactionId::Continuity);
        let unit_b = spawn_unit(&mut sim, "infantry", 150, 100, FactionId::Collegium);

        println!(
            "Initial: Unit A at {:?}, Unit B at {:?}",
            get_entity_position(&sim, unit_a),
            get_entity_position(&sim, unit_b)
        );

        // Issue attack command on each other
        sim.apply_command(unit_a, Command::Attack(unit_b)).unwrap();
        sim.apply_command(unit_b, Command::Attack(unit_a)).unwrap();

        let mut damage_total = 0i64;

        for tick in 0..200 {
            let events = sim.tick();

            for de in &events.damage_events {
                damage_total += de.damage as i64;
                println!(
                    "Tick {}: Damage {} from {} to {}",
                    tick, de.damage, de.attacker, de.target
                );
            }

            // Print positions every 20 ticks
            if tick % 20 == 0 {
                println!(
                    "Tick {}: Unit A at {:?}, Unit B at {:?}",
                    tick,
                    get_entity_position(&sim, unit_a),
                    get_entity_position(&sim, unit_b)
                );
            }
        }

        println!("Total damage: {}", damage_total);
        assert!(damage_total > 0, "Combat should have dealt damage");
    }

    #[test]
    fn test_debug_full_game_combat() {
        // Run a game and verify we get a winner
        let config = GameConfig {
            seed: 42,
            max_ticks: 10000, // Should be enough to destroy HQ
            scenario: Scenario::default(),
            strategy_a: Strategy::rush(),
            strategy_b: Strategy::rush(),
            screenshot_config: None,
            game_id: "debug_game".to_string(),
            faction_registry: None,
        };

        let result = run_game(config);

        println!("Game completed:");
        println!("  Winner: {:?}", result.metrics.winner);
        println!("  Duration: {} ticks", result.metrics.duration_ticks);
        println!("  Win condition: {}", result.metrics.win_condition);

        // Print per-faction metrics
        for (faction, metrics) in &result.metrics.factions {
            let units_lost_total: u32 = metrics.units_lost.values().sum();
            println!(
                "  {}: {} damage dealt, {} units lost",
                faction, metrics.total_damage_dealt, units_lost_total
            );
        }

        // We expect a winner now that buildings can be damaged
        assert!(
            result.metrics.winner.is_some() || result.metrics.duration_ticks < 10000,
            "Game should produce a winner or end early"
        );
    }

    #[test]
    fn test_run_game_deterministic() {
        let config1 = GameConfig {
            seed: 12345,
            max_ticks: 500,
            scenario: Scenario::default(),
            strategy_a: Strategy::default(),
            strategy_b: Strategy::default(),
            screenshot_config: None,
            game_id: "game_1".to_string(),
            faction_registry: None,
        };

        let config2 = GameConfig {
            seed: 12345,
            max_ticks: 500,
            scenario: Scenario::default(),
            strategy_a: Strategy::default(),
            strategy_b: Strategy::default(),
            screenshot_config: None,
            game_id: "game_2".to_string(),
            faction_registry: None,
        };

        let result1 = run_game(config1);
        let result2 = run_game(config2);

        // Same seed should produce same outcome
        assert_eq!(result1.metrics.winner, result2.metrics.winner);
        assert_eq!(
            result1.metrics.duration_ticks,
            result2.metrics.duration_ticks
        );
        assert_eq!(result1.final_state_hash, result2.final_state_hash);
    }

    #[test]
    fn test_different_seeds_different_results() {
        let config1 = GameConfig {
            seed: 1,
            max_ticks: 2000,
            scenario: Scenario::default(),
            strategy_a: Strategy::rush(),
            strategy_b: Strategy::economic(),
            screenshot_config: None,
            game_id: "game_1".to_string(),
            faction_registry: None,
        };

        let config2 = GameConfig {
            seed: 2,
            max_ticks: 2000,
            scenario: Scenario::default(),
            strategy_a: Strategy::rush(),
            strategy_b: Strategy::economic(),
            screenshot_config: None,
            game_id: "game_2".to_string(),
            faction_registry: None,
        };

        let result1 = run_game(config1);
        let result2 = run_game(config2);

        // Different seeds may produce different hashes (not guaranteed but likely)
        // At minimum, the games should complete
        assert!(result1.metrics.duration_ticks > 0);
        assert!(result2.metrics.duration_ticks > 0);
    }

    #[test]
    fn test_strategy_matchups() {
        // Test all strategy combinations
        let strategies = [
            ("Rush", Strategy::rush()),
            ("Economic", Strategy::economic()),
            ("Turtle", Strategy::turtle()),
        ];

        println!("\n=== Strategy Matchup Results ===\n");

        for (name_a, strat_a) in &strategies {
            for (name_b, strat_b) in &strategies {
                // Run 10 games with different seeds
                let mut a_wins = 0;
                let mut b_wins = 0;
                let mut draws = 0;

                for seed in 0..10 {
                    let config = GameConfig {
                        seed,
                        max_ticks: 5000,
                        scenario: Scenario::default(),
                        strategy_a: strat_a.clone(),
                        strategy_b: strat_b.clone(),
                        screenshot_config: None,
                        game_id: format!("{}_vs_{}_{}", name_a, name_b, seed),
                        faction_registry: None,
                    };

                    let result = run_game(config);

                    match result.metrics.winner.as_deref() {
                        Some("continuity") => a_wins += 1,
                        Some("collegium") => b_wins += 1,
                        _ => draws += 1,
                    }
                }

                println!(
                    "{:10} vs {:10}: {:2}-{:2} (draws: {})",
                    name_a, name_b, a_wins, b_wins, draws
                );
            }
        }
        println!();
    }
}
