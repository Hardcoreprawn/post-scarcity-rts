# ADR-002: Rust + Bevy

## Status

Accepted

## Context

Choosing a programming language and game engine/framework is one of the most consequential decisions for a project. Key requirements:

1. **Deterministic simulation** for lockstep multiplayer
2. **Performance** for 1000+ units at 60fps
3. **Safety** to reduce bugs in complex simulation
4. **Cross-platform** support (Windows, Linux, macOS)
5. **AI-assisted development** compatibility

Options considered:

| Option | Pros | Cons |
| ------ | ---- | ---- |
| C++ + Custom | Maximum control, industry standard | Memory safety burden, slow iteration |
| C# + Unity | Rapid iteration, large ecosystem | GC pauses, licensing, less control |
| Godot 4 | Open source, built-in multiplayer | No native ECS, smaller ecosystem |
| Rust + Bevy | Safety, performance, modern ECS | Learning curve, smaller talent pool |

## Decision

We will use **Rust** as the programming language and **Bevy** as the game framework.

### Rust

- **Memory safety without GC**: No garbage collection pauses affecting frame times
- **Ownership model**: Prevents data races, crucial for deterministic simulation
- **Zero-cost abstractions**: High-level code compiles to efficient assembly
- **Excellent tooling**: cargo, rust-analyzer, clippy, rustfmt
- **AI-friendly**: Strict compiler catches errors early, reducing AI hallucination impact

### Bevy

- **Native ECS**: First-class ECS architecture, not bolted on
- **Modern design**: Learned from prior engines' mistakes
- **Active community**: Rapid development, growing ecosystem
- **Data-driven**: Assets and configuration load from files
- **No editor lock-in**: Code-first approach, optional editors

## Consequences

### Positive

- **Reliability**: Compiler catches memory bugs, race conditions, and type errors
- **Performance**: No GC, predictable frame times, cache-friendly ECS
- **Determinism**: No runtime surprises from GC or dynamic dispatch
- **Maintainability**: Ownership model forces clean architecture
- **AI development**: Strict compiler validates AI-generated code immediately

### Negative

- **Learning curve**: Borrow checker frustrates newcomers
- **Compile times**: Rust compilation is slower than C++ for large projects
- **Ecosystem maturity**: Some libraries still in development
- **Bevy instability**: Pre-1.0, breaking changes between versions

### Mitigations

- Document Rust patterns in coding standards
- Use workspace structure for parallel compilation
- Pin dependency versions
- Abstract Bevy-specific code behind interfaces
- Start with conservative Bevy features, add as needed

## Related

- [ADR-001: ECS Architecture](./ADR-001-ecs-architecture.md)
- [Tech Stack](../tech-stack.md)
