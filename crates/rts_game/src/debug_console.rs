//! Debug console for development and testing.
//!
//! Provides an in-game console for:
//! - Spawning units and buildings
//! - Modifying resources
//! - Teleporting units
//! - Enabling god mode
//! - Game speed controls
//! - Triggering victory/defeat
//!
//! Toggle with backtick (`) key. Only available with `dev-tools` feature.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use rts_core::math::{Fixed, Vec2Fixed};

use crate::bundles::{
    BarracksBundle, DepotBundle, HarvesterBundle, SupplyDepotBundle, TechLabBundle, TurretBundle,
    UnitBundle,
};
use crate::components::{GameFaction, GameHealth, GamePosition, PlayerFaction, Selected};
use crate::economy::PlayerResources;
use crate::victory::GameState;

/// Debug console plugin.
///
/// Only active when `dev-tools` feature is enabled.
pub struct DebugConsolePlugin;

impl Plugin for DebugConsolePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugConsoleState>()
            .add_systems(Update, toggle_console)
            .add_systems(Update, render_console.after(toggle_console));
    }
}

/// State for the debug console.
#[derive(Resource)]
pub struct DebugConsoleState {
    /// Whether the console is visible.
    pub visible: bool,
    /// Current command input.
    pub input: String,
    /// Command history.
    pub history: Vec<String>,
    /// History index for up/down navigation.
    pub history_index: Option<usize>,
    /// Output messages.
    pub output: Vec<(OutputLevel, String)>,
    /// God mode enabled.
    pub god_mode: bool,
    /// Game speed multiplier.
    pub game_speed: f32,
}

impl Default for DebugConsoleState {
    fn default() -> Self {
        Self {
            visible: false,
            input: String::new(),
            history: Vec::new(),
            history_index: None,
            output: Vec::new(),
            god_mode: false,
            game_speed: 1.0, // Normal speed
        }
    }
}

/// Output message level for console.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputLevel {
    /// Informational message.
    Info,
    /// Success message.
    Success,
    /// Warning message.
    Warning,
    /// Error message.
    Error,
}

impl OutputLevel {
    fn color(&self) -> egui::Color32 {
        match self {
            Self::Info => egui::Color32::LIGHT_GRAY,
            Self::Success => egui::Color32::GREEN,
            Self::Warning => egui::Color32::YELLOW,
            Self::Error => egui::Color32::RED,
        }
    }
}

/// Toggle console visibility with backtick key.
fn toggle_console(keyboard: Res<ButtonInput<KeyCode>>, mut state: ResMut<DebugConsoleState>) {
    if keyboard.just_pressed(KeyCode::Backquote) {
        state.visible = !state.visible;
        if state.visible {
            state.input.clear();
        }
    }
}

/// Render the debug console UI.
fn render_console(
    mut commands: Commands,
    mut state: ResMut<DebugConsoleState>,
    mut contexts: EguiContexts,
    mut resources: ResMut<PlayerResources>,
    player_faction: Res<PlayerFaction>,
    mut selected: Query<(Entity, &mut GamePosition, &GameFaction), With<Selected>>,
    mut all_units: Query<(Entity, &mut GameHealth, &GameFaction)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<GameState>,
    mut time: ResMut<Time<Virtual>>,
) {
    if !state.visible {
        return;
    }

    let ctx = contexts.ctx_mut();

    egui::Window::new("Debug Console")
        .default_width(500.0)
        .default_height(300.0)
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            // Output area (scrollable)
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for (level, msg) in &state.output {
                        ui.colored_label(level.color(), msg);
                    }
                });

            ui.separator();

            // Status bar
            ui.horizontal(|ui| {
                ui.label(format!("Speed: {:.1}x", state.game_speed));
                if state.god_mode {
                    ui.colored_label(egui::Color32::GOLD, "GOD MODE");
                }
            });

            ui.separator();

            // Input field
            let _response = ui.horizontal(|ui| {
                ui.label(">");
                ui.text_edit_singleline(&mut state.input)
            });

            // Handle enter key
            if keyboard.just_pressed(KeyCode::Enter) && !state.input.is_empty() {
                let cmd = state.input.clone();
                state.history.push(cmd.clone());
                state.history_index = None;

                execute_command(
                    &cmd,
                    &mut commands,
                    &mut state,
                    &mut resources,
                    &player_faction,
                    &mut selected,
                    &mut all_units,
                    &mut game_state,
                    &mut time,
                );

                state.input.clear();
            }

            // Handle up/down for history
            if keyboard.just_pressed(KeyCode::ArrowUp) && !state.history.is_empty() {
                let idx = match state.history_index {
                    Some(i) if i > 0 => i - 1,
                    Some(i) => i,
                    None => state.history.len() - 1,
                };
                state.history_index = Some(idx);
                state.input = state.history[idx].clone();
            }
            if keyboard.just_pressed(KeyCode::ArrowDown) {
                if let Some(idx) = state.history_index {
                    if idx + 1 < state.history.len() {
                        state.history_index = Some(idx + 1);
                        state.input = state.history[idx + 1].clone();
                    } else {
                        state.history_index = None;
                        state.input.clear();
                    }
                }
            }

            // Help message
            ui.separator();
            ui.collapsing("Commands", |ui| {
                ui.label("help - Show this help");
                ui.label("spawn <type> [x] [y] - Spawn unit (infantry, harvester, ranger)");
                ui.label("building <type> [x] [y] - Spawn building (depot, barracks, supply, techlab, turret)");
                ui.label("kill - Kill selected units");
                ui.label("kill_all - Kill all enemy units");
                ui.label("teleport <x> <y> - Teleport selected units");
                ui.label("god - Toggle god mode (invincibility)");
                ui.label("resources <amount> - Set feedstock");
                ui.label("speed <multiplier> - Set game speed (affects simulation)");
                ui.label("win - Trigger victory");
                ui.label("lose - Trigger defeat");
                ui.label("clear - Clear console output");
            });
        });
}

/// Execute a console command.
#[allow(clippy::too_many_arguments)]
fn execute_command(
    cmd: &str,
    commands: &mut Commands,
    state: &mut ResMut<DebugConsoleState>,
    resources: &mut ResMut<PlayerResources>,
    player_faction: &Res<PlayerFaction>,
    selected: &mut Query<(Entity, &mut GamePosition, &GameFaction), With<Selected>>,
    all_units: &mut Query<(Entity, &mut GameHealth, &GameFaction)>,
    game_state: &mut ResMut<GameState>,
    time: &mut ResMut<Time<Virtual>>,
) {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    let cmd_name = parts[0].to_lowercase();
    let args = &parts[1..];

    state.output.push((OutputLevel::Info, format!("> {}", cmd)));

    match cmd_name.as_str() {
        "help" => {
            state.output.push((
                OutputLevel::Info,
                "Available commands: spawn, kill, kill_all, teleport, god, resources, speed, win, lose, clear".to_string(),
            ));
        }

        "spawn" => {
            if args.is_empty() {
                state.output.push((
                    OutputLevel::Error,
                    "Usage: spawn <type> [x] [y]".to_string(),
                ));
                return;
            }

            let unit_type = args[0].to_lowercase();
            let x: f32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(100.0);
            let y: f32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(100.0);
            let pos = Vec2::new(x, y);

            match unit_type.as_str() {
                "infantry" => {
                    commands.spawn(UnitBundle::new(pos, player_faction.faction, 100));
                    state.output.push((
                        OutputLevel::Success,
                        format!("Spawned infantry at ({}, {})", x, y),
                    ));
                }
                "harvester" => {
                    commands.spawn(HarvesterBundle::new(pos, player_faction.faction));
                    state.output.push((
                        OutputLevel::Success,
                        format!("Spawned harvester at ({}, {})", x, y),
                    ));
                }
                "ranger" => {
                    commands.spawn(UnitBundle::new(pos, player_faction.faction, 60));
                    state.output.push((
                        OutputLevel::Success,
                        format!("Spawned ranger at ({}, {})", x, y),
                    ));
                }
                _ => {
                    state.output.push((
                        OutputLevel::Error,
                        format!("Unknown unit type: {}", unit_type),
                    ));
                }
            }
        }

        "kill" => {
            let count = selected.iter().count();
            for (entity, _, _) in selected.iter() {
                commands.entity(entity).despawn_recursive();
            }
            state.output.push((
                OutputLevel::Success,
                format!("Killed {} selected units", count),
            ));
        }

        "kill_all" => {
            let mut count = 0;
            for (entity, _, faction) in all_units.iter() {
                if faction.faction != player_faction.faction {
                    commands.entity(entity).despawn_recursive();
                    count += 1;
                }
            }
            state.output.push((
                OutputLevel::Success,
                format!("Killed {} enemy units", count),
            ));
        }

        "teleport" => {
            if args.len() < 2 {
                state
                    .output
                    .push((OutputLevel::Error, "Usage: teleport <x> <y>".to_string()));
                return;
            }

            let Ok(x) = args[0].parse::<f32>() else {
                state
                    .output
                    .push((OutputLevel::Error, "Invalid x coordinate".to_string()));
                return;
            };
            let Ok(y) = args[1].parse::<f32>() else {
                state
                    .output
                    .push((OutputLevel::Error, "Invalid y coordinate".to_string()));
                return;
            };

            let new_pos = Vec2Fixed::new(Fixed::from_num(x), Fixed::from_num(y));
            let mut count = 0;

            for (_, mut pos, _) in selected.iter_mut() {
                pos.value = new_pos;
                count += 1;
            }

            if count > 0 {
                state.output.push((
                    OutputLevel::Success,
                    format!("Teleported {} units to ({}, {})", count, x, y),
                ));
            } else {
                state.output.push((
                    OutputLevel::Warning,
                    "No units selected to teleport".to_string(),
                ));
            }
        }

        "god" => {
            state.god_mode = !state.god_mode;
            if state.god_mode {
                state.output.push((
                    OutputLevel::Success,
                    "God mode ENABLED - Player units are invincible".to_string(),
                ));
            } else {
                state
                    .output
                    .push((OutputLevel::Info, "God mode DISABLED".to_string()));
            }
        }

        "resources" | "res" => {
            if args.is_empty() {
                state.output.push((
                    OutputLevel::Info,
                    format!("Current resources: {}", resources.feedstock),
                ));
                return;
            }

            let Ok(amount) = args[0].parse::<i32>() else {
                state
                    .output
                    .push((OutputLevel::Error, "Invalid amount".to_string()));
                return;
            };

            resources.feedstock = amount;
            state
                .output
                .push((OutputLevel::Success, format!("Set resources to {}", amount)));
        }

        "speed" => {
            if args.is_empty() {
                let current_speed = state.game_speed;
                state.output.push((
                    OutputLevel::Info,
                    format!("Current speed: {}x", current_speed),
                ));
                return;
            }

            let Ok(speed) = args[0].parse::<f32>() else {
                state
                    .output
                    .push((OutputLevel::Error, "Invalid speed".to_string()));
                return;
            };

            let new_speed = speed.clamp(0.1, 10.0);
            state.game_speed = new_speed;
            time.set_relative_speed(new_speed);
            state.output.push((
                OutputLevel::Success,
                format!("Game speed set to {}x", new_speed),
            ));
        }

        "win" => {
            **game_state = GameState::Victory;
            state
                .output
                .push((OutputLevel::Success, "Victory triggered!".to_string()));
        }

        "lose" => {
            **game_state = GameState::Defeat;
            state
                .output
                .push((OutputLevel::Success, "Defeat triggered!".to_string()));
        }

        "building" | "build" => {
            if args.is_empty() {
                state.output.push((
                    OutputLevel::Error,
                    "Usage: building <type> [x] [y]".to_string(),
                ));
                state.output.push((
                    OutputLevel::Info,
                    "Types: depot, barracks, supply, techlab, turret".to_string(),
                ));
                return;
            }

            let building_type = args[0].to_lowercase();
            let x: f32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(200.0);
            let y: f32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(200.0);
            let pos = Vec2::new(x, y);

            match building_type.as_str() {
                "depot" => {
                    commands.spawn(DepotBundle::new(pos, player_faction.faction));
                    state.output.push((
                        OutputLevel::Success,
                        format!("Spawned depot at ({}, {})", x, y),
                    ));
                }
                "barracks" => {
                    commands.spawn(BarracksBundle::new(pos, player_faction.faction));
                    state.output.push((
                        OutputLevel::Success,
                        format!("Spawned barracks at ({}, {})", x, y),
                    ));
                }
                "supply" | "supplydepot" => {
                    commands.spawn(SupplyDepotBundle::new(pos, player_faction.faction));
                    state.output.push((
                        OutputLevel::Success,
                        format!("Spawned supply depot at ({}, {})", x, y),
                    ));
                }
                "techlab" | "tech" => {
                    commands.spawn(TechLabBundle::new(pos, player_faction.faction));
                    state.output.push((
                        OutputLevel::Success,
                        format!("Spawned tech lab at ({}, {})", x, y),
                    ));
                }
                "turret" => {
                    commands.spawn(TurretBundle::new(pos, player_faction.faction));
                    state.output.push((
                        OutputLevel::Success,
                        format!("Spawned turret at ({}, {})", x, y),
                    ));
                }
                _ => {
                    state.output.push((
                        OutputLevel::Error,
                        format!("Unknown building type: {}", building_type),
                    ));
                }
            }
        }

        "clear" => {
            state.output.clear();
        }

        _ => {
            state.output.push((
                OutputLevel::Error,
                format!(
                    "Unknown command: {}. Type 'help' for available commands.",
                    cmd_name
                ),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::BuildingType;

    #[test]
    fn test_output_level_colors() {
        // Just verify the colors are different
        assert_ne!(OutputLevel::Info.color(), OutputLevel::Error.color());
        assert_ne!(OutputLevel::Success.color(), OutputLevel::Warning.color());
    }

    #[test]
    fn test_console_state_default() {
        let state = DebugConsoleState::default();
        assert!(!state.visible);
        assert!(state.input.is_empty());
        assert!(state.history.is_empty());
        assert!(state.output.is_empty());
        assert!(!state.god_mode);
        assert_eq!(state.game_speed, 1.0); // Normal speed
    }

    #[test]
    fn test_building_type_values() {
        // Verify building types have expected costs
        assert_eq!(BuildingType::Depot.cost(), 400);
        assert_eq!(BuildingType::Barracks.cost(), 150);
        assert_eq!(BuildingType::Turret.cost(), 75);
    }
}
