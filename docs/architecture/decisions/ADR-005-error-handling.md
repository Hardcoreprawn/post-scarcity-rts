# ADR-005: Error Handling Strategy

## Status

Accepted

## Context

Rust provides two error handling mechanisms:

1. **`panic!`**: Unrecoverable errors, crashes the thread
2. **`Result<T, E>`**: Recoverable errors, returned to caller

Games have unique requirements:

- Some errors are truly unrecoverable (corrupted save file)
- Some errors should be handled gracefully (network timeout)
- Some errors matter in debug but should be silent in release
- Error messages need to be actionable for debugging

We need a consistent strategy across all crates.

## Decision

### 1. Use `thiserror` for Error Types

Each domain has its own error enum:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GameError {
    #[error("Failed to load faction '{faction}': {message}")]
    FactionLoadError { faction: String, message: String },
    
    #[error("Desync at tick {tick}")]
    DesyncDetected { tick: u64, local_hash: u64, remote_hash: u64 },
}
```

### 2. Error Hierarchy

```text
GameError (top-level, covers all game errors)
├── DataError (loading, parsing)
├── SimulationError (invalid state, entity not found)
├── NetworkError (connection, desync)
└── IoError (file system)
```

### 3. When to Panic vs Return Error

| Situation | Action |
| --------- | ------ |
| Programmer bug (invariant violated) | `panic!` (indicates bug to fix) |
| User data error (bad save file) | `Result` with helpful message |
| External error (network down) | `Result`, allow retry |
| Impossible state (match arm) | `unreachable!()` |
| Debug-only validation | `debug_assert!()` |

### 4. Error Context with `anyhow` (Applications Only)

For binaries (`main.rs`), use `anyhow` to add context:

```rust
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let config = load_config("game.ron")
        .context("Failed to load game configuration")?;
    Ok(())
}
```

Libraries (`lib.rs`) should use concrete error types.

### 5. Tracing Integration

Errors should be logged when they occur:

```rust
fn load_faction(id: &str) -> Result<Faction, GameError> {
    let data = read_file(path).map_err(|e| {
        tracing::error!(%id, error = %e, "Failed to load faction");
        GameError::FactionLoadError { 
            faction: id.to_string(), 
            message: e.to_string() 
        }
    })?;
    Ok(data)
}
```

## Consequences

### Positive

- **Type-safe errors**: Compiler ensures all errors are handled
- **Actionable messages**: Context helps debugging
- **Consistent patterns**: Team knows what to expect
- **No string typing**: Structured errors can be matched on

### Negative

- **Boilerplate**: Defining error types takes effort
- **Conversion overhead**: `From` impls needed for error chaining

### Mitigations

- Start with broad error categories, refine as needed
- Use `#[from]` attribute for automatic conversion
- Keep error types in dedicated `error.rs` modules

## Related

- [Coding Standards - Error Handling](../../standards/coding-standards.md#error-handling)
- [ADR-006: Tracing and Logging](./ADR-006-tracing.md)
