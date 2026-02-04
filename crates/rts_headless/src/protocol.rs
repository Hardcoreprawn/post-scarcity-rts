//! JSON protocol for headless game communication.
//!
//! The headless runner communicates via JSON lines (one JSON object per line):
//!
//! **Input (stdin):** Commands from the AI controller
//! **Output (stdout):** Game state updates and responses
//!
//! # Protocol Flow
//!
//! 1. Runner starts, outputs `{"type":"ready","version":"1.0"}`
//! 2. AI sends commands as JSON lines
//! 3. Runner outputs state after each tick (or on `query` command)
//! 4. On game end, outputs `{"type":"game_over","result":"victory"|"defeat"}`
//!
//! # Example Session
//!
//! ```text
//! <- {"type":"ready","version":"1.0","tick":0}
//! -> {"cmd":"spawn","unit_type":"harvester","x":100,"y":100}
//! <- {"type":"spawned","entity_id":5,"unit_type":"harvester"}
//! -> {"cmd":"tick","count":60}
//! <- {"type":"state","tick":60,"entities":[...],"resources":{"feedstock":1000}}
//! -> {"cmd":"move","entity_id":5,"target_x":200,"target_y":200}
//! <- {"type":"ack","cmd":"move"}
//! -> {"cmd":"query"}
//! <- {"type":"state","tick":60,...}
//! ```

use serde::{Deserialize, Serialize};

// ============================================================================
// Input Commands (AI -> Runner)
// ============================================================================

/// Commands that can be sent to the headless runner.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum Command {
    /// Advance simulation by N ticks (default: 1).
    Tick {
        #[serde(default = "default_tick_count")]
        count: u32,
    },

    /// Query current game state without advancing time.
    Query,

    /// Spawn a unit at position.
    Spawn {
        unit_type: String,
        x: f64,
        y: f64,
        #[serde(default)]
        faction: Option<u8>,
    },

    /// Spawn a building at position.
    SpawnBuilding {
        building_type: String,
        x: f64,
        y: f64,
        #[serde(default)]
        faction: Option<u8>,
    },

    /// Issue move command to entity.
    Move {
        entity_id: u32,
        target_x: f64,
        target_y: f64,
    },

    /// Issue attack command to entity.
    Attack { entity_id: u32, target_id: u32 },

    /// Issue stop command to entity.
    Stop { entity_id: u32 },

    /// Set player resources.
    SetResources { amount: u32 },

    /// Teleport entity to position.
    Teleport { entity_id: u32, x: f64, y: f64 },

    /// Kill an entity.
    Kill { entity_id: u32 },

    /// Set game speed multiplier.
    Speed { multiplier: f64 },

    /// Force victory condition.
    Win,

    /// Force defeat condition.
    Lose,

    /// Quit the game.
    Quit,

    /// Save current state hash (for determinism verification).
    Hash,

    /// Load a scenario file.
    LoadScenario { path: String },

    /// Save a screenshot (only works in graphical mode).
    Screenshot { path: String },
}

fn default_tick_count() -> u32 {
    1
}

// ============================================================================
// Output Responses (Runner -> AI)
// ============================================================================

/// Responses sent from the headless runner.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    /// Runner is ready to accept commands.
    Ready { version: String, tick: u64 },

    /// Acknowledgment of a command.
    Ack { cmd: String },

    /// Error processing a command.
    Error {
        message: String,
        cmd: Option<String>,
    },

    /// Current game state.
    State {
        tick: u64,
        entities: Vec<EntityState>,
        resources: ResourceState,
        game_status: GameStatus,
        hash: u64,
    },

    /// Entity was spawned.
    Spawned { entity_id: u32, unit_type: String },

    /// Game has ended.
    GameOver {
        result: GameResult,
        ticks: u64,
        stats: MatchStatsOutput,
    },

    /// State hash for determinism verification.
    StateHash { tick: u64, hash: u64 },

    /// Goodbye message before shutdown.
    Bye,
}

// ============================================================================
// State Types
// ============================================================================

/// State of a single entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityState {
    pub id: u32,
    pub entity_type: EntityType,
    pub x: f64,
    pub y: f64,
    pub faction: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<HealthState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cargo: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

/// Type of entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Unit { kind: String },
    Building { kind: String },
    ResourceNode,
    Projectile,
}

/// Health state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthState {
    pub current: u32,
    pub max: u32,
}

/// Resource state for a player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceState {
    pub feedstock: u32,
}

/// Current game status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameStatus {
    InProgress,
    Victory,
    Defeat,
}

/// Game result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameResult {
    Victory,
    Defeat,
    Draw,
}

/// Match statistics output.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MatchStatsOutput {
    pub duration_ticks: u64,
    pub units_produced: u32,
    pub units_lost: u32,
    pub enemies_killed: u32,
    pub resources_gathered: u32,
}

// ============================================================================
// Helpers
// ============================================================================

impl Response {
    /// Create a ready response.
    pub fn ready(tick: u64) -> Self {
        Self::Ready {
            version: "1.0".to_string(),
            tick,
        }
    }

    /// Create an acknowledgment.
    pub fn ack(cmd: &str) -> Self {
        Self::Ack {
            cmd: cmd.to_string(),
        }
    }

    /// Create an error response.
    pub fn error(message: impl Into<String>, cmd: Option<&str>) -> Self {
        Self::Error {
            message: message.into(),
            cmd: cmd.map(String::from),
        }
    }

    /// Serialize to JSON line (with newline).
    pub fn to_json_line(&self) -> String {
        let mut json = serde_json::to_string(self).unwrap_or_else(|e| {
            format!(
                r#"{{"type":"error","message":"Serialization failed: {}"}}"#,
                e
            )
        });
        json.push('\n');
        json
    }
}

impl Command {
    /// Parse from a JSON line.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Get command name for acknowledgment.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Tick { .. } => "tick",
            Self::Query => "query",
            Self::Spawn { .. } => "spawn",
            Self::SpawnBuilding { .. } => "spawn_building",
            Self::Move { .. } => "move",
            Self::Attack { .. } => "attack",
            Self::Stop { .. } => "stop",
            Self::SetResources { .. } => "set_resources",
            Self::Teleport { .. } => "teleport",
            Self::Kill { .. } => "kill",
            Self::Speed { .. } => "speed",
            Self::Win => "win",
            Self::Lose => "lose",
            Self::Quit => "quit",
            Self::Hash => "hash",
            Self::LoadScenario { .. } => "load_scenario",
            Self::Screenshot { .. } => "screenshot",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tick_command() {
        let json = r#"{"cmd":"tick","count":60}"#;
        let cmd = Command::from_json(json).unwrap();
        assert!(matches!(cmd, Command::Tick { count: 60 }));
    }

    #[test]
    fn test_parse_spawn_command() {
        let json = r#"{"cmd":"spawn","unit_type":"harvester","x":100.0,"y":200.0}"#;
        let cmd = Command::from_json(json).unwrap();
        assert!(matches!(
            cmd,
            Command::Spawn {
                unit_type,
                x,
                y,
                ..
            } if unit_type == "harvester" && x == 100.0 && y == 200.0
        ));
    }

    #[test]
    fn test_serialize_state_response() {
        let resp = Response::State {
            tick: 100,
            entities: vec![],
            resources: ResourceState { feedstock: 500 },
            game_status: GameStatus::InProgress,
            hash: 12345,
        };
        let json = resp.to_json_line();
        assert!(json.contains(r#""type":"state""#));
        assert!(json.contains(r#""tick":100"#));
    }

    #[test]
    fn test_default_tick_count() {
        let json = r#"{"cmd":"tick"}"#;
        let cmd = Command::from_json(json).unwrap();
        assert!(matches!(cmd, Command::Tick { count: 1 }));
    }
}
