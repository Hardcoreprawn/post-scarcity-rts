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

## Module Breakdown

### `/src/core/`

| Module | Responsibility |
| ------ | -------------- |
| `engine.rs` | Main game loop, timing, state management |
| `ecs.rs` | Entity Component System implementation |
| `renderer.rs` | Rendering abstraction layer |
| `input.rs` | Input handling and command mapping |
| `audio.rs` | Audio engine integration |
| `physics.rs` | Collision detection, pathfinding |

### `/src/factions/`

| Module | Responsibility |
| ------ | -------------- |
| `faction.rs` | Base faction trait and shared logic |
| `continuity/` | Continuity Authority-specific units, buildings, tech |
| `collegium/` | Collegium-specific implementations |
| `tinkers/` | Tinkers' Union implementations |
| `biosovereigns/` | Bio-Sovereigns implementations |
| `zephyr/` | Zephyr Guild implementations |

### `/src/ui/`

| Module | Responsibility |
| ------ | -------------- |
| `hud.rs` | In-game HUD elements |
| `menus.rs` | Menu screens |
| `minimap.rs` | Minimap rendering and interaction |
| `selection.rs` | Unit selection box and group management |

### `/src/networking/`

| Module | Responsibility |
| ------ | -------------- |
| `lobby.rs` | Game lobby and matchmaking |
| `lockstep.rs` | Lockstep synchronization |
| `commands.rs` | Command serialization |

### `/src/ai/`

| Module | Responsibility |
| ------ | -------------- |
| `director.rs` | High-level AI decision making |
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
