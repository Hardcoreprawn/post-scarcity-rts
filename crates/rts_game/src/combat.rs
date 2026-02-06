//! Combat system for handling attacks, damage, and unit death.

use bevy::prelude::*;

use rts_core::components::EntityId;

use crate::components::{
    Armor, ArmorType, AttackTarget, CombatStats, DamageType, Dead, GameFaction, GameHealth,
    GamePosition, MovementTarget, PlayerFaction, Regeneration, Unit,
};
use crate::economy::PlayerResources;
use crate::simulation::CoreSimulation;

/// Range at which units will auto-acquire targets.
pub const AUTO_ATTACK_RANGE: f32 = 200.0;

/// Duration weapon fire effects persist (seconds).
const WEAPON_FIRE_DURATION: f32 = 0.15;
/// Delay before despawning dead units (seconds).
const DEATH_DESPAWN_DELAY: f32 = 0.6;
/// Duration of the death effect visual (seconds).
const DEATH_EFFECT_DURATION: f32 = 0.4;
/// Size of the death effect sprite.
const DEATH_EFFECT_SIZE: f32 = 18.0;

/// Component for weapon fire visual effects.
#[derive(Component)]
pub struct WeaponFire {
    /// Start position of the beam/projectile.
    pub from: Vec2,
    /// End position of the beam/projectile.
    pub to: Vec2,
    /// Time remaining before despawn.
    pub timer: f32,
    /// Color of the weapon fire.
    pub color: Color,
}

/// Component for death effect visual.
#[derive(Component)]
pub struct DeathEffect {
    /// Time remaining before despawn.
    pub timer: f32,
}

/// Component storing despawn delay for dead units.
#[derive(Component, Debug, Clone, Copy)]
pub struct DeathTimer {
    /// Time remaining before despawn.
    pub timer: f32,
}

/// Get weapon fire color based on damage type.
#[must_use]
pub fn weapon_fire_color(damage_type: DamageType) -> Color {
    match damage_type {
        DamageType::Kinetic => Color::srgb(1.0, 0.8, 0.0), // Yellow-orange
        DamageType::Energy => Color::srgb(0.3, 0.8, 1.0),  // Cyan-blue
        DamageType::Explosive => Color::srgb(1.0, 0.4, 0.2), // Hot red-orange
    }
}

/// Plugin for combat systems.
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_attack_cooldowns,
                regenerate_health,
                acquire_attack_targets,
                execute_attacks,
                process_deaths,
                update_death_effects,
                cleanup_dead_entities,
            )
                .chain(),
        )
        .add_systems(Update, (update_weapon_fire, render_weapon_fire))
        .add_systems(Update, sync_projectile_visuals);
    }
}

/// Updates attack cooldown timers.
fn update_attack_cooldowns(time: Res<Time>, mut attackers: Query<&mut CombatStats>) {
    let dt = time.delta_seconds();
    for mut stats in attackers.iter_mut() {
        stats.tick_cooldown(dt);
    }
}

/// Regenerates health for entities with Regeneration component.
fn regenerate_health(
    time: Res<Time>,
    mut healers: Query<(&mut GameHealth, &mut Regeneration), Without<Dead>>,
) {
    let dt = time.delta_seconds();
    for (mut health, mut regen) in healers.iter_mut() {
        // Skip if already at full health
        if health.is_full() {
            regen.accumulator = 0.0;
            continue;
        }

        // Accumulate regeneration
        regen.accumulator += regen.per_second * dt;

        // Apply whole HP amounts
        let heal_amount = regen.accumulator.floor() as u32;
        if heal_amount > 0 {
            health.heal(heal_amount);
            regen.accumulator -= heal_amount as f32;
        }
    }
}

/// Auto-acquire attack targets for idle units with combat capability.
fn acquire_attack_targets(
    mut commands: Commands,
    attackers: Query<
        (Entity, &GamePosition, &GameFaction, &CombatStats),
        (
            Without<AttackTarget>,
            Without<MovementTarget>,
            Without<Dead>,
        ),
    >,
    potential_targets: Query<
        (Entity, &GamePosition, &GameFaction, &GameHealth),
        (Without<Dead>, Without<AttackTarget>),
    >,
) {
    for (attacker_entity, attacker_pos, attacker_faction, stats) in attackers.iter() {
        let my_pos = attacker_pos.as_vec2();
        let mut closest_target: Option<(Entity, f32)> = None;

        // Find the closest enemy within auto-attack range
        for (target_entity, target_pos, target_faction, _health) in potential_targets.iter() {
            // Skip same faction
            if target_faction.faction == attacker_faction.faction {
                continue;
            }
            // Skip self
            if target_entity == attacker_entity {
                continue;
            }

            let target_world = target_pos.as_vec2();
            let dist = my_pos.distance(target_world);

            // Check if within auto-attack range (use weapon range or auto-attack range, whichever is larger)
            let acquisition_range = stats.range.max(AUTO_ATTACK_RANGE);
            if dist <= acquisition_range && closest_target.map_or(true, |(_, d)| dist < d) {
                closest_target = Some((target_entity, dist));
            }
        }

        // Assign target if found
        if let Some((target, _)) = closest_target {
            commands
                .entity(attacker_entity)
                .insert(AttackTarget { target });
        }
    }
}

/// Execute attacks on targets in range when cooldown is ready.
fn execute_attacks(
    mut commands: Commands,
    mut attackers: Query<
        (
            Entity,
            &GamePosition,
            &mut CombatStats,
            &AttackTarget,
            &GameFaction,
        ),
        Without<Dead>,
    >,
    mut targets: Query<(&GamePosition, Option<&Armor>, &GameFaction), Without<Dead>>,
) {
    for (attacker_entity, attacker_pos, mut stats, attack_target, attacker_faction) in
        attackers.iter_mut()
    {
        // Check if target still exists and is valid
        let Ok((target_pos, target_armor, target_faction)) = targets.get_mut(attack_target.target)
        else {
            // Target is gone
            commands.entity(attacker_entity).remove::<AttackTarget>();
            continue;
        };

        // Don't attack allies (in case somehow assigned)
        if target_faction.faction == attacker_faction.faction {
            commands.entity(attacker_entity).remove::<AttackTarget>();
            continue;
        }

        let my_pos = attacker_pos.as_vec2();
        let target_world = target_pos.as_vec2();
        let dist = my_pos.distance(target_world);

        // Check range and cooldown
        if dist <= stats.range && stats.can_attack() {
            // Calculate damage with armor modifier
            let armor_type = target_armor.map_or(ArmorType::Unarmored, |a| a.armor_type);
            let modifier = armor_type.damage_modifier(stats.damage_type);
            let final_damage = (stats.damage as f32 * modifier).round() as u32;

            stats.start_cooldown();

            // Spawn weapon fire visual
            commands.spawn(WeaponFire {
                from: my_pos,
                to: target_world,
                timer: WEAPON_FIRE_DURATION,
                color: weapon_fire_color(stats.damage_type),
            });

            tracing::debug!("Attack! {} damage ({}x modifier)", final_damage, modifier);
        }
    }
}

/// Marks units with zero health as dead.
fn process_deaths(
    mut commands: Commands,
    dying: Query<(Entity, &GameHealth, &GamePosition), Without<Dead>>,
) {
    for (entity, health, position) in dying.iter() {
        if health.current == 0 {
            commands.entity(entity).insert(Dead).insert(DeathTimer {
                timer: DEATH_DESPAWN_DELAY,
            });
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgba(1.0, 0.3, 0.1, 0.8),
                        custom_size: Some(Vec2::splat(DEATH_EFFECT_SIZE)),
                        ..default()
                    },
                    transform: Transform::from_translation(position.as_vec2().extend(0.5)),
                    ..default()
                },
                DeathEffect {
                    timer: DEATH_EFFECT_DURATION,
                },
            ));
            tracing::info!("Entity {:?} died", entity);
        }
    }
}

fn update_death_effects(
    time: Res<Time>,
    mut commands: Commands,
    mut effects: Query<(Entity, &mut DeathEffect)>,
) {
    let dt = time.delta_seconds();
    for (entity, mut effect) in effects.iter_mut() {
        effect.timer -= dt;
        if effect.timer <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Cleans up dead entities after a short delay.
fn cleanup_dead_entities(
    mut commands: Commands,
    time: Res<Time>,
    mut dead: Query<(Entity, &mut DeathTimer, Option<&Unit>, Option<&GameFaction>), With<Dead>>,
    attack_targets: Query<(Entity, &AttackTarget)>,
    player_faction: Res<PlayerFaction>,
    mut resources: ResMut<PlayerResources>,
) {
    let dt = time.delta_seconds();
    for (dead_entity, mut timer, unit, faction) in dead.iter_mut() {
        // Clear any attack targets pointing to this entity
        for (attacker, target) in attack_targets.iter() {
            if target.target == dead_entity {
                commands.entity(attacker).remove::<AttackTarget>();
            }
        }

        timer.timer -= dt;
        if timer.timer > 0.0 {
            continue;
        }

        // Release supply for player units
        if let (Some(unit), Some(faction)) = (unit, faction) {
            if faction.faction == player_faction.faction {
                resources.supply_used = (resources.supply_used - unit.supply()).max(0);
            }
        }

        // Despawn the dead entity
        commands.entity(dead_entity).despawn_recursive();
    }
}

/// Updates weapon fire timers and despawns expired ones.
fn update_weapon_fire(
    mut commands: Commands,
    time: Res<Time>,
    mut fires: Query<(Entity, &mut WeaponFire)>,
) {
    let dt = time.delta_seconds();
    for (entity, mut fire) in fires.iter_mut() {
        fire.timer -= dt;
        if fire.timer <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Renders weapon fire effects using gizmos.
fn render_weapon_fire(mut gizmos: Gizmos, fires: Query<&WeaponFire>) {
    for fire in fires.iter() {
        // Fade out over time
        let alpha = (fire.timer / WEAPON_FIRE_DURATION).clamp(0.0, 1.0);
        let color = fire.color.with_alpha(alpha);
        // Draw beam line
        gizmos.line_2d(fire.from, fire.to, color);
        // Draw impact hit marker
        gizmos.circle_2d(fire.to, 5.0, color);
    }
}

// ============================================================================
// Projectile Rendering
// ============================================================================

/// Component marking a Bevy entity as a visual projectile synced from core.
#[derive(Component)]
pub struct GameProjectile {
    /// The core simulation entity ID of this projectile.
    pub core_id: EntityId,
}

/// Projectile visual size.
const PROJECTILE_SIZE: f32 = 6.0;

/// Syncs projectile visuals from core simulation tick events.
///
/// Each tick, the core provides a list of active projectile positions.
/// This system spawns new visual entities for newly-seen projectiles,
/// updates positions for existing ones, and despawns visuals for
/// projectiles that no longer exist.
fn sync_projectile_visuals(
    mut commands: Commands,
    core: Res<CoreSimulation>,
    mut projectiles: Query<(Entity, &mut GameProjectile, &mut Transform)>,
) {
    let events = &core.last_events;

    // Build a set of active core projectile IDs and their positions
    let active: std::collections::HashMap<EntityId, (f32, f32)> = events
        .projectile_positions
        .iter()
        .map(|(id, pos)| (*id, (pos.x.to_num::<f32>(), pos.y.to_num::<f32>())))
        .collect();

    // Track which core IDs have matching Bevy entities
    let mut seen_core_ids = std::collections::HashSet::new();

    // Update existing projectile visuals or despawn removed ones
    for (entity, proj, mut transform) in projectiles.iter_mut() {
        if let Some(&(x, y)) = active.get(&proj.core_id) {
            transform.translation.x = x;
            transform.translation.y = y;
            seen_core_ids.insert(proj.core_id);
        } else {
            // Projectile no longer exists in core — despawn visual
            commands.entity(entity).despawn();
        }
    }

    // Spawn visuals for new projectiles
    for (core_id, (x, y)) in &active {
        if !seen_core_ids.contains(core_id) {
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(1.0, 0.9, 0.3), // Bright yellow
                        custom_size: Some(Vec2::splat(PROJECTILE_SIZE)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(*x, *y, 0.5)),
                    ..default()
                },
                GameProjectile { core_id: *core_id },
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rts_core::math::{Fixed, Vec2Fixed};

    #[test]
    fn death_effect_spawns_for_dead_units() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, process_deaths);

        let entity = app
            .world_mut()
            .spawn((
                GameHealth {
                    current: 0,
                    max: 100,
                },
                GamePosition::new(Vec2Fixed::new(Fixed::from_num(0), Fixed::from_num(0))),
            ))
            .id();

        app.update();

        let world = app.world();
        assert!(world.get::<Dead>(entity).is_some());
        assert!(world.get::<DeathTimer>(entity).is_some());

        let world = app.world_mut();
        let effects_count = world.query::<&DeathEffect>().iter(world).count();
        assert_eq!(effects_count, 1);
    }
}
