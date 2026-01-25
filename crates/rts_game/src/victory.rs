//! Victory and defeat condition handling.
//!
//! Detects when the game ends and displays appropriate UI.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::components::{GameDepot, GameFaction, GameHealth, PlayerFaction};

/// Plugin for victory/defeat conditions.
pub struct VictoryPlugin;

impl Plugin for VictoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameState>()
            .init_resource::<MatchStats>()
            .add_systems(Update, check_victory_conditions)
            .add_systems(Update, victory_ui.after(check_victory_conditions));
    }
}

/// Current state of the game match.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameState {
    /// Game is in progress.
    #[default]
    Playing,
    /// Player has won.
    Victory,
    /// Player has lost.
    Defeat,
}

/// Statistics tracked during the match.
#[derive(Resource, Default, Clone, Debug)]
pub struct MatchStats {
    /// Total feedstock collected.
    pub feedstock_collected: i32,
    /// Total units produced.
    pub units_produced: i32,
    /// Total units lost.
    pub units_lost: i32,
    /// Total enemy units killed.
    pub enemies_killed: i32,
    /// Total buildings constructed.
    pub buildings_built: i32,
    /// Match duration in seconds.
    pub match_duration: f32,
}

/// Checks if victory or defeat conditions have been met.
///
/// Victory occurs when all enemy depots are destroyed.
/// Defeat occurs when all player depots are destroyed.
pub fn check_victory_conditions(
    mut game_state: ResMut<GameState>,
    player_faction: Res<PlayerFaction>,
    depots: Query<(&GameFaction, &GameHealth), With<GameDepot>>,
    time: Res<Time>,
    mut stats: ResMut<MatchStats>,
) {
    // Don't check if game already ended
    if *game_state != GameState::Playing {
        return;
    }

    // Track match duration
    stats.match_duration += time.delta_seconds();

    let player_faction_id = player_faction.faction;
    let mut player_has_depot = false;
    let mut enemy_has_depot = false;

    for (faction, health) in depots.iter() {
        // Only count living depots
        if health.current > 0 {
            if faction.faction == player_faction_id {
                player_has_depot = true;
            } else {
                enemy_has_depot = true;
            }
        }
    }

    // Check victory/defeat
    if !player_has_depot {
        *game_state = GameState::Defeat;
        tracing::info!("DEFEAT - Player depot destroyed!");
    } else if !enemy_has_depot {
        *game_state = GameState::Victory;
        tracing::info!("VICTORY - Enemy depot destroyed!");
    }
}

/// Displays victory/defeat UI overlay.
fn victory_ui(
    game_state: Res<GameState>,
    stats: Res<MatchStats>,
    mut egui_contexts: EguiContexts,
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    entities: Query<Entity, Without<Window>>,
) {
    if *game_state == GameState::Playing {
        return;
    }

    let Some(ctx) = egui_contexts.try_ctx_mut() else {
        return;
    };

    let is_victory = *game_state == GameState::Victory;
    let title = if is_victory { "VICTORY" } else { "DEFEAT" };
    let color = if is_victory {
        egui::Color32::from_rgb(50, 200, 50)
    } else {
        egui::Color32::from_rgb(200, 50, 50)
    };

    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200)))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);

                // Title
                ui.label(egui::RichText::new(title).size(72.0).color(color).strong());

                ui.add_space(40.0);

                // Match statistics
                ui.label(
                    egui::RichText::new("Match Statistics")
                        .size(24.0)
                        .color(egui::Color32::WHITE),
                );
                ui.add_space(20.0);

                let duration_mins = (stats.match_duration / 60.0).floor() as i32;
                let duration_secs = (stats.match_duration % 60.0).floor() as i32;

                ui.label(
                    egui::RichText::new(format!(
                        "Duration: {}:{:02}",
                        duration_mins, duration_secs
                    ))
                    .size(18.0)
                    .color(egui::Color32::LIGHT_GRAY),
                );

                ui.add_space(60.0);

                // Restart button
                let restart_btn = ui.add_sized(
                    [200.0, 50.0],
                    egui::Button::new(
                        egui::RichText::new("Play Again (R)")
                            .size(20.0)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_rgb(60, 60, 80)),
                );

                if restart_btn.clicked() || keyboard.just_pressed(KeyCode::KeyR) {
                    // Restart the game by despawning everything and re-running spawn
                    // For now, just reset game state - full restart requires more work
                    tracing::info!("Restart requested - respawning game...");

                    // Despawn all game entities (keeping windows)
                    for entity in entities.iter() {
                        commands.entity(entity).despawn_recursive();
                    }
                }

                ui.add_space(20.0);

                // Quit hint
                ui.label(
                    egui::RichText::new("Press ESC to quit")
                        .size(14.0)
                        .color(egui::Color32::GRAY),
                );
            });
        });
}
