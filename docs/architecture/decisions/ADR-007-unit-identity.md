# ADR-007: Unified Unit Identity System

## Status

Accepted

## Context

The RTS game needs to support:

- **Many unit types** (100+ per faction × 5 factions = 50+ units eventually)
- **Many concurrent units** (thousands on screen)
- **Data-driven definitions** (stats from RON files, not code)
- **Fast runtime queries** (e.g., "find all enemy harvesters")
- **Deterministic simulation** (lockstep multiplayer, replay)

**Current problems:**

1. `UnitType` enum requires code changes for every new unit
2. `UnitTypeId(u32)` in rts_core is disconnected from RON data
3. Two parallel identity systems = confusion and bugs
4. Hardcoded `to_unit_id(faction)` mapping is error-prone

## Decision

**ONE unified identity system** that works for both crates:

### The Core Types (in `rts_core`)

```rust
/// Numeric ID for a unit kind. Assigned at data load time.
/// 
/// This is the PRIMARY identity used everywhere at runtime.
/// - Cheap: Copy, 2 bytes
/// - Deterministic: same across all clients (derived from load order)
/// - Fast: array indexing, no hashing
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnitKindId(pub u16);

/// Bitflags for fast unit classification queries.
/// Computed from RON tags at load time, cached per unit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct UnitRole(pub u32);

impl UnitRole {
    // Movement
    pub const GROUND: Self    = Self(1 << 0);
    pub const AIR: Self       = Self(1 << 1);
    pub const NAVAL: Self     = Self(1 << 2);
    
    // Combat class
    pub const INFANTRY: Self  = Self(1 << 3);
    pub const VEHICLE: Self   = Self(1 << 4);
    pub const MECH: Self      = Self(1 << 5);
    pub const ARTILLERY: Self = Self(1 << 6);
    
    // Special roles
    pub const HARVESTER: Self = Self(1 << 7);
    pub const SUPPORT: Self   = Self(1 << 8);
    pub const SCOUT: Self     = Self(1 << 9);
    pub const TRANSPORT: Self = Self(1 << 10);
    pub const HEALER: Self    = Self(1 << 11);
    pub const BUILDER: Self   = Self(1 << 12);
    
    // Tier
    pub const TIER_1: Self    = Self(1 << 13);
    pub const TIER_2: Self    = Self(1 << 14);
    pub const TIER_3: Self    = Self(1 << 15);
    
    // Combat capability
    pub const COMBATANT: Self = Self(1 << 16);
    
    #[inline]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
    
    #[inline] 
    pub const fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }
    
    #[inline]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}
```

### The Registry (in `rts_core`)

```rust
/// Metadata for a unit kind, stored in registry.
#[derive(Clone, Debug)]
pub struct UnitKindInfo {
    /// Numeric ID (index into registry).
    pub id: UnitKindId,
    /// String ID from RON (e.g., "field_engineer").
    pub string_id: String,
    /// Faction this unit belongs to.
    pub faction: FactionId,
    /// Cached role flags for fast queries.
    pub role: UnitRole,
}

/// Central registry mapping between IDs. Built once at load time.
#[derive(Default)]
pub struct UnitKindRegistry {
    /// Lookup by numeric ID (O(1) array index).
    by_id: Vec<UnitKindInfo>,
    /// Lookup by (faction, string_id) → numeric ID.
    by_string: HashMap<(FactionId, String), UnitKindId>,
}

impl UnitKindRegistry {
    /// Register a unit kind, returns its numeric ID.
    pub fn register(&mut self, faction: FactionId, string_id: &str, role: UnitRole) -> UnitKindId {
        let id = UnitKindId(self.by_id.len() as u16);
        let info = UnitKindInfo {
            id,
            string_id: string_id.to_string(),
            faction,
            role,
        };
        self.by_id.push(info);
        self.by_string.insert((faction, string_id.to_string()), id);
        id
    }
    
    /// O(1) lookup by numeric ID.
    #[inline]
    pub fn get(&self, id: UnitKindId) -> Option<&UnitKindInfo> {
        self.by_id.get(id.0 as usize)
    }
    
    /// Lookup by faction + string ID (for spawning from data).
    pub fn find(&self, faction: FactionId, string_id: &str) -> Option<UnitKindId> {
        self.by_string.get(&(faction, string_id.to_string())).copied()
    }
    
    /// Get role flags for a unit kind (O(1)).
    #[inline]
    pub fn role(&self, id: UnitKindId) -> UnitRole {
        self.by_id.get(id.0 as usize).map_or(UnitRole::default(), |i| i.role)
    }
}
```

### Unit Component (shared)

```rust
/// Component identifying what kind of unit this is.
#[derive(Component, Clone, Copy, Debug)]
pub struct UnitKind {
    /// The numeric ID (for data lookup).
    pub id: UnitKindId,
    /// Cached role flags (for fast queries without registry lookup).
    pub role: UnitRole,
}
```

## Data Flow

```text
1. LOAD TIME (once at startup):
   RON files → FactionData → UnitKindRegistry
   
   For each unit in each faction:
     - Parse tags → compute UnitRole
     - Register in UnitKindRegistry → get UnitKindId
     - Store mapping: (faction, "field_engineer") → UnitKindId(42)

2. SPAWN TIME:
   Production system knows faction + string_id (from UI selection)
   → registry.find(faction, "field_engineer") → UnitKindId(42)
   → registry.get(42) → UnitKindInfo { role, ... }
   → spawn entity with UnitKind { id: 42, role }

3. RUNTIME QUERIES:
   AI/Combat system iterates units:
   → Read unit.role directly (no registry lookup!)
   → unit.role.contains(UnitRole::HARVESTER) // O(1) bit check

4. SAVE/LOAD:
   → Save: unit.id (u16) + faction
   → Load: registry.get(id) reconstructs everything
```

## Why This Design?

| Requirement | Solution |
| ----------- | -------- |
| Deterministic | UnitKindId(u16) is just a number, same everywhere |
| Fast queries | UnitRole bitflags cached in component, no lookup |
| Data-driven | String IDs from RON, mapped to numeric at load |
| Memory efficient | 2 bytes ID + 4 bytes role = 6 bytes per unit |
| Scalable | No code changes for new units, just RON data |
| Debuggable | Registry has string IDs for logging/tools |

## Usage Examples

### AI Targeting

```rust
// Query all harvesters
for (entity, kind, pos) in query.iter() {
    if kind.role.contains(UnitRole::HARVESTER) {
        // Target this harvester
    }
}
```

### Production Queue

```rust
// UI selects a unit to build (by string ID from RON)
let kind_id = registry.find(player_faction, "field_engineer")?;
production_queue.add(kind_id);

// When complete, spawn:
let info = registry.get(kind_id)?;
commands.spawn(UnitBundle::new(pos, info));
```

### Sprites

```rust
fn get_sprite(kind: &UnitKind, registry: &UnitKindRegistry) -> SpriteIndex {
    let info = registry.get(kind.id).unwrap();
    
    // Use string_id for faction-specific sprites
    match (info.faction, info.string_id.as_str()) {
        (FactionId::Tinkers, "field_engineer") => TINKER_ENGINEER_SPRITE,
        _ if kind.role.contains(UnitRole::HARVESTER) => GENERIC_HARVESTER_SPRITE,
        _ => DEFAULT_SPRITE,
    }
}
```

## Migration Path

1. Add `UnitKindId`, `UnitRole`, `UnitKindRegistry` to `rts_core`
2. Build registry from RON data in `FactionDataPlugin`
3. Add `UnitKind` component, update spawn code
4. Update AI/combat/UI to use `UnitRole` bitflags
5. Remove old `UnitType` enum and `UnitTypeId(u32)`

## Consequences

### Positive

- **One source of truth**: numeric ID everywhere, strings only in registry
- **Blazing fast**: 6 bytes per unit, bit operations for queries
- **Deterministic**: numeric IDs derived from load order (sorted for consistency)
- **Scalable**: 65,536 unit kinds possible (u16), more than enough
- **Zero code changes** for new unit types

### Negative  

- Registry must be available for string↔ID conversion
- Load order matters (solved by sorting faction files alphabetically)

## References

- [Data-Oriented Design](https://dataorienteddesign.com/)
- [Interned Strings Pattern](https://en.wikipedia.org/wiki/String_interning)
