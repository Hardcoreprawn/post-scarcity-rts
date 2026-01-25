//! Game plugins for Bevy.
//!
//! This module provides the main plugin group for the game client,
//! aggregating all gameplay plugins into a single registration point.

use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;

// Import plugins from submodules
use crate::ai::AiPlugin;
use crate::camera::CameraPlugin;
use crate::combat::CombatPlugin;
use crate::construction::ConstructionPlugin;
use crate::data_loader::FactionDataPlugin;
use crate::economy::EconomyPlugin;
use crate::input::InputPlugin;
use crate::production::ProductionPlugin;
use crate::render::RenderPlugin;
use crate::selection::SelectionPlugin;
use crate::simulation::SimulationPlugin;
use crate::sprites::SpriteLoaderPlugin;
use crate::ui::GameUiPlugin;
use crate::victory::VictoryPlugin;

// Re-export commonly used types for convenience
pub use crate::bundles::{
    faction_color, BarracksBundle, DepotBundle, HarvesterBundle, ResourceNodeBundle, TurretBundle,
    UnitBundle,
};
pub use crate::economy::PlayerResources;
pub use crate::input::InputMode;
pub use crate::selection::{SelectionHighlight, SelectionState};

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
            .add(FactionDataPlugin) // Load faction data first
            .add(SpriteLoaderPlugin) // Load sprites early
            .add(CameraPlugin)
            .add(SelectionPlugin)
            .add(InputPlugin)
            .add(RenderPlugin)
            .add(SimulationPlugin)
            .add(EconomyPlugin)
            .add(CombatPlugin)
            .add(ProductionPlugin)
            .add(ConstructionPlugin)
            .add(GameUiPlugin)
            .add(AiPlugin)
            .add(VictoryPlugin)
    }
}

/// Headless plugin group for simulation-only testing.
///
/// This runs the game logic without any rendering, camera, or input.
/// Useful for balance testing and automated verification.
///
/// # Example
/// ```ignore
/// App::new()
///     .add_plugins(MinimalPlugins)
///     .add_plugins(HeadlessGamePlugins)
///     .run();
/// ```
pub struct HeadlessGamePlugins;

impl PluginGroup for HeadlessGamePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(FactionDataPlugin)
            .add(SimulationPlugin)
            .add(EconomyPlugin)
            .add(CombatPlugin)
            .add(ProductionPlugin)
            .add(AiPlugin)
    }
}
