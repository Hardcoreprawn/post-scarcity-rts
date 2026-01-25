//! Camera plugin for 2D camera control.
//!
//! Provides WASD/Arrow key panning, mouse wheel zoom, and edge scrolling.

use bevy::prelude::*;

/// Plugin for 2D camera control with pan and zoom.
///
/// Provides:
/// - WASD/Arrow key panning
/// - Mouse wheel zoom
/// - Optional edge scrolling
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraSettings>()
            .add_systems(Startup, spawn_camera)
            .add_systems(
                Update,
                (camera_keyboard_pan, camera_mouse_zoom, camera_edge_scroll),
            );
    }
}

/// Settings for camera behavior.
#[derive(Resource)]
pub struct CameraSettings {
    /// Pan speed in units per second.
    pub pan_speed: f32,
    /// Zoom speed multiplier.
    pub zoom_speed: f32,
    /// Minimum zoom level (most zoomed in).
    pub min_zoom: f32,
    /// Maximum zoom level (most zoomed out).
    pub max_zoom: f32,
    /// Enable edge scrolling when mouse is near window edges.
    pub edge_scroll_enabled: bool,
    /// Width of the edge scroll zone in pixels.
    pub edge_scroll_margin: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            pan_speed: 500.0,
            zoom_speed: 0.1,
            min_zoom: 0.5,
            max_zoom: 3.0,
            edge_scroll_enabled: true,
            edge_scroll_margin: 20.0,
        }
    }
}

/// Marker component for the main game camera.
#[derive(Component)]
pub struct MainCamera;

/// Spawns the main 2D camera.
fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

/// Handles keyboard-based camera panning (WASD and arrow keys).
fn camera_keyboard_pan(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    settings: Res<CameraSettings>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    let Ok(mut transform) = camera_query.get_single_mut() else {
        return;
    };

    let mut direction = Vec2::ZERO;

    // Horizontal movement
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    // Vertical movement
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }

    // Normalize to prevent faster diagonal movement
    if direction != Vec2::ZERO {
        direction = direction.normalize();
    }

    // Apply movement scaled by delta time and zoom level
    let zoom_factor = transform.scale.x;
    let delta = direction * settings.pan_speed * time.delta_seconds() * zoom_factor;
    transform.translation.x += delta.x;
    transform.translation.y += delta.y;
}

/// Handles mouse wheel zoom.
fn camera_mouse_zoom(
    mut scroll_events: EventReader<bevy::input::mouse::MouseWheel>,
    settings: Res<CameraSettings>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    let Ok(mut transform) = camera_query.get_single_mut() else {
        return;
    };

    for event in scroll_events.read() {
        let zoom_delta = -event.y * settings.zoom_speed;
        let new_scale =
            (transform.scale.x + zoom_delta).clamp(settings.min_zoom, settings.max_zoom);
        transform.scale = Vec3::splat(new_scale);
    }
}

/// Handles edge scrolling when mouse is near window edges.
fn camera_edge_scroll(
    windows: Query<&Window>,
    time: Res<Time>,
    settings: Res<CameraSettings>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    if !settings.edge_scroll_enabled {
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok(mut transform) = camera_query.get_single_mut() else {
        return;
    };

    let mut direction = Vec2::ZERO;
    let margin = settings.edge_scroll_margin;

    // Check edges
    if cursor_position.x < margin {
        direction.x -= 1.0;
    }
    if cursor_position.x > window.width() - margin {
        direction.x += 1.0;
    }
    if cursor_position.y < margin {
        direction.y += 1.0; // Bevy's window Y is inverted
    }
    if cursor_position.y > window.height() - margin {
        direction.y -= 1.0;
    }

    if direction != Vec2::ZERO {
        direction = direction.normalize();
        let zoom_factor = transform.scale.x;
        let delta = direction * settings.pan_speed * time.delta_seconds() * zoom_factor;
        transform.translation.x += delta.x;
        transform.translation.y += delta.y;
    }
}
