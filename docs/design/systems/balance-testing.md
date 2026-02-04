# Automated Balance Testing

## Overview

The headless simulation engine (`rts_headless`) runs thousands of games automatically to detect balance issues before they reach players. This document outlines the **balance factors** considered and how they interact.

---

## Balance Factors

### 1. Attack Range

**Definition:** Distance at which a unit can engage enemies (fixed-point units in RON files).

| Category | Range (game units) | Typical Use |
| -------- | ----------------- | ----------- |
| Melee | 10-20 | Infantry close combat |
| Short | 30-50 | Standard infantry, light vehicles |
| Medium | 60-80 | Ranged infantry, medium tanks |
| Long | 90-110 | Artillery, snipers |
| Extreme | 120+ | Siege platforms, capital ships |

**Balance Impact:**

- **Kiting advantage:** Long-range units can damage short-range units while retreating
- **First-strike:** Range determines who initiates combat, gaining DPS advantage
- **Map coverage:** High-range units control more area with fewer units
- **Counter-play:** Short-range units must close distance, requiring speed or cover

**In RON files:**

```ron
// Range is stored as Fixed-point (I32F32)
// Value = desired_range × 4294967296
// Example: 50.0 range = 214748364800
combat: Some((
    damage: 12,
    range: 214748364800,  // 50.0 game units
    attack_cooldown: 30,
    armor: 5,
)),
```

### 2. Movement Speed

**Definition:** Units moved per tick (fixed-point).

| Category | Speed | Typical Units |
| -------- | ----- | ------------- |
| Very Slow | 3-5 | Artillery, super-heavy |
| Slow | 6-8 | Heavy mechs, siege |
| Normal | 9-11 | Infantry, standard vehicles |
| Fast | 12-16 | Scouts, light vehicles |
| Very Fast | 17+ | Pure scouts, interceptors |

**Balance Impact:**

- **Engagement control:** Fast units choose when to fight
- **Kiting synergy:** Speed + range enables hit-and-run tactics
- **Reinforcement timing:** Faster units arrive sooner, affecting composition
- **Escape ability:** Damaged fast units can disengage

**Speed-Range Interaction Matrix:**

| | Short Range | Long Range |
| - | ----------- | ---------- |
| **Slow** | Needs protection, ambush tactics | Siege role, positional |
| **Fast** | Flanker, requires closing gap | Dominant kiter, oppressive |

### 3. Map Size

**Definition:** Battle arena dimensions in game units.

| Size | Dimensions | Characteristics |
| --- | --- | --- |
| Small | 256×256 | Fast games, favors brawlers |
| Medium | 512×512 | Standard balance target |
| Large | 768×768 | Favors range, requires mobility |
| Huge | 1024×1024 | Strategic, multiple fronts |

**Balance Impact:**

- **Engagement distance:** Larger maps favor long-range units
- **Economic pressure:** More distance to resources
- **Rush viability:** Small maps enable early aggression
- **Composition diversity:** Large maps reward combined arms

### 4. Terrain and Line of Fire

**Definition:** Obstacles that block or modify combat.

**Terrain Types:**

- **Open:** No modifiers, pure stat comparison
- **Rough:** Slows movement, provides light cover
- **Forest/Urban:** Blocks line of fire, provides cover
- **Heights:** Range and damage bonuses

**Balance Impact:**

- **Cover negates range advantage:** Long-range units lose value in dense terrain
- **Chokepoints:** Funnel units, favor high-DPS over high-range
- **Height control:** Critical for artillery and sniper units
- **Approach routes:** Terrain defines safe paths for short-range units

**Current Headless Implementation:**

```rust
// terrain is not yet simulated in headless tests
// Games run on flat, open terrain
// TODO: Add terrain generation for balance testing
```

---

## Composite Balance Metrics

### Effective DPS

```text
Effective DPS = (Base Damage / Attack Cooldown) × (1 - Miss Rate)
```

### Effective Range Value (ERV)

Combines range with terrain access:

```text
ERV = Range × Terrain Factor × (Engagement Time / Total Battle Time)
```

### Cost Efficiency Ratio

```text
Cost Efficiency = (Resources Worth of Kills) / (Unit Cost)
```

### Speed-Range Index (SRI)

Measures kiting potential:

```text
SRI = (Speed / Average Speed) × (Range / Average Range)
SRI > 1.5 = Strong kiter
SRI < 0.5 = Needs support to engage
```

---

## Current Balance State (as of 2026-02-04)

### Continuity Authority vs Collegium

**100-game sample with faction data:**

- Continuity: 74% win rate
- Collegium: 26% win rate

**Likely causes:**

1. **Security Team** (Continuity T1) has more armor than **Research Assistant** (Collegium T1)
2. Continuity units have higher HP on average
3. Collegium's drone swarm advantage not fully realized without proper AI

### Balance Adjustments Needed

| Unit | Current Issue | Suggested Fix |
| --- | --- | --- |
| Research Assistant | Too fragile | Increase HP 50→60 or reduce cost 30→25 |
| Attack Drone Squadron | High DPS but vulnerable | Add swarm bonus when grouped |
| Guardian Mech | Dominates mid-game | Increase cost 300→350 |

---

## Testing Commands

### Quick Balance Check (100 games)

```bash
cargo run --release -p rts_headless -- batch -c 100 --seed 12345 \
  --faction-data crates/rts_game/assets/data/factions
```

### Extended Balance Run (1000 games)

```bash
cargo run --release -p rts_headless -- batch -c 1000 --seed 99999 \
  --faction-data crates/rts_game/assets/data/factions \
  -o results/balance_run
```

### Single Faction vs Faction

```bash
# Uses scenario RON file for specific matchup
cargo run --release -p rts_headless -- single \
  --scenario crates/rts_headless/scenarios/continuity_vs_collegium.ron
```

---

## Future Improvements

### Terrain Integration (Priority: High)

Add procedural terrain to headless tests:

- Obstacle density parameter (0.0 = open, 1.0 = dense)
- Height variation
- Resource placement variety

### Multi-Map Testing (Priority: Medium)

Run balance tests across multiple map sizes to detect:

- Rush-favorable conditions
- Tech-favorable conditions
- Map-specific balance outliers

### Unit Role Tracking (Priority: Medium)

Track per-unit-type performance:

- Win rate contribution
- Average lifespan
- Resource efficiency
- Kill count by tier

### Asymmetric AI (Priority: High)

Different AI strategies per faction:

- Continuity: Defensive, tech-focused
- Collegium: Swarm tactics
- Zephyr: Air control
- Tinkers: Adapt and counter
- Sculptors: Sustain and attrition

---

## Related Documents

- [Combat System](./combat.md)
- [Economy System](./economy.md)
- [AI Testing Toolchain](./ai-testing-and-toolchain.md)
- [Faction Documents](../factions/)
