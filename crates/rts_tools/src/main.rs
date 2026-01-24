//! Post-Scarcity RTS - Development Tools

use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "rts-tools")]
#[command(about = "Development tools for Post-Scarcity RTS")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate data files
    Validate {
        /// Path to data directory
        #[arg(default_value = "assets/data")]
        path: String,
    },
}

fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { path } => {
            tracing::info!("Validating data files in: {path}");
            match rts_tools::validate::validate_data_directory(std::path::Path::new(&path)) {
                Ok(()) => tracing::info!("Validation passed"),
                Err(e) => {
                    tracing::error!("Validation failed: {e}");
                    std::process::exit(1);
                }
            }
        }
    }
}
