# Development Roadmap

**Last Updated:** February 4, 2026
**Status:** Phase 3.0 (In Progress) — Testing Infrastructure operational, faction data wiring complete

## Overview

This roadmap outlines the development phases for Post-Scarcity RTS. Following the [January 2026 Review](review-2026-01-26.md), we have rebaselined to prioritize **gameplay readability**, a **two-faction vertical slice** (Continuity vs Collegium), and **sustainable testing** before expanding to the full faction roster.

**Target Vertical Slice:** Continuity Authority vs Collegium of Minds — asymmetric balance with full 8-tier tech trees and 5-10 units per tier.

## Non-Negotiables (Critical)

**If these are not met, the project will fail in playtests and external reviews.**

1. **Readable Combat at Strategic Zoom**

- Units, team ownership, and combat state must be understood at a glance.
- Health, selection, and damage feedback are mandatory before content expansion.

1. **Responsive RTS Controls**

- Core commands must be reliable and predictable (Attack-Move, Hold, Patrol, Stop).

1. **Cohesive Visual Identity**

- A consistent style guide, faction silhouettes, and VFX language are required for retention and marketing.

1. **UX Clarity Over Feature Count**

- Features that reduce clarity or add cognitive load are deferred until UX is proven.

---

## Roadmap Governance (Must Follow)

To avoid random phase switching, we will **follow the roadmap** unless it is explicitly updated.

**Rules:**

1. **Single Active Phase:** Only one phase may be “In Progress” at a time.
2. **Change Requires Update:** Any deviation requires updating this roadmap first.
3. **Issue Intake Cadence:** Review open GitHub issues weekly and map them into the correct phase.
4. **Gate Before Advance:** A phase is not complete until its exit criteria are met or formally revised.
5. **Scope Discipline:** New work must align to the current phase unless the roadmap changes.

### Guiding Principles

- **Two-Faction Vertical Slice** — Prove asymmetric "fun" with Continuity vs Collegium before scaling.
- **8 Tiers, 5-10 Units Each** — Build strategic depth with full tech trees before adding more factions.
- **Readable Chaos** — Visual clarity is a gameplay requirement, not just polish.
- **Automated Balance** — No new factions until the tooling can test them automatically.
- **Steam-Ready** — Architecture supports Steam integration from the start.

---

## Immediate: Code Cleanup (Before Phase 3.1)

**Goal:** Remove dead code and consolidate duplicated structures before adding new features. Identified during codebase review on February 4, 2026.

### Dead Code — Delete

- [x] **Delete empty `src/` directory** at project root — contains only empty child folders (`ai/`, `core/`, `factions/`, `networking/`, `ui/`) from pre-workspace structure

### Duplication — Consolidate

- [x] **Consolidate `ProductionQueue` in `rts_core`** — ~~two incompatible versions existed~~
  - Removed `ProductionComplete` struct and `legacy_production_system()` from systems.rs
  - Updated `simulation.rs` to use `production_system()` from production.rs
  - Entity now has `building: Option<ProductionBuilding>` for production state
  - `TickEvents` now uses `Vec<ProductionEvent>` instead of `Vec<ProductionComplete>`

- [x] **Consolidate damage calculation** — migration to resistance system in progress:
  - ✅ Core formula implemented: `calculate_resistance_damage()` in combat.rs
  - ✅ Schema updated: `CombatStats` now has `resistance`, `armor_penetration`, `armor_class`, `weapon_size`
  - ✅ Combat/projectile systems updated to use resistance damage
  - [ ] Update `data_loader.rs` to parse new fields
  - [ ] Migrate faction RON data (convert armor → resistance)
  - [ ] Remove deprecated `armor_type`, `armor_value` fields

- [x] **Remove duplicate `calculate_resistance_based_damage()`** — removed unused wrapper function and `ResistanceCombatStats` struct from systems.rs (dead code)

### Minor Cleanup

- [ ] **(Optional)** Remove unused `expected_map_size` field in `visual_rating.rs` line 79, or implement its intended use

**Verification:**

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -W clippy::all
```

**Exit Criteria:**

- [x] `src/` directory deleted
- [x] Single `ProductionQueue` definition in `rts_core`
- [x] No duplicate damage calculation functions
- [ ] All tests passing

---

## Phase 0: Foundation (Complete) ✅

Establish development infrastructure and project skeleton.

- [x] Repository & Workspace Setup
- [x] CI/CD Pipeline (Windows/Linux/macOS)
- [x] Pre-commit hooks & formatting

## Phase 1: Core Engine (Complete) ✅

Build the fundamental game systems.

- [x] ECS Foundation (Bevy)
- [x] Rendering Pipeline (Sprites, Camera, Zoom)
- [x] Deterministic Simulation Loop
- [x] Flowfield Pathfinding & Avoidance

## Phase 2: Gameplay Foundation (Complete) ✅

Implement core RTS mechanics with minimal assets.

- [x] Resource System (Gathering, Economy, Storage)
- [x] Unit Production & queues
- [x] Combat System (Health, Damage, Armor)
- [x] Building Construction & Tech gating
- [x] Basic UI (HUD, Command Card)
- [x] Basic AI (Attack/Defend thresholds)

---

## Phase 2.7: Readability & Control (Weeks 12-15) ✅ COMPLETE

**Goal:** The game must be readable and controllable. No new features until the current ones feel responsive.

### 2.7.1 Visual Feedback (Priority High)

- [x] Health bars (units & buildings)
- [x] Selection circles & highlighting
- [x] Combat legend overlay (health/selection cues + faction colors)
- [x] Damage feedback (flash on hit)
- [x] Building placement ghost fix
- [x] Range indicators (attack/vision) when selected
- [x] Faction silhouette/readability pass (strategic zoom target)
  - [x] Unit outline ring for readability
  - [ ] Faction-specific silhouette art pass (deferred to Phase 3.2)
- [x] VFX language baseline (damage types, critical hits, ability activations)
  - [x] Damage-type weapon fire tinting baseline
  - [ ] Ability/critical-hit VFX pass (deferred to Phase 3.2)

### 2.7.2 Core Controls (Priority High)

- [x] Attack-Move (A-Click)
- [x] Stop (S) / Hold Position (H)
- [x] Patrol (P)
- [x] Double-click to select all of type
- [x] Minimap interaction (click to move/pan)
- [x] UI feedback for issued commands (acknowledgement cues: ping + audio hook)

### 2.7.3 Accessibility Base

- [x] UI Scaling support
- [x] Rebindable keys foundation
- [x] Basic UI contrast mode (minimum viable for readability)

### 2.7.4 Determinism & Core Wiring (Priority High)

- [x] **Core Simulation Drives Client:** Bevy client renders state from `rts_core::Simulation` (no parallel float sim)
- [x] **Projectile System Integrated:** `projectile_system` runs in core tick loop
- [x] **Determinism Hashing:** Per-tick state hash logged in dev builds
- [x] **Replay-Ready Command Stream:** Commands are the only sim inputs (UI/render never mutates sim state)

**Exit Criteria:**

- [x] A new player can understand combat state at a glance.
- [x] All standard RTS commands function reliably.
- [x] Unit tests for command issuance logic.
- [x] Client visuals reflect core sim state (no sim drift).
- [x] Readability baseline hit: units/teams/health identifiable in < 2 seconds.

---

## Phase 2.8: Critical Gameplay Fixes (Weeks 15-16) ✅ COMPLETE

**Goal:** Fix game-breaking bugs discovered in playtesting. These block the Vertical Slice gate.

### 2.8.1 Pathfinding Integration (Priority Critical) ✅

- [x] Add `path_waypoints` field to core Entity for multi-step movement
- [x] Store NavGrid in Simulation; initialize from map data
- [x] Modify `Command::MoveTo` to call `find_path()` and store waypoints
- [x] Path-following: move toward first waypoint, pop on arrival
- [x] Integrate `mark_building_in_navgrid()` when buildings placed/destroyed
- [x] Unit tests for obstacle avoidance

### 2.8.2 Combat Damage Sync (Priority Critical) ✅

- [x] Add `sync_attack_targets_to_core` system (Bevy AttackTarget → core attack_target)
- [x] Verify damage flows: core combat → damage_events → health sync → death
- [x] Unit tests for damage application and death trigger

### 2.8.3 Economy Flow Polish ✅

- [x] Player harvester auto-return to last resource node after deposit (`assigned_node` persists)
- [ ] Visual feedback when harvester assigned to node (line or icon) — deferred to Phase 3.6

**Exit Criteria:**

- [x] Units pathfind around obstacles (no terrain clipping)
- [x] Units die when health reaches 0
- [x] Harvesters complete full gather→deposit→return loops without manual intervention

---

## Backlog Alignment (GitHub Issues)

These issues are actively tracked and mapped to the roadmap phases for clarity and prioritization.

### Phase 2.7 (Readability, Control, Determinism)

- [Issue #6](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/6) — Health bars (combat readability) ✅
- [Issue #22](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/22) — Death feedback (delay despawn / effect) ✅
- [Issue #12](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/12) — Build placement ghost visuals ✅ (duplicates: #16, #19)
- [Issue #13](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/13) — Resource bar/tooltips polish ✅ (duplicates: #17, #20)
- [Issue #11](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/11) — Allow selection of enemy units ✅ (duplicates: #15, #18)
- [Issue #14](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/14) — Input conflict on B key for build menu ✅
- [Issue #23](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/23) — Selection radius should use collider/size ✅
- [Issue #21](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/21) — Attack commands should honor shift-queue ✅
- [Issue #8](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/8) — Attack-move / Stop / Hold / Patrol commands ✅
- [Issue #24](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/24) — Resolve simulation duplication (core authoritative) ✅
- [Issue #29](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/29) — Determinism leak in `Vec2Fixed` ✅
- [Issue #26](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/26) — Consolidate component definitions (core ↔ view mirroring) ✅
- [Issue #30](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/30) — Decouple victory logic from view layer ✅

### Phase 2.8 (Critical Gameplay Fixes) — NEW

- **NEW** — Pathfinding integration (connect A* to movement system)
- **NEW** — Combat damage sync (Bevy AttackTarget → core attack_target)
- **NEW** — Harvester auto-return after deposit

### Phase 3.0 (Testing Infrastructure) — IN PROGRESS

- [Issue #34](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/34) — Headless runner & fixed timestep for AI/CI ✅
- [Issue #7](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/7) — CI determinism validation (infrastructure ready)
- **COMPLETE** — In-game debug console ✅
- **COMPLETE** — Scenario system (RON format) ✅
- **COMPLETE** — Faction data wiring for headless ✅
- **COMPLETE** — Procedural map generation ✅
- [x] Replay save/load system

### Phase 3.1 (Data Wiring / Combat Depth)

- [Issue #25](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/25) — Replace legacy `UnitType` with data-driven `UnitKindId`
- [Issue #1](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/1) — Projectiles & splash damage (combat depth)

### Quality & Process (Ongoing)

- [Issue #32](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/32) — Replace mock simulation tests with real engine tests
- [Issue #33](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/33) — Enforce determinism testing standards in CONTRIBUTING
- [Issue #31](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/31) — Tighten clippy lints + security audit guidance

### Hygiene / Maintenance

- [Issue #27](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/27) — Stub/unused crates cleanup
- [Issue #35](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/35) — Windows clippy incremental access warning
- [Issue #3](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/3) — Component duplication audit (post-Phase 3)

### Phase 4.2 (AI Architecture)

- [Issue #9](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/9) — AI improvements beyond thresholds
- [Issue #28](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/28) — Externalize AI parameters into config (expanded: multi-trigger AI)

### Phase 5 (Pathfinding Scale)

- [Issue #4](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/4) — Flowfield/formation pathfinding for mass movement

---

## Vertical Slice Gate (Required to Enter Phase 3.5)

This is the hard ship gate for any external demo or publisher review.

**Faction Requirements:**

- [ ] Continuity Authority fully playable from data (8 tiers, 5-10 units per tier)
- [ ] Collegium of Minds fully playable from data (8 tiers, 5-10 units per tier)
- [ ] Asymmetric balance validated (45-55% win rate range via automated testing)

**Core Gameplay:**

- [x] Core commands complete and reliable (Attack-Move, Hold, Patrol, Stop)
- [x] Visual readability baseline met (health bars, selection, hit feedback)
- [x] Determinism checks operational (hash comparison working, CI integration pending)
- [ ] One polished 2-player map with resource flow and expansions
- [x] **Phase 2.8 complete:** Pathfinding works, combat kills units, harvesters loop

**Architecture Requirements:**

- [ ] **Unified Player Interface:** PlayerFacade implemented, AI uses same APIs as player
- [ ] **Fair AI Testing:** AI cannot see or target non-visible enemies

**Development Tooling:**

- [x] **Testing infrastructure operational:** Debug console, headless runner
- [x] **Automated balance testing:** 400+ games/sec with faction data
- [ ] **Balance Dashboard:** Win rate tracking over batches
- [ ] **Replay Viewer:** Watch recorded games for debugging
- [ ] **Sprite Preview Pipeline:** Hot-reload and zoom-level preview
- [ ] **Stat Editor GUI:** Tweak RON values with live simulation

---

## Phase 3.0: Testing Infrastructure (Weeks 17-18) 📅 IN PROGRESS

**Goal:** Enable automated and assisted testing so bugs can be caught without manual playthroughs.

### 3.0.1 In-Game Debug Console

- [x] Backtick toggle, egui window
- [x] Commands: `spawn`, `kill`, `teleport`, `god_mode`, `resources`, `speed`, `win`, `lose`
- [x] Gated behind `#[cfg(feature = "dev-tools")]`

### 3.0.2 Scenario System

- [x] Define RON scenario format: entities, initial commands, win conditions
- [x] ScenarioLoader populates Simulation from file
- [x] At least 3 scenarios: empty, skirmish, stress-test (skirmish_1v1 complete)

### 3.0.3 Replay System ✅ COMPLETE

- [x] `save_replay(path)` serializes CommandStream + scenario ID
- [x] `load_replay(path)` recreates simulation and plays back commands
- [x] Replay viewer mode (read-only spectate)

### 3.0.4 Headless Runner & CI Integration ✅ COMPLETE

- [x] Binary target: `rts_headless` that loads scenario, runs N ticks, reports state
- [x] Batch testing: 235+ games/sec, tested 10K+ games with 50/50 balance
- [x] Determinism validation: state hash comparison across runs
- [x] Metrics collection: per-game JSON with damage, K/D, resources, events
- [x] Screenshot system: timed snapshots + major battle triggers
- [x] ASCII visualizer for debugging game states
- [x] Strategy system: Rush, Economic, Turtle, FastExpand, Harassment, AllIn
- [x] CI integration: block on determinism divergence (`final_state_hash` in batch results)

### 3.0.5 Dynamic Spawn & Map Generation ✅ COMPLETE

- [x] Spawn generator with patterns: Corners, Horizontal, Vertical, Random, Arena, Cross
- [x] Pseudo-symmetric spawn validation (fair distances)
- [x] Configurable randomness factor for spawn variance
- [x] Procedural map generation (terrain, resources, obstacles)
- [x] Resource placement with symmetry constraints
- [x] Multiple map templates/sizes

### 3.0.6 Faction Data Wiring ✅ COMPLETE

**Issue (Resolved):** Headless tests were using hardcoded generic units, not faction RON data.
Range values in RON files were 10× too small, causing combat lockups.

- [x] Load FactionData RON files in headless runner
- [x] Spawn faction-specific units with correct stats
- [x] Fixed range scale mismatch (RON values now 40-120 units, not 4-12)
- [x] Validate Continuity vs Collegium end-to-end (data → simulation → metrics)
- [ ] Use production chain from faction data (buildings → units) — deferred to Phase 3

**Performance:** 400+ games/sec with real faction data (restored from 0.0 games/sec after fix)

**Exit Criteria:**

- [x] Debug console works in dev builds
- [x] `cargo run --bin rts_headless -- batch` completes without graphics
- [x] Headless uses real faction data (not hardcoded units)
- [x] CI blocks on determinism divergence

---

## Phase 3: The Vertical Slice (Weeks 19-30) 📅 PLANNED

**Goal:** A fully playable, polished two-faction experience with **Continuity Authority vs Collegium of Minds**. Both factions have complete 8-tier tech trees with 5-10 units per tier. This is our proof-of-concept for publishers/players.

### 3.1 Development Tooling — PRIORITY

Build the tools needed for rapid iteration on gameplay and visuals.

#### 3.1.1 Balance Dashboard

- [ ] Web-based dashboard showing win rates over time
- [ ] Per-unit performance metrics (K/D, cost efficiency, lifespan)
- [ ] Trend detection: flag units with >55% or <45% win contribution
- [ ] Export to CSV for external analysis
- [ ] Integration with headless batch runner

#### 3.1.2 Replay Viewer

- [ ] `save_replay(path)` serializes CommandStream + scenario ID
- [ ] `load_replay(path)` recreates simulation and plays back commands
- [ ] Read-only spectate mode with timeline scrubbing
- [ ] Slow-motion and fast-forward controls
- [ ] Event markers: "First Blood", "Major Battle", "Victory"

#### 3.1.3 Sprite Preview Pipeline

- [ ] Hot-reload watcher for texture changes
- [ ] Multi-zoom preview (strategic/tactical/close-up)
- [ ] Faction color overlay preview
- [ ] Animation state preview (idle/move/attack/death)
- [ ] Silhouette readability check tool

#### 3.1.4 Stat Editor GUI

- [ ] egui-based panel for editing unit stats in-game
- [ ] Live preview: spawn unit with modified stats
- [ ] Save changes back to RON files on demand
- [ ] Side-by-side comparison (original vs modified)
- [ ] Quick balance presets (+10% HP, -10% cost, etc.)

### 3.1.5 Unified Player Interface (PlayerFacade)

**Why:** Currently, headless AI and GUI player use different code paths to interact with the simulation. AI has global vision (cheating), and attack targeting bypasses the Command system in some places. This breaks the principle that AI should be a fair, representative opponent.

**Architecture Goal:** Both human players and AI use identical interfaces — same commands, same visibility, same information access. The only difference is who makes the decisions.

#### 3.1.5.1 PlayerFacade Trait (rts_core)

- [ ] Create `player_facade.rs` module in `rts_core`
- [ ] Define `PlayerFacade` trait:
  - `issue_command(unit: EntityId, cmd: Command)` — single unit control
  - `issue_commands(units: &[EntityId], cmd: Command)` — group control
  - `get_visible_entities(faction: FactionId) -> Vec<EntityId>` — only what player can see
  - `get_own_entities(faction: FactionId) -> Vec<EntityId>` — player's units/buildings
  - `query_unit_info(id: EntityId) -> Option<UnitInfo>` — basic info (if visible)
  - `get_resources(faction: FactionId) -> PlayerResources` — economy state
- [ ] Export in `rts_core::prelude`

#### 3.1.5.2 Core Visibility System

- [ ] Add `vision_range: Option<Fixed>` to `Entity` struct
- [ ] Add `vision_range: Option<Fixed>` to `UnitData` struct
- [ ] Add `is_visible_to(viewer_faction, target_id) -> bool` method to `Simulation`
- [ ] Add `get_visible_enemies(faction) -> Vec<(EntityId, Position)>` helper
- [ ] Default vision range = 2× attack range when not specified

#### 3.1.5.3 Unify Attack Targeting

- [ ] Remove or make `set_attack_target()` private (`pub(crate)`)
- [ ] Update GUI `acquire_attack_targets()` to issue `Command::Attack` via command buffer
- [ ] Update `auto_attack_system` to respect visibility
- [ ] Remove direct `set_attack_target()` calls in sync systems

#### 3.1.5.4 Implement Facades for Both Frontends

- [ ] Create `SimulationPlayerFacade` struct (wraps `&mut Simulation` + `FactionId`)
- [ ] Update `game_runner.rs` headless batch:
  - Replace `sim.entities().sorted_ids()` with `facade.get_visible_enemies()`
  - Replace `sim.apply_command()` with `facade.issue_command()`
- [ ] Update `ai.rs` GUI AI:
  - Filter enemy queries through visibility
  - Route commands through `CoreCommandBuffer`

**Exit Criteria:**

- [ ] `PlayerFacade` trait defined and implemented
- [ ] AI cannot target units outside vision range
- [ ] All unit commands flow through `Command` enum (no direct state mutation)
- [ ] Headless batch runner uses same interface as GUI AI
- [ ] Unit tests verify AI cannot "cheat"

### 3.2 Combat System Migration (Resistance-Based)

Migrate from flat armor subtraction to percentage-based damage reduction. See [combat.md](design/systems/combat.md) for full design.

**Why:** Flat armor creates non-linear "cliffs" (damage 10 - armor 5 = 5; damage 8 - armor 5 = 3 — 40% DPS loss for 20% damage reduction). Percentage-based scales smoothly and enables meaningful counter-play.

#### 3.2.1 Core Formula Change

- [x] Replace `final_damage = base_damage - armor` with `final_damage = base_damage × (1 - resistance%)`
- [x] Implement resistance cap at 75%
- [x] Add armor penetration stat (ignores % of resistance)
- [x] Add minimum damage floor (1)

#### 3.2.2 Schema Updates

- [x] Add `resistance` field (0-75%) to combat struct
- [x] Add `armor_penetration` field (0-100%) to weapons
- [x] Add `damage_type` enum (Kinetic, Explosive, Energy, BioAcid, Fire)
- [x] Add `weapon_size` enum (Light, Medium, Heavy)
- [x] Add `armor_class` enum (Light, Medium, Heavy, Air, Building)
- [ ] Update `data_loader.rs` to parse new fields

#### 3.2.3 Data Migration

- [ ] Migrate Continuity Authority units (convert armor → resistance)
- [ ] Migrate Collegium of Minds units
- [ ] Add armor penetration values to anti-armor weapons
- [ ] Assign damage types to all weapons
- [ ] Assign weapon sizes and armor classes

#### 3.2.4 Size Class Tracking System

- [x] Light weapons deal reduced damage to Heavy armor (50%)
- [x] Heavy weapons deal reduced damage to Light targets (25% — tracking penalty)
- [x] Medium weapons are versatile (75-100% vs all sizes)

#### 3.2.5 Validation

- [ ] Update balance tests for new combat formula
- [ ] Verify 45-55% win rate maintained after migration
- [ ] Document resistance/penetration guidelines for future units

**Exit Criteria:**

- [ ] All combat uses percentage-based damage reduction
- [ ] Flat `armor` field removed from all RON files
- [ ] Size class modifiers working (Heavy vs Light penalties)
- [ ] Balance validated on new system

### 3.3 Tier System & Unit Roster

Define the 8-tier structure for both factions.

#### Tier Structure

| Tier | Name | Unlock | Unit Count Target |
| --- | --- | --- | --- |
| T1 | Basic | Start | 5-6 units |
| T2 | Trained | Barracks II | 5-6 units |
| T3 | Elite | Tech Lab | 6-8 units |
| T4 | Advanced | Factory | 6-8 units |
| T5 | Specialist | War Hall | 5-6 units |
| T6 | Heavy | Heavy Bay | 4-5 units |
| T7 | Super | Command Center | 3-4 units |
| T8 | Ultimate | Capital Dock | 1-2 units |

#### Continuity Authority (40-50 units total)

- [ ] T1: Security Team, Scout Drone, Worker, Light Turret, Militia
- [ ] T2: Tactical Squad, Recon Vehicle, Engineer, Mortar Pit
- [ ] T3: Guardian Mech, Sniper Team, Shield Bearer, Combat Medic, Bunker
- [ ] T4: Siege Tank, APC, Artillery Platform, EMP Drone
- [ ] T5: Stealth Operative, Heavy Gunner, Field Commander
- [ ] T6: Thor Walker, Devastator, Mobile Fortress
- [ ] T7: Capital Mech, Orbital Strike Beacon
- [ ] T8: Titan (super-unit)

#### Collegium of Minds (40-50 units total)

- [ ] T1: Research Assistant, Probe, Worker Automaton, Sensor Node
- [ ] T2: Drone Squadron, Data Analyst, Hacker Unit, Turret Bot
- [ ] T3: Neural Knight, Psi-Operative, Mind Shield, Bio-Lab
- [ ] T4: Hover Tank, Teleporter, Plasma Artillery, Swarm Controller
- [ ] T5: Phase Assassin, Gravity Manipulator, Network Node
- [ ] T6: Colossus Walker, Mind Flayer, Siege Brain
- [ ] T7: Quantum Core, Reality Bender
- [ ] T8: Overmind (super-unit)

### 3.4 Technical Wiring (GDD Alignment)

- [ ] **Issue #25 (Active):** Replace legacy `UnitType` with data-driven `UnitKindId`.
- [ ] **Data Wiring:** Connect FactionData RON files to actual Unit/Building spawning.
- [x] **Headless Integration:** Faction RON loads in `rts_headless` for balance testing.
- [ ] **No Hardcoded Spawns:** All scenario/unit spawns are driven by data definitions.
- [ ] **Visibility System:** Core visibility via `is_visible_to()` (see Phase 3.1.5)
- [ ] **Fog of War (Full):** Explored/unexplored terrain states (depends on 3.1.5)
- [ ] **Line of Sight:** Units cannot shoot what they cannot see (depends on 3.1.5)

### 3.5 Procedural Map Generation

- [ ] **MapConfig RON:** Size, resource density, symmetry mode, obstacle density
- [ ] **Terrain Types:** Passable, impassable (cliffs), slow (rough)
- [ ] **Resource Placement:** Pseudo-symmetric with variance factor
- [ ] **Start Position Generator:** Fair distance, resource access validation
- [ ] **Obstacle Placement:** Choke points, expansion areas
- [ ] **NavGrid Integration:** Generate walkable grid from terrain

### 3.6 Visual Identity

- [ ] **Sprite Audit:** Replace "programmer art" with cohesive placeholders.
- [ ] **Silhouette Pass:** Faction-specific silhouettes at strategic zoom.
- [ ] **Asset Pipeline:** Define automated import process & sprite atlas tools.
- [ ] **Animation:** Basic Idle (breathing) / Move (bobbing) / Attack (recoil) states.
- [ ] **Audio:** Basic SFX for specific unit types (gunfire, engines).
- [ ] **Terrain:** Basic tileset variation (not just flat color).
- [ ] **Ability/Hit VFX:** Ability activation and critical-hit VFX pass.

### 3.7 The Slice Content

- [ ] **Map:** One polished 2-player map with distinct terrain functionality.
- [ ] **Both Factions:** Continuity + Collegium fully playable (All 8 tiers).
- [ ] **AI:** "Standard" AI personality that uses the full tech tree.
- [ ] **Tutorial:** A 5-minute onboarding flow (text/triggers).

### 3.8 Performance & Pipeline

- [ ] **Performance:** Benchmark suite (1k pathfinding, UI redraw).
- [ ] **Asset Pipeline:** Hot-reloading watcher for textures/data.

**Exit Criteria:**

- [ ] "Vertical Slice" build labeled and archived.
- [ ] Both factions playable with 8 tiers each.
- [ ] Automated headless simulation runs 100 battles < 1 min.
- [ ] Balance validated: 45-55% win rate for Continuity vs Collegium.
- [ ] All 4 development tools operational.

---

## Phase 4: Faction Expansion (Weeks 31-40)

**Goal:** Expand to full 5-faction roster using the infrastructure and tooling built in Phase 3. Continuity and Collegium are already complete from the Vertical Slice.

### 4.0 Process & Document Health

- [ ] **Documentation:** Quarterly Architecture vs Implementation review.
- [ ] **AI Gym:** Automated gameplay evaluation loop ([Details](design/systems/ai-testing-and-toolchain.md)).

### 4.1 Tinkers' Union (Faction 3)

- [ ] Define 8-tier unit roster (40-50 units)
- [ ] Implement unique mechanics (Scrap Salvage, Makeshift Repairs)
- [ ] Asset rollout
- [ ] Balance pass: 3-way matchup (vs Continuity and Collegium)
- [ ] Automated win rate validation

### 4.2 Sculptors of Flesh (Faction 4)

- [ ] Define 8-tier unit roster (40-50 units)
- [ ] Implement unique mechanics (Regeneration, Bio-Adaptation)
- [ ] Asset rollout
- [ ] Balance pass: 4-way matchup
- [ ] Automated win rate validation

### 4.3 Zephyr Guild (Faction 5)

- [ ] Define 8-tier unit roster (40-50 units)
- [ ] Implement unique mechanics (Flight, Speed Buffs)
- [ ] Asset rollout
- [ ] Balance pass: 5-way matchup (full roster)
- [ ] Automated win rate validation

### 4.4 Advanced AI (Multi-Trigger Architecture)

**Replaces legacy "60-second grace period" with proper AI decision-making.**

**AI Trigger System:**

- [ ] **Economic Trigger:** Attack when resource income exceeds player by 20%+ for 30s
- [ ] **Scout Trigger:** Send initial scout; attack after discovering player base location
- [ ] **Wave Trigger:** Periodic attack waves that scale with game time (first at 2min, then every 90s)
- [ ] **Build-Order Trigger:** Configurable attack timings per AI personality
- [ ] **Threat Response:** Defend when attacked; counterattack after repelling

**AI Personality Profiles (config-driven):**

- [ ] **Aggressor:** Scout early, attack frequently, favor military production
- [ ] **Turtle:** Defend until tech advantage, heavy wave at 10min
- [ ] **Adaptive:** Mix triggers based on what works (simple learning)

### 4.5 Automated Balance Tuning

- [ ] Time-to-Kill (TTK) Matrix validation (all 5 factions).
- [ ] Regression testing using Headless Runner.
- [ ] Cost derivation formula (prevent "cheap OP" units).
- [ ] Per-tier balance validation (T1 vs T1, T8 vs T8).

**Exit Criteria:**

- [ ] All 5 factions playable with 8 tiers each (200-250 total units).
- [ ] Automated balance tests passing (45-55% win rate for all matchups).
- [ ] All faction-specific mechanics implemented and balanced.

---

## Phase 5: Advanced Simulation (Weeks 31-38)

**Goal:** Implement the complex GDD features deferred during the Vertical Slice.

### 5.1 Tactical Depth

- [ ] **Height Advantage:** Damage/Range bonus from cliffs.
- [ ] **Cover System:** Damage reduction in craters/ruins.
- [ ] **Veterancy:** Units gain stats on kills.
- [ ] **Projectile Physics:** (Optional) Switch from hitscan if needed for gameplay feel.

### 5.2 Advanced AI

- [ ] Behavior Trees / Utility AI integration.
- [ ] Distinct Personalities (Aggressive, Turtle, Experimental).
- [ ] Difficulty Tiers.

### 5.3 Polish & Accessibility

- [ ] **Accessibility:** Colorblind modes & High contrast.
- [ ] **Feedback:** Advanced audio mix for combat clarity.

---

## Phase 6: Multiplayer & Networking (Weeks 39-46)

**Goal:** Turn the deterministic sim into a networked game.

### 6.1 Networking

- [ ] Lockstep protocol implementation.
- [ ] Desync detection & recovery tools.
- [ ] Lobby system.

### 6.2 Platform Features

- [ ] Steamworks integration.
- [ ] Replay system with fast-forward.
- [ ] Spectator mode.

---

## Phase 7: Content & Campaign (Weeks 47+)

**Goal:** Narrative content and distinct game modes.

- [ ] Campaign Mission Framework (triggers, objectives).
- [ ] Mission scripting.
- [ ] Briefing screens / Narrative delivery.
- [ ] Alternate Victory Conditions (Economic, Domination).

---

## Deferred / Out of Scope (For Now)

- Modding Support (SDK).
- Co-op Survival Mode.

---

## Recent Session Log (February 4, 2026)

### Roadmap Updated for Vertical Slice ✅

Roadmap restructured to target two-faction vertical slice:

- **Target:** Continuity Authority vs Collegium of Minds
- **Scope:** 8 tiers per faction, 5-10 units per tier (80-100 units for slice)
- **Tooling:** Balance Dashboard, Replay Viewer, Sprite Pipeline, Stat Editor
- **Phase 4:** Remaining 3 factions (Tinkers, Sculptors, Zephyr)

### Range Scale Bug Fixed ✅

Discovered and fixed SEVERE performance bottleneck:

- **Root cause:** Faction RON files had range values of 4-12 units (Fixed-point raw bits), but simulation expected 40-120 units
- **Symptom:** 3461 seconds for 5 games (0.0 games/sec) — units could never close to attack range
- **Fix:** Multiplied all range values in 5 faction RON files by 10×
- **Result:** 400+ games/sec with faction data (restored)

### Balance State (Continuity vs Collegium)

**100-game sample with faction data:**

- Continuity: 74% win rate
- Collegium: 26% win rate
- **Likely cause:** Continuity T1 (Security Team) has more armor than Collegium T1 (Research Assistant)

### Current Status

**70 tests passing.** Faction data loading fully operational. Performance restored.

### Next Steps

1. ~~Build Balance Dashboard (track win rates over batches)~~
2. ~~Adjust Collegium stats to reach 45-55% balance target~~ ✅ Achieved ~55/45
3. **Implement Combat System Migration (Phase 3.2)** — Resistance-based % damage reduction
4. Define additional T2-T8 units for both factions
5. Implement Replay Viewer for debugging

### Phase 2.8 Verification Complete ✅ (February 4, 2026)

Discovered Phase 2.8 (Critical Gameplay Fixes) was already complete:

- **2.8.1 Pathfinding Integration:** `path_waypoints` in Entity, NavGrid in Simulation, A* integration with MoveTo, waypoint following, building NavGrid updates, comprehensive tests
- **2.8.2 Combat Damage Sync:** `sync_attack_targets_to_core`, `clear_removed_attack_targets`, health sync, death processing — all implemented
- **2.8.3 Economy Flow:** Harvester `assigned_node` persists through gather→deposit→return cycle, auto-return working

**Phase 2.8 marked COMPLETE.** Only visual feedback for harvester assignment deferred.

### CI Determinism Validation Added ✅ (February 4, 2026)

Added determinism check to GitHub Actions CI workflow:

- **New CI job:** `determinism` runs 10 games with same seed twice
- **Hash comparison:** `final_state_hash` added to `GameMetrics`, compared between runs
- **Workflow:** Build headless → Run batch 1 → Run batch 2 → Compare hashes with jq
- **Result:** CI will now block on any determinism divergence

### Combat System Redesign ✅

Identified fundamental balance issue with flat armor creating non-linear "cliffs":

- **Problem:** `damage - armor` causes 20% damage reduction to become 40% DPS loss
- **Solution:** Percentage-based resistance with cap at 75%
- **Counter-play:** Armor penetration stat, size class modifiers, damage types
- **Document Updated:** [combat.md](design/systems/combat.md) revised with full design
- **Migration Added:** Phase 3.2 in roadmap tracks implementation steps

### Unified Player Interface Architecture (February 4, 2026)

Identified architectural divergence between headless AI and GUI player:

**Problems Found:**

- AI has global vision (iterates ALL entities, no visibility check)
- Attack targeting uses `set_attack_target()` directly, bypassing `Command::Attack`
- Headless batch runner and GUI use different code paths

### Solution: PlayerFacade Architecture

- Create `PlayerFacade` trait defining "what can a player do"
- Add `is_visible_to()` visibility check to core `Simulation`
- Add `vision_range` to units (defaults to 2× attack range)
- Route ALL targeting through `Command::Attack`
- Both AI and human use identical interface — only decision-making differs

**Benefits:**

- Fair AI opponents (same information as player)
- Representative testing (AI behavior matches player capabilities)
- Fast batch testing (direct facade calls, no Bevy overhead)
- Multiplayer-ready (same visibility for all clients)

**Files to create/modify:**

- ✅ NEW: `crates/rts_core/src/player_facade.rs` — Created with PlayerFacade trait, SimulationPlayerFacade impl, 6 tests
- ✅ MOD: `crates/rts_core/src/simulation.rs` — Added `vision_range` to Entity, `is_visible_to()`, `get_visible_enemies_for()`, `get_faction_entities()`
- ✅ MOD: `crates/rts_core/src/lib.rs` — Registered player_facade module
- ✅ MOD: `crates/rts_core/src/components.rs` — Added `with_armor_class()` builder
- MOD: `crates/rts_core/src/data/unit_data.rs` (add vision_range parsing)
- MOD: `crates/rts_headless/src/game_runner.rs` (use facade)
- MOD: `crates/rts_game/src/ai.rs` (filter through visibility)
- MOD: `crates/rts_game/src/combat.rs` (route auto-attack through commands)

### Vision & Intelligence System Design (February 4, 2026)

Created comprehensive vision system design to enable genuine faction asymmetry:

**Core Design Principles:**

- **Sight range ≠ attack range** — Artillery can't see what it's shooting
- **Scout/spotter synergy** — Scouts provide vision, artillery provides firepower
- **Faction identity through vision** — Each faction finds intel differently

**Key Mechanics:**

| Unit Type | Sight Range | Attack Range | Role |
| --------- | ----------- | ------------ | ---- |
| Scout | 14-18 | 0-2 | Vision platform, expendable |
| Infantry | 8-10 | 5-7 | Self-sufficient |
| Sniper | 6-8 | 12-14 | Needs spotters |
| Artillery | 4-6 | 16-20 | Totally blind without spotters |

**Faction Vision Doctrines:**

- **Continuity Authority:** Panopticon — sensor network, Watcher drones (tethered)
- **Collegium:** Distributed Awareness — fast cheap scouts, network bonuses, sniper doctrine
- **Tinkers' Union:** Sensor Improvisation — placeable beacons, armed scouts, smoke
- **Sculptors:** Organic Senses — bonding to enemies, bio-sensors, spore vision
- **Zephyr Guild:** Air Superiority — altitude advantage, massive range from air scouts

**Collegium Sniper Doctrine (Example):**

1. Scout Drones maintain vision (cloaked, expendable, sight 16)
2. Hover Tanks fire from max range (attack 14, sight 8)
3. Shield Drones protect the snipers
4. Enemy must kill invisible scouts OR close distance under fire

**Documents Created/Updated:**

- NEW: [Vision & Intelligence System](design/systems/vision-and-intel.md)
- MOD: [Combat System](design/systems/combat.md) — Added vision integration section
- MOD: [Collegium Faction](design/factions/collegium.md) — Expanded Scout Drone and Hover Tank with explicit sniper doctrine
