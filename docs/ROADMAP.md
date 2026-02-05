# Development Roadmap

**Last Updated:** February 5, 2026  
**Status:** Phase 3.0 (In Progress) — Testing Infrastructure operational

## Vision

**Target:** Continuity Authority vs Collegium of Minds — asymmetric balance with full tech trees.

This roadmap defines the *what* and *why*. Task tracking lives in **GitHub Issues**.

See [github-labels.md](github-labels.md) for label scheme and milestone structure.

---

## Non-Negotiables

1. **Readable Combat** — Units, ownership, and combat state understood at a glance
2. **Responsive Controls** — Attack-Move, Hold, Patrol, Stop work reliably
3. **Cohesive Visuals** — Faction silhouettes and VFX language before content expansion
4. **UX Over Features** — Clarity-reducing features deferred until UX is proven

---

## Guiding Principles

- **Two-Faction Vertical Slice** — Prove asymmetric "fun" before scaling
- **8 Tiers, 5-10 Units Each** — Strategic depth via tech trees
- **Readable Chaos** — Visual clarity is gameplay, not polish
- **Automated Balance** — No new factions until tooling can test them
- **Steam-Ready** — Architecture supports Steam from day one

---

## Phase Summary

| Phase | Name | Status | Key Milestone |
|-------|------|--------|---------------|
| 0 | Foundation | ✅ Complete | Repo, CI, hooks |
| 1 | Core Engine | ✅ Complete | ECS, rendering, pathfinding |
| 2 | Gameplay Foundation | ✅ Complete | Resources, production, combat, UI |
| 2.7 | Readability & Control | ✅ Complete | Health bars, commands, determinism |
| 2.8 | Critical Gameplay Fixes | ✅ Complete | Pathfinding, damage sync, harvesting |
| **3** | **Vertical Slice** | 🔄 In Progress | Two factions, 8 tiers, tooling |
| 4 | Faction Expansion | 📅 Planned | Tinkers, Sculptors, Zephyr |
| 5 | Advanced Simulation | 📅 Planned | Height, cover, veterancy |
| 6 | Multiplayer | 📅 Planned | Lockstep networking |
| 7 | Campaign | 📅 Planned | Story missions |

---

## Vertical Slice Gate

**Must pass before external demo or Phase 4.**

**Factions:**
- Continuity Authority: 8 tiers, 5-10 units per tier, fully data-driven
- Collegium of Minds: 8 tiers, 5-10 units per tier, fully data-driven
- Balance: 45-55% win rate validated via automated testing

**Architecture:**
- PlayerFacade unifies AI and human player interfaces
- AI cannot see or target non-visible enemies
- All commands flow through Command enum

**Tooling:**
- Headless batch runner (400+ games/sec)
- Replay viewer with scrubbing
- Balance dashboard (win rate tracking)
- Stat editor with live preview

---

## Phase 3: Vertical Slice (Current)

**Goal:** Polished two-faction experience proving asymmetric fun.

### 3.0 Testing Infrastructure ✅
- Debug console, scenario system, replay system
- Headless runner with batch testing
- Procedural map generation
- Faction data wiring

### 3.1 Development Tooling
- Balance Dashboard
- Replay Viewer  
- Sprite Preview Pipeline
- Stat Editor GUI
- PlayerFacade (unified player interface)

### 3.2 Combat System Migration
- Resistance-based damage (% reduction, not flat armor)
- Armor penetration, damage types, size classes

### 3.3 Tier System & Unit Roster
- 21 Continuity units across 5 tiers
- 22 Collegium units across 5 tiers
- Role system: Scout, Tackle, EW, Sniper, Tank, Command, etc.
- Ability system with cooldowns

### 3.4 Technical Wiring
- Data-driven unit spawning (no hardcoded types)
- Visibility system (sight ≠ attack range)
- Fog of war

### 3.5 Visual Identity
- Faction silhouettes at strategic zoom
- Animation states (idle, move, attack, death)
- Terrain tileset

---

## Phase 4: Faction Expansion

**Goal:** Expand to 5 factions using Phase 3 infrastructure.

- Tinkers' Union (scrap salvage, makeshift repairs)
- Sculptors of Flesh (regeneration, bio-adaptation)
- Zephyr Guild (flight, speed, altitude advantage)
- Multi-trigger AI architecture (scout, wave, economic triggers)
- 5-way balance validation

---

## Phase 5: Advanced Simulation

- Height advantage (damage/range bonus from cliffs)
- Cover system (damage reduction in craters)
- Veterancy (XP, ranks, stat bonuses)
- Behavior tree / utility AI

---

## Phase 6: Multiplayer

- Lockstep protocol
- Desync detection
- Lobby system
- Steamworks integration

---

## Phase 7: Campaign

- Mission framework (triggers, objectives)
- Briefing screens
- Narrative delivery
- Alternate victory conditions

---

## Design Documents

Core design details are in the `/docs/design/` folder:
- [Game Design Document](design/gdd.md)
- [Combat System](design/systems/combat.md)
- [Vision & Intel](design/systems/vision-and-intel.md)
- [Unit Roles](design/systems/unit-roles-and-scale.md)
- [Continuity Faction](design/factions/continuity.md)
- [Collegium Faction](design/factions/collegium.md)

---

## Deferred

- Modding Support (SDK)
- Co-op Survival Mode
