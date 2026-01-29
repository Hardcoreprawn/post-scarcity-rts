//! Selection plugin for unit selection mechanics.
//!
//! Provides click-to-select, box selection, and visual highlighting.

use bevy::gizmos::gizmos::Gizmos;
use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::camera::MainCamera;
use crate::components::{GameFaction, PlayerFaction, Selectable, Selected, UnitDataId};
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
            .init_resource::<DoubleClickState>()
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

/// Tracks double-click timing for selection.
#[derive(Resource, Default)]
struct DoubleClickState {
    last_click_time: f32,
    last_clicked_unit: Option<String>,
}

const DOUBLE_CLICK_THRESHOLD: f32 = 0.3;

fn double_click_unit_id(
    state: &mut DoubleClickState,
    clicked_unit: Option<&UnitDataId>,
    now: f32,
) -> Option<String> {
    let clicked_id = clicked_unit.map(|id| id.as_str().to_string());
    let is_double_click = clicked_id.is_some()
        && clicked_id == state.last_clicked_unit
        && (now - state.last_click_time) <= DOUBLE_CLICK_THRESHOLD;

    state.last_click_time = now;
    state.last_clicked_unit = clicked_id.clone();

    if is_double_click {
        clicked_id
    } else {
        None
    }
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
    mut double_click_state: ResMut<DoubleClickState>,
    mut commands: Commands,
    selectables: Query<
        (
            Entity,
            &Transform,
            Option<&Sprite>,
            &GameFaction,
            Option<&UnitDataId>,
        ),
        With<Selectable>,
    >,
    selected: Query<Entity, With<Selected>>,
    placement: Res<BuildingPlacement>,
    mut egui_contexts: EguiContexts,
    time: Res<Time>,
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

            // Find and select unit under cursor
            let mut closest: Option<(Entity, f32, Option<&UnitDataId>)> = None;

            for (entity, transform, sprite, _faction, unit_data_id) in selectables.iter() {
                let click_radius = selection_radius(sprite);
                let distance = transform.translation.truncate().distance(world_position);
                if distance < click_radius && closest.map_or(true, |(_, d, _)| distance < d) {
                    closest = Some((entity, distance, unit_data_id));
                }
            }

            if let Some((entity, _, unit_data_id)) = closest {
                let now = time.elapsed_seconds();
                let double_click_id =
                    double_click_unit_id(&mut double_click_state, unit_data_id, now);

                if let Some(unit_id) = double_click_id {
                    for (candidate, _, _sprite, _faction, candidate_id) in selectables.iter() {
                        if candidate_id.map(|id| id.as_str()) == Some(unit_id.as_str()) {
                            commands.entity(candidate).insert(Selected);
                        }
                    }
                } else {
                    commands.entity(entity).insert(Selected);
                }
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

    // Select all units within the box
    let min = start.min(end);
    let max = start.max(end);
    let selection_rect = Rect::from_corners(min, max);

    for (entity, transform, _faction) in selectables.iter() {
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

fn selection_radius(sprite: Option<&Sprite>) -> f32 {
    const FALLBACK_RADIUS: f32 = 20.0;
    let size = sprite
        .and_then(|sprite| sprite.custom_size)
        .unwrap_or(Vec2::new(FALLBACK_RADIUS * 2.0, FALLBACK_RADIUS * 2.0));
    let radius = size.x.max(size.y) / 2.0;
    radius.max(FALLBACK_RADIUS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn double_click_detects_same_unit_type() {
        let mut state = DoubleClickState::default();
        let unit_id = UnitDataId::new("infantry");

        let first = double_click_unit_id(&mut state, Some(&unit_id), 1.0);
        assert!(first.is_none());

        let second = double_click_unit_id(&mut state, Some(&unit_id), 1.2);
        assert_eq!(second.as_deref(), Some("infantry"));
    }

    #[test]
    fn double_click_ignores_delayed_clicks() {
        let mut state = DoubleClickState::default();
        let unit_id = UnitDataId::new("infantry");

        let _ = double_click_unit_id(&mut state, Some(&unit_id), 1.0);
        let second = double_click_unit_id(&mut state, Some(&unit_id), 2.0);
        assert!(second.is_none());
    }

    #[test]
    fn selection_radius_scales_with_sprite_size() {
        let small = Sprite {
            custom_size: Some(Vec2::new(20.0, 20.0)),
            ..Default::default()
        };
        let large = Sprite {
            custom_size: Some(Vec2::new(80.0, 40.0)),
            ..Default::default()
        };

        let small_radius = selection_radius(Some(&small));
        let large_radius = selection_radius(Some(&large));
        let fallback_radius = selection_radius(None);

        assert!(large_radius > small_radius);
        assert!(small_radius >= fallback_radius);
    }
}
