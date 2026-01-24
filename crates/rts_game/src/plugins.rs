//! Game plugins for Bevy.
//!
//! This module contains the core gameplay plugins that integrate Bevy's
//! rendering and input systems with the rts_core simulation layer.
//!
//! Since rts_core components are designed for deterministic simulation and
//! don't derive Bevy's Component trait, this module provides wrapper
//! components that bridge the simulation layer to the rendering layer.

use bevy::app::PluginGroupBuilder;
use bevy::gizmos::gizmos::Gizmos;
use bevy::prelude::*;

use rts_core::components::Command as CoreCommand;
use rts_core::factions::FactionId;
use rts_core::math::{Fixed, Vec2Fixed};

// ============================================================================
// Bevy Component Wrappers
// ============================================================================

/// Wrapper for rts_core::Position that implements Bevy Component.
///
/// This bridges the simulation's fixed-point positions to the render layer.
#[derive(Component, Debug, Clone, Copy)]
pub struct GamePosition {
    /// The fixed-point position from the simulation.
    pub value: Vec2Fixed,
}

impl GamePosition {
    /// Create a new game position.
    #[must_use]
    pub const fn new(value: Vec2Fixed) -> Self {
        Self { value }
    }

    /// Create a position at the origin.
    pub const ORIGIN: Self = Self {
        value: Vec2Fixed::ZERO,
    };

    /// Convert to Bevy Vec2 for rendering.
    #[must_use]
    pub fn as_vec2(&self) -> Vec2 {
        Vec2::new(self.value.x.to_num(), self.value.y.to_num())
    }
}

/// Wrapper for health that implements Bevy Component.
#[derive(Component, Debug, Clone, Copy)]
pub struct GameHealth {
    /// Current health points.
    pub current: i32,
    /// Maximum health points.
    pub max: i32,
}

impl GameHealth {
    /// Create new health at full.
    #[must_use]
    pub const fn new(max: i32) -> Self {
        Self { current: max, max }
    }

    /// Check if entity is at full health.
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.current >= self.max
    }

    /// Get health as a ratio (0.0 to 1.0).
    #[must_use]
    pub fn ratio(&self) -> f32 {
        if self.max == 0 {
            0.0
        } else {
            self.current as f32 / self.max as f32
        }
    }
}

/// Marker component for entities that can be selected.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Selectable;

/// Marker component for entities currently selected.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Selected;

/// Component for faction membership.
#[derive(Component, Debug, Clone, Copy)]
pub struct GameFaction {
    /// The faction this entity belongs to.
    pub faction: FactionId,
}

/// Command queue wrapper for Bevy.
#[derive(Component, Debug, Clone, Default)]
pub struct GameCommandQueue {
    /// Pending commands.
    pub commands: std::collections::VecDeque<CoreCommand>,
}

impl GameCommandQueue {
    /// Create an empty command queue.
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: std::collections::VecDeque::new(),
        }
    }

    /// Add a command to the back of the queue.
    pub fn push(&mut self, command: CoreCommand) {
        self.commands.push_back(command);
    }

    /// Replace all commands with a single new command.
    pub fn set(&mut self, command: CoreCommand) {
        self.commands.clear();
        self.commands.push_back(command);
    }
}

// ============================================================================
// Plugin Group
// ============================================================================

/// Main plugin group containing all game client plugins.
///
/// This bundles together camera, selection, input, and rendering plugins
/// for easy registration with the Bevy app.
///
/// # Example
/// ```ignore
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(GamePlugins)
///     .run();
/// ```
pub struct GamePlugins;

impl PluginGroup for GamePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(CameraPlugin)
            .add(SelectionPlugin)
            .add(InputPlugin)
            .add(RenderPlugin)
    }
}

// ============================================================================
// Camera Plugin
// ============================================================================

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

// ============================================================================
// Selection Plugin
// ============================================================================

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
    selectables: Query<(Entity, &Transform), With<Selectable>>,
    selected: Query<Entity, With<Selected>>,
) {
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
            let click_radius = 20.0;
            let mut closest: Option<(Entity, f32)> = None;

            for (entity, transform) in selectables.iter() {
                let distance = transform.translation.truncate().distance(world_position);
                if distance < click_radius {
                    if closest.map_or(true, |(_, d)| distance < d) {
                        closest = Some((entity, distance));
                    }
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
    selectables: Query<(Entity, &Transform), With<Selectable>>,
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

    let shift_held =
        keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

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

    for (entity, transform) in selectables.iter() {
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

// ============================================================================
// Input Plugin
// ============================================================================

/// Plugin for game input handling.
///
/// Provides:
/// - Right-click to issue move commands
/// - Attack-move with A + right-click
/// - Stop command with S key
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputMode>()
            .add_systems(Update, update_input_mode)
            .add_systems(Update, handle_move_command)
            .add_systems(Update, handle_stop_command);
    }
}

/// Current input mode for command issuing.
#[derive(Resource, Default, PartialEq, Eq)]
pub enum InputMode {
    /// Normal mode - right-click moves.
    #[default]
    Normal,
    /// Attack mode - right-click attack-moves.
    AttackMove,
}

/// Updates the input mode based on key presses.
fn update_input_mode(keyboard: Res<ButtonInput<KeyCode>>, mut input_mode: ResMut<InputMode>) {
    // Hold A for attack-move mode
    if keyboard.pressed(KeyCode::KeyA) {
        *input_mode = InputMode::AttackMove;
    } else {
        *input_mode = InputMode::Normal;
    }
}

/// Handles right-click to issue move/attack-move commands.
fn handle_move_command(
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    input_mode: Res<InputMode>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut selected_units: Query<&mut GameCommandQueue, With<Selected>>,
) {
    if !mouse_button.just_pressed(MouseButton::Right) {
        return;
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

    let Some(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position)
    else {
        return;
    };

    // Convert Bevy Vec2 to fixed-point
    let target = Vec2Fixed::new(
        Fixed::from_num(world_position.x),
        Fixed::from_num(world_position.y),
    );

    let shift_held =
        keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    let command = match *input_mode {
        InputMode::Normal => CoreCommand::MoveTo(target),
        InputMode::AttackMove => CoreCommand::AttackMove(target),
    };

    for mut queue in selected_units.iter_mut() {
        if shift_held {
            // Queue the command
            queue.push(command.clone());
        } else {
            // Replace existing commands
            queue.set(command.clone());
        }
    }
}

/// Handles S key to issue stop commands.
fn handle_stop_command(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selected_units: Query<&mut GameCommandQueue, With<Selected>>,
) {
    if keyboard.just_pressed(KeyCode::KeyS) {
        for mut queue in selected_units.iter_mut() {
            queue.set(CoreCommand::Stop);
        }
    }
}

// ============================================================================
// Render Plugin
// ============================================================================

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
fn render_health_bars(units: Query<(&Transform, &GameHealth)>, mut gizmos: Gizmos) {
    const BAR_WIDTH: f32 = 40.0;
    const BAR_HEIGHT: f32 = 4.0;
    const BAR_OFFSET: f32 = 25.0;

    for (transform, health) in units.iter() {
        if health.is_full() {
            continue; // Don't show health bar for full health units
        }

        let pos = transform.translation.truncate();
        let bar_center = pos + Vec2::new(0.0, BAR_OFFSET);

        // Background (red)
        gizmos.rect_2d(
            bar_center,
            0.0,
            Vec2::new(BAR_WIDTH, BAR_HEIGHT),
            Color::srgba(0.5, 0.0, 0.0, 0.8),
        );

        // Health portion (green)
        let health_width = BAR_WIDTH * health.ratio();
        let health_offset = (BAR_WIDTH - health_width) / 2.0;

        gizmos.rect_2d(
            bar_center - Vec2::new(health_offset, 0.0),
            0.0,
            Vec2::new(health_width, BAR_HEIGHT),
            Color::srgba(0.0, 0.8, 0.0, 0.8),
        );
    }
}

/// Returns the faction color for rendering.
#[must_use]
pub fn faction_color(faction: FactionId) -> Color {
    match faction {
        FactionId::Continuity => Color::srgb(0.2, 0.4, 0.8),    // Blue
        FactionId::Collegium => Color::srgb(0.8, 0.6, 0.2),     // Gold
        FactionId::Tinkers => Color::srgb(0.6, 0.4, 0.2),       // Brown
        FactionId::BioSovereigns => Color::srgb(0.2, 0.7, 0.3), // Green
        FactionId::Zephyr => Color::srgb(0.6, 0.8, 0.9),        // Sky blue
    }
}

/// Bundle for spawning a unit with all required game components.
///
/// This creates a visible unit with placeholder sprite rendering,
/// selection capability, and command queue.
#[derive(Bundle)]
pub struct UnitBundle {
    /// The sprite bundle for the unit.
    pub sprite: SpriteBundle,
    /// Whether this unit can be selected.
    pub selectable: Selectable,
    /// The faction this unit belongs to.
    pub faction: GameFaction,
    /// The command queue for this unit.
    pub command_queue: GameCommandQueue,
    /// The game position (synced to transform).
    pub position: GamePosition,
    /// The unit's health.
    pub health: GameHealth,
}

impl UnitBundle {
    /// Creates a new unit bundle with faction-colored sprite.
    #[must_use]
    pub fn new(position: Vec2, faction: FactionId, max_health: i32) -> Self {
        Self {
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: faction_color(faction),
                    custom_size: Some(Vec2::new(32.0, 32.0)),
                    ..default()
                },
                transform: Transform::from_translation(position.extend(0.0)),
                ..default()
            },
            selectable: Selectable,
            faction: GameFaction { faction },
            command_queue: GameCommandQueue::new(),
            position: GamePosition::new(Vec2Fixed::new(
                Fixed::from_num(position.x),
                Fixed::from_num(position.y),
            )),
            health: GameHealth::new(max_health),
        }
    }
}
