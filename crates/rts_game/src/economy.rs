//! Economy plugin for resource gathering.
//!
//! Handles harvester AI, resource node depletion, and player resource updates.

use bevy::prelude::*;

use crate::components::{
    GameDepot, GameFaction, GameHarvester, GameHarvesterState, GamePosition, GameResourceNode,
    MovementTarget, ResourceNodeType,
};

/// Distance threshold for harvester interactions.
pub const HARVEST_DISTANCE: f32 = 50.0;
const HARVEST_DISTANCE_SQ: f32 = HARVEST_DISTANCE * HARVEST_DISTANCE;

/// Player resources for the local player.
#[derive(Resource, Debug, Clone)]
pub struct PlayerResources {
    /// Current feedstock amount.
    pub feedstock: i32,
    /// Maximum feedstock storage capacity.
    pub feedstock_cap: i32,
    /// Current supply used.
    pub supply_used: i32,
    /// Maximum supply available.
    pub supply_cap: i32,
}

impl Default for PlayerResources {
    fn default() -> Self {
        Self {
            feedstock: 500,
            feedstock_cap: 10000, // Large cap, grows with buildings
            supply_used: 5,       // 3 infantry (1 each) + 1 harvester (2) = 5
            supply_cap: 10,       // From depot only, build Supply Depots for more
        }
    }
}

/// Plugin for the resource gathering economy.
///
/// Handles:
/// - Harvester AI (find nodes, gather, return to depot)
/// - Resource node depletion
/// - Player resource updates
pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                harvester_ai,
                count_harvesters_per_node.after(harvester_ai),
                harvester_gathering.after(count_harvesters_per_node),
                update_node_visuals.after(harvester_gathering),
            ),
        );
    }
}

/// Helper to calculate squared distance between two positions.
pub fn distance_sq(a: &GamePosition, b: &GamePosition) -> f32 {
    let dx: f32 = (a.value.x - b.value.x).to_num();
    let dy: f32 = (a.value.y - b.value.y).to_num();
    dx * dx + dy * dy
}

/// Harvester AI state machine - finds nodes, returns to depots.
fn harvester_ai(
    mut commands: Commands,
    mut harvesters: Query<(Entity, &GamePosition, &GameFaction, &mut GameHarvester)>,
    nodes: Query<(Entity, &GamePosition, &GameResourceNode)>,
    depots: Query<(Entity, &GamePosition, &GameFaction), With<GameDepot>>,
) {
    for (harvester_entity, harvester_pos, harvester_faction, mut harvester) in harvesters.iter_mut()
    {
        match harvester.state {
            GameHarvesterState::Idle => {
                if harvester.is_empty() {
                    // Check for manually assigned node first
                    let target_node = if let Some(assigned) = harvester.assigned_node {
                        // Check if assigned node still exists and isn't depleted
                        if let Ok((_, pos, node)) = nodes.get(assigned) {
                            if !node.is_depleted() {
                                Some((assigned, pos.value))
                            } else {
                                // Assigned node depleted, clear assignment
                                harvester.assigned_node = None;
                                None
                            }
                        } else {
                            // Assigned node no longer exists
                            harvester.assigned_node = None;
                            None
                        }
                    } else {
                        None
                    };

                    // Use assigned node or find nearest
                    let node_target = target_node.or_else(|| {
                        nodes
                            .iter()
                            .filter(|(_, _, node)| !node.is_depleted())
                            .min_by(|(_, pos_a, _), (_, pos_b, _)| {
                                let dist_a = distance_sq(harvester_pos, pos_a);
                                let dist_b = distance_sq(harvester_pos, pos_b);
                                dist_a.partial_cmp(&dist_b).unwrap()
                            })
                            .map(|(e, p, _)| (e, p.value))
                    });

                    if let Some((node_entity, node_pos)) = node_target {
                        harvester.state = GameHarvesterState::MovingToNode(node_entity);
                        commands
                            .entity(harvester_entity)
                            .insert(MovementTarget { target: node_pos });
                    }
                } else {
                    // Has resources, find friendly depot
                    if let Some((depot_entity, depot_pos, _)) = depots
                        .iter()
                        .filter(|(_, _, faction)| faction.faction == harvester_faction.faction)
                        .min_by(|(_, pos_a, _), (_, pos_b, _)| {
                            let dist_a = distance_sq(harvester_pos, pos_a);
                            let dist_b = distance_sq(harvester_pos, pos_b);
                            dist_a.partial_cmp(&dist_b).unwrap()
                        })
                    {
                        harvester.state = GameHarvesterState::Returning(depot_entity);
                        commands.entity(harvester_entity).insert(MovementTarget {
                            target: depot_pos.value,
                        });
                    }
                }
            }

            GameHarvesterState::MovingToNode(node_entity) => {
                // Check if node still exists and has resources
                if let Ok((_, node_pos, node)) = nodes.get(node_entity) {
                    if node.is_depleted() {
                        harvester.state = GameHarvesterState::Idle;
                        commands.entity(harvester_entity).remove::<MovementTarget>();
                    } else if distance_sq(harvester_pos, node_pos) < HARVEST_DISTANCE_SQ {
                        // Arrived at node
                        harvester.state = GameHarvesterState::Gathering(node_entity);
                        commands.entity(harvester_entity).remove::<MovementTarget>();
                    }
                    // else keep moving
                } else {
                    // Node no longer exists
                    harvester.state = GameHarvesterState::Idle;
                    commands.entity(harvester_entity).remove::<MovementTarget>();
                }
            }

            GameHarvesterState::Gathering(_) => {
                // Gathering is handled by harvester_gathering system
                if harvester.is_full() {
                    // Find depot to return to
                    if let Some((depot_entity, depot_pos, _)) = depots
                        .iter()
                        .filter(|(_, _, faction)| faction.faction == harvester_faction.faction)
                        .min_by(|(_, pos_a, _), (_, pos_b, _)| {
                            let dist_a = distance_sq(harvester_pos, pos_a);
                            let dist_b = distance_sq(harvester_pos, pos_b);
                            dist_a.partial_cmp(&dist_b).unwrap()
                        })
                    {
                        harvester.state = GameHarvesterState::Returning(depot_entity);
                        commands.entity(harvester_entity).insert(MovementTarget {
                            target: depot_pos.value,
                        });
                    } else {
                        harvester.state = GameHarvesterState::Idle;
                    }
                }
            }

            GameHarvesterState::Returning(depot_entity) => {
                if let Ok((_, depot_pos, _)) = depots.get(depot_entity) {
                    if distance_sq(harvester_pos, depot_pos) < HARVEST_DISTANCE_SQ {
                        // Arrived at depot
                        harvester.state = GameHarvesterState::Depositing;
                        commands.entity(harvester_entity).remove::<MovementTarget>();
                    }
                    // else keep moving
                } else {
                    // Depot no longer exists, find another
                    harvester.state = GameHarvesterState::Idle;
                    commands.entity(harvester_entity).remove::<MovementTarget>();
                }
            }

            GameHarvesterState::Depositing => {
                // Depositing is instant for now, handled below
            }
        }
    }
}

/// Handles actual resource gathering and depositing.
fn harvester_gathering(
    mut harvesters: Query<(&GamePosition, &GameFaction, &mut GameHarvester)>,
    mut nodes: Query<(Entity, &GamePosition, &mut GameResourceNode)>,
    depots: Query<(Entity, &GamePosition, &GameFaction), With<GameDepot>>,
    mut resources: ResMut<PlayerResources>,
) {
    for (harvester_pos, harvester_faction, mut harvester) in harvesters.iter_mut() {
        match harvester.state {
            GameHarvesterState::Gathering(node_entity) => {
                // Cooldown for gathering
                if harvester.cooldown_timer > 0 {
                    harvester.cooldown_timer -= 1;
                    continue;
                }

                if let Ok((_, _, mut node)) = nodes.get_mut(node_entity) {
                    if node.is_depleted() {
                        harvester.state = GameHarvesterState::Idle;
                    } else {
                        // Gather resources
                        let to_gather = harvester.gather_rate.min(harvester.available_capacity());
                        let gathered = node.extract(to_gather);
                        harvester.load(gathered);
                        harvester.cooldown_timer = harvester.gather_cooldown;

                        // Check if full
                        if harvester.is_full() {
                            // Will be handled by harvester_ai next frame
                        }
                    }
                } else {
                    harvester.state = GameHarvesterState::Idle;
                }
            }

            GameHarvesterState::Depositing => {
                // Find the depot we're at
                let at_depot = depots.iter().any(|(_, depot_pos, depot_faction)| {
                    depot_faction.faction == harvester_faction.faction
                        && distance_sq(harvester_pos, depot_pos) < HARVEST_DISTANCE_SQ
                });

                if at_depot {
                    let load = harvester.unload();
                    // Add to player resources (respecting cap)
                    let space = resources.feedstock_cap - resources.feedstock;
                    let deposited = load.min(space);
                    resources.feedstock += deposited;

                    // Go back to gathering
                    harvester.state = GameHarvesterState::Idle;
                } else {
                    // Not at depot anymore
                    harvester.state = GameHarvesterState::Idle;
                }
            }

            _ => {}
        }
    }
}

/// Counts harvesters targeting each node and updates current_harvesters.
fn count_harvesters_per_node(
    harvesters: Query<&GameHarvester>,
    mut nodes: Query<(Entity, &mut GameResourceNode)>,
) {
    // Reset all counts
    for (_, mut node) in nodes.iter_mut() {
        node.current_harvesters = 0;
    }

    // Count harvesters targeting each node
    for harvester in harvesters.iter() {
        let target = match harvester.state {
            GameHarvesterState::MovingToNode(e) | GameHarvesterState::Gathering(e) => Some(e),
            _ => None,
        };
        if let Some(target_entity) = target {
            if let Ok((_, mut node)) = nodes.get_mut(target_entity) {
                node.current_harvesters = node.current_harvesters.saturating_add(1);
            }
        }
    }
}

/// Updates node visuals based on type and state.
fn update_node_visuals(mut nodes: Query<(&GameResourceNode, &mut Sprite)>) {
    for (node, mut sprite) in nodes.iter_mut() {
        match node.node_type {
            ResourceNodeType::Permanent => {
                // Show harvester crowding with color shift
                if node.current_harvesters > node.optimal_harvesters {
                    // Overcrowded - yellowish warning
                    sprite.color = Color::srgba(0.6, 0.7, 0.2, 1.0);
                } else {
                    // Healthy - bright green
                    sprite.color = Color::srgba(0.3, 0.8, 0.4, 1.0);
                }
            }
            ResourceNodeType::Temporary => {
                if node.is_depleted() {
                    // Fade out depleted nodes
                    sprite.color = Color::srgba(0.4, 0.3, 0.1, 0.3);
                } else {
                    // Scale brightness based on remaining (assuming max ~1000)
                    let ratio = (node.remaining as f32 / 1000.0).clamp(0.4, 1.0);
                    sprite.color = Color::srgba(0.9, 0.7, 0.2, ratio);
                }
            }
        }
    }
}
