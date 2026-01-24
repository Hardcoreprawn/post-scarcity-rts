//! Post-Scarcity RTS - Dedicated Server

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting Post-Scarcity RTS Dedicated Server");

    let config = rts_server::ServerConfig::default();
    tracing::info!("Listening on port {}", config.port);

    // TODO: Start server loop
}
