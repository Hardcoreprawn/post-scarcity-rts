//! Tech tree data structures for data-driven technology definitions.

use serde::{Deserialize, Serialize};

use crate::math::{fixed_serde, Fixed};

/// Type of effect a technology provides.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TechEffectType {
    /// Modify a stat by a percentage.
    StatModifierPercent {
        /// The stat to modify (e.g., "damage", "speed", "health").
        stat: String,
        /// Percentage modifier (e.g., 15 for +15%).
        percent: i32,
    },

    /// Modify a stat by a flat amount.
    StatModifierFlat {
        /// The stat to modify.
        stat: String,
        /// Flat amount to add.
        amount: i32,
    },

    /// Unlock a unit or building for production.
    Unlock {
        /// ID of the unit or building to unlock.
        target_id: String,
    },

    /// Grant a new ability to units.
    GrantAbility {
        /// ID of the ability to grant.
        ability_id: String,
        /// Target unit or building IDs.
        targets: Vec<String>,
    },

    /// Modify vision range.
    VisionModifier {
        /// Additional vision range.
        #[serde(with = "fixed_serde")]
        range: Fixed,
    },

    /// Modify production speed.
    ProductionSpeedModifier {
        /// Percentage modifier (e.g., 25 for +25% faster).
        percent: i32,
    },

    /// Modify resource gathering efficiency.
    GatheringModifier {
        /// Percentage modifier.
        percent: i32,
    },

    /// Modify unit/building cost.
    CostModifier {
        /// Percentage modifier (negative for reduction, e.g., -10 for -10% cost).
        percent: i32,
    },

    /// Custom effect handled by game logic.
    Custom {
        /// Effect identifier for custom handling.
        effect_id: String,
        /// Optional parameters.
        params: Vec<String>,
    },
}

/// Effect of researching a technology.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechEffect {
    /// Type of effect.
    pub effect_type: TechEffectType,

    /// Unit or building IDs this effect applies to.
    /// Empty means it applies globally or to the faction.
    #[serde(default)]
    pub applies_to: Vec<String>,

    /// Tags this effect applies to (e.g., "infantry", "vehicle").
    /// Alternative to specifying individual targets.
    #[serde(default)]
    pub applies_to_tags: Vec<String>,
}

/// Data-driven technology definition.
///
/// Defines all properties of a technology that can be loaded from
/// configuration files.
///
/// # Example RON
///
/// ```ron
/// TechData(
///     id: "enhanced_training",
///     name: "tech.continuity.enhanced_training.name",
///     description: "tech.continuity.enhanced_training.desc",
///     cost: 100,
///     research_time: 900,  // 30 seconds at 30 ticks/sec
///     effects: [
///         TechEffect(
///             effect_type: StatModifierPercent(stat: "damage", percent: 15),
///             applies_to_tags: ["infantry"],
///         ),
///     ],
///     prerequisites: [],
///     tier: 1,
///     exclusive_with: [],
///     researched_at: "research_institute",
/// )
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechData {
    /// Unique string identifier for this technology.
    pub id: String,

    /// Localization key for the technology's display name.
    pub name: String,

    /// Localization key for the technology's description.
    pub description: String,

    /// Feedstock cost to research.
    pub cost: i32,

    /// Research time in simulation ticks.
    pub research_time: u32,

    /// Effects granted when this technology is completed.
    #[serde(default)]
    pub effects: Vec<TechEffect>,

    /// Technology IDs that must be researched first.
    #[serde(default)]
    pub prerequisites: Vec<String>,

    /// Tech tier this technology belongs to (1, 2, or 3).
    #[serde(default = "default_tier")]
    pub tier: u8,

    /// Technologies that become unavailable if this is researched.
    /// Used for mutually exclusive branch choices.
    #[serde(default)]
    pub exclusive_with: Vec<String>,

    /// Building ID where this technology can be researched.
    #[serde(default)]
    pub researched_at: Option<String>,

    /// Whether this is a faction doctrine (major branch choice).
    #[serde(default)]
    pub is_doctrine: bool,

    /// Branch name for UI grouping.
    #[serde(default)]
    pub branch: Option<String>,

    /// Icon resource path for UI.
    #[serde(default)]
    pub icon: Option<String>,
}

/// Default tier for technologies without explicit tier.
const fn default_tier() -> u8 {
    1
}

impl TechData {
    /// Check if this technology has a specific prerequisite.
    #[must_use]
    pub fn requires(&self, tech_id: &str) -> bool {
        self.prerequisites.iter().any(|t| t == tech_id)
    }

    /// Check if this technology is mutually exclusive with another.
    #[must_use]
    pub fn excludes(&self, tech_id: &str) -> bool {
        self.exclusive_with.iter().any(|t| t == tech_id)
    }

    /// Check if this technology provides an effect that modifies a stat.
    #[must_use]
    pub fn modifies_stat(&self, stat: &str) -> bool {
        self.effects.iter().any(|e| match &e.effect_type {
            TechEffectType::StatModifierPercent { stat: s, .. } => s == stat,
            TechEffectType::StatModifierFlat { stat: s, .. } => s == stat,
            _ => false,
        })
    }

    /// Get all stat modifiers from this technology's effects.
    pub fn get_stat_modifiers(&self) -> Vec<(&str, i32, bool)> {
        self.effects
            .iter()
            .filter_map(|e| match &e.effect_type {
                TechEffectType::StatModifierPercent { stat, percent } => {
                    Some((stat.as_str(), *percent, true))
                }
                TechEffectType::StatModifierFlat { stat, amount } => {
                    Some((stat.as_str(), *amount, false))
                }
                _ => None,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tech() -> TechData {
        TechData {
            id: "enhanced_training".to_string(),
            name: "tech.test.name".to_string(),
            description: "tech.test.desc".to_string(),
            cost: 100,
            research_time: 900,
            effects: vec![TechEffect {
                effect_type: TechEffectType::StatModifierPercent {
                    stat: "damage".to_string(),
                    percent: 15,
                },
                applies_to: vec![],
                applies_to_tags: vec!["infantry".to_string()],
            }],
            prerequisites: vec!["basic_research".to_string()],
            tier: 1,
            exclusive_with: vec!["defensive_doctrine".to_string()],
            researched_at: Some("research_institute".to_string()),
            is_doctrine: false,
            branch: None,
            icon: None,
        }
    }

    #[test]
    fn test_requires() {
        let tech = create_test_tech();
        assert!(tech.requires("basic_research"));
        assert!(!tech.requires("unknown_tech"));
    }

    #[test]
    fn test_excludes() {
        let tech = create_test_tech();
        assert!(tech.excludes("defensive_doctrine"));
        assert!(!tech.excludes("other_doctrine"));
    }

    #[test]
    fn test_modifies_stat() {
        let tech = create_test_tech();
        assert!(tech.modifies_stat("damage"));
        assert!(!tech.modifies_stat("speed"));
    }

    #[test]
    fn test_get_stat_modifiers() {
        let tech = create_test_tech();
        let modifiers = tech.get_stat_modifiers();
        assert_eq!(modifiers.len(), 1);
        assert_eq!(modifiers[0], ("damage", 15, true));
    }
}
