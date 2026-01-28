# Architecture Overview

## System Architecture

The game is built on a component-based architecture with clear separation between engine systems and game logic.

```text
┌─────────────────────────────────────────────────────────────────┐
│                        Game Application                          │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │   Factions  │  │  Campaign   │  │ Multiplayer │   Game Layer │
│  └─────────────┘  └─────────────┘  └─────────────┘              │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐   │
│  │  Units  │ │Buildings│ │   AI    │ │ Combat  │ │ Economy │   │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘   │
│                                                   Systems Layer │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐   │
│  │ Render  │ │  Input  │ │  Audio  │ │ Physics │ │   Net   │   │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘   │
│                                                    Core Layer   │
└─────────────────────────────────────────────────────────────────┘
```

## Core Systems

### 1. Entity Component System (ECS)

All game objects are entities composed of data components processed by systems:

- **Entities**: Unique IDs (units, buildings, projectiles)
- **Components**: Pure data (Position, Health, Faction, Movement)
- **Systems**: Logic processors (MovementSystem, CombatSystem, RenderSystem)

### 2. Game Loop

```text
┌──────────────────────────────────────────────────────┐
│                     Game Loop                         │
│  ┌────────┐  ┌────────┐  ┌────────┐  ┌────────┐     │
│  │ Input  │→ │ Update │→ │ Render │→ │  Sync  │     │
│  │ Poll   │  │ Logic  │  │ Frame  │  │ Network│     │
│  └────────┘  └────────┘  └────────┘  └────────┘     │
│      ↑                                    │          │
│      └────────────────────────────────────┘          │
└──────────────────────────────────────────────────────┘
```

- **Fixed timestep** for simulation (e.g., 20 ticks/second)
- **Variable timestep** for rendering (uncapped or vsync)
- **Deterministic simulation** for multiplayer lockstep

### 3. Networking Architecture

Lockstep deterministic simulation for RTS multiplayer:

```text
┌─────────────┐         ┌─────────────┐
│   Client A  │◄───────►│   Client B  │
│  Simulation │         │  Simulation │
└──────┬──────┘         └──────┬──────┘
       │                       │
       ▼                       ▼
   Commands              Commands
       │                       │
       └───────►┌───┐◄─────────┘
                │   │
            Turn Buffer
```

- Commands are exchanged, not state
- All clients run identical simulation
- Hash verification for desync detection

### 4. AI Architecture

```text
┌─────────────────────────────────────┐
│           AI Director               │
├─────────────────────────────────────┤
│  ┌───────────┐  ┌───────────────┐  │
│  │ Strategic │  │   Tactical    │  │
│  │   Layer   │  │    Layer      │  │
│  │           │  │               │  │
│  │ • Expand  │  │ • Unit micro  │  │
│  │ • Tech    │  │ • Formation   │  │
│  │ • Attack  │  │ • Target      │  │
│  └───────────┘  └───────────────┘  │
├─────────────────────────────────────┤
│         Faction Personality         │
│  (aggression, expansion, tech bias) │
└─────────────────────────────────────┘
```

## Module Breakdown (Workspace Crates)

### `crates/rts_core/`

| Module | Responsibility |
| ------ | -------------- |
| `simulation.rs` | Deterministic simulation loop and tick orchestration |
| `systems.rs` | Simulation systems (movement, combat, production) |
| `components.rs` | Core simulation components and commands |
| `math.rs` | Fixed-point math types and helpers |
| `pathfinding.rs` | Deterministic A* pathfinding |
| `data/` | Data definitions for factions, units, tech (no IO) |

### `crates/rts_game/`

| Module | Responsibility |
| ------ | -------------- |
| `simulation.rs` | Client-side command processing and visual movement plumbing |
| `input.rs` | Input handling and command mapping |
| `render.rs` | Rendering systems and visuals |
| `ui.rs` | HUD, menus, and UI interaction |
| `selection.rs` | Unit selection and group management |
| `data_loader.rs` | Loading FactionData from RON and registry wiring |
| `ai.rs` | Game-layer AI helpers and adapters |

### `crates/rts_server/`

| Module | Responsibility |
| ------ | -------------- |
| `network.rs` | Networking primitives and message routing |
| `lobby.rs` | Game lobby and matchmaking |
| `lib.rs` | Server integration with deterministic core |

### `crates/rts_tools/`

| Module | Responsibility |
| ------ | -------------- |
| `validate.rs` | Data validation and linting |
| `main.rs` | Tool entry point |

### `crates/rts_test_utils/`

| Module | Responsibility |
| ------ | -------------- |
| `determinism.rs` | Determinism helpers and hash utilities |
| `fixtures.rs` | Test fixtures for sim scenarios |
| `tactical.rs` | Unit-level tactical decisions |
| `personalities/` | Faction-specific AI behaviors |

## Data Flow

```text
Input Events
     │
     ▼
┌─────────────┐
│   Commands  │ ──► Network (if multiplayer)
└─────────────┘
     │
     ▼
┌─────────────┐
│ Turn Buffer │
└─────────────┘
     │
     ▼
┌─────────────┐
│ Simulation  │ ──► State Changes
└─────────────┘
     │
     ▼
┌─────────────┐
│  Renderer   │ ──► Frame Output
└─────────────┘
```

## Key Design Principles

1. **Determinism First**: All game logic must be deterministic for multiplayer
2. **Data-Driven**: Unit stats, tech trees, and balance in external data files
3. **Moddability**: Clean interfaces for faction/unit modding
4. **Separation of Concerns**: Clear boundaries between systems
5. **Performance Budget**: 1000+ units at 60fps target

## Technology Stack

**Decided: Rust + Bevy** — see [Tech Stack Decision](./tech-stack.md) for full analysis.

## Related Documents

- [Tech Stack Decision](./tech-stack.md)
- [Architecture Decision Records](./decisions/)
- [Coding Standards](../standards/coding-standards.md)
- [Networking Protocol](./networking.md)
- [AI Architecture](./ai-architecture.md)
