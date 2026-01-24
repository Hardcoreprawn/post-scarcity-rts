//! Building data structures for data-driven building definitions.

use serde::{Deserialize, Serialize};

use crate::math::Fixed;

/// Data-driven building definition.
///
/// Defines all properties of a building type that can be loaded from
/// configuration files. Used to populate `BuildingBlueprint` instances.
///
/// # Example RON
///
/// ```ron
/// BuildingData(
///     id: "training_center",
///     name: "building.continuity.training_center.name",
///     description: "building.continuity.training_center.desc",
///     cost: 150,
///     build_time: 180,
///     health: 500,
///     produces: ["security_team", "crowd_management_unit", "patrol_vehicle"],
///     tech_required: [],
///     provides_tech: [],
///     tier: 1,
/// )
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingData {
    /// Unique string identifier for this building type.
    pub id: String,

    /// Localization key for the building's display name.
    pub name: String,

    /// Localization key for the building's description.
    pub description: String,

    /// Feedstock cost to construct this building.
    pub cost: i32,

    /// Construction time in simulation ticks.
    pub build_time: u32,

    /// Maximum health points.
    pub health: i32,

    /// Unit IDs this building can produce.
    #[serde(default)]
    pub produces: Vec<String>,

    /// Technology IDs required to construct this building.
    #[serde(default)]
    pub tech_required: Vec<String>,

    /// Technologies unlocked by constructing this building.
    #[serde(default)]
    pub provides_tech: Vec<String>,

    /// Tech tier this building belongs to (1, 2, or 3).
    #[serde(default = "default_tier")]
    pub tier: u8,

    /// Whether this building can be attacked.
    #[serde(default = "default_true")]
    pub targetable: bool,

    /// Armor value that reduces incoming damage.
    #[serde(default)]
    pub armor: i32,

    /// Vision radius in game units.
    #[serde(default, with = "option_fixed_serde")]
    pub vision_range: Option<Fixed>,

    /// Tags for categorization (e.g., "production", "defense", "economy").
    #[serde(default)]
    pub tags: Vec<String>,

    /// Whether this building harvests resources.
    #[serde(default)]
    pub is_harvester: bool,

    /// Whether this building is the faction's main base.
    #[serde(default)]
    pub is_main_base: bool,
}

/// Default tier for buildings without explicit tier.
const fn default_tier() -> u8 {
    1
}

/// Default to true for targetable.
const fn default_true() -> bool {
    true
}

/// Serde support for optional fixed-point numbers.
mod option_fixed_serde {
    use crate::math::Fixed;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Serialize an optional fixed-point number.
    pub fn serialize<S>(value: &Option<Fixed>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(v) => v.to_bits().serialize(serializer),
            None => serializer.serialize_none(),
        }
    }

    /// Deserialize an optional fixed-point number.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Fixed>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<i64>::deserialize(deserializer)?;
        Ok(opt.map(Fixed::from_bits))
    }
}

impl BuildingData {
    /// Check if this building requires a specific technology.
    #[must_use]
    pub fn requires_tech(&self, tech_id: &str) -> bool {
        self.tech_required.iter().any(|t| t == tech_id)
    }

    /// Check if this building can produce the specified unit.
    #[must_use]
    pub fn can_produce(&self, unit_id: &str) -> bool {
        self.produces.iter().any(|u| u == unit_id)
    }

    /// Check if this building unlocks a specific technology.
    #[must_use]
    pub fn unlocks_tech(&self, tech_id: &str) -> bool {
        self.provides_tech.iter().any(|t| t == tech_id)
    }

    /// Check if this building has the specified tag.
    #[must_use]
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_building() -> BuildingData {
        BuildingData {
            id: "training_center".to_string(),
            name: "building.test.name".to_string(),
            description: "building.test.desc".to_string(),
            cost: 150,
            build_time: 180,
            health: 500,
            produces: vec!["security_team".to_string(), "patrol_vehicle".to_string()],
            tech_required: vec![],
            provides_tech: vec!["basic_training".to_string()],
            tier: 1,
            targetable: true,
            armor: 10,
            vision_range: Some(Fixed::from_num(15)),
            tags: vec!["production".to_string()],
            is_harvester: false,
            is_main_base: false,
        }
    }

    #[test]
    fn test_can_produce() {
        let building = create_test_building();
        assert!(building.can_produce("security_team"));
        assert!(building.can_produce("patrol_vehicle"));
        assert!(!building.can_produce("tank"));
    }

    #[test]
    fn test_unlocks_tech() {
        let building = create_test_building();
        assert!(building.unlocks_tech("basic_training"));
        assert!(!building.unlocks_tech("advanced_training"));
    }

    #[test]
    fn test_has_tag() {
        let building = create_test_building();
        assert!(building.has_tag("production"));
        assert!(!building.has_tag("defense"));
    }
}
