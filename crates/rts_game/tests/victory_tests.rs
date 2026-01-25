//! Victory condition tests.
//!
//! Tests for win/lose detection based on depot destruction.

use bevy::prelude::*;
use rts_core::factions::FactionId;

// Re-create minimal components for testing without full game dependency.
// This mirrors the structure in the game but avoids circular dependencies.

/// Minimal depot marker for testing.
#[derive(Component, Debug, Clone, Copy, Default)]
struct TestDepot;

/// Minimal faction component for testing.
#[derive(Component, Debug, Clone, Copy)]
struct TestFaction {
    faction: FactionId,
}

/// Minimal health component for testing.
#[derive(Component, Debug, Clone, Copy)]
struct TestHealth {
    current: u32,
    #[allow(dead_code)] // Kept for API parity with GameHealth
    max: u32,
}

impl TestHealth {
    fn new(max: u32) -> Self {
        Self { current: max, max }
    }

    fn dead() -> Self {
        Self {
            current: 0,
            max: 100,
        }
    }
}

/// Resource tracking player faction for testing.
#[derive(Resource, Debug, Clone, Copy)]
struct TestPlayerFaction {
    faction: FactionId,
}

impl Default for TestPlayerFaction {
    fn default() -> Self {
        Self {
            faction: FactionId::Continuity,
        }
    }
}

/// Game state for testing.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
enum TestGameState {
    #[default]
    Playing,
    Victory,
    Defeat,
}

/// Simple victory check logic (mirrors the actual implementation).
fn check_test_victory(
    mut game_state: ResMut<TestGameState>,
    player_faction: Res<TestPlayerFaction>,
    depots: Query<(&TestFaction, &TestHealth), With<TestDepot>>,
) {
    if *game_state != TestGameState::Playing {
        return;
    }

    let player_id = player_faction.faction;
    let mut player_has_depot = false;
    let mut enemy_has_depot = false;

    for (faction, health) in depots.iter() {
        if health.current > 0 {
            if faction.faction == player_id {
                player_has_depot = true;
            } else {
                enemy_has_depot = true;
            }
        }
    }

    if !player_has_depot {
        *game_state = TestGameState::Defeat;
    } else if !enemy_has_depot {
        *game_state = TestGameState::Victory;
    }
}

// =============================================================================
// Victory Detection Tests
// =============================================================================

#[test]
fn test_game_starts_in_playing_state() {
    let mut app = App::new();
    app.init_resource::<TestGameState>();

    let state = app.world().resource::<TestGameState>();
    assert_eq!(*state, TestGameState::Playing);
}

#[test]
fn test_victory_when_enemy_depot_destroyed() {
    let mut app = App::new();

    // Setup resources
    app.init_resource::<TestGameState>();
    app.insert_resource(TestPlayerFaction {
        faction: FactionId::Continuity,
    });
    app.add_systems(Update, check_test_victory);

    // Spawn player depot (alive)
    app.world_mut().spawn((
        TestDepot,
        TestFaction {
            faction: FactionId::Continuity,
        },
        TestHealth::new(1500),
    ));

    // Spawn enemy depot (dead - 0 health)
    app.world_mut().spawn((
        TestDepot,
        TestFaction {
            faction: FactionId::Collegium,
        },
        TestHealth::dead(),
    ));

    // Run update
    app.update();

    // Should be victory
    let state = app.world().resource::<TestGameState>();
    assert_eq!(
        *state,
        TestGameState::Victory,
        "Player should win when enemy depot is destroyed"
    );
}

#[test]
fn test_defeat_when_player_depot_destroyed() {
    let mut app = App::new();

    app.init_resource::<TestGameState>();
    app.insert_resource(TestPlayerFaction {
        faction: FactionId::Continuity,
    });
    app.add_systems(Update, check_test_victory);

    // Spawn player depot (dead)
    app.world_mut().spawn((
        TestDepot,
        TestFaction {
            faction: FactionId::Continuity,
        },
        TestHealth::dead(),
    ));

    // Spawn enemy depot (alive)
    app.world_mut().spawn((
        TestDepot,
        TestFaction {
            faction: FactionId::Collegium,
        },
        TestHealth::new(1500),
    ));

    app.update();

    let state = app.world().resource::<TestGameState>();
    assert_eq!(
        *state,
        TestGameState::Defeat,
        "Player should lose when their depot is destroyed"
    );
}

#[test]
fn test_no_result_while_both_depots_alive() {
    let mut app = App::new();

    app.init_resource::<TestGameState>();
    app.insert_resource(TestPlayerFaction {
        faction: FactionId::Continuity,
    });
    app.add_systems(Update, check_test_victory);

    // Both depots alive
    app.world_mut().spawn((
        TestDepot,
        TestFaction {
            faction: FactionId::Continuity,
        },
        TestHealth::new(1500),
    ));
    app.world_mut().spawn((
        TestDepot,
        TestFaction {
            faction: FactionId::Collegium,
        },
        TestHealth::new(1500),
    ));

    app.update();

    let state = app.world().resource::<TestGameState>();
    assert_eq!(
        *state,
        TestGameState::Playing,
        "Game should continue while both depots are alive"
    );
}

#[test]
fn test_victory_with_multiple_enemy_depots() {
    let mut app = App::new();

    app.init_resource::<TestGameState>();
    app.insert_resource(TestPlayerFaction {
        faction: FactionId::Continuity,
    });
    app.add_systems(Update, check_test_victory);

    // Player depot alive
    app.world_mut().spawn((
        TestDepot,
        TestFaction {
            faction: FactionId::Continuity,
        },
        TestHealth::new(1500),
    ));

    // Multiple enemy depots - all must be dead for victory
    app.world_mut().spawn((
        TestDepot,
        TestFaction {
            faction: FactionId::Collegium,
        },
        TestHealth::dead(),
    ));
    app.world_mut().spawn((
        TestDepot,
        TestFaction {
            faction: FactionId::Collegium,
        },
        TestHealth::dead(),
    ));

    app.update();

    let state = app.world().resource::<TestGameState>();
    assert_eq!(
        *state,
        TestGameState::Victory,
        "Victory requires ALL enemy depots destroyed"
    );
}

#[test]
fn test_no_victory_if_one_enemy_depot_survives() {
    let mut app = App::new();

    app.init_resource::<TestGameState>();
    app.insert_resource(TestPlayerFaction {
        faction: FactionId::Continuity,
    });
    app.add_systems(Update, check_test_victory);

    // Player depot alive
    app.world_mut().spawn((
        TestDepot,
        TestFaction {
            faction: FactionId::Continuity,
        },
        TestHealth::new(1500),
    ));

    // One enemy dead, one alive
    app.world_mut().spawn((
        TestDepot,
        TestFaction {
            faction: FactionId::Collegium,
        },
        TestHealth::dead(),
    ));
    app.world_mut().spawn((
        TestDepot,
        TestFaction {
            faction: FactionId::Collegium,
        },
        TestHealth::new(500), // Damaged but alive
    ));

    app.update();

    let state = app.world().resource::<TestGameState>();
    assert_eq!(
        *state,
        TestGameState::Playing,
        "Game continues if any enemy depot survives"
    );
}

#[test]
fn test_defeat_priority_over_victory() {
    // Edge case: mutual annihilation - both depots destroyed same frame
    // This shouldn't happen normally, but if it does, defeat takes priority
    // (player's depot destruction is checked first in current implementation)
    let mut app = App::new();

    app.init_resource::<TestGameState>();
    app.insert_resource(TestPlayerFaction {
        faction: FactionId::Continuity,
    });
    app.add_systems(Update, check_test_victory);

    // Both depots dead
    app.world_mut().spawn((
        TestDepot,
        TestFaction {
            faction: FactionId::Continuity,
        },
        TestHealth::dead(),
    ));
    app.world_mut().spawn((
        TestDepot,
        TestFaction {
            faction: FactionId::Collegium,
        },
        TestHealth::dead(),
    ));

    app.update();

    let state = app.world().resource::<TestGameState>();
    assert_eq!(
        *state,
        TestGameState::Defeat,
        "Mutual destruction should result in defeat"
    );
}

#[test]
fn test_state_does_not_change_after_victory() {
    let mut app = App::new();

    app.init_resource::<TestGameState>();
    app.insert_resource(TestPlayerFaction {
        faction: FactionId::Continuity,
    });
    app.add_systems(Update, check_test_victory);

    // Player alive, enemy dead
    app.world_mut().spawn((
        TestDepot,
        TestFaction {
            faction: FactionId::Continuity,
        },
        TestHealth::new(1500),
    ));
    app.world_mut().spawn((
        TestDepot,
        TestFaction {
            faction: FactionId::Collegium,
        },
        TestHealth::dead(),
    ));

    // First update - should trigger victory
    app.update();
    assert_eq!(
        *app.world().resource::<TestGameState>(),
        TestGameState::Victory
    );

    // Simulate respawning enemy (shouldn't happen, but test the guard)
    app.world_mut().spawn((
        TestDepot,
        TestFaction {
            faction: FactionId::Collegium,
        },
        TestHealth::new(1500),
    ));

    // Second update - should remain victory
    app.update();
    let state = app.world().resource::<TestGameState>();
    assert_eq!(
        *state,
        TestGameState::Victory,
        "Victory state should be permanent"
    );
}

#[test]
fn test_damaged_depot_still_counts_as_alive() {
    let mut app = App::new();

    app.init_resource::<TestGameState>();
    app.insert_resource(TestPlayerFaction {
        faction: FactionId::Continuity,
    });
    app.add_systems(Update, check_test_victory);

    // Player depot nearly dead (1 HP)
    app.world_mut().spawn((
        TestDepot,
        TestFaction {
            faction: FactionId::Continuity,
        },
        TestHealth {
            current: 1,
            max: 1500,
        },
    ));

    // Enemy depot nearly dead (1 HP)
    app.world_mut().spawn((
        TestDepot,
        TestFaction {
            faction: FactionId::Collegium,
        },
        TestHealth {
            current: 1,
            max: 1500,
        },
    ));

    app.update();

    let state = app.world().resource::<TestGameState>();
    assert_eq!(
        *state,
        TestGameState::Playing,
        "Damaged depots (HP > 0) should count as alive"
    );
}

// =============================================================================
// Multi-faction Tests
// =============================================================================

#[test]
fn test_victory_against_multiple_factions() {
    let mut app = App::new();

    app.init_resource::<TestGameState>();
    app.insert_resource(TestPlayerFaction {
        faction: FactionId::Continuity,
    });
    app.add_systems(Update, check_test_victory);

    // Player depot
    app.world_mut().spawn((
        TestDepot,
        TestFaction {
            faction: FactionId::Continuity,
        },
        TestHealth::new(1500),
    ));

    // Multiple enemy factions - all dead
    app.world_mut().spawn((
        TestDepot,
        TestFaction {
            faction: FactionId::Collegium,
        },
        TestHealth::dead(),
    ));
    app.world_mut().spawn((
        TestDepot,
        TestFaction {
            faction: FactionId::Tinkers,
        },
        TestHealth::dead(),
    ));
    app.world_mut().spawn((
        TestDepot,
        TestFaction {
            faction: FactionId::BioSovereigns,
        },
        TestHealth::dead(),
    ));

    app.update();

    let state = app.world().resource::<TestGameState>();
    assert_eq!(
        *state,
        TestGameState::Victory,
        "Should win when all enemy factions eliminated"
    );
}

#[test]
fn test_no_victory_if_any_enemy_faction_survives() {
    let mut app = App::new();

    app.init_resource::<TestGameState>();
    app.insert_resource(TestPlayerFaction {
        faction: FactionId::Continuity,
    });
    app.add_systems(Update, check_test_victory);

    // Player depot
    app.world_mut().spawn((
        TestDepot,
        TestFaction {
            faction: FactionId::Continuity,
        },
        TestHealth::new(1500),
    ));

    // Some enemies dead, one alive
    app.world_mut().spawn((
        TestDepot,
        TestFaction {
            faction: FactionId::Collegium,
        },
        TestHealth::dead(),
    ));
    app.world_mut().spawn((
        TestDepot,
        TestFaction {
            faction: FactionId::Tinkers,
        },
        TestHealth::new(1500), // Still alive!
    ));

    app.update();

    let state = app.world().resource::<TestGameState>();
    assert_eq!(
        *state,
        TestGameState::Playing,
        "Game continues if any enemy faction has depots"
    );
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_no_depots_at_all_means_defeat() {
    // Edge case: what if there are no depots spawned?
    // This is a degenerate case but should be handled gracefully.
    let mut app = App::new();

    app.init_resource::<TestGameState>();
    app.insert_resource(TestPlayerFaction {
        faction: FactionId::Continuity,
    });
    app.add_systems(Update, check_test_victory);

    // No depots spawned at all

    app.update();

    let state = app.world().resource::<TestGameState>();
    // With no player depots, player has lost
    assert_eq!(
        *state,
        TestGameState::Defeat,
        "No player depots should mean defeat"
    );
}

#[test]
fn test_player_faction_can_be_any_faction() {
    // Test that victory detection works for any player faction
    for player_faction in [
        FactionId::Continuity,
        FactionId::Collegium,
        FactionId::Tinkers,
        FactionId::BioSovereigns,
        FactionId::Zephyr,
    ] {
        let mut app = App::new();

        app.init_resource::<TestGameState>();
        app.insert_resource(TestPlayerFaction {
            faction: player_faction,
        });
        app.add_systems(Update, check_test_victory);

        // Player depot alive
        app.world_mut().spawn((
            TestDepot,
            TestFaction {
                faction: player_faction,
            },
            TestHealth::new(1500),
        ));

        // Enemy depot dead (pick a different faction)
        let enemy_faction = match player_faction {
            FactionId::Continuity => FactionId::Collegium,
            _ => FactionId::Continuity,
        };
        app.world_mut().spawn((
            TestDepot,
            TestFaction {
                faction: enemy_faction,
            },
            TestHealth::dead(),
        ));

        app.update();

        let state = app.world().resource::<TestGameState>();
        assert_eq!(
            *state,
            TestGameState::Victory,
            "Victory should work for player faction {:?}",
            player_faction
        );
    }
}
