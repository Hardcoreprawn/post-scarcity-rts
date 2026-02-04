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

pub mod buildings;
pub mod combat;
pub mod components;
pub mod data;
pub mod economy;
pub mod error;
pub mod factions;
pub mod map_generation;
pub mod math;
pub mod pathfinding;
pub mod production;
pub mod replay;
pub mod simulation;
pub mod systems;
pub mod unit_kind;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::combat::{
        calculate_resistance_damage, convert_flat_armor_to_resistance, ArmorClass,
        ExtendedDamageType, ResistanceStats, WeaponSize, WeaponStats, MAX_RESISTANCE, MIN_DAMAGE,
    };
    pub use crate::components::*;
    pub use crate::data::{
        BuildingData, FactionData, TechData, TechEffect, TechEffectType, UnitData,
    };
    pub use crate::economy::{
        Depot, EconomyEvent, Feedstock, Harvester, HarvesterState, PlayerEconomy, ResourceNode,
    };
    pub use crate::error::{GameError, Result};
    pub use crate::factions::FactionId;
    pub use crate::map_generation::{
        generate_map, GeneratedMap, MapConfig, ResourcePlacement, SpawnPoint, SymmetryMode,
        TerrainCell,
    };
    pub use crate::math::Fixed;
    pub use crate::production::{
        BlueprintRegistry, Building, BuildingBlueprint, BuildingTypeId, ProductionError,
        ProductionEvent, ProductionItem, ProductionQueue, TechId, UnitBlueprint, UnitTypeId,
    };
    pub use crate::replay::{Replay, ReplayCommand, ReplayPlayer, REPLAY_VERSION};
    pub use crate::simulation::Simulation;
    pub use crate::unit_kind::{UnitKindId, UnitKindInfo, UnitKindRegistry, UnitRole};
}
