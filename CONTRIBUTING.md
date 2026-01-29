# Contributing to Post-Scarcity RTS

Thank you for your interest in contributing! This document covers guidelines for both human developers and AI coding assistants.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Code Contribution Workflow](#code-contribution-workflow)
- [Coding Guidelines](#coding-guidelines)
- [AI Assistant Guidelines](#ai-assistant-guidelines)
- [Testing Requirements](#testing-requirements)
- [Documentation](#documentation)

## Getting Started

### Prerequisites

- **Rust 1.75+**: Install via [rustup](https://rustup.rs/)
- **Git**: For version control
- **Git Bash** (Windows) or standard shell (Linux/macOS)

### Quick Start

```bash
# Clone the repository
git clone https://github.com/your-org/rts-game.git
cd rts-game

# Install git hooks
sh tools/install-hooks.sh

# Build the workspace
cargo build

# Run tests
cargo test --workspace

# Run the game (with dev tools)
cargo run -p rts_game --features dev-tools
```

## Development Setup

### IDE Setup

**VS Code (Recommended):**

1. Install `rust-analyzer` extension
2. Install `Even Better TOML` extension
3. Install `crates` extension for dependency management

**Settings for this project:**

```json
{
  "rust-analyzer.cargo.features": ["dev-tools"],
  "rust-analyzer.check.command": "clippy"
}
```

### Environment Variables

```bash
# Set log level (options: error, warn, info, debug, trace)
RUST_LOG=info

# Enable debug logging for specific crates
RUST_LOG=warn,rts_core=debug,rts_game=debug

# Enable Bevy's dynamic linking for faster iteration (dev only)
BEVY_DYNAMIC_LINKING=1
```

## Code Contribution Workflow

### Branch Naming

```text
feature/faction-tech-trees     # New features
bugfix/pathfinding-stuck       # Bug fixes
docs/api-documentation         # Documentation
refactor/ecs-cleanup           # Code refactoring
```

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```text
type(scope): brief description

Longer explanation if needed.

Fixes #123
```

**Types:** `feat`, `fix`, `docs`, `refactor`, `test`, `perf`, `chore`

**Examples:**

```text
feat(factions): add Bio-Sovereign creep spread system
fix(combat): correct armor calculation for explosive damage
docs(adr): add decision record for fixed-point math
refactor(core): extract component registration into trait
test(determinism): add replay comparison test
```

### Pull Request Process

1. Create a feature branch from `develop`
2. Make your changes with clear commits
3. Ensure all checks pass:
   - `cargo fmt --check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test --workspace`
   - `cargo doc --no-deps`
4. Open PR against `develop`
5. Fill out the PR template
6. Address review feedback
7. Squash and merge when approved

**CI Note:**

- Pull requests run a **fast path** (format, clippy, and tests on Ubuntu).
- Full matrix tests (Linux/Windows/macOS) + docs + release build run on `main`/`develop` pushes.

## Coding Guidelines

### Must Follow

1. **No `unsafe` code** without explicit approval and documentation
2. **All public APIs must be documented** with `///` doc comments
3. **Use fixed-point math in `rts_core`** — no floating point in simulation
4. **Errors must be typed** — use `thiserror`, not string errors
5. **Format with `rustfmt`** — enforced by pre-commit
6. **Pass `clippy` without warnings** — enforced by CI

### Strongly Recommended

1. **Prefer returning `Result` over panicking**
2. **Use `tracing` for logging**, not `println!`
3. **Add tests for new functionality**
4. **Update documentation when changing behavior**
5. **Keep functions small and focused**

### Crate-Specific Rules

#### `rts_core` (Deterministic Simulation)

- **NO** Bevy dependencies
- **NO** floating-point math
- **NO** system randomness (use seeded RNG only)
- **NO** IO operations (file, network)
- **NO** `std::time` in simulation logic
- Components must derive `Serialize, Deserialize`

#### `rts_game` (Bevy Client)

- Keep Bevy-specific code separate from game logic
- Use resources for shared state
- Prefer systems over direct function calls

#### `rts_server` (Dedicated Server)

- No rendering dependencies
- Async code with `tokio`
- Handle disconnections gracefully

## AI Assistant Guidelines

This section is for AI coding assistants (GitHub Copilot, Claude, etc.).

### Context to Always Consider

1. **This is a Rust + Bevy project** — use idiomatic Rust patterns
2. **Determinism is critical** — `rts_core` must be bit-perfect across platforms
3. **ECS architecture** — use components + systems, not OOP inheritance
4. **Fixed-point math** — use `Fixed` type (I32F32) in simulation code
5. **Workspace structure** — know which crate code belongs in

### Before Writing Code

- Read the relevant ADRs in `docs/architecture/decisions/`
- Check existing patterns in the codebase
- Verify the target crate (core vs game vs server)

### Code Generation Rules

```rust
// ✅ DO: Use fixed-point in rts_core
use crate::math::Fixed;
let speed: Fixed = Fixed::from_num(5);

// ❌ DON'T: Use floating point in rts_core
let speed: f32 = 5.0; // WRONG for simulation code

// ✅ DO: Use Result types
fn load_data(path: &str) -> Result<Data, GameError> { ... }

// ❌ DON'T: Panic on errors
fn load_data(path: &str) -> Data {
    std::fs::read(path).unwrap() // WRONG
}

// ✅ DO: Use tracing
tracing::info!(player_id = %id, "Player joined");

// ❌ DON'T: Use println
println!("Player {} joined", id); // WRONG

// ✅ DO: Document public APIs
/// Calculates damage after armor reduction.
///
/// # Arguments
/// * `base_damage` - Raw damage before armor
/// * `armor` - Target's armor value
pub fn calculate_damage(base_damage: u32, armor: u32) -> u32 { ... }
```

### Component Design

```rust
// ✅ Good: Pure data, derives required traits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}

// Methods for behavior are fine
impl Health {
    pub fn apply_damage(&mut self, amount: u32) -> u32 { ... }
}

// ❌ Bad: Logic in component, non-deterministic
pub struct Health {
    current: f32,  // Wrong: float
    last_damage_time: Instant, // Wrong: system time
}
```

### Testing Requirements for AI

When generating new features:

1. Add unit tests in the same file
2. For `rts_core`, add determinism test
3. Use descriptive test names

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_cannot_go_negative() {
        let mut health = Health::new(100);
        let damage = health.apply_damage(150);
        assert_eq!(health.current, 0);
        assert_eq!(damage, 100); // Only dealt 100 damage
    }
}
```

## Testing Requirements

### Test Categories

| Type | Location | Command |
| ---- | -------- | ------- |
| Unit tests | Same file as code | `cargo test -p rts_core` |
| Integration | `tests/` directory | `cargo test --test integration` |
| Determinism | `rts_test_utils` | Special harness |
| Benchmarks | `benches/` | `cargo bench` |

### Coverage Goals

| Crate | Target |
| ----- | ------ |
| `rts_core` | 80%+ (critical path) |
| `rts_game` | 50%+ (UI is hard to test) |
| `rts_server` | 70%+ |

### Determinism Testing

All simulation code must pass determinism tests:

```rust
use rts_test_utils::determinism::verify_determinism;

#[test]
fn simulation_is_deterministic() {
    let result = verify_determinism(
        5,    // Run 5 times
        1000, // 1000 ticks each
        setup_game,
        |sim| sim.step(),
        |sim| sim.state_hash(),
    );
    assert!(result.is_deterministic);
}
```

## Documentation

### Required Documentation

1. **Public APIs**: All `pub` items need `///` doc comments
2. **Modules**: Add `//!` module-level docs to each `mod.rs`/`lib.rs`
3. **ADRs**: Significant decisions need an ADR
4. **README**: Update if adding new features or changing setup

### Building Docs

```bash
# Build and open documentation
cargo doc --open

# Check for documentation warnings
cargo doc --no-deps 2>&1 | grep -i warning
```

## Questions?

- Check existing [ADRs](docs/architecture/decisions/) for design rationale
- Open an issue for questions or proposals
- Tag maintainers for architecture discussions

Thank you for contributing!
