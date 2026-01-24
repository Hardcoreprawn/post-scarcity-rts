//! Post-Scarcity RTS - Game Client

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting Post-Scarcity RTS");

    if let Err(e) = rts_game::run() {
        tracing::error!("Game error: {e}");
        std::process::exit(1);
    }
}
