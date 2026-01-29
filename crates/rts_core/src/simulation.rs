//! Core simulation loop.
//!
//! The simulation runs at a fixed tick rate and processes
//! all game logic deterministically. This module is the heart of the
//! RTS game engine, managing all entities and running systems each tick.
//!
//! # Determinism
//!
//! All operations in this module are fully deterministic:
//! - No floating-point math (uses fixed-point via [`Fixed`])
//! - No system randomness (use seeded RNG if needed)
//! - Consistent iteration order (sorted entity IDs)
//! - Same inputs always produce same outputs
//!
//! # Example
//!
//! ```
//! use rts_core::simulation::{Simulation, EntitySpawnParams};
//! use rts_core::components::Command;
//! use rts_core::math::{Fixed, Vec2Fixed};
//!
//! let mut sim = Simulation::new();
//!
//! // Spawn a unit
//! let unit = sim.spawn_entity(EntitySpawnParams {
//!     position: Some(Vec2Fixed::ZERO),
//!     movement: Some(Fixed::from_num(2)),
//!     ..Default::default()
//! });
//!
//! // Issue a move command
//! sim.apply_command(unit, Command::MoveTo(Vec2Fixed::new(
//!     Fixed::from_num(10),
//!     Fixed::from_num(10),
//! )));
//!
//! // Advance simulation
//! sim.tick();
//! ```

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

use crate::components::{
    AttackTarget, CombatStats, Command, CommandQueue, EntityId, Health, Movement, PatrolState,
    Position, ProductionQueue, Projectile, Velocity,
};
use crate::error::{GameError, Result};
use crate::math::{Fixed, Vec2Fixed};
use crate::systems::{
    calculate_damage, command_processing_system, health_system, movement_system, production_system,
    CombatEvent, DamageEvent, PositionLookup, ProductionComplete,
};

/// Ticks per second for the simulation.
pub const TICK_RATE: u32 = 20;

/// Duration of one tick in milliseconds.
pub const TICK_DURATION_MS: u32 = 1000 / TICK_RATE;

/// An entity with optional components.
///
/// Entities are composed of optional components. Only components that are
/// `Some` are active for this entity. This allows flexible entity composition
/// without a full ECS framework.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entity {
    /// Unique identifier for this entity.
    pub id: EntityId,
    /// World position (required for most entities).
    pub position: Option<Position>,
    /// Velocity for movement.
    pub velocity: Option<Velocity>,
    /// Health for damageable entities.
    pub health: Option<Health>,
    /// Command queue for controllable units.
    pub command_queue: Option<CommandQueue>,
    /// Movement capabilities.
    pub movement: Option<Movement>,
    /// Attack target tracking.
    pub attack_target: Option<AttackTarget>,
    /// Combat statistics.
    pub combat_stats: Option<CombatStats>,
    /// Production queue for buildings.
    pub production_queue: Option<ProductionQueue>,
    /// Patrol state for units executing patrol commands.
    pub patrol_state: Option<PatrolState>,
    /// Projectile data for projectile entities.
    pub projectile: Option<Projectile>,
}

impl Entity {
    /// Create a new entity with the given ID and no components.
    #[must_use]
    pub fn new(id: EntityId) -> Self {
        Self {
            id,
            position: None,
            velocity: None,
            health: None,
            command_queue: None,
            movement: None,
            attack_target: None,
            combat_stats: None,
            production_queue: None,
            patrol_state: None,
            projectile: None,
        }
    }
}

/// Parameters for spawning a new entity.
///
/// Use this struct to specify which components the new entity should have.
/// All fields are optional - only provide the components you need.
#[derive(Debug, Clone, Default)]
pub struct EntitySpawnParams {
    /// Initial position in world space.
    pub position: Option<Vec2Fixed>,
    /// Initial velocity.
    pub velocity: Option<Velocity>,
    /// Maximum health (entity starts at full health).
    pub health: Option<u32>,
    /// Movement speed (units per tick).
    pub movement: Option<Fixed>,
    /// Combat statistics.
    pub combat_stats: Option<CombatStats>,
    /// Whether this entity has a production queue.
    pub has_production_queue: bool,
}

/// Storage for all entities in the simulation.
///
/// Uses a `HashMap` for O(1) entity lookup by ID, with deterministic
/// iteration via sorted keys when processing systems.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntityStorage {
    /// Map of entity ID to entity data.
    entities: HashMap<EntityId, Entity>,
    /// Next entity ID to assign.
    next_id: EntityId,
}

impl EntityStorage {
    /// Create empty entity storage.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            next_id: 1,
        }
    }

    /// Insert a new entity and return its ID.
    pub fn insert(&mut self, mut entity: Entity) -> EntityId {
        let id = self.next_id;
        self.next_id += 1;
        entity.id = id;
        self.entities.insert(id, entity);
        id
    }

    /// Remove an entity by ID.
    pub fn remove(&mut self, id: EntityId) -> Option<Entity> {
        self.entities.remove(&id)
    }

    /// Get an entity by ID.
    #[must_use]
    pub fn get(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(&id)
    }

    /// Get a mutable reference to an entity by ID.
    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.entities.get_mut(&id)
    }

    /// Check if an entity exists.
    #[must_use]
    pub fn contains(&self, id: EntityId) -> bool {
        self.entities.contains_key(&id)
    }

    /// Get the number of entities.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Check if storage is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Get sorted entity IDs for deterministic iteration.
    #[must_use]
    pub fn sorted_ids(&self) -> Vec<EntityId> {
        let mut ids: Vec<_> = self.entities.keys().copied().collect();
        ids.sort_unstable();
        ids
    }

    /// Iterate over all entities (not in deterministic order).
    pub fn iter(&self) -> impl Iterator<Item = (&EntityId, &Entity)> {
        self.entities.iter()
    }

    /// Iterate mutably over all entities (not in deterministic order).
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&EntityId, &mut Entity)> {
        self.entities.iter_mut()
    }
}

/// Events generated during a simulation tick.
///
/// These events can be used by the game layer to trigger effects,
/// sounds, animations, etc.
#[derive(Debug, Clone, Default)]
pub struct TickEvents {
    /// Damage events from combat.
    pub damage_events: Vec<DamageEvent>,
    /// Entities that died this tick.
    pub deaths: Vec<EntityId>,
    /// Production completions.
    pub production_complete: Vec<ProductionComplete>,
    /// Entities spawned this tick.
    pub spawned: Vec<EntityId>,
}

/// The core game simulation.
///
/// This struct owns all game state and provides methods
/// to advance the simulation deterministically. The simulation
/// processes systems in a fixed order each tick to ensure
/// identical results across all clients in multiplayer.
///
/// # System Execution Order
///
/// Each tick, systems run in this order:
/// 1. **Command Processing** - Convert player commands to unit actions
/// 2. **Movement** - Update positions based on velocities
/// 3. **Combat** - Process attacks and deal damage
/// 4. **Health** - Check for deaths and remove dead entities
/// 5. **Production** - Advance building production queues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Simulation {
    /// Current simulation tick.
    tick: u64,
    /// All entities in the simulation.
    entities: EntityStorage,
}

impl Simulation {
    /// Create a new empty simulation.
    ///
    /// The simulation starts at tick 0 with no entities.
    ///
    /// # Example
    ///
    /// ```
    /// use rts_core::simulation::Simulation;
    ///
    /// let sim = Simulation::new();
    /// assert_eq!(sim.get_tick(), 0);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            tick: 0,
            entities: EntityStorage::new(),
        }
    }

    /// Get the current tick number.
    ///
    /// The tick counter starts at 0 and increments by 1 each time
    /// [`tick()`](Self::tick) is called.
    #[must_use]
    pub const fn get_tick(&self) -> u64 {
        self.tick
    }

    /// Get a reference to the entity storage.
    #[must_use]
    pub fn entities(&self) -> &EntityStorage {
        &self.entities
    }

    /// Advance the simulation by one tick.
    ///
    /// Runs all systems in deterministic order and increments the tick counter.
    /// Returns events generated during this tick for use by the game layer.
    ///
    /// # System Order
    ///
    /// 1. Command processing (converts commands to velocities)
    /// 2. Movement (applies velocities to positions)
    /// 3. Combat (processes attacks)
    /// 4. Health (removes dead entities)
    /// 5. Production (advances build queues)
    ///
    /// # Example
    ///
    /// ```
    /// use rts_core::simulation::Simulation;
    ///
    /// let mut sim = Simulation::new();
    /// let events = sim.tick();
    /// assert_eq!(sim.get_tick(), 1);
    /// ```
    pub fn tick(&mut self) -> TickEvents {
        let mut events = TickEvents::default();

        // Get sorted entity IDs for deterministic processing
        let entity_ids = self.entities.sorted_ids();

        // 1. Command Processing System
        self.run_command_processing_system(&entity_ids);

        // 1.5 Patrol System
        self.run_patrol_system(&entity_ids);

        // 1.6 Attack Chase System
        self.run_attack_chase_system(&entity_ids);

        // 2. Movement System
        self.run_movement_system(&entity_ids);

        // 3. Combat System
        events.damage_events = self.run_combat_system(&entity_ids);

        // 3.5 Projectile System
        let mut projectile_damage = self.run_projectile_system(&entity_ids);
        events.damage_events.append(&mut projectile_damage);

        // 4. Health System - identify and remove dead entities
        events.deaths = self.run_health_system(&entity_ids);
        for dead_id in &events.deaths {
            self.entities.remove(*dead_id);
        }

        // 5. Production System
        events.production_complete = self.run_production_system(&entity_ids);

        // Increment tick counter
        self.tick += 1;

        #[cfg(debug_assertions)]
        {
            let hash = self.state_hash();
            tracing::debug!(tick = self.tick, state_hash = hash, "Simulation state hash");
        }

        events
    }

    /// Run the command processing system on all applicable entities.
    fn run_command_processing_system(&mut self, entity_ids: &[EntityId]) {
        // Process each entity with required components
        for &id in entity_ids {
            if let Some(entity) = self.entities.get_mut(id) {
                // Check if entity has all required components
                let has_all = entity.command_queue.is_some()
                    && entity.position.is_some()
                    && entity.velocity.is_some()
                    && entity.movement.is_some();

                if has_all {
                    // Process this entity individually
                    let command_queue = entity.command_queue.as_mut().unwrap();
                    let position = entity.position.as_ref().unwrap();
                    let velocity = entity.velocity.as_mut().unwrap();
                    let movement = entity.movement.as_ref().unwrap();

                    let mut single = vec![(id, command_queue, position, velocity, movement)];
                    command_processing_system(&mut single);
                }
            }
        }
    }

    /// Run patrol movement logic for entities with patrol commands.
    fn run_patrol_system(&mut self, entity_ids: &[EntityId]) {
        let arrival_threshold_sq = Fixed::from_num(1);

        for &id in entity_ids {
            let Some(entity) = self.entities.get_mut(id) else {
                continue;
            };

            let Some(command_queue) = entity.command_queue.as_mut() else {
                continue;
            };

            let Some(position) = entity.position.as_ref() else {
                continue;
            };

            let Some(velocity) = entity.velocity.as_mut() else {
                continue;
            };

            let Some(movement) = entity.movement.as_ref() else {
                continue;
            };

            match command_queue.current() {
                Some(Command::Patrol(target)) => {
                    let target = *target;
                    let mut state = entity.patrol_state.unwrap_or(PatrolState {
                        origin: position.value,
                        target,
                        heading_to_target: true,
                    });

                    if state.target != target {
                        state = PatrolState {
                            origin: position.value,
                            target,
                            heading_to_target: true,
                        };
                    }

                    let desired = if state.heading_to_target {
                        state.target
                    } else {
                        state.origin
                    };

                    let dist_sq = position.value.distance_squared(desired);
                    if dist_sq <= arrival_threshold_sq {
                        state.heading_to_target = !state.heading_to_target;
                        velocity.value = Vec2Fixed::ZERO;
                    } else {
                        let diff = desired - position.value;
                        let direction = crate::systems::normalize_vec2(diff);
                        velocity.value = Vec2Fixed::new(
                            direction.x * movement.speed,
                            direction.y * movement.speed,
                        );
                    }

                    entity.patrol_state = Some(state);
                }
                _ => {
                    entity.patrol_state = None;
                }
            }
        }
    }

    /// Run attack chase logic for entities with attack commands.
    fn run_attack_chase_system(&mut self, entity_ids: &[EntityId]) {
        let arrival_threshold_sq = Fixed::from_num(1);

        for &id in entity_ids {
            let Some(Command::Attack(target_id)) = self
                .entities
                .get(id)
                .and_then(|entity| entity.command_queue.as_ref())
                .and_then(|queue| queue.current().cloned())
            else {
                continue;
            };

            let Some(target_pos) = self
                .entities
                .get(target_id)
                .and_then(|target| target.position.map(|pos| pos.value))
            else {
                if let Some(entity) = self.entities.get_mut(id) {
                    if let Some(command_queue) = entity.command_queue.as_mut() {
                        command_queue.pop();
                    }
                    if let Some(velocity) = entity.velocity.as_mut() {
                        velocity.value = Vec2Fixed::ZERO;
                    }
                }
                continue;
            };

            let Some(entity) = self.entities.get_mut(id) else {
                continue;
            };

            let Some(command_queue) = entity.command_queue.as_mut() else {
                continue;
            };

            let Some(position) = entity.position.as_ref() else {
                continue;
            };

            let Some(velocity) = entity.velocity.as_mut() else {
                continue;
            };

            let Some(movement) = entity.movement.as_ref() else {
                continue;
            };

            if command_queue.current().cloned() != Some(Command::Attack(target_id)) {
                velocity.value = Vec2Fixed::ZERO;
                continue;
            };

            let dist_sq = position.value.distance_squared(target_pos);
            if dist_sq <= arrival_threshold_sq {
                velocity.value = Vec2Fixed::ZERO;
            } else {
                let diff = target_pos - position.value;
                let direction = crate::systems::normalize_vec2(diff);
                velocity.value =
                    Vec2Fixed::new(direction.x * movement.speed, direction.y * movement.speed);
            }
        }
    }

    /// Run the movement system on all applicable entities.
    fn run_movement_system(&mut self, entity_ids: &[EntityId]) {
        for &id in entity_ids {
            if let Some(entity) = self.entities.get_mut(id) {
                if let (Some(position), Some(velocity)) =
                    (entity.position.as_mut(), entity.velocity.as_ref())
                {
                    let mut single = vec![(id, position, velocity)];
                    movement_system(&mut single);
                }
            }
        }
    }

    /// Run the combat system on all applicable entities.
    fn run_combat_system(&mut self, entity_ids: &[EntityId]) -> Vec<DamageEvent> {
        // Build position lookup
        let positions: Vec<(EntityId, Position)> = entity_ids
            .iter()
            .filter_map(|&id| {
                self.entities
                    .get(id)
                    .and_then(|e| e.position.map(|p| (id, p)))
            })
            .collect();
        let pos_lookup = PositionLookup::new(&positions);

        let mut all_damage_events = Vec::new();

        // Process attackers one at a time to avoid borrow issues
        for &attacker_id in entity_ids {
            let attacker_data = {
                let entity = match self.entities.get(attacker_id) {
                    Some(e) => e,
                    None => continue,
                };

                // Check if this entity can attack
                let position = match &entity.position {
                    Some(p) => *p,
                    None => continue,
                };
                let attack_target = match &entity.attack_target {
                    Some(t) => *t,
                    None => continue,
                };
                let combat_stats = match &entity.combat_stats {
                    Some(s) => *s,
                    None => continue,
                };

                (position, attack_target, combat_stats)
            };

            let (position, mut attack_target, mut combat_stats) = attacker_data;

            // Find target and deal damage
            if let Some(target_id) = attack_target.target {
                // Tick down cooldown
                if combat_stats.cooldown_remaining > 0 {
                    combat_stats.cooldown_remaining -= 1;
                }

                // Get target position
                if let Some(target_pos) = pos_lookup.get(target_id) {
                    // Check range
                    let range_sq = combat_stats.range * combat_stats.range;
                    let dist_sq = position.value.distance_squared(target_pos.value);

                    if dist_sq <= range_sq && combat_stats.cooldown_remaining == 0 {
                        if combat_stats.uses_projectiles() {
                            let projectile = Projectile::new(
                                attacker_id,
                                target_id,
                                combat_stats.damage,
                                combat_stats.damage_type,
                                combat_stats.projectile_speed,
                            );
                            self.spawn_projectile(position.value, projectile);
                            combat_stats.cooldown_remaining = combat_stats.attack_cooldown;
                        } else if let Some(target_entity) = self.entities.get_mut(target_id) {
                            if let (Some(ref mut health), Some(target_stats)) =
                                (target_entity.health.as_mut(), target_entity.combat_stats)
                            {
                                let damage = calculate_damage(
                                    combat_stats.damage,
                                    combat_stats.damage_type,
                                    target_stats.armor_type,
                                    target_stats.armor_value,
                                );
                                health.apply_damage(damage);

                                all_damage_events.push(DamageEvent {
                                    attacker: attacker_id,
                                    target: target_id,
                                    damage,
                                });

                                // Reset cooldown
                                combat_stats.cooldown_remaining = combat_stats.attack_cooldown;

                                // Clear target if dead
                                if health.is_dead() {
                                    attack_target.clear();
                                }
                            }
                        }
                    }
                } else {
                    // Target doesn't exist
                    attack_target.clear();
                }
            }

            // Update attacker's components
            if let Some(entity) = self.entities.get_mut(attacker_id) {
                entity.attack_target = Some(attack_target);
                entity.combat_stats = Some(combat_stats);
            }
        }

        all_damage_events
    }

    /// Run the projectile system on all active projectiles.
    fn run_projectile_system(&mut self, entity_ids: &[EntityId]) -> Vec<DamageEvent> {
        let positions: Vec<(EntityId, Position)> = entity_ids
            .iter()
            .filter_map(|&id| {
                self.entities
                    .get(id)
                    .and_then(|e| e.position.map(|p| (id, p)))
            })
            .collect();
        let pos_lookup = PositionLookup::new(&positions);

        let mut projectile_data: Vec<(EntityId, Position, Projectile)> = entity_ids
            .iter()
            .filter_map(|&id| {
                self.entities
                    .get(id)
                    .and_then(|entity| Some((id, entity.position?, entity.projectile?)))
            })
            .collect();

        if projectile_data.is_empty() {
            return Vec::new();
        }

        let mut target_data: Vec<(EntityId, Health, CombatStats)> = entity_ids
            .iter()
            .filter_map(|&id| {
                self.entities
                    .get(id)
                    .and_then(|entity| Some((id, entity.health?, entity.combat_stats?)))
            })
            .collect();

        let mut projectile_refs: Vec<(EntityId, &mut Position, &Projectile)> = projectile_data
            .iter_mut()
            .map(|(id, position, projectile)| (*id, position, &*projectile))
            .collect();

        let mut target_refs: Vec<(EntityId, &mut Health, &CombatStats)> = target_data
            .iter_mut()
            .map(|(id, health, combat_stats)| (*id, health, &*combat_stats))
            .collect();

        let updates =
            crate::systems::projectile_system(&mut projectile_refs, &mut target_refs, &pos_lookup);

        let mut position_map: std::collections::HashMap<EntityId, Position> = projectile_data
            .iter()
            .map(|(id, position, _)| (*id, *position))
            .collect();

        let mut damage_events = Vec::new();

        for update in updates {
            if update.hit {
                if let Some(CombatEvent::ProjectileHit {
                    source,
                    target,
                    damage,
                }) = update.event
                {
                    damage_events.push(DamageEvent {
                        attacker: source,
                        target,
                        damage,
                    });
                    if let Some(entity) = self.entities.get_mut(target) {
                        if let Some(health) = entity.health.as_mut() {
                            health.apply_damage(damage);
                        }
                    }
                }
                self.entities.remove(update.projectile_id);
            } else if let Some(new_pos) = position_map.remove(&update.projectile_id) {
                if let Some(entity) = self.entities.get_mut(update.projectile_id) {
                    entity.position = Some(new_pos);
                }
            }
        }

        damage_events
    }

    /// Run the health system and return dead entity IDs.
    fn run_health_system(&self, entity_ids: &[EntityId]) -> Vec<EntityId> {
        let health_data: Vec<(EntityId, &Health)> = entity_ids
            .iter()
            .filter_map(|&id| {
                self.entities
                    .get(id)
                    .and_then(|e| e.health.as_ref().map(|h| (id, h)))
            })
            .collect();

        health_system(&health_data)
    }

    /// Run the production system and return completed productions.
    fn run_production_system(&mut self, entity_ids: &[EntityId]) -> Vec<ProductionComplete> {
        let mut completions = Vec::new();

        for &id in entity_ids {
            if let Some(entity) = self.entities.get_mut(id) {
                if let (Some(position), Some(queue)) =
                    (entity.position.as_ref(), entity.production_queue.as_mut())
                {
                    let mut single = vec![(id, position, queue)];
                    let mut results = production_system(&mut single);
                    completions.append(&mut results);
                }
            }
        }

        completions
    }

    /// Spawn a projectile entity at the given position.
    fn spawn_projectile(&mut self, position: Vec2Fixed, projectile: Projectile) -> EntityId {
        let mut entity = Entity::new(0);
        entity.position = Some(Position::new(position));
        entity.projectile = Some(projectile);
        self.entities.insert(entity)
    }

    /// Spawn a new entity with the given parameters.
    ///
    /// Returns the unique ID assigned to the new entity.
    ///
    /// # Arguments
    ///
    /// * `params` - Configuration for the new entity's components
    ///
    /// # Example
    ///
    /// ```
    /// use rts_core::simulation::{Simulation, EntitySpawnParams};
    /// use rts_core::math::{Fixed, Vec2Fixed};
    ///
    /// let mut sim = Simulation::new();
    /// let unit = sim.spawn_entity(EntitySpawnParams {
    ///     position: Some(Vec2Fixed::new(Fixed::from_num(100), Fixed::from_num(50))),
    ///     health: Some(100),
    ///     movement: Some(Fixed::from_num(2)),
    ///     ..Default::default()
    /// });
    /// ```
    pub fn spawn_entity(&mut self, params: EntitySpawnParams) -> EntityId {
        let mut entity = Entity::new(0); // ID will be assigned by storage

        if let Some(pos) = params.position {
            entity.position = Some(Position::new(pos));
        }

        if let Some(vel) = params.velocity {
            entity.velocity = Some(vel);
        } else if params.position.is_some() {
            // Default to zero velocity for positioned entities
            entity.velocity = Some(Velocity::ZERO);
        }

        if let Some(max_health) = params.health {
            entity.health = Some(Health::new(max_health));
        }

        if let Some(speed) = params.movement {
            entity.movement = Some(Movement {
                speed,
                target: None,
            });
            // Units with movement get a command queue
            entity.command_queue = Some(CommandQueue::new());
        }

        if let Some(stats) = params.combat_stats {
            entity.combat_stats = Some(stats);
            entity.attack_target = Some(AttackTarget::new());
        }

        if params.has_production_queue {
            entity.production_queue = Some(ProductionQueue::new());
        }

        self.entities.insert(entity)
    }

    /// Remove an entity from the simulation.
    ///
    /// Returns `Ok(())` if the entity was removed, or an error if it didn't exist.
    ///
    /// # Arguments
    ///
    /// * `id` - The entity ID to remove
    ///
    /// # Errors
    ///
    /// Returns [`GameError::EntityNotFound`] if the entity doesn't exist.
    ///
    /// # Example
    ///
    /// ```
    /// use rts_core::simulation::{Simulation, EntitySpawnParams};
    ///
    /// let mut sim = Simulation::new();
    /// let id = sim.spawn_entity(EntitySpawnParams::default());
    /// sim.despawn_entity(id).unwrap();
    /// ```
    pub fn despawn_entity(&mut self, id: EntityId) -> Result<()> {
        if self.entities.remove(id).is_some() {
            Ok(())
        } else {
            Err(GameError::EntityNotFound(id))
        }
    }

    /// Queue a command for an entity.
    ///
    /// The command is added to the entity's command queue and will be
    /// processed on subsequent ticks.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to command
    /// * `command` - The command to execute
    ///
    /// # Errors
    ///
    /// Returns [`GameError::EntityNotFound`] if the entity doesn't exist,
    /// or [`GameError::InvalidState`] if the entity has no command queue.
    ///
    /// # Example
    ///
    /// ```
    /// use rts_core::simulation::{Simulation, EntitySpawnParams};
    /// use rts_core::components::Command;
    /// use rts_core::math::{Fixed, Vec2Fixed};
    ///
    /// let mut sim = Simulation::new();
    /// let unit = sim.spawn_entity(EntitySpawnParams {
    ///     position: Some(Vec2Fixed::ZERO),
    ///     movement: Some(Fixed::from_num(2)),
    ///     ..Default::default()
    /// });
    ///
    /// sim.apply_command(unit, Command::MoveTo(Vec2Fixed::new(
    ///     Fixed::from_num(50),
    ///     Fixed::from_num(50),
    /// ))).unwrap();
    /// ```
    pub fn apply_command(&mut self, entity: EntityId, command: Command) -> Result<()> {
        let ent = self
            .entities
            .get_mut(entity)
            .ok_or(GameError::EntityNotFound(entity))?;

        let queue = ent.command_queue.as_mut().ok_or_else(|| {
            GameError::InvalidState(format!("Entity {} has no command queue", entity))
        })?;

        queue.set(command);
        Ok(())
    }

    /// Queue a command without clearing existing commands.
    ///
    /// Unlike [`apply_command`](Self::apply_command), this adds the command
    /// to the back of the queue rather than replacing all existing commands.
    ///
    /// # Errors
    ///
    /// Same as [`apply_command`](Self::apply_command).
    pub fn queue_command(&mut self, entity: EntityId, command: Command) -> Result<()> {
        let ent = self
            .entities
            .get_mut(entity)
            .ok_or(GameError::EntityNotFound(entity))?;

        let queue = ent.command_queue.as_mut().ok_or_else(|| {
            GameError::InvalidState(format!("Entity {} has no command queue", entity))
        })?;

        queue.push(command);
        Ok(())
    }

    /// Set an entity's attack target.
    ///
    /// # Errors
    ///
    /// Returns an error if the entity doesn't exist or has no combat capability.
    pub fn set_attack_target(&mut self, entity: EntityId, target: EntityId) -> Result<()> {
        let ent = self
            .entities
            .get_mut(entity)
            .ok_or(GameError::EntityNotFound(entity))?;

        let attack_target = ent
            .attack_target
            .as_mut()
            .ok_or_else(|| GameError::InvalidState(format!("Entity {} cannot attack", entity)))?;

        attack_target.target = Some(target);
        Ok(())
    }

    /// Get an entity by ID.
    #[must_use]
    pub fn get_entity(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(id)
    }

    /// Calculate a hash of the current simulation state.
    ///
    /// Used for desync detection in multiplayer. Two simulations
    /// with identical state will produce identical hashes.
    #[must_use]
    pub fn state_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        // Hash tick
        self.tick.hash(&mut hasher);

        // Hash entities in deterministic order
        let ids = self.entities.sorted_ids();
        ids.len().hash(&mut hasher);

        for id in ids {
            if let Some(entity) = self.entities.get(id) {
                id.hash(&mut hasher);

                // Hash position
                if let Some(ref pos) = entity.position {
                    pos.value.x.to_bits().hash(&mut hasher);
                    pos.value.y.to_bits().hash(&mut hasher);
                }

                // Hash health
                if let Some(ref health) = entity.health {
                    health.current.hash(&mut hasher);
                    health.max.hash(&mut hasher);
                }

                // Hash velocity
                if let Some(ref vel) = entity.velocity {
                    vel.value.x.to_bits().hash(&mut hasher);
                    vel.value.y.to_bits().hash(&mut hasher);
                }

                // Hash projectile
                if let Some(ref projectile) = entity.projectile {
                    projectile.source.hash(&mut hasher);
                    projectile.target.hash(&mut hasher);
                    projectile.damage.hash(&mut hasher);
                    projectile.damage_type.hash(&mut hasher);
                    projectile.speed.to_bits().hash(&mut hasher);
                }

                // Hash patrol state
                if let Some(ref patrol) = entity.patrol_state {
                    patrol.origin.x.to_bits().hash(&mut hasher);
                    patrol.origin.y.to_bits().hash(&mut hasher);
                    patrol.target.x.to_bits().hash(&mut hasher);
                    patrol.target.y.to_bits().hash(&mut hasher);
                    patrol.heading_to_target.hash(&mut hasher);
                }
            }
        }

        hasher.finish()
    }

    /// Serialize the simulation state for replay or network sync.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn serialize(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| GameError::InvalidState(format!("Failed to serialize simulation: {}", e)))
    }

    /// Deserialize simulation state from bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails.
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data).map_err(|e| {
            GameError::InvalidState(format!("Failed to deserialize simulation: {}", e))
        })
    }
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_new() {
        let sim = Simulation::new();
        assert_eq!(sim.get_tick(), 0);
        assert!(sim.entities.is_empty());
    }

    #[test]
    fn test_spawn_entity() {
        let mut sim = Simulation::new();
        let id = sim.spawn_entity(EntitySpawnParams {
            position: Some(Vec2Fixed::new(Fixed::from_num(10), Fixed::from_num(20))),
            health: Some(100),
            ..Default::default()
        });

        assert_eq!(id, 1);
        let entity = sim.get_entity(id).unwrap();
        assert_eq!(entity.position.unwrap().value.x, Fixed::from_num(10));
        assert_eq!(entity.health.unwrap().current, 100);
    }

    #[test]
    fn test_despawn_entity() {
        let mut sim = Simulation::new();
        let id = sim.spawn_entity(EntitySpawnParams::default());

        assert!(sim.despawn_entity(id).is_ok());
        assert!(sim.get_entity(id).is_none());
        assert!(sim.despawn_entity(id).is_err());
    }

    #[test]
    fn test_tick_increments() {
        let mut sim = Simulation::new();
        assert_eq!(sim.get_tick(), 0);

        sim.tick();
        assert_eq!(sim.get_tick(), 1);

        sim.tick();
        assert_eq!(sim.get_tick(), 2);
    }

    #[test]
    fn test_movement_integration() {
        let mut sim = Simulation::new();
        let id = sim.spawn_entity(EntitySpawnParams {
            position: Some(Vec2Fixed::new(Fixed::from_num(0), Fixed::from_num(0))),
            movement: Some(Fixed::from_num(5)),
            ..Default::default()
        });

        // Issue move command
        sim.apply_command(
            id,
            Command::MoveTo(Vec2Fixed::new(Fixed::from_num(100), Fixed::from_num(0))),
        )
        .unwrap();

        // Run a tick
        sim.tick();

        // Entity should have moved
        let entity = sim.get_entity(id).unwrap();
        let pos = entity.position.unwrap();
        assert!(pos.value.x > Fixed::from_num(0));
    }

    #[test]
    fn test_patrol_toggles_heading() {
        let mut sim = Simulation::new();
        let id = sim.spawn_entity(EntitySpawnParams {
            position: Some(Vec2Fixed::ZERO),
            movement: Some(Fixed::from_num(2)),
            ..Default::default()
        });

        let target = Vec2Fixed::new(Fixed::from_num(10), Fixed::from_num(0));
        sim.apply_command(id, Command::Patrol(target)).unwrap();

        sim.tick();
        let state = sim.get_entity(id).unwrap().patrol_state.unwrap();
        assert!(state.heading_to_target);

        if let Some(entity) = sim.entities.get_mut(id) {
            entity.position = Some(Position::new(target));
        }

        sim.tick();
        let state = sim.get_entity(id).unwrap().patrol_state.unwrap();
        assert!(!state.heading_to_target);
    }

    #[test]
    fn test_attack_command_chases_target() {
        let mut sim = Simulation::new();
        let attacker = sim.spawn_entity(EntitySpawnParams {
            position: Some(Vec2Fixed::ZERO),
            movement: Some(Fixed::from_num(3)),
            ..Default::default()
        });

        let target = sim.spawn_entity(EntitySpawnParams {
            position: Some(Vec2Fixed::new(Fixed::from_num(20), Fixed::from_num(0))),
            ..Default::default()
        });

        sim.apply_command(attacker, Command::Attack(target))
            .unwrap();
        sim.tick();

        let entity = sim.get_entity(attacker).unwrap();
        let pos = entity.position.unwrap();
        assert!(pos.value.x > Fixed::from_num(0));
    }

    #[test]
    fn test_deterministic_hash() {
        let mut sim1 = Simulation::new();
        let mut sim2 = Simulation::new();

        // Same operations
        let id1 = sim1.spawn_entity(EntitySpawnParams {
            position: Some(Vec2Fixed::new(Fixed::from_num(10), Fixed::from_num(20))),
            health: Some(100),
            movement: Some(Fixed::from_num(2)),
            ..Default::default()
        });
        let id2 = sim2.spawn_entity(EntitySpawnParams {
            position: Some(Vec2Fixed::new(Fixed::from_num(10), Fixed::from_num(20))),
            health: Some(100),
            movement: Some(Fixed::from_num(2)),
            ..Default::default()
        });

        sim1.apply_command(
            id1,
            Command::MoveTo(Vec2Fixed::new(Fixed::from_num(50), Fixed::from_num(50))),
        )
        .unwrap();
        sim2.apply_command(
            id2,
            Command::MoveTo(Vec2Fixed::new(Fixed::from_num(50), Fixed::from_num(50))),
        )
        .unwrap();

        sim1.tick();
        sim2.tick();

        // Hashes must be identical
        assert_eq!(sim1.state_hash(), sim2.state_hash());
    }

    #[test]
    fn test_projectile_hits_target() {
        let mut sim = Simulation::new();
        let attacker = sim.spawn_entity(EntitySpawnParams {
            position: Some(Vec2Fixed::ZERO),
            health: Some(100),
            movement: Some(Fixed::from_num(1)),
            combat_stats: Some(
                CombatStats::new(10, Fixed::from_num(10), 10)
                    .with_projectile_speed(Fixed::from_num(10)),
            ),
            ..Default::default()
        });

        let target = sim.spawn_entity(EntitySpawnParams {
            position: Some(Vec2Fixed::new(Fixed::from_num(5), Fixed::from_num(0))),
            health: Some(5),
            combat_stats: Some(CombatStats::default()),
            ..Default::default()
        });

        sim.set_attack_target(attacker, target).unwrap();

        sim.tick();
        sim.tick();

        if let Some(entity) = sim.get_entity(target) {
            let target_health = entity.health.unwrap().current;
            assert!(target_health < 5);
        }

        let has_projectiles = sim
            .entities()
            .iter()
            .any(|(_, entity)| entity.projectile.is_some());
        assert!(!has_projectiles);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut sim = Simulation::new();
        sim.spawn_entity(EntitySpawnParams {
            position: Some(Vec2Fixed::new(Fixed::from_num(42), Fixed::from_num(24))),
            health: Some(75),
            ..Default::default()
        });
        sim.tick();

        let bytes = sim.serialize().unwrap();
        let restored = Simulation::deserialize(&bytes).unwrap();

        assert_eq!(sim.get_tick(), restored.get_tick());
        assert_eq!(sim.state_hash(), restored.state_hash());
    }

    #[test]
    fn test_health_system_removes_dead() {
        let mut sim = Simulation::new();
        let id = sim.spawn_entity(EntitySpawnParams {
            position: Some(Vec2Fixed::ZERO),
            health: Some(10),
            ..Default::default()
        });

        // Manually set health to 0
        if let Some(entity) = sim.entities.get_mut(id) {
            if let Some(ref mut health) = entity.health {
                health.current = 0;
            }
        }

        let events = sim.tick();
        assert!(events.deaths.contains(&id));
        assert!(sim.get_entity(id).is_none());
    }
}
