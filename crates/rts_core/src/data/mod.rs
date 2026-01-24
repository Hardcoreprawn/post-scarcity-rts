//! Data structures for faction configuration.
//!
//! This module contains pure data structures that define faction units,
//! buildings, and tech trees. All structs are designed to be deserialized
//! from RON files.
//!
//! **Note:** This module contains no IO - it only defines data types.
//! File loading is handled by `rts_game`.

mod building_data;
mod faction_data;
mod tech_data;
mod unit_data;

pub use building_data::BuildingData;
pub use faction_data::{FactionData, StartingEntity};
pub use tech_data::{TechData, TechEffect, TechEffectType};
pub use unit_data::{CombatStats, UnitData};
