# Game Design Document

## Post-Scarcity RTS

**Version:** 0.1  
**Last Updated:** January 2026  
**Status:** Pre-Production

---

## Executive Summary

A real-time strategy game set in a near-future world where advanced fabrication technology has made material scarcity obsolete — but power structures remain. Five distinct factions clash over the fundamental question: *"What do we do in a world where scarcity is ending, but power structures remain?"*

---

## Core Vision

### High Concept

Classic base-building RTS with modern systems, featuring five ideologically distinct factions fighting over control of post-scarcity technology. Combines the strategic depth of StarCraft with the asymmetric faction design of Company of Heroes.

### Target Experience

- **Strategic depth** — meaningful decisions at macro and micro levels
- **Faction identity** — each faction plays fundamentally differently
- **Narrative weight** — factions represent real philosophical tensions
- **Competitive viability** — balanced for esports while accessible
- **Satisfying combat** — visceral, readable, and tactically rich

### Pillars

1. **Asymmetric Factions** — Not just different units, but different *ways of playing*
2. **Economic Tension** — Feedstock is abundant but *refined* resources require control
3. **Ideological Warfare** — Victory isn't just military; it's proving your philosophy works
4. **Readable Chaos** — 1000 units fighting should be understandable at a glance

---

## Critical Design & UX Focus (Non-Negotiable)

These are **must-pass gates** for future development. If they are not met, the game will fail in playtests and external reviews regardless of feature depth.

1. **Readability at Strategic Zoom**
   - Unit class, ownership, and health state must be clear within 1–2 seconds.
   - Visual language (silhouettes, outlines, VFX) takes priority over detail.
2. **Responsive RTS Controls**
   - Core commands must be fast, reliable, and predictable (Attack-Move, Hold, Patrol, Stop).
3. **Cohesive Visual Identity**
   - A defined style guide and faction-specific silhouettes/colors are mandatory.
4. **UX Clarity Over Feature Count**
   - Features that introduce cognitive load or reduce clarity are deferred until UX is proven.

---

## Setting

### The World

The year is 2147. Molecular fabrication — "the Forge" — has made material scarcity obsolete. Any raw organic matter ("feedstock") can be converted into anything: food, medicine, weapons, buildings.

But scarcity of *power* remains. Who controls the Forge? Who decides what gets made? Old institutions cling to relevance by controlling feedstock. New movements demand open access. And in the margins, those who reject both forge their own path.

### Tone

- Grounded sci-fi (no aliens, no magic)
- Political without being preachy
- **Hopeful beginnings, dark possibilities** — every faction starts with good intentions
- **Anti-fanaticism** — the core message is that any ideology taken to extremes leads to dystopia
- Industrial aesthetic with organic and high-tech variants

### Thematic Core

Every faction believes they're the good guys. Every faction has a point. And every faction, if they "win" completely, creates a nightmare.

The game explores what happens when reasonable ideas become uncompromising ideologies:

| Faction | Good Intention | Fanatical Extreme |
| ------- | -------------- | ----------------- |
| **Continuity Authority** | Managed transition | Eternal surveillance state |
| **Collegium** | Open knowledge | Technocratic tyranny of "experts" |
| **Tinkers' Union** | Self-sufficiency | Isolationist tribalism |
| **Sculptors** | Bodily autonomy | Aesthetic tyranny, forced "improvement" |
| **Zephyr Guild** | Freedom of movement | Anarcho-capitalist exploitation |

**The player's choices determine which path their faction takes.** Campaigns have multiple endings — and the "complete victory" ending is often the darkest.

### Key Locations

- **The Foundry Cities** — Continuity Authority industrial strongholds
- **The Academies** — Collegium research campuses
- **The Makerspaces** — Tinkers' Union territory, workshop zones
- **The Salons** — Sculptor transformation galleries and clinics
- **The Sky Docks** — Zephyr Guild floating trade platforms

---

## Factions

### Overview

| Faction | Philosophy | Playstyle | Resource Twist |
| ------- | ---------- | --------- | -------------- |
| **Continuity Authority** | Control abundance | Slow, heavy, overwhelming | Stockpiles feedstock, rations production |
| **Collegium** | Share knowledge | Fast expand, drones, adaptive | Open-source buildings, distributed economy |
| **Tinkers' Union** | Build independently | Mobile, modular, creative | Salvage from battlefield, custom builds |
| **Sculptors** | Transform humanity | Elite specialists, bespoke units | Patronage economy, commission system |
| **Zephyr Guild** | Exploit the chaos | Airborne, mobile bases, harassment | Steal enemy resources, trade routes |

*Detailed faction documents in `/docs/design/factions/`*

---

## Core Gameplay Loop

```text
    ┌─────────────────────────────────────────────────┐
    │                                                 │
    ▼                                                 │
┌───────────┐    ┌───────────┐    ┌───────────┐      │
│  Gather   │───►│  Build    │───►│  Combat   │──────┘
│ Feedstock │    │ & Research│    │ & Expand  │
└───────────┘    └───────────┘    └───────────┘
```

### Phase Breakdown

#### Early Game (0-5 minutes)

- Establish base
- Begin feedstock harvesting
- Scout enemy position
- First unit production

#### Mid Game (5-15 minutes)  

- Expand to additional feedstock nodes
- Tech choices define strategy
- First major engagements
- Map control becomes crucial

#### Late Game (15+ minutes)

- Tier 3 units and superweapons
- Resource attrition
- Decisive battles
- Victory push

---

## Economy System

### Primary Resource: Feedstock

Feedstock is organic matter that fuels fabrication. It exists in three forms:

1. **Raw Feedstock** — Harvested from nodes, carried by harvesters
2. **Refined Feedstock** — Processed at refineries, used for advanced units
3. **Essence** — Specialized resource for Sculptors

### Harvesting

| Faction | Harvester Type | Mechanic |
| ------- | ------------- | -------- |
| Continuity Authority | Heavy Harvester | Slow, armored, large capacity |
| Collegium | Harvester Swarm | Many small drones, fast return |
| Tinkers' Union | Salvage Rigs | Harvest + battlefield salvage |
| Sculptors | Essence Collectors | Autonomous, gather organic matter |
| Zephyr Guild | Sky Collectors | Mobile, can steal from enemies |

### Resource Nodes

```text
[Rich Node]           [Standard Node]        [Depleted Node]
████████████          ████████               ████
10,000 feedstock      5,000 feedstock        1,000 feedstock
+50 gather rate       +30 gather rate        +10 gather rate
```

### Economic Pressure

- Nodes deplete over time, forcing expansion
- Controlling center map resources is key
- Harassment disrupts enemy economy
- Different factions value resources differently

---

## Combat System

### Unit Categories

| Category | Role | Examples |
| -------- | ---- | -------- |
| Infantry | Cheap, flexible, capture points | Jackboots, Academics, Scavengers |
| Vehicles | Mobile, varied roles | Trikes, APCs, Tanks |
| Mechs | Heavy assault, faction identity | Enforcers, War-Forms, Scrap-Titans |
| Air | Harassment, support | Zeppelins, Drones, Gunships |
| Special | Faction-specific | Harvester Drones, Bio-Constructs |

### Damage Types

```text
┌─────────────────────────────────────────────┐
│           Damage Type Matrix                │
├─────────────┬───────┬───────┬───────┬──────┤
│             │ Light │ Medium│ Heavy │ Air  │
├─────────────┼───────┼───────┼───────┼──────┤
│ Kinetic     │ 100%  │ 75%   │ 50%   │ 75%  │
│ Explosive   │ 75%   │ 100%  │ 125%  │ 50%  │
│ Energy      │ 100%  │ 100%  │ 100%  │ 100% │
│ Bio-Acid    │ 125%  │ 100%  │ 75%   │ 100% │
└─────────────┴───────┴───────┴───────┴──────┘
```

### Combat Mechanics

- **Line of Sight** — Units need vision to attack
- **Fog of War** — Unexplored areas hidden, explored areas show terrain only
- **Cover** — Terrain features provide damage reduction
- **Height Advantage** — High ground gives range and damage bonus
- **Veterancy** — Units gain experience, become more effective

### Control Groups

Standard RTS control groups (0-9) plus:

- Quick-select all units of type
- Quick-select all production buildings
- Camera hotkeys for bases

---

## Tech Trees

### Structure

Each faction has three tiers of technology:

```text
Tier 1: Core Units
   │
   ▼
Tier 2: Advanced Units + First Abilities
   │
   ▼
Tier 3: Elite Units + Superweapons
```

### Tech Buildings

| Faction | Tech Building | Specialty |
| ------- | ------------ | --------- |
| Continuity Authority | Civic Institute | Administrative upgrades |
| Collegium | Research Array | Drone improvements, open-source |
| Tinkers' Union | Workshop Hall | Modular parts, custom builds |
| Sculptors | Archive | Customization trees, form design |
| Zephyr Guild | Sky Foundry | Airship upgrades |

### Research Choices

At each tier, players make exclusive choices that define their strategy:

**Example (Continuity Authority Tier 2):**

- **Option A:** Administrative Expansion — Unlocks bureaucratic buffs, building efficiency
- **Option B:** Enforcement Doctrine — Infantry buffs, suppression abilities

---

## Victory Conditions

### Standard Match

- **Annihilation** — Destroy all enemy structures
- **Domination** — Control 3/5 victory points for 5 minutes
- **Economic Victory** — Accumulate 50,000 refined feedstock

### Campaign Missions

- Mission-specific objectives
- Story-driven goals
- Optional challenges for bonus rewards

---

## Multiplayer & Skirmish

The primary gameplay focus is on **replayable skirmish and multiplayer** content. Campaigns teach and tell stories; skirmish provides endless depth.

### Skirmish Modes (Single & Multiplayer)

| Mode | Players | Description |
| ---- | ------- | ----------- |
| **Standard** | 1v1 to 4v4 | Classic base-building RTS |
| **Annihilation** | 1v1 to 8 FFA | Last faction standing |
| **Domination** | 2v2 to 4v4 | Control victory points |
| **King of the Hill** | 3+ FFA | Hold center, timer counts down |
| **Economic Victory** | Any | First to resource threshold wins |
| **Survival** | 1-4 Co-op | Waves of enemies, endless scaling |
| **Scenario** | Varies | Pre-built asymmetric challenges |

### AI Opponents

Robust AI for single-player skirmish:

| Difficulty | Behavior |
| ---------- | -------- |
| **Recruit** | Slow, predictable, teaches mechanics |
| **Soldier** | Competent, standard builds, minor mistakes |
| **Veteran** | Optimized builds, multi-pronged attacks |
| **Commander** | Near-optimal play, adapts to player |
| **Nightmare** | Cheats slightly (vision), relentless aggression |

AI personalities add variety:

- **Aggressive** — Early pressure, all-in strategies
- **Defensive** — Turtle, tech up, late-game power
- **Economic** — Fast expand, out-scale opponents
- **Harasser** — Constant raids, never commits fully
- **Adaptive** — Changes strategy based on opponent

### Ranked Multiplayer

| Mode | Description |
| ---- | ----------- |
| **1v1 Ranked** | Competitive ladder |
| **2v2 Ranked** | Team ladder |
| **Custom Games** | Any configuration |
| **Co-op vs AI** | Campaign missions, 2 players |
| **FFA** | 3-8 player free-for-all |

### Matchmaking

- ELO-based ranking
- Faction-specific ratings
- Map vetoes
- Rematch option

### Network Architecture

- Lockstep deterministic simulation
- Peer-to-peer for games, server for matchmaking
- Replay saving (command log)

---

## Single Player

### Narrative Philosophy

#### "Every utopia is someone else's dystopia."

Each campaign follows a faction's journey from hopeful idealism to a crossroads where the player must choose between compromise and fanaticism. The campaigns are designed to:

1. **Start sympathetic** — Make players believe in their faction's cause
2. **Introduce doubt** — Show the costs and contradictions
3. **Force hard choices** — Missions with no clean answers
4. **Branch endings** — Player choices lead to light or dark conclusions

The "total victory" ending for each faction is intentionally the darkest — showing what happens when any ideology wins completely. The healthier endings involve compromise, coexistence, or recognizing limits.

### Campaign Structure

Each faction has a **10-12 mission campaign** designed for focused storytelling:

```text
┌─────────────────────────────────────────────────────┐
│                CAMPAIGN ARC (10-12 missions)        │
├─────────────────────────────────────────────────────┤
│  ACT 1: HOPE (Missions 1-3)                         │
│  - Establish faction identity                       │
│  - "We're the good guys" narrative                  │
│  - Tutorial integration                             │
├─────────────────────────────────────────────────────┤
│  ACT 2: CONFLICT (Missions 4-7)                     │
│  - Encounter other factions                         │
│  - Moral complications emerge                       │
│  - Hard choices (affect ending)                     │
├─────────────────────────────────────────────────────┤
│  ACT 3: RECKONING (Missions 8-10/12)                │
│  - Consequences of earlier choices                  │
│  - Branching mission paths                          │
│  - Multiple endings based on player decisions       │
└─────────────────────────────────────────────────────┘
```

### Campaign Endings

Each campaign has **3-4 possible endings**:

| Ending Type | Description |
| ----------- | ----------- |
| **Dark Victory** | Total faction dominance — becomes the new oppressor |
| **Pyrrhic Victory** | Win the war, lose the soul — faction survives but corrupted |
| **Compromise** | Find middle ground — healthiest ending, requires sacrifice |
| **Redemption** | Reject fanaticism — character breaks from faction ideology |

The "best" endings require the hardest choices during gameplay.

### Faction Campaigns

| Campaign | Title | Missions | Core Question |
| -------- | ----- | -------- | ------------- |
| **Continuity Authority** | "Orderly Transition" | 11 | When does protection become oppression? |
| **Collegium** | "Open Source" | 10 | Can knowledge be forced on people? |
| **Tinkers' Union** | "Right to Repair" | 10 | What's the cost of total independence? |
| **Bio-Sovereigns** | "The Bloom" | 12 | How much humanity can you sacrifice for transcendence? |
| **Zephyr Guild** | "Sky Lords" | 10 | Is freedom without responsibility just exploitation? |

### Mission Types

- Traditional RTS missions (base building + objectives)
- Commando missions (small squad, no base)
- Defense missions (waves of enemies)
- Puzzle missions (limited resources, specific solution)

### Difficulty Modes

- Story (easy)
- Commander (normal)
- General (hard)
- Ironclad (brutal + permadeath)

---

## User Interface

### HUD Layout

```text
┌─────────────────────────────────────────────────────────┐
│ [Resources]                              [Menu] [?]     │
├─────────────────────────────────────────────────────────┤
│                                                         │
│                                                         │
│                    GAME WORLD                           │
│                                                         │
│                                                         │
├─────────────┬───────────────────────┬──────────────────┤
│             │                       │                  │
│   MINIMAP   │   SELECTION INFO      │  COMMAND PANEL   │
│             │                       │                  │
└─────────────┴───────────────────────┴──────────────────┘
```

### Key UI Elements

- **Resource Bar** — Feedstock, refined, supply cap
- **Minimap** — Terrain, units, alerts
- **Selection Panel** — Selected unit(s) info, health, abilities
- **Command Panel** — Context-sensitive actions
- **Production Queue** — Building queue status

---

## Audio Design

### Music

- Faction-specific themes
- Dynamic intensity based on combat
- Ambient tracks for exploration

### Sound Design Principles

- **Clarity** — Important sounds cut through
- **Faction Identity** — Each faction sounds different
- **Feedback** — Every action has audio confirmation
- **Spatialization** — 3D audio for unit positions

---

## Art Direction

### Visual Style

- Stylized realism (not photorealistic, not cartoony)
- Strong silhouettes for unit readability
- Faction color schemes:
  - **Continuity Authority:** Navy blue, gray, white
  - **Collegium:** White, blue, green
  - **Tinkers' Union:** Orange, yellow, copper
  - **Bio-Sovereigns:** Purple, green, organic textures
  - **Zephyr Guild:** Gold, sky blue, brass

### Unit Readability

- Distinct silhouettes at strategic zoom
- Faction insignias visible
- Health bars with armor indicators
- Status effect icons

---

## Accessibility

### Visual

- Colorblind modes
- High contrast UI option
- Scalable UI elements
- Screen reader support for menus

### Motor

- Rebindable keys
- Adjustable game speed
- Pause in single player
- Click-and-drag selection options

### Cognitive

- Adjustable difficulty
- Tutorial system
- In-game wiki/encyclopedia
- Suggested actions (optional)

---

## Related Documents

- [Campaign & Skirmish Design](campaign-structure.md)
- [Faction: The Continuity Authority](factions/continuity.md)
- [Faction: The Collegium](factions/collegium.md)
- [Faction: The Tinkers' Union](factions/tinkers.md)
- [Faction: The Bio-Sovereigns](factions/biosovereigns.md)
- [Faction: The Zephyr Guild](factions/zephyr.md)
- [Economy System](systems/economy.md)
- [Combat System](systems/combat.md)
- [Tech Trees](systems/tech-trees.md)
- [Architecture Overview](../architecture/overview.md)
