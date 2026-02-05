# Unit Roles and Scale Framework

> *"A fleet is not a collection of ships. It's a machine made of ships."*

## Design Philosophy

Inspired by EVE Online's depth: every unit has a **role**, every role has **counters**, and victory comes from **composition and synergy**, not just raw numbers. Asymmetric faction design means each faction interprets roles differently — a Collegium "tank" works nothing like a Continuity "tank."

### Scale Progression

Units exist across a **weight class** spectrum:

| Class | Description | HP Range | Cost Range | Pop |
| ----- | ----------- | -------- | ---------- | --- |
| **Light** | Expendable, fast, swarm potential | 40-120 | 25-75 | 1 |
| **Medium** | Core combat units, versatile | 150-400 | 100-275 | 2-3 |
| **Heavy** | Powerful anchors, army centerpieces | 450-800 | 300-500 | 4-6 |
| **Capital** | Game-defining presence | 800-1500 | 600-1000 | 8-12 |
| **Super** | One-per-army strategic assets | 1500+ | 1000+ | 15+ |

---

## Core Combat Roles

### 1. Scout / Recon

**Purpose:** Vision, information, early warning.

| Faction | Unit | Unique Trait |
| ------- | ---- | ------------ |
| Continuity | Patrol Vehicle | Reveals cloaked, institutional surveillance |
| Collegium | Scout Drone | Cloaked when stationary, fastest |
| Tinkers | Sprint Bike | Armed, can lay sensors |
| Sculptors | Courier | Relay shares vision, psychic marking |
| Zephyr | Scout Balloon | Flying, near-invisible |

**Elite Variant:** Recon units that survive long enough gain veterancy bonuses to sight range and movement speed. Legendary scouts become near-impossible to catch.

---

### 2. Tackle / Crowd Control

**Purpose:** Pin targets, prevent escape, enable engagement.

*Currently Missing* — Most factions lack dedicated tackle. This is the "hero tackle" role from EVE that makes big kills possible.

| Faction | Unit | Mechanic |
| ------- | ---- | -------- |
| Continuity | **Compliance Officer** | Tether immobilizes target, both stuck |
| Collegium | **Disruptor Drone** | EMP field slows all in area |
| Tinkers | **Grappler Mech** | Magnetic clamp, drags target |
| Sculptors | **Bonded** | Psychic tether, damage shared |
| Zephyr | **Harpoon Gunship** | Sky anchor, grounds air units |

---

### 3. Electronic Warfare (EW)

**Purpose:** Force multiplier through debuffs. Make enemies weaker, allies stronger.

*Critical Gap* — EW is underrepresented. EVE's EW ships define fleet composition.

**EW Types:**

| Effect | Description | Counter |
| ------ | ----------- | ------- |
| **Jamming** | Target can't attack for duration | EW resistance, spread targets |
| **Tracking Disruption** | Target accuracy reduced | Get closer, use AoE |
| **Target Painting** | Target takes +damage from all sources | Kill the painter |
| **Sensor Dampening** | Target sight range reduced | Advance, use scouts |
| **Energy Neutralization** | Drain ability cooldowns/energy | Stay at range |

| Faction | EW Unit | Primary Effect |
| ------- | ------- | -------------- |
| Continuity | **Compliance Broadcaster** | Jamming burst (brief, area) |
| Collegium | **ECM Drone Swarm** | Distributed jamming (weaker but redundant) |
| Tinkers | **Junk Jammer** | Improvised ECM, unreliable but cheap |
| Sculptors | **Whisperer** | Tracking disruption (psychic interference) |
| Zephyr | **Signal Pirate** | Sensor dampening (comms interference) |

---

### 4. DPS — Brawler

**Purpose:** High damage at close range. Wins if they get in, dies if kited.

| Faction | Unit | Trait |
| ------- | ---- | ----- |
| Continuity | **Guardian Mech** | Heavy armor, twin cannons, defensive posture toggle |
| Collegium | **Attack Drone Squadron** | Glass cannon swarm, splits to chase |
| Tinkers | **Tinker Mech (Power Fist)** | Configurable, melee devastation |
| Sculptors | **Exemplar** | Silent melee terror, unflinching |
| Zephyr | **Cloudrunner Squad** | Board and sabotage, melee air assault |

---

### 5. DPS — Sniper / Skirmisher

**Purpose:** High damage at range. Needs spotters, dies to gap-closers.

| Faction | Unit | Trait |
| ------- | ---- | ----- |
| Continuity | **Pacification Platform** | Deploy-to-fire, precision |
| Collegium | **Hover Tank** | Range 14, sight 8, glass cannon |
| Tinkers | **Railgun Buggy** | Capacitor Dump burst, mobile |
| Sculptors | **Reclaimer** | Acid projector, siege capability |
| Zephyr | **Corsair Gunship** | Strafe runs from altitude |

---

### 6. Tank / Anchor

**Purpose:** Absorb damage, hold position, anchor the line.

*Currently Mixed* — Some factions have tanks, others don't. Should be explicit role.

| Faction | Unit | Mechanic |
| ------- | ---- | -------- |
| Continuity | **Phalanx Carrier** | Massive armor, small DPS, troops inside |
| Collegium | **Shield Drone (buffed)** | Shield bubble protects area |
| Tinkers | **Tinker Mech (Shield + Repair)** | Self-healing tank |
| Sculptors | **Warden Form** | Regenerating wall of meat, taunts |
| Zephyr | **Ironclad Barge** | Armored transport, takes hits for fleet |

---

### 7. Logistics / Support

**Purpose:** Keep army alive. High-priority targets.

| Faction | Unit | Trait |
| ------- | ---- | ----- |
| Continuity | **Field Hospital** | Mobile healing station, slow |
| Collegium | **Shield Drone** | Shield projection, emergency restore |
| Tinkers | **Support Carrier** | Mobile repair, module swap |
| Sculptors | **Attendant** | Direct healing, Stabilize prevents death |
| Zephyr | **Supply Barge** | Resupply (restore abilities) |

---

### 8. Artillery / Siege

**Purpose:** Break fortifications, area denial, long-range murder.

| Faction | Unit | Trait |
| ------- | ---- | ----- |
| Continuity | **Pacification Platform** | Precision bombardment |
| Collegium | **Beam Array** | Energy weapon, no travel time |
| Tinkers | **Cobbled Launcher** | Inaccurate, cheap, smoke option |
| Sculptors | **Chorus** | Psychic artillery, nearly blind without spotters |
| Zephyr | **Bomber Zeppelin** | Carpet bombing, self-spots from altitude |

---

### 9. Command / Boosting

**Purpose:** Aura buffs, army-wide improvements. Losing commander hurts.

*New Role* — Explicit command units that buff nearby allies.

| Faction | Unit | Aura Effect |
| ------- | ---- | ----------- |
| Continuity | **Command Post (Mobile)** | +10% accuracy, +2 sight, coordination |
| Collegium | **Network Node (Drone)** | +20% attack speed in radius |
| Tinkers | **Doc's Workshop** | +25% repair speed, module swap anywhere |
| Sculptors | **Conductor** | Harmonic link: nearby units share 15% damage |
| Zephyr | **Admiral's Flagship** | +15% speed, +10% damage to fleet |

---

### 10. Carrier / Force Multiplier

**Purpose:** Launch subordinate units, create local superiority.

| Faction | Unit | Payload |
| ------- | ---- | ------- |
| Continuity | **Rapid Response Platform** | Deploys Security Teams anywhere |
| Collegium | **Archon Core** | Auto-spawns attack drones |
| Tinkers | **Mobile Foundry** | Full production anywhere |
| Sculptors | **The Gift** | Converts enemies, "spawns" from captures |
| Zephyr | **Carrier Platform** | Produces T1-T2 air units |

---

## Advanced Roles

### 11. Interdictor / Area Denial

**Purpose:** Control space. Enemies cannot enter/exit freely.

| Faction | Unit | Mechanic |
| ------- | ---- | -------- |
| Continuity | **Checkpoint Station** | Deployable barrier + scanner |
| Collegium | **EMP Mine Layer** | Stationary denial field |
| Tinkers | **Sensor Net Drone** | Reveals + slows in area |
| Sculptors | **Spore Cloud** | Damage over time in area |
| Zephyr | **Storm Engine** | Weather control, damage + slow |

---

### 12. Stealth / Ambusher

**Purpose:** Strike from hidden, delete priority targets.

| Faction | Unit | Mechanic |
| ------- | ---- | -------- |
| Continuity | **Black Ops Team** | Cloaked infantry, assassins |
| Collegium | **Scout Drone + Attack Squadron** | Scouts cloak, attacks swarm reveal |
| Tinkers | **Saboteur** | Mines + demo charges |
| Sculptors | **Stalker** | Organic stealth, burrow-ambush |
| Zephyr | **Ghost Balloon** | Cloaked spotter, coordinates ambush |

---

### 13. Heavy Assault / Dreadnought

**Purpose:** Bring overwhelming firepower. Slow, expensive, decisive.

| Faction | Unit | Role |
| ------- | ---- | ---- |
| Continuity | **Sovereign Platform** | Quadruped fortress |
| Collegium | **Archon Core** | AI command, spawns drones |
| Tinkers | **Magnum Opus** | 4-slot masterwork mech |
| Sculptors | **Magnum Opus** | Flesh horror, devastating melee |
| Zephyr | **Sky Dreadnought** | Aerial battleship |

---

### 14. Superweapon / Strategic Asset

**Purpose:** Game-ending capability if left unchecked.

| Faction | Unit | Effect |
| ------- | ---- | ------ |
| Continuity | **Deterrence System** | Orbital kinetic strike |
| Collegium | **Singularity Node** | EMP disables all electronics |
| Tinkers | **Disruptor Array** | System Crash area disable |
| Sculptors | **Genesis Chamber** | Mass conversion facility |
| Zephyr | **Storm Engine** | Weather control superweapon |

---

## Veterancy and Elite Variants

### Veterancy System

Units gain XP for:

- Dealing damage
- Taking damage (and surviving)
- Completing objectives
- Being in winning engagements

| Rank | XP Required | Bonus |
| ---- | ----------- | ----- |
| **Trained** | 0 | Base stats |
| **Veteran** | 100 | +10% HP, +5% damage |
| **Elite** | 300 | +20% HP, +10% damage, +1 ability level |
| **Legendary** | 1000 | +30% HP, +15% damage, unique trait |

### Legendary Traits (Examples)

| Unit Type | Legendary Trait |
| --------- | --------------- |
| Scout | "Ghost" — Permanent cloak even when moving |
| Brawler | "Berserker" — +50% damage below 25% HP |
| Sniper | "Deadeye" — Ignores 50% armor |
| Tank | "Immovable" — Cannot be knocked back/stunned |
| Commander | "Inspiring" — Aura radius doubled |
| Artillery | "Precision" — +50% vs buildings |
| EW | "Specialist" — Effect duration doubled |

---

## Fleet Composition Synergies

### The Holy Trinity: Scout + Sniper + EW

- Scout provides vision beyond sniper range
- Sniper deals damage from safety
- EW debuffs targets, preventing effective response

### The Deathball: Tank + DPS + Logistics

- Tank absorbs damage, holds position
- DPS stacks behind tank, deals damage
- Logistics keeps tank alive

### The Roam: Tackle + Brawler + Scout

- Scout finds targets
- Tackle pins them
- Brawler deletes them before help arrives

### The Skyfall: Carrier + Fighters + Command

- Carrier produces fighters
- Fighters overwhelm defenders
- Command buffs fighter stats

---

## Unit Count Targets

To achieve EVE-like variety, each faction needs:

| Category | Current | Target | Notes |
| -------- | ------- | ------ | ----- |
| Light units | 3 | 5-6 | Add tackle, EW, stealth |
| Medium units | 3 | 6-8 | Add tank, command, more DPS variants |
| Heavy units | 3 | 4-5 | Add heavy EW, heavy tackle |
| Capital | 0-1 | 2-3 | Carriers, dreadnoughts, command ships |
| Super | 1 | 1-2 | Superweapons, titans |

**Total per faction: 18-24 units** (up from current ~9-10)

---

## Faction Role Distribution

### Continuity Authority — Combined Arms Doctrine

| Role | Unit | Notes |
| ---- | ---- | ----- |
| Scout | Patrol Vehicle | Reveals cloaked |
| Tackle | Compliance Officer | Tether-locks targets |
| EW | Compliance Broadcaster | Area jamming |
| Brawler | Guardian Mech | Defensive Posture toggle |
| Sniper | Pacification Platform | Deploy-to-fire precision |
| Tank | Phalanx Carrier | Massive armor, carries troops |
| Logistics | Field Hospital | Mobile healing |
| Artillery | Compliance Cannon | Slow siege, massive damage |
| Command | Mobile Command Post | Coordination aura |
| Carrier | Rapid Response Platform | Deploys Security Teams |
| Interdictor | Checkpoint Station | Barrier + scanner |
| Stealth | Black Ops Team | Assassin infantry |
| Dreadnought | Sovereign Platform | Quadruped command fortress |
| Super | Deterrence System | Orbital strikes |

*Identity:* Professional combined arms. Every role filled competently, no standouts, no gaps. Predictable but effective.

---

### Collegium — Distributed Swarm

| Role | Unit | Notes |
| ---- | ---- | ----- |
| Scout | Scout Drone | Cloaked when stationary |
| Tackle | Disruptor Drone | EMP field slows |
| EW | ECM Swarm | Distributed jamming |
| Brawler | Attack Drone Squadron | Splits, swarms |
| Sniper | Hover Tank | Range 14, sight 8 |
| Tank | Shield Node | Area shield projection |
| Logistics | Shield Drone | Shield + restore |
| Artillery | Beam Array | Energy weapon, instant |
| Command | Network Node | Attack speed aura |
| Carrier | Archon Core | Auto-spawns drones |
| Interdictor | EMP Mine Layer | Denial field |
| Stealth | Infiltrator Drone | Hacks buildings |
| Dreadnought | Archon Prime | Enhanced AI core |
| Super | Singularity Node | Global EMP |

*Identity:* Redundancy through numbers. Lose one unit, the swarm adapts. Weak individually, devastating collectively.

---

### Tinkers' Union — Modular Adaptation

| Role | Unit | Notes |
| ---- | ---- | ----- |
| Scout | Sprint Bike | Armed, lays sensors |
| Tackle | Grappler Mech | Magnetic clamp module |
| EW | Junk Jammer | Cheap, unreliable ECM |
| Brawler | Tinker Mech (Fists) | Melee loadout |
| Sniper | Railgun Buggy / Mech (Cannon) | Capacitor Dump |
| Tank | Tinker Mech (Shield + Repair) | Self-sustaining |
| Logistics | Support Carrier | Mobile repair bay |
| Artillery | Cobbled Launcher | Inaccurate, cheap |
| Command | Doc's Workshop | Repair aura |
| Carrier | Mobile Foundry | Full production |
| Interdictor | Sensor Net Drone | Reveal + slow |
| Stealth | Saboteur Team | Mines, demo charges |
| Dreadnought | Magnum Opus | 4-slot masterwork |
| Super | Disruptor Array | Mass disable |

*Identity:* The right tool for every job. Reconfigure on the fly. Loses to specialists, beats generalists.

---

### Sculptors — Elite Specialists

| Role | Unit | Notes |
| ---- | ---- | ----- |
| Scout | Courier | Fast, Relay shares vision |
| Tackle | Bonded | Psychic tether |
| EW | Whisperer | Tracking disruption, Unnerve |
| Brawler | Exemplar | Silent melee terror |
| Sniper | Reclaimer | Acid projector |
| Tank | Warden Form | Regen wall, taunt |
| Logistics | Attendant | Heal, Stabilize |
| Artillery | Chorus | Psychic AOE |
| Command | Conductor | Harmonic damage sharing |
| Carrier | The Gift | Converts enemies |
| Interdictor | Spore Generator | DOT denial zone |
| Stealth | Stalker | Burrow-ambush |
| Dreadnought | Magnum Opus | Flesh horror |
| Super | Genesis Chamber | Mass conversion |

*Identity:* Quality over quantity. Every unit is expensive, powerful, and losing them hurts. Master of specialists.

---

### Zephyr Guild — Air Superiority

| Role | Unit | Notes |
| ---- | ---- | ----- |
| Scout | Scout Balloon | Near-invisible flying |
| Tackle | Harpoon Gunship | Sky anchor, grounds flyers |
| EW | Signal Pirate | Sensor dampening |
| Brawler | Cloudrunner Squad | Boarding, melee air |
| Sniper | Corsair Gunship | Strafe runs |
| Tank | Ironclad Barge | Armored transport |
| Logistics | Supply Barge | Resupply abilities |
| Artillery | Bomber Zeppelin | Carpet bombs |
| Command | Admiral's Flagship | Fleet aura |
| Carrier | Carrier Platform | Produces air units |
| Interdictor | Storm Engine | Weather denial |
| Stealth | Ghost Balloon | Cloaked spotter |
| Dreadnought | Sky Dreadnought | Aerial battleship |
| Super | Storm Array | Weather superweapon |

*Identity:* Control the skies, control everything. Weak on ground, devastating from above.

---

## Implementation Priority

### Phase 1: Core Roles (Essential)

1. **Tackle unit** for each faction — enables engagement control
2. **EW unit** for each faction — force multiplier
3. **Tank variant** for each faction — creates front line

### Phase 2: Command & Carriers

1. **Command units** — aura-based gameplay
2. **Carrier expansion** — force multiplication

### Phase 3: Advanced Roles

1. **Interdictors** — area denial
2. **Stealth variants** — ambush gameplay
3. **Heavy assault expansion** — more capital options

### Phase 4: Veterancy System

1. XP tracking
2. Rank bonuses
3. Legendary traits

---

## Related Documents

- [Combat System](combat.md)
- [Vision and Intelligence](vision-and-intel.md)
- [Faction Design Documents](../factions/)
