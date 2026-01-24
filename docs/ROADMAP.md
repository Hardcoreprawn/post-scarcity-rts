# Development Roadmap

## Overview

This roadmap outlines the development phases for Post-Scarcity RTS, with testable milestones at each stage. Each phase builds on the previous, ensuring we always have a working, demonstrable build.

### Guiding Principles

- **Always Shippable** — Every milestone produces a testable build
- **Open Assets First** — Use free/open-source assets until art pipeline is ready
- **Automate Everything** — CI/CD and hooks enforce quality from day one
- **Steam-Ready** — Architecture supports Steam integration from the start

---

## Phase 0: Foundation (Weeks 1-2)

Establish development infrastructure and project skeleton.

### 0.1 Repository Setup

- [x] Initialize Git repository
- [ ] Configure `.gitignore` for Rust/Bevy
- [ ] Set up branch protection rules (main, develop)
- [ ] Create PR template and issue templates

#### 0.2 Rust Project Initialization

- [ ] Initialize Cargo workspace
- [ ] Configure `Cargo.toml` with core dependencies
- [ ] Set up workspace structure (crates for core, game, editor)
- [ ] Verify `cargo build` and `cargo run` work

#### 0.3 Pre-commit Hooks

- [ ] Install `pre-commit` framework
- [ ] Configure hooks:
  - `cargo fmt --check` — Code formatting
  - `cargo clippy` — Linting
  - `cargo test` — Unit tests pass
  - `cargo doc` — Documentation builds
  - Commit message format validation
- [ ] Document hook setup in CONTRIBUTING.md

#### 0.4 CI/CD Pipeline

- [ ] GitHub Actions workflow for:
  - Build (Windows, Linux, macOS)
  - Test suite
  - Clippy lints
  - Documentation generation
  - Release builds (tagged commits)
- [ ] Badge status in README
- [ ] Artifact uploads for builds

**Exit Criteria:**

- [ ] `cargo build --release` succeeds on all platforms via CI
- [ ] Pre-commit hooks block badly formatted code
- [ ] Empty window opens with Bevy

---

## Phase 1: Core Engine (Weeks 3-6)

Build the fundamental game systems that all features depend on.

### 1.1 ECS Foundation

- [ ] Entity spawning/despawning
- [ ] Component registration system
- [ ] System scheduling and ordering
- [ ] Event system for game events
- [ ] **Test:** Spawn 1000 entities, verify frame time < 16ms

#### 1.2 Rendering Pipeline

- [ ] 2D sprite rendering (placeholder assets)
- [ ] Camera system (pan, zoom)
- [ ] Basic tilemap or terrain rendering
- [ ] Unit selection visuals (circles, highlights)
- [ ] **Test:** Render 500 sprites at 60fps

#### 1.3 Input System

- [ ] Mouse input (click, drag, box select)
- [ ] Keyboard shortcuts
- [ ] Camera controls (WASD, edge pan, mouse wheel)
- [ ] Input mapping/rebinding foundation
- [ ] **Test:** Select units, issue move commands

#### 1.4 Pathfinding

- [ ] Navigation mesh or grid generation
- [ ] A* or flowfield pathfinding
- [ ] Unit steering and avoidance
- [ ] Formation movement basics
- [ ] **Test:** 100 units navigate around obstacles without stacking

#### 1.5 Basic Simulation Loop

- [ ] Fixed timestep game loop
- [ ] Deterministic update ordering
- [ ] Game state serialization (for saves/netcode)
- [ ] **Test:** Record and replay 60 seconds of gameplay identically

**Exit Criteria:**

- [ ] Units move on command with pathfinding
- [ ] Simulation is deterministic (replay produces identical results)
- [ ] Stable 60fps with 500 units

---

## Phase 2: Gameplay Foundation (Weeks 7-12)

Implement core RTS mechanics with one faction.

### 2.1 Resource System

- [ ] Feedstock resource nodes
- [ ] Harvester units (gather, return, deposit)
- [ ] Resource storage and display
- [ ] Basic economy loop
- [ ] **Test:** Harvesters collect resources, player can spend them

#### 2.2 Unit Production

- [ ] Building placement system
- [ ] Production queues
- [ ] Build times and costs
- [ ] Rally points
- [ ] **Test:** Build structure, train units, units rally to point

#### 2.3 Combat System

- [ ] Health and damage
- [ ] Attack commands and auto-attack
- [ ] Damage types and armor
- [ ] Unit death and cleanup
- [ ] **Test:** Two armies fight, units die, combat resolves

#### 2.4 Building System

- [ ] Building placement grid
- [ ] Construction time
- [ ] Building health and destruction
- [ ] Tech requirements
- [ ] **Test:** Build tech tree progression works

#### 2.5 Basic UI

- [ ] HUD (resources, supply, minimap placeholder)
- [ ] Unit selection panel
- [ ] Command card (actions/abilities)
- [ ] Production panel
- [ ] **Test:** All core actions accessible via UI

**Exit Criteria:**

- [ ] Complete gameplay loop: gather → build → train → fight
- [ ] One faction playable (Continuity Authority as baseline)
- [ ] Win/lose conditions (destroy enemy base)

---

## Phase 3: Content Expansion (Weeks 13-20)

Implement all five factions with unique mechanics.

### 3.1 Faction Framework

- [ ] Faction data loading (RON files)
- [ ] Faction-specific unit stats
- [ ] Faction-specific buildings
- [ ] Tech tree per faction
- [ ] **Test:** Switch factions, all data loads correctly

#### 3.2 Continuity Authority (Complete)

- [ ] All units implemented
- [ ] Stockpile mechanic
- [ ] Heavy mech feel
- [ ] **Test:** Playable vs AI

#### 3.3 The Collegium

- [ ] Drone swarm mechanics
- [ ] Network scaling bonus
- [ ] Open-source discount
- [ ] **Test:** Playable vs AI

#### 3.4 The Tinkers' Union

- [ ] Modular mech system
- [ ] Module swapping
- [ ] Salvage mechanic
- [ ] Mobile buildings
- [ ] **Test:** Playable vs AI

#### 3.5 The Sculptors

- [ ] Patronage economy system
- [ ] Bespoke unit production
- [ ] Essence harvesting
- [ ] Customization/adaptation system
- [ ] **Test:** Playable vs AI

#### 3.6 The Zephyr Guild

- [ ] Flying units primary
- [ ] Trade route system
- [ ] Piracy mechanics
- [ ] Mobile base
- [ ] **Test:** Playable vs AI

**Exit Criteria:**

- [ ] All five factions playable
- [ ] Each faction feels distinct
- [ ] Balance pass complete (internal testing)

---

## Phase 4: AI Opponents (Weeks 21-26)

Create competent AI opponents for single-player.

### 4.1 AI Framework

- [ ] Behavior tree or utility AI system
- [ ] AI decision timing (not per-frame)
- [ ] Difficulty scaling hooks
- [ ] **Test:** AI makes decisions, doesn't freeze game

#### 4.2 Economic AI

- [ ] Resource gathering management
- [ ] Build order execution
- [ ] Expansion timing
- [ ] **Test:** AI sustains economy for 15 minutes

#### 4.3 Military AI

- [ ] Army composition decisions
- [ ] Attack timing
- [ ] Target prioritization
- [ ] Retreat logic
- [ ] **Test:** AI attacks player base appropriately

#### 4.4 Faction-Specific AI

- [ ] Each faction uses unique mechanics
- [ ] AI exploits faction strengths
- [ ] **Test:** Each faction AI behaves distinctly

#### 4.5 Difficulty Levels

- [ ] Easy (slower, makes mistakes)
- [ ] Normal (fair play)
- [ ] Hard (optimal builds, multi-tasking)
- [ ] **Test:** Each difficulty feels appropriate

**Exit Criteria:**

- [ ] AI provides challenge at all difficulty levels
- [ ] AI uses faction mechanics correctly
- [ ] No obvious exploits or cheese strats against AI

---

## Phase 5: Multiplayer (Weeks 27-34)

Implement deterministic lockstep multiplayer.

### 5.1 Networking Foundation

- [ ] QUIC-based networking (quinn crate)
- [ ] Lobby system
- [ ] Player connection/disconnection handling
- [ ] **Test:** Two clients connect to server

#### 5.2 Lockstep Simulation

- [ ] Command serialization
- [ ] Input delay and buffering
- [ ] Synchronization checks (checksums)
- [ ] Desync detection and handling
- [ ] **Test:** Two players play for 10 minutes without desync

#### 5.3 Steam Integration

- [ ] Steamworks SDK integration
- [ ] Steam authentication
- [ ] Steam matchmaking
- [ ] Steam lobbies
- [ ] **Test:** Find and join game via Steam

#### 5.4 Replay System

- [ ] Record all inputs
- [ ] Playback at variable speed
- [ ] Save/load replays
- [ ] **Test:** Watch replay of completed game

#### 5.5 Spectator Mode

- [ ] Observer slots
- [ ] Fog of war toggle for observers
- [ ] Delayed spectating (anti-cheat)
- [ ] **Test:** Third player spectates match

**Exit Criteria:**

- [ ] Stable 1v1 matches via Steam
- [ ] Replays work correctly
- [ ] No desyncs in 95% of games

---

## Phase 6: Polish & Content (Weeks 35-44)

Replace placeholder assets and polish the experience.

### 6.1 Asset Pipeline

- [ ] Asset loading system (hot reload in dev)
- [ ] Sprite atlas generation
- [ ] Audio system integration
- [ ] **Test:** Replace placeholder, verify loads correctly

#### 6.2 Visual Polish

- [ ] Unit sprites/models (open assets or commissioned)
- [ ] Building sprites/models
- [ ] Terrain tiles
- [ ] Visual effects (attacks, explosions, abilities)
- [ ] UI art pass

#### 6.3 Audio

- [ ] Sound effects (units, combat, UI)
- [ ] Music (menu, battle, faction themes)
- [ ] Ambient audio
- [ ] Voice lines (if budget allows)

#### 6.4 Maps

- [ ] Map editor or generation tool
- [ ] 5+ 1v1 maps
- [ ] 3+ 2v2 maps
- [ ] Campaign maps (if applicable)

#### 6.5 Campaign/Tutorials

- [ ] Tutorial mission (teaches basics)
- [ ] Faction introductions
- [ ] Campaign missions (scope TBD)

**Exit Criteria:**

- [ ] No placeholder assets in release build
- [ ] Audio complete
- [ ] Sufficient maps for launch

---

## Phase 7: Release Preparation (Weeks 45-52)

Prepare for Steam release.

### 7.1 Steam Store

- [ ] Store page assets (capsules, screenshots, trailer)
- [ ] Store description and tags
- [ ] Coming Soon page live
- [ ] Wishlist campaign

#### 7.2 Steamworks Features

- [ ] Achievements
- [ ] Cloud saves
- [ ] Steam Deck verification
- [ ] Workshop support (maps/mods) — stretch goal

#### 7.3 QA & Testing

- [ ] Closed beta testing
- [ ] Bug triage and fixes
- [ ] Performance optimization pass
- [ ] Memory leak testing
- [ ] Compatibility testing

#### 7.4 Localization

- [ ] String externalization
- [ ] English complete
- [ ] Additional languages (scope TBD)

#### 7.5 Launch

- [ ] Release build
- [ ] Day-one patch plan
- [ ] Community channels (Discord, forums)
- [ ] Press outreach

**Exit Criteria:**

- [ ] Steam review approved
- [ ] No critical bugs
- [ ] Launch!

---

## Asset Strategy

### Open/Free Assets (Development Phase)

| Category | Source | License |
| -------- | ------ | ------- |
| Sprites | Kenney.nl | CC0 |
| Icons | Game-icons.net | CC BY 3.0 |
| UI | Kenney UI Pack | CC0 |
| Audio SFX | Freesound.org | Various (check each) |
| Music | Kevin MacLeod | CC BY 3.0 |
| Fonts | Google Fonts | OFL |

### Asset Replacement Plan

1. Use open assets for all prototyping
2. Commission/create final assets in Phase 6
3. Maintain asset manifest tracking placeholder vs final
4. Ensure all licenses are compatible with commercial release

---

## CI/CD Pipeline Details

### Pre-commit Hooks

```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: cargo-fmt
        name: cargo fmt
        entry: cargo fmt --all --check
        language: system
        pass_filenames: false
        
      - id: cargo-clippy
        name: cargo clippy
        entry: cargo clippy --all-targets -- -D warnings
        language: system
        pass_filenames: false
        
      - id: cargo-test
        name: cargo test
        entry: cargo test --workspace
        language: system
        pass_filenames: false
        
      - id: cargo-doc
        name: cargo doc
        entry: cargo doc --no-deps
        language: system
        pass_filenames: false
```

### GitHub Actions

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --all --check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo test --workspace
      - run: cargo doc --no-deps

  build:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release
      - uses: actions/upload-artifact@v4
        with:
          name: build-${{ matrix.os }}
          path: target/release/rts-game*
```

### Release Workflow

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags: ['v*']

jobs:
  release:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release
      - name: Upload to Steam
        # Steam CLI upload step
        run: echo "Steam upload would happen here"
```

---

## Risk Mitigation

| Risk | Mitigation |
| ---- | ---------- |
| Scope creep | Strict milestone definitions, cut features not phases |
| Multiplayer desyncs | Determinism tests from Phase 1, checksums always on |
| Performance issues | Benchmark tests at each phase, profile early |
| Asset licensing | Track all assets in manifest, verify licenses before use |
| Steam rejection | Follow Steamworks guidelines, test on Steam Deck early |
| Team burnout | Realistic timelines, buffer weeks between phases |

---

## Timeline Summary

| Phase | Duration | Cumulative |
| ----- | -------- | ---------- |
| 0: Foundation | 2 weeks | Week 2 |
| 1: Core Engine | 4 weeks | Week 6 |
| 2: Gameplay | 6 weeks | Week 12 |
| 3: Content | 8 weeks | Week 20 |
| 4: AI | 6 weeks | Week 26 |
| 5: Multiplayer | 8 weeks | Week 34 |
| 6: Polish | 10 weeks | Week 44 |
| 7: Release | 8 weeks | Week 52 |

Total: ~52 weeks (1 year)

---

## Related Documents

- [Architecture Overview](architecture/overview.md)
- [Tech Stack](architecture/tech-stack.md)
- [Coding Standards](standards/coding-standards.md)
- [Game Design Document](design/gdd.md)
