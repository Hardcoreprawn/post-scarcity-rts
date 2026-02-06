//! AI system for non-player factions.
//!
//! Provides basic AI behavior for enemy factions:
//! - Resource gathering with harvesters
//! - Unit production when resources allow
//! - Attacking enemy units when there are enough combat units

use bevy::prelude::*;
use rts_core::factions::FactionId;

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
}

impl Default for AiState {
    fn default() -> Self {
        Self {
            production_timer: 0.0,
            attack_timer: 0.0,
            harvester_timer: 0.0,
            game_time: 0.0,
        }
    }
}

/// Minimum game time before AI will attack (gives player time to prepare).
const AI_GRACE_PERIOD: f32 = 60.0;

/// Minimum units AI needs before considering an attack.
const AI_ATTACK_THRESHOLD: usize = 5;

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

/// AI attack orders - sends combat units to attack enemies.
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

    // Group idle combat units by faction
    let mut faction_units: std::collections::HashMap<FactionId, Vec<Entity>> =
        std::collections::HashMap::new();

    for (entity, faction, _, _) in combat_units.iter() {
        if faction.faction == player_faction.faction {
            continue;
        }
        faction_units
            .entry(faction.faction)
            .or_default()
            .push(entity);
    }

    // For each AI faction with enough units, attack!
    for (faction_id, units) in faction_units {
        // Need enough idle units to attack
        if units.len() < AI_ATTACK_THRESHOLD {
            continue;
        }

        // Find nearest enemy to the first unit
        let first_unit = units[0];
        let Ok((_, _, first_pos, _)) = combat_units.get(first_unit) else {
            continue;
        };
        let my_pos = first_pos.as_vec2();

        // Find nearest enemy from a different faction
        let nearest_enemy = enemies
            .iter()
            .filter(|(_, f, _)| f.faction != faction_id)
            .min_by(|(_, _, a), (_, _, b)| {
                let dist_a = a.as_vec2().distance_squared(my_pos);
                let dist_b = b.as_vec2().distance_squared(my_pos);
                dist_a.partial_cmp(&dist_b).unwrap()
            });

        if let Some((_, _, enemy_pos)) = nearest_enemy {
            let unit_count = units.len();
            // Send all idle units to attack
            for unit_entity in &units {
                if let Ok((_, _, _, _)) = combat_units.get(*unit_entity) {
                    commands.entity(*unit_entity).insert(MovementTarget {
                        target: enemy_pos.value,
                    });
                }
            }
            tracing::info!("AI {:?} attacking with {} units", faction_id, unit_count);
        }
    }
}
