//! AI system for non-player factions.
//!
//! Provides basic AI behavior for enemy factions:
//! - Resource gathering with harvesters
//! - Unit production when resources allow
//! - Wave-based attacks: units accumulate at rally points before attacking as coordinated waves

use bevy::prelude::*;
use rts_core::factions::FactionId;
use rts_core::math::Vec2Fixed;
use std::collections::HashMap;

use crate::components::{
    CombatStats, GameDepot, GameFaction, GameHarvester, GameHarvesterState, GamePosition,
    GameProductionQueue, GameResourceNode, MovementTarget, PlayerFaction, UnitType,
};

/// Plugin for AI-controlled factions.
pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AiState>()
            .add_systems(Update, ai_production)
            .add_systems(Update, ai_harvester_assignment)
            .add_systems(Update, ai_attack_orders);
    }
}

/// State tracking for AI decisions.
#[derive(Resource)]
pub struct AiState {
    /// Timer for production decisions.
    pub production_timer: f32,
    /// Timer for attack decisions.
    pub attack_timer: f32,
    /// Timer for harvester assignments.
    pub harvester_timer: f32,
    /// Grace period before AI can attack (seconds since game start).
    pub game_time: f32,
    /// Wave-based attack state per faction.
    pub wave_state: HashMap<FactionId, WaveState>,
}

/// Per-faction wave attack state.
#[derive(Debug, Clone)]
pub struct WaveState {
    /// Units accumulating at rally point for next wave.
    pub rally_units: Vec<Entity>,
    /// Time when next wave can be launched.
    pub next_wave_time: f32,
    /// Number of waves launched so far.
    pub wave_number: u32,
    /// Rally point location.
    pub rally_point: Vec2Fixed,
}

impl WaveState {
    fn new(rally_point: Vec2Fixed) -> Self {
        Self {
            rally_units: Vec::new(),
            next_wave_time: 0.0,
            wave_number: 0,
            rally_point,
        }
    }
}

impl Default for AiState {
    fn default() -> Self {
        Self {
            production_timer: 0.0,
            attack_timer: 0.0,
            harvester_timer: 0.0,
            game_time: 0.0,
            wave_state: HashMap::new(),
        }
    }
}

/// Minimum game time before AI will attack (gives player time to prepare).
const AI_GRACE_PERIOD: f32 = 60.0;

/// Minimum units AI needs before launching a wave.
const MIN_WAVE_SIZE: usize = 8;

/// Time between waves (seconds).
const WAVE_INTERVAL: f32 = 20.0;

/// AI resources per faction (simulated - not using player resources).
#[derive(Component)]
pub struct AiFaction {
    /// Current feedstock.
    pub feedstock: i32,
    /// Units produced count.
    pub units_produced: i32,
}

impl Default for AiFaction {
    fn default() -> Self {
        Self {
            feedstock: 500,
            units_produced: 0,
        }
    }
}

/// AI production system - queues units when resources allow.
fn ai_production(
    time: Res<Time>,
    mut ai_state: ResMut<AiState>,
    player_faction: Res<PlayerFaction>,
    mut ai_depots: Query<(&GameFaction, &mut GameProductionQueue), With<GameDepot>>,
    harvesters: Query<&GameFaction, With<GameHarvester>>,
    combat_units: Query<&GameFaction, With<CombatStats>>,
) {
    ai_state.production_timer += time.delta_seconds();

    // Only check every 2 seconds
    if ai_state.production_timer < 2.0 {
        return;
    }
    ai_state.production_timer = 0.0;

    for (faction, mut production) in ai_depots.iter_mut() {
        // Skip player faction
        if faction.faction == player_faction.faction {
            continue;
        }

        // Skip if queue is full
        if !production.can_queue() {
            continue;
        }

        // Count this faction's units
        let harvester_count = harvesters
            .iter()
            .filter(|f| f.faction == faction.faction)
            .count();
        let combat_count = combat_units
            .iter()
            .filter(|f| f.faction == faction.faction)
            .count();

        // Get faction-specific unit IDs
        let harvester_id = UnitType::Harvester.to_unit_id(faction.faction).to_string();
        let infantry_id = UnitType::Infantry.to_unit_id(faction.faction).to_string();
        let ranger_id = UnitType::Ranger.to_unit_id(faction.faction).to_string();

        // Prioritize: 1 harvester first, then combat units, then more harvesters
        if harvester_count == 0 {
            production.enqueue(harvester_id);
            tracing::info!("AI {:?} queued Harvester (first)", faction.faction);
        } else if combat_count < 3 {
            // Build infantry until we have 3
            production.enqueue(infantry_id);
            tracing::info!(
                "AI {:?} queued Infantry (combat {})",
                faction.faction,
                combat_count
            );
        } else if harvester_count < 2 {
            production.enqueue(harvester_id);
            tracing::info!("AI {:?} queued Harvester (second)", faction.faction);
        } else if combat_count < 6 {
            // Mix of infantry and rangers
            if combat_count % 2 == 0 {
                production.enqueue(infantry_id);
            } else {
                production.enqueue(ranger_id);
            }
            tracing::info!(
                "AI {:?} queued combat unit (total {})",
                faction.faction,
                combat_count
            );
        }
    }
}

/// AI harvester assignment - sends idle harvesters to resource nodes.
fn ai_harvester_assignment(
    time: Res<Time>,
    mut ai_state: ResMut<AiState>,
    player_faction: Res<PlayerFaction>,
    mut commands: Commands,
    mut harvesters: Query<
        (Entity, &GameFaction, &mut GameHarvester, &GamePosition),
        Without<MovementTarget>,
    >,
    nodes: Query<(Entity, &GamePosition), With<GameResourceNode>>,
) {
    ai_state.harvester_timer += time.delta_seconds();

    // Only check every 1 second
    if ai_state.harvester_timer < 1.0 {
        return;
    }
    ai_state.harvester_timer = 0.0;

    for (entity, faction, mut harvester, pos) in harvesters.iter_mut() {
        // Skip player faction
        if faction.faction == player_faction.faction {
            continue;
        }

        // Only assign if idle
        if !matches!(harvester.state, GameHarvesterState::Idle) {
            continue;
        }

        // Find nearest node
        let my_pos = pos.as_vec2();
        let nearest_node = nodes.iter().min_by(|(_, a), (_, b)| {
            let dist_a = a.as_vec2().distance_squared(my_pos);
            let dist_b = b.as_vec2().distance_squared(my_pos);
            dist_a.partial_cmp(&dist_b).unwrap()
        });

        if let Some((node_entity, node_pos)) = nearest_node {
            harvester.state = GameHarvesterState::MovingToNode(node_entity);
            harvester.assigned_node = Some(node_entity);
            commands.entity(entity).insert(MovementTarget {
                target: node_pos.value,
            });
            tracing::debug!("AI {:?} harvester assigned to node", faction.faction);
        }
    }
}

/// AI attack orders - uses wave-based attacks with rally points.
/// Units accumulate at rally points, then attack as coordinated waves.
fn ai_attack_orders(
    time: Res<Time>,
    mut ai_state: ResMut<AiState>,
    player_faction: Res<PlayerFaction>,
    mut commands: Commands,
    combat_units: Query<
        (Entity, &GameFaction, &GamePosition, &CombatStats),
        Without<MovementTarget>,
    >,
    enemies: Query<(Entity, &GameFaction, &GamePosition), With<CombatStats>>,
    depots: Query<(&GameFaction, &GamePosition), With<GameDepot>>,
) {
    // Track total game time
    ai_state.game_time += time.delta_seconds();

    // Don't attack during grace period
    if ai_state.game_time < AI_GRACE_PERIOD {
        return;
    }

    ai_state.attack_timer += time.delta_seconds();

    // Only check every 5 seconds
    if ai_state.attack_timer < 5.0 {
        return;
    }
    ai_state.attack_timer = 0.0;

    // Initialize wave state for factions that don't have it yet
    for (faction, depot_pos) in depots.iter() {
        if faction.faction == player_faction.faction {
            continue;
        }
        ai_state
            .wave_state
            .entry(faction.faction)
            .or_insert_with(|| {
                // Place rally point offset from depot
                let depot_vec = depot_pos.as_vec2();
                let rally_offset = Vec2::new(100.0, 100.0);
                let rally_pos = depot_vec + rally_offset;
                WaveState::new(Vec2Fixed::new(
                    rts_core::math::Fixed::from_num(rally_pos.x),
                    rts_core::math::Fixed::from_num(rally_pos.y),
                ))
            });
    }

    // Group idle combat units by faction
    let mut faction_idle_units: HashMap<FactionId, Vec<Entity>> = HashMap::new();

    for (entity, faction, _, _) in combat_units.iter() {
        if faction.faction == player_faction.faction {
            continue;
        }
        faction_idle_units
            .entry(faction.faction)
            .or_default()
            .push(entity);
    }

    // Store game time to avoid borrow checker issues
    let current_game_time = ai_state.game_time;

    // Process each AI faction
    for (faction_id, idle_units) in faction_idle_units {
        let Some(wave_state) = ai_state.wave_state.get_mut(&faction_id) else {
            continue;
        };

        // Add newly idle units to rally
        for &unit in &idle_units {
            if !wave_state.rally_units.contains(&unit) {
                wave_state.rally_units.push(unit);
                // Send unit to rally point
                commands.entity(unit).insert(MovementTarget {
                    target: wave_state.rally_point,
                });
                tracing::debug!(
                    "AI {:?} unit rallying (wave {}, total {})",
                    faction_id,
                    wave_state.wave_number + 1,
                    wave_state.rally_units.len()
                );
            }
        }

        // Check if we should launch a wave
        let has_enough_units = wave_state.rally_units.len() >= MIN_WAVE_SIZE;
        let wave_ready = current_game_time >= wave_state.next_wave_time;

        if has_enough_units && wave_ready {
            // Find nearest enemy to launch the wave at
            let first_unit = wave_state.rally_units[0];
            let Ok((_, _, first_pos, _)) = combat_units.get(first_unit) else {
                continue;
            };
            let my_pos = first_pos.as_vec2();

            let nearest_enemy = enemies
                .iter()
                .filter(|(_, f, _)| f.faction != faction_id)
                .min_by(|(_, _, a), (_, _, b)| {
                    let dist_a = a.as_vec2().distance_squared(my_pos);
                    let dist_b = b.as_vec2().distance_squared(my_pos);
                    dist_a.partial_cmp(&dist_b).unwrap()
                });

            if let Some((_, _, enemy_pos)) = nearest_enemy {
                let unit_count = wave_state.rally_units.len();
                
                // Launch wave - send all rallied units to attack
                for &unit_entity in &wave_state.rally_units {
                    commands.entity(unit_entity).insert(MovementTarget {
                        target: enemy_pos.value,
                    });
                }

                tracing::info!(
                    "AI {:?} launching wave {} with {} units",
                    faction_id,
                    wave_state.wave_number + 1,
                    unit_count
                );

                // Update wave state
                wave_state.wave_number += 1;
                wave_state.next_wave_time = current_game_time + WAVE_INTERVAL;
                wave_state.rally_units.clear();
            }
        }
    }
}
