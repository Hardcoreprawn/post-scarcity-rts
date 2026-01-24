# Post-Scarcity RTS

A real-time strategy game exploring the tension between control and freedom in a world where scarcity is ending but power structures remain.

## Vision

Five factions clash over the fundamental question: *"What do we do in a world where scarcity is ending, but power structures remain?"*

- **The Continuity Authority** — Control it
- **The Collegium** — Share it
- **The Tinkers' Union** — Build it
- **The Sculptors** — Transcend the body itself
- **The Zephyr Guild** — Exploit it

## Tech Stack

- **Language:** Rust 1.75+
- **Engine:** Bevy 0.14 (ECS game engine)
- **Why Rust:** Memory safety, deterministic simulation, zero-cost abstractions, AI-friendly development

See [Tech Stack Decision](docs/architecture/tech-stack.md) for full rationale.

## Project Structure

```text
rts-game/
├── crates/                  # Rust workspace crates
│   ├── rts_core/            # Deterministic simulation (no Bevy)
│   ├── rts_game/            # Bevy game client
│   ├── rts_server/          # Headless dedicated server
│   ├── rts_tools/           # Development utilities
│   └── rts_test_utils/      # Shared test helpers
├── docs/                    # All documentation
│   ├── architecture/        # Technical architecture docs
│   │   └── decisions/       # Architecture Decision Records
│   ├── design/              # Game design documents
│   └── standards/           # Coding standards and guidelines
├── assets/                  # Game assets
│   ├── data/                # RON data files
│   ├── audio/               # Sound effects and music
│   └── ui/                  # UI assets
└── config/                  # Configuration files
```

## Getting Started

### Prerequisites

- Rust 1.75+ ([rustup.rs](https://rustup.rs/))
- pre-commit (`pip install pre-commit`)

### Quick Start

```bash
# Clone and enter directory
git clone https://github.com/your-org/rts-game.git
cd rts-game

# Install pre-commit hooks
pre-commit install

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run the game
cargo run -p rts_game --features dev-tools
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for full setup instructions.

## Documentation

- [Development Roadmap](docs/ROADMAP.md) — Phased milestones and timeline
- [Contributing Guide](CONTRIBUTING.md) — How to contribute (humans & AI)
- [Architecture Overview](docs/architecture/overview.md)
- [Architecture Decisions](docs/architecture/decisions/) — ADRs explaining key choices
- [Tech Stack](docs/architecture/tech-stack.md)
- [Coding Standards](docs/standards/coding-standards.md)
- [Game Design Document](docs/design/gdd.md)
- [Faction Design](docs/design/factions/)

## CI/CD

| Badge | Status |
| ----- | ------ |
| Build | ![CI](https://github.com/your-org/rts-game/workflows/CI/badge.svg) |
| Audit | ![Security](https://github.com/your-org/rts-game/workflows/Security/badge.svg) |

## License

TBD
