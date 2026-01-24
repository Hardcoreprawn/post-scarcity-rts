# ADR-004: Cargo Workspace Structure

## Status

Accepted

## Context

As the project grows, a single crate becomes unwieldy:

- Compile times increase with crate size
- All code depends on all dependencies
- Hard to separate concerns (simulation vs rendering)
- Difficult to build headless server or tools

We need a structure that:

1. Enables parallel compilation
2. Separates deterministic core from rendering
3. Allows headless server builds
4. Supports development tools
5. Shares test utilities

## Decision

We will use a **Cargo workspace** with five crates:

```text
rts-game/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── rts_core/           # Deterministic simulation
│   ├── rts_game/           # Bevy game client
│   ├── rts_server/         # Headless dedicated server
│   ├── rts_tools/          # Development utilities
│   └── rts_test_utils/     # Shared test helpers
```

### Crate Responsibilities

| Crate | Purpose | Dependencies |
| ----- | ------- | ------------ |
| `rts_core` | Deterministic simulation, ECS, game logic | No Bevy, no IO, no randomness |
| `rts_game` | Bevy integration, rendering, UI, audio | `rts_core`, Bevy |
| `rts_server` | Dedicated multiplayer server | `rts_core`, networking only |
| `rts_tools` | Asset converters, validators, map editor | `rts_core`, CLI tools |
| `rts_test_utils` | Test fixtures, determinism harness | Minimal, used by all |

### Dependency Rules

```text
rts_game ────────────┐
                     ├──► rts_core ◄──── rts_server
rts_tools ───────────┘         ▲
                               │
rts_test_utils ────────────────┘ (dev-dependency for all)
```

**Critical rule**: `rts_core` has NO dependencies on Bevy, rendering, or IO. This ensures the simulation is portable and deterministic.

### Workspace Features

```toml
# Root Cargo.toml manages shared dependencies
[workspace.dependencies]
bevy = { version = "0.14", default-features = false }
serde = { version = "1.0", features = ["derive"] }
# ... all versions pinned here

# Child crates reference workspace
[dependencies]
bevy.workspace = true
```

## Consequences

### Positive

- **Parallel compilation**: Crates compile independently
- **Clear boundaries**: Simulation logic can't accidentally depend on rendering
- **Flexible builds**: Server builds skip Bevy entirely
- **Testability**: Core logic tests run fast (no Bevy startup)
- **Version consistency**: Workspace manages all dependency versions

### Negative

- **Boilerplate**: More Cargo.toml files to maintain
- **Module visibility**: Must explicitly export public APIs
- **Refactoring friction**: Moving code between crates is harder

### Mitigations

- Use workspace inheritance for common settings
- Document public API guidelines
- Prefer moving logic to core crate when possible

## Related

- [ADR-002: Rust + Bevy](./ADR-002-rust-bevy.md)
- [Project Conventions](../../standards/project-conventions.md)
