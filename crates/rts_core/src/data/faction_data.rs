//! Faction data structure combining all faction-specific definitions.

use serde::{Deserialize, Serialize};

use super::building_data::BuildingData;
use super::tech_data::TechData;
use super::unit_data::UnitData;
use crate::factions::FactionId;

/// Complete faction data definition.
///
/// Contains all units, buildings, and technologies for a single faction.
/// Loaded from a RON file at game startup.
///
/// # Example RON
///
/// ```ron
/// FactionData(
///     id: Continuity,
///     display_name: "faction.continuity.name",
///     description: "faction.continuity.desc",
///     units: [...],
///     buildings: [...],
///     technologies: [...],
/// )
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionData {
    /// Faction identifier.
    pub id: FactionId,

    /// Localization key for faction display name.
    pub display_name: String,

    /// Localization key for faction description.
    pub description: String,

    /// All unit types available to this faction.
    pub units: Vec<UnitData>,

    /// All building types available to this faction.
    pub buildings: Vec<BuildingData>,

    /// All technologies available to this faction.
    pub technologies: Vec<TechData>,

    /// Primary color RGB values (0-255).
    #[serde(default = "default_primary_color")]
    pub primary_color: [u8; 3],

    /// Secondary color RGB values (0-255).
    #[serde(default = "default_secondary_color")]
    pub secondary_color: [u8; 3],

    /// Starting units when a match begins.
    #[serde(default)]
    pub starting_units: Vec<StartingEntity>,

    /// Starting buildings when a match begins.
    #[serde(default)]
    pub starting_buildings: Vec<StartingEntity>,

    /// Starting feedstock amount.
    #[serde(default = "default_starting_feedstock")]
    pub starting_feedstock: i32,
}

/// Default primary color (blue-ish).
const fn default_primary_color() -> [u8; 3] {
    [100, 100, 200]
}

/// Default secondary color (white).
const fn default_secondary_color() -> [u8; 3] {
    [255, 255, 255]
}

/// Default starting feedstock.
const fn default_starting_feedstock() -> i32 {
    500
}

/// Definition for a starting unit or building.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartingEntity {
    /// ID of the unit or building type.
    pub type_id: String,

    /// Relative spawn offset from player start position.
    #[serde(default)]
    pub offset_x: i32,

    /// Relative spawn offset from player start position.
    #[serde(default)]
    pub offset_y: i32,
}

impl FactionData {
    /// Find a unit by its ID.
    #[must_use]
    pub fn get_unit(&self, id: &str) -> Option<&UnitData> {
        self.units.iter().find(|u| u.id == id)
    }

    /// Find a building by its ID.
    #[must_use]
    pub fn get_building(&self, id: &str) -> Option<&BuildingData> {
        self.buildings.iter().find(|b| b.id == id)
    }

    /// Find a technology by its ID.
    #[must_use]
    pub fn get_technology(&self, id: &str) -> Option<&TechData> {
        self.technologies.iter().find(|t| t.id == id)
    }

    /// Get all units at a specific tier.
    pub fn units_at_tier(&self, tier: u8) -> impl Iterator<Item = &UnitData> {
        self.units.iter().filter(move |u| u.tier == tier)
    }

    /// Get all buildings at a specific tier.
    pub fn buildings_at_tier(&self, tier: u8) -> impl Iterator<Item = &BuildingData> {
        self.buildings.iter().filter(move |b| b.tier == tier)
    }

    /// Get all technologies at a specific tier.
    pub fn technologies_at_tier(&self, tier: u8) -> impl Iterator<Item = &TechData> {
        self.technologies.iter().filter(move |t| t.tier == tier)
    }

    /// Get all doctrine (branch) technologies.
    pub fn doctrines(&self) -> impl Iterator<Item = &TechData> {
        self.technologies.iter().filter(|t| t.is_doctrine)
    }

    /// Validate internal consistency of faction data.
    ///
    /// Checks for:
    /// - Unit references in buildings are valid
    /// - Tech prerequisites exist
    /// - Building references in units are valid
    ///
    /// Returns a list of validation errors.
    #[must_use]
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // Check building production references
        for building in &self.buildings {
            for unit_id in &building.produces {
                if self.get_unit(unit_id).is_none() {
                    errors.push(format!(
                        "Building '{}' produces unknown unit '{}'",
                        building.id, unit_id
                    ));
                }
            }

            for tech_id in &building.tech_required {
                // Accept both technologies and buildings as prerequisites
                if self.get_technology(tech_id).is_none() && self.get_building(tech_id).is_none() {
                    errors.push(format!(
                        "Building '{}' requires unknown tech or building '{}'",
                        building.id, tech_id
                    ));
                }
            }
        }

        // Check unit production location references
        for unit in &self.units {
            for building_id in &unit.produced_at {
                if self.get_building(building_id).is_none() {
                    errors.push(format!(
                        "Unit '{}' produced at unknown building '{}'",
                        unit.id, building_id
                    ));
                }
            }

            for tech_id in &unit.tech_required {
                // Accept both technologies and buildings as prerequisites
                if self.get_technology(tech_id).is_none() && self.get_building(tech_id).is_none() {
                    errors.push(format!(
                        "Unit '{}' requires unknown tech or building '{}'",
                        unit.id, tech_id
                    ));
                }
            }
        }

        // Check tech prerequisites
        for tech in &self.technologies {
            for prereq_id in &tech.prerequisites {
                // Accept both technologies and buildings as prerequisites
                if self.get_technology(prereq_id).is_none()
                    && self.get_building(prereq_id).is_none()
                {
                    errors.push(format!(
                        "Tech '{}' has unknown prerequisite '{}'",
                        tech.id, prereq_id
                    ));
                }
            }

            for exclusive_id in &tech.exclusive_with {
                if self.get_technology(exclusive_id).is_none() {
                    errors.push(format!(
                        "Tech '{}' is exclusive with unknown tech '{}'",
                        tech.id, exclusive_id
                    ));
                }
            }
        }

        // Check starting entities
        for entity in &self.starting_units {
            if self.get_unit(&entity.type_id).is_none() {
                errors.push(format!("Starting unit '{}' not found", entity.type_id));
            }
        }

        for entity in &self.starting_buildings {
            if self.get_building(&entity.type_id).is_none() {
                errors.push(format!("Starting building '{}' not found", entity.type_id));
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_faction_data() -> FactionData {
        FactionData {
            id: FactionId::Continuity,
            display_name: "faction.continuity.name".to_string(),
            description: "faction.continuity.desc".to_string(),
            units: vec![UnitData {
                id: "security_team".to_string(),
                name: "unit.security_team.name".to_string(),
                description: "unit.security_team.desc".to_string(),
                cost: 50,
                build_time: 120,
                health: 80,
                speed: crate::math::Fixed::from_num(10),
                combat: None,
                tech_required: vec![],
                tier: 1,
                produced_at: vec!["training_center".to_string()],
                tags: vec!["infantry".to_string()],
            }],
            buildings: vec![BuildingData {
                id: "training_center".to_string(),
                name: "building.training_center.name".to_string(),
                description: "building.training_center.desc".to_string(),
                cost: 150,
                build_time: 180,
                health: 500,
                produces: vec!["security_team".to_string()],
                tech_required: vec![],
                provides_tech: vec![],
                tier: 1,
                targetable: true,
                armor: 10,
                vision_range: None,
                tags: vec!["production".to_string()],
                is_harvester: false,
                is_main_base: false,
            }],
            technologies: vec![],
            primary_color: [0, 50, 150],
            secondary_color: [255, 255, 255],
            starting_units: vec![],
            starting_buildings: vec![StartingEntity {
                type_id: "training_center".to_string(),
                offset_x: 0,
                offset_y: 0,
            }],
            starting_feedstock: 500,
        }
    }

    #[test]
    fn test_get_unit() {
        let faction = create_test_faction_data();
        assert!(faction.get_unit("security_team").is_some());
        assert!(faction.get_unit("unknown").is_none());
    }

    #[test]
    fn test_get_building() {
        let faction = create_test_faction_data();
        assert!(faction.get_building("training_center").is_some());
        assert!(faction.get_building("unknown").is_none());
    }

    #[test]
    fn test_validate_valid_data() {
        let faction = create_test_faction_data();
        let errors = faction.validate();
        assert!(errors.is_empty(), "Errors: {:?}", errors);
    }

    #[test]
    fn test_validate_invalid_reference() {
        let mut faction = create_test_faction_data();
        faction.buildings[0]
            .produces
            .push("unknown_unit".to_string());

        let errors = faction.validate();
        assert!(!errors.is_empty());
        assert!(errors[0].contains("unknown unit"));
    }
}
