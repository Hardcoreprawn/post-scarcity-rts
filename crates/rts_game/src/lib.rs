//! # RTS Game
//!
//! Main game client for Post-Scarcity RTS.
//!
//! This crate integrates the deterministic core with Bevy
//! for rendering, UI, audio, and input handling.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use bevy::log::LogPlugin;
use bevy::prelude::*;
use rts_core::factions::FactionId;
use rts_core::math::{Fixed, Vec2Fixed};

use crate::components::{Collider, GamePosition, PlayerFaction, Stationary};

// Core modules
pub mod ai;
pub mod bundles;
pub mod camera;
pub mod combat;
pub mod components;
pub mod construction;
pub mod data_loader;
pub mod economy;
pub mod input;
pub mod plugins;
pub mod production;
pub mod render;
pub mod selection;
pub mod simulation;
pub mod sprites;
pub mod ui;
pub mod victory;

use components::UnderConstruction;
pub use data_loader::{BevyUnitKindRegistry, FactionDataPlugin, FactionRegistry};
pub use plugins::HeadlessGamePlugins;
use plugins::{
    DepotBundle, GamePlugins, HarvesterBundle, ResourceNodeBundle, TurretBundle, UnitBundle,
};

/// Run the game.
///
/// # Errors
///
/// Returns an error if the game fails to initialize.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Post-Scarcity RTS".into(),
                    resolution: (1920.0, 1080.0).into(),
                    ..default()
                }),
                ..default()
            })
            .disable::<LogPlugin>(), // Logging already initialized in main.rs
    );

    // Add game plugins (camera, selection, input, rendering)
    app.add_plugins(GamePlugins);

    // Set background color (dark gray ground)
    app.insert_resource(ClearColor(Color::srgb(0.15, 0.15, 0.18)));

    // Add startup systems for visuals and test units
    app.add_systems(Startup, (spawn_ground_grid, spawn_test_units));

    #[cfg(feature = "dev-tools")]
    {
        // Add development tools
        tracing::info!("Development tools enabled");
    }

    app.run();

    Ok(())
}

/// Spawns test units for demonstration.
fn spawn_test_units(mut commands: Commands, player_faction: Res<PlayerFaction>) {
    let player = player_faction.faction;
    // Pick an enemy faction different from player
    let enemy = match player {
        FactionId::Continuity => FactionId::Collegium,
        FactionId::Collegium => FactionId::Continuity,
        FactionId::Tinkers => FactionId::BioSovereigns,
        FactionId::BioSovereigns => FactionId::Tinkers,
        FactionId::Zephyr => FactionId::Continuity,
    };

    tracing::info!("Player faction: {:?}, Enemy faction: {:?}", player, enemy);

    // ========================================================================
    // Starting Bases - Separated by ~1200 units
    // ========================================================================

    // Player base on the left side
    let player_base = Vec2::new(-600.0, 0.0);
    commands.spawn(DepotBundle::new(player_base, player));
    tracing::info!("Spawned player depot at {:?}", player_base);

    // Enemy base on the right side
    let enemy_base = Vec2::new(600.0, 0.0);
    commands.spawn(DepotBundle::new(enemy_base, enemy));
    tracing::info!("Spawned enemy depot at {:?}", enemy_base);

    // ========================================================================
    // Starting Turrets - Basic defense near each depot
    // ========================================================================

    // Player starting turret (spawned complete, not under construction)
    let player_turret = commands
        .spawn(TurretBundle::new(
            player_base + Vec2::new(50.0, 0.0),
            player,
        ))
        .id();
    commands.entity(player_turret).remove::<UnderConstruction>();

    // Enemy starting turret
    let enemy_turret = commands
        .spawn(TurretBundle::new(enemy_base + Vec2::new(-50.0, 0.0), enemy))
        .id();
    commands.entity(enemy_turret).remove::<UnderConstruction>();

    tracing::info!("Spawned 2 starting turrets");

    // ========================================================================
    // Starting Units - Near their respective bases
    // Players start with 3 basic infantry - must build production buildings
    // to train more units (barracks, vehicle depot, etc.)
    // ========================================================================

    // Player starting units (near player depot)
    let player_units = [
        player_base + Vec2::new(60.0, 40.0),
        player_base + Vec2::new(60.0, -40.0),
        player_base + Vec2::new(80.0, 0.0),
    ];
    for position in player_units {
        commands.spawn(UnitBundle::new(position, player, 100));
    }
    tracing::info!("Spawned {} player combat units", player_units.len());

    // Enemy starting units (near enemy depot)
    let enemy_units = [
        enemy_base + Vec2::new(-60.0, 40.0),
        enemy_base + Vec2::new(-60.0, -40.0),
        enemy_base + Vec2::new(-80.0, 0.0),
    ];
    for position in enemy_units {
        commands.spawn(UnitBundle::new(position, enemy, 100));
    }
    tracing::info!("Spawned {} enemy combat units", enemy_units.len());

    // ========================================================================
    // Harvesters - Near their respective bases
    // ========================================================================

    // Player harvester (just 1 to start - build more at depot)
    commands.spawn(HarvesterBundle::new(
        player_base + Vec2::new(-40.0, 0.0),
        player,
    ));
    tracing::info!("Spawned 1 player harvester");

    // Enemy harvesters (AI gets 2 for challenge)
    let enemy_harvesters = [
        enemy_base + Vec2::new(-20.0, 50.0),
        enemy_base + Vec2::new(-20.0, -50.0),
    ];
    for position in enemy_harvesters {
        commands.spawn(HarvesterBundle::new(position, enemy));
    }
    tracing::info!("Spawned {} enemy harvesters", enemy_harvesters.len());

    // ========================================================================
    // Resource Nodes
    // ========================================================================

    // Permanent resource nodes near each base (infinite but degrading yield)
    commands.spawn(ResourceNodeBundle::permanent(
        player_base + Vec2::new(-80.0, 80.0),
        3,
    ));
    commands.spawn(ResourceNodeBundle::permanent(
        enemy_base + Vec2::new(80.0, 80.0),
        3,
    ));
    tracing::info!("Spawned 2 permanent resource nodes (near bases)");

    // Temporary resource nodes in the contested center - forces expansion
    let temporary_nodes = [
        // Central contested zone
        (Vec2::new(0.0, 0.0), 1500),     // Dead center - high value
        (Vec2::new(-100.0, 80.0), 800),  // Center-left
        (Vec2::new(100.0, 80.0), 800),   // Center-right
        (Vec2::new(-100.0, -80.0), 800), // Center-left bottom
        (Vec2::new(100.0, -80.0), 800),  // Center-right bottom
        // Northern expansion
        (Vec2::new(-200.0, 200.0), 600),
        (Vec2::new(0.0, 250.0), 1000),
        (Vec2::new(200.0, 200.0), 600),
        // Southern expansion
        (Vec2::new(-200.0, -200.0), 600),
        (Vec2::new(0.0, -250.0), 1000),
        (Vec2::new(200.0, -200.0), 600),
    ];

    for (position, remaining) in temporary_nodes {
        commands.spawn(ResourceNodeBundle::temporary(position, remaining));
    }
    tracing::info!("Spawned {} temporary resource nodes", temporary_nodes.len());

    // ========================================================================
    // Terrain Features - Visual obstacles (hint at future pathfinding)
    // ========================================================================

    // Rocky outcrops along the center line - forces flanking later
    let terrain_features = [
        // Central ridge - runs north-south
        Vec2::new(0.0, 150.0),
        Vec2::new(0.0, -150.0),
        // Flanking rocks
        Vec2::new(-250.0, 100.0),
        Vec2::new(-250.0, -100.0),
        Vec2::new(250.0, 100.0),
        Vec2::new(250.0, -100.0),
    ];

    for position in terrain_features {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.3, 0.28, 0.25), // Rocky brown
                    custom_size: Some(Vec2::new(80.0, 60.0)),
                    ..default()
                },
                transform: Transform::from_translation(position.extend(-0.8)), // Behind everything
                ..default()
            },
            GamePosition::new(Vec2Fixed::new(
                Fixed::from_num(position.x),
                Fixed::from_num(position.y),
            )),
            Collider::new(80.0, 60.0),
            Stationary,
        ));
    }
    tracing::info!("Spawned {} terrain features", terrain_features.len());
}

/// Spawns a visible ground grid for reference.
fn spawn_ground_grid(mut commands: Commands) {
    // Create ground tiles as a visible grid
    const GRID_SIZE: i32 = 20;
    const TILE_SIZE: f32 = 64.0;

    for x in -GRID_SIZE..=GRID_SIZE {
        for y in -GRID_SIZE..=GRID_SIZE {
            // Checker pattern with subtle color variation
            let is_light = (x + y) % 2 == 0;
            let color = if is_light {
                Color::srgb(0.22, 0.24, 0.28)
            } else {
                Color::srgb(0.18, 0.20, 0.24)
            };

            commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(
                    x as f32 * TILE_SIZE,
                    y as f32 * TILE_SIZE,
                    -1.0, // Behind units
                )),
                ..default()
            });
        }
    }

    tracing::info!("Spawned ground grid");
}
