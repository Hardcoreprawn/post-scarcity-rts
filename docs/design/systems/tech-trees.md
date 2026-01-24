# Tech Trees

## Overview

Tech trees define each faction's progression from basic units to powerful late-game options. Each faction's tech tree reflects their philosophy and creates meaningful strategic choices.

---

## Tech Tree Philosophy

### Design Principles

1. **No Dead Ends** — Every tech path should be viable
2. **Meaningful Choices** — Branching paths create different playstyles
3. **Faction Identity** — Tech reflects faction philosophy
4. **Timing Windows** — Different techs peak at different game phases

### Structure

All factions follow a similar tier structure:

```text
TIER 1 (0-5 min)
├── Basic units
├── Economy buildings
└── First upgrades

TIER 2 (5-15 min)
├── Advanced units
├── CHOICE: Branch A or Branch B
└── Faction-specific mechanics

TIER 3 (15+ min)
├── Elite units
├── Superweapons
└── Final upgrades
```

---

## Tech Building Requirements

### Universal Pattern

| Tech Level | Building Required | Cost | Build Time |
| ---------- | ----------------- | ---- | ---------- |
| Tier 1 | Main Base | Starting | — |
| Tier 2 | Tech Building 1 | 150-200 | 30-45s |
| Tier 3 | Tech Building 2 | 350-500 | 60-90s |

### Faction Tech Buildings

| Faction | Tier 2 Building | Tier 3 Building |
| ------- | --------------- | --------------- |
| Continuity Authority | Civic Institute | Strategic Directorate |
| Collegium | Data Node | Research Campus |
| Tinkers' Union | Parts Depot | Master Workshop |
| Sculptors | Archive | Sanctum |
| Zephyr Guild | Navigation Tower | Admiral's Bridge |

---

## The Continuity Authority Tech Tree

### Continuity Tier 1

**Buildings:**

- Administrative Center (Starting)
- Recruitment Office (150)
- Logistics Depot (200)

**Units:**

- Compliance Officers
- Security Team
- Patrol Vehicle

**Upgrades:**

| Tech | Effect | Cost |
| ---- | ------ | ---- |
| Combat Stims | Infantry +15% damage | 100 |
| Reinforced Plating | Infantry +10 armor | 100 |
| Extended Optics | +2 sight range | 75 |

### Tier 2

**Building:** Civic Institute (200)

**Branch Choice — Pick One:**

#### Branch A: Administrative Expansion

```text
Focus: Efficiency, building bonuses
├── Streamlined Logistics (+25% building efficiency)
├── Extended Jurisdiction (+3 control radius)
└── Emergency Powers (ability)
```

#### Branch B: Enforcement Doctrine

```text
Focus: Infantry, area control
├── Compliance Training (+30% anti-infantry)
├── Authority Presence (-10% enemy morale near units)
└── Occupation Protocol (+50% captured building production)
```

**New Units:**

- Enforcer Mech
- Siege Tank
- APC Transport

### Tier 3

**Building:** Strategic Command (500)

**Unlocks:**

- War Colossus
- Gunship Squadron
- Decimator Battery

**Upgrades:**

| Tech | Effect | Cost |
| ---- | ------ | ---- |
| Reactor Overload | Mech self-destruct ability | 300 |
| Orbital Network | Map reveal every 120s | 400 |
| Total Mobilization | +25% production speed | 500 |

---

## The Collegium Tech Tree

### Collegium Tier 1

**Buildings:**

- Assembly Core (Starting)
- Drone Bay (100)
- Micro-Refinery (100)

**Units:**

- Research Assistant
- Scout Drone
- Constructor Bot

**Upgrades:**

| Tech | Effect | Cost |
| ---- | ------ | ---- |
| Optimized Algorithms | Drones +10% speed | 75 |
| Distributed Targeting | Better focus fire | 100 |
| Emergency Protocols | Auto-retreat at 20% HP | 75 |

### Collegium Tier 2

**Building:** Data Node (150)

**Branch Choice — Pick One:**

#### Branch A: Swarm Doctrine

```text
Focus: Quantity, regeneration
├── Hive Mind (+20% damage in groups of 6+)
├── Rapid Fabrication (+40% drone production)
└── Self-Repair Swarm (group regeneration)
```

#### Branch B: Quality Doctrine

```text
Focus: Individual unit power
├── Advanced Alloys (+15 armor all units)
├── Focused Beams (+25% energy damage)
└── Shielding Mastery (+50% shield capacity)
```

**New Units:**

- Attack Drone Squadron
- Shield Drone
- Hover Tank

### Collegium Tier 3

**Building:** Research Campus (350)

**Unlocks:**

- Archon Core
- Zeppelin Lab
- Singularity Node

**Upgrades:**

| Tech | Effect | Cost |
| ---- | ------ | ---- |
| Fabrication Mastery | Buildings -25% cost | 300 |
| Network Supremacy | +5 vision range | 250 |
| Open Source Victory | Enemies drop resources | 400 |

---

## The Tinkers' Union Tech Tree

### Tinkers Tier 1

**Buildings:**

- Workshop Hall (Starting, Mobile)
- Salvage Post (75, Mobile)

**Units:**

- Field Techs
- Scout Bike
- Modular Rover

**Upgrades:**

| Tech | Effect | Cost |
| ---- | ------ | ---- |
| Quick Repairs | +25% repair speed | 75 |
| Fuel Injection | Vehicles +15% speed | 100 |
| Scrap Efficiency | +25% salvage | 75 |

### Tinkers Tier 2

**Building:** Parts Depot (150)

**Unique: Module System** — Instead of branches, unlock mech components:

| Module Type | Options |
| ----------- | ------- |
| Arm Weapons | Plasma Claw, Gatling Arm, Shield Generator |
| Back Mounts | Rocket Pod, Repair Arm, Jump Jets |

**New Units:**

- Tinker Mech (modular)
- Workshop Truck
- Scout Blimp

### Tinkers Tier 3

**Building:** Master Workshop (300)

**Unlocks:**

- Grand Tinker Mech
- Mobile Workshop
- Parts Swarm

**Upgrades:**

| Tech | Effect | Cost |
| ---- | ------ | ---- |
| Overclock Protocols | Abilities last longer | 250 |
| Field Fabrication | Build at War Rig | 300 |
| Scrap Mastery | +50% salvage, +25% repairs | 350 |

---

## The Sculptors Tech Tree

### Sculptors Tier 1

**Buildings:**

- Atelier (Starting)
- Studio (150)
- Clinic (100)

**Units:**

- Therapist
- Aesthetic
- Courier

**Upgrades:**

| Tech | Effect | Cost |
| ---- | ------ | ---- |
| Enhanced Healing | +25% regen all | 100 |
| Optimized Forms | +10% speed all | 100 |
| Sensory Suite | +2 sight range | 75 |

### Sculptors Tier 2

**Building:** Archive (250)

**Branch Choice — Pick One:**

#### Branch A: Perfection Doctrine

```text
Focus: Individual excellence
├── Hardened Tissue (+15 armor all)
├── Redundant Systems (survive killing blow once)
└── Living Weapons (regenerating ammo)
```

#### Branch B: Harmony Doctrine

```text
Focus: Unit synergy
├── Symbiotic Bond (share 20% healing)
├── Collective Senses (shared vision)
└── Resonance (+50% aura strength)
```

**New Units:**

- Masterwork
- Symbiont
- Dissolution

### Sculptors Tier 3

**Building:** Sanctum (450)

**Unlocks:**

- Paragon
- Muse
- Genesis Chamber

**Upgrades:**

| Tech | Effect | Cost |
| ---- | ------ | ---- |
| Metamorphosis | Units can change role once | 300 |
| Eternal Forms | Dead units yield 50% essence | 350 |
| Apotheosis | Paragons gain second ability | 400 |

---

## The Zephyr Guild Tech Tree

### Zephyr Tier 1

**Buildings:**

- Sky Platform (Starting, Floating)
- Hangar Bay (150, Floating)
- Trade Depot (100)

**Units:**

- Cloudrunner
- Scout Balloon
- Boarding Crew

**Upgrades:**

| Tech | Effect | Cost |
| ---- | ------ | ---- |
| Improved Engines | Air +15% speed | 100 |
| Reinforced Hulls | Airships +20 armor | 100 |
| Extended Range | Trade +25% income | 75 |

### Zephyr Tier 2

**Building:** Navigation Tower (175)

**Branch Choice — Pick One:**

#### Branch A: Pirate Fleet

```text
Focus: Economy disruption
├── Enhanced Piracy (+50% steal)
├── Intimidation (-15% enemy damage near airships)
└── Rapid Boarding (+50% capture speed)
```

#### Branch B: War Fleet

```text
Focus: Direct combat
├── Heavy Ordnance (Bombers +30% damage)
├── Evasive Maneuvers (+25% dodge)
└── Combined Arms (+20% with ground support)
```

**New Units:**

- Corsair Gunship
- Bomber Zeppelin
- Transport Barge

### Zephyr Tier 3

**Building:** Admiral's Bridge (400)

**Unlocks:**

- Sky Dreadnought
- Carrier Platform
- Storm Engine

**Upgrades:**

| Tech | Effect | Cost |
| ---- | ------ | ---- |
| Fleet Admiral | Capital ship aura | 300 |
| Monopoly | Trade +50%, enemy trade -25% | 350 |
| Sky Supremacy | All air +10% speed/damage | 400 |

---

## Tech Timing Reference

### When Techs Become Available

| Game Time | Expected Tech State |
| --------- | ------------------- |
| 0-2 min | Tier 1 basics |
| 2-5 min | Tier 1 complete, Tier 2 building |
| 5-8 min | Tier 2 units, first branch |
| 8-12 min | Full Tier 2, branch complete |
| 12-15 min | Tier 3 building |
| 15+ min | Tier 3 units, superweapons |

### Scouting Tech

Knowing enemy tech path is crucial:

| Building Spotted | Indicates |
| ---------------- | --------- |
| Early Tier 2 building | Tech rush |
| Multiple production | Aggression |
| Defensive structures | Turtle |
| Expansion | Economic play |

---

## Counter-Tech Relationships

### Soft Counters (Timing)

| If Enemy Goes... | Consider... |
| ---------------- | ----------- |
| Fast Tier 2 | Early aggression before it's ready |
| Heavy investment in one path | Counter-tech |
| Early expansion | Timing attack |
| Mass Tier 1 | Tier 2 transition |

### Hard Counters (Unit Types)

| Against... | Use... |
| ---------- | ------ |
| Massed infantry | AOE, suppression |
| Heavy armor | Armor-piercing, abilities |
| Air dominance | AA, anti-air |
| Siege | Mobility, flanking |

---

## Related Documents

- [Main GDD](../gdd.md)
- [Economy System](./economy.md)
- [Combat System](./combat.md)
- [Faction Documents](../factions/)
