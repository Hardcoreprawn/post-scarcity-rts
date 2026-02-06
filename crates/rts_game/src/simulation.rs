//! Core simulation driver for the client.
//!
//! This module advances the deterministic core simulation and
//! syncs positions back into Bevy for rendering.

use std::collections::HashMap;

use bevy::prelude::*;
use rts_core::combat::ArmorClass;
use rts_core::components::{
    CombatStats as CoreCombatStats, Command as CoreCommand, DamageType as CoreDamageType, EntityId,
    FactionMember,
};
use rts_core::math::Fixed;
use rts_core::pathfinding::CellType;
use rts_core::simulation::{EntitySpawnParams, Simulation, TickEvents, TICK_RATE};

use crate::components::{
    Armor, ArmorType, AttackTarget, Building, CombatStats, CoreEntityId, DamageType, GameDepot,
    GameFaction, GameHealth, GamePosition, MovementTarget, Stationary,
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
            .init_resource::<BuildingFootprints>()
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
        app.add_systems(
            Update,
            sync_attack_targets_to_core.in_set(CoreSimulationSet::SyncIn),
        );
        app.add_systems(
            Update,
            clear_removed_attack_targets.in_set(CoreSimulationSet::SyncIn),
        );
        app.add_systems(
            Update,
            sync_movement_targets_to_core
                .in_set(CoreSimulationSet::SyncIn)
                .before(apply_command_buffer),
        );
        // Building NavGrid integration
        app.add_systems(
            PreUpdate,
            sync_building_to_navgrid.after(sync_spawned_entities),
        );
        app.add_systems(PreUpdate, clear_building_from_navgrid_on_despawn);
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
                if stats.projectile_speed > 0.0 {
                    core_stats =
                        core_stats.with_projectile_speed(Fixed::from_num(stats.projectile_speed));
                }
                if stats.splash_radius > 0.0 {
                    core_stats =
                        core_stats.with_splash_radius(Fixed::from_num(stats.splash_radius));
                }
                core_stats
            } else {
                CoreCombatStats::new(0, Fixed::ZERO, 1)
            };

            if let Some(armor) = armor {
                stats = stats
                    .with_resistance(map_armor_type_to_class(armor.armor_type), armor.value as u8);
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

/// Sync Bevy AttackTarget components to the core simulation.
///
/// This ensures that when Bevy assigns a target (via acquire_attack_targets in combat.rs),
/// the core simulation knows about it and can apply damage.
fn sync_attack_targets_to_core(
    mut core: ResMut<CoreSimulation>,
    attackers: Query<(&CoreEntityId, &AttackTarget), Changed<AttackTarget>>,
    targets: Query<&CoreEntityId>,
) {
    for (attacker_core_id, attack_target) in attackers.iter() {
        // Look up the target's core entity ID
        if let Ok(target_core_id) = targets.get(attack_target.target) {
            if let Err(e) = core
                .sim
                .set_attack_target(attacker_core_id.0, target_core_id.0)
            {
                tracing::debug!("Failed to sync attack target to core: {}", e);
            }
        }
    }
}

/// Clear attack targets in core when Bevy AttackTarget component is removed.
fn clear_removed_attack_targets(
    mut core: ResMut<CoreSimulation>,
    mut removed: RemovedComponents<AttackTarget>,
    entities: Query<&CoreEntityId>,
) {
    for entity in removed.read() {
        if let Ok(core_id) = entities.get(entity) {
            let _ = core.sim.clear_attack_target(core_id.0);
        }
    }
}

/// Sync Bevy MovementTarget components to core simulation MoveTo commands.
///
/// This ensures that when Bevy systems (like harvester AI) set a MovementTarget,
/// the core simulation receives a MoveTo command to actually move the entity.
fn sync_movement_targets_to_core(
    mut core_commands: ResMut<CoreCommandBuffer>,
    targets: Query<(&CoreEntityId, &MovementTarget), Changed<MovementTarget>>,
) {
    for (core_id, movement_target) in targets.iter() {
        core_commands.set(core_id.0, CoreCommand::MoveTo(movement_target.target));
    }
}

/// Marker component indicating this building has been synced to the NavGrid.
#[derive(Component)]
struct NavGridSynced;

/// Resource to track building footprints for cleanup on despawn.
#[derive(Resource, Default)]
pub struct BuildingFootprints {
    /// Map from entity to (position, (cells_x, cells_y))
    footprints: HashMap<Entity, (rts_core::math::Vec2Fixed, (u32, u32))>,
}

/// Sync newly spawned buildings to the NavGrid by marking their cells as blocked.
fn sync_building_to_navgrid(
    mut commands: Commands,
    mut core: ResMut<CoreSimulation>,
    mut footprints: ResMut<BuildingFootprints>,
    buildings: Query<(Entity, &GamePosition, &Building), (Added<Building>, Without<NavGridSynced>)>,
) {
    for (entity, position, building) in buildings.iter() {
        let (width, height) = building.building_type.size();
        let nav_grid = core.sim.nav_grid_mut();
        let cell_size: f32 = nav_grid.cell_size().to_num();

        // Calculate how many cells this building covers
        let cells_x = ((width / cell_size).ceil() as u32).max(1);
        let cells_y = ((height / cell_size).ceil() as u32).max(1);

        // Get the grid cell for the building position (center-aligned)
        if let Some((center_x, center_y)) = nav_grid.world_to_grid(position.value) {
            // Calculate the top-left cell of the building footprint
            let half_x = cells_x / 2;
            let half_y = cells_y / 2;
            let start_x = center_x.saturating_sub(half_x);
            let start_y = center_y.saturating_sub(half_y);

            // Mark all cells covered by the building as blocked
            for dy in 0..cells_y {
                for dx in 0..cells_x {
                    nav_grid.set_cell(start_x + dx, start_y + dy, CellType::Blocked);
                }
            }

            tracing::debug!(
                "Synced building {:?} at {:?} to NavGrid: {}x{} cells starting at ({}, {})",
                building.building_type,
                position.value,
                cells_x,
                cells_y,
                start_x,
                start_y
            );
        }

        // Track footprint for cleanup on despawn
        footprints
            .footprints
            .insert(entity, (position.value, (cells_x, cells_y)));

        // Mark as synced so we don't process it again
        commands.entity(entity).insert(NavGridSynced);
    }
}

/// Clear building cells from NavGrid when buildings are despawned.
fn clear_building_from_navgrid_on_despawn(
    mut core: ResMut<CoreSimulation>,
    mut removed: RemovedComponents<Building>,
    mut footprints: ResMut<BuildingFootprints>,
) {
    for entity in removed.read() {
        if let Some((position, (cells_x, cells_y))) = footprints.footprints.remove(&entity) {
            let nav_grid = core.sim.nav_grid_mut();

            if let Some((center_x, center_y)) = nav_grid.world_to_grid(position) {
                let half_x = cells_x / 2;
                let half_y = cells_y / 2;
                let start_x = center_x.saturating_sub(half_x);
                let start_y = center_y.saturating_sub(half_y);

                for dy in 0..cells_y {
                    for dx in 0..cells_x {
                        nav_grid.set_cell(start_x + dx, start_y + dy, CellType::Walkable);
                    }
                }

                tracing::debug!(
                    "Cleared building at {:?} from NavGrid: {}x{} cells",
                    position,
                    cells_x,
                    cells_y
                );
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

/// Map client armor type to core armor class for resistance-based damage.
fn map_armor_type_to_class(armor_type: ArmorType) -> ArmorClass {
    match armor_type {
        ArmorType::Unarmored => ArmorClass::Light,
        ArmorType::Light => ArmorClass::Light,
        ArmorType::Heavy => ArmorClass::Heavy,
        ArmorType::Structure => ArmorClass::Building,
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
