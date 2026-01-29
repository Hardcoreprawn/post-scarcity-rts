//! Render plugin for basic sprite rendering.
//!
//! Provides position syncing, selection highlights, and health bars.

use bevy::gizmos::gizmos::Gizmos;
use bevy::prelude::*;

use crate::bundles::faction_color;
use crate::components::{
    CombatStats, GameFaction, GameHealth, GamePosition, Selectable, Selected, UnderConstruction,
};
use crate::selection::SelectionHighlight;
use crate::simulation::CoreSimulationSet;

/// Plugin for basic sprite rendering.
///
/// Provides:
/// - Placeholder colored rectangles for units
/// - Different colors per faction
/// - Health bars above units
pub struct RenderPlugin;

/// Plugin for damage flash feedback only (no gizmo dependencies).
///
/// Testing helper: `RenderPlugin` already registers these systems.
/// Avoid adding both in the same app to prevent duplicate registrations.
pub struct DamageFlashPlugin;

/// Plugin for range indicator updates only (no gizmo dependencies).
///
/// Testing helper: `RenderPlugin` already registers these systems.
/// Avoid adding both in the same app to prevent duplicate registrations.
pub struct RangeIndicatorPlugin;

/// Plugin for outline updates only (no gizmo dependencies).
///
/// Testing helper: `RenderPlugin` already registers these systems.
/// Avoid adding both in the same app to prevent duplicate registrations.
pub struct OutlinePlugin;

/// Event emitted when a command is issued for feedback.
#[derive(Event, Debug, Clone, Copy)]
pub struct CommandFeedbackEvent {
    /// World position for feedback.
    pub position: Vec2,
}

impl Plugin for DamageFlashPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, track_damage_flash)
            .add_systems(Update, update_damage_flash.after(track_damage_flash));
    }
}

impl Plugin for RangeIndicatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_range_indicators);
    }
}

impl Plugin for OutlinePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_unit_outlines);
    }
}

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CommandFeedbackEvent>()
            .add_systems(
                Update,
                sync_transform_from_position.after(CoreSimulationSet::SyncOut),
            )
            .add_systems(Update, render_selection_highlight)
            .add_systems(Update, render_health_bars)
            .add_systems(Update, update_range_indicators)
            .add_systems(
                Update,
                render_range_indicators.after(update_range_indicators),
            )
            .add_systems(Update, update_unit_outlines)
            .add_systems(Update, render_unit_outlines.after(update_unit_outlines))
            .add_systems(Update, track_damage_flash)
            .add_systems(Update, update_damage_flash.after(track_damage_flash))
            .add_systems(Update, spawn_command_pings)
            .add_systems(Update, update_command_pings.after(spawn_command_pings))
            .add_systems(Update, render_command_pings);
    }
}

/// Duration of command ping feedback (seconds).
const COMMAND_PING_DURATION: f32 = 0.45;

/// Component for command ping visuals.
#[derive(Component, Debug, Clone, Copy)]
struct CommandPing {
    position: Vec2,
    timer: f32,
}

/// Duration of the damage flash effect (seconds).
const DAMAGE_FLASH_DURATION: f32 = 0.2;

/// Component storing base color and timer for damage flash.
#[derive(Component, Debug, Clone, Copy)]
pub struct DamageFlash {
    /// Remaining flash time in seconds.
    pub timer: f32,
    /// Base color to restore after flash.
    pub base_color: Color,
}

/// Component for rendering a unit's attack range indicator.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct RangeIndicator {
    /// Range radius in world units.
    pub radius: f32,
}

/// Component for an outline ring used to improve unit readability.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct UnitOutline {
    /// Outline radius in world units.
    pub radius: f32,
    /// Outline color.
    pub color: Color,
}

/// Tracks last known health to detect damage events.
#[derive(Component, Debug, Clone, Copy)]
struct LastHealth {
    value: u32,
}

/// Syncs Bevy Transform from GamePosition.
///
/// This bridges the simulation layer's fixed-point positions to
/// Bevy's floating-point transforms for rendering.
fn sync_transform_from_position(
    mut query: Query<(&GamePosition, &mut Transform), Changed<GamePosition>>,
) {
    for (position, mut transform) in query.iter_mut() {
        let pos = position.as_vec2();
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
    }
}

/// Renders selection highlights around selected units.
fn render_selection_highlight(
    selected: Query<&Transform, With<SelectionHighlight>>,
    mut gizmos: Gizmos,
) {
    for transform in selected.iter() {
        let pos = transform.translation.truncate();
        // Draw a circle around selected units
        gizmos.circle_2d(pos, 25.0, Color::srgba(0.0, 1.0, 0.0, 0.8));
    }
}

/// Renders health bars above units.
fn render_health_bars(
    units: Query<(&Transform, &Sprite, &GameHealth, Option<&UnderConstruction>)>,
    mut gizmos: Gizmos,
) {
    const BAR_HEIGHT: f32 = 4.0;
    const BAR_PADDING: f32 = 8.0;

    for (transform, sprite, health, under_construction) in units.iter() {
        // Always show bar for buildings under construction
        let is_constructing = under_construction.is_some();

        // Skip full health units (unless under construction)
        if health.is_full() && !is_constructing {
            continue;
        }

        let pos = transform.translation.truncate();

        // Get sprite size to position bar above it
        let sprite_height = sprite.custom_size.map_or(32.0, |s| s.y);
        let bar_width = sprite.custom_size.map_or(40.0, |s| s.x.max(40.0));

        let bar_center = pos + Vec2::new(0.0, sprite_height / 2.0 + BAR_PADDING);

        // Background (dark)
        gizmos.rect_2d(
            bar_center,
            0.0,
            Vec2::new(bar_width, BAR_HEIGHT),
            Color::srgba(0.2, 0.2, 0.2, 0.9),
        );

        // Progress bar fill
        let ratio = health.ratio();
        let fill_width = bar_width * ratio;
        let bar_left = bar_center.x - bar_width / 2.0;
        let fill_center_x = bar_left + fill_width / 2.0;

        // Color: yellow for construction, green->red gradient for health
        let fill_color = if is_constructing {
            Color::srgba(1.0, 0.8, 0.0, 0.9) // Yellow for building
        } else if ratio > 0.5 {
            Color::srgba(0.0, 0.8, 0.0, 0.9) // Green
        } else if ratio > 0.25 {
            Color::srgba(0.8, 0.6, 0.0, 0.9) // Orange
        } else {
            Color::srgba(0.8, 0.0, 0.0, 0.9) // Red
        };

        if fill_width > 0.0 {
            gizmos.rect_2d(
                Vec2::new(fill_center_x, bar_center.y),
                0.0,
                Vec2::new(fill_width, BAR_HEIGHT),
                fill_color,
            );
        }
    }
}

/// Updates range indicators for selected combat units.
fn update_range_indicators(
    mut commands: Commands,
    selected_missing: Query<(Entity, &CombatStats), (With<Selected>, Without<RangeIndicator>)>,
    mut selected_with: Query<(&mut RangeIndicator, &CombatStats), With<Selected>>,
    deselected: Query<Entity, (With<RangeIndicator>, Without<Selected>)>,
) {
    for (entity, stats) in selected_missing.iter() {
        commands.entity(entity).insert(RangeIndicator {
            radius: stats.range,
        });
    }

    for (mut indicator, stats) in selected_with.iter_mut() {
        indicator.radius = stats.range;
    }

    for entity in deselected.iter() {
        commands.entity(entity).remove::<RangeIndicator>();
    }
}

/// Renders range indicators using gizmos.
fn render_range_indicators(indicators: Query<(&Transform, &RangeIndicator)>, mut gizmos: Gizmos) {
    for (transform, indicator) in indicators.iter() {
        let pos = transform.translation.truncate();
        gizmos.circle_2d(pos, indicator.radius, Color::srgba(0.0, 0.7, 1.0, 0.5));
    }
}

/// Updates unit outlines for readability.
fn update_unit_outlines(
    mut commands: Commands,
    units: Query<(Entity, &Sprite, &GameFaction), (With<Selectable>, Without<UnitOutline>)>,
    mut outlined: Query<(&mut UnitOutline, &Sprite, &GameFaction), With<Selectable>>,
    removed: Query<Entity, (With<UnitOutline>, Without<Selectable>)>,
) {
    for (entity, sprite, faction) in units.iter() {
        let radius = outline_radius(sprite);
        let color = faction_color(faction.faction);
        commands
            .entity(entity)
            .insert(UnitOutline { radius, color });
    }

    for (mut outline, sprite, faction) in outlined.iter_mut() {
        outline.radius = outline_radius(sprite);
        outline.color = faction_color(faction.faction);
    }

    for entity in removed.iter() {
        commands.entity(entity).remove::<UnitOutline>();
    }
}

/// Render unit outlines as thin rings.
fn render_unit_outlines(outlines: Query<(&Transform, &UnitOutline)>, mut gizmos: Gizmos) {
    for (transform, outline) in outlines.iter() {
        let pos = transform.translation.truncate();
        gizmos.circle_2d(pos, outline.radius, outline.color.with_alpha(0.35));
    }
}

fn outline_radius(sprite: &Sprite) -> f32 {
    let size = sprite.custom_size.unwrap_or(Vec2::new(32.0, 32.0));
    size.x.max(size.y) / 2.0 + 2.0
}

/// Detects damage and applies a brief flash to unit sprites.
fn track_damage_flash(
    mut commands: Commands,
    mut units: Query<(
        Entity,
        &GameHealth,
        &mut Sprite,
        Option<&mut LastHealth>,
        Option<&mut DamageFlash>,
    )>,
) {
    for (entity, health, mut sprite, last_health, flash) in units.iter_mut() {
        match last_health {
            Some(mut last) => {
                let base_color = flash
                    .as_ref()
                    .map(|existing| existing.base_color)
                    .unwrap_or(sprite.color);

                if health.current < last.value {
                    let flash_color = base_color.lighter(0.6);
                    sprite.color = flash_color;

                    if let Some(mut existing) = flash {
                        existing.timer = DAMAGE_FLASH_DURATION;
                        existing.base_color = base_color;
                    } else {
                        commands.entity(entity).insert(DamageFlash {
                            timer: DAMAGE_FLASH_DURATION,
                            base_color,
                        });
                    }
                }

                last.value = health.current;
            }
            None => {
                commands.entity(entity).insert(LastHealth {
                    value: health.current,
                });
            }
        }
    }
}

/// Updates damage flash timers and restores base colors.
fn update_damage_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut flashes: Query<(Entity, &mut Sprite, &mut DamageFlash)>,
) {
    let dt = time.delta_seconds();
    for (entity, mut sprite, mut flash) in flashes.iter_mut() {
        flash.timer -= dt;
        if flash.timer <= 0.0 {
            sprite.color = flash.base_color;
            commands.entity(entity).remove::<DamageFlash>();
        }
    }
}

fn spawn_command_pings(mut commands: Commands, mut events: EventReader<CommandFeedbackEvent>) {
    for event in events.read() {
        commands.spawn(CommandPing {
            position: event.position,
            timer: COMMAND_PING_DURATION,
        });
    }
}

fn update_command_pings(
    mut commands: Commands,
    time: Res<Time>,
    mut pings: Query<(Entity, &mut CommandPing)>,
) {
    let dt = time.delta_seconds();
    for (entity, mut ping) in pings.iter_mut() {
        ping.timer -= dt;
        if ping.timer <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn render_command_pings(pings: Query<&CommandPing>, mut gizmos: Gizmos) {
    for ping in pings.iter() {
        let alpha = (ping.timer / COMMAND_PING_DURATION).clamp(0.0, 1.0);
        let radius = 18.0 + (1.0 - alpha) * 10.0;
        gizmos.circle_2d(
            ping.position,
            radius,
            Color::srgba(0.1, 0.8, 1.0, 0.6 * alpha),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_ping_spawns_from_event() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<CommandFeedbackEvent>();
        app.add_systems(Update, spawn_command_pings);

        app.world_mut().send_event(CommandFeedbackEvent {
            position: Vec2::new(5.0, -3.0),
        });

        app.update();

        let mut pings = app.world_mut().query::<&CommandPing>();
        let ping = pings.single(app.world());
        assert_eq!(ping.position, Vec2::new(5.0, -3.0));
    }
}
