# Coding Standards

## Overview

This document defines coding standards, conventions, and best practices for the project. Consistency improves readability, reduces bugs, and makes collaboration easier.

## General Principles

### 1. Clarity Over Cleverness

```javascript
// Bad: Clever but unclear
let x = a > b ? (c < d ? e : f) : (g > h ? i : j);

// Good: Clear and readable
let result;
if (a > b) {
    result = c < d ? e : f;
} else {
    result = g > h ? i : j;
}
```

### 2. Single Responsibility

Each function, struct, or module should do one thing well.

### 3. Fail Fast

Validate inputs early. Return errors immediately rather than propagating bad state.

### 4. Document Intent, Not Implementation

Comments explain *why*, not *what*.

```javascript
// Bad: What is obvious from code
// Increment counter by 1
counter += 1;

// Good: Why we're doing this
// Reset selection when switching factions to avoid cross-faction unit references
selection.clear();
```

## Naming Conventions

### Files and Directories

| Type | Convention | Example |
| ---- | ---------- | ------- |
| Source files | snake_case | `unit_controller.rs` |
| Test files | snake_case with `_test` suffix | `unit_controller_test.rs` |
| Directories | snake_case | `bio_sovereigns/` |
| Config files | kebab-case | `game-config.ron` |

### Code Elements

| Type | Convention | Example |
| ---- | ---------- | ------- |
| Types/Structs | PascalCase | `HarvesterDrone` |
| Functions | snake_case | `calculate_damage()` |
| Variables | snake_case | `unit_count` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_UNIT_CAP` |
| Enums | PascalCase | `FactionType::Continuity` |
| Components (ECS) | PascalCase, noun | `Health`, `Position` |
| Systems (ECS) | PascalCase, verb phrase | `MovementSystem` |

### Prefixes and Suffixes

| Suffix | Meaning | Example |
| ------ | ------- | ------- |
| `Component` | ECS component (optional) | `HealthComponent` |
| `System` | ECS system | `CombatSystem` |
| `Event` | Event type | `UnitDestroyedEvent` |
| `Resource` | Shared resource | `GameTimeResource` |
| `Config` | Configuration struct | `FactionConfig` |

## Code Organization

### File Structure (Rust Example)

```rust
// 1. Module documentation
//! This module handles unit combat calculations.

// 2. Imports (grouped and sorted)
use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::{Health, Position};
use crate::factions::FactionId;

// 3. Constants
const BASE_DAMAGE_MULTIPLIER: f32 = 1.0;

// 4. Type definitions
#[derive(Component, Debug, Clone)]
pub struct CombatStats {
    pub attack: u32,
    pub defense: u32,
    pub range: f32,
}

// 5. Implementations
impl CombatStats {
    pub fn new(attack: u32, defense: u32, range: f32) -> Self {
        Self { attack, defense, range }
    }
}

// 6. Systems/Functions
pub fn combat_system(
    mut query: Query<(&CombatStats, &mut Health)>,
) {
    // Implementation
}

// 7. Tests (in same file or separate)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_calculation() {
        // Test implementation
    }
}
```

### Module Organization

```text
src/
├── lib.rs              # Library root, public API
├── main.rs             # Binary entry point
├── components/         # ECS components
│   ├── mod.rs
│   ├── combat.rs
│   └── movement.rs
├── systems/            # ECS systems
│   ├── mod.rs
│   ├── combat.rs
│   └── movement.rs
└── factions/           # Faction implementations
    ├── mod.rs
    ├── faction.rs      # Base trait
    └── continuity/
        ├── mod.rs
        └── units.rs
```

## Error Handling

### Use Result Types

```rust
// Bad: Panic on error
fn load_config(path: &str) -> Config {
    let file = File::open(path).unwrap(); // Panics!
    // ...
}

// Good: Return Result
fn load_config(path: &str) -> Result<Config, ConfigError> {
    let file = File::open(path)?;
    // ...
}
```

### Custom Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum GameError {
    #[error("Failed to load faction data: {0}")]
    FactionLoadError(String),
    
    #[error("Invalid unit ID: {0}")]
    InvalidUnitId(u32),
    
    #[error("Network error: {0}")]
    NetworkError(#[from] std::io::Error),
}
```

## Documentation

### Public API Documentation

All public functions, structs, and modules must have documentation.

```rust
/// Calculates damage dealt from attacker to defender.
///
/// Takes into account armor type, damage type, and any active buffs.
///
/// # Arguments
/// * `attacker` - The attacking unit's combat stats
/// * `defender` - The defending unit's combat stats
/// * `buffs` - Active buff modifiers
///
/// # Returns
/// The final damage value after all calculations
///
/// # Example
/// ```
/// let damage = calculate_damage(&attacker, &defender, &buffs);
/// defender.health -= damage;
/// ```
pub fn calculate_damage(
    attacker: &CombatStats,
    defender: &CombatStats,
    buffs: &BuffList,
) -> u32 {
    // Implementation
}
```

### Architecture Decision Records (ADRs)

Major decisions should be documented in `/docs/architecture/decisions/`:

```markdown
# ADR-001: Use ECS Architecture

## Status
Accepted

## Context
We need an architecture that supports 1000+ units efficiently.

## Decision
Use Entity Component System (ECS) pattern.

## Consequences
- Better cache locality for batch processing
- Steeper learning curve for new developers
- Natural fit for data-driven design
```

## Testing

### Test Naming

```rust
#[test]
fn unit_takes_damage_when_attacked() { }

#[test]
fn harvester_returns_to_base_when_full() { }

#[test]
fn faction_tech_tree_unlocks_correctly() { }
```

### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // Group related tests
    mod damage_calculation {
        use super::*;
        
        #[test]
        fn basic_damage_applies_correctly() { }
        
        #[test]
        fn armor_reduces_damage() { }
    }
    
    mod movement {
        use super::*;
        
        #[test]
        fn unit_moves_to_target() { }
    }
}
```

### Test Coverage Goals

| Category | Target |
| -------- | ------ |
| Core systems | 80%+ |
| Game logic | 70%+ |
| UI code | 50%+ |
| Integration | Key paths covered |

## Performance Guidelines

### Allocation

- Avoid allocations in hot paths (game loop)
- Pre-allocate collections where size is known
- Use object pools for frequently created/destroyed objects

### Data Layout

- Keep frequently-accessed data together (cache-friendly)
- Use Structure of Arrays (SoA) for batch processing
- Profile before optimizing

### Determinism

For multiplayer lockstep:

- No floating-point non-determinism (use fixed-point if needed)
- No random without synchronized seeds
- No system time in simulation

## Version Control

### Commit Messages

```text
type(scope): brief description

Longer explanation if needed.

Fixes #123
```

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `perf`, `chore`

Examples:

```text
feat(factions): add Bio-Sovereign unit roster
fix(combat): correct damage calculation for armor types
docs(architecture): update networking diagram
refactor(ecs): extract component registration
```

### Branch Naming

```text
feature/faction-tech-trees
bugfix/unit-pathfinding
docs/api-documentation
```

## Code Review Checklist

- [ ] Code compiles without warnings
- [ ] Tests pass
- [ ] New code has tests
- [ ] Public API is documented
- [ ] No hardcoded magic numbers
- [ ] Error cases handled
- [ ] Performance impact considered
- [ ] Determinism maintained (for simulation code)

## Related Documents

- [Architecture Overview](../architecture/overview.md)
- [Tech Stack](../architecture/tech-stack.md)
- [ADR-003: Fixed-Point Math](../architecture/decisions/ADR-003-fixed-point-math.md)
- [ADR-005: Error Handling](../architecture/decisions/ADR-005-error-handling.md)
- [ADR-006: Tracing and Logging](../architecture/decisions/ADR-006-tracing.md)
- [Contributing Guide](../../CONTRIBUTING.md)
- [Testing Guide](./testing-guide.md)
