//! Scenario loading and configuration.
//!
//! Scenarios define the initial game state for headless testing, including
//! faction setups, starting units/buildings, and victory conditions.

use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error type for scenario operations.
#[derive(Error, Debug)]
pub enum ScenarioError {
    /// File not found.
    #[error("Scenario file not found: {0}")]
    FileNotFound(String),
    /// Failed to read file.
    #[error("Failed to read scenario file: {0}")]
    ReadError(#[from] std::io::Error),
    /// Failed to parse RON.
    #[error("Failed to parse scenario: {0}")]
    ParseError(#[from] ron::error::SpannedError),
}

/// Map size presets for procedural generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MapSize {
    /// 512x512 world units.
    #[default]
    Small,
    /// 768x768 world units.
    Medium,
    /// 1024x1024 world units.
    Large,
}

/// A complete scenario configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    /// Scenario name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Map dimensions (width, height) in world units.
    pub map_size: (u32, u32),
    /// Faction setups for each player.
    pub factions: Vec<FactionSetup>,
    /// Victory conditions.
    pub victory_conditions: VictoryConditions,
    /// Initial resource setup.
    pub initial_resources: ResourceSetup,
}

impl Default for Scenario {
    fn default() -> Self {
        Self {
            name: "Default Skirmish".to_string(),
            description: "A basic 1v1 skirmish scenario".to_string(),
            map_size: (512, 512), // Match faction spawn positions
            factions: vec![
                FactionSetup::default_continuity(),
                FactionSetup::default_collegium(),
            ],
            victory_conditions: VictoryConditions::default(),
            initial_resources: ResourceSetup::default(),
        }
    }
}

impl Scenario {
    /// Load a scenario from a RON file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ScenarioError> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(ScenarioError::FileNotFound(path.display().to_string()));
        }
        let contents = std::fs::read_to_string(path)?;
        let scenario: Scenario = ron::from_str(&contents)?;
        Ok(scenario)
    }

    /// Load from a RON string (useful for embedded scenarios).
    pub fn from_ron_str(ron: &str) -> Result<Self, ScenarioError> {
        let scenario: Scenario = ron::from_str(ron)?;
        Ok(scenario)
    }

    /// Create a standard 1v1 skirmish scenario.
    #[must_use]
    pub fn skirmish_1v1() -> Self {
        Self {
            name: "Standard 1v1 Skirmish".to_string(),
            description: "Balanced starting positions for faction matchup testing".to_string(),
            map_size: (512, 512),
            factions: vec![
                FactionSetup {
                    faction_id: "continuity".to_string(),
                    ai_controller: AiController::Sandbox,
                    starting_units: vec![
                        UnitPlacement::new("scout", 64, 256, 2),
                        UnitPlacement::new("harvester", 80, 256, 1),
                    ],
                    starting_buildings: vec![BuildingPlacement::new("command_center", 48, 256)],
                    spawn_position: (48, 256),
                    starting_resources: 1000,
                },
                FactionSetup {
                    faction_id: "collegium".to_string(),
                    ai_controller: AiController::Sandbox,
                    starting_units: vec![
                        UnitPlacement::new("scout", 448, 256, 2),
                        UnitPlacement::new("harvester", 432, 256, 1),
                    ],
                    starting_buildings: vec![BuildingPlacement::new("command_center", 464, 256)],
                    spawn_position: (464, 256),
                    starting_resources: 1000,
                },
            ],
            victory_conditions: VictoryConditions {
                elimination: true,
                time_limit_ticks: Some(36000), // 10 minutes at 60 tps
                resource_threshold: None,
            },
            initial_resources: ResourceSetup {
                ore_nodes: vec![
                    OreNode::new(128, 200, 5000),
                    OreNode::new(128, 312, 5000),
                    OreNode::new(384, 200, 5000),
                    OreNode::new(384, 312, 5000),
                    OreNode::new(256, 256, 10000), // Contested center
                ],
            },
        }
    }

    /// Create a scenario from a procedurally generated map.
    ///
    /// Uses `rts_core::map_generation` to create terrain, resources, and spawn points.
    #[must_use]
    pub fn from_procedural_map(seed: u64, map_size: MapSize) -> Self {
        use rts_core::map_generation::{generate_map, MapConfig};

        let config = match map_size {
            MapSize::Small => MapConfig::small(),
            MapSize::Medium => MapConfig::medium(),
            MapSize::Large => MapConfig::large(),
        }
        .with_seed(seed);

        let generated = generate_map(config);
        let world_w = generated.world_width();
        let world_h = generated.world_height();

        // Convert spawn points to faction setups
        let mut factions = Vec::new();
        let faction_ids = ["continuity", "collegium"];

        for (i, spawn) in generated.spawn_points.iter().enumerate() {
            if i >= 2 {
                break;
            } // Max 2 factions for now

            let x: i32 = spawn.position.x.to_num();
            let y: i32 = spawn.position.y.to_num();

            factions.push(FactionSetup {
                faction_id: faction_ids[i].to_string(),
                ai_controller: AiController::Sandbox,
                starting_units: vec![
                    UnitPlacement::new("scout", x + 16, y, 2),
                    UnitPlacement::new("harvester", x + 32, y, 1),
                ],
                starting_buildings: vec![BuildingPlacement::new("command_center", x, y)],
                spawn_position: (x, y),
                starting_resources: 1000,
            });
        }

        // Convert resources
        let ore_nodes: Vec<OreNode> = generated
            .resources
            .iter()
            .map(|r| {
                let x: i32 = r.position.x.to_num();
                let y: i32 = r.position.y.to_num();
                OreNode::new(x, y, r.amount)
            })
            .collect();

        Self {
            name: format!("Procedural Map (seed: {})", seed),
            description: format!("{}x{} procedurally generated map", world_w, world_h),
            map_size: (world_w, world_h),
            factions,
            victory_conditions: VictoryConditions {
                elimination: true,
                time_limit_ticks: Some(36000),
                resource_threshold: None,
            },
            initial_resources: ResourceSetup { ore_nodes },
        }
    }

    /// Get the generated terrain data for this scenario (if using procedural map).
    ///
    /// Returns None for non-procedural scenarios.
    #[must_use]
    pub fn generate_terrain(&self, seed: u64) -> Option<rts_core::map_generation::GeneratedMap> {
        use rts_core::map_generation::{generate_map, MapConfig};

        let config = MapConfig {
            width: self.map_size.0 / 8,
            height: self.map_size.1 / 8,
            cell_size: 8,
            ..MapConfig::default()
        }
        .with_seed(seed);

        Some(generate_map(config))
    }
}

/// Setup for a single faction in the scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionSetup {
    /// Faction identifier ("continuity", "collegium", etc.).
    pub faction_id: String,
    /// How this faction is controlled.
    pub ai_controller: AiController,
    /// Starting units.
    pub starting_units: Vec<UnitPlacement>,
    /// Starting buildings.
    pub starting_buildings: Vec<BuildingPlacement>,
    /// Spawn position (x, y).
    pub spawn_position: (i32, i32),
    /// Starting resources.
    pub starting_resources: i64,
}

impl FactionSetup {
    /// Create default Continuity faction setup.
    #[must_use]
    pub fn default_continuity() -> Self {
        Self {
            faction_id: "continuity".to_string(),
            ai_controller: AiController::Sandbox,
            starting_units: vec![
                UnitPlacement::new("scout", 64, 256, 2),
                UnitPlacement::new("harvester", 80, 256, 1),
            ],
            starting_buildings: vec![BuildingPlacement::new("command_center", 48, 256)],
            spawn_position: (48, 256),
            starting_resources: 1000,
        }
    }

    /// Create default Collegium faction setup.
    #[must_use]
    pub fn default_collegium() -> Self {
        Self {
            faction_id: "collegium".to_string(),
            ai_controller: AiController::Sandbox,
            starting_units: vec![
                UnitPlacement::new("scout", 448, 256, 2),
                UnitPlacement::new("harvester", 432, 256, 1),
            ],
            starting_buildings: vec![BuildingPlacement::new("command_center", 464, 256)],
            spawn_position: (464, 256),
            starting_resources: 1000,
        }
    }
}

/// How a faction's units are controlled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiController {
    /// Full autonomous AI (sandbox mode).
    Sandbox,
    /// Follow a scripted strategy file.
    Scripted(String),
    /// Controlled via JSON protocol.
    External,
    /// No control (for spectator/testing).
    None,
}

/// Placement of a unit at scenario start.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitPlacement {
    /// Unit type identifier.
    pub kind: String,
    /// Position (x, y).
    pub position: (i32, i32),
    /// Number of units to spawn.
    pub count: u32,
}

impl UnitPlacement {
    /// Create a new unit placement.
    #[must_use]
    pub fn new(kind: impl Into<String>, x: i32, y: i32, count: u32) -> Self {
        Self {
            kind: kind.into(),
            position: (x, y),
            count,
        }
    }
}

/// Placement of a building at scenario start.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingPlacement {
    /// Building type identifier.
    pub kind: String,
    /// Position (x, y).
    pub position: (i32, i32),
}

impl BuildingPlacement {
    /// Create a new building placement.
    #[must_use]
    pub fn new(kind: impl Into<String>, x: i32, y: i32) -> Self {
        Self {
            kind: kind.into(),
            position: (x, y),
        }
    }
}

/// Victory conditions for the scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VictoryConditions {
    /// Victory by eliminating all enemy structures.
    pub elimination: bool,
    /// Optional time limit in ticks.
    pub time_limit_ticks: Option<u64>,
    /// Optional resource threshold for economic victory.
    pub resource_threshold: Option<i64>,
}

impl Default for VictoryConditions {
    fn default() -> Self {
        Self {
            elimination: true,
            time_limit_ticks: None,
            resource_threshold: None,
        }
    }
}

/// Resource setup for the map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSetup {
    /// Ore/resource node placements.
    pub ore_nodes: Vec<OreNode>,
}

impl Default for ResourceSetup {
    fn default() -> Self {
        Self {
            ore_nodes: vec![OreNode::new(128, 128, 5000), OreNode::new(384, 384, 5000)],
        }
    }
}

/// An ore/resource node on the map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OreNode {
    /// Position (x, y).
    pub position: (i32, i32),
    /// Total resources available.
    pub amount: i64,
}

impl OreNode {
    /// Create a new ore node.
    #[must_use]
    pub fn new(x: i32, y: i32, amount: i64) -> Self {
        Self {
            position: (x, y),
            amount,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_scenario() {
        let scenario = Scenario::default();
        assert_eq!(scenario.factions.len(), 2);
        assert_eq!(scenario.factions[0].faction_id, "continuity");
        assert_eq!(scenario.factions[1].faction_id, "collegium");
    }

    #[test]
    fn test_skirmish_scenario() {
        let scenario = Scenario::skirmish_1v1();
        assert_eq!(scenario.map_size, (512, 512));
        assert!(scenario.victory_conditions.elimination);
        assert_eq!(scenario.initial_resources.ore_nodes.len(), 5);
    }

    #[test]
    fn test_parse_from_ron() {
        let ron = r#"
            Scenario(
                name: "Test",
                description: "Test scenario",
                map_size: (100, 100),
                factions: [],
                victory_conditions: VictoryConditions(
                    elimination: true,
                    time_limit_ticks: None,
                    resource_threshold: None,
                ),
                initial_resources: ResourceSetup(
                    ore_nodes: [],
                ),
            )
        "#;
        let scenario = Scenario::from_ron_str(ron).unwrap();
        assert_eq!(scenario.name, "Test");
    }
}
