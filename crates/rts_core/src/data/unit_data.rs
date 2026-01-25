//! Unit data structures for data-driven unit definitions.

use serde::{Deserialize, Serialize};

use crate::math::{fixed_serde, Fixed};

/// Combat statistics for a unit.
///
/// Optional combat data - units without combat stats are non-combatants
/// (e.g., harvesters, scouts).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CombatStats {
    /// Base damage per attack.
    pub damage: u32,

    /// Attack range in game units.
    #[serde(with = "fixed_serde")]
    pub range: Fixed,

    /// Cooldown between attacks in ticks.
    pub attack_cooldown: u32,

    /// Armor value that reduces incoming damage.
    #[serde(default)]
    pub armor: u32,
}

/// Data-driven unit definition.
///
/// Defines all properties of a unit type that can be loaded from
/// configuration files. Used to populate `UnitBlueprint` instances.
///
/// # Example RON
///
/// ```ron
/// UnitData(
///     id: "security_team",
///     name: "unit.continuity.security_team.name",
///     description: "unit.continuity.security_team.desc",
///     cost: 50,
///     build_time: 120,
///     health: 80,
///     speed: 42949672960,  // Fixed-point for 10.0
///     combat: Some(CombatStats(
///         damage: 12,
///         range: 21474836480,  // Fixed-point for 5.0
///         attack_cooldown: 30,
///         armor: 5,
///     )),
///     tech_required: [],
///     tier: 1,
/// )
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitData {
    /// Unique string identifier for this unit type.
    ///
    /// Used for referencing in other data files and for save/load.
    pub id: String,

    /// Localization key for the unit's display name.
    pub name: String,

    /// Localization key for the unit's description.
    pub description: String,

    /// Feedstock cost to produce this unit.
    pub cost: u32,

    /// Production time in simulation ticks.
    pub build_time: u32,

    /// Maximum health points.
    pub health: u32,

    /// Movement speed (fixed-point).
    #[serde(with = "fixed_serde")]
    pub speed: Fixed,

    /// Combat statistics (None for non-combat units).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub combat: Option<CombatStats>,

    /// Technologies required to produce this unit.
    #[serde(default)]
    pub tech_required: Vec<String>,

    /// Tech tier this unit belongs to (1, 2, or 3).
    #[serde(default = "default_tier")]
    pub tier: u8,

    /// Building ID that can produce this unit.
    #[serde(default)]
    pub produced_at: Vec<String>,

    /// Tags for categorization and targeting (e.g., "infantry", "vehicle", "air").
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Default tier for units without explicit tier.
const fn default_tier() -> u8 {
    1
}

impl UnitData {
    /// Check if this unit requires a specific technology.
    #[must_use]
    pub fn requires_tech(&self, tech_id: &str) -> bool {
        self.tech_required.iter().any(|t| t == tech_id)
    }

    /// Check if this unit has the specified tag.
    #[must_use]
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    /// Check if this unit can engage in combat.
    #[must_use]
    pub fn is_combatant(&self) -> bool {
        self.combat.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_unit() -> UnitData {
        UnitData {
            id: "test_unit".to_string(),
            name: "unit.test.name".to_string(),
            description: "unit.test.desc".to_string(),
            cost: 100,
            build_time: 60,
            health: 100,
            speed: Fixed::from_num(5),
            combat: Some(CombatStats {
                damage: 10,
                range: Fixed::from_num(3),
                attack_cooldown: 30,
                armor: 5,
            }),
            tech_required: vec!["enhanced_training".to_string()],
            tier: 1,
            produced_at: vec!["training_center".to_string()],
            tags: vec!["infantry".to_string()],
        }
    }

    #[test]
    fn test_requires_tech() {
        let unit = create_test_unit();
        assert!(unit.requires_tech("enhanced_training"));
        assert!(!unit.requires_tech("unknown_tech"));
    }

    #[test]
    fn test_has_tag() {
        let unit = create_test_unit();
        assert!(unit.has_tag("infantry"));
        assert!(!unit.has_tag("vehicle"));
    }

    #[test]
    fn test_is_combatant() {
        let mut unit = create_test_unit();
        assert!(unit.is_combatant());

        unit.combat = None;
        assert!(!unit.is_combatant());
    }
}
