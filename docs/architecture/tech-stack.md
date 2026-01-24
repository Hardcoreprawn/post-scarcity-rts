# Tech Stack Decision

## Status: Decided — Rust + Bevy

Rust is our language of choice: reliable, guarded, and safe. Its memory safety guarantees and zero-cost abstractions make it ideal for deterministic game simulation and AI-assisted development.

This document outlines the technology choices for the game engine and development stack.

## Requirements

### Hard Requirements

- **Deterministic simulation** for lockstep multiplayer
- **Handle 1000+ units** at 60fps
- **Cross-platform** (Windows primary, Linux/Mac secondary)
- **Modding support** (data-driven design)
- **Reasonable iteration speed** during development

### Nice to Have

- Active community and ecosystem
- Good tooling (debuggers, profilers)
- Asset pipeline support
- Built-in networking primitives

## Options Analysis

### Option 1: Rust + Bevy

| Aspect | Assessment |
| ------ | ---------- |
| Performance | Excellent — zero-cost abstractions, no GC |
| Safety | Excellent — memory safety guarantees |
| ECS | Built-in, well-designed |
| Learning curve | Steep — borrow checker, new paradigms |
| Ecosystem | Growing — crates for most needs |
| Multiplayer | Manual — need to implement lockstep |
| Modding | Good — WASM plugins possible |

**Pros:**

- Modern language with excellent performance
- Built-in ECS architecture
- Growing game dev community
- Memory safety prevents entire classes of bugs

**Cons:**

- Steeper learning curve
- Smaller talent pool
- Some libraries still maturing

### Option 2: C++ + Custom Engine

| Aspect | Assessment |
| ------ | ---------- |
| Performance | Excellent — full control |
| Safety | Manual — discipline required |
| ECS | Build or integrate (EnTT) |
| Learning curve | Moderate — familiar paradigms |
| Ecosystem | Mature — solutions for everything |
| Multiplayer | Manual — full control |
| Modding | Good — Lua/scripting integration |

**Pros:**

- Industry standard, proven
- Maximum control and performance
- Huge ecosystem and libraries
- Large talent pool

**Cons:**

- Memory management burden
- Slower iteration than managed languages
- Build system complexity

### Option 3: C# + Unity

| Aspect | Assessment |
| ------ | ---------- |
| Performance | Good — DOTS for ECS performance |
| Safety | Good — managed memory |
| ECS | DOTS available but learning curve |
| Learning curve | Low — familiar tooling |
| Ecosystem | Excellent — Asset Store |
| Multiplayer | Good — Netcode for GameObjects |
| Modding | Moderate — requires planning |

**Pros:**

- Rapid iteration
- Excellent tooling and editor
- Large ecosystem
- Easy prototyping

**Cons:**

- Runtime licensing considerations
- Less control over engine internals
- GC can cause hitches without care

### Option 4: Godot 4

| Aspect | Assessment |
| ------ | ---------- |
| Performance | Good — C++ core, GDScript/C# bindings |
| Safety | Good — managed or optional C++ |
| ECS | Not built-in — third-party or custom |
| Learning curve | Low — intuitive editor |
| Ecosystem | Growing — smaller than Unity |
| Multiplayer | Good — built-in high-level API |
| Modding | Excellent — open source, PCK system |

**Pros:**

- Fully open source (MIT)
- No licensing fees
- Built-in multiplayer support
- Active development

**Cons:**

- Smaller ecosystem
- No built-in ECS (node-based)
- Less proven for large-scale RTS

## Recommendation

### Primary Recommendation: Rust + Bevy

Rationale:

1. ECS architecture aligns with RTS needs (many entities, batch processing)
2. Determinism is easier to achieve without GC
3. Performance headroom for 1000+ units
4. Modern tooling (cargo, rust-analyzer)
5. Growing ecosystem specifically for games

### Fallback Considered

Godot 4 with C++ GDExtension was evaluated but Rust was chosen for its superior safety guarantees and determinism.

## Decision

### Confirmed: Rust + Bevy

Rust's ownership model and compile-time guarantees align perfectly with our needs:

- **Deterministic behavior** — No GC pauses means consistent frame times
- **Memory safety** — Eliminates entire classes of bugs at compile time
- **Performance** — Zero-cost abstractions, ideal for 1000+ unit simulation
- **AI-friendly** — Strict compiler catches errors early, perfect for AI-assisted development
- **ECS-native** — Bevy's ECS maps naturally to RTS architecture

### Core Dependencies

```toml
[dependencies]
bevy = "0.14"
bevy_egui = "0.28"          # UI
bevy_rapier2d = "0.27"       # Physics (if needed)
serde = "1.0"                # Serialization
ron = "0.8"                  # Data files
tokio = "1.0"                # Async networking
```

### Additional Dependencies (As Needed)

```toml
# Networking
tokio = "1.0"                # Async runtime
quinn = "0.11"               # QUIC protocol

# Data
serde = "1.0"                # Serialization
ron = "0.8"                  # Rusty Object Notation for data files

# Audio
kira = "0.8"                 # Game audio
```

## Related Documents

- [Architecture Overview](./overview.md)
- [ADR-002: Rust + Bevy Decision](./decisions/ADR-002-rust-bevy.md)
- [ADR-003: Fixed-Point Math](./decisions/ADR-003-fixed-point-math.md)
- [ADR-004: Workspace Structure](./decisions/ADR-004-workspace-structure.md)
- [Performance Budget](./performance.md)
