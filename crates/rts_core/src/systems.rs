//! Simulation systems.
//!
//! Systems contain the logic that processes components.
//! Each system should do one thing well.
//!
//! All systems are pure functions that operate on component data.
//! They use fixed-point math for deterministic simulation.

use crate::combat::calculate_resistance_damage;
use crate::components::{
    ArmorType, AttackTarget, CombatStats, Command, CommandQueue, DamageType, EntityId, Health,
    Movement, Position, Projectile, Velocity,
};
use crate::math::{Fixed, Vec2Fixed};

/// Updates entity positions based on their velocities.
///
/// This is the core movement integration step. Each entity's position
/// is updated by adding its velocity (which represents units per tick).
///
/// # Arguments
/// * `entities` - Slice of entities with mutable positions and read-only velocities
///
/// # Example
/// ```ignore
/// let mut entities = vec![
///     (1, &mut position, &velocity),
/// ];
/// movement_system(&mut entities);
/// ```
pub fn movement_system(entities: &mut [(EntityId, &mut Position, &Velocity)]) {
    for (_entity_id, position, velocity) in entities.iter_mut() {
        position.value = position.value + velocity.value;
    }
}

/// Processes command queues and converts commands to movement velocity.
///
/// Examines the current command for each entity and sets appropriate velocity:
/// - `MoveTo`: Calculates direction toward target, sets velocity based on movement speed
/// - `Stop`: Sets velocity to zero
/// - `HoldPosition`: Sets velocity to zero
/// - Other commands: No velocity change (handled by other systems)
///
/// When a `MoveTo` command reaches its destination (within a small threshold),
/// the command is popped from the queue.
///
/// # Arguments
/// * `entities` - Slice of entities with their command queues, positions, velocities, movement stats, and path waypoints
pub fn command_processing_system(
    entities: &mut [(
        EntityId,
        &mut CommandQueue,
        &Position,
        &mut Velocity,
        &Movement,
        &mut Option<Vec<Vec2Fixed>>,
    )],
) {
    // Threshold for considering arrival at destination (squared distance)
    let arrival_threshold_sq = Fixed::from_num(1);

    for (_entity_id, command_queue, position, velocity, movement, path_waypoints) in
        entities.iter_mut()
    {
        match command_queue.current() {
            Some(Command::MoveTo(target)) => {
                // If we have waypoints, follow them; otherwise go directly to target
                let next_target = if let Some(waypoints) = path_waypoints.as_mut() {
                    if let Some(first) = waypoints.first() {
                        *first
                    } else {
                        // No waypoints left, go to final target
                        *target
                    }
                } else {
                    *target
                };

                let diff = next_target - position.value;
                let dist_sq = position.value.distance_squared(next_target);

                // Check if we've arrived at current waypoint/target
                if dist_sq <= arrival_threshold_sq {
                    // Check if we have more waypoints
                    if let Some(waypoints) = path_waypoints.as_mut() {
                        if !waypoints.is_empty() {
                            // Remove first waypoint, continue to next
                            waypoints.remove(0);
                            if waypoints.is_empty() {
                                // No more waypoints, check if at final target
                                let final_dist_sq = position.value.distance_squared(*target);
                                if final_dist_sq <= arrival_threshold_sq {
                                    velocity.value = Vec2Fixed::ZERO;
                                    command_queue.pop();
                                    **path_waypoints = None;
                                }
                            }
                            continue;
                        }
                    }
                    // Arrived at final destination
                    velocity.value = Vec2Fixed::ZERO;
                    command_queue.pop();
                    **path_waypoints = None;
                } else {
                    // Calculate direction and set velocity
                    let direction = normalize_vec2(diff);
                    velocity.value =
                        Vec2Fixed::new(direction.x * movement.speed, direction.y * movement.speed);
                }
            }
            Some(Command::Stop) => {
                velocity.value = Vec2Fixed::ZERO;
                **path_waypoints = None;
                command_queue.pop();
            }
            Some(Command::HoldPosition) => {
                velocity.value = Vec2Fixed::ZERO;
                **path_waypoints = None;
                // HoldPosition stays active (don't pop)
            }
            Some(Command::AttackMove(target)) => {
                // Move toward position (attack logic handled by combat system)
                // TODO: Implement pathfinding for AttackMove
                let target = *target;
                let diff = target - position.value;
                let dist_sq = position.value.distance_squared(target);

                if dist_sq <= arrival_threshold_sq {
                    velocity.value = Vec2Fixed::ZERO;
                    command_queue.pop();
                } else {
                    let direction = normalize_vec2(diff);
                    velocity.value =
                        Vec2Fixed::new(direction.x * movement.speed, direction.y * movement.speed);
                }
            }
            Some(Command::Patrol(_)) | Some(Command::Follow(_)) | Some(Command::Guard(_)) => {
                // These require additional state tracking - placeholder for now
            }
            Some(Command::Attack(_)) => {
                // Attack command: movement handled by combat system based on range
            }
            None => {
                // No command - stop moving
                velocity.value = Vec2Fixed::ZERO;
            }
        }
    }
}

/// Processes health and identifies dead entities for removal.
///
/// Scans all entities with health components and returns a list of
/// entity IDs that should be removed (health <= 0).
///
/// # Arguments
/// * `entities` - Slice of entities with their health components
///
/// # Returns
/// Vector of entity IDs that are dead and should be removed from the simulation
pub fn health_system(entities: &[(EntityId, &Health)]) -> Vec<EntityId> {
    entities
        .iter()
        .filter(|(_id, health)| health.is_dead())
        .map(|(id, _)| *id)
        .collect()
}

/// Position lookup helper for combat system.
pub struct PositionLookup<'a> {
    positions: &'a [(EntityId, Position)],
}

impl<'a> PositionLookup<'a> {
    /// Create a new position lookup from a slice of entity positions.
    pub fn new(positions: &'a [(EntityId, Position)]) -> Self {
        Self { positions }
    }

    /// Get the position of an entity by ID.
    pub fn get(&self, entity_id: EntityId) -> Option<Position> {
        self.positions
            .iter()
            .find(|(id, _)| *id == entity_id)
            .map(|(_, pos)| *pos)
    }
}

// ============================================================================
// Combat Events
// ============================================================================

/// Events generated by the combat system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombatEvent {
    /// An attack has started (for animations/sound).
    AttackStarted {
        /// Entity performing the attack.
        attacker: EntityId,
        /// Entity being attacked.
        target: EntityId,
        /// Type of damage being dealt.
        damage_type: DamageType,
    },
    /// Damage was dealt to a target.
    DamageDealt {
        /// Entity that dealt the damage.
        attacker: EntityId,
        /// Entity that received damage.
        target: EntityId,
        /// Damage before modifiers.
        base_damage: u32,
        /// Damage after type/armor modifiers.
        final_damage: u32,
        /// Type of damage dealt.
        damage_type: DamageType,
    },
    /// A unit was killed.
    UnitKilled {
        /// Entity responsible for the kill.
        killer: EntityId,
        /// Entity that died.
        victim: EntityId,
    },
    /// A projectile was spawned.
    ProjectileSpawned {
        /// Entity that fired the projectile.
        source: EntityId,
        /// Target entity for the projectile.
        target: EntityId,
        /// Damage the projectile will deal.
        damage: u32,
        /// Type of damage.
        damage_type: DamageType,
        /// Starting position of the projectile.
        start_pos: Vec2Fixed,
        /// Travel speed per tick.
        speed: Fixed,
    },
    /// A projectile hit its target.
    ProjectileHit {
        /// Entity that fired the projectile.
        source: EntityId,
        /// Entity that was hit.
        target: EntityId,
        /// Damage dealt.
        damage: u32,
    },
    /// Target is out of range.
    OutOfRange {
        /// Entity trying to attack.
        attacker: EntityId,
        /// Entity being targeted.
        target: EntityId,
        /// Current distance to target.
        distance: Fixed,
        /// Attack range of the attacker.
        range: Fixed,
    },
}

/// Result of combat processing for a single attack (legacy compatibility).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DamageEvent {
    /// The entity dealing damage.
    pub attacker: EntityId,
    /// The entity receiving damage.
    pub target: EntityId,
    /// Amount of damage dealt.
    pub damage: u32,
}

// ============================================================================
// Damage Calculation
// ============================================================================

/// Calculate final damage after applying damage type vs armor type modifiers.
///
/// The damage calculation follows these steps:
/// 1. Apply damage type effectiveness multiplier
/// 2. Subtract flat armor reduction
/// 3. Ensure minimum 1 damage (unless multiplier is 0%)
///
/// All calculations use fixed-point math for determinism.
#[must_use]
pub fn calculate_damage(
    base_damage: u32,
    damage_type: DamageType,
    armor_type: ArmorType,
    armor_value: u32,
) -> u32 {
    let multiplier = damage_type.effectiveness_vs(armor_type);

    // If multiplier is zero (e.g., biological vs building), no damage
    if multiplier == Fixed::ZERO {
        return 0;
    }

    // Apply multiplier (both base_damage and multiplier are non-negative)
    let damage_fixed = Fixed::from_num(base_damage) * multiplier;

    // Convert back to u32 (truncate toward zero, always non-negative)
    let modified_damage: u32 = damage_fixed.to_num();

    // Subtract armor (minimum 1 damage if not immune)
    if modified_damage <= armor_value {
        1 // Minimum damage
    } else {
        modified_damage - armor_value
    }
}

// ============================================================================
// Combat System
// ============================================================================

/// Target information for combat lookups.
pub struct TargetInfo {
    /// Entity ID.
    pub id: EntityId,
    /// Position of the target.
    pub position: Position,
    /// Armor type of the target.
    pub armor_type: ArmorType,
    /// Flat armor reduction value.
    pub armor_value: u32,
}

/// Handles attack logic for combat units with damage types and armor.
///
/// For each attacking entity:
/// 1. Ticks down attack cooldown
/// 2. Checks if target exists and is in range
/// 3. When cooldown is ready and target is in range:
///    - For hitscan weapons: deals damage immediately
///    - For projectile weapons: spawns a projectile
/// 4. Calculates damage with type effectiveness and armor reduction
///
/// # Arguments
/// * `attackers` - Entities with attack capability
/// * `targets` - Health and combat stats of potential targets
/// * `positions` - Position lookup for range checking
///
/// # Returns
/// Tuple of (damage events for legacy compat, full combat events, projectile spawn data)
pub fn combat_system(
    attackers: &mut [(EntityId, &Position, &mut AttackTarget, &mut CombatStats)],
    targets: &mut [(EntityId, &mut Health, &CombatStats)],
    positions: &PositionLookup<'_>,
) -> (Vec<DamageEvent>, Vec<CombatEvent>) {
    let mut damage_events = Vec::new();
    let mut combat_events = Vec::new();

    for (attacker_id, attacker_pos, attack_target, combat_stats) in attackers.iter_mut() {
        // Tick down cooldown
        combat_stats.tick_cooldown();

        // Check if we have a target
        let Some(target_id) = attack_target.target else {
            continue;
        };

        // Get target position
        let Some(target_pos) = positions.get(target_id) else {
            // Target doesn't exist - clear it
            attack_target.clear();
            continue;
        };

        // Check range (using squared distance to avoid sqrt)
        let range_sq = combat_stats.range * combat_stats.range;
        let dist_sq = attacker_pos.value.distance_squared(target_pos.value);

        if dist_sq > range_sq {
            // Target out of range
            let distance = fixed_sqrt(dist_sq);
            combat_events.push(CombatEvent::OutOfRange {
                attacker: *attacker_id,
                target: target_id,
                distance,
                range: combat_stats.range,
            });
            continue;
        }

        // Check if ready to attack
        if !combat_stats.can_attack() {
            continue;
        }

        // Find target in the targets list
        let target_entry = targets.iter_mut().find(|(id, _, _)| *id == target_id);

        let Some((_, target_health, target_combat)) = target_entry else {
            // Target not in health list - clear it
            attack_target.clear();
            continue;
        };

        // Emit attack started event
        combat_events.push(CombatEvent::AttackStarted {
            attacker: *attacker_id,
            target: target_id,
            damage_type: combat_stats.damage_type,
        });

        // Check if this is a projectile weapon
        if combat_stats.uses_projectiles() {
            // Spawn projectile - damage will be applied when it hits
            combat_events.push(CombatEvent::ProjectileSpawned {
                source: *attacker_id,
                target: target_id,
                damage: combat_stats.damage,
                damage_type: combat_stats.damage_type,
                start_pos: attacker_pos.value,
                speed: combat_stats.projectile_speed,
            });
            combat_stats.reset_cooldown();
        } else {
            // Hitscan weapon - apply damage immediately using resistance-based system
            let base_damage = combat_stats.damage;
            let weapon_stats = combat_stats.to_weapon_stats();
            let target_stats = target_combat.to_resistance_stats();
            let final_damage = calculate_resistance_damage(&weapon_stats, &target_stats);

            target_health.apply_damage(final_damage);

            // Emit damage event
            combat_events.push(CombatEvent::DamageDealt {
                attacker: *attacker_id,
                target: target_id,
                base_damage,
                final_damage,
                damage_type: combat_stats.damage_type,
            });

            // Legacy damage event
            damage_events.push(DamageEvent {
                attacker: *attacker_id,
                target: target_id,
                damage: final_damage,
            });

            // Reset cooldown
            combat_stats.reset_cooldown();

            // Check for kill
            if target_health.is_dead() {
                combat_events.push(CombatEvent::UnitKilled {
                    killer: *attacker_id,
                    victim: target_id,
                });
                attack_target.clear();
            }
        }
    }

    (damage_events, combat_events)
}

// ============================================================================
// Projectile System
// ============================================================================

/// Result of projectile processing.
#[derive(Debug, Clone)]
pub struct ProjectileUpdate {
    /// Projectile entity ID.
    pub projectile_id: EntityId,
    /// Whether the projectile hit its target and should be removed.
    pub hit: bool,
    /// Combat event if damage was dealt.
    pub event: Option<CombatEvent>,
}

/// Processes active projectiles, moving them toward targets and applying damage on hit.
///
/// # Arguments
/// * `projectiles` - Active projectile entities with positions
/// * `targets` - Target entities with health and combat stats
/// * `positions` - Position lookup for target positions
///
/// # Returns
/// Vector of projectile updates (which hit, damage dealt)
pub fn projectile_system(
    projectiles: &mut [(EntityId, &mut Position, &Projectile)],
    targets: &mut [(EntityId, &mut Health, &CombatStats)],
    positions: &PositionLookup<'_>,
) -> Vec<ProjectileUpdate> {
    let mut updates = Vec::new();

    for (proj_id, proj_pos, projectile) in projectiles.iter_mut() {
        // Get target position
        let Some(target_pos) = positions.get(projectile.target) else {
            // Target gone - projectile fizzles
            updates.push(ProjectileUpdate {
                projectile_id: *proj_id,
                hit: true, // Remove it
                event: None,
            });
            continue;
        };

        // Calculate direction to target
        let diff = target_pos.value - proj_pos.value;
        let dist_sq = proj_pos.value.distance_squared(target_pos.value);
        let speed_sq = projectile.speed * projectile.speed;

        // Check if we've arrived (within one tick of movement)
        if dist_sq <= speed_sq {
            // Hit the target!
            if let Some((_, target_health, target_combat)) = targets
                .iter_mut()
                .find(|(id, _, _)| *id == projectile.target)
            {
                // Create weapon stats from projectile data using resistance-based system
                use crate::combat::{ExtendedDamageType, WeaponStats};
                let weapon_stats = WeaponStats::new(
                    projectile.damage,
                    ExtendedDamageType::from_damage_type(projectile.damage_type),
                );
                let target_stats = target_combat.to_resistance_stats();
                let final_damage = calculate_resistance_damage(&weapon_stats, &target_stats);

                target_health.apply_damage(final_damage);

                updates.push(ProjectileUpdate {
                    projectile_id: *proj_id,
                    hit: true,
                    event: Some(CombatEvent::ProjectileHit {
                        source: projectile.source,
                        target: projectile.target,
                        damage: final_damage,
                    }),
                });
            } else {
                updates.push(ProjectileUpdate {
                    projectile_id: *proj_id,
                    hit: true,
                    event: None,
                });
            }
        } else {
            // Move toward target
            let direction = normalize_vec2(diff);
            proj_pos.value = proj_pos.value
                + Vec2Fixed::new(
                    direction.x * projectile.speed,
                    direction.y * projectile.speed,
                );

            updates.push(ProjectileUpdate {
                projectile_id: *proj_id,
                hit: false,
                event: None,
            });
        }
    }

    updates
}

// ============================================================================
// Auto-Attack System
// ============================================================================

/// Finds enemies in range and sets attack targets for units without commands.
///
/// This implements auto-attack behavior for idle units.
///
/// # Arguments
/// * `units` - Units that can potentially attack
/// * `enemies` - Enemy positions with their IDs
/// * `is_enemy` - Function to check if two entities are enemies
///
/// # Returns
/// Number of new attack targets acquired
pub fn auto_attack_system<F>(
    units: &mut [(
        EntityId,
        &Position,
        &mut AttackTarget,
        &CombatStats,
        &CommandQueue,
    )],
    enemies: &[(EntityId, Position)],
    is_enemy: F,
) -> usize
where
    F: Fn(EntityId, EntityId) -> bool,
{
    let mut targets_acquired = 0;

    for (unit_id, position, attack_target, combat_stats, command_queue) in units.iter_mut() {
        // Skip if already has a target or has commands
        if attack_target.target.is_some() {
            continue;
        }

        // Only auto-attack if idle or on hold position
        let should_auto_attack = command_queue.is_empty()
            || matches!(command_queue.current(), Some(Command::HoldPosition));

        if !should_auto_attack {
            continue;
        }

        // Find closest enemy in range
        let range_sq = combat_stats.range * combat_stats.range;
        let mut best_target: Option<(EntityId, Fixed)> = None;

        for (enemy_id, enemy_pos) in enemies {
            if !is_enemy(*unit_id, *enemy_id) {
                continue;
            }

            let dist_sq = position.value.distance_squared(enemy_pos.value);
            if dist_sq <= range_sq {
                match best_target {
                    None => best_target = Some((*enemy_id, dist_sq)),
                    Some((_, best_dist)) if dist_sq < best_dist => {
                        best_target = Some((*enemy_id, dist_sq));
                    }
                    _ => {}
                }
            }
        }

        if let Some((target_id, _)) = best_target {
            attack_target.target = Some(target_id);
            targets_acquired += 1;
        }
    }

    targets_acquired
}

/// Normalizes a 2D vector using fixed-point math.
///
/// Uses integer square root approximation to avoid floating-point
/// operations while maintaining determinism.
///
/// Returns zero vector if input has zero length.
pub(crate) fn normalize_vec2(v: Vec2Fixed) -> Vec2Fixed {
    let len_sq = v.x * v.x + v.y * v.y;

    if len_sq == Fixed::ZERO {
        return Vec2Fixed::ZERO;
    }

    // Use integer square root on the raw bits, then convert back
    // This avoids overflow issues with Newton-Raphson on fixed-point
    let len = fixed_sqrt(len_sq);

    if len == Fixed::ZERO {
        return Vec2Fixed::ZERO;
    }

    Vec2Fixed::new(v.x / len, v.y / len)
}

/// Computes the square root of a fixed-point number using binary search.
///
/// This is deterministic and avoids overflow issues.
fn fixed_sqrt(value: Fixed) -> Fixed {
    if value <= Fixed::ZERO {
        return Fixed::ZERO;
    }

    // Binary search for sqrt
    let mut low = Fixed::ZERO;
    let mut high = if value > Fixed::from_num(1) {
        value
    } else {
        Fixed::from_num(1)
    };

    // 32 iterations gives us good precision for I32F32
    for _ in 0..32 {
        let mid = (low + high) / Fixed::from_num(2);
        let mid_sq = mid.saturating_mul(mid);

        if mid_sq <= value {
            low = mid;
        } else {
            high = mid;
        }
    }

    low
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Velocity;

    #[test]
    fn test_movement_system() {
        let mut pos = Position::new(Vec2Fixed::new(Fixed::from_num(10), Fixed::from_num(20)));
        let vel = Velocity::new(Vec2Fixed::new(Fixed::from_num(1), Fixed::from_num(-2)));

        let mut entities = vec![(1u64, &mut pos, &vel)];
        movement_system(&mut entities);

        assert_eq!(pos.value.x, Fixed::from_num(11));
        assert_eq!(pos.value.y, Fixed::from_num(18));
    }

    #[test]
    fn test_health_system_identifies_dead() {
        let alive = Health::new(100);
        let dead = Health {
            current: 0,
            max: 100,
        };
        let also_dead = Health {
            current: 0,
            max: 50,
        };

        let entities = vec![(1u64, &alive), (2u64, &dead), (3u64, &also_dead)];
        let dead_list = health_system(&entities);

        assert_eq!(dead_list.len(), 2);
        assert!(dead_list.contains(&2u64));
        assert!(dead_list.contains(&3u64));
        assert!(!dead_list.contains(&1u64));
    }

    #[test]
    fn test_normalize_vec2() {
        let v = Vec2Fixed::new(Fixed::from_num(3), Fixed::from_num(4));
        let normalized = normalize_vec2(v);

        // Length should be approximately 1 (within fixed-point precision)
        let len_sq = normalized.x * normalized.x + normalized.y * normalized.y;
        let one = Fixed::from_num(1);
        let epsilon = Fixed::from_num(0.01);

        assert!((len_sq - one).abs() < epsilon);
    }

    #[test]
    fn test_normalize_zero_vector() {
        let zero = Vec2Fixed::ZERO;
        let result = normalize_vec2(zero);
        assert_eq!(result, Vec2Fixed::ZERO);
    }

    // ========================================================================
    // Damage Calculation Tests
    // ========================================================================

    #[test]
    fn test_damage_kinetic_vs_all_armor_types() {
        let base_damage = 100;

        // Kinetic: 100% vs unarmored
        let dmg = calculate_damage(base_damage, DamageType::Kinetic, ArmorType::Unarmored, 0);
        assert_eq!(dmg, 100);

        // Kinetic: 75% vs light
        let dmg = calculate_damage(base_damage, DamageType::Kinetic, ArmorType::Light, 0);
        assert_eq!(dmg, 75);

        // Kinetic: 50% vs heavy
        let dmg = calculate_damage(base_damage, DamageType::Kinetic, ArmorType::Heavy, 0);
        assert_eq!(dmg, 50);

        // Kinetic: 25% vs building
        let dmg = calculate_damage(base_damage, DamageType::Kinetic, ArmorType::Building, 0);
        assert_eq!(dmg, 25);
    }

    #[test]
    fn test_damage_energy_ignores_armor_type() {
        let base_damage = 100;

        // Energy: 75% vs all armor types
        assert_eq!(
            calculate_damage(base_damage, DamageType::Energy, ArmorType::Unarmored, 0),
            75
        );
        assert_eq!(
            calculate_damage(base_damage, DamageType::Energy, ArmorType::Light, 0),
            75
        );
        assert_eq!(
            calculate_damage(base_damage, DamageType::Energy, ArmorType::Heavy, 0),
            75
        );
        assert_eq!(
            calculate_damage(base_damage, DamageType::Energy, ArmorType::Building, 0),
            75
        );
    }

    #[test]
    fn test_damage_explosive_vs_all_armor_types() {
        let base_damage = 100;

        // Explosive: 50% vs unarmored
        let dmg = calculate_damage(base_damage, DamageType::Explosive, ArmorType::Unarmored, 0);
        assert_eq!(dmg, 50);

        // Explosive: 50% vs light
        let dmg = calculate_damage(base_damage, DamageType::Explosive, ArmorType::Light, 0);
        assert_eq!(dmg, 50);

        // Explosive: 75% vs heavy
        let dmg = calculate_damage(base_damage, DamageType::Explosive, ArmorType::Heavy, 0);
        assert_eq!(dmg, 75);

        // Explosive: 150% vs building
        let dmg = calculate_damage(base_damage, DamageType::Explosive, ArmorType::Building, 0);
        assert_eq!(dmg, 150);
    }

    #[test]
    fn test_damage_biological_vs_all_armor_types() {
        let base_damage = 100;

        // Biological: 150% vs unarmored
        let dmg = calculate_damage(base_damage, DamageType::Biological, ArmorType::Unarmored, 0);
        assert_eq!(dmg, 150);

        // Biological: 50% vs light
        let dmg = calculate_damage(base_damage, DamageType::Biological, ArmorType::Light, 0);
        assert_eq!(dmg, 50);

        // Biological: 50% vs heavy
        let dmg = calculate_damage(base_damage, DamageType::Biological, ArmorType::Heavy, 0);
        assert_eq!(dmg, 50);

        // Biological: 0% vs building (immune)
        let dmg = calculate_damage(base_damage, DamageType::Biological, ArmorType::Building, 0);
        assert_eq!(dmg, 0);
    }

    #[test]
    fn test_damage_armor_reduction() {
        // 100 base damage, kinetic vs unarmored (100%), 10 armor
        let dmg = calculate_damage(100, DamageType::Kinetic, ArmorType::Unarmored, 10);
        assert_eq!(dmg, 90);

        // 100 base damage, kinetic vs light (75%), 10 armor = 75 - 10 = 65
        let dmg = calculate_damage(100, DamageType::Kinetic, ArmorType::Light, 10);
        assert_eq!(dmg, 65);
    }

    #[test]
    fn test_damage_minimum_one() {
        // High armor should still deal minimum 1 damage (unless immune)
        let dmg = calculate_damage(10, DamageType::Kinetic, ArmorType::Heavy, 100);
        assert_eq!(dmg, 1);

        // But biological vs building is truly immune
        let dmg = calculate_damage(1000, DamageType::Biological, ArmorType::Building, 0);
        assert_eq!(dmg, 0);
    }

    #[test]
    fn test_damage_determinism() {
        // Same inputs must always produce same outputs
        for _ in 0..100 {
            let dmg1 = calculate_damage(77, DamageType::Explosive, ArmorType::Heavy, 5);
            let dmg2 = calculate_damage(77, DamageType::Explosive, ArmorType::Heavy, 5);
            assert_eq!(dmg1, dmg2);
        }
    }

    // ========================================================================
    // Combat System Tests
    // ========================================================================

    #[test]
    fn test_combat_system_applies_damage_type() {
        let attacker_pos = Position::new(Vec2Fixed::new(Fixed::from_num(0), Fixed::from_num(0)));
        let target_pos = Position::new(Vec2Fixed::new(Fixed::from_num(3), Fixed::from_num(0)));

        let mut attack_target = AttackTarget::with_target(2);
        // Use the new resistance-based system: Explosive damage with Heavy weapon (bonus vs buildings)
        let mut attacker_stats = CombatStats::new(100, Fixed::from_num(10), 30)
            .with_damage_type(DamageType::Explosive)
            .with_weapon_size(crate::combat::WeaponSize::Heavy);

        let mut target_health = Health::new(200);
        // Use new armor_class field instead of legacy armor_type
        let target_stats =
            CombatStats::default().with_armor_class(crate::combat::ArmorClass::Building);

        let positions = [(2u64, target_pos)];
        let position_lookup = PositionLookup::new(&positions);

        let mut attackers = vec![(1u64, &attacker_pos, &mut attack_target, &mut attacker_stats)];
        let mut targets = vec![(2u64, &mut target_health, &target_stats)];

        let (damage_events, combat_events) =
            combat_system(&mut attackers, &mut targets, &position_lookup);

        // New resistance-based system: Explosive vs Building = 150%, Heavy weapon vs Building = 150%
        // 100 * 1.5 * 1.5 = 225 damage
        assert_eq!(damage_events.len(), 1);
        assert_eq!(damage_events[0].damage, 225);

        // Should have AttackStarted and DamageDealt events
        assert!(combat_events
            .iter()
            .any(|e| matches!(e, CombatEvent::AttackStarted { .. })));
        assert!(combat_events.iter().any(|e| matches!(
            e,
            CombatEvent::DamageDealt {
                final_damage: 225,
                ..
            }
        )));
    }

    #[test]
    fn test_combat_system_respects_cooldown() {
        let attacker_pos = Position::new(Vec2Fixed::ZERO);
        let target_pos = Position::new(Vec2Fixed::ZERO);

        let mut attack_target = AttackTarget::with_target(2);
        let mut attacker_stats = CombatStats::new(10, Fixed::from_num(10), 5);
        attacker_stats.cooldown_remaining = 3; // Not ready to attack

        let mut target_health = Health::new(100);
        let target_stats = CombatStats::default();

        let positions = [(2u64, target_pos)];
        let position_lookup = PositionLookup::new(&positions);

        // Tick 1: Should not deal damage while on cooldown (3->2)
        {
            let mut attackers =
                vec![(1u64, &attacker_pos, &mut attack_target, &mut attacker_stats)];
            let mut targets = vec![(2u64, &mut target_health, &target_stats)];
            let (damage_events, _) = combat_system(&mut attackers, &mut targets, &position_lookup);
            assert!(damage_events.is_empty());
        }
        assert_eq!(attacker_stats.cooldown_remaining, 2); // Ticked down

        // Tick 2: Still on cooldown (2->1)
        {
            let mut attackers =
                vec![(1u64, &attacker_pos, &mut attack_target, &mut attacker_stats)];
            let mut targets = vec![(2u64, &mut target_health, &target_stats)];
            let (damage_events, _) = combat_system(&mut attackers, &mut targets, &position_lookup);
            assert!(damage_events.is_empty());
        }
        assert_eq!(attacker_stats.cooldown_remaining, 1);

        // Tick 3: Cooldown reaches 0 (1->0), attack fires, cooldown resets to 5
        {
            let mut attackers =
                vec![(1u64, &attacker_pos, &mut attack_target, &mut attacker_stats)];
            let mut targets = vec![(2u64, &mut target_health, &target_stats)];
            let (damage_events, _) = combat_system(&mut attackers, &mut targets, &position_lookup);
            assert_eq!(damage_events.len(), 1);
        }
        assert_eq!(attacker_stats.cooldown_remaining, 5); // Reset after attack
    }

    #[test]
    fn test_combat_system_range_check() {
        let attacker_pos = Position::new(Vec2Fixed::new(Fixed::from_num(0), Fixed::from_num(0)));
        let target_pos = Position::new(Vec2Fixed::new(Fixed::from_num(100), Fixed::from_num(0)));

        let mut attack_target = AttackTarget::with_target(2);
        let mut attacker_stats = CombatStats::new(10, Fixed::from_num(5), 1); // Range 5

        let mut target_health = Health::new(100);
        let target_stats = CombatStats::default();

        let positions = [(2u64, target_pos)];
        let position_lookup = PositionLookup::new(&positions);

        let mut attackers = vec![(1u64, &attacker_pos, &mut attack_target, &mut attacker_stats)];
        let mut targets = vec![(2u64, &mut target_health, &target_stats)];

        let (damage_events, combat_events) =
            combat_system(&mut attackers, &mut targets, &position_lookup);

        // Target is 100 units away, range is 5 - should not hit
        assert!(damage_events.is_empty());

        // Should have OutOfRange event
        assert!(combat_events
            .iter()
            .any(|e| matches!(e, CombatEvent::OutOfRange { .. })));
    }

    #[test]
    fn test_combat_system_kill_event() {
        let attacker_pos = Position::new(Vec2Fixed::ZERO);
        let target_pos = Position::new(Vec2Fixed::ZERO);

        let mut attack_target = AttackTarget::with_target(2);
        let mut attacker_stats = CombatStats::new(100, Fixed::from_num(10), 1);

        let mut target_health = Health::new(50); // Will die from 100 damage
        let target_stats = CombatStats::default();

        let positions = [(2u64, target_pos)];
        let position_lookup = PositionLookup::new(&positions);

        let mut attackers = vec![(1u64, &attacker_pos, &mut attack_target, &mut attacker_stats)];
        let mut targets = vec![(2u64, &mut target_health, &target_stats)];

        let (_, combat_events) = combat_system(&mut attackers, &mut targets, &position_lookup);

        // Should have UnitKilled event
        assert!(combat_events.iter().any(|e| matches!(
            e,
            CombatEvent::UnitKilled {
                killer: 1,
                victim: 2
            }
        )));

        // Attack target should be cleared
        assert!(attack_target.target.is_none());
    }

    #[test]
    fn test_combat_stats_builder() {
        use crate::combat::{ArmorClass, WeaponSize};
        let stats = CombatStats::new(50, Fixed::from_num(8), 20)
            .with_damage_type(DamageType::Energy)
            .with_resistance(ArmorClass::Heavy, 45)
            .with_weapon_size(WeaponSize::Medium)
            .with_projectile_speed(Fixed::from_num(2));

        assert_eq!(stats.damage, 50);
        assert_eq!(stats.damage_type, DamageType::Energy);
        assert_eq!(stats.armor_class, ArmorClass::Heavy);
        assert_eq!(stats.resistance, 45);
        assert_eq!(stats.range, Fixed::from_num(8));
        assert_eq!(stats.attack_cooldown, 20);
        assert_eq!(stats.projectile_speed, Fixed::from_num(2));
        assert!(stats.uses_projectiles());
    }
}
