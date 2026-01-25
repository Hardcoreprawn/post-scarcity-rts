//! Unified unit identity system for data-driven unit types.
//!
//! This module provides a single source of truth for unit identity:
//! - [`UnitKindId`]: Numeric ID for fast, deterministic runtime use
//! - [`UnitRole`]: Bitflags for fast classification queries
//! - [`UnitKindRegistry`]: Maps between IDs and provides metadata
//!
//! # Design Goals
//!
//! - **One system**: Works for both simulation (rts_core) and rendering (rts_game)
//! - **Data-driven**: No hardcoded unit types, everything from RON files
//! - **Fast**: Numeric IDs and bitflags, no string operations at runtime
//! - **Deterministic**: Numeric IDs are stable for networking/replay
//! - **Scalable**: Add new units by editing data files, no code changes

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::factions::FactionId;

/// Numeric identifier for a unit kind.
///
/// This is the primary identity used everywhere at runtime:
/// - Cheap: `Copy`, 2 bytes
/// - Deterministic: Same value across all clients (derived from sorted load order)
/// - Fast: Array indexing for lookups, no hashing needed
///
/// The ID is assigned when loading faction data and maps to a string ID
/// (like "field_engineer") through the [`UnitKindRegistry`].
///
/// # Example
///
/// ```
/// use rts_core::unit_kind::UnitKindId;
///
/// let id = UnitKindId::new(42);
/// assert_eq!(id.as_u16(), 42);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct UnitKindId(u16);

impl UnitKindId {
    /// Sentinel value indicating no unit kind.
    pub const NONE: Self = Self(u16::MAX);

    /// Create a new unit kind ID.
    #[must_use]
    pub const fn new(id: u16) -> Self {
        Self(id)
    }

    /// Get the raw numeric value.
    #[must_use]
    pub const fn as_u16(self) -> u16 {
        self.0
    }

    /// Check if this is a valid ID (not NONE).
    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.0 != u16::MAX
    }
}

/// Bitflags for fast unit classification queries.
///
/// Computed from RON `tags` array at load time, then cached per unit.
/// Enables O(1) queries like "is this a ground harvester?"
///
/// # Example
///
/// ```
/// use rts_core::unit_kind::UnitRole;
///
/// let role = UnitRole::GROUND.union(UnitRole::HARVESTER);
/// assert!(role.contains(UnitRole::HARVESTER));
/// assert!(role.intersects(UnitRole::GROUND));
/// assert!(!role.contains(UnitRole::AIR));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct UnitRole(u32);

impl UnitRole {
    // ========================================
    // Movement categories
    // ========================================

    /// Unit moves on ground.
    pub const GROUND: Self = Self(1 << 0);
    /// Unit flies.
    pub const AIR: Self = Self(1 << 1);
    /// Unit moves on water.
    pub const NAVAL: Self = Self(1 << 2);

    // ========================================
    // Combat class
    // ========================================

    /// Infantry unit (typically cheap, numerous).
    pub const INFANTRY: Self = Self(1 << 3);
    /// Vehicle unit (wheeled/tracked).
    pub const VEHICLE: Self = Self(1 << 4);
    /// Mech/walker unit.
    pub const MECH: Self = Self(1 << 5);
    /// Long-range artillery.
    pub const ARTILLERY: Self = Self(1 << 6);

    // ========================================
    // Special roles
    // ========================================

    /// Harvests resources.
    pub const HARVESTER: Self = Self(1 << 7);
    /// Support unit (buffs, repairs, etc.).
    pub const SUPPORT: Self = Self(1 << 8);
    /// Fast reconnaissance unit.
    pub const SCOUT: Self = Self(1 << 9);
    /// Can transport other units.
    pub const TRANSPORT: Self = Self(1 << 10);
    /// Can heal other units.
    pub const HEALER: Self = Self(1 << 11);
    /// Can construct buildings.
    pub const BUILDER: Self = Self(1 << 12);

    // ========================================
    // Tier classification
    // ========================================

    /// Tier 1 (early game).
    pub const TIER_1: Self = Self(1 << 13);
    /// Tier 2 (mid game).
    pub const TIER_2: Self = Self(1 << 14);
    /// Tier 3 (late game).
    pub const TIER_3: Self = Self(1 << 15);

    // ========================================
    // Combat capability
    // ========================================

    /// Can attack (has combat stats).
    pub const COMBATANT: Self = Self(1 << 16);

    // ========================================
    // Methods
    // ========================================

    /// Empty role (no flags set).
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Check if all flags in `other` are set in `self`.
    #[inline]
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Check if any flags in `other` are set in `self`.
    #[inline]
    #[must_use]
    pub const fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    /// Combine two roles (union of flags).
    #[inline]
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Get raw bits for serialization.
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Create from raw bits.
    #[must_use]
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    /// Build role flags from RON tags and unit metadata.
    ///
    /// Called once at load time for each unit definition.
    #[must_use]
    pub fn from_tags(tags: &[String], tier: u8, has_combat: bool) -> Self {
        let mut role = Self::empty();

        for tag in tags {
            let flag = match tag.as_str() {
                // Movement
                "ground" => Self::GROUND,
                "air" => Self::AIR,
                "naval" => Self::NAVAL,
                // Combat class
                "infantry" => Self::INFANTRY,
                "vehicle" => Self::VEHICLE,
                "mech" => Self::MECH,
                "artillery" => Self::ARTILLERY,
                // Special roles
                "harvester" => Self::HARVESTER,
                "support" => Self::SUPPORT,
                "scout" => Self::SCOUT,
                "transport" => Self::TRANSPORT,
                "healer" => Self::HEALER,
                "builder" | "repair" => Self::BUILDER,
                // Unknown tags are ignored (allows future expansion)
                _ => Self::empty(),
            };
            role = role.union(flag);
        }

        // Set tier flag
        role = role.union(match tier {
            1 => Self::TIER_1,
            2 => Self::TIER_2,
            3 => Self::TIER_3,
            _ => Self::empty(),
        });

        // Set combat capability
        if has_combat {
            role = role.union(Self::COMBATANT);
        }

        role
    }
}

// Implement BitOr for ergonomic flag combination
impl std::ops::BitOr for UnitRole {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl std::ops::BitOrAssign for UnitRole {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.union(rhs);
    }
}

/// Metadata about a unit kind, stored in the registry.
#[derive(Clone, Debug)]
pub struct UnitKindInfo {
    /// Numeric ID (index in registry).
    pub id: UnitKindId,
    /// String ID from RON data (e.g., "field_engineer").
    pub string_id: String,
    /// Faction this unit belongs to.
    pub faction: FactionId,
    /// Cached role flags for fast queries.
    pub role: UnitRole,
}

/// Central registry mapping between unit identities.
///
/// Built once at data load time, then used for lookups throughout
/// the game. Provides O(1) access by numeric ID.
///
/// # Thread Safety
///
/// The registry is immutable after construction, so it can be
/// safely shared across threads (wrapped in Arc if needed).
#[derive(Default, Debug)]
pub struct UnitKindRegistry {
    /// Lookup by numeric ID (O(1) array index).
    by_id: Vec<UnitKindInfo>,
    /// Lookup by (faction, string_id) → numeric ID.
    by_string: HashMap<(FactionId, String), UnitKindId>,
}

impl UnitKindRegistry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a unit kind and return its assigned numeric ID.
    ///
    /// # Arguments
    ///
    /// * `faction` - The faction this unit belongs to
    /// * `string_id` - The string ID from RON data
    /// * `role` - Pre-computed role flags
    ///
    /// # Returns
    ///
    /// The assigned `UnitKindId` for this unit.
    pub fn register(&mut self, faction: FactionId, string_id: &str, role: UnitRole) -> UnitKindId {
        // Check for duplicate registration
        if let Some(&existing) = self.by_string.get(&(faction, string_id.to_string())) {
            return existing;
        }

        let id = UnitKindId::new(self.by_id.len() as u16);
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

    /// Get unit info by numeric ID. O(1) array lookup.
    #[inline]
    #[must_use]
    pub fn get(&self, id: UnitKindId) -> Option<&UnitKindInfo> {
        if !id.is_valid() {
            return None;
        }
        self.by_id.get(id.0 as usize)
    }

    /// Find a unit's numeric ID by faction and string ID.
    ///
    /// Used when spawning from data (UI selection, etc.).
    #[must_use]
    pub fn find(&self, faction: FactionId, string_id: &str) -> Option<UnitKindId> {
        self.by_string
            .get(&(faction, string_id.to_string()))
            .copied()
    }

    /// Get role flags for a unit kind. O(1).
    ///
    /// Returns `UnitRole::empty()` if the ID is invalid.
    #[inline]
    #[must_use]
    pub fn role(&self, id: UnitKindId) -> UnitRole {
        self.get(id).map_or(UnitRole::empty(), |info| info.role)
    }

    /// Get the string ID for a unit kind. O(1).
    #[must_use]
    pub fn string_id(&self, id: UnitKindId) -> Option<&str> {
        self.get(id).map(|info| info.string_id.as_str())
    }

    /// Get the faction for a unit kind. O(1).
    #[must_use]
    pub fn faction(&self, id: UnitKindId) -> Option<FactionId> {
        self.get(id).map(|info| info.faction)
    }

    /// Get all registered unit kinds.
    pub fn all(&self) -> impl Iterator<Item = &UnitKindInfo> {
        self.by_id.iter()
    }

    /// Get all unit kinds for a specific faction.
    pub fn by_faction(&self, faction: FactionId) -> impl Iterator<Item = &UnitKindInfo> {
        self.by_id
            .iter()
            .filter(move |info| info.faction == faction)
    }

    /// Get all unit kinds matching a role.
    pub fn by_role(&self, role: UnitRole) -> impl Iterator<Item = &UnitKindInfo> {
        self.by_id
            .iter()
            .filter(move |info| info.role.contains(role))
    }

    /// Total number of registered unit kinds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// Check if registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_kind_id_basic() {
        let id = UnitKindId::new(42);
        assert_eq!(id.as_u16(), 42);
        assert!(id.is_valid());

        let none = UnitKindId::NONE;
        assert!(!none.is_valid());
    }

    #[test]
    fn test_unit_kind_id_copy() {
        let id = UnitKindId::new(10);
        let copy = id;
        assert_eq!(id, copy);
    }

    #[test]
    fn test_unit_role_empty() {
        let role = UnitRole::empty();
        assert_eq!(role.bits(), 0);
        assert!(!role.contains(UnitRole::GROUND));
        assert!(!role.intersects(UnitRole::INFANTRY));
    }

    #[test]
    fn test_unit_role_single_flag() {
        let role = UnitRole::HARVESTER;
        assert!(role.contains(UnitRole::HARVESTER));
        assert!(!role.contains(UnitRole::INFANTRY));
    }

    #[test]
    fn test_unit_role_union() {
        let role = UnitRole::GROUND.union(UnitRole::INFANTRY);
        assert!(role.contains(UnitRole::GROUND));
        assert!(role.contains(UnitRole::INFANTRY));
        assert!(role.contains(UnitRole::GROUND.union(UnitRole::INFANTRY)));
        assert!(!role.contains(UnitRole::AIR));
    }

    #[test]
    fn test_unit_role_intersects() {
        let role = UnitRole::GROUND | UnitRole::INFANTRY | UnitRole::COMBATANT;
        assert!(role.intersects(UnitRole::GROUND));
        assert!(role.intersects(UnitRole::INFANTRY | UnitRole::AIR)); // INFANTRY matches
        assert!(!role.intersects(UnitRole::AIR));
    }

    #[test]
    fn test_unit_role_bitor_operator() {
        let role = UnitRole::GROUND | UnitRole::HARVESTER;
        assert!(role.contains(UnitRole::GROUND));
        assert!(role.contains(UnitRole::HARVESTER));
    }

    #[test]
    fn test_unit_role_from_tags_basic() {
        let tags = vec!["ground".to_string(), "infantry".to_string()];
        let role = UnitRole::from_tags(&tags, 1, true);

        assert!(role.contains(UnitRole::GROUND));
        assert!(role.contains(UnitRole::INFANTRY));
        assert!(role.contains(UnitRole::TIER_1));
        assert!(role.contains(UnitRole::COMBATANT));
        assert!(!role.contains(UnitRole::AIR));
        assert!(!role.contains(UnitRole::HARVESTER));
    }

    #[test]
    fn test_unit_role_from_tags_harvester() {
        let tags = vec![
            "ground".to_string(),
            "vehicle".to_string(),
            "harvester".to_string(),
        ];
        let role = UnitRole::from_tags(&tags, 1, false);

        assert!(role.contains(UnitRole::GROUND));
        assert!(role.contains(UnitRole::VEHICLE));
        assert!(role.contains(UnitRole::HARVESTER));
        assert!(!role.contains(UnitRole::COMBATANT)); // No combat
    }

    #[test]
    fn test_unit_role_from_tags_tier2() {
        let tags = vec!["air".to_string(), "transport".to_string()];
        let role = UnitRole::from_tags(&tags, 2, false);

        assert!(role.contains(UnitRole::AIR));
        assert!(role.contains(UnitRole::TRANSPORT));
        assert!(role.contains(UnitRole::TIER_2));
        assert!(!role.contains(UnitRole::TIER_1));
    }

    #[test]
    fn test_unit_role_from_tags_unknown_ignored() {
        let tags = vec![
            "ground".to_string(),
            "unknown_future_tag".to_string(),
            "infantry".to_string(),
        ];
        let role = UnitRole::from_tags(&tags, 1, true);

        // Should still have ground and infantry
        assert!(role.contains(UnitRole::GROUND));
        assert!(role.contains(UnitRole::INFANTRY));
    }

    #[test]
    fn test_unit_role_builder_and_repair() {
        // Both "builder" and "repair" should map to BUILDER
        let tags1 = vec!["builder".to_string()];
        let tags2 = vec!["repair".to_string()];

        let role1 = UnitRole::from_tags(&tags1, 1, false);
        let role2 = UnitRole::from_tags(&tags2, 1, false);

        assert!(role1.contains(UnitRole::BUILDER));
        assert!(role2.contains(UnitRole::BUILDER));
    }

    #[test]
    fn test_registry_empty() {
        let registry = UnitKindRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = UnitKindRegistry::new();

        let id = registry.register(
            FactionId::Tinkers,
            "field_engineer",
            UnitRole::GROUND | UnitRole::INFANTRY | UnitRole::BUILDER,
        );

        assert_eq!(id.as_u16(), 0);
        assert_eq!(registry.len(), 1);

        let info = registry.get(id).unwrap();
        assert_eq!(info.id, id);
        assert_eq!(info.string_id, "field_engineer");
        assert_eq!(info.faction, FactionId::Tinkers);
        assert!(info.role.contains(UnitRole::INFANTRY));
    }

    #[test]
    fn test_registry_multiple_units() {
        let mut registry = UnitKindRegistry::new();

        let id1 = registry.register(FactionId::Tinkers, "field_engineer", UnitRole::INFANTRY);
        let id2 = registry.register(FactionId::Tinkers, "utility_hauler", UnitRole::HARVESTER);
        let id3 = registry.register(FactionId::Continuity, "security_team", UnitRole::INFANTRY);

        assert_eq!(id1.as_u16(), 0);
        assert_eq!(id2.as_u16(), 1);
        assert_eq!(id3.as_u16(), 2);
        assert_eq!(registry.len(), 3);
    }

    #[test]
    fn test_registry_find_by_string() {
        let mut registry = UnitKindRegistry::new();

        let id = registry.register(FactionId::Tinkers, "field_engineer", UnitRole::INFANTRY);

        // Find existing
        let found = registry.find(FactionId::Tinkers, "field_engineer");
        assert_eq!(found, Some(id));

        // Not found - wrong faction
        let not_found = registry.find(FactionId::Continuity, "field_engineer");
        assert!(not_found.is_none());

        // Not found - wrong string
        let not_found = registry.find(FactionId::Tinkers, "nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_registry_duplicate_registration() {
        let mut registry = UnitKindRegistry::new();

        let id1 = registry.register(FactionId::Tinkers, "field_engineer", UnitRole::INFANTRY);
        let id2 = registry.register(FactionId::Tinkers, "field_engineer", UnitRole::INFANTRY);

        // Should return same ID, not create duplicate
        assert_eq!(id1, id2);
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_registry_role_lookup() {
        let mut registry = UnitKindRegistry::new();

        let role = UnitRole::GROUND | UnitRole::HARVESTER | UnitRole::TIER_1;
        let id = registry.register(FactionId::Tinkers, "utility_hauler", role);

        assert_eq!(registry.role(id), role);
        assert_eq!(registry.role(UnitKindId::NONE), UnitRole::empty());
    }

    #[test]
    fn test_registry_string_id_lookup() {
        let mut registry = UnitKindRegistry::new();

        let id = registry.register(FactionId::Tinkers, "field_engineer", UnitRole::INFANTRY);

        assert_eq!(registry.string_id(id), Some("field_engineer"));
        assert_eq!(registry.string_id(UnitKindId::NONE), None);
    }

    #[test]
    fn test_registry_by_faction() {
        let mut registry = UnitKindRegistry::new();

        registry.register(FactionId::Tinkers, "unit1", UnitRole::INFANTRY);
        registry.register(FactionId::Tinkers, "unit2", UnitRole::HARVESTER);
        registry.register(FactionId::Continuity, "unit3", UnitRole::INFANTRY);

        let tinker_units: Vec<_> = registry.by_faction(FactionId::Tinkers).collect();
        assert_eq!(tinker_units.len(), 2);

        let continuity_units: Vec<_> = registry.by_faction(FactionId::Continuity).collect();
        assert_eq!(continuity_units.len(), 1);
    }

    #[test]
    fn test_registry_by_role() {
        let mut registry = UnitKindRegistry::new();

        registry.register(
            FactionId::Tinkers,
            "engineer",
            UnitRole::INFANTRY | UnitRole::BUILDER,
        );
        registry.register(FactionId::Tinkers, "hauler", UnitRole::HARVESTER);
        registry.register(
            FactionId::Continuity,
            "security",
            UnitRole::INFANTRY | UnitRole::COMBATANT,
        );

        let infantry: Vec<_> = registry.by_role(UnitRole::INFANTRY).collect();
        assert_eq!(infantry.len(), 2);

        let harvesters: Vec<_> = registry.by_role(UnitRole::HARVESTER).collect();
        assert_eq!(harvesters.len(), 1);
    }

    #[test]
    fn test_registry_get_invalid() {
        let registry = UnitKindRegistry::new();

        assert!(registry.get(UnitKindId::new(0)).is_none());
        assert!(registry.get(UnitKindId::new(100)).is_none());
        assert!(registry.get(UnitKindId::NONE).is_none());
    }

    #[test]
    fn test_unit_role_serialization() {
        let role = UnitRole::GROUND | UnitRole::INFANTRY | UnitRole::COMBATANT;

        // Round-trip through bits
        let bits = role.bits();
        let restored = UnitRole::from_bits(bits);
        assert_eq!(role, restored);
    }

    #[test]
    fn test_unit_kind_id_serialization() {
        use bincode;

        let id = UnitKindId::new(42);
        let encoded = bincode::serialize(&id).unwrap();
        let decoded: UnitKindId = bincode::deserialize(&encoded).unwrap();

        assert_eq!(id, decoded);
    }
}
