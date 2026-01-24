//! # RTS Game
//!
//! Main game client for Post-Scarcity RTS.
//!
//! This crate integrates the deterministic core with Bevy
//! for rendering, UI, audio, and input handling.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]

use bevy::prelude::*;
use rts_core::factions::FactionId;

pub mod data_loader;
pub mod plugins;

pub use data_loader::{FactionDataPlugin, FactionRegistry};
use plugins::{GamePlugins, UnitBundle};

/// Run the game.
///
/// # Errors
///
/// Returns an error if the game fails to initialize.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Post-Scarcity RTS".into(),
            resolution: (1920.0, 1080.0).into(),
            ..default()
        }),
        ..default()
    }));

    // Add game plugins (camera, selection, input, rendering)
    app.add_plugins(GamePlugins);

    // Set background color (dark gray ground)
    app.insert_resource(ClearColor(Color::srgb(0.15, 0.15, 0.18)));

    // Add startup system to spawn test units
    app.add_systems(Startup, spawn_test_units);

    #[cfg(feature = "dev-tools")]
    {
        // Add development tools
        tracing::info!("Development tools enabled");
    }

    app.run();

    Ok(())
}

/// Spawns test units for demonstration.
fn spawn_test_units(mut commands: Commands) {
    tracing::info!("Spawning test units...");

    // Spawn units for different factions at various positions
    let test_units = [
        // Continuity Authority (blue) - grouped on the left
        (Vec2::new(-200.0, 100.0), FactionId::Continuity, 100),
        (Vec2::new(-250.0, 50.0), FactionId::Continuity, 100),
        (Vec2::new(-200.0, 0.0), FactionId::Continuity, 80),
        // Collegium (gold) - grouped in the center
        (Vec2::new(0.0, 150.0), FactionId::Collegium, 120),
        (Vec2::new(50.0, 100.0), FactionId::Collegium, 120),
        // Tinkers (brown) - on the right
        (Vec2::new(200.0, -50.0), FactionId::Tinkers, 90),
        (Vec2::new(250.0, 0.0), FactionId::Tinkers, 90),
        // Bio-Sovereigns (green) - bottom
        (Vec2::new(-100.0, -200.0), FactionId::BioSovereigns, 150),
        // Zephyr (sky blue) - top right
        (Vec2::new(150.0, 200.0), FactionId::Zephyr, 70),
        (Vec2::new(200.0, 180.0), FactionId::Zephyr, 70),
    ];

    for (position, faction, health) in test_units {
        commands.spawn(UnitBundle::new(position, faction, health));
    }

    tracing::info!("Spawned {} test units", test_units.len());
}
