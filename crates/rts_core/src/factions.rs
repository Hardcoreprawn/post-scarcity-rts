//! Faction definitions and identifiers.

use serde::{Deserialize, Serialize};

/// Unique identifier for factions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FactionId {
    /// The Continuity Authority - bureaucratic stability through governance.
    Continuity,
    /// The Collegium - distributed academic cooperative.
    Collegium,
    /// The Tinkers' Union - maker movement mechanics.
    Tinkers,
    /// The Bio-Sovereigns - engineered ecosystems.
    BioSovereigns,
    /// The Zephyr Guild - aerial trade and mobility.
    Zephyr,
}

impl FactionId {
    /// Get the display name for this faction.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Continuity => "The Continuity Authority",
            Self::Collegium => "The Collegium",
            Self::Tinkers => "The Tinkers' Union",
            Self::BioSovereigns => "The Bio-Sovereigns",
            Self::Zephyr => "The Zephyr Guild",
        }
    }

    /// Get the short name for this faction.
    #[must_use]
    pub const fn short_name(&self) -> &'static str {
        match self {
            Self::Continuity => "Continuity",
            Self::Collegium => "Collegium",
            Self::Tinkers => "Tinkers",
            Self::BioSovereigns => "Bio-Sovereigns",
            Self::Zephyr => "Zephyr",
        }
    }
}
