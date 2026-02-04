# Combat System

## Overview

Combat in Post-Scarcity RTS emphasizes readability, tactical depth, and faction identity. Battles should be visually clear, strategically meaningful, and feel distinct depending on which factions are fighting.

---

## Design Philosophy (EVE Online Principles)

The combat system draws inspiration from EVE Online's battle-tested balance:

| Principle | Implementation |
| --------- | -------------- |
| **Multiplicative stacking** | Damage reduction is percentage-based, not flat |
| **Diminishing returns** | Resistance capped at 75%; stacking armor has less value |
| **Counter-play for everything** | Armor penetration, size modifiers, damage types |
| **Rock/Paper/Scissors** | Every stat has a trade-off; no stat is universally best |
| **Readable complexity** | Formulas are multiplicative chains, easy to mentally estimate |

**Core Balance Loop:**

```text
Heavy Armor  →  countered by  →  Armor Penetration
Light Speed  →  countered by  →  Tracking weapons, AOE
High DPS     →  countered by  →  Resistance stacking
Long Range   →  countered by  →  Fast closers, indirect fire
```

---

## Core Combat Mechanics

### Damage Calculation

Damage uses multiplicative reduction (EVE Online-style) to avoid flat armor "cliffs":

```text
Effective Resistance = Armor Resistance × (1 - Armor Penetration)
Damage Reduction     = Effective Resistance (capped at 75%)
Final Damage         = Base Damage × Damage Type Modifier × (1 - Damage Reduction)
Minimum Damage       = 1
```

**Why Multiplicative?**  
Flat armor subtraction creates non-linear balance cliffs (e.g., 10 damage - 5 armor = 5; 8 damage - 5 armor = 3 — a 20% damage reduction causes 40% DPS loss). Percentage-based reduction scales smoothly and is easier to balance.

**Design Goals:**  

- All stats have trade-offs (rock/paper/scissors)
- Counter-play exists for every defensive stat
- No single stat dominates at high values

### Damage Types

| Type | Strong Against | Weak Against |
| ---- | -------------- | ------------ |
| **Kinetic** | Light, Air | Heavy |
| **Explosive** | Heavy, Buildings | Air, Spread units |
| **Energy** | All (neutral) | Shielded |
| **Bio-Acid** | Light, regenerating | Heavy |
| **Fire** | Bio-units, Light | Mechanical |

### Damage Matrix

Damage type modifiers stack multiplicatively with resistance:

```text
                    Target Armor Class
                 Light   Medium   Heavy   Air   Building
Kinetic          100%    75%      50%     75%   50%
Explosive        75%     100%     125%    50%   150%
Energy           100%    100%     100%    100%  75%
Bio-Acid         125%    100%     75%     100%  100%
Fire             125%    100%     75%     100%  125%
```

**Example Calculation:**

- Kinetic weapon (base 50 damage) vs Heavy target (50% resistance, 0% pen):
- `50 × 50% (Kinetic vs Heavy) × (1 - 50% resistance) = 50 × 0.5 × 0.5 = 12.5 → 13 damage`

**Strategic Implication:**  
Low damage type modifier + high resistance = very little damage. Players must match damage types OR bring armor penetration.

### Armor Resistance

Armor provides **percentage-based damage reduction** (inspired by EVE Online):

| Armor Class | Base Resistance | Max Resistance (Cap) |
| ----------- | --------------- | -------------------- |
| Light       | 10-20%          | 50%                  |
| Medium      | 25-40%          | 65%                  |
| Heavy       | 45-60%          | 75%                  |

**Key Properties:**

- Resistance is percentage-based, not flat
- Hard cap at 75% prevents invulnerability
- Higher resistance = slower movement (trade-off)
- Armor penetration counters high resistance

### Armor Penetration

Weapons can have armor penetration to counter heavy armor:

| Penetration | Effect | Typical Weapons |
| ------------ | ------ | --------------- |
| None (0%) | Full armor applies | Standard weapons |
| Low (25%) | 25% of resistance ignored | Kinetic rounds |
| Medium (50%) | 50% of resistance ignored | Armor-piercing |
| High (75%) | 75% of resistance ignored | Anti-tank, siege |
| Full (100%) | Ignore all armor | Pure energy, acids |

**Example:**  
Heavy unit with 60% resistance vs weapon with 50% penetration:  
`Effective Resistance = 60% × (1 - 50%) = 30%`  
Attacker deals 70% damage instead of 40%.

### Size Class Modifiers

Weapon tracking vs target size creates strategic trade-offs:

| Weapon Size | vs Light | vs Medium | vs Heavy |
| ----------- | -------- | --------- | -------- |
| Light       | 100%     | 75%       | 50%      |
| Medium      | 75%      | 100%      | 100%     |
| Heavy       | 25%      | 75%       | 100%     |

**Design Intent:**

- Heavy weapons (artillery, siege) can't track fast light units
- Light weapons (infantry rifles) struggle vs heavy armor
- Medium weapons are versatile but not optimal
- Creates "screen with light units" counter-play

---

## Attack Mechanics

### Attack Speed

Units have attack cooldowns:

```text
DPS = (Base Damage) / (Attack Cooldown)
```

| Speed Category | Cooldown |
| -------------- | -------- |
| Very Fast | 0.5s |
| Fast | 1.0s |
| Normal | 1.5s |
| Slow | 2.0s |
| Very Slow | 3.0s |

### Range

| Category | Range (units) |
| -------- | ------------- |
| Melee | 1 |
| Short | 3-5 |
| Medium | 6-8 |
| Long | 9-12 |
| Artillery | 13+ |

### Line of Sight

- Units need vision to attack
- Some units have indirect fire (can attack without direct LoS)
- Fog of war blocks targeting

---

## Health and Regeneration

### Health Pools

| Unit Tier | Typical HP |
| --------- | ---------- |
| Tier 1 Infantry | 50-100 |
| Tier 1 Vehicle | 100-200 |
| Tier 2 Units | 200-400 |
| Tier 3 Units | 500-1200 |

### Regeneration

| Condition | Regen Rate |
| --------- | ---------- |
| Out of combat (10s) | 1 HP/s |
| Sculptors near Clinic | 2-5 HP/s |
| Near repair unit | 10 HP/s |
| In repair building | 20 HP/s |

---

## Terrain and Positioning

### Height Advantage

| Position      | Effect                  |
| ------------- | ----------------------- |
| Higher ground | +25% range, +10% damage |
| Lower ground  | -10% damage             |

### Cover

Cover reduces incoming damage:

| Cover Type | Damage Reduction |
| ---------- | ---------------- |
| Light (bushes) | 10% |
| Medium (walls, craters) | 25% |
| Heavy (bunkers, buildings) | 50% |

### Terrain Types

| Terrain | Movement Effect | Combat Effect |
| ------- | --------------- | ------------- |
| Open | Normal | None |
| Rough | -25% speed | Light cover |
| Water | Infantry blocked | None |
| Forest | -15% speed | Medium cover, blocks LoS |
| Urban | Normal | Heavy cover available |

---

## Unit Categories

### Infantry

| Property | Value |
| -------- | ----- |
| Armor Class | Light |
| Speed | Medium-Fast |
| Special | Can garrison, capture |

**Strengths:** Cheap, flexible, capture points  
**Weaknesses:** Low HP, vulnerable to AOE

### Vehicles

| Property | Value |
| -------- | ----- |
| Armor Class | Medium |
| Speed | Fast |
| Special | Varied roles |

**Strengths:** Mobile, good DPS  
**Weaknesses:** Can't capture, larger target

### Mechs

| Property | Value |
| -------- | ----- |
| Armor Class | Heavy |
| Speed | Slow |
| Special | Faction signature |

**Strengths:** High HP, high damage  
**Weaknesses:** Slow, expensive, vulnerable to kiting

### Air

| Property | Value |
| -------- | ----- |
| Armor Class | Air |
| Speed | Fast |
| Special | Ignores terrain |

**Strengths:** Mobility, hard to catch  
**Weaknesses:** Dedicated AA, can't capture

---

## Combat Abilities

### Ability Types

| Type | Description |
| ---- | ----------- |
| Active | Manually triggered, cooldown-based |
| Passive | Always active |
| Toggle | On/off states (siege mode) |
| Triggered | Automatic under conditions |

### Common Ability Categories

| Category | Examples |
| -------- | -------- |
| Damage | Burst fire, artillery strike, AOE |
| Movement | Charge, jump, teleport |
| Defensive | Shield, heal, cloak |
| Utility | Reveal, slow, disable |

### Ability Cooldowns

| Power Level | Cooldown |
| ----------- | -------- |
| Minor | 10-30s |
| Standard | 30-60s |
| Major | 60-120s |
| Ultimate | 120-300s |

---

## Status Effects

| Effect | Description | Duration |
| ------ | ----------- | -------- |
| **Slow** | -30% movement speed | 3-5s |
| **Stun** | Cannot act | 1-3s |
| **Silence** | Cannot use abilities | 5-10s |
| **Blind** | Reduced sight range | 5s |
| **Burn** | Damage over time, no regen | 5-10s |
| **Poison** | Damage over time | 10-15s |
| **Armor Break** | -50% armor | 5-10s |

---

## Veterancy System

Units gain experience from combat:

### XP Sources

| Action | XP Gained |
| ------ | --------- |
| Killing unit | Victim's cost × 0.5 |
| Damaging unit | Damage dealt × 0.1 |
| Surviving battle | Flat 10 XP |

### Veterancy Levels

| Level | XP Required | Bonus |
| ----- | ----------- | ----- |
| Veteran | 100 | +10% damage, +10% HP |
| Elite | 300 | +20% damage, +20% HP, ability upgrade |
| Legendary | 600 | +30% damage, +30% HP, unique ability |

### Preserving Veterans

- Units retain veterancy when healed
- Some factions can transfer veterancy
- High-level units are strategic assets

---

## Control Groups and Micro

### Selection Priority

When selecting mixed groups:

1. Combat units first
2. Workers last
3. Heroes always selectable

### Attack Move

- Units attack enemies along path
- Priority: Nearest threat → Largest threat
- Can be overridden with focus fire

### Patrol

- Units loop between points
- Attack enemies in range
- Return to patrol path after combat

### Hold Position

- Units don't move
- Attack enemies in range only
- Good for defense

---

## Combat AI (For Units)

### Targeting Priority

1. Unit attacking this unit
2. Nearest enemy in range
3. Nearest enemy overall
4. If healer: lowest HP ally

### Automatic Behaviors

| Behavior | Trigger | Action |
| -------- | ------- | ------ |
| Auto-attack | Enemy in range | Attack |
| Retreat | HP < 20% | Move away (toggleable) |
| Ability use | Conditions met | Use ability (AI-dependent) |

---

## Squad Mechanics

Some units operate as squads:

### Squad Properties

| Property | Description |
| -------- | ----------- |
| Size | Number of members (e.g., 4) |
| Health | Shared pool or per-member |
| Damage | Scales with members |
| Reinforcement | Can replenish at buildings |

### Faction Squad Units

| Faction | Squad Unit | Size |
| ------- | ---------- | ---- |
| Continuity Authority | Compliance Officers | 5 |
| Collegium | Attack Drone Squadron | 4 |
| Tinkers' Union | Field Techs | 4 |
| Sculptors | Aesthetics | 3 |
| Zephyr Guild | Cloudrunners | 4 |

---

## Formations

### Formation Types

| Formation | Use Case |
| --------- | -------- |
| Line | Ranged units, max firepower |
| Box | Mixed units, protection |
| Wedge | Assault, break through |
| Scatter | Anti-AOE |

### Formation Commands

- **Group** — Units maintain relative positions
- **Ungroup** — Units act independently
- **Spread** — Increase spacing (anti-AOE)
- **Clump** — Decrease spacing (focus fire)

---

## Building Combat

### Building Health

| Building Type | HP Range |
| ------------- | -------- |
| Economy | 500-1000 |
| Production | 800-1500 |
| Defense | 1000-2000 |
| Main Base | 2000-3000 |

### Siege Units

Units with building damage bonuses:

| Faction | Siege Unit | Building Bonus |
| ------- | ---------- | -------------- |
| Continuity Authority | Heavy Tank | +50% |
| Collegium | (None dedicated) | — |
| Tinkers' Union | (Improvised explosives) | — |
| Sculptors | Dissolution | +75% |
| Zephyr Guild | Bomber Zeppelin | +50% |

---

## Defensive Structures

### Turret Types

| Type | Target | DPS | Range | Cost |
| ---- | ------ | --- | ----- | ---- |
| Light Turret | Ground | 20 | 6 | 75-100 |
| Heavy Turret | Ground | 40 | 8 | 150-200 |
| AA Turret | Air | 30 | 10 | 100-150 |
| Artillery | Ground | 50 | 15 | 200-300 |

### Walls and Barriers

- Block unit movement
- HP scales with faction theme
- Can be destroyed by siege

---

## Combat Pacing

### Engagement Length

| Battle Size | Expected Duration |
| ----------- | ----------------- |
| Small skirmish | 10-30s |
| Medium battle | 30-60s |
| Major engagement | 60-120s |

### Time to Kill (TTK)

Balanced around:

- Tier 1 should die in 3-5 seconds
- Tier 2 should die in 8-15 seconds
- Tier 3 should die in 20-40 seconds

This allows time for micro while keeping battles dynamic.

---

## Related Documents

- [Main GDD](../gdd.md)
- [Economy System](./economy.md)
- [Tech Trees](./tech-trees.md)
- [Faction Documents](../factions/)
