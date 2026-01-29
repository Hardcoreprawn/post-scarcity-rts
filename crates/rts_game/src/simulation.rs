//! Core simulation driver for the client.
//!
//! This module advances the deterministic core simulation and
//! syncs positions back into Bevy for rendering.

use std::collections::HashMap;

use bevy::prelude::*;
use rts_core::components::{Command as CoreCommand, EntityId};
use rts_core::math::Fixed;
use rts_core::simulation::{EntitySpawnParams, Simulation, TICK_RATE};

use crate::components::{
    AttackTarget, CoreEntityId, GameCommandQueue, GamePosition, MovementTarget, Stationary,
};

/// Systems that emit commands into the core simulation.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ClientCommandSet {
    /// Command gathering from input/UI.
    Gather,
}

/// Unit movement speed (units per second).
pub const UNIT_SPEED: f32 = 150.0;

/// Unit collision radius for selection/formation spacing.
pub const UNIT_RADIUS: f32 = 20.0;

/// Command issuance mode for the core simulation.
#[derive(Debug, Clone, Copy)]
enum CoreCommandMode {
    /// Replace the queue with a single command.
    Replace,
    /// Add a command to the queue.
    Queue,
}

/// Buffered command request targeting the core simulation.
#[derive(Debug, Clone)]
struct CoreCommandRequest {
    entity: EntityId,
    command: CoreCommand,
    mode: CoreCommandMode,
}

/// Command buffer for issuing core simulation commands from the client.
#[derive(Resource, Default)]
pub struct CoreCommandBuffer {
    pending: Vec<CoreCommandRequest>,
}

impl CoreCommandBuffer {
    /// Replace the command queue for an entity.
    pub fn set(&mut self, entity: EntityId, command: CoreCommand) {
        self.pending.push(CoreCommandRequest {
            entity,
            command,
            mode: CoreCommandMode::Replace,
        });
    }

    /// Queue a command for an entity.
    pub fn queue(&mut self, entity: EntityId, command: CoreCommand) {
        self.pending.push(CoreCommandRequest {
            entity,
            command,
            mode: CoreCommandMode::Queue,
        });
    }
}

/// Core simulation state and entity mapping.
#[derive(Resource, Default)]
pub struct CoreSimulation {
    /// Core deterministic simulation state.
    pub sim: Simulation,
    /// Accumulator for fixed-step ticking.
    accumulator: f32,
    /// Map of Bevy entities to core entity IDs.
    entity_map: HashMap<Entity, EntityId>,
}

impl CoreSimulation {
    fn register_entity(&mut self, entity: Entity, id: EntityId) {
        self.entity_map.insert(entity, id);
    }

    fn unregister_entity(&mut self, entity: Entity) -> Option<EntityId> {
        self.entity_map.remove(&entity)
    }
}

/// Core simulation ordering.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum CoreSimulationSet {
    /// Sync inputs from Bevy into the core simulation.
    SyncIn,
    /// Advance the core simulation.
    Tick,
    /// Sync core simulation output back to Bevy.
    SyncOut,
}

/// Plugin that advances the core simulation and mirrors positions into Bevy.
pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CoreSimulation>()
            .init_resource::<CoreCommandBuffer>()
            .configure_sets(
                Update,
                (
                    CoreSimulationSet::SyncIn,
                    CoreSimulationSet::Tick,
                    CoreSimulationSet::SyncOut,
                )
                    .chain(),
            )
            .configure_sets(
                Update,
                ClientCommandSet::Gather.before(CoreSimulationSet::SyncIn),
            )
            .add_systems(PreUpdate, sync_spawned_entities)
            .add_systems(PreUpdate, sync_removed_entities)
            .add_systems(
                Update,
                sync_movement_targets.in_set(CoreSimulationSet::SyncIn),
            )
            .add_systems(
                Update,
                sync_attack_targets.in_set(CoreSimulationSet::SyncIn),
            )
            .add_systems(
                Update,
                apply_command_buffer.in_set(CoreSimulationSet::SyncIn),
            )
            .add_systems(Update, tick_core_simulation.in_set(CoreSimulationSet::Tick))
            .add_systems(
                Update,
                sync_positions_from_core.in_set(CoreSimulationSet::SyncOut),
            );
    }
}

fn unit_speed_per_tick() -> Fixed {
    Fixed::from_num(UNIT_SPEED / TICK_RATE as f32)
}

fn sync_spawned_entities(
    mut commands: Commands,
    mut core: ResMut<CoreSimulation>,
    spawned: Query<(Entity, &GamePosition, Option<&Stationary>), Without<CoreEntityId>>,
) {
    let speed = unit_speed_per_tick();

    for (entity, position, stationary) in spawned.iter() {
        let params = EntitySpawnParams {
            position: Some(position.value),
            movement: if stationary.is_some() {
                None
            } else {
                Some(speed)
            },
            ..Default::default()
        };

        let core_id = core.sim.spawn_entity(params);
        core.register_entity(entity, core_id);
        commands.entity(entity).insert(CoreEntityId(core_id));
    }
}

fn sync_removed_entities(
    mut removed: RemovedComponents<CoreEntityId>,
    mut core: ResMut<CoreSimulation>,
) {
    for entity in removed.read() {
        if let Some(core_id) = core.unregister_entity(entity) {
            let _ = core.sim.despawn_entity(core_id);
        }
    }
}

fn sync_movement_targets(
    mut core_commands: ResMut<CoreCommandBuffer>,
    changed_targets: Query<
        (&CoreEntityId, &MovementTarget),
        (Changed<MovementTarget>, With<GameCommandQueue>),
    >,
    mut removed_targets: RemovedComponents<MovementTarget>,
    core_ids: Query<&CoreEntityId>,
) {
    for (core_id, target) in changed_targets.iter() {
        core_commands.set(core_id.0, CoreCommand::MoveTo(target.target));
    }

    for entity in removed_targets.read() {
        if let Ok(core_id) = core_ids.get(entity) {
            core_commands.set(core_id.0, CoreCommand::Stop);
        }
    }
}

fn sync_attack_targets(
    mut core_commands: ResMut<CoreCommandBuffer>,
    changed_targets: Query<
        (&CoreEntityId, &AttackTarget),
        (Changed<AttackTarget>, With<GameCommandQueue>),
    >,
    core_targets: Query<&CoreEntityId>,
) {
    for (core_id, attack_target) in changed_targets.iter() {
        if let Ok(target_core) = core_targets.get(attack_target.target) {
            core_commands.set(core_id.0, CoreCommand::Attack(target_core.0));
        }
    }
}

fn apply_command_buffer(
    mut core: ResMut<CoreSimulation>,
    mut core_commands: ResMut<CoreCommandBuffer>,
) {
    for request in core_commands.pending.drain(..) {
        let result = match request.mode {
            CoreCommandMode::Replace => core.sim.apply_command(request.entity, request.command),
            CoreCommandMode::Queue => core.sim.queue_command(request.entity, request.command),
        };

        if result.is_err() {
            tracing::debug!("Failed to apply core command");
        }
    }
}

fn tick_core_simulation(time: Res<Time>, mut core: ResMut<CoreSimulation>) {
    core.accumulator += time.delta_seconds();
    let step = 1.0 / TICK_RATE as f32;

    while core.accumulator >= step {
        core.sim.tick();
        core.accumulator -= step;
    }
}

fn sync_positions_from_core(
    core: Res<CoreSimulation>,
    mut entities: Query<(&CoreEntityId, &mut GamePosition)>,
) {
    for (core_id, mut position) in entities.iter_mut() {
        if let Some(entity) = core.sim.get_entity(core_id.0) {
            if let Some(core_pos) = entity.position {
                position.value = core_pos.value;
            }
        }
    }
}
