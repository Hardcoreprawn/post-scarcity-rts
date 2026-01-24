# Combat System

## Overview

Combat in Post-Scarcity RTS emphasizes readability, tactical depth, and faction identity. Battles should be visually clear, strategically meaningful, and feel distinct depending on which factions are fighting.

---

## Core Combat Mechanics

### Damage Calculation

```text
Final Damage = (Base Damage × Damage Type Modifier) - Armor
Minimum Damage = 1
```

### Damage Types

| Type | Strong Against | Weak Against |
| ---- | -------------- | ------------ |
| **Kinetic** | Light, Air | Heavy |
| **Explosive** | Heavy, Buildings | Air, Spread units |
| **Energy** | All (neutral) | Shielded |
| **Bio-Acid** | Light, regenerating | Heavy |
| **Fire** | Bio-units, Light | Mechanical |

### Damage Matrix

```text
                    Target Armor Class
                 Light   Medium   Heavy   Air   Building
Kinetic          100%    75%      50%     75%   50%
Explosive        75%     100%     125%    50%   150%
Energy           100%    100%     100%    100%  75%
Bio-Acid         125%    100%     75%     100%  100%
Fire             125%    100%     75%     100%  125%
```

### Armor

Armor reduces incoming damage by a flat amount:

- 10 armor = -10 damage per hit
- Armor penetration: some weapons ignore armor

| Armor Class | Typical Armor Value |
| ----------- | ------------------- |
| Light | 0-10 |
| Medium | 15-30 |
| Heavy | 40-80 |

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
