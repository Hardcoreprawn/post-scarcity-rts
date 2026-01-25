//! Simulation plugin for processing game commands and unit movement.
//!
//! Handles command processing, unit movement, and separation behavior.

use bevy::prelude::*;
use rts_core::components::Command as CoreCommand;
use rts_core::math::{Fixed, Vec2Fixed};

use crate::components::{Collider, GameCommandQueue, GamePosition, MovementTarget, Stationary};

/// Unit movement speed (units per second).
pub const UNIT_SPEED: f32 = 150.0;

/// Unit collision radius for separation.
pub const UNIT_RADIUS: f32 = 20.0;

/// Separation force strength - how strongly units push apart.
pub const SEPARATION_FORCE: f32 = 200.0;

/// Minimum distance before separation kicks in.
pub const SEPARATION_DISTANCE: f32 = UNIT_RADIUS * 2.0;

/// Plugin for processing game commands and unit movement.
///
/// This handles:
/// - Processing commands from unit command queues
/// - Moving units toward their targets
pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, process_commands)
            .add_systems(Update, unit_movement.after(process_commands))
            .add_systems(Update, unit_separation.after(unit_movement))
            .add_systems(Update, building_collision.after(unit_separation));
    }
}

/// Processes commands from unit command queues and sets movement targets.
fn process_commands(
    mut commands: Commands,
    mut units: Query<(Entity, &mut GameCommandQueue, Option<&MovementTarget>)>,
) {
    for (entity, mut queue, current_target) in units.iter_mut() {
        if let Some(command) = queue.current() {
            match command {
                CoreCommand::MoveTo(target) | CoreCommand::AttackMove(target) => {
                    // Set the movement target if we don't have one or it's different
                    let should_update = current_target.map(|t| t.target != *target).unwrap_or(true);

                    if should_update {
                        commands
                            .entity(entity)
                            .insert(MovementTarget { target: *target });
                    }
                }
                CoreCommand::Stop => {
                    // Remove movement target and clear commands
                    commands.entity(entity).remove::<MovementTarget>();
                    queue.pop();
                }
                CoreCommand::HoldPosition => {
                    // Remove movement target but keep the command
                    commands.entity(entity).remove::<MovementTarget>();
                }
                _ => {
                    // For other commands, just pop and continue
                    queue.pop();
                }
            }
        }
    }
}

/// Moves units toward their movement targets.
fn unit_movement(
    mut commands: Commands,
    time: Res<Time>,
    mut units: Query<(
        Entity,
        &mut GamePosition,
        &mut GameCommandQueue,
        &MovementTarget,
    )>,
) {
    let delta = time.delta_seconds();

    for (entity, mut position, mut queue, target) in units.iter_mut() {
        let current = position.value;
        let goal = target.target;

        // Calculate direction to target
        let dx = goal.x - current.x;
        let dy = goal.y - current.y;

        // Calculate distance squared (avoid sqrt when possible)
        let dist_sq: f32 = (dx * dx + dy * dy).to_num();

        // Arrival threshold
        const ARRIVAL_THRESHOLD: f32 = 5.0;
        const ARRIVAL_THRESHOLD_SQ: f32 = ARRIVAL_THRESHOLD * ARRIVAL_THRESHOLD;

        if dist_sq < ARRIVAL_THRESHOLD_SQ {
            // Arrived at destination
            position.value = goal;
            commands.entity(entity).remove::<MovementTarget>();
            queue.pop(); // Command completed
        } else {
            // Move toward target
            let dist = dist_sq.sqrt();
            let move_dist = UNIT_SPEED * delta;

            if move_dist >= dist {
                // Will arrive this frame
                position.value = goal;
            } else {
                // Normalize direction and move
                let nx = Fixed::from_num(dx.to_num::<f32>() / dist * move_dist);
                let ny = Fixed::from_num(dy.to_num::<f32>() / dist * move_dist);
                position.value = Vec2Fixed::new(current.x + nx, current.y + ny);
            }
        }
    }
}

/// Separates overlapping units by pushing them apart.
///
/// This creates natural clustering behavior where units don't stack
/// on top of each other but instead spread out around their destinations.
fn unit_separation(
    time: Res<Time>,
    mut units: Query<(Entity, &mut GamePosition), Without<Stationary>>,
) {
    let delta = time.delta_seconds();

    // Collect all positions first to avoid borrow issues
    let positions: Vec<(Entity, Vec2)> = units
        .iter()
        .map(|(e, p)| (e, Vec2::new(p.value.x.to_num(), p.value.y.to_num())))
        .collect();

    // For each unit, calculate separation force from nearby units
    for (entity, mut position) in units.iter_mut() {
        let my_pos = Vec2::new(position.value.x.to_num(), position.value.y.to_num());
        let mut separation = Vec2::ZERO;
        let mut neighbor_count = 0;

        for (other_entity, other_pos) in &positions {
            if entity == *other_entity {
                continue;
            }

            let diff = my_pos - *other_pos;
            let dist_sq = diff.length_squared();

            if dist_sq > 0.0 && dist_sq < SEPARATION_DISTANCE * SEPARATION_DISTANCE {
                let dist = dist_sq.sqrt();
                // Stronger push when closer (inverse relationship)
                let strength = 1.0 - (dist / SEPARATION_DISTANCE);
                separation += diff.normalize() * strength;
                neighbor_count += 1;
            }
        }

        if neighbor_count > 0 {
            // Average and apply separation
            separation /= neighbor_count as f32;
            let push = separation * SEPARATION_FORCE * delta;

            position.value = Vec2Fixed::new(
                Fixed::from_num(my_pos.x + push.x),
                Fixed::from_num(my_pos.y + push.y),
            );
        }
    }
}

/// Pushes units out of building and terrain colliders.
fn building_collision(
    mut units: Query<&mut GamePosition, Without<Stationary>>,
    colliders: Query<(&GamePosition, &Collider), With<Stationary>>,
) {
    // Small margin (5.0) so units stop at edge but stay in attack/harvest range
    const COLLISION_MARGIN: f32 = 5.0;

    for mut unit_pos in units.iter_mut() {
        let my_pos = Vec2::new(unit_pos.value.x.to_num(), unit_pos.value.y.to_num());

        for (coll_game_pos, collider) in colliders.iter() {
            let coll_pos = Vec2::new(
                coll_game_pos.value.x.to_num(),
                coll_game_pos.value.y.to_num(),
            );

            if let Some(push) = collider.push_out(my_pos, coll_pos, COLLISION_MARGIN) {
                unit_pos.value = Vec2Fixed::new(
                    Fixed::from_num(my_pos.x + push.x),
                    Fixed::from_num(my_pos.y + push.y),
                );
                break; // Only apply one collision per frame to avoid jitter
            }
        }
    }
}
