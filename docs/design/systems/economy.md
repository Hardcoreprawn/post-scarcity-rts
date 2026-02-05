# Economy System

## Overview

The economy in Post-Scarcity RTS is built around a central tension: feedstock is abundant, but *control* of feedstock is not. Each faction interacts with resources differently, creating asymmetric economic gameplay.

---

## Resources

### Primary Resource: Feedstock

Feedstock is raw organic matter that can be converted into anything through fabrication technology.

| Property | Value |
| -------- | ----- |
| Starting Amount | 500 |
| Node Sizes | Rich (10,000), Standard (5,000), Depleted (1,000) |
| Base Gather Rate | 30/s per harvester |
| Refining Ratio | 1:1 (varies by faction) |

### Secondary Resources

| Resource | Faction | Generation | Usage |
| -------- | ------- | ---------- | ----- |
| Essence | Sculptors | Clinics, organic harvesting | Modifications, Tier 3 |
| Salvage | Tinkers' Union | Battlefield wrecks | Bonus income, repairs |
| Trade Income | Zephyr Guild | Trade routes | Passive gold generation |

### Tiered Resources (Universal)

> **TODO:** Implement T2/T3 resources in game code.

Beyond faction-specific resources, all factions require **universal rare resources** for top-tier units:

| Tier | Name | Source | Used For |
| ---- | ---- | ------ | -------- |
| T1 | Feedstock | Permanent base nodes, field deposits | Basic units, buildings, economy |
| T2 | **Exotics** | Contested field nodes only (mid-map) | T2 units, upgrades, advanced buildings |
| T3 | **Cores** | 1-2 rare deposits per map (dangerous locations) | T3 units, superweapons, faction pinnacles |

**Lore:**

- **Exotics** — Rare elements that can't be synthesized (platinum group metals, lanthanides). Hoards from old-world tech facilities, recycled from pre-collapse machinery.
- **Cores** — Fragments of pre-collapse AI substrates or quantum computing remnants. Irreplaceable relics that enable the most advanced fabrication.

**Strategic Intent:**

- Feedstock is safe but only gets you basic forces
- Exotics force early/mid-game expansion and conflict
- Cores force late-game fights over high-value objectives
- No turtling to victory — must contest the map for T2/T3

### Supply (Population Cap)

All factions share supply mechanics:

| Building | Supply Provided |
| -------- | --------------- |
| Main Base | 10 |
| Supply Structure | +10 each |
| Max Supply | 200 |

---

## Universal Salvage

All factions can recover resources from battlefield wrecks. This creates tactical decisions around controlling wreck-littered areas.

### Wreck Generation

When a unit dies, its wreck persists on the battlefield:

| Unit Tier | Salvage Value | Wreck Lifetime |
| --------- | ------------- | -------------- |
| Tier 1 | 25% of cost | 10 seconds |
| Tier 2 | 25% of cost | 10 seconds |
| Tier 3 | 25% of cost | 10 seconds |
| Building | 15% of cost | 15 seconds |

**Example:** A Tier 2 Guardian Mech (300 cost) leaves a wreck worth 75 feedstock.

### Salvage Collection

Any **battleline unit** (infantry, mech, vehicle — not harvesters, scouts, or drones) can collect from nearby wrecks:

| Property | Value |
| -------- | ----- |
| Collection Radius | 100 units |
| Auto-collect | Yes (when idle and in range) |
| Interrupts on | Enemy contact (combat priority) |

**Tier-Scaled Collection Rate:**

| Collector Tier | Rate | Time for 50-value wreck |
| -------------- | ---- | ----------------------- |
| Tier 1 | 1/tick | 50 ticks (~0.8 sec) |
| Tier 2 | 2/tick | 25 ticks (~0.4 sec) |
| Tier 3 | 4/tick | 12 ticks (~0.2 sec) |

Higher-tier units salvage faster, giving them utility beyond raw combat power.

### Salvage Behavior

1. **Unit stops moving** while salvaging (vulnerability window)
2. **Cannot attack** until salvage complete or interrupted
3. **Combat interrupts salvage** — unit will engage if enemies approach
4. **Partial collection** — if interrupted, collected resources are kept, remaining stay in wreck
5. **Contested wrecks** — both sides can race to collect

### Tactical Implications

- **Push to deny salvage** — Secure kill zones to collect enemy wrecks
- **Hold ground after battles** — Recoup losses by salvaging both sides
- **Tier advantage** — High-tier units clear battlefields 4x faster
- **Risk vs reward** — Salvaging units are vulnerable; protect them or lose tempo

### Faction Bonuses

The Tinkers' Union has **enhanced salvage** as a faction identity:

| Tinkers Bonus | Effect |
| ------------- | ------ |
| Salvage value | +50% (37.5% of cost instead of 25%) |
| Collection rate | +100% (double speed at all tiers) |
| Salvage Rig | Can also collect, treats all wrecks as +25% value |

---

## Harvesting by Faction

### The Continuity Authority

```text
[Feedstock Node] → [Cargo Hauler] → [Heavy Refinery] → [Storage/Production]
```

**Harvesters:** Cargo Hauler

- Slow, heavily armored
- Capacity: 200 feedstock
- Cost: 100
- Can defend itself

**Economic Buildings:**

- **Heavy Refinery** (200) — +50% processing efficiency
- **Distribution Hub** (150) — +2 production queue slots
- **Fortified Depot** (100) — Stores feedstock, stockpile bonuses

#### Unique Mechanic: Stockpiling

| Stockpile Level | Bonus |
| --------------- | ----- |
| 500+ | +5% production speed |
| 1000+ | +10% production speed |
| 2000+ | +15% production speed, unlock elite units |

---

### The Collegium

```text
[Feedstock Node] → [Harvester Swarm] → [Micro-Refinery] → [Distributed Network]
```

**Harvesters:** Harvester Swarm (4 drones)

- Very fast, fragile
- Capacity: 50 each (200 total)
- Cost: 75 for 4
- No weapons

**Economic Buildings:**

- **Micro-Refinery** (100) — Small, efficient, stackable
- **Data Node** (75) — Network bonuses, tech speed
- **Fabricator Array** (150) — Mass production

#### Unique Mechanic: Network Scaling

| Adjacent Buildings | Bonus |
| ------------------ | ----- |
| 2 | +5% efficiency |
| 3 | +10% efficiency |
| 4+ | +15% efficiency |

**Open-Source Discount:**

- Each building type after the first costs 10% less
- 5th Micro-Refinery costs only 60 instead of 100

---

### The Tinkers' Union

```text
[Feedstock Node] → [Salvage Rig] → [Workshop] → [Production]
        +
[Battlefield] → [Salvage Rig] → [Bonus Resources]
```

**Harvesters:** Salvage Rig

- Medium speed, armed
- Capacity: 100 feedstock
- Cost: 75
- Can harvest wrecks

**Economic Buildings:**

- **Workshop Hall** (Starting) — Mobile HQ
- **Salvage Post** (75) — Harvester return, mobile
- **Parts Depot** (100) — Repairs, upgrades
- **Trade Post** (150) — Converts salvage to refined

#### Unique Mechanic: Salvage

| Wreck Type | Salvage Value |
| ---------- | ------------- |
| Infantry | 10 |
| Vehicle | 30 |
| Mech | 75 |
| Building | 50-200 |

Salvage is *bonus* — doesn't replace normal harvesting but supplements it.

---

### The Sculptors

```text
[Patronage Network] → [Clinics] → [Steady Income]
        +
[Essence Collectors] → [Organic Matter] → [Essence]
```

**Harvesters:** Essence Collector (45)

- Elegant autonomous unit
- Gathers organic matter from terrain
- Returns to nearest Clinic or Salon
- Non-threatening appearance

**Economic Buildings:**

- **Clinic** (100) — Generates patronage income, heals nearby units
- **Harvester Bay** (75) — Produces Essence Collectors
- **Salon** (150) — Expansion structure, extends build radius

#### Unique Mechanic: Patronage Economy

| Clinics | Income |
| ------- | ------ |
| 1 | 15/s |
| 2 | 35/s |
| 3 | 60/s |
| 4+ | +30/s each |

---

### The Zephyr Guild

```text
[Trade Depot A] ←→ [Trade Depot B] → [Passive Trade Income]
        +
[Enemy Harvester] → [Piracy] → [Stolen Resources]
```

**Harvesters:** Sky Collector

- Flying, fast, fragile
- Capacity: 75
- Cost: 60
- Ability: Pirate

**Economic Buildings:**

- **Trade Depot** (100) — Establishes trade routes
- **Sky Refinery** (150) — Airborne processing
- **Smuggler's Den** (125) — Piracy bonuses

#### Unique Mechanic: Trade Routes

| Route Length | Income/s | Risk |
| ------------ | -------- | ---- |
| Local | 5 | Low |
| Regional | 10 | Medium |
| Cross-map | 20 | High |

Trade ships travel routes and can be intercepted.

---

## Resource Flow Comparison

```text
         Gather Speed   Capacity   Safety   Special
         ───────────────────────────────────────────
Direct.  ████░░░░░░     ██████████ ████████ Stockpile bonus
Colleg.  ██████████     ████░░░░░░ ████░░░░ Network bonus
Mechan.  ██████░░░░     ██████░░░░ ██████░░ Salvage bonus
Bio-Sov  ██░░░░░░░░     N/A        ████████ Creep passive
Zephyr   ████████░░     ████░░░░░░ ████░░░░ Trade + piracy
```

---

## Economic Pressure

### Resource Depletion

Nodes deplete over time:

- Rich → Standard → Depleted → Empty
- Depletion time: ~5 minutes active harvesting
- Forces expansion and map control

### Expansion Timing

| Game Phase | Expected Expansions |
| ---------- | ------------------- |
| Early (0-5 min) | 0-1 |
| Mid (5-15 min) | 1-3 |
| Late (15+ min) | 3-5 |

### Harassment Value

| Target | Economic Impact |
| ------ | --------------- |
| Kill 1 harvester | ~30s production loss + replacement cost |
| Kill 3 harvesters | ~90s production loss, major setback |
| Destroy refinery | ~60s rebuild, lose stockpile bonus |
| Destroy expansion | Several minutes of wasted investment |

---

## Production Costs

### Cost Categories

| Tier | Feedstock Range | Build Time |
| ---- | --------------- | ---------- |
| Tier 1 | 25-75 | 5-15s |
| Tier 2 | 100-300 | 20-40s |
| Tier 3 | 400-800 | 45-90s |
| Buildings | 50-500 | 15-60s |

### Cost Efficiency

Units have "efficiency ratings" for balance:

```text
Efficiency = (Combat Value) / (Cost + Build Time Value)
```

Tier 1 units are most efficient per-resource, but Tier 3 units are most efficient per-supply.

---

## Economic Counters

| Strategy | Counter |
| -------- | ------- |
| Fast expand | Early aggression, deny territory |
| Turtle economy | Harassment, cut supply lines |
| Harassment focus | Defend workers, build static defense |
| All-in rush | Survive, out-economy after |

---

## Balance Principles

1. **No faction should have strictly better economy** — trade-offs matter
2. **Early harassment should be impactful but not game-ending**
3. **Expansion should be rewarded but risky**
4. **Late-game should favor the player with better macro**

---

## Related Documents

- [Main GDD](../gdd.md)
- [Combat System](./combat.md)
- [Faction Documents](../factions/)
