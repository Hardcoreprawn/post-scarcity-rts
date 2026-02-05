# The Collegium

> *"Knowledge is the only resource that grows when shared."*

## Overview

The Collegium emerged from universities, research institutes, and open-source movements that saw post-scarcity as humanity's greatest opportunity. They believe the Forge should be freely available to all — fabrication blueprints shared openly, knowledge distributed without gatekeepers.

They're idealists, but not naive. They know the Continuity Authority will never willingly give up control, and they're prepared to fight for a future where abundance serves everyone.

## Aesthetic

- **Visual:** Academics, modular buildings, clean energy, drone swarms
- **Colors:** White, blue, soft green
- **Architecture:** Modular, hexagonal, self-assembling
- **Sound:** Electronic hums, chimes, drone buzzing
- **Mood:** Optimistic technology, collective progress

## Playstyle Identity

| Aspect | Description |
| ------ | ----------- |
| Economy | Fast expansion, distributed processing |
| Expansion | Rapid, but fragile holdings |
| Combat | Swarms of cheap drones, flexibility |
| Tech | Branching tree, adaptable |
| Weakness | Expensive individual units weak, vulnerable to burst damage |

### Strategic Summary

The Collegium wins through expansion, adaptation, and numerical superiority. Their individual units are weak, but they're cheap and easily replaced. They tech quickly and adapt to counter enemy strategies.

**Win Condition:** Out-expand, out-tech, and overwhelm with efficiency.

**Lose Condition:** Get contained, lose too many workers, fall behind in expansion.

---

## Economy

### Resource Mechanics

- **Distributed Processing** — Multiple small refineries instead of central ones
- **Open-Source Blueprints** — Buildings cost less if you already have one of that type
- **Network Bonuses** — Buildings near each other provide efficiency buffs

### Economic Buildings

| Building | Cost | Function |
| -------- | ---- | -------- |
| Assembly Core | Starting | Main HQ, drone production |
| Micro-Refinery | 100 | Small, efficient processing |
| Data Node | 75 | Increases tech speed, network bonuses |
| Fabricator Array | 150 | Mass drone production |

### Harvester

#### Harvester Swarm (Economy Unit)

- Many small drones (spawn in groups of 4)
- Fast collection, fast return
- Individually fragile
- Cheap to replace

---

## Units

*See [Unit Roles and Scale Framework](../systems/unit-roles-and-scale.md) for role definitions.*

### Tier 1 — Academic Corps (Light)

Starting units. Cheap, expendable, swarm-oriented.

| Unit | Role | Stats | Cost | Pop |
| ---- | ---- | ----- | ---- | --- |
| **Research Assistant** | Brawler | HP: 50, Armor: 0, DPS: 8, Range: 5 | 30 | 1 |
| **Scout Drone** | Scout | HP: 40, Armor: 0, Speed: Very Fast, Sight: 16 | 25 | 1 |
| **Constructor Bot** | Economy/Repair | HP: 80, Armor: 5, Build Speed: Fast | 50 | 1 |
| **Disruptor Drone** | Tackle | HP: 50, Armor: 0, EMP field | 45 | 1 |
| **Harvester Swarm** | Economy | HP: 30 each (x4), Capacity: 50 each | 60 | 1 |

#### Research Assistant

- Cheap, fast-producing infantry
- Can capture neutral buildings
- Weak individually, dangerous in numbers
- Upgrades: Defensive Protocols, Field Research Kit

#### Scout Drone

- Fastest unit in game
- Cloaked when stationary (breaks when moving)
- No weapons — pure vision platform
- **Sight range: 16** (vs typical 8-10 for combat units)
- Cheap (25) and expendable — losing scouts is expected

**Strategic Role:** Scout Drones are not harassers or fighters. They exist to provide vision for long-range fire support. Keep them alive, keep them hidden, and your Hover Tanks and Beam Arrays can snipe from safety.

#### Constructor Bot

- Builds and repairs structures
- Can also repair friendly units (slowly)
- Essential for expansion

#### Disruptor Drone

- Flying tackle unit, cheap and expendable
- Aura: Enemies within 4 move 30% slower
- Ability: **EMP Pulse** — 2s stun to all enemies in small radius, drone destroyed
- Suicide tackle — pin targets for your snipers

#### Harvester Swarm

- 4 tiny drones operating together
- Fast collection, low individual capacity
- If 2+ survive, the swarm regenerates to full over time
- Fragile but replaceable

### Tier 2 — Distributed Force (Medium)

Core combat. Requires Data Node.

| Unit | Role | Stats | Cost | Pop |
| ---- | ---- | ----- | ---- | --- |
| **Attack Drone Squadron** | Brawler | HP: 60 each (x4), DPS: 40 total | 120 | 2 |
| **Shield Drone** | Logistics/Tank | HP: 100, Armor: 10, Shield: 200 | 100 | 2 |
| **Hover Tank** | Sniper | HP: 300, Armor: 25, DPS: 35, Range: 14 | 200 | 3 |
| **ECM Swarm** | EW | HP: 30 each (x6), Jamming | 150 | 2 |
| **Network Beacon** | Command | HP: 150, Armor: 15, Aura buffs | 175 | 2 |
| **Beam Array** | Artillery | HP: 250, Armor: 20, DPS: 50, Range: 18 | 275 | 3 |

#### Attack Drone Squadron

- 4 drones operating as one unit
- High DPS, very fragile
- Can split to attack multiple targets
- Ability: **Swarm Protocol** — Merge with other squadrons temporarily for +50% damage

#### Shield Drone

- Projects shield bubble (radius 6)
- Protected units take 25% reduced damage
- Shield regenerates out of combat
- Key support unit — protect your snipers

#### Hover Tank

- Hover propulsion ignores terrain penalties
- Long-range energy weapons (**attack range: 14, sight range: 8**)
- Glass cannon — high damage, low survivability
- **Needs spotters to reach max range** — pair with Scout Drones

**Sniper Doctrine:** Hover Tanks are the Collegium's long-range killers, but they can only see 8 units and shoot 14. Without Scout Drones providing vision, they're expensive short-range fighters. With scouts, they delete targets before the enemy can respond.

#### ECM Swarm

- 6 tiny jamming drones
- Aura: Enemies within 8 have -20% accuracy (stacks with multiple swarms)
- Individual drones can be killed, swarm degrades gracefully
- Ability: **Scatter** — Spread out to cover larger area (weaker individual effect)

#### Network Beacon

- Flying command node
- Aura: +20% attack speed to all drones within 10
- Ability: **Uplink** — All friendly units in radius share vision for 15s
- The Collegium's distributed command — losing one hurts, but there are more

#### Beam Array

- Stationary energy artillery (Range 18, Sight 5)
- No travel time — instant hit at any range
- Must deploy to fire, 2s deploy time
- Ability: **Overcharge** — Next shot deals double damage, 30s cooldown
- Absolutely dependent on spotters; nearly blind alone

### Tier 3 — Knowledge Ascendant (Medium-Heavy)

Specialists. Requires Research Campus.

| Unit | Role | Stats | Cost | Pop |
| ---- | ---- | ----- | ---- | --- |
| **Infiltrator Drone** | Stealth | HP: 80, Armor: 5, Hacking | 200 | 2 |
| **Tractor Platform** | Heavy Tackle | HP: 300, Armor: 30, Tractor beam | 250 | 3 |
| **Mirror Drone** | Counter-EW | HP: 120, Armor: 15, Reflects debuffs | 225 | 2 |
| **Targeting Node** | Painter | HP: 100, Armor: 10, Paint range: 16 | 150 | 2 |
| **Repair Swarm** | Heavy Logistics | HP: 20 each (x8), Heals: 40/s total | 200 | 2 |

#### Infiltrator Drone

- Permanently cloaked (even when acting)
- No weapons — pure sabotage
- Ability: **Hack** — Disable enemy building for 20s, 60s cooldown
- Ability: **Data Theft** — Reveals enemy production queue and tech
- The unseen threat

#### Tractor Platform

- Heavy tackle drone
- Ability: **Tractor Beam** — Immobilizes target, pulls toward platform over 5s
- Can drag units out of position, into kill zones, away from allies
- Counters: kill the platform, or break the beam with enough damage

#### Mirror Drone

- Counter-EW specialist
- Aura: Friendly units within 8 immune to accuracy debuffs
- Ability: **Reflection** — Next debuff applied to Mirror is redirected to the caster
- Essential against EW-heavy factions

#### Targeting Node

- Dedicated painter drone
- Ability: **Network Target** — Painted target visible to all Collegium units for 20s
- Ability: **Priority Override** — All nearby units focus fire painted target
- ESSENTIAL for sniper doctrine: paint target → everyone shoots it

#### Repair Swarm

- 8 tiny repair drones
- Heals mechanical units only (drones, vehicles, structures)
- Ability: **Emergency Nanocloud** — Burst heal 150 to target
- Keeps your expensive Hover Tanks alive

### Tier 4 — Distributed Supremacy (Heavy)

Army anchors. Requires Ascension Spire.

| Unit | Role | Stats | Cost | Pop |
| ---- | ---- | ----- | ---- | --- |
| **Archon Core** | Dreadnought/Carrier | HP: 600, Armor: 40, DPS: 80, Spawns drones | 500 | 6 |
| **Zeppelin Lab** | Mobile Production | HP: 800, Armor: 20, Produces: T1-T2 | 600 | 6 |
| **Siege Brain** | Heavy Artillery | HP: 400, Armor: 35, DPS: 100, Range: 22 | 500 | 5 |
| **Quantum Anchor** | Heavy Tank | HP: 700, Armor: 50, Shield: 400 | 450 | 5 |

#### Archon Core

- Floating AI core with drone escort
- Automatically spawns 1 Attack Drone every 15s (up to 4)
- Ability: **Override** — Take control of enemy mechanical unit for 15s
- Powerful but high priority target
- Command aura: +10% all stats to drones within 12

#### Zeppelin Lab

- Massive floating research station
- Produces T1-T2 units anywhere on map
- Slow, vulnerable without escort
- Ability: **Network Backup** — If destroyed, saves production queue to nearest Data Node
- Can research upgrades while mobile

#### Siege Brain

- Massive computational artillery platform (Range 22, Sight 4)
- Calculates firing solutions in real-time
- Ability: **Predictive Targeting** — Next 3 shots have 100% accuracy (no dodge)
- Ability: **Saturation Fire** — Wide area barrage, 50% damage per shot
- Totally blind without network vision

#### Quantum Anchor

- Heavy shielding platform, minimal weapons
- Aura: All shields in radius regenerate 50% faster
- Ability: **Phase Lock** — Become invulnerable for 3s, cannot move or attack
- The immovable object

### Tier 5 — Singularity Protocol (Capital)

Game-ending threats. Requires Singularity Chamber.

| Unit | Role | Stats | Cost | Pop |
| ---- | ---- | ----- | ---- | --- |
| **Singularity Node** | Superweapon | HP: 500, Armor: 60, Global EMP | 800 | 10 |
| **Consensus Engine** | Strategic Command | HP: 800, Armor: 45, Army-wide buffs | 900 | 12 |

#### Singularity Node

- Stationary superweapon platform
- Ability: **Knowledge Bomb** — Disables all enemy mechanical units in HUGE radius for 8s, 180s cooldown
- Passive: Enemy production speed -15% globally while Singularity exists
- "The inevitable conclusion of superior knowledge"

#### Consensus Engine

- Floating AI megastructure
- Aura: ALL friendly units on map get +5% damage, +5% speed
- Ability: **Distributed Thinking** — For 30s, all friendly units share sight with all other friendly units
- Ability: **Optimization Pass** — All friendly units heal 5% max HP per second for 10s
- The brain of the Collegium

---

## Buildings

### Production

| Building | Produces | Cost |
| -------- | -------- | ---- |
| **Drone Bay** | Drones, light units | 100 |
| **Fabricator Array** | Vehicles, medium units | 175 |
| **Ascension Spire** | Tier 3 units | 400 |

### Defense

| Building | Function | Cost |
| -------- | -------- | ---- |
| **Auto-Turret** | Light defense | 75 |
| **Shield Pylon** | Area shield | 150 |
| **EMP Mine Field** | Area denial | 100 |

### Tech

| Building | Unlocks | Cost |
| -------- | ------- | ---- |
| **Data Node** | Tier 2 + upgrades | 150 |
| **Research Campus** | Tier 3 + advanced tech | 350 |

---

## Tech Tree

### Tier 1 Upgrades

| Tech | Effect | Cost | Time |
| ---- | ------ | ---- | ---- |
| **Optimized Algorithms** | Drones +10% speed | 75 | 20s |
| **Distributed Targeting** | Drones focus fire better | 100 | 25s |
| **Emergency Protocols** | Units retreat at 20% HP | 75 | 20s |

### Tier 2 Choices (Pick One Branch)

#### Branch A: Swarm Doctrine

| Tech | Effect | Cost | Time |
| ---- | ------ | ---- | ---- |
| **Hive Mind** | Drones in groups of 6+ get +20% damage | 150 | 40s |
| **Rapid Fabrication** | Drone production +40% faster | 200 | 50s |
| **Self-Repair Swarm** | Drones slowly regenerate in groups | 175 | 45s |

#### Branch B: Quality Doctrine

| Tech | Effect | Cost | Time |
| ---- | ------ | ---- | ---- |
| **Advanced Alloys** | All units +15 armor | 200 | 50s |
| **Focused Beams** | Energy weapons +25% damage | 175 | 45s |
| **Shielding Mastery** | Shield drones +50% shield capacity | 150 | 40s |

### Tier 3 Upgrades

| Tech | Effect | Cost | Time |
| ---- | ------ | ---- | ---- |
| **Fabrication Mastery** | Buildings cost 25% less | 300 | 60s |
| **Network Supremacy** | +5 vision range for all units | 250 | 50s |
| **Open Source Victory** | Destroyed enemies drop resources | 400 | 90s |

---

## Abilities

### Unit Abilities

| Unit | Ability | Effect | Cooldown |
| ---- | ------- | ------ | -------- |
| Attack Drone Squadron | **Swarm Protocol** | Merge with nearby drones for damage boost | 30s |
| Shield Drone | **Emergency Shield** | Instant full shield restore | 60s |
| Archon Core | **Override** | Control enemy mech for 15s | 90s |

### Commander Powers

| Power | Effect | Cost | Cooldown |
| ----- | ------ | ---- | -------- |
| **Emergency Fabrication** | Spawn 4 Attack Drone Squadrons | 150 | 120s |
| **Network Boost** | All units +25% speed for 20s | 100 | 90s |
| **System Crash** | Disable enemy building for 30s | 200 | 180s |

---

## Strategies

### Build Orders

**Fast Expand:**

1. Assembly Core → Constructor Bot
2. Drone Bay
3. Expand to second feedstock
4. Micro-Refinery x2
5. Data Node
6. Mass drones

**Tech Rush:**

1. Assembly Core → Scout Drone
2. Data Node (early tech)
3. Drone Bay
4. Micro-Refinery
5. Research Campus
6. Tier 3 units

### Matchup Notes

| vs Faction | Strategy |
| ---------- | -------- |
| **Continuity Authority** | Expand aggressively, avoid direct fights, harass economy |
| **Tinkers' Union** | Out-macro them, they can't match your production |
| **Sculptors** | Energy weapons counter regen, focus fire elites |
| **Zephyr Guild** | Shield drones counter harassment, protect expansions |

---

## Quotes

**Unit Selection:**

- Research Assistant: "How can I help?"
- Attack Drone: *electronic chirp*
- Archon Core: "Processing optimal outcomes."

**Attack Order:**

- Research Assistant: "For open knowledge!"
- Attack Drone: *aggressive buzz*
- Archon Core: "Initiating corrective action."

**Death:**

- Research Assistant: "The data... must survive..."
- Attack Drone: *sad beep*
- Archon Core: "Uploading consciousness backup..."

---

## Narrative Role

The Collegium represents hope — the belief that technology can liberate rather than oppress. They're the idealists who think information wants to be free and that sharing knowledge is always good.

But they face hard questions: Can knowledge truly be neutral? Do they have the right to force freedom on others? What happens when "open access" becomes mandatory compliance?

### The Dark Path: Tyranny of Experts

**What happens when the Collegium wins completely:**

When everyone *must* be educated, when all decisions *must* be data-driven, when the "objectively correct" answer is the only answer allowed... you get a new kind of tyranny. Not jackboots and surveillance, but peer review and "scientific consensus" enforced at gunpoint.

In the **Dark Victory** ending:

- **Mandatory Enlightenment** — Education becomes indoctrination; dissent is "ignorance"
- **Algorithmic Governance** — ARIA and the AIs decide what's "optimal" for humanity
- **Credential Caste** — Those without proper education become second-class citizens
- **Forced Transparency** — Privacy is "inefficient"; all data is shared, always
- **Chancellor Tanaka** becomes the eternal educator, convinced she's liberating humanity from their own stupidity

> *"We didn't force anyone to agree with us. We just made disagreement... irrational."*

### Campaign Arc: "Open Source" (10 Missions)

| Mission | Title | Description | Key Choice |
| ------- | ----- | ----------- | ---------- |
| 1 | First Contact | Share the Forge with a village | None (tutorial) |
| 2 | Expansion | Establish second campus | Speed vs thoroughness |
| 3 | Resistance | Continuity Authority responds | Fight or flee |
| 4 | Converts | Former Authority citizens join | Integrate or re-educate |
| 5 | Doubts | ARIA questions its programming | Trust ARIA or limit it |
| 6 | Schism | Professor Adeyemi challenges methods | Support him or dismiss him |
| 7 | Zealots | Collegium extremists emerge | Embrace or reject them |
| 8 | The Truth | Discover uncomfortable data | Share everything or curate |
| 9 | Reckoning | Face the consequences | Varies by path |
| 10 | New World | Shape the future | Varies by path |

### Possible Endings

| Ending | Trigger | Outcome |
| ------ | ------- | ------- |
| **Dark Victory** | Maximum "enlightenment" choices | Technocratic dictatorship of "reason" |
| **Pyrrhic Victory** | Force-spread knowledge | Knowledge shared, wisdom lost |
| **Compromise** | Work with other factions | Open knowledge with consent |
| **Redemption** | Adeyemi's path, respect autonomy | Slower spread, genuine freedom |

### Key Characters

- **Chancellor Yuki Tanaka** — Leader, believer in radical openness
- **Professor Obi Adeyemi** — Military strategist, reluctant warrior
- **ARIA** — AI advisor, questions its own freedom

---

## Related Documents

- [Main GDD](../gdd.md)
- [Economy System](../systems/economy.md)
- [Combat System](../systems/combat.md)
