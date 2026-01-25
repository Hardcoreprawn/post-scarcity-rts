//! Selection plugin for unit selection mechanics.
//!
//! Provides click-to-select, box selection, and visual highlighting.

use bevy::gizmos::gizmos::Gizmos;
use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::camera::MainCamera;
use crate::components::{GameFaction, PlayerFaction, Selectable, Selected};
use crate::construction::BuildingPlacement;

/// Plugin for unit selection mechanics.
///
/// Provides:
/// - Click to select single unit
/// - Box select with drag
/// - Selection visual highlighting
pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectionState>()
            .init_resource::<PlayerFaction>()
            .add_systems(Update, handle_selection_input)
            .add_systems(Update, update_selection_box)
            .add_systems(Update, apply_box_selection)
            .add_systems(Update, sync_selection_visuals);
    }
}

/// Tracks the current selection box state.
#[derive(Resource, Default)]
pub struct SelectionState {
    /// Starting point of the selection box (if dragging).
    pub drag_start: Option<Vec2>,
    /// Whether we're currently dragging a selection box.
    pub is_dragging: bool,
}

/// Visual component for selection highlight.
#[derive(Component)]
pub struct SelectionHighlight;

/// Handles mouse input for selection.
fn handle_selection_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut selection_state: ResMut<SelectionState>,
    mut commands: Commands,
    player_faction: Res<PlayerFaction>,
    selectables: Query<(Entity, &Transform, &GameFaction), With<Selectable>>,
    selected: Query<Entity, With<Selected>>,
    placement: Res<BuildingPlacement>,
    mut egui_contexts: EguiContexts,
) {
    // Skip selection when in building placement mode
    if placement.placing.is_some() {
        return;
    }

    // Skip selection when egui is using the pointer
    if let Some(ctx) = egui_contexts.try_ctx_mut() {
        if ctx.is_pointer_over_area() || ctx.wants_pointer_input() {
            return;
        }
    }

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // Convert screen position to world position
    let Some(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position)
    else {
        return;
    };

    // Start drag on left mouse button press
    if mouse_button.just_pressed(MouseButton::Left) {
        selection_state.drag_start = Some(world_position);
        selection_state.is_dragging = false;
    }

    // Detect dragging (moved more than a threshold)
    if mouse_button.pressed(MouseButton::Left) {
        if let Some(start) = selection_state.drag_start {
            if start.distance(world_position) > 5.0 {
                selection_state.is_dragging = true;
            }
        }
    }

    // Handle release
    if mouse_button.just_released(MouseButton::Left) {
        let shift_held =
            keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

        if !selection_state.is_dragging {
            // Single click - select unit under cursor
            if !shift_held {
                // Clear existing selection
                for entity in selected.iter() {
                    commands.entity(entity).remove::<Selected>();
                }
            }

            // Find and select unit under cursor (only player faction)
            let click_radius = 20.0;
            let mut closest: Option<(Entity, f32)> = None;

            for (entity, transform, faction) in selectables.iter() {
                // Only allow selecting player's own units
                if faction.faction != player_faction.faction {
                    continue;
                }
                let distance = transform.translation.truncate().distance(world_position);
                if distance < click_radius && closest.map_or(true, |(_, d)| distance < d) {
                    closest = Some((entity, distance));
                }
            }

            if let Some((entity, _)) = closest {
                commands.entity(entity).insert(Selected);
            }
        }

        selection_state.drag_start = None;
        selection_state.is_dragging = false;
    }
}

/// Updates the visual selection box during dragging.
fn update_selection_box(
    selection_state: Res<SelectionState>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut gizmos: Gizmos,
) {
    if !selection_state.is_dragging {
        return;
    }

    let Some(start) = selection_state.drag_start else {
        return;
    };

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Some(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position)
    else {
        return;
    };

    // Draw selection box using gizmos
    let min = start.min(world_position);
    let max = start.max(world_position);
    let center = (min + max) / 2.0;
    let size = max - min;

    gizmos.rect_2d(center, 0.0, size, Color::srgba(0.0, 1.0, 0.0, 0.8));
}

/// Applies box selection when drag ends.
fn apply_box_selection(
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    selection_state: Res<SelectionState>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut commands: Commands,
    player_faction: Res<PlayerFaction>,
    selectables: Query<(Entity, &Transform, &GameFaction), With<Selectable>>,
    selected: Query<Entity, With<Selected>>,
) {
    if !selection_state.is_dragging || !mouse_button.just_released(MouseButton::Left) {
        return;
    }

    let Some(start) = selection_state.drag_start else {
        return;
    };

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Some(end) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    // Clear existing selection if shift not held
    if !shift_held {
        for entity in selected.iter() {
            commands.entity(entity).remove::<Selected>();
        }
    }

    // Select all units within the box (only player faction)
    let min = start.min(end);
    let max = start.max(end);
    let selection_rect = Rect::from_corners(min, max);

    for (entity, transform, faction) in selectables.iter() {
        // Only allow selecting player's own units
        if faction.faction != player_faction.faction {
            continue;
        }
        let pos = transform.translation.truncate();
        if selection_rect.contains(pos) {
            commands.entity(entity).insert(Selected);
        }
    }
}

/// Syncs visual highlights with selection state.
fn sync_selection_visuals(
    mut commands: Commands,
    selected: Query<Entity, (With<Selected>, Without<SelectionHighlight>)>,
    deselected: Query<Entity, (With<SelectionHighlight>, Without<Selected>)>,
) {
    // Add highlight to newly selected entities
    for entity in selected.iter() {
        commands.entity(entity).insert(SelectionHighlight);
    }

    // Remove highlight from deselected entities
    for entity in deselected.iter() {
        commands.entity(entity).remove::<SelectionHighlight>();
    }
}
