//! Render plugin for basic sprite rendering.
//!
//! Provides position syncing, selection highlights, and health bars.

use bevy::gizmos::gizmos::Gizmos;
use bevy::prelude::*;

use crate::components::{GameHealth, GamePosition, UnderConstruction};
use crate::selection::SelectionHighlight;

/// Plugin for basic sprite rendering.
///
/// Provides:
/// - Placeholder colored rectangles for units
/// - Different colors per faction
/// - Health bars above units
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync_transform_from_position)
            .add_systems(Update, render_selection_highlight)
            .add_systems(Update, render_health_bars);
    }
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
