# Vision & Intelligence System

## Design Philosophy

Vision is not just "can I see them" — it's a core strategic layer that enables genuine faction asymmetry and creates meaningful synergies between unit types.

**Inspirations:**

- **EVE Online:** Scouts/tackle freeze targets, long-range ships snipe from safety
- **Warhammer 40K:** Spotters call in artillery; different armies have radically different vision doctrines
- **Company of Heroes:** True sight vs camouflage; recon units are fragile but essential
- **StarCraft:** Observers, scans, overlords — each faction finds vision differently

### Core Principle: Vision Creates Roles

Without deliberate vision design, RTS units converge toward "10% stat variants" — everything becomes an all-rounder with minor differences. By making vision a first-class mechanic:

- **Scouts** have a real job beyond "fast unit that dies"
- **Artillery** needs support to function (can't see what they're shooting)
- **Stealth** becomes meaningful (hiding from limited vision, not omniscient fog)
- **Faction identity** emerges from *how* each faction acquires information

---

## Vision Mechanics

### Sight Range

Every unit has a **sight range** — the radius within which they reveal enemies.

| Category | Typical Sight Range | Notes |
| -------- | ------------------- | ----- |
| Infantry | 8-10 | Good baseline sight |
| Vehicles | 10-12 | Elevated position helps |
| Artillery | 4-6 | Can't see what they're shooting |
| Scouts | 14-18 | Primary role is vision |
| Buildings | 10-12 | Static but reliable |
| Air Units | 12-15 | High altitude, good sightlines |

**Key Design Rule:** Attack range and sight range are *independent*. This creates the scout/artillery dynamic:

```text
Artillery:  Range 16, Sight 5   → Can shoot far, but can't see far
Scout:      Range 0,  Sight 16  → Can see far, but can't shoot

Together:   Scout spots, artillery fires
```

### Shared Vision

Units share vision with allies. When a scout sees an enemy, all friendly units can target it.

**Faction Variations:**

| Faction | Vision Sharing |
| ------- | -------------- |
| Continuity Authority | Full sharing via central command net |
| Collegium | Full sharing + bonus range when grouped ("network effect") |
| Tinkers' Union | Limited sharing — only units in same "band" |
| Sculptors | Shared through psionic links (requires living units) |
| Zephyr Guild | Only air units share vision globally; ground is local |

### Detection vs Revelation

Two types of "seeing":

1. **Revelation** — Enemy is visible, can be targeted
2. **Detection** — Reveals cloaked/stealthed units

Not all vision sources detect stealth. Dedicated sensor units are required.

---

## Unit Archetypes: Vision Roles

### The Scout

**Role:** See things, don't die, don't fight.

| Property | Value |
| -------- | ----- |
| Sight Range | 14-18 (best in game) |
| Attack | None or minimal |
| Speed | Very Fast |
| HP | Very Low (25-50) |
| Cost | Cheap (25-50) |
| Stealth | Often cloaked when stationary |

**Design Intent:** Scouts are *not* fast fighters. They are eyes. Their value is in staying alive and maintaining vision. Making them armed creates "scout that kills workers" which is a different unit (harasser).

**Faction Scout Identity:**

| Faction | Scout Unit | Unique Trait |
| ------- | ---------- | ------------ |
| Continuity Authority | Watcher Drone | Tethered to command post (limited range but very stealthy) |
| Collegium | Scout Drone | Cloaked when stationary, fastest unit in game |
| Tinkers' Union | Outrider Bike | Fast, can lay sensor beacons, armed (light) |
| Sculptors | Courier | Fast enhanced human, Relay ability shares vision; Whisperers can "mark" enemies psychically |
| Zephyr Guild | Cloudskimmer | Flying scout, can spot from high altitude (huge range) |

### The Spotter

**Role:** Maintain vision for fire support units. May be a scout, may be a forward unit.

**Key Mechanic:** Some units have "spotter" tag — they provide targeting data for indirect fire.

| Spotter Benefits |
| ---------------- |
| Artillery can fire at enemies within spotter's sight |
| Some spotters provide "painted target" bonus (+accuracy, +damage) |
| Losing spotters blinds your artillery |

### The Artillery

**Role:** Long-range damage dealer that cannot self-spot.

| Property | Value |
| -------- | ----- |
| Sight Range | 4-6 (very short) |
| Attack Range | 14-20 (very long) |
| Damage | Very High |
| Speed | Very Slow or None |
| HP | Low-Medium |
| Cost | High |

**Design Intent:** Artillery without spotters is nearly useless. This creates:

- Combined arms requirement
- Vulnerability when scouts die
- Strategic depth in positioning

**Faction Artillery Identity:**

| Faction | Artillery Unit | Unique Trait |
| ------- | -------------- | ------------ |
| Continuity Authority | Compliance Cannon | Slow siege, massive damage, requires forward observers |
| Collegium | Beam Array | Energy weapon, no travel time, needs network node for targeting |
| Tinkers' Union | Cobbled Launcher | Inaccurate but cheap, can fire smoke for cover |
| Sculptors | Chorus | Psychic artillery — fused Whisperers project pain/terror; nearly blind, needs Couriers or Whisperers to spot |
| Zephyr Guild | Bomber Zeppelin | Mobile artillery, self-spots from altitude but fragile |

### The Sensor Platform

**Role:** Detect stealth, extend vision, provide intel.

Examples:

- Sensor towers (buildings)
- Mobile sensor vehicles
- Drone swarms
- Psionic sensitives

---

## Stealth System

### Stealth States

| State | Description |
| ----- | ----------- |
| **Visible** | Normal unit, everyone can see |
| **Concealed** | Hidden in terrain cover (forests, smoke) — revealed by proximity |
| **Cloaked** | Active stealth — invisible until detected or attacking |
| **Burrowed** | Underground — can't be seen or attacked (Sculptors specialty) |

### Detection Mechanics

| Detection Source | Range | Notes |
| ---------------- | ----- | ----- |
| Base sight | 0 | Normal units can't detect stealth |
| Detector units | 8-12 | Dedicated anti-stealth units |
| Sensor buildings | 12-15 | Static but reliable |
| AOE detection | Varies | Scan abilities, flares |
| Proximity reveal | 2-3 | Getting too close breaks stealth |

### Cloak Mechanics

When a unit cloaks:

- Invisible to enemies without detection
- Can't attack while cloaked (breaks stealth)
- Some abilities can be used while cloaked
- Moving while cloaked may create "shimmer" (partial reveal)

**Cloak Energy:**

Some factions have energy-based cloaking:

- Cloak drains energy while active
- Running out = revealed
- Creates decision: "How long can I stay hidden?"

---

## Faction Vision Doctrines

### Continuity Authority: Panopticon

**Philosophy:** See everything through surveillance networks.

| Strength | Weakness |
| -------- | -------- |
| Sensor buildings provide excellent coverage | Mobile vision is limited |
| Watcher drones are very stealthy | Drones are tethered, can't go far |
| Can "scan" anywhere on map (ability) | Scan has long cooldown |
| Central command shares all vision | Kill command = blind the army |

**Playstyle:** Build sensor coverage, use scans for key intel, protect central command net.

### Collegium: Distributed Awareness

**Philosophy:** Many eyes, open data, network effects.

| Strength | Weakness |
| -------- | -------- |
| Scout drones are fastest and cloak | Individual scouts are fragile |
| Network bonus: +2 sight when 3+ units together | Spread-out forces lose bonus |
| "Open Source" — killed enemies briefly reveal surroundings | Requires kills to trigger |
| ARIA can hack enemy sensors (ability) | Hacking requires proximity |

**Playstyle:** Swarm scouts, maintain network density, hack enemy intel.

**Key Synergy — Collegium Sniper Doctrine:**

1. Scout Drones maintain vision (cloaked, expendable)
2. Hover Tanks or Beam Arrays fire from maximum range
3. Shield Drones protect the fire support
4. Enemy must kill invisible scouts or close distance under fire

### Tinkers' Union: Sensor Improvisation

**Philosophy:** Make do with what you have. Gadgets and grit.

| Strength | Weakness |
| -------- | -------- |
| Sensor beacons can be placed anywhere | Beacons are visible, destructible |
| Outrider Bikes are armed scouts | Temptation to fight = dead scouts |
| Can salvage enemy sensor equipment | Requires winning fights first |
| Smoke grenades block enemy vision | Smoke blocks your vision too |

**Playstyle:** Plant beacons, use smoke for cover, leverage terrain knowledge.

### Sculptors: Psychic Network

**Philosophy:** Enhanced minds sense what eyes cannot.

| Strength | Weakness |
| -------- | -------- |
| Whisperers can "mark" enemies (psychic tracking) | Marking requires line of sight initially |
| Marked targets stay visible even through fog | Marking is obvious — enemy knows they're tagged |
| Chorus shares vision with all Whisperers | Chorus is nearly blind alone (sight 4) |
| Couriers share vision via Relay ability | Couriers are fragile, non-combatant |

**Playstyle:** Mark high-value targets, maintain psychic network between Whisperers and Chorus. Losing your Whisperers blinds your Chorus artillery.

### Zephyr Guild: Air Superiority

**Philosophy:** Own the skies, own the information.

| Strength | Weakness |
| -------- | -------- |
| Air scouts have massive sight range | Air scouts need air control |
| Cloudskimmers can spot from huge distances | Ground vision is limited |
| Mobile bases can reposition sensors | Bases are high-value targets |
| Can intercept enemy communications | Requires tech investment |

**Playstyle:** Achieve air control, leverage altitude for intel, deny enemy air scouts.

---

## Vision in Combat

### Artillery Targeting

Artillery units have two targeting modes:

1. **Direct Fire** — Can only shoot what they personally see (rare for artillery)
2. **Indirect Fire** — Can shoot any enemy visible to allies

**Targeting Priority for Indirect Fire:**

1. Targets marked by spotters (highest priority)
2. Targets visible to multiple allies
3. Targets at edge of vision (lowest accuracy)

### "Painted Target" Mechanic

Some units can "paint" targets:

```text
Painted Target:
- Marked enemy takes +25% damage from indirect fire
- Artillery has +50% accuracy vs marked target
- Paint lasts 5 seconds, can be refreshed
- Spotter must maintain line of sight
```

**Painter Units:**

| Faction | Painter | Notes |
| ------- | ------- | ----- |
| Continuity Authority | Designator | Dedicated unit, long paint range |
| Collegium | Any drone (upgrade) | Network target sharing |
| Tinkers' Union | Outrider Bike (ability) | Short duration, risky |
| Sculptors | Whisperer (psychic mark) | Marked targets take +damage from Chorus |
| Zephyr Guild | Cloudskimmer (upgrade) | Paint from altitude |

### Vision Loss Events

When scouts/spotters die:

1. Enemy immediately leaves vision
2. Artillery loses targets, stops firing
3. Last known position marked on minimap
4. "Vision lost" audio cue

**Design Intent:** Killing enemy scouts is *powerful*. It blinds their artillery and creates attack windows.

---

## Counter-Intel & Stealth Tactics

### Denying Enemy Vision

| Tactic | Effect |
| ------ | ------ |
| Kill their scouts | Blinds artillery, creates fog |
| Smoke/obscuration | Blocks sight lines temporarily |
| Jamming abilities | Disrupts sensor equipment |
| Stealth units | Invisible to unprepared enemies |
| Bait and switch | Fake movements in visible areas |

### Intel Advantage

Having better vision means:

- You see engagements coming
- Your artillery fires first
- You can target high-value units (spotters, HQ)
- Enemy walks into ambushes

**Intel is a force multiplier.** A smaller army with vision beats a larger army that's blind.

---

## Implementation Notes

### Data Model

```rust
// Vision properties on units
pub struct VisionStats {
    /// How far this unit can see (in game units)
    pub sight_range: Fixed,
    
    /// Can this unit detect cloaked/stealthed enemies?
    pub detection_range: Option<Fixed>,
    
    /// Does this unit share vision globally, locally, or not at all?
    pub vision_sharing: VisionSharing,
    
    /// Can this unit "paint" targets for artillery?
    pub can_paint_targets: bool,
    
    /// Stealth capability (None, Cloak, Burrow)
    pub stealth: Option<StealthType>,
}

pub enum VisionSharing {
    Global,          // All allies see what this unit sees
    Local { range: Fixed },  // Only nearby allies share vision
    None,            // Solo vision (rare)
}

pub enum StealthType {
    Cloak { energy_cost: Fixed },
    CloakWhenStationary,
    Burrow,
    Concealment,  // Only in cover
}
```

### Gameplay Parameters

```ron
// Example: Collegium Scout Drone
(
    sight_range: 16,
    detection_range: None,  // Can't detect stealth
    vision_sharing: Global,
    can_paint_targets: false,
    stealth: Some(CloakWhenStationary),
)

// Example: Collegium Hover Tank (sniper)
(
    sight_range: 8,
    detection_range: None,
    vision_sharing: Global,
    can_paint_targets: false,
    stealth: None,
    attack_range: 14,  // Long range but can't self-spot beyond 8
)

// Example: Collegium Beam Array (artillery)
(
    sight_range: 5,
    detection_range: None,
    vision_sharing: Global,
    can_paint_targets: false,
    stealth: None,
    attack_range: 18,  // Very long range, totally blind
)
```

---

## Balancing Vision

### Key Balance Levers

| Lever | Effect |
| ----- | ------ |
| Scout cost | How easily can vision be established? |
| Scout durability | How easily is vision lost? |
| Sight range | How much area can one scout cover? |
| Detection availability | How hard is stealth to counter? |
| Artillery blind spot | How dependent is artillery on spotters? |

### Faction Balance via Vision

| Faction | Vision Advantage | Vision Weakness |
| ------- | ---------------- | --------------- |
| Continuity | Best static coverage | Worst mobile scouting |
| Collegium | Fastest, cheapest scouts | Fragile, no detection |
| Tinkers | Most versatile (beacons) | Requires setup time |
| Sculptors | Best tracking (bonding) | Slow, limited range |
| Zephyr | Best range (altitude) | Dependent on air control |

---

## Related Documents

- [Combat System](./combat.md)
- [Faction Documents](../factions/)
- [GDD](../gdd.md)

---

## Open Questions

1. **Fog of War Type:** Full fog (unexplored = black) or shroud (explored = terrain visible, units hidden)?
2. **Vision persistence:** How long do last-known-positions stay on map?
3. **Minimap vision:** Does minimap show real-time dots or only ping on detection?
4. **Audio cues:** How much audio feedback for vision events (spotted, lost contact)?
