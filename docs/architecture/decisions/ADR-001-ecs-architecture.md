# ADR-001: Entity Component System Architecture

## Status

Accepted

## Context

Real-time strategy games must efficiently handle thousands of entities (units, buildings, projectiles) with different combinations of behaviors. Traditional object-oriented inheritance hierarchies become unwieldy:

- A "flying unit that can harvest" requires multiple inheritance or awkward composition
- Adding new behaviors requires modifying class hierarchies
- Cache performance suffers when objects are scattered in memory
- Serialization and networking become complex

We need an architecture that:

1. Handles 1000+ entities at 60fps
2. Allows flexible combination of behaviors
3. Supports deterministic simulation for multiplayer
4. Enables data-driven design for modding

## Decision

We will use the **Entity Component System (ECS)** architectural pattern:

- **Entities** are unique identifiers (just IDs, no data)
- **Components** are pure data structs (Position, Health, Movement)
- **Systems** are functions that process entities with specific component combinations

Specifically, we will use **Bevy's ECS** for the game client and a custom lightweight ECS for the deterministic core (to avoid Bevy dependencies in `rts_core`).

### Example

```rust
// Components (pure data)
struct Position { x: Fixed, y: Fixed }
struct Health { current: u32, max: u32 }
struct Movement { speed: Fixed, target: Option<Vec2> }

// System (logic)
fn movement_system(query: Query<(&mut Position, &Movement)>) {
    for (mut pos, movement) in query.iter_mut() {
        if let Some(target) = movement.target {
            // Move toward target
        }
    }
}
```

## Consequences

### Positive

- **Performance**: Systems process contiguous arrays of components (cache-friendly)
- **Flexibility**: New entity types are just new combinations of existing components
- **Testability**: Systems are pure functions, easy to unit test
- **Parallelism**: Independent systems can run in parallel
- **Determinism**: Predictable system ordering enables lockstep multiplayer
- **Modding**: New components and systems can be added without modifying core

### Negative

- **Learning curve**: ECS is unfamiliar to developers from OOP backgrounds
- **Debugging**: Entity relationships are implicit, harder to inspect
- **Boilerplate**: Simple behaviors require component + system definitions
- **Bevy churn**: Bevy's ECS API changes between versions

### Mitigations

- Document common patterns in coding standards
- Use Bevy's built-in inspector plugins for debugging
- Create helper macros for common patterns
- Pin Bevy version and batch upgrades

## Related

- [ADR-002: Rust + Bevy](./ADR-002-rust-bevy.md)
- [Architecture Overview](../overview.md)
