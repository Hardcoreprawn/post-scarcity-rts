//! Core simulation driver for the client.
//!
//! This module advances the deterministic core simulation and
//! syncs positions back into Bevy for rendering.

use std::collections::HashMap;

use bevy::prelude::*;
use rts_core::components::{
    ArmorType as CoreArmorType, CombatStats as CoreCombatStats, Command as CoreCommand,
    DamageType as CoreDamageType, EntityId, FactionMember,
};
use rts_core::math::Fixed;
use rts_core::simulation::{EntitySpawnParams, Simulation, TickEvents, TICK_RATE};

use crate::components::{
    Armor, ArmorType, CombatStats, CoreEntityId, DamageType, GameDepot, GameFaction, GameHealth,
    GamePosition, Stationary,
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
pub enum CoreCommandMode {
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

/// Record of a command applied to the core simulation.
#[derive(Debug, Clone)]
pub struct CommandRecord {
    /// Simulation tick when the command was applied.
    pub tick: u64,
    /// Target core entity.
    pub entity: EntityId,
    /// Command issued.
    pub command: CoreCommand,
    /// Whether the command replaced or queued.
    pub mode: CoreCommandMode,
}

/// Replay-ready command stream.
#[derive(Resource, Default)]
pub struct CommandStream {
    /// Ordered command records.
    pub records: Vec<CommandRecord>,
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
    /// Latest core tick events.
    pub last_events: TickEvents,
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
            .init_resource::<CommandStream>()
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
                apply_command_buffer.in_set(CoreSimulationSet::SyncIn),
            )
            .add_systems(Update, tick_core_simulation.in_set(CoreSimulationSet::Tick))
            .add_systems(
                Update,
                sync_positions_from_core.in_set(CoreSimulationSet::SyncOut),
            );
        app.add_systems(
            Update,
            sync_health_from_core.in_set(CoreSimulationSet::SyncOut),
        );
    }
}

fn unit_speed_per_tick() -> Fixed {
    Fixed::from_num(UNIT_SPEED / TICK_RATE as f32)
}

fn sync_spawned_entities(
    mut commands: Commands,
    mut core: ResMut<CoreSimulation>,
    spawned: Query<
        (
            Entity,
            &GamePosition,
            Option<&Stationary>,
            Option<&GameHealth>,
            Option<&CombatStats>,
            Option<&Armor>,
            Option<&GameFaction>,
            Option<&GameDepot>,
        ),
        Without<CoreEntityId>,
    >,
) {
    let speed = unit_speed_per_tick();

    for (entity, position, stationary, health, combat_stats, armor, faction, depot) in
        spawned.iter()
    {
        let mut core_combat = None;
        if combat_stats.is_some() || armor.is_some() {
            let mut stats = if let Some(stats) = combat_stats {
                let cooldown_ticks =
                    (stats.attack_cooldown * TICK_RATE as f32).round().max(1.0) as u32;
                let mut core_stats = CoreCombatStats::new(
                    stats.damage,
                    Fixed::from_num(stats.range),
                    cooldown_ticks,
                );
                core_stats = core_stats.with_damage_type(map_damage_type(stats.damage_type));
                core_stats
            } else {
                CoreCombatStats::new(0, Fixed::ZERO, 1)
            };

            if let Some(armor) = armor {
                stats = stats.with_armor(map_armor_type(armor.armor_type), armor.value);
            }

            core_combat = Some(stats);
        }

        let params = EntitySpawnParams {
            position: Some(position.value),
            movement: if stationary.is_some() {
                None
            } else {
                Some(speed)
            },
            health: health.map(|health| health.max),
            combat_stats: core_combat,
            faction: faction.map(|faction| FactionMember::new(faction.faction, 0)),
            is_depot: depot.is_some(),
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

fn apply_command_buffer(
    mut core: ResMut<CoreSimulation>,
    mut core_commands: ResMut<CoreCommandBuffer>,
    mut command_stream: ResMut<CommandStream>,
) {
    for request in core_commands.pending.drain(..) {
        command_stream.records.push(CommandRecord {
            tick: core.sim.get_tick(),
            entity: request.entity,
            command: request.command.clone(),
            mode: request.mode,
        });
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
        core.last_events = core.sim.tick();
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

fn sync_health_from_core(
    core: Res<CoreSimulation>,
    mut entities: Query<(&CoreEntityId, &mut GameHealth)>,
) {
    for (core_id, mut health) in entities.iter_mut() {
        if let Some(entity) = core.sim.get_entity(core_id.0) {
            if let Some(core_health) = entity.health {
                health.current = core_health.current;
                health.max = core_health.max;
            }
        }
    }
}

fn map_damage_type(damage_type: DamageType) -> CoreDamageType {
    match damage_type {
        DamageType::Kinetic => CoreDamageType::Kinetic,
        DamageType::Energy => CoreDamageType::Energy,
        DamageType::Explosive => CoreDamageType::Explosive,
    }
}

fn map_armor_type(armor_type: ArmorType) -> CoreArmorType {
    match armor_type {
        ArmorType::Unarmored => CoreArmorType::Unarmored,
        ArmorType::Light => CoreArmorType::Light,
        ArmorType::Heavy => CoreArmorType::Heavy,
        ArmorType::Structure => CoreArmorType::Building,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::GameCommandQueue;

    #[test]
    fn command_stream_records_commands() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(SimulationPlugin);

        let entity = app
            .world_mut()
            .spawn((GamePosition::ORIGIN, GameCommandQueue::new()))
            .id();

        app.update();

        let core_id = app.world().get::<CoreEntityId>(entity).unwrap().0;
        app.world_mut()
            .resource_mut::<CoreCommandBuffer>()
            .set(core_id, CoreCommand::Stop);

        app.update();

        let stream = app.world().resource::<CommandStream>();
        assert_eq!(stream.records.len(), 1);
        assert_eq!(stream.records[0].entity, core_id);
        assert_eq!(stream.records[0].command, CoreCommand::Stop);
    }

    #[test]
    fn spawned_health_syncs_to_core() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(SimulationPlugin);

        let entity = app
            .world_mut()
            .spawn((GamePosition::ORIGIN, GameHealth::new(42)))
            .id();

        app.update();

        let core_id = app.world().get::<CoreEntityId>(entity).unwrap().0;
        let core_health = app
            .world()
            .resource::<CoreSimulation>()
            .sim
            .get_entity(core_id)
            .unwrap()
            .health
            .unwrap();

        assert_eq!(core_health.max, 42);
        assert_eq!(core_health.current, 42);
    }
}
