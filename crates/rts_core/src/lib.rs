//! # RTS Core
//!
//! Deterministic game simulation core for Post-Scarcity RTS.
//!
//! This crate contains **only** deterministic logic:
//! - No rendering
//! - No IO
//! - No system randomness
//! - No floating-point math (uses fixed-point)
//!
//! This separation enables:
//! - Lockstep multiplayer (identical simulation across clients)
//! - Headless server builds
//! - Replay systems
//! - Determinism testing
//!
//! ## Crate Structure
//!
//! - [`components`] - ECS component definitions
//! - [`systems`] - Simulation systems
//! - [`factions`] - Faction definitions and mechanics
//! - [`simulation`] - Core simulation loop
//! - [`math`] - Fixed-point math utilities

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]

pub mod buildings;
pub mod components;
pub mod economy;
pub mod error;
pub mod factions;
pub mod math;
pub mod pathfinding;
pub mod production;
pub mod simulation;
pub mod systems;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::components::*;
    pub use crate::economy::{
        Depot, EconomyEvent, Feedstock, Harvester, HarvesterState, PlayerEconomy, ResourceNode,
    };
    pub use crate::error::{GameError, Result};
    pub use crate::factions::FactionId;
    pub use crate::math::Fixed;
    pub use crate::production::{
        BlueprintRegistry, Building, BuildingBlueprint, BuildingTypeId, ProductionError,
        ProductionEvent, ProductionItem, ProductionQueue, TechId, UnitBlueprint, UnitTypeId,
    };
    pub use crate::simulation::Simulation;
}
