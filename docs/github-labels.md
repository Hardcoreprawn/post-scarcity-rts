# GitHub Labels & Milestones

Create these labels in the GitHub repository settings.

## Phase Labels

| Label | Color | Description |
|-------|-------|-------------|
| `phase:0` | #0E8A16 | Foundation (Complete) |
| `phase:1` | #0E8A16 | Core Engine (Complete) |
| `phase:2` | #0E8A16 | Gameplay Foundation (Complete) |
| `phase:3` | #FBCA04 | Vertical Slice (Current) |
| `phase:4` | #C5DEF5 | Faction Expansion |
| `phase:5` | #C5DEF5 | Advanced Simulation |
| `phase:6` | #C5DEF5 | Multiplayer |
| `phase:7` | #C5DEF5 | Campaign |

## Priority Labels

| Label | Color | Description |
|-------|-------|-------------|
| `priority:critical` | #B60205 | Blocks all other work |
| `priority:high` | #D93F0B | Must complete this phase |
| `priority:medium` | #FBCA04 | Should complete this phase |
| `priority:low` | #0E8A16 | Nice to have |
| `priority:someday` | #C5DEF5 | Backlog idea |

## Type Labels

| Label | Color | Description |
|-------|-------|-------------|
| `type:bug` | #D73A4A | Something broken |
| `type:feature` | #A2EEEF | New functionality |
| `type:refactor` | #7057FF | Code improvement |
| `type:idea` | #F9D0C4 | Design exploration (not committed) |
| `type:exploration` | #E4E669 | Technical spike |

## Status Labels

| Label | Color | Description |
|-------|-------|-------------|
| `status:blocked` | #B60205 | Waiting on dependency |
| `status:needs-design` | #FBCA04 | Requires design decision |
| `status:ready` | #0E8A16 | Ready to implement |

## Milestones

1. **Phase 3.0: Testing Infrastructure** - Current
2. **Vertical Slice Gate** - Ship quality for Continuity vs Collegium
3. **Phase 4: Faction Expansion** - Add remaining 3 factions
4. **Phase 5: Advanced Simulation** - Height, cover, veterancy
5. **Phase 6: Multiplayer** - Lockstep networking
6. **Phase 7: Campaign** - Story content

---

## Quick Setup Commands

Run these with GitHub CLI (`gh`):

```bash
# Phase labels
gh label create "phase:0" --color "0E8A16" --description "Foundation (Complete)"
gh label create "phase:1" --color "0E8A16" --description "Core Engine (Complete)"
gh label create "phase:2" --color "0E8A16" --description "Gameplay Foundation (Complete)"
gh label create "phase:3" --color "FBCA04" --description "Vertical Slice (Current)"
gh label create "phase:4" --color "C5DEF5" --description "Faction Expansion"
gh label create "phase:5" --color "C5DEF5" --description "Advanced Simulation"
gh label create "phase:6" --color "C5DEF5" --description "Multiplayer"
gh label create "phase:7" --color "C5DEF5" --description "Campaign"

# Priority labels
gh label create "priority:critical" --color "B60205" --description "Blocks all other work"
gh label create "priority:high" --color "D93F0B" --description "Must complete this phase"
gh label create "priority:medium" --color "FBCA04" --description "Should complete this phase"
gh label create "priority:low" --color "0E8A16" --description "Nice to have"
gh label create "priority:someday" --color "C5DEF5" --description "Backlog idea"

# Type labels
gh label create "type:bug" --color "D73A4A" --description "Something broken"
gh label create "type:feature" --color "A2EEEF" --description "New functionality"
gh label create "type:refactor" --color "7057FF" --description "Code improvement"
gh label create "type:idea" --color "F9D0C4" --description "Design exploration"
gh label create "type:exploration" --color "E4E669" --description "Technical spike"

# Status labels
gh label create "status:blocked" --color "B60205" --description "Waiting on dependency"
gh label create "status:needs-design" --color "FBCA04" --description "Requires design decision"
gh label create "status:ready" --color "0E8A16" --description "Ready to implement"
```
