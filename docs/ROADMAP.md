# Development Roadmap

## Overview

This roadmap outlines the development phases for Post-Scarcity RTS, with testable milestones at each stage. Each phase builds on the previous, ensuring we always have a working, demonstrable build.

### Guiding Principles

- **Always Shippable** — Every milestone produces a testable build
- **Open Assets First** — Use free/open-source assets until art pipeline is ready
- **Automate Everything** — CI/CD and hooks enforce quality from day one
- **Steam-Ready** — Architecture supports Steam integration from the start

---

## Phase 0: Foundation (Weeks 1-2) ✅

Establish development infrastructure and project skeleton.

### 0.1 Repository Setup ✅

- [x] Initialize Git repository
- [x] Configure `.gitignore` for Rust/Bevy
- [x] Set up branch protection rules (main, develop)
- [x] Create PR template and issue templates

#### 0.2 Rust Project Initialization ✅

- [x] Initialize Cargo workspace
- [x] Configure `Cargo.toml` with core dependencies
- [x] Set up workspace structure (crates for core, game, editor)
- [x] Verify `cargo build` and `cargo run` work

#### 0.3 Pre-commit Hooks

- [ ] Install `pre-commit` framework
- [ ] Configure hooks:
  - `cargo fmt --check` — Code formatting
  - `cargo clippy` — Linting
  - `cargo test` — Unit tests pass
  - `cargo doc` — Documentation builds
  - Commit message format validation
- [ ] Document hook setup in CONTRIBUTING.md

#### 0.4 CI/CD Pipeline ✅ ~~[#10](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/10)~~

- [x] GitHub Actions workflow for:
  - Build (Windows, Linux, macOS)
  - Test suite
  - Clippy lints
  - Documentation generation
  - Release builds (tagged commits)
- [ ] Badge status in README
- [x] Artifact uploads for builds

**Exit Criteria:**

- [x] `cargo build --release` succeeds on all platforms via CI
- [ ] Pre-commit hooks block badly formatted code
- [x] Empty window opens with Bevy

---

## Phase 1: Core Engine (Weeks 3-6) ✅

Build the fundamental game systems that all features depend on.

### 1.1 ECS Foundation ✅

- [x] Entity spawning/despawning
- [x] Component registration system
- [x] System scheduling and ordering
- [x] Event system for game events
- [x] **Test:** Spawn 1000 entities, verify frame time < 16ms

#### 1.2 Rendering Pipeline ✅

- [x] 2D sprite rendering (placeholder assets)
- [x] Camera system (pan, zoom)
- [x] Basic tilemap or terrain rendering
- [x] Unit selection visuals (circles, highlights)
- [x] **Test:** Render 500 sprites at 60fps

#### 1.3 Input System ✅

- [x] Mouse input (click, drag, box select)
- [x] Keyboard shortcuts
- [x] Camera controls (WASD, edge pan, mouse wheel)
- [ ] Input mapping/rebinding foundation
- [x] **Test:** Select units, issue move commands

#### 1.4 Pathfinding ✅ (A* only — [#4](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/4) tracks flowfield)

- [x] Navigation mesh or grid generation
- [x] Flowfield pathfinding (A* implemented, flowfield deferred)
- [x] Unit steering and avoidance
- [x] Formation movement basics
- [x] **Test:** 100 units navigate around obstacles without stacking

#### 1.5 Basic Simulation Loop ✅

- [x] Fixed timestep game loop
- [x] Deterministic update ordering
- [x] Game state serialization (for saves/netcode)
- [x] **Test:** Record and replay 60 seconds of gameplay identically

**Exit Criteria:**

- [x] Units move on command with pathfinding
- [x] Simulation is deterministic (replay produces identical results)
- [x] Stable 60fps with 500 units

---

## Phase 2: Gameplay Foundation (Weeks 7-12) ✅

Implement core RTS mechanics with one faction.

### 2.1 Resource System ✅

- [x] Feedstock resource nodes (temporary & permanent)
- [x] Harvester units (gather, return, deposit)
- [x] Resource storage and display
- [x] Basic economy loop
- [x] Harvesters remember assigned nodes
- [x] **Test:** Harvesters collect resources, player can spend them

#### 2.2 Unit Production ✅

- [x] Building placement system (depot spawning)
- [x] Production queues
- [x] Build times and costs
- [x] Rally points
- [x] **Test:** Build structure, train units, units rally to point

#### 2.3 Combat System ✅ (hitscan only — [#1](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/1) tracks projectiles)

- [x] Health and damage
- [x] Attack commands and auto-attack
- [x] Damage types and armor
- [x] Unit death and cleanup
- [x] Faction-restricted selection/commands
- [ ] Projectile system (deferred to Phase 3)
- [ ] Splash damage for explosives (deferred to Phase 3)
- [x] **Test:** Two armies fight, units die, combat resolves

#### 2.4 Building System ✅

- [x] Building placement (depot exists, B key opens build menu)
- [x] Construction time for buildings (UnderConstruction component)
- [x] Building health and destruction
- [x] Tech requirements (TechLab gates Ranger production)
- [x] Additional building types (Barracks, SupplyDepot, TechLab, Turret)
- [x] **Test:** Build tech tree progression works

#### 2.5 Basic UI ✅

- [x] HUD (resources, supply, minimap)
- [x] Unit selection panel
- [x] Command card (actions/abilities)
- [x] Production panel
- [x] **Test:** All core actions accessible via UI

#### 2.6 Basic AI ✅

- [x] AI faction production (harvesters, combat units)
- [x] AI harvester management
- [x] AI attack behavior (threshold-based)
- [x] **Test:** AI provides basic challenge

**Exit Criteria:**

- [x] Complete gameplay loop: gather → build → train → fight
- [x] One faction playable (Continuity Authority as baseline)

---

## Phase 2.7: Core UX Polish (Weeks 12-14) 🔄 IN PROGRESS

Essential visual feedback and controls that should exist before faction testing.

### 2.7.1 Victory Conditions ✅ ~~[#5](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/5)~~

- [x] Win condition: destroy enemy command building (Depot)
- [x] Lose condition: player command building destroyed
- [x] Victory screen with match statistics
- [x] Defeat screen with retry option
- [x] **Test:** 13 unit tests covering edge cases (multi-faction, mutual destruction, etc.)

#### 2.7.2 Visual Feedback 🔄 NEXT [#6](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/6)

- [ ] Health bars above units and buildings
- [ ] Selection circles under selected units
- [ ] Rally point visuals (line/flag from building to point)
- [ ] Building placement ghost preview (fix existing bug)
- [ ] Damage flash/feedback on hit
- [ ] **Test:** Player can read game state at a glance

#### 2.7.3 Core Commands ⏳ [#8](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/8)

- [ ] Attack-move command (A + click)
- [ ] Patrol command (P + click two points)
- [ ] Stop command (S key)
- [ ] Hold position command (H key)
- [ ] **Test:** All standard RTS commands work

#### 2.7.4 Minimap Interaction ⏳

- [ ] Click minimap to pan camera
- [ ] Right-click minimap to issue move commands
- [ ] Minimap shows unit positions accurately
- [ ] Minimap ping system (Alt+click)
- [ ] **Test:** Can play game primarily via minimap

#### 2.7.5 Production QoL ⏳

- [ ] Select all buildings of type (Ctrl+click icon)
- [ ] Global production tab (see all queues)
- [ ] Production hotkeys (consistent across factions)
- [ ] Queue reordering (shift+click to move)
- [ ] **Test:** Efficient macro gameplay possible

#### 2.7.6 Sprite Improvements 🔄

- [x] Basic unit sprites (infantry, ranger, harvester)
- [x] Basic building sprites (depot, barracks, supply_depot, tech_lab, turret)
- [x] Resource node sprites (temporary/permanent feedstock)
- [x] Faction color tinting system
- [ ] Improve unit sprites (more detailed, readable silhouettes)
- [ ] Improve building sprites (distinct shapes per type)
- [ ] Unit idle/move/attack animation frames (basic)
- [ ] Death animations / destruction effects
- [ ] **Test:** Units distinguishable at strategic zoom

**Exit Criteria:**

- [ ] Game feels "complete" even with one faction
- [ ] New players can understand what's happening
- [ ] All core RTS commands implemented
- [ ] Visual feedback is clear and immediate

---

## Phase 3: Faction Differentiation (Weeks 15-24) 🔄 IN PROGRESS

Implement all five factions with unique mechanics. **Staged rollout** to validate asymmetric balance before scaling.

### 3.0 Faction Rollout Strategy

**Stage A (Weeks 15-17):** Continuity Authority + Collegium

- Two contrasting factions (heavy control vs swarm)
- Validate asymmetric combat balance
- Establish faction integration patterns

**Stage B (Weeks 18-20):** Add Tinkers' Union + Sculptors

- Expand to 4 factions
- Balance pass with 6 matchups
- Unique mechanics (modular, organic)

**Stage C (Weeks 21-24):** Add Zephyr Guild + Full Balance

- Complete 5-faction roster
- Full 10-matchup balance matrix
- Faction-specific AI behaviors

### 3.1 Faction Data Files ✅

- [x] FactionData schema (RON deserialization)
- [x] FactionDataPlugin loader
- [x] Continuity Authority data (10 units, 12 buildings, 10 techs)
- [x] Collegium data (9 units, 10 buildings, 12 techs)
- [x] Tinkers' Union data (10 units, 10 buildings, 12 techs)
- [x] Sculptors data (10 units, 11 buildings, 14 techs)
- [x] Zephyr Guild data (10 units, 14 buildings, 12 techs)
- [x] **Test:** All RON files parse correctly

#### 3.2 Faction Base Stats 🔄 NEXT [#2](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/2)

- [ ] Wire FactionData to unit spawning (use faction stats from RON)
- [ ] Wire FactionData to building spawning (use faction stats from RON)
- [ ] Faction selection at game start
- [ ] All 5 factions playable with Tier 1 units
- [ ] **Test:** Switch factions, units have correct stats

#### 3.3 Tech System (Deferred)

*Design documented in [tech-trees.md](design/systems/tech-trees.md). Implementation after base factions work.*

- [ ] ResearchedTechs component per faction
- [ ] Research queue system (at buildings)
- [ ] Branch exclusivity locking
- [ ] Buff system with diminishing returns
- [ ] Research UI on building command cards
- [ ] Unit/ability unlocks from research
- [ ] **Test:** Research a tech, see effect applied

#### 3.4 Continuity Authority

- [x] Data complete (security forces theme)
- [ ] Tier 1 units spawnable with base stats
- [ ] Stockpile mechanic (deferred to 3.3)
- [ ] **Test:** Playable vs AI with base units

#### 3.5 The Collegium

- [x] Data complete (drone swarms theme)
- [ ] Tier 1 units spawnable with base stats
- [ ] Network scaling (deferred to 3.3)
- [ ] **Test:** Playable vs AI with base units

#### 3.6 The Tinkers' Union

- [x] Data complete (modular mechs theme)
- [ ] Tier 1 units spawnable with base stats
- [ ] Module system (deferred to 3.3)
- [ ] Mobile buildings (deferred)
- [ ] **Test:** Playable vs AI with base units

#### 3.7 The Sculptors

- [x] Data complete (organic biotech theme)
- [ ] Tier 1 units spawnable with base stats
- [ ] Regeneration mechanics (deferred to 3.3)
- [ ] **Test:** Playable vs AI with base units

#### 3.8 The Zephyr Guild

- [x] Data complete (air superiority theme)
- [ ] Tier 1 units spawnable with base stats
- [ ] Trade/piracy systems (deferred to 3.3)
- [ ] **Test:** Playable vs AI with base units

**Exit Criteria:**

- [ ] All five factions playable
- [ ] Each faction feels distinct
- [ ] Basic balance pass (unit stats tuned)

---

## Phase 3.5: Balance Testing Infrastructure

Automated testing to ensure game balance across factions.
*Note: Expand after factions are playable - tests exist, need faction integration to be meaningful.*

### 3.5.1 Unit Balance Tests ✅

- [x] Damage formula verification
- [x] Armor reduction calculations
- [x] Time-to-kill matrices
- [x] Cost efficiency analysis
- [x] Army composition analysis framework
- [x] **Test:** 27 balance tests passing

#### 3.5.2 Simulation-Based Balance Testing

- [ ] Headless simulation runner (no rendering)
- [ ] 1v1 matchup simulator
- [ ] Automated army composition testing
- [ ] Win rate tracking per matchup
- [ ] **Test:** Run 1000 simulated battles in < 1 minute

#### 3.5.3 Faction Balance Matrix

- [ ] Unit vs unit TTK matrix (all factions × all units)
- [ ] Cost-adjusted combat efficiency
- [ ] Counter-unit relationship verification
- [ ] Faction army composition win rates
- [ ] **Test:** All matchups within 45-55% win rate

#### 3.5.4 Economy Balance Testing

- [ ] Harvester ROI timing per faction
- [ ] Build order viability analysis
- [ ] Expansion timing simulations
- [ ] Resource denial impact measurement
- [ ] **Test:** All factions can sustain production by minute 3

#### 3.5.5 Regression Testing [#7](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/7)

- [ ] Golden balance snapshots
- [ ] Stat change impact reports
- [ ] Automated balance alerts on PRs
- [ ] Historical balance tracking
- [ ] Determinism validation in CI (state hash comparison)
- [ ] **Test:** CI fails if balance regresses significantly
- [ ] **Test:** CI fails if simulation determinism breaks

**Exit Criteria:**

- [ ] Can simulate 1000 battles headlessly in < 60 seconds
- [ ] Balance matrix shows all matchups within acceptable range
- [ ] CI alerts on balance-breaking changes

---

## Phase 4: Advanced AI (Weeks 25-30)

Create competent AI opponents with faction-specific behaviors.

> **Note:** Basic victory conditions moved to Phase 2.7

### 4.1 AI Framework [#9](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/9)

- [x] Basic AI production/attack behavior (threshold-based)
- [ ] Behavior tree or utility AI system
- [ ] AI decision timing (not per-frame)
- [ ] Difficulty scaling (Easy/Normal/Hard)
- [ ] Scouting behavior
- [ ] Reactive defense (respond to player rushes)
- [ ] Retreat logic (pull back damaged army)
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

## Phase 5: Multiplayer (Weeks 31-38)

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

## Phase 6: Polish & Content (Weeks 39-50)

Replace placeholder assets and polish the experience.

### 6.1 Asset Pipeline

- [ ] Asset loading system (hot reload in dev)
- [ ] Sprite atlas generation
- [ ] Audio system integration
- [ ] **Test:** Replace placeholder, verify loads correctly

#### 6.2 Visual Polish

- [ ] Final unit sprites (professional pixel art or 3D renders)
- [ ] Final building sprites (distinct, faction-themed)
- [ ] Terrain tiles (biome variety)
- [ ] Visual effects (attacks, explosions, abilities)
- [ ] Unit animations (idle, move, attack, death)
- [ ] UI art pass (faction-themed panels)
- [ ] **Sprite quality pass** — Replace all placeholder sprites with polished versions

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

## Phase 7: Release Preparation (Weeks 51-58)

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

## Known Technical Debt

Issues to address during polish phases or when they become blockers.

| Issue | Description | Priority | Phase |
| ----- | ----------- | -------- | ----- |
| [#3](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/3) | Component duplication between rts_core/rts_game | Low | 6 |
| [#4](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/4) | Flowfield pathfinding for mass movement | Medium | 5 |

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

| Phase | Duration | Status |
| ----- | -------- | ------ |
| 0: Foundation | 2 weeks | ✅ Complete |
| 1: Core Engine | 4 weeks | ✅ Complete |
| 2: Gameplay | 6 weeks | ✅ Complete |
| **2.7: Core UX Polish** | **3 weeks** | **🔄 In Progress** |
| 3: Faction Differentiation | 10 weeks | 🔄 In Progress (data done) |
| 3.5: Balance Testing | 2 weeks | ⏸️ Framework ready |
| 4: Advanced AI | 6 weeks | ⏳ Pending |
| 5: Multiplayer | 8 weeks | ⏳ Pending |
| 6: Polish & Content | 12 weeks | ⏳ Pending |
| 7: Release | 8 weeks | ⏳ Pending |

Total: ~58 weeks (~14 months)

---

## Related Documents

- [Architecture Overview](architecture/overview.md)
- [Tech Stack](architecture/tech-stack.md)
- [Coding Standards](standards/coding-standards.md)
- [Game Design Document](design/gdd.md)
