//! ECS component definitions.
//!
//! Components are pure data with no behavior. All game entities
//! are composed of these components.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::factions::FactionId;
use crate::math::{fixed_serde, Fixed, Vec2Fixed};

/// Unique identifier for entities.
pub type EntityId = u64;

// ============================================================================
// Combat Types
// ============================================================================

/// Damage type classification for weapons.
///
/// Each damage type has different effectiveness against various armor types,
/// creating a rock-paper-scissors dynamic in combat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DamageType {
    /// Kinetic damage (bullets, shells) - reduced by armor.
    /// Best vs unarmored, worst vs buildings.
    #[default]
    Kinetic,
    /// Energy damage (lasers, plasma) - partially ignores armor.
    /// Consistent 75% damage vs all targets.
    Energy,
    /// Explosive damage (missiles, bombs) - splash damage.
    /// Bonus damage vs buildings and heavy armor.
    Explosive,
    /// Biological damage (Sculptors faction) - special effects.
    /// Extremely effective vs unarmored, useless vs buildings.
    Biological,
}

impl DamageType {
    /// Get the damage multiplier for this damage type against an armor type.
    ///
    /// Returns a fixed-point multiplier (100 = 100% damage).
    #[must_use]
    pub fn effectiveness_vs(self, armor: ArmorType) -> Fixed {
        use ArmorType::*;
        use DamageType::*;

        let percent = match (self, armor) {
            // Kinetic: full vs unarmored, reduced by armor
            (Kinetic, Unarmored) => 100,
            (Kinetic, Light) => 75,
            (Kinetic, Heavy) => 50,
            (Kinetic, Building) => 25,

            // Energy: consistent 75% vs all (ignores armor)
            (Energy, _) => 75,

            // Explosive: weak vs soft targets, strong vs buildings
            (Explosive, Unarmored) => 50,
            (Explosive, Light) => 50,
            (Explosive, Heavy) => 75,
            (Explosive, Building) => 150,

            // Biological: strong vs unarmored, useless vs buildings
            (Biological, Unarmored) => 150,
            (Biological, Light) => 50,
            (Biological, Heavy) => 50,
            (Biological, Building) => 0,
        };

        Fixed::from_num(percent) / Fixed::from_num(100)
    }
}

/// Armor type classification for units and structures.
///
/// Determines how much damage is reduced from various damage types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ArmorType {
    /// Unarmored targets (infantry, light units).
    /// Vulnerable to kinetic and biological damage.
    #[default]
    Unarmored,
    /// Light armor (light vehicles, scouts).
    /// Moderate protection against most damage types.
    Light,
    /// Heavy armor (tanks, mechs, heavy vehicles).
    /// Strong protection against kinetic, vulnerable to explosive.
    Heavy,
    /// Building armor (structures, turrets, walls).
    /// Very strong vs kinetic, vulnerable to explosive.
    Building,
}

impl ArmorType {
    /// Get the armor type appropriate for a unit type.
    #[must_use]
    pub fn from_unit_type(unit_type: &super::components::UnitType) -> Self {
        use super::components::UnitType;
        match unit_type {
            UnitType::Infantry => ArmorType::Unarmored,
            UnitType::Vehicle => ArmorType::Light,
            UnitType::Mech => ArmorType::Heavy,
            UnitType::Aircraft => ArmorType::Light,
            UnitType::Structure => ArmorType::Building,
        }
    }
}

/// Position component in world space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    /// World position.
    pub value: Vec2Fixed,
}

impl Position {
    /// Create a new position at the given coordinates.
    #[must_use]
    pub const fn new(value: Vec2Fixed) -> Self {
        Self { value }
    }

    /// Create a position at the origin.
    pub const ORIGIN: Self = Self {
        value: Vec2Fixed::ZERO,
    };
}

/// Velocity component for moving entities.
///
/// Represents the direction and speed of movement using fixed-point math
/// for deterministic simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Velocity {
    /// Velocity vector (units per tick).
    pub value: Vec2Fixed,
}

impl Velocity {
    /// Create a new velocity.
    #[must_use]
    pub const fn new(value: Vec2Fixed) -> Self {
        Self { value }
    }

    /// Zero velocity (stationary).
    pub const ZERO: Self = Self {
        value: Vec2Fixed::ZERO,
    };

    /// Check if the entity is stationary.
    #[must_use]
    pub fn is_stationary(&self) -> bool {
        self.value.x == Fixed::ZERO && self.value.y == Fixed::ZERO
    }
}

/// Type classification for units.
///
/// Determines movement capabilities, terrain interaction, and
/// what weapons can target this unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnitType {
    /// Ground infantry units - slow, can traverse rough terrain.
    Infantry,
    /// Ground vehicles - faster, restricted to roads/open terrain.
    Vehicle,
    /// Mechs - versatile bipedal units, can traverse most terrain.
    Mech,
    /// Aircraft - ignore ground terrain, require anti-air to counter.
    Aircraft,
    /// Stationary structures - buildings, turrets, walls.
    Structure,
}

impl UnitType {
    /// Check if this unit type can fly.
    #[must_use]
    pub const fn is_airborne(&self) -> bool {
        matches!(self, Self::Aircraft)
    }

    /// Check if this unit type is mobile.
    #[must_use]
    pub const fn is_mobile(&self) -> bool {
        !matches!(self, Self::Structure)
    }

    /// Check if this is a ground unit.
    #[must_use]
    pub const fn is_ground(&self) -> bool {
        matches!(self, Self::Infantry | Self::Vehicle | Self::Mech)
    }
}

/// Component linking an entity to a faction.
///
/// All controllable entities belong to a faction and player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactionMember {
    /// The faction this entity belongs to.
    pub faction: FactionId,
    /// Player index within the faction (for team games).
    pub player_index: u8,
}

impl FactionMember {
    /// Create a new faction membership.
    #[must_use]
    pub const fn new(faction: FactionId, player_index: u8) -> Self {
        Self {
            faction,
            player_index,
        }
    }

    /// Check if two entities are allies (same faction).
    #[must_use]
    pub const fn is_allied_with(&self, other: &Self) -> bool {
        matches!(
            (&self.faction, &other.faction),
            (FactionId::Continuity, FactionId::Continuity)
                | (FactionId::Collegium, FactionId::Collegium)
                | (FactionId::Tinkers, FactionId::Tinkers)
                | (FactionId::BioSovereigns, FactionId::BioSovereigns)
                | (FactionId::Zephyr, FactionId::Zephyr)
        )
    }
}

/// Marker component for entities that can be selected by the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Selectable;

/// Marker component for entities currently selected by the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Selected;

/// A command that can be issued to a unit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    /// Move to a target position.
    MoveTo(Vec2Fixed),
    /// Attack a specific entity.
    Attack(EntityId),
    /// Attack-move to a position (engage enemies along the way).
    AttackMove(Vec2Fixed),
    /// Hold position and engage nearby enemies.
    HoldPosition,
    /// Stop all actions.
    Stop,
    /// Patrol between current position and target.
    Patrol(Vec2Fixed),
    /// Follow another unit.
    Follow(EntityId),
    /// Guard another unit (attack anything that attacks it).
    Guard(EntityId),
}

/// Queue of commands for a unit to execute.
///
/// Commands are executed in order. Units process the front command
/// until complete, then move to the next.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CommandQueue {
    /// The queue of pending commands.
    pub commands: VecDeque<Command>,
}

impl CommandQueue {
    /// Create an empty command queue.
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: VecDeque::new(),
        }
    }

    /// Add a command to the back of the queue.
    pub fn push(&mut self, command: Command) {
        self.commands.push_back(command);
    }

    /// Replace all commands with a single new command.
    pub fn set(&mut self, command: Command) {
        self.commands.clear();
        self.commands.push_back(command);
    }

    /// Get the current command being executed.
    #[must_use]
    pub fn current(&self) -> Option<&Command> {
        self.commands.front()
    }

    /// Remove and return the current command (when completed).
    pub fn pop(&mut self) -> Option<Command> {
        self.commands.pop_front()
    }

    /// Clear all commands.
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// Check if the queue is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Get the number of queued commands.
    #[must_use]
    pub fn len(&self) -> usize {
        self.commands.len()
    }
}

/// Component for tracking the current attack target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttackTarget {
    /// The entity being targeted for attack.
    pub target: Option<EntityId>,
    /// Ticks until the unit can attack again.
    pub cooldown: u32,
}

impl AttackTarget {
    /// Create a new attack target component with no target.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            target: None,
            cooldown: 0,
        }
    }

    /// Create with a specific target.
    #[must_use]
    pub const fn with_target(target: EntityId) -> Self {
        Self {
            target: Some(target),
            cooldown: 0,
        }
    }

    /// Check if ready to attack.
    #[must_use]
    pub const fn can_attack(&self) -> bool {
        self.cooldown == 0
    }

    /// Tick down the cooldown.
    pub fn tick(&mut self) {
        if self.cooldown > 0 {
            self.cooldown -= 1;
        }
    }

    /// Clear the current target.
    pub fn clear(&mut self) {
        self.target = None;
    }
}

impl Default for AttackTarget {
    fn default() -> Self {
        Self::new()
    }
}

/// Movement component for mobile units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Movement {
    /// Movement speed in units per tick.
    #[serde(with = "fixed_serde")]
    pub speed: Fixed,
    /// Current movement target (if any).
    pub target: Option<Vec2Fixed>,
}

/// Component tracking patrol behavior between two points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatrolState {
    /// Patrol origin (starting point).
    pub origin: Vec2Fixed,
    /// Patrol target (destination point).
    pub target: Vec2Fixed,
    /// Whether the unit is heading toward the target.
    pub heading_to_target: bool,
}

/// Health component for damageable entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Health {
    /// Current health points.
    pub current: u32,
    /// Maximum health points.
    pub max: u32,
}

impl Health {
    /// Create new health component at full health.
    #[must_use]
    pub const fn new(max: u32) -> Self {
        Self { current: max, max }
    }

    /// Check if entity is dead (health == 0).
    #[must_use]
    pub const fn is_dead(&self) -> bool {
        self.current == 0
    }

    /// Check if entity is at full health.
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.current >= self.max
    }

    /// Apply damage, returning actual damage dealt.
    /// Uses saturating subtraction to prevent underflow.
    pub fn apply_damage(&mut self, amount: u32) -> u32 {
        let actual = amount.min(self.current);
        self.current = self.current.saturating_sub(actual);
        actual
    }

    /// Heal the entity, returning actual amount healed.
    /// Uses saturating addition to prevent overflow.
    pub fn heal(&mut self, amount: u32) -> u32 {
        let headroom = self.max.saturating_sub(self.current);
        let actual = amount.min(headroom);
        self.current = self.current.saturating_add(actual);
        actual
    }

    /// Get health as a percentage (0-100).
    #[must_use]
    pub fn percentage(&self) -> u32 {
        if self.max == 0 {
            0
        } else {
            (self.current * 100) / self.max
        }
    }
}

/// Faction ownership component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Owned {
    /// Owning faction.
    pub faction: FactionId,
    /// Owning player within faction.
    pub player: u8,
}

/// Combat stats component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatStats {
    /// Base attack damage.
    pub damage: u32,
    /// Type of damage this unit deals.
    pub damage_type: DamageType,
    /// Type of armor this unit has.
    pub armor_type: ArmorType,
    /// Flat armor reduction (applied after multipliers).
    pub armor_value: u32,
    /// Attack range in world units.
    #[serde(with = "fixed_serde")]
    pub range: Fixed,
    /// Attack cooldown in ticks.
    pub attack_cooldown: u32,
    /// Current cooldown remaining.
    pub cooldown_remaining: u32,
    /// Projectile speed (0 = instant/hitscan).
    #[serde(with = "fixed_serde")]
    pub projectile_speed: Fixed,
}

impl CombatStats {
    /// Create new combat stats with default types.
    #[must_use]
    pub fn new(damage: u32, range: Fixed, attack_cooldown: u32) -> Self {
        Self {
            damage,
            damage_type: DamageType::Kinetic,
            armor_type: ArmorType::Unarmored,
            armor_value: 0,
            range,
            attack_cooldown,
            cooldown_remaining: 0,
            projectile_speed: Fixed::ZERO,
        }
    }

    /// Builder method to set damage type.
    #[must_use]
    pub const fn with_damage_type(mut self, damage_type: DamageType) -> Self {
        self.damage_type = damage_type;
        self
    }

    /// Builder method to set armor type and value.
    #[must_use]
    pub const fn with_armor(mut self, armor_type: ArmorType, armor_value: u32) -> Self {
        self.armor_type = armor_type;
        self.armor_value = armor_value;
        self
    }

    /// Builder method to set projectile speed.
    #[must_use]
    pub fn with_projectile_speed(mut self, speed: Fixed) -> Self {
        self.projectile_speed = speed;
        self
    }

    /// Check if this unit uses projectiles (non-instant attacks).
    #[must_use]
    pub fn uses_projectiles(&self) -> bool {
        self.projectile_speed > Fixed::ZERO
    }

    /// Check if ready to attack.
    #[must_use]
    pub const fn can_attack(&self) -> bool {
        self.cooldown_remaining == 0
    }

    /// Reset cooldown after attacking.
    pub fn reset_cooldown(&mut self) {
        self.cooldown_remaining = self.attack_cooldown;
    }

    /// Tick down the cooldown by one.
    pub fn tick_cooldown(&mut self) {
        if self.cooldown_remaining > 0 {
            self.cooldown_remaining -= 1;
        }
    }
}

impl Default for CombatStats {
    fn default() -> Self {
        Self {
            damage: 10,
            damage_type: DamageType::Kinetic,
            armor_type: ArmorType::Unarmored,
            armor_value: 0,
            range: Fixed::from_num(5),
            attack_cooldown: 30,
            cooldown_remaining: 0,
            projectile_speed: Fixed::ZERO,
        }
    }
}

// ============================================================================
// Projectile Component
// ============================================================================

/// A projectile entity traveling toward a target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Projectile {
    /// Entity that fired this projectile.
    pub source: EntityId,
    /// Target entity.
    pub target: EntityId,
    /// Damage to deal on impact.
    pub damage: u32,
    /// Type of damage.
    pub damage_type: DamageType,
    /// Travel speed per tick.
    #[serde(with = "fixed_serde")]
    pub speed: Fixed,
}

impl Projectile {
    /// Create a new projectile.
    #[must_use]
    pub fn new(
        source: EntityId,
        target: EntityId,
        damage: u32,
        damage_type: DamageType,
        speed: Fixed,
    ) -> Self {
        Self {
            source,
            target,
            damage,
            damage_type,
            speed,
        }
    }
}

/// Marker component for buildings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Building {
    /// Building footprint width.
    pub width: u8,
    /// Building footprint height.
    pub height: u8,
}

/// An item being produced in a production queue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionItem {
    /// Identifier for what's being produced.
    pub unit_id: String,
    /// Total ticks required to produce this item.
    pub build_time: u32,
}

/// Production queue component for buildings that produce units.
///
/// Buildings with this component can queue up units/items for production.
/// The first item in the queue is actively being produced.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ProductionQueue {
    /// Items waiting to be produced.
    pub queue: VecDeque<ProductionItem>,
    /// Current production progress in ticks (0 to item's build_time).
    pub progress: u32,
    /// Rally point for produced units.
    pub rally_point: Option<Vec2Fixed>,
}

impl ProductionQueue {
    /// Create an empty production queue.
    #[must_use]
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            progress: 0,
            rally_point: None,
        }
    }

    /// Create a production queue with a rally point.
    #[must_use]
    pub fn with_rally_point(rally_point: Vec2Fixed) -> Self {
        Self {
            queue: VecDeque::new(),
            progress: 0,
            rally_point: Some(rally_point),
        }
    }

    /// Add an item to the production queue.
    pub fn enqueue(&mut self, unit_id: String, build_time: u32) {
        self.queue.push_back(ProductionItem {
            unit_id,
            build_time,
        });
    }

    /// Get the currently producing item.
    #[must_use]
    pub fn current(&self) -> Option<&ProductionItem> {
        self.queue.front()
    }

    /// Check if production is complete for the current item.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.queue
            .front()
            .map(|item| self.progress >= item.build_time)
            .unwrap_or(false)
    }

    /// Complete the current item and return it.
    pub fn complete(&mut self) -> Option<ProductionItem> {
        if self.is_complete() {
            self.progress = 0;
            self.queue.pop_front()
        } else {
            None
        }
    }

    /// Advance production by one tick.
    pub fn tick(&mut self) {
        if !self.queue.is_empty() {
            self.progress += 1;
        }
    }

    /// Check if the queue is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Get the number of items in the queue.
    #[must_use]
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Cancel all production.
    pub fn clear(&mut self) {
        self.queue.clear();
        self.progress = 0;
    }
}
