# Campaign & Skirmish Design

## Design Philosophy

**Post-Scarcity RTS is a skirmish-first game with narrative campaigns.**

The campaigns teach mechanics, tell stories, and explore the thematic depth of each faction. But the core gameplay loop — the thing players return to hundreds of times — is skirmish and multiplayer.

### Priority Order

1. **Skirmish vs AI** — Must be excellent, infinitely replayable
2. **Multiplayer** — Competitive and casual modes
3. **Campaigns** — Short, impactful, memorable
4. **Co-op** — Campaign and survival modes

---

## Campaign Design

### The Anti-Fanaticism Message

Every faction starts sympathetic. Every faction has good intentions. And every faction, if they pursue total victory, becomes the villain.

The core message: **Any ideology, taken to its extreme, leads to dystopia.**

This isn't "both sides" centrism — it's a warning that **certainty is dangerous**. The healthiest endings require doubt, compromise, and the humility to admit you might be wrong.

### Campaign Structure

Each faction has **10-12 missions** following a three-act structure:

```text
┌─────────────────────────────────────────────────────┐
│                   CAMPAIGN ARC                      │
├─────────────────────────────────────────────────────┤
│                                                     │
│  ┌──────────────────────────────────────────────┐   │
│  │ ACT 1: HOPE (3 missions)                     │   │
│  │ • Tutorial integration                       │   │
│  │ • Faction identity established               │   │
│  │ • "We're the good guys" narrative            │   │
│  │ • No major choices (builds investment)       │   │
│  └──────────────────────────────────────────────┘   │
│                        ↓                            │
│  ┌──────────────────────────────────────────────┐   │
│  │ ACT 2: CONFLICT (4 missions)                 │   │
│  │ • Encounter other factions                   │   │
│  │ • Moral complications emerge                 │   │
│  │ • KEY CHOICES (affect ending)                │   │
│  │ • NPCs challenge player assumptions          │   │
│  └──────────────────────────────────────────────┘   │
│                        ↓                            │
│  ┌──────────────────────────────────────────────┐   │
│  │ ACT 3: RECKONING (3-5 missions)              │   │
│  │ • Consequences of earlier choices            │   │
│  │ • Branching mission paths                    │   │
│  │ • Multiple endings                           │   │
│  │ • Thematic payoff                            │   │
│  └──────────────────────────────────────────────┘   │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### Mission Length

| Mission Type | Target Length | Notes |
| ------------ | ------------- | ----- |
| Tutorial | 10-15 min | Teach core mechanics |
| Standard | 20-30 min | Base building + objectives |
| Commando | 15-20 min | Small squad, no base |
| Defense | 25-35 min | Waves of enemies |
| Climax | 30-45 min | Final missions, epic scale |

#### Total campaign time: 4-6 hours per faction

### Choice System

Choices matter through **accumulated weight**, not binary flags:

```text
Choice Points: 0-100 scale per axis

                CONTROL
                   ↑
                   │
     ISOLATION ←───┼───→ OPENNESS
                   │
                   ↓
                FREEDOM
```

Ending determined by:

- Total choice points across missions
- Key binary decisions (2-3 per campaign)
- Hidden metrics (civilian casualties, economic choices)

### Ending Types

Each campaign has **4 possible endings**:

| Type | Choice Pattern | Tone |
| ---- | -------------- | ---- |
| **Dark Victory** | Maximum faction ideology | Grimdark, unsettling |
| **Pyrrhic Victory** | High ideology, some doubt | Bittersweet, costly |
| **Compromise** | Balanced choices | Hopeful, realistic |
| **Redemption** | Reject faction extremes | Optimistic, brave |

The "Redemption" ending is the hardest to achieve — requires making choices that hurt your faction's short-term interests.

---

## Faction Campaign Summaries

### Continuity Authority: "Orderly Transition"

**Missions:** 11  
**Core Question:** When does protection become oppression?  
**Key NPC:** Coordinator Chen (voice of doubt)

| Ending | Description |
| ------ | ----------- |
| Dark Victory | Eternal surveillance state |
| Pyrrhic | Order maintained, humanity diminished |
| Compromise | Shared governance with other factions |
| Redemption | Authority dissolved, Chen leads transition |

### Collegium: "Open Source"

**Missions:** 10  
**Core Question:** Can knowledge be forced on people?  
**Key NPC:** Professor Adeyemi (voice of doubt)

| Ending | Description |
| ------ | ----------- |
| Dark Victory | Technocratic tyranny of "experts" |
| Pyrrhic | Knowledge shared, wisdom lost |
| Compromise | Open knowledge with consent |
| Redemption | Adeyemi leads, respects autonomy |

### Tinkers' Union: "Right to Repair"

**Missions:** 10  
**Core Question:** What's the cost of total independence?  
**Key NPC:** Doc Oyelaran (voice of openness)

| Ending | Description |
| ------ | ----------- |
| Dark Victory | Fragmented fortress communities |
| Pyrrhic | Independent but isolated |
| Compromise | Networked workshops, open trade |
| Redemption | Makers who teach, not hoard |

### Sculptors: "Becoming"

**Missions:** 11  
**Core Question:** When does helping become controlling?  
**Key NPC:** Dr. Elara Vasquez (voice of consent)

| Ending | Description |
| ------ | ----------- |
| Dark Victory | Mandatory modification, beautiful tyranny |
| Pyrrhic | Modified majority, resentful minority |
| Compromise | Parallel societies coexist |
| Redemption | Voluntary modification, true freedom |

### Zephyr Guild: "Sky Lords"

**Missions:** 10  
**Core Question:** Is freedom without responsibility exploitation?  
**Key NPC:** Captain Elena Vasquez (voice of ethics)

| Ending | Description |
| ------ | ----------- |
| Dark Victory | Anarcho-capitalist extraction |
| Pyrrhic | Wealthy but despised |
| Compromise | Prosperous with fair practices |
| Redemption | Traders who serve, not extract |

---

## Skirmish Design

### Core Modes

| Mode | Players | Win Condition | Notes |
| ---- | ------- | ------------- | ----- |
| **Standard** | 1v1 to 4v4 | Annihilation | Classic RTS |
| **Domination** | 2v2 to 4v4 | Hold 3/5 points | Control-focused |
| **King of the Hill** | 3-8 FFA | Hold center | Last-faction-standing variant |
| **Economic** | Any | 50k resources | Race to wealth |
| **Survival** | 1-4 Co-op | Survive waves | Endless scaling |
| **Assassination** | 1v1 to 3v3 | Kill enemy hero | High-stakes |

### Map Types

| Type | Size | Players | Duration |
| ---- | ---- | ------- | -------- |
| **Duel** | Small | 1v1 | 10-20 min |
| **Skirmish** | Medium | 2v2 | 20-30 min |
| **Battlefield** | Large | 3v3, 4v4 | 30-45 min |
| **Epic** | Huge | 4v4, FFA | 45-60+ min |

### AI Opponents

#### Difficulty Levels

| Level | Behavior | Handicap |
| ----- | -------- | -------- |
| **Recruit** | Slow, predictable | -20% income |
| **Soldier** | Competent, minor mistakes | None |
| **Veteran** | Optimized builds | None |
| **Commander** | Near-optimal, adapts | None |
| **Nightmare** | Cheats (vision, +10% income) | +10% income |

#### AI Personalities

| Personality | Behavior |
| ----------- | -------- |
| **Aggressive** | Early rushes, all-in timing attacks |
| **Defensive** | Turtles, maxes tech, late-game push |
| **Economic** | Fast expand, tries to out-scale |
| **Harasser** | Constant raids, never commits |
| **Adaptive** | Changes based on opponent |
| **Random** | Picks personality per match |

Each faction also has a **canonical AI** that plays to faction strengths.

---

## Co-op Modes

### Campaign Co-op

- All campaign missions playable 2-player
- Each player controls a commander
- Shared resources, split production
- Difficulty scales with player count

### Survival Mode

Endless wave defense:

```text
Wave 1-10:   Tutorial difficulty, single faction enemies
Wave 11-30:  Mixed factions, increasing tech level
Wave 31-50:  Boss waves every 10, faction abilities
Wave 51+:    Nightmare scaling, multi-faction assaults
```

Leaderboards track:

- Highest wave reached
- Resources gathered
- Kill count
- Speed clear (for fixed waves)

### Scenarios

Pre-built asymmetric challenges:

| Scenario | Description |
| -------- | ----------- |
| **Last Stand** | Defend fortress against endless waves |
| **Rescue** | Extract units from hostile territory |
| **Raid** | Destroy target before reinforcements |
| **Escort** | Protect convoy across map |
| **Holdout** | Survive with limited resources |

---

## Progression Systems

### Campaign Unlocks

Completing campaigns unlocks:

- Faction-specific commander powers (skirmish)
- Alternate unit skins
- Lore entries
- Achievement emblems

### Skirmish Progression

Separate from campaign:

- Player level (cosmetic)
- Faction affinity (games played)
- Ranked ladder placement
- Achievement hunting

---

## Replayability

### Why Players Return

| Content | Hook |
| ------- | ---- |
| Campaigns | Multiple endings, missed choices |
| Skirmish | Faction matchup variety, AI personalities |
| Ranked | Competitive ladder climbing |
| Co-op | Survival high scores, challenge modes |
| Custom | Map editor, modding support |

### Session Length Targets

| Mode | Target |
| ---- | ------ |
| Quick Skirmish | 15-20 min |
| Standard Match | 25-35 min |
| Campaign Mission | 20-30 min |
| Survival Run | 30-60 min |
| Ranked Match | 20-40 min |

---

## Related Documents

- [Game Design Document](gdd.md)
- [Faction Designs](factions/)
- [Economy System](systems/economy.md)
- [Combat System](systems/combat.md)
