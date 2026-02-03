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

use crate::bundles::{HarvesterBundle, UnitBundle};
use crate::components::{GameFaction, GameHealth, GamePosition, PlayerFaction, Selected};
use crate::economy::PlayerResources;

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
#[derive(Resource, Default)]
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
    mut selected: Query<(Entity, &GamePosition, &GameFaction), With<Selected>>,
    mut all_units: Query<(Entity, &mut GameHealth, &GameFaction)>,
    keyboard: Res<ButtonInput<KeyCode>>,
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
                ui.label("spawn <type> [x] [y] - Spawn unit (infantry, harvester)");
                ui.label("kill - Kill selected units");
                ui.label("kill_all - Kill all enemy units");
                ui.label("teleport <x> <y> - Teleport selected units");
                ui.label("god - Toggle god mode (invincibility)");
                ui.label("resources <amount> - Set feedstock");
                ui.label("speed <multiplier> - Set game speed");
                ui.label("win - Trigger victory");
                ui.label("lose - Trigger defeat");
                ui.label("clear - Clear console output");
            });
        });
}

/// Execute a console command.
fn execute_command(
    cmd: &str,
    commands: &mut Commands,
    state: &mut ResMut<DebugConsoleState>,
    resources: &mut ResMut<PlayerResources>,
    player_faction: &Res<PlayerFaction>,
    selected: &mut Query<(Entity, &GamePosition, &GameFaction), With<Selected>>,
    all_units: &mut Query<(Entity, &mut GameHealth, &GameFaction)>,
) {
    let parts: Vec<&str> = cmd.trim().split_whitespace().collect();
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

            let _new_pos = Vec2Fixed::new(Fixed::from_num(x), Fixed::from_num(y));
            let count = selected.iter().count();

            // Note: We can't mutate GamePosition here since we only have & not &mut
            // This would require a separate system or different query structure
            state.output.push((
                OutputLevel::Warning,
                format!(
                    "Teleport not fully implemented - would move {} units to ({}, {})",
                    count, x, y
                ),
            ));
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
            state.output.push((
                OutputLevel::Success,
                format!("Game speed set to {}x", new_speed),
            ));
            state.output.push((
                OutputLevel::Warning,
                "Note: Speed change not yet wired to simulation".to_string(),
            ));
        }

        "win" => {
            state
                .output
                .push((OutputLevel::Success, "Victory triggered!".to_string()));
            state.output.push((
                OutputLevel::Warning,
                "Note: Victory trigger not yet wired to game state".to_string(),
            ));
        }

        "lose" => {
            state
                .output
                .push((OutputLevel::Success, "Defeat triggered!".to_string()));
            state.output.push((
                OutputLevel::Warning,
                "Note: Defeat trigger not yet wired to game state".to_string(),
            ));
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
        assert_eq!(state.game_speed, 0.0); // Default f32
    }
}
