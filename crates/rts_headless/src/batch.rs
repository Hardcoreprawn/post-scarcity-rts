//! Batch game runner for balance testing.
//!
//! Runs multiple games in parallel using rayon to collect balance
//! metrics across many games efficiently.

use crate::faction_loader::FactionRegistry;
use crate::game_runner::{run_game, GameConfig};
use crate::metrics::{BatchSummary, GameMetrics};
use crate::scenario::Scenario;
use crate::screenshot::{ScreenshotConfig, ScreenshotMode};
use crate::strategies::Strategy;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Configuration for a batch run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// Scenario to run
    pub scenario: String,
    /// Number of games to run
    pub game_count: u32,
    /// Maximum parallel games (0 = use rayon default)
    pub parallel_games: u32,
    /// Output directory for results
    pub output_dir: PathBuf,
    /// Screenshot capture mode
    pub screenshot_mode: ScreenshotMode,
    /// Starting seed for deterministic runs
    pub seed_start: u64,
    /// Maximum ticks per game (0 = unlimited)
    pub max_ticks: u64,
    /// Strategy override for faction A
    pub strategy_a: Option<String>,
    /// Strategy override for faction B
    pub strategy_b: Option<String>,
    /// Path to faction data directory (optional, enables data-driven units)
    pub faction_data_path: Option<PathBuf>,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            scenario: "skirmish_1v1".to_string(),
            game_count: 100,
            parallel_games: 0,
            output_dir: PathBuf::from("results"),
            screenshot_mode: ScreenshotMode::Disabled,
            seed_start: 0,
            max_ticks: 36000, // 10 minutes at 60 tps
            strategy_a: None,
            strategy_b: None,
            faction_data_path: None,
        }
    }
}

impl BatchConfig {
    /// Create config for a specific scenario
    pub fn new(scenario: &str, game_count: u32) -> Self {
        Self {
            scenario: scenario.to_string(),
            game_count,
            ..Default::default()
        }
    }

    /// Set output directory
    pub fn with_output(mut self, dir: PathBuf) -> Self {
        self.output_dir = dir;
        self
    }

    /// Set seed start
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed_start = seed;
        self
    }

    /// Set strategies
    pub fn with_strategies(mut self, a: &str, b: &str) -> Self {
        self.strategy_a = Some(a.to_string());
        self.strategy_b = Some(b.to_string());
        self
    }
}

/// Results from a batch run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResults {
    /// Configuration used
    pub config: BatchConfig,
    /// Individual game metrics
    pub games: Vec<GameMetrics>,
    /// Aggregate summary
    pub summary: BatchSummary,
    /// Total runtime
    pub duration_seconds: f64,
    /// Errors encountered
    pub errors: Vec<BatchError>,
}

impl BatchResults {
    /// Save results to JSON file
    pub fn save(&self, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(path, json)
    }

    /// Load results from JSON file
    pub fn load(path: &std::path::Path) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        serde_json::from_str(&json).map_err(std::io::Error::other)
    }
}

/// Error during batch run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchError {
    /// Game index
    pub game_index: u32,
    /// Seed used
    pub seed: u64,
    /// Error message
    pub message: String,
}

/// Progress tracking for batch runs
#[derive(Debug)]
pub struct BatchProgress {
    /// Total games
    pub total: u32,
    /// Completed games
    pub completed: Arc<AtomicU32>,
    /// Start time
    pub start_time: Instant,
    /// Partial results for live stats
    partial_wins: Arc<std::sync::Mutex<std::collections::HashMap<String, u32>>>,
}

impl BatchProgress {
    /// Create new progress tracker
    pub fn new(total: u32) -> Self {
        Self {
            total,
            completed: Arc::new(AtomicU32::new(0)),
            start_time: Instant::now(),
            partial_wins: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Record a completed game
    pub fn record_completion(&self, winner: Option<&str>) {
        self.completed.fetch_add(1, Ordering::Relaxed);
        if let Some(w) = winner {
            if let Ok(mut wins) = self.partial_wins.lock() {
                *wins.entry(w.to_string()).or_insert(0) += 1;
            }
        }
    }

    /// Get current completion count
    pub fn current(&self) -> u32 {
        self.completed.load(Ordering::Relaxed)
    }

    /// Get completion percentage
    pub fn percentage(&self) -> f64 {
        self.current() as f64 / self.total.max(1) as f64 * 100.0
    }

    /// Get estimated time remaining
    pub fn eta(&self) -> Duration {
        let completed = self.current();
        if completed == 0 {
            return Duration::from_secs(0);
        }

        let elapsed = self.start_time.elapsed();
        let per_game = elapsed.as_secs_f64() / completed as f64;
        let remaining = self.total.saturating_sub(completed);
        Duration::from_secs_f64(per_game * remaining as f64)
    }

    /// Get current win rates
    pub fn current_win_rates(&self) -> std::collections::HashMap<String, f64> {
        let completed = self.current();
        if completed == 0 {
            return std::collections::HashMap::new();
        }

        if let Ok(wins) = self.partial_wins.lock() {
            wins.iter()
                .map(|(k, v)| (k.clone(), *v as f64 / completed as f64))
                .collect()
        } else {
            std::collections::HashMap::new()
        }
    }

    /// Display progress to stderr
    pub fn display(&self) {
        let completed = self.current();
        let eta = self.eta();
        let rates = self.current_win_rates();

        eprintln!("╔════════════════════════════════════╗");
        eprintln!(
            "║ Batch Progress: {:>4}/{:<4} ({:>5.1}%) ║",
            completed,
            self.total,
            self.percentage()
        );
        eprintln!(
            "║ ETA: {:>28} ║",
            format!("{}m {}s", eta.as_secs() / 60, eta.as_secs() % 60)
        );
        if !rates.is_empty() {
            eprintln!("╟────────────────────────────────────╢");
            eprintln!("║ Win Rates So Far:                  ║");
            for (faction, rate) in &rates {
                eprintln!("║   {:<12}: {:>5.1}%              ║", faction, rate * 100.0);
            }
        }
        eprintln!("╚════════════════════════════════════╝");
    }
}

/// Run a single game using the real simulation engine.
fn run_single_game(
    scenario: &str,
    seed: u64,
    config: &BatchConfig,
    faction_registry: Option<Arc<FactionRegistry>>,
) -> Result<GameMetrics, String> {
    use crate::spawn_generator::{generate_dynamic_scenario, SpawnConfig};

    // Load or create base scenario
    let base_scenario = if scenario == "skirmish_1v1" {
        Scenario::skirmish_1v1()
    } else {
        Scenario::default()
    };

    // Apply dynamic spawns based on seed
    let spawn_config = SpawnConfig::default();
    let scenario_data = generate_dynamic_scenario(seed, &base_scenario, &spawn_config);

    // Parse or use default strategies
    let strategy_a = config
        .strategy_a
        .as_ref()
        .map(|s| match s.as_str() {
            "rush" => Strategy::rush(),
            "economic" | "eco" => Strategy::economic(),
            "balanced" => Strategy::default(),
            "turtle" => Strategy::turtle(),
            "harassment" => Strategy::harassment(),
            "fast_expand" => Strategy::fast_expand(),
            "all_in" => Strategy::all_in(),
            _ => Strategy::default(),
        })
        .unwrap_or_default();

    let strategy_b = config
        .strategy_b
        .as_ref()
        .map(|s| match s.as_str() {
            "rush" => Strategy::rush(),
            "economic" | "eco" => Strategy::economic(),
            "balanced" => Strategy::default(),
            "turtle" => Strategy::turtle(),
            "harassment" => Strategy::harassment(),
            "fast_expand" => Strategy::fast_expand(),
            "all_in" => Strategy::all_in(),
            _ => Strategy::default(),
        })
        .unwrap_or_default();

    // Screenshot config if enabled
    let screenshot_config = if config.screenshot_mode != ScreenshotMode::Disabled {
        Some(ScreenshotConfig::new(
            config.screenshot_mode,
            config.output_dir.join("screenshots"),
            &format!("game_{}", seed),
        ))
    } else {
        None
    };

    let game_config = GameConfig {
        seed,
        max_ticks: config.max_ticks,
        scenario: scenario_data,
        strategy_a,
        strategy_b,
        screenshot_config,
        game_id: format!("game_{}", seed),
        faction_registry,
    };

    let result = run_game(game_config);
    let mut metrics = result.metrics;
    metrics.final_state_hash = result.final_state_hash;
    Ok(metrics)
}

/// Run a batch of games
pub fn run_batch(config: BatchConfig) -> BatchResults {
    use crate::faction_loader::load_factions_from_path;

    let start = Instant::now();
    let progress = BatchProgress::new(config.game_count);
    let progress_arc = Arc::new(progress);

    info!(
        "Starting batch run: {} games of '{}'",
        config.game_count, config.scenario
    );

    // Load faction data if path is provided
    let faction_registry: Option<Arc<FactionRegistry>> =
        if let Some(ref path) = config.faction_data_path {
            match load_factions_from_path(path) {
                Ok(registry) => {
                    info!(
                        "Loaded faction data from {:?}: {} factions",
                        path,
                        registry.faction_count()
                    );
                    Some(Arc::new(registry))
                }
                Err(e) => {
                    warn!(
                        "Failed to load faction data from {:?}: {}, using defaults",
                        path, e
                    );
                    None
                }
            }
        } else {
            None
        };

    // Configure thread pool if specified
    if config.parallel_games > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(config.parallel_games as usize)
            .build_global()
            .ok(); // Ignore if already set
    }

    let results: Vec<Result<GameMetrics, BatchError>> = (0..config.game_count)
        .into_par_iter()
        .map(|i| {
            let seed = config.seed_start.wrapping_add(i as u64);
            let registry_clone = faction_registry.clone();

            match run_single_game(&config.scenario, seed, &config, registry_clone) {
                Ok(metrics) => {
                    progress_arc.record_completion(metrics.winner.as_deref());

                    let completed = progress_arc.current();
                    if completed % 10 == 0 {
                        debug!("Progress: {}/{}", completed, config.game_count);
                    }
                    if completed % 100 == 0 {
                        progress_arc.display();
                    }

                    Ok(metrics)
                }
                Err(e) => {
                    warn!("Game {} failed: {}", i, e);
                    Err(BatchError {
                        game_index: i,
                        seed,
                        message: e,
                    })
                }
            }
        })
        .collect();

    let (games, errors): (Vec<_>, Vec<_>) = results.into_iter().partition(Result::is_ok);
    let games: Vec<GameMetrics> = games.into_iter().filter_map(Result::ok).collect();
    let errors: Vec<BatchError> = errors.into_iter().filter_map(Result::err).collect();

    let summary = BatchSummary::from_games(&games);
    let duration_seconds = start.elapsed().as_secs_f64();

    info!(
        "Batch complete: {} games in {:.1}s ({:.1} games/sec)",
        games.len(),
        duration_seconds,
        games.len() as f64 / duration_seconds
    );

    BatchResults {
        config,
        games,
        summary,
        duration_seconds,
        errors,
    }
}

/// Verify determinism by running same seeds multiple times
pub fn verify_determinism(scenario: &str, seed: u64, runs: u32) -> bool {
    let results: Vec<GameMetrics> = (0..runs)
        .map(|_| {
            run_single_game(scenario, seed, &BatchConfig::default(), None)
                .expect("Game should complete")
        })
        .collect();

    // All runs should have same outcome
    let first = &results[0];
    results.iter().all(|r| {
        r.winner == first.winner
            && r.duration_ticks == first.duration_ticks
            && r.win_condition == first.win_condition
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_config_default() {
        let config = BatchConfig::default();
        assert_eq!(config.game_count, 100);
        assert_eq!(config.scenario, "skirmish_1v1");
    }

    #[test]
    fn test_batch_config_builder() {
        let config = BatchConfig::new("custom_scenario", 500)
            .with_output(PathBuf::from("/tmp/results"))
            .with_seed(12345);

        assert_eq!(config.scenario, "custom_scenario");
        assert_eq!(config.game_count, 500);
        assert_eq!(config.seed_start, 12345);
    }

    #[test]
    fn test_progress_tracking() {
        let progress = BatchProgress::new(100);
        assert_eq!(progress.current(), 0);
        assert_eq!(progress.percentage(), 0.0);

        progress.record_completion(Some("faction_a"));
        progress.record_completion(Some("faction_b"));
        progress.record_completion(Some("faction_a"));

        assert_eq!(progress.current(), 3);

        let rates = progress.current_win_rates();
        assert!((rates["faction_a"] - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_run_batch_small() {
        let config = BatchConfig::new("test", 10);
        let results = run_batch(config);

        assert_eq!(results.games.len(), 10);
        assert!(results.errors.is_empty());
        assert!(results.duration_seconds > 0.0);
    }

    #[test]
    fn test_batch_summary_calculated() {
        let config = BatchConfig::new("test", 20);
        let results = run_batch(config);

        // Summary should have win rates
        assert!(results.summary.total_games > 0);
    }

    #[test]
    fn test_verify_determinism() {
        // Our stub is deterministic
        assert!(verify_determinism("test", 12345, 5));
    }

    #[test]
    fn test_batch_results_save_load() {
        let config = BatchConfig::new("test", 5);
        let results = run_batch(config);

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("results.json");

        results.save(&path).unwrap();
        assert!(path.exists());

        let loaded = BatchResults::load(&path).unwrap();
        assert_eq!(loaded.games.len(), 5);
        assert_eq!(loaded.config.scenario, "test");
    }
}
