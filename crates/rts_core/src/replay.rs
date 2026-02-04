//! Replay system for recording and playing back games.
//!
//! Replays store the initial scenario state and the stream of commands
//! issued during the game. This allows deterministic recreation of any game.

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::components::{Command, EntityId};
use crate::error::{GameError, Result};
use crate::simulation::Simulation;

/// A single command record for replay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayCommand {
    /// Simulation tick when the command was issued.
    pub tick: u64,
    /// Target entity for the command.
    pub entity: EntityId,
    /// The command that was issued.
    pub command: Command,
}

impl ReplayCommand {
    /// Create a new replay command record.
    #[must_use]
    pub const fn new(tick: u64, entity: EntityId, command: Command) -> Self {
        Self {
            tick,
            entity,
            command,
        }
    }
}

/// Replay file format version for compatibility.
pub const REPLAY_VERSION: u32 = 1;

/// Complete replay data structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Replay {
    /// Replay format version.
    pub version: u32,
    /// Scenario identifier or name.
    pub scenario_id: String,
    /// Random seed used for the game.
    pub seed: u64,
    /// Serialized initial simulation state.
    pub initial_state: Vec<u8>,
    /// Stream of commands in tick order.
    pub commands: Vec<ReplayCommand>,
    /// Final tick when the game ended.
    pub final_tick: u64,
    /// Final state hash for verification.
    pub final_hash: u64,
}

impl Replay {
    /// Create a new replay from a simulation's initial state.
    pub fn new(
        scenario_id: impl Into<String>,
        seed: u64,
        initial_state: &Simulation,
    ) -> Result<Self> {
        let state_bytes = initial_state.serialize()?;
        Ok(Self {
            version: REPLAY_VERSION,
            scenario_id: scenario_id.into(),
            seed,
            initial_state: state_bytes,
            commands: Vec::new(),
            final_tick: 0,
            final_hash: 0,
        })
    }

    /// Record a command for replay.
    pub fn record_command(&mut self, tick: u64, entity: EntityId, command: Command) {
        self.commands
            .push(ReplayCommand::new(tick, entity, command));
    }

    /// Finalize the replay with end-game state.
    pub fn finalize(&mut self, final_tick: u64, final_hash: u64) {
        self.final_tick = final_tick;
        self.final_hash = final_hash;
    }

    /// Save the replay to a file.
    ///
    /// # Errors
    /// Returns an error if serialization or file writing fails.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let bytes = bincode::serialize(self)
            .map_err(|e| GameError::InvalidState(format!("Failed to serialize replay: {}", e)))?;
        std::fs::write(path.as_ref(), bytes)
            .map_err(|e| GameError::InvalidState(format!("Failed to write replay file: {}", e)))?;
        Ok(())
    }

    /// Load a replay from a file.
    ///
    /// # Errors
    /// Returns an error if file reading or deserialization fails.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let bytes = std::fs::read(path.as_ref())
            .map_err(|e| GameError::InvalidState(format!("Failed to read replay file: {}", e)))?;
        let replay: Self = bincode::deserialize(&bytes)
            .map_err(|e| GameError::InvalidState(format!("Failed to deserialize replay: {}", e)))?;

        // Version check
        if replay.version != REPLAY_VERSION {
            return Err(GameError::InvalidState(format!(
                "Replay version mismatch: expected {}, got {}",
                REPLAY_VERSION, replay.version
            )));
        }

        Ok(replay)
    }

    /// Get the initial simulation state for playback.
    ///
    /// # Errors
    /// Returns an error if state deserialization fails.
    pub fn restore_initial_state(&self) -> Result<Simulation> {
        Simulation::deserialize(&self.initial_state)
    }

    /// Get commands for a specific tick.
    #[must_use]
    pub fn commands_at_tick(&self, tick: u64) -> Vec<&ReplayCommand> {
        self.commands
            .iter()
            .filter(|cmd| cmd.tick == tick)
            .collect()
    }

    /// Get the total duration of the replay in ticks.
    #[must_use]
    pub const fn duration(&self) -> u64 {
        self.final_tick
    }

    /// Get the total number of commands in the replay.
    #[must_use]
    pub fn command_count(&self) -> usize {
        self.commands.len()
    }
}

/// Replay playback controller.
#[derive(Debug)]
pub struct ReplayPlayer {
    /// The replay being played.
    replay: Replay,
    /// Current simulation state.
    simulation: Simulation,
    /// Current playback tick.
    current_tick: u64,
    /// Index into the command stream.
    command_index: usize,
    /// Playback speed multiplier (1.0 = normal, 2.0 = 2x, 0.5 = half).
    pub playback_speed: f64,
    /// Whether playback is paused.
    pub paused: bool,
}

impl ReplayPlayer {
    /// Create a new replay player from a replay.
    ///
    /// # Errors
    /// Returns an error if the initial state cannot be restored.
    pub fn new(replay: Replay) -> Result<Self> {
        let simulation = replay.restore_initial_state()?;
        Ok(Self {
            replay,
            simulation,
            current_tick: 0,
            command_index: 0,
            playback_speed: 1.0,
            paused: false,
        })
    }

    /// Advance the replay by one tick.
    ///
    /// Returns true if there are more ticks to play.
    pub fn advance(&mut self) -> bool {
        if self.paused || self.current_tick >= self.replay.final_tick {
            return self.current_tick < self.replay.final_tick;
        }

        // Apply all commands for the current tick
        while self.command_index < self.replay.commands.len() {
            let cmd = &self.replay.commands[self.command_index];
            if cmd.tick > self.current_tick {
                break;
            }
            // Apply the command to the simulation
            let _ = self
                .simulation
                .apply_command(cmd.entity, cmd.command.clone());
            self.command_index += 1;
        }

        // Tick the simulation
        self.simulation.tick();
        self.current_tick += 1;

        self.current_tick < self.replay.final_tick
    }

    /// Seek to a specific tick.
    ///
    /// # Errors
    /// Returns an error if state restoration fails.
    pub fn seek(&mut self, target_tick: u64) -> Result<()> {
        // Reset to initial state
        self.simulation = self.replay.restore_initial_state()?;
        self.current_tick = 0;
        self.command_index = 0;

        // Advance to target tick
        while self.current_tick < target_tick && self.current_tick < self.replay.final_tick {
            // Apply commands for current tick
            while self.command_index < self.replay.commands.len() {
                let cmd = &self.replay.commands[self.command_index];
                if cmd.tick > self.current_tick {
                    break;
                }
                let _ = self
                    .simulation
                    .apply_command(cmd.entity, cmd.command.clone());
                self.command_index += 1;
            }
            self.simulation.tick();
            self.current_tick += 1;
        }

        Ok(())
    }

    /// Get the current tick.
    #[must_use]
    pub const fn current_tick(&self) -> u64 {
        self.current_tick
    }

    /// Get a reference to the current simulation state.
    #[must_use]
    pub const fn simulation(&self) -> &Simulation {
        &self.simulation
    }

    /// Get the replay being played.
    #[must_use]
    pub const fn replay(&self) -> &Replay {
        &self.replay
    }

    /// Check if the replay has finished.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.current_tick >= self.replay.final_tick
    }

    /// Verify the replay produces the expected final hash.
    ///
    /// # Errors
    /// Returns an error if state restoration fails or hash mismatch.
    pub fn verify(&mut self) -> Result<bool> {
        self.seek(self.replay.final_tick)?;
        let actual_hash = self.simulation.state_hash();
        Ok(actual_hash == self.replay.final_hash)
    }

    /// Toggle pause state.
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Set playback speed.
    pub fn set_speed(&mut self, speed: f64) {
        self.playback_speed = speed.clamp(0.1, 10.0);
    }

    /// Get progress as a percentage (0-100).
    #[must_use]
    pub fn progress_percent(&self) -> f64 {
        if self.replay.final_tick == 0 {
            100.0
        } else {
            (self.current_tick as f64 / self.replay.final_tick as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Command;
    use crate::math::Vec2Fixed;
    use crate::simulation::EntitySpawnParams;

    fn create_test_simulation() -> Simulation {
        let mut sim = Simulation::new();
        // Spawn a test unit
        sim.spawn_entity(EntitySpawnParams {
            position: Some(Vec2Fixed::new(
                crate::math::Fixed::from_num(100),
                crate::math::Fixed::from_num(100),
            )),
            health: Some(100),
            movement: Some(crate::math::Fixed::from_num(10)),
            combat_stats: Some(crate::components::CombatStats::default()),
            faction: Some(crate::components::FactionMember::new(
                crate::factions::FactionId::Continuity,
                0,
            )),
            ..Default::default()
        });
        sim
    }

    #[test]
    fn test_replay_create() {
        let sim = create_test_simulation();
        let replay = Replay::new("test_scenario", 12345, &sim);
        assert!(replay.is_ok());

        let replay = replay.unwrap();
        assert_eq!(replay.version, REPLAY_VERSION);
        assert_eq!(replay.scenario_id, "test_scenario");
        assert_eq!(replay.seed, 12345);
        assert!(replay.commands.is_empty());
    }

    #[test]
    fn test_replay_record_commands() {
        let sim = create_test_simulation();
        let mut replay = Replay::new("test_scenario", 12345, &sim).unwrap();

        replay.record_command(0, 1, Command::Stop);
        replay.record_command(
            5,
            1,
            Command::MoveTo(Vec2Fixed::new(
                crate::math::Fixed::from_num(200),
                crate::math::Fixed::from_num(200),
            )),
        );
        replay.record_command(10, 2, Command::HoldPosition);

        assert_eq!(replay.command_count(), 3);
        assert_eq!(replay.commands_at_tick(0).len(), 1);
        assert_eq!(replay.commands_at_tick(5).len(), 1);
        assert_eq!(replay.commands_at_tick(10).len(), 1);
        assert_eq!(replay.commands_at_tick(7).len(), 0);
    }

    #[test]
    fn test_replay_finalize() {
        let sim = create_test_simulation();
        let mut replay = Replay::new("test_scenario", 12345, &sim).unwrap();

        replay.finalize(1000, 0xDEADBEEF);

        assert_eq!(replay.duration(), 1000);
        assert_eq!(replay.final_hash, 0xDEADBEEF);
    }

    #[test]
    fn test_replay_save_load() {
        let sim = create_test_simulation();
        let mut replay = Replay::new("test_scenario", 12345, &sim).unwrap();

        replay.record_command(0, 1, Command::Stop);
        replay.finalize(100, 0x12345678);

        // Save to temp file
        let temp_path = std::env::temp_dir().join("test_replay.bin");
        assert!(replay.save(&temp_path).is_ok());

        // Load and verify
        let loaded = Replay::load(&temp_path);
        assert!(loaded.is_ok());

        let loaded = loaded.unwrap();
        assert_eq!(loaded.scenario_id, "test_scenario");
        assert_eq!(loaded.seed, 12345);
        assert_eq!(loaded.command_count(), 1);
        assert_eq!(loaded.duration(), 100);
        assert_eq!(loaded.final_hash, 0x12345678);

        // Cleanup
        let _ = std::fs::remove_file(temp_path);
    }

    #[test]
    fn test_replay_restore_state() {
        let sim = create_test_simulation();
        let replay = Replay::new("test_scenario", 12345, &sim).unwrap();

        let restored = replay.restore_initial_state();
        assert!(restored.is_ok());

        let restored = restored.unwrap();
        assert_eq!(restored.entities().len(), sim.entities().len());
    }

    #[test]
    fn test_replay_player_creation() {
        let sim = create_test_simulation();
        let mut replay = Replay::new("test_scenario", 12345, &sim).unwrap();
        replay.finalize(100, 0);

        let player = ReplayPlayer::new(replay);
        assert!(player.is_ok());

        let player = player.unwrap();
        assert_eq!(player.current_tick(), 0);
        assert!(!player.is_finished());
    }

    #[test]
    fn test_replay_player_advance() {
        let sim = create_test_simulation();
        let mut replay = Replay::new("test_scenario", 12345, &sim).unwrap();
        replay.finalize(10, 0);

        let mut player = ReplayPlayer::new(replay).unwrap();

        // Advance 5 ticks
        for _ in 0..5 {
            assert!(player.advance());
        }
        assert_eq!(player.current_tick(), 5);
        assert!(!player.is_finished());

        // Advance to end
        while player.advance() {}
        assert!(player.is_finished());
    }

    #[test]
    fn test_replay_player_seek() {
        let sim = create_test_simulation();
        let mut replay = Replay::new("test_scenario", 12345, &sim).unwrap();
        replay.finalize(100, 0);

        let mut player = ReplayPlayer::new(replay).unwrap();

        // Seek to tick 50
        assert!(player.seek(50).is_ok());
        assert_eq!(player.current_tick(), 50);

        // Seek back to tick 10
        assert!(player.seek(10).is_ok());
        assert_eq!(player.current_tick(), 10);
    }

    #[test]
    fn test_replay_player_pause() {
        let sim = create_test_simulation();
        let mut replay = Replay::new("test_scenario", 12345, &sim).unwrap();
        replay.finalize(100, 0);

        let mut player = ReplayPlayer::new(replay).unwrap();

        player.paused = true;
        let tick_before = player.current_tick();
        player.advance();
        assert_eq!(player.current_tick(), tick_before);

        player.toggle_pause();
        player.advance();
        assert_eq!(player.current_tick(), tick_before + 1);
    }

    #[test]
    fn test_replay_player_progress() {
        let sim = create_test_simulation();
        let mut replay = Replay::new("test_scenario", 12345, &sim).unwrap();
        replay.finalize(100, 0);

        let mut player = ReplayPlayer::new(replay).unwrap();

        assert!((player.progress_percent() - 0.0).abs() < 0.01);

        player.seek(50).unwrap();
        assert!((player.progress_percent() - 50.0).abs() < 0.01);

        player.seek(100).unwrap();
        assert!((player.progress_percent() - 100.0).abs() < 0.01);
    }
}
