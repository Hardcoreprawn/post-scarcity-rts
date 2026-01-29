# Development Roadmap

**Last Updated:** January 29, 2026
**Status:** Phase 2.7 (In Progress)

## Overview

This roadmap outlines the development phases for Post-Scarcity RTS. Following the [January 2026 Review](review-2026-01-26.md), we have rebaselined to prioritize **gameplay readability**, a **vertical slice**, and **sustainable testing** before expanding to multiple factions.

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

- **Vertical Slice First** — Prove the "fun" with one polished faction before scaling.
- **Readable Chaos** — Visual clarity is a gameplay requirement, not just polish.
- **Automated Balance** — No new factions until we can test them automatically.
- **Steam-Ready** — Architecture supports Steam integration from the start.

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

## Phase 2.7: Readability & Control (Weeks 12-15) 🔄 IN PROGRESS

**Goal:** The game must be readable and controllable. No new features until the current ones feel responsive.

### 2.7.1 Visual Feedback (Priority High)

- [x] Health bars (units & buildings)
- [x] Selection circles & highlighting
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

- [ ] **Core Simulation Drives Client:** Bevy client renders state from `rts_core::Simulation` (no parallel float sim)
- [ ] **Projectile System Integrated:** `projectile_system` runs in core tick loop
- [ ] **Determinism Hashing:** Per-tick state hash logged in dev builds
- [ ] **Replay-Ready Command Stream:** Commands are the only sim inputs (UI/render never mutates sim state)

**Exit Criteria:**

- [ ] A new player can understand combat state at a glance.
- [ ] All standard RTS commands function reliably.
- [x] Unit tests for command issuance logic.
- [ ] Client visuals reflect core sim state (no sim drift).
- [ ] Readability baseline hit: units/teams/health identifiable in < 2 seconds.

---

## Backlog Alignment (GitHub Issues)

These issues are actively tracked and mapped to the roadmap phases for clarity and prioritization.

### Phase 2.7 (Readability, Control, Determinism)

- [Issue #6](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/6) — Health bars (combat readability)
- [Issue #22](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/22) — Death feedback (delay despawn / effect)
- [Issue #12](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/12) — Build placement ghost visuals (duplicates: #16, #19)
- [Issue #13](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/13) — Resource bar/tooltips polish (duplicates: #17, #20)
- [Issue #11](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/11) — Allow selection of enemy units (duplicates: #15, #18)
- [Issue #14](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/14) — Input conflict on B key for build menu
- [Issue #23](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/23) — Selection radius should use collider/size
- [Issue #21](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/21) — Attack commands should honor shift-queue
- [Issue #8](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/8) — Attack-move / Stop / Hold / Patrol commands
- [Issue #24](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/24) — Resolve simulation duplication (core authoritative)
- [Issue #29](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/29) — Determinism leak in `Vec2Fixed`
- [Issue #26](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/26) — Consolidate component definitions (core ↔ view mirroring)
- [Issue #30](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/30) — Decouple victory logic from view layer

### Phase 3.4 (Infrastructure Catch-up)

- [Issue #34](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/34) — Headless runner & fixed timestep for AI/CI
- [Issue #7](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/7) — CI determinism validation

### Phase 3.1 (Data Wiring / Combat Depth)

- [Issue #25](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/25) — Replace legacy `UnitType` with data-driven `UnitKindId`
- [Issue #1](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/1) — Projectiles & splash damage (combat depth)

### Quality & Process (Immediate)

- [Issue #32](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/32) — Replace mock simulation tests with real engine tests
- [Issue #33](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/33) — Enforce determinism testing standards in CONTRIBUTING
- [Issue #31](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/31) — Tighten clippy lints + security audit guidance

### Hygiene / Maintenance

- [Issue #27](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/27) — Stub/unused crates cleanup
- [Issue #28](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/28) — Externalize AI parameters into config
- [Issue #35](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/35) — Windows clippy incremental access warning
- [Issue #3](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/3) — Component duplication audit (post-Phase 3)

### Phase 4 (AI Depth)

- [Issue #9](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/9) — AI improvements beyond thresholds

### Phase 5 (Pathfinding Scale)

- [Issue #4](https://github.com/Hardcoreprawn/post-scarcity-rts/issues/4) — Flowfield/formation pathfinding for mass movement

---

## Vertical Slice Gate (Required to Enter Phase 3)

This is the hard ship gate for any external demo or publisher review.

- [ ] One faction fully playable from data (no hardcoded spawns)
- [ ] Core commands complete and reliable (Attack-Move, Hold, Patrol, Stop)
- [ ] Visual readability baseline met (health bars, selection, hit feedback)
- [ ] Determinism checks in CI (hash divergence fails build)
- [ ] One polished 2-player map with resource flow and expansions

---

## Phase 3: The Vertical Slice (Weeks 16-22) 📅 PLANNED

**Goal:** A fully playable, polished single-player experience with **one faction** (Continuity Authority). This is our proof-of-concept for publishers/players.

### 3.1 Technical Wiring (GDD Alignment)

- [ ] **Data Wiring:** Connect FactionData RON files to actual Unit/Building spawning.
- [ ] **No Hardcoded Spawns:** All scenario/unit spawns are driven by data definitions.
- [ ] **Fog of War (Prototype):** Basic explored/unexplored/visible states.
- [ ] **Line of Sight:** Units cannot shoot what they cannot see.

### 3.2 Visual Identity

- [ ] **Sprite Audit:** Replace "programmer art" with cohesive placeholders.
- [ ] **Silhouette Pass:** Faction-specific silhouettes at strategic zoom (deferred from Phase 2.7.1).
- [ ] **Asset Pipeline:** Define automated import process & sprite atlas tools.
- [ ] **Animation:** Basic Idle (breathing) / Move (bobbing) / Attack (recoil) states.
- [ ] **Audio:** Basic SFX for specific unit types (gunfire, engines).
- [ ] **Terrain:** Basic tileset variation (not just flat color).
- [ ] **Ability/Hit VFX:** Ability activation and critical-hit VFX pass (deferred from Phase 2.7.1).

### 3.3 The Slice Content

- [ ] **Map:** One polished 2-player map with distinct terrain functionality.
- [ ] **Faction:** Continuity Authority fully playable (Tier 1-3).
- [ ] **AI:** "Standard" AI personality that uses the full tech tree.
- [ ] **Tutorial:** A 5-minute onboarding flow (text/triggers).

### 3.4 Infrastructure Catch-up

- [ ] **Headless Simulation Runner:** Run games without graphics (critical for balance).
- [ ] **Performance:** Benchmark suite (1k pathfinding, UI redraw).
- [ ] **Determinism CI:** Fail builds if simulation diverges on replay.
- [ ] **Asset Pipeline:** Hot-reloading watcher for textures/data ([Details](design/systems/ai-testing-and-toolchain.md)).

**Exit Criteria:**

- [ ] "Vertical Slice" build labeled and archived.
- [ ] Automated headless simulation can run 100 battles < 1 min.
- [ ] Factions data driving gameplay 100%.

---

## Phase 4: Faction Rollout (Weeks 23-30)

**Goal:** Expand to full roster using the infrastructure built in Phase 3.

### 4.0 Process & Document Health

- [ ] **Documentation:** Quarterly Architecture vs Implementation review.
- [ ] **AI Gym:** Automated gameplay evaluation loop ([Details](design/systems/ai-testing-and-toolchain.md)).

### 4.1 Faction Batch A: Collegium & Tinkers' Union

- [ ] Implement unique mechanics (Drone Swarms / Scrap Salvage).
- [ ] Asset rollout for Batch A.
- [ ] Balance pass: 3-way matchup.

### 4.2 Faction Batch B: Sculptors & Zephyr Guild

- [ ] Implement unique mechanics (Regeneration / Flight).
- [ ] Asset rollout for Batch B.
- [ ] Balance pass: 5-way matchup.

### 4.3 Automated Balance Tuning

- [ ] Time-to-Kill (TTK) Matrix validation.
- [ ] Regression testing using Headless Runner.
- [ ] Cost derivation formula (prevent "cheap OP" units).

**Exit Criteria:**

- [ ] All 5 factions playable.
- [ ] Automated balance tests passing (+/- 5% win rate deviation).

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
- Procedural Map Generation.
- Co-op Survival Mode.
