# ADR-006: Tracing and Logging Strategy

## Status

Accepted

## Context

Debugging game issues requires visibility into what happened:

- Multiplayer desyncs need to trace divergence points
- Performance issues need timing information
- AI decisions need explanation for tuning
- Network issues need connection state tracking

We need structured logging that:

1. Works in both sync and async code
2. Supports filtering by level and module
3. Integrates with profiling tools
4. Doesn't impact release performance

## Decision

We will use the **`tracing`** crate ecosystem:

- `tracing`: Core instrumentation macros
- `tracing-subscriber`: Log output formatting
- `tracing-appender`: File output (optional)

### Log Levels

| Level | Use |
| ----- | --- |
| `error!` | Unrecoverable errors, bugs |
| `warn!` | Recoverable issues, degraded state |
| `info!` | Significant events (game start, player join) |
| `debug!` | Detailed flow (system execution, AI decisions) |
| `trace!` | Very detailed (every ECS query, network packet) |

### Structured Logging

Use key-value pairs, not string interpolation:

```rust
// Good: Structured, filterable
tracing::info!(
    player_id = %player.id,
    faction = ?faction,
    "Player selected faction"
);

// Bad: String concatenation
tracing::info!("Player {} selected faction {:?}", player.id, faction);
```

### Spans for Context

Wrap related operations in spans:

```rust
fn process_turn(turn: u32) {
    let _span = tracing::info_span!("turn", number = turn).entered();
    
    // All logs within this function include turn number
    process_commands();
    run_simulation();
    send_state();
}
```

### Configuration

```rust
// main.rs
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_tracing() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}
```

Set log level via environment:

```bash
# Show info and above
RUST_LOG=info cargo run

# Show debug for our crates only
RUST_LOG=warn,rts_core=debug,rts_game=debug cargo run

# Trace networking
RUST_LOG=rts_server::network=trace cargo run
```

### Performance Considerations

- `trace!` and `debug!` are compiled out in release (via feature flags)
- Use `tracing::instrument` attribute sparingly (has overhead)
- Expensive computations should be guarded:

```rust
if tracing::enabled!(tracing::Level::DEBUG) {
    let expensive_state = compute_debug_info();
    tracing::debug!(?expensive_state, "Debug info");
}
```

## Consequences

### Positive

- **Structured data**: Easy to filter, search, and aggregate
- **Async-safe**: Works correctly with async code and parallelism
- **Zero-cost disabled**: Release builds skip trace/debug logs
- **Profiler integration**: Can connect to Tracy, Chrome tracing

### Negative

- **Compile time**: `tracing` macros add compilation overhead
- **Learning curve**: More complex than `println!`
- **Verbosity**: Structured logging is more verbose to write

### Mitigations

- Create helper macros for common patterns
- Use IDE snippets for log statements
- Only instrument hot paths when debugging specific issues

## Related

- [ADR-005: Error Handling](./ADR-005-error-handling.md)
- [Coding Standards](../../standards/coding-standards.md)
