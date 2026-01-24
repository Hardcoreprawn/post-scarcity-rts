//! Error types for the game simulation.

use thiserror::Error;

/// Result type alias using [`GameError`].
pub type Result<T> = std::result::Result<T, GameError>;

/// Top-level error type for all game simulation errors.
#[derive(Debug, Error)]
pub enum GameError {
    /// Failed to load faction data.
    #[error("Failed to load faction data: {0}")]
    FactionLoadError(String),

    /// Invalid unit identifier.
    #[error("Invalid unit ID: {0}")]
    InvalidUnitId(u32),

    /// Invalid building identifier.
    #[error("Invalid building ID: {0}")]
    InvalidBuildingId(u32),

    /// Invalid entity reference.
    #[error("Entity not found: {0}")]
    EntityNotFound(u64),

    /// Data file parsing error.
    #[error("Failed to parse data file '{path}': {message}")]
    DataParseError {
        /// Path to the file that failed to parse.
        path: String,
        /// Error message.
        message: String,
    },

    /// Tech tree requirement not met.
    #[error("Tech requirement not met: {0}")]
    TechRequirementNotMet(String),

    /// Insufficient resources.
    #[error("Insufficient resources: need {required} {resource}, have {available}")]
    InsufficientResources {
        /// Resource type.
        resource: String,
        /// Amount required.
        required: u32,
        /// Amount available.
        available: u32,
    },

    /// Invalid game state.
    #[error("Invalid game state: {0}")]
    InvalidState(String),

    /// Desync detected in multiplayer.
    #[error("Desync detected at tick {tick}: local hash {local_hash}, remote hash {remote_hash}")]
    DesyncDetected {
        /// Tick where desync occurred.
        tick: u64,
        /// Local simulation hash.
        local_hash: u64,
        /// Remote simulation hash.
        remote_hash: u64,
    },
}
