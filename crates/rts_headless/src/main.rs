//! Headless RTS game runner.
//!
//! This binary runs the game without graphics, controlled via JSON on stdin/stdout.
//! Designed for AI agents, CI testing, and replay verification.
//!
//! # Usage
//!
//! ```bash
//! # Interactive mode - read commands from stdin
//! cargo run -p rts_headless
//!
//! # Run a single game with scenario
//! cargo run -p rts_headless -- run --scenario skirmish_1v1
//!
//! # Run batch balance test
//! cargo run -p rts_headless -- batch --scenario skirmish_1v1 --count 1000 --output results/
//!
//! # Analyze batch results
//! cargo run -p rts_headless -- analyze --input results/batch.json --suggest
//!
//! # Generate visual review report
//! cargo run -p rts_headless -- review --screenshots results/screenshots --output report.html
//! ```
//!
//! # Protocol
//!
//! Input (stdin): JSON commands, one per line
//! Output (stdout): JSON responses, one per line
//! Logs (stderr): Debug information
//!
//! See the protocol module for command/response format.

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use rts_headless::{
    analyzer::analyze_batch,
    ascii_visualizer::{render_ascii, visualize_game_folder, AsciiConfig, ScreenshotState},
    batch::{run_batch, BatchConfig, BatchResults},
    runner::{HeadlessConfig, HeadlessRunner},
    screenshot::ScreenshotMode,
    visual_review::BatchVisualReview,
};

#[derive(Parser)]
#[command(name = "rts_headless")]
#[command(about = "Headless RTS game runner for AI testing and CI")]
#[command(version)]
struct Cli {
    /// Enable verbose logging to stderr
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a single interactive game
    Run {
        /// Scenario file to load
        #[arg(short, long)]
        scenario: Option<String>,

        /// Output state after every tick
        #[arg(long)]
        auto_state: bool,
    },

    /// Run batch of games for balance testing
    Batch {
        /// Scenario to run
        #[arg(short, long, default_value = "skirmish_1v1")]
        scenario: String,

        /// Number of games to run
        #[arg(short, long, default_value = "100")]
        count: u32,

        /// Maximum parallel games (0 = auto)
        #[arg(short, long, default_value = "0")]
        parallel: u32,

        /// Output directory for results
        #[arg(short, long, default_value = "results")]
        output: PathBuf,

        /// Starting random seed
        #[arg(long, default_value = "0")]
        seed: u64,

        /// Enable screenshot capture
        #[arg(long)]
        screenshots: bool,

        /// Path to faction data directory for data-driven unit stats
        #[arg(long)]
        faction_data: Option<PathBuf>,

        /// Maximum game duration in minutes (game time, not wall clock)
        /// Default: 10 min for batch, 30 min for interactive
        #[arg(long, default_value = "10")]
        duration_minutes: u32,

        /// Quick mode: 5-minute games for rapid iteration
        #[arg(long, conflicts_with = "duration_minutes")]
        quick: bool,

        /// Extended mode: 60-minute games for late-game testing
        #[arg(long, conflicts_with = "duration_minutes")]
        extended: bool,
    },

    /// Analyze batch results and suggest balance changes
    Analyze {
        /// Input batch results JSON file
        #[arg(short, long)]
        input: PathBuf,

        /// Generate balance suggestions
        #[arg(long)]
        suggest: bool,

        /// Output markdown report
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate visual review report from screenshots
    Review {
        /// Screenshot manifest or directory
        #[arg(short, long)]
        screenshots: PathBuf,

        /// Output HTML report path
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Display ASCII visualization of game screenshots
    Visualize {
        /// Screenshot JSON file or directory containing screenshots
        #[arg(short, long)]
        path: PathBuf,

        /// Width of ASCII output
        #[arg(long, default_value = "80")]
        width: usize,

        /// Height of ASCII output  
        #[arg(long, default_value = "24")]
        height: usize,

        /// Disable colored output
        #[arg(long)]
        no_color: bool,
    },

    /// Verify determinism by running same seed multiple times
    Verify {
        /// Scenario to test
        #[arg(short, long, default_value = "skirmish_1v1")]
        scenario: String,

        /// Seed to verify
        #[arg(long, default_value = "12345")]
        seed: u64,

        /// Number of verification runs
        #[arg(short, long, default_value = "5")]
        runs: u32,
    },

    /// Replay a recorded game
    Replay {
        /// Replay file path
        #[arg(short, long)]
        file: PathBuf,

        /// Verify replay produces identical hash
        #[arg(long)]
        verify: bool,
    },

    /// Run N ticks for benchmarking
    Benchmark {
        /// Number of ticks to run
        #[arg(short, long, default_value = "36000")]
        ticks: u64,

        /// Scenario to benchmark
        #[arg(short, long)]
        scenario: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    // Initialize logging to stderr (stdout is for protocol)
    let log_level = if cli.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(true),
        )
        .with(tracing_subscriber::filter::LevelFilter::from_level(
            log_level,
        ))
        .init();

    match cli.command {
        Some(Commands::Run {
            scenario,
            auto_state,
        }) => {
            cmd_run(scenario, auto_state);
        }
        Some(Commands::Batch {
            scenario,
            count,
            parallel,
            output,
            seed,
            screenshots,
            faction_data,
            duration_minutes,
            quick,
            extended,
        }) => {
            cmd_batch(
                scenario,
                count,
                parallel,
                output,
                seed,
                screenshots,
                faction_data,
                duration_minutes,
                quick,
                extended,
            );
        }
        Some(Commands::Analyze {
            input,
            suggest,
            output,
        }) => {
            cmd_analyze(input, suggest, output);
        }
        Some(Commands::Review {
            screenshots,
            output,
        }) => {
            cmd_review(screenshots, output);
        }
        Some(Commands::Visualize {
            path,
            width,
            height,
            no_color,
        }) => {
            cmd_visualize(path, width, height, no_color);
        }
        Some(Commands::Verify {
            scenario,
            seed,
            runs,
        }) => {
            cmd_verify(scenario, seed, runs);
        }
        Some(Commands::Replay { file, verify }) => {
            cmd_replay(file, verify);
        }
        Some(Commands::Benchmark { ticks, scenario }) => {
            cmd_benchmark(ticks, scenario);
        }
        None => {
            // Default: interactive mode
            cmd_run(None, false);
        }
    }
}

/// Run a single interactive game
fn cmd_run(scenario: Option<String>, auto_state: bool) {
    tracing::info!("Starting interactive session");

    let config = HeadlessConfig {
        auto_state_output: auto_state,
        scenario_path: scenario,
    };

    let runner = HeadlessRunner::with_config(config);
    runner.run();
}

/// Run batch of games for balance testing
fn cmd_batch(
    scenario: String,
    count: u32,
    parallel: u32,
    output: PathBuf,
    seed: u64,
    screenshots: bool,
    faction_data: Option<PathBuf>,
    duration_minutes: u32,
    quick: bool,
    extended: bool,
) {
    use rts_headless::batch::EXTENDED_DEFAULT_MAX_TICKS;
    use std::time::Instant;

    let batch_start = Instant::now();

    // Determine max_ticks from duration options
    // Ticks = minutes * 60 seconds * 60 ticks/second
    const TICKS_PER_MINUTE: u64 = 60 * 60;
    let max_ticks = if quick {
        5 * TICKS_PER_MINUTE // 5 minutes - rapid testing
    } else if extended {
        EXTENDED_DEFAULT_MAX_TICKS // 60 minutes - late game testing
    } else {
        (duration_minutes as u64) * TICKS_PER_MINUTE
    };

    // System diagnostics
    let num_cpus = std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(1);

    let game_duration_str = if max_ticks >= 60 * TICKS_PER_MINUTE {
        format!("{} hour(s)", max_ticks / (60 * TICKS_PER_MINUTE))
    } else {
        format!("{} minutes", max_ticks / TICKS_PER_MINUTE)
    };

    tracing::info!(
        scenario = %scenario,
        count = count,
        parallel = parallel,
        seed = seed,
        output = %output.display(),
        cpus_available = num_cpus,
        screenshots = screenshots,
        faction_data = ?faction_data,
        max_ticks = max_ticks,
        game_duration = %game_duration_str,
        "Batch configuration"
    );

    // Ensure output directory exists
    if let Err(e) = std::fs::create_dir_all(&output) {
        tracing::error!(error = %e, path = %output.display(), "Failed to create output directory");
        eprintln!(
            "FATAL: Cannot create output directory '{}': {}",
            output.display(),
            e
        );
        std::process::exit(1);
    }

    tracing::info!(
        "Starting batch run: {} games of '{}' ({} game time each)",
        count,
        scenario,
        game_duration_str
    );

    let config = BatchConfig {
        scenario,
        game_count: count,
        parallel_games: parallel,
        output_dir: output.clone(),
        screenshot_mode: if screenshots {
            ScreenshotMode::StateDump
        } else {
            ScreenshotMode::Disabled
        },
        seed_start: seed,
        max_ticks,
        strategy_a: None,
        strategy_b: None,
        faction_data_path: faction_data,
    };

    let results = run_batch(config);

    let batch_duration = batch_start.elapsed();

    tracing::info!(
        games_completed = results.games.len(),
        games_failed = results.errors.len(),
        total_duration_secs = format!("{:.1}", batch_duration.as_secs_f64()),
        "Batch execution finished"
    );

    // Save results
    let results_path = output.join("batch_results.json");
    if let Err(e) = results.save(&results_path) {
        tracing::error!(error = %e, path = %results_path.display(), "Failed to save results");
        eprintln!("FATAL: Failed to save results: {}", e);
        std::process::exit(1);
    }

    // Print summary
    eprintln!("\n{}", "=".repeat(50));
    eprintln!("BATCH COMPLETE");
    eprintln!("{}", "=".repeat(50));
    eprintln!("Games played: {}", results.games.len());
    if !results.errors.is_empty() {
        eprintln!("Games FAILED: {} ⚠️", results.errors.len());
    }
    eprintln!("Duration: {:.1}s", results.duration_seconds);
    eprintln!(
        "Throughput: {:.1} games/sec",
        results.games.len() as f64 / results.duration_seconds.max(0.001)
    );
    eprintln!("\nWin Rates:");
    for (faction, rate) in &results.summary.win_rates {
        eprintln!("  {}: {:.1}%", faction, rate * 100.0);
    }

    // Report errors if any
    if !results.errors.is_empty() {
        eprintln!("\n⚠️  GAME FAILURES:");
        for error in results.errors.iter().take(10) {
            eprintln!(
                "  Game {} (seed {}): {}",
                error.game_index, error.seed, error.message
            );
        }
        if results.errors.len() > 10 {
            eprintln!("  ... and {} more failures", results.errors.len() - 10);
        }
    }

    eprintln!("\nResults saved to: {}", results_path.display());

    // Run quick analysis
    let analysis = analyze_batch(&results);
    if !analysis.outliers.is_empty() {
        eprintln!("\nBalance Issues Detected:");
        for outlier in analysis.outliers_by_severity().iter().take(3) {
            eprintln!(
                "  [{:?}] {}/{}: {:.2}",
                outlier.severity, outlier.category, outlier.metric, outlier.value
            );
        }
    }
}

/// Analyze batch results
fn cmd_analyze(input: PathBuf, suggest: bool, output: Option<PathBuf>) {
    tracing::info!("Loading batch results from: {}", input.display());

    let results = match BatchResults::load(&input) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to load results: {}", e);
            std::process::exit(1);
        }
    };

    let analysis = analyze_batch(&results);

    // Output report
    let report = analysis.to_markdown();

    if let Some(out_path) = output {
        if let Err(e) = std::fs::write(&out_path, &report) {
            eprintln!("Failed to write report: {}", e);
            std::process::exit(1);
        }
        eprintln!("Report saved to: {}", out_path.display());
    } else {
        println!("{}", report);
    }

    if suggest && !analysis.suggestions.is_empty() {
        eprintln!("\nBalance Suggestions:");
        for (i, s) in analysis.suggestions_by_confidence().iter().enumerate() {
            eprintln!(
                "  {}. {} -> {} (confidence: {:.0}%)",
                i + 1,
                s.target,
                s.suggested,
                s.confidence * 100.0
            );
            eprintln!("     {}", s.reasoning);
        }
    }
}

/// Generate visual review report
fn cmd_review(screenshots: PathBuf, output: PathBuf) {
    tracing::info!("Generating visual review from: {}", screenshots.display());

    // Load or create batch review
    let review = if screenshots.is_file() {
        match BatchVisualReview::load(&screenshots) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to load review: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Scan directory for manifests
        let review = BatchVisualReview::new("visual_review");
        eprintln!("Note: Directory scanning not yet implemented");
        eprintln!("Please provide a review JSON file directly");
        review
    };

    // Generate HTML
    let html = review.to_html();
    if let Err(e) = std::fs::write(&output, &html) {
        eprintln!("Failed to write report: {}", e);
        std::process::exit(1);
    }

    eprintln!("Visual review report saved to: {}", output.display());
    eprintln!("  Screenshots: {}", review.reports.len());
    eprintln!("  Average score: {:.1}", review.average_score);
    eprintln!("  Pass rate: {:.1}%", review.pass_rate * 100.0);
}

/// Display ASCII visualization of game screenshots
fn cmd_visualize(path: PathBuf, width: usize, height: usize, no_color: bool) {
    tracing::info!("Visualizing: {}", path.display());

    let config = AsciiConfig {
        width,
        height,
        show_health: true,
        show_legend: true,
        use_color: !no_color,
    };

    if path.is_file() {
        // Single file visualization
        match ScreenshotState::load(&path) {
            Ok(state) => {
                let output = render_ascii(&state, &config);
                println!("{}", output);
            }
            Err(e) => {
                eprintln!("Failed to load screenshot: {}", e);
                std::process::exit(1);
            }
        }
    } else if path.is_dir() {
        // Directory visualization - show all screenshots
        match visualize_game_folder(&path, &config) {
            Ok(output) => {
                println!("{}", output);
            }
            Err(e) => {
                eprintln!("Failed to visualize directory: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("Path does not exist: {}", path.display());
        std::process::exit(1);
    }
}

/// Verify determinism
fn cmd_verify(scenario: String, seed: u64, runs: u32) {
    tracing::info!(
        "Verifying determinism: {} with seed {} ({} runs)",
        scenario,
        seed,
        runs
    );

    let deterministic = rts_headless::batch::verify_determinism(&scenario, seed, runs);

    if deterministic {
        eprintln!("PASS: All {} runs produced identical results", runs);
    } else {
        eprintln!("FAIL: Non-determinism detected!");
        std::process::exit(1);
    }
}

/// Replay a recorded game
fn cmd_replay(file: PathBuf, verify: bool) {
    use rts_core::replay::{Replay, ReplayPlayer};

    if verify {
        tracing::info!("Verifying replay: {}", file.display());
    } else {
        tracing::info!("Playing replay: {}", file.display());
    }

    // Load the replay
    let replay = match Replay::load(&file) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to load replay: {}", e);
            std::process::exit(1);
        }
    };

    eprintln!("Loaded replay:");
    eprintln!("  Scenario: {}", replay.scenario_id);
    eprintln!("  Seed: {}", replay.seed);
    eprintln!("  Commands: {}", replay.command_count());
    eprintln!("  Duration: {} ticks", replay.duration());

    // Create player
    let mut player = match ReplayPlayer::new(replay) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to create replay player: {}", e);
            std::process::exit(1);
        }
    };

    if verify {
        // Verify mode: play to end and check hash
        eprintln!("Verifying replay...");
        match player.verify() {
            Ok(true) => {
                eprintln!("PASS: Replay verification successful");
                eprintln!("  Expected hash: {:016x}", player.replay().final_hash);
                eprintln!("  Actual hash:   {:016x}", player.simulation().state_hash());
            }
            Ok(false) => {
                eprintln!("FAIL: Replay produced different hash!");
                eprintln!("  Expected: {:016x}", player.replay().final_hash);
                eprintln!("  Actual:   {:016x}", player.simulation().state_hash());
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("FAIL: Error during verification: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Play mode: run replay and show progress
        let total = player.replay().duration();
        let mut last_percent = 0;

        while player.advance() {
            let percent = (player.current_tick() * 100 / total.max(1)) as u32;
            if percent > last_percent && percent % 10 == 0 {
                eprintln!("Progress: {}%", percent);
                last_percent = percent;
            }
        }

        eprintln!("Replay complete at tick {}", player.current_tick());
        eprintln!(
            "Final state hash: {:016x}",
            player.simulation().state_hash()
        );

        // Print final game state summary
        let sim = player.simulation();
        eprintln!("\nFinal State:");
        eprintln!("  Entities: {}", sim.entities().len());
    }
}

/// Run benchmark
fn cmd_benchmark(ticks: u64, scenario: Option<String>) {
    use rts_headless::scenario::Scenario;
    use std::time::Instant;

    tracing::info!("Running {} tick benchmark", ticks);

    // Load scenario
    let scenario_data = if let Some(s) = &scenario {
        tracing::info!("Using scenario: {}", s);
        match Scenario::load(s) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to load scenario: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        Scenario::skirmish_1v1()
    };

    // Create simulation from scenario
    let mut sim = rts_core::simulation::Simulation::with_nav_grid(
        scenario_data.map_size.0,
        scenario_data.map_size.1,
        rts_core::math::Fixed::from_num(32),
    );

    // Spawn initial entities
    for faction in &scenario_data.factions {
        let faction_id = match faction.faction_id.as_str() {
            "continuity" => rts_core::factions::FactionId::Continuity,
            "collegium" => rts_core::factions::FactionId::Collegium,
            _ => rts_core::factions::FactionId::Continuity,
        };

        for unit in &faction.starting_units {
            for _ in 0..unit.count {
                sim.spawn_entity(rts_core::simulation::EntitySpawnParams {
                    position: Some(rts_core::math::Vec2Fixed::new(
                        rts_core::math::Fixed::from_num(unit.position.0),
                        rts_core::math::Fixed::from_num(unit.position.1),
                    )),
                    health: Some(100),
                    movement: Some(rts_core::math::Fixed::from_num(10)),
                    combat_stats: Some(rts_core::components::CombatStats::default()),
                    faction: Some(rts_core::components::FactionMember::new(faction_id, 0)),
                    ..Default::default()
                });
            }
        }
    }

    eprintln!("Starting benchmark with {} entities", sim.entities().len());
    eprintln!("Running {} ticks...", ticks);

    // Warmup
    for _ in 0..100 {
        sim.tick();
    }

    // Benchmark
    let start = Instant::now();
    for _ in 0..ticks {
        sim.tick();
    }
    let elapsed = start.elapsed();

    let tps = ticks as f64 / elapsed.as_secs_f64();

    eprintln!("\n{}", "=".repeat(50));
    eprintln!("BENCHMARK RESULTS");
    eprintln!("{}", "=".repeat(50));
    eprintln!("Ticks: {}", ticks);
    eprintln!("Duration: {:.3}s", elapsed.as_secs_f64());
    eprintln!("Ticks/second: {:.1}", tps);
    eprintln!("ms/tick: {:.4}", elapsed.as_millis() as f64 / ticks as f64);
    eprintln!("Final entities: {}", sim.entities().len());
    eprintln!("State hash: {:016x}", sim.state_hash());
}
