//! Headless game runner for AI testing and CI verification.
//!
//! This crate provides a headless game runner that can be controlled via
//! JSON commands on stdin, with game state output on stdout. This enables:
//!
//! - **AI testing**: An AI agent can play the game without graphics
//! - **CI verification**: Automated testing of game logic and determinism
//! - **Replay verification**: Check that replays produce identical results
//!
//! # Protocol
//!
//! Communication uses JSON lines (one JSON object per line):
//!
//! - **stdin**: Commands from controller (tick, spawn, move, etc.)
//! - **stdout**: State updates and responses (JSON)
//! - **stderr**: Debug logs (human-readable)
//!
//! See [`protocol`] module for the full command/response specification.
//!
//! # Example
//!
//! ```bash
//! # Run interactively
//! echo '{"cmd":"tick","count":60}' | cargo run -p rts_headless
//!
//! # Run a scenario
//! cargo run -p rts_headless -- --scenario scenarios/skirmish.ron
//!
//! # Verify determinism
//! cargo run -p rts_headless -- --replay replay.bin --verify
//! ```

pub mod analyzer;
pub mod ascii_visualizer;
pub mod batch;
pub mod faction_loader;
pub mod game_runner;
pub mod metrics;
pub mod protocol;
pub mod runner;
pub mod scenario;
pub mod screenshot;
pub mod spawn_generator;
pub mod strategies;
pub mod visual_rating;
pub mod visual_review;

pub use analyzer::{analyze_batch, BalanceAnalysis, BalanceSuggestion};
pub use ascii_visualizer::{render_ascii, visualize_game_folder, AsciiConfig};
pub use batch::{run_batch, BatchConfig, BatchResults};
pub use faction_loader::{default_faction_data_dir, load_all_factions, FactionRegistry};
pub use game_runner::GameRunner;
pub use metrics::{BatchSummary, GameMetrics, MetricsCollector};
pub use protocol::{Command, Response};
pub use runner::HeadlessRunner;
pub use scenario::{MapSize, Scenario};
pub use screenshot::{
    ScreenshotConfig, ScreenshotManager, ScreenshotMode, ScreenshotPlugin, ScreenshotTrigger,
    VisualState,
};
pub use spawn_generator::{generate_dynamic_scenario, SpawnConfig, SpawnPattern};
pub use strategies::Strategy;
pub use visual_rating::{
    analyze_screenshots_in_dir, BatchVisualScore, VisualAnalyzer, VisualScore,
};
pub use visual_review::{BatchVisualReview, VisualQualityReport};
