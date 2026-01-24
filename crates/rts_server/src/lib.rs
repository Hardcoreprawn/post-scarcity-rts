//! # RTS Dedicated Server
//!
//! Headless dedicated server for multiplayer games.
//!
//! Runs the simulation without rendering, handling
//! network synchronization between clients.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]

pub mod lobby;
pub mod network;

/// Server configuration.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Port to listen on.
    pub port: u16,
    /// Maximum players per game.
    pub max_players: u8,
    /// Tick rate (should match client).
    pub tick_rate: u32,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 7777,
            max_players: 8,
            tick_rate: rts_core::simulation::TICK_RATE,
        }
    }
}
