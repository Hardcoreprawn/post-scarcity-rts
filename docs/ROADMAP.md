# Development Roadmap

**Last Updated:** January 26, 2026
**Status:** Phase 2.7 (In Progress)

## Overview

This roadmap outlines the development phases for Post-Scarcity RTS. Following the [January 2026 Review](review-2026-01-26.md), we have rebaselined to prioritize **gameplay readability**, a **vertical slice**, and **sustainable testing** before expanding to multiple factions.

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

- [ ] Health bars (units & buildings)
- [ ] Selection circles & highlighting
- [ ] Damage feedback (flash on hit)
- [ ] Building placement ghost fix
- [ ] Range indicators (attack/vision) when selected

### 2.7.2 Core Controls (Priority High)

- [ ] Attack-Move (A-Click)
- [ ] Stop (S) / Hold Position (H)
- [ ] Patrol (P)
- [ ] Double-click to select all of type
- [ ] Minimap interaction (click to move/pan)

### 2.7.3 Accessibility Base

- [ ] UI Scaling support
- [ ] Rebindable keys foundation

**Exit Criteria:**

- [ ] A new player can understand combat state at a glance.
- [ ] All standard RTS commands function reliably.
- [ ] Unit tests for command issuance logic.

---

## Phase 3: The Vertical Slice (Weeks 16-22) 📅 PLANNED

**Goal:** A fully playable, polished single-player experience with **one faction** (Continuity Authority). This is our proof-of-concept for publishers/players.

### 3.1 Technical Wiring (GDD Alignment)

- [ ] **Data Wiring:** Connect FactionData RON files to actual Unit/Building spawning.
- [ ] **Fog of War (Prototype):** Basic explored/unexplored/visible states.
- [ ] **Line of Sight:** Units cannot shoot what they cannot see.

### 3.2 Visual Identity

- [ ] **Sprite Audit:** Replace "programmer art" with cohesive placeholders.
- [ ] **Asset Pipeline:** Define automated import process & sprite atlas tools.
- [ ] **Animation:** Basic Idle (breathing) / Move (bobbing) / Attack (recoil) states.
- [ ] **Audio:** Basic SFX for specific unit types (gunfire, engines).
- [ ] **Terrain:** Basic tileset variation (not just flat color).

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
