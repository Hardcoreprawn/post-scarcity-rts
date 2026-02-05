//! Victory condition tests.
//!
//! Tests for win/lose detection based on core simulation events.

use bevy::prelude::*;
use rts_core::factions::FactionId;

use rts_game::components::PlayerFaction;
use rts_game::simulation::CoreSimulation;
use rts_game::victory::{check_victory_conditions, GameState, MatchStats};

fn setup_app(player_faction: FactionId) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<GameState>();
    app.init_resource::<MatchStats>();
    app.insert_resource(PlayerFaction::new(player_faction));
    app.insert_resource(CoreSimulation::default());
    app.add_systems(Update, check_victory_conditions);
    app
}

#[test]
fn game_starts_in_playing_state() {
    let app = setup_app(FactionId::Continuity);
    let state = app.world().resource::<GameState>();
    assert_eq!(*state, GameState::Playing);
}

#[test]
fn victory_when_core_reports_player_winner() {
    let mut app = setup_app(FactionId::Continuity);
    app.world_mut()
        .resource_mut::<CoreSimulation>()
        .last_events
        .game_end = Some(FactionId::Continuity);

    app.update();

    let state = app.world().resource::<GameState>();
    assert_eq!(*state, GameState::Victory);
}

#[test]
fn defeat_when_core_reports_enemy_winner() {
    let mut app = setup_app(FactionId::Continuity);
    app.world_mut()
        .resource_mut::<CoreSimulation>()
        .last_events
        .game_end = Some(FactionId::Collegium);

    app.update();

    let state = app.world().resource::<GameState>();
    assert_eq!(*state, GameState::Defeat);
}

#[test]
fn no_result_when_no_winner_reported() {
    let mut app = setup_app(FactionId::Continuity);
    app.update();

    let state = app.world().resource::<GameState>();
    assert_eq!(*state, GameState::Playing);
}
