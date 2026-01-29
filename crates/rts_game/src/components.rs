//! Game component wrappers for Bevy.
//!
//! Since rts_core components are designed for deterministic simulation and
//! don't derive Bevy's Component trait, this module provides wrapper
//! components that bridge the simulation layer to the rendering layer.

use bevy::prelude::*;
use rts_core::components::Command as CoreCommand;
use rts_core::factions::FactionId;
use rts_core::math::Vec2Fixed;
use rts_core::unit_kind::{UnitKindId, UnitRole};

// ============================================================================
// Core Component Wrappers
// ============================================================================

/// Wrapper for rts_core::Position that implements Bevy Component.
///
/// This bridges the simulation's fixed-point positions to the render layer.
#[derive(Component, Debug, Clone, Copy)]
pub struct GamePosition {
    /// The fixed-point position from the simulation.
    pub value: Vec2Fixed,
}

impl GamePosition {
    /// Create a new game position.
    #[must_use]
    pub const fn new(value: Vec2Fixed) -> Self {
        Self { value }
    }

    /// Create a position at the origin.
    pub const ORIGIN: Self = Self {
        value: Vec2Fixed::ZERO,
    };

    /// Convert to Bevy Vec2 for rendering.
    #[must_use]
    pub fn as_vec2(&self) -> Vec2 {
        Vec2::new(self.value.x.to_num(), self.value.y.to_num())
    }
}

/// Wrapper for health that implements Bevy Component.
#[derive(Component, Debug, Clone, Copy)]
pub struct GameHealth {
    /// Current health points.
    pub current: u32,
    /// Maximum health points.
    pub max: u32,
}

impl GameHealth {
    /// Create new health at full.
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

    /// Get health as a ratio (0.0 to 1.0).
    #[must_use]
    pub fn ratio(&self) -> f32 {
        if self.max == 0 {
            0.0
        } else {
            self.current as f32 / self.max as f32
        }
    }

    /// Heal this entity by the given amount, capped at max.
    pub fn heal(&mut self, amount: u32) {
        self.current = self.current.saturating_add(amount).min(self.max);
    }

    /// Apply damage, returning actual damage dealt.
    pub fn apply_damage(&mut self, amount: u32) -> u32 {
        let actual = amount.min(self.current);
        self.current = self.current.saturating_sub(actual);
        actual
    }
}

/// Component for health regeneration over time.
#[derive(Component, Debug, Clone, Copy)]
pub struct Regeneration {
    /// Health points regenerated per second.
    pub per_second: f32,
    /// Accumulated partial HP (for slow regen rates).
    pub accumulator: f32,
}

impl Regeneration {
    /// Create a new regeneration component.
    #[must_use]
    pub fn new(per_second: f32) -> Self {
        Self {
            per_second,
            accumulator: 0.0,
        }
    }
}

/// Marker component for entities that can be selected.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Selectable;

/// Component tracking the current movement target.
#[derive(Component, Debug, Clone, Copy)]
pub struct MovementTarget {
    /// The target position to move toward.
    pub target: Vec2Fixed,
}

/// Component tracking patrol behavior between two points.
#[derive(Component, Debug, Clone, Copy)]
pub struct PatrolState {
    /// Patrol origin (starting point).
    pub origin: Vec2Fixed,
    /// Patrol target (destination point).
    pub target: Vec2Fixed,
    /// Whether the unit is heading toward the target.
    pub heading_to_target: bool,
}

/// Marker component for entities currently selected.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Selected;

/// Component for faction membership.
#[derive(Component, Debug, Clone, Copy)]
pub struct GameFaction {
    /// The faction this entity belongs to.
    pub faction: FactionId,
}

/// Resource tracking which faction the local player controls.
#[derive(Resource, Debug, Clone, Copy)]
pub struct PlayerFaction {
    /// The faction controlled by the local player.
    pub faction: FactionId,
}

impl Default for PlayerFaction {
    fn default() -> Self {
        // Allow faction selection via environment variable for testing
        // RTS_FACTION=collegium cargo run
        let faction = std::env::var("RTS_FACTION")
            .ok()
            .and_then(|s| match s.to_lowercase().as_str() {
                "continuity" | "1" => Some(FactionId::Continuity),
                "collegium" | "2" => Some(FactionId::Collegium),
                "tinkers" | "3" => Some(FactionId::Tinkers),
                "biosovereigns" | "sculptors" | "4" => Some(FactionId::BioSovereigns),
                "zephyr" | "5" => Some(FactionId::Zephyr),
                _ => None,
            })
            .unwrap_or(FactionId::Continuity);

        Self { faction }
    }
}

impl PlayerFaction {
    /// Create with a specific faction.
    #[must_use]
    pub fn new(faction: FactionId) -> Self {
        Self { faction }
    }
}

/// Command queue wrapper for Bevy.
#[derive(Component, Debug, Clone, Default)]
pub struct GameCommandQueue {
    /// Pending commands.
    pub commands: std::collections::VecDeque<CoreCommand>,
}

impl GameCommandQueue {
    /// Create an empty command queue.
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: std::collections::VecDeque::new(),
        }
    }

    /// Add a command to the back of the queue.
    pub fn push(&mut self, command: CoreCommand) {
        self.commands.push_back(command);
    }

    /// Replace all commands with a single new command.
    pub fn set(&mut self, command: CoreCommand) {
        self.commands.clear();
        self.commands.push_back(command);
    }

    /// Pop the next command from the front.
    pub fn pop(&mut self) -> Option<CoreCommand> {
        self.commands.pop_front()
    }

    /// Peek at the current command without removing it.
    #[must_use]
    pub fn current(&self) -> Option<&CoreCommand> {
        self.commands.front()
    }
}

// ============================================================================
// Economy Components
// ============================================================================

/// Type of resource node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResourceNodeType {
    /// Permanent node near bases - infinite but yield degrades with harvester count.
    /// Provides safe baseline income but incentivizes expansion.
    Permanent,
    /// Temporary field node - high yield but depletes.
    /// Forces conflict and map control.
    #[default]
    Temporary,
}

/// Component for resource nodes that harvesters can gather from.
#[derive(Component, Debug, Clone, Copy)]
pub struct GameResourceNode {
    /// Type of this node (permanent or temporary).
    pub node_type: ResourceNodeType,
    /// Remaining feedstock in this node (ignored for Permanent nodes).
    pub remaining: i32,
    /// Base amount gathered per harvest tick.
    pub base_gather_rate: i32,
    /// Optimal number of harvesters before yield degradation (Permanent only).
    pub optimal_harvesters: u8,
    /// Current number of harvesters working this node (updated each frame).
    pub current_harvesters: u8,
}

impl GameResourceNode {
    /// Create a temporary (depletable) resource node.
    #[must_use]
    pub const fn temporary(remaining: i32, gather_rate: i32) -> Self {
        Self {
            node_type: ResourceNodeType::Temporary,
            remaining,
            base_gather_rate: gather_rate,
            optimal_harvesters: 255, // No limit for temporary
            current_harvesters: 0,
        }
    }

    /// Create a permanent (infinite but degrading) resource node.
    #[must_use]
    pub const fn permanent(gather_rate: i32, optimal_harvesters: u8) -> Self {
        Self {
            node_type: ResourceNodeType::Permanent,
            remaining: i32::MAX, // Effectively infinite
            base_gather_rate: gather_rate,
            optimal_harvesters,
            current_harvesters: 0,
        }
    }

    /// Check if this node is depleted.
    #[must_use]
    pub fn is_depleted(&self) -> bool {
        match self.node_type {
            ResourceNodeType::Permanent => false, // Never depletes
            ResourceNodeType::Temporary => self.remaining <= 0,
        }
    }

    /// Get effective gather rate accounting for harvester crowding.
    #[must_use]
    pub fn effective_gather_rate(&self) -> i32 {
        match self.node_type {
            ResourceNodeType::Permanent => {
                // Yield degrades when over optimal harvester count
                let excess = self
                    .current_harvesters
                    .saturating_sub(self.optimal_harvesters);
                if excess == 0 {
                    self.base_gather_rate
                } else {
                    // Each excess harvester halves the per-harvester yield
                    // Formula: base_rate / (1 + excess)
                    self.base_gather_rate / (1 + excess as i32)
                }
            }
            ResourceNodeType::Temporary => self.base_gather_rate, // Full rate always
        }
    }

    /// Extract resources from this node.
    pub fn extract(&mut self, amount: i32) -> i32 {
        match self.node_type {
            ResourceNodeType::Permanent => {
                // Permanent nodes always yield (based on effective rate)
                amount.min(self.effective_gather_rate())
            }
            ResourceNodeType::Temporary => {
                let extracted = amount.min(self.remaining);
                self.remaining -= extracted;
                extracted
            }
        }
    }
}

/// State of a harvester unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GameHarvesterState {
    /// Idle, waiting for orders or auto-assignment.
    #[default]
    Idle,
    /// Moving to a resource node.
    MovingToNode(Entity),
    /// Actively gathering from a node.
    Gathering(Entity),
    /// Returning to a depot to deposit resources.
    Returning(Entity),
    /// Depositing resources at a depot.
    Depositing,
}

/// Component for harvester units that gather resources.
#[derive(Component, Debug, Clone, Copy)]
pub struct GameHarvester {
    /// Maximum feedstock this harvester can carry.
    pub capacity: i32,
    /// Current load of feedstock being carried.
    pub current_load: i32,
    /// Amount gathered per harvest tick.
    pub gather_rate: i32,
    /// Current state of the harvester.
    pub state: GameHarvesterState,
    /// Ticks between gather actions.
    pub gather_cooldown: u32,
    /// Current cooldown counter.
    pub cooldown_timer: u32,
    /// Manually assigned node to return to (if any).
    pub assigned_node: Option<Entity>,
}

impl GameHarvester {
    /// Create a new harvester with the given capacity and gather rate.
    #[must_use]
    pub const fn new(capacity: i32, gather_rate: i32) -> Self {
        Self {
            capacity,
            current_load: 0,
            gather_rate,
            state: GameHarvesterState::Idle,
            gather_cooldown: 30, // ~0.5 seconds at 60fps
            cooldown_timer: 0,
            assigned_node: None,
        }
    }

    /// Check if the harvester is full.
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.current_load >= self.capacity
    }

    /// Check if the harvester is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.current_load <= 0
    }

    /// Available cargo space.
    #[must_use]
    pub const fn available_capacity(&self) -> i32 {
        self.capacity - self.current_load
    }

    /// Load resources into the harvester.
    pub fn load(&mut self, amount: i32) -> i32 {
        let space = self.available_capacity();
        let loaded = amount.min(space);
        self.current_load += loaded;
        loaded
    }

    /// Unload all resources from the harvester.
    pub fn unload(&mut self) -> i32 {
        let amount = self.current_load;
        self.current_load = 0;
        amount
    }
}

/// Marker component for depot buildings that accept resource deposits.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct GameDepot;

/// Marker component for entities that should not be affected by unit separation.
/// Used for buildings, resource nodes, and other stationary objects.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Stationary;

/// Collision shape for buildings and terrain obstacles.
/// Units will be pushed away from entities with this component.
#[derive(Component, Debug, Clone, Copy)]
pub struct Collider {
    /// Half-width of collision box.
    pub half_width: f32,
    /// Half-height of collision box.
    pub half_height: f32,
}

impl Collider {
    /// Create a new rectangular collider.
    #[must_use]
    pub const fn new(width: f32, height: f32) -> Self {
        Self {
            half_width: width / 2.0,
            half_height: height / 2.0,
        }
    }

    /// Check if a point is inside this collider (centered at origin).
    #[must_use]
    pub fn contains_point(&self, point: Vec2, collider_pos: Vec2) -> bool {
        let rel = point - collider_pos;
        rel.x.abs() < self.half_width && rel.y.abs() < self.half_height
    }

    /// Get push vector to eject a point from this collider.
    #[must_use]
    pub fn push_out(&self, point: Vec2, collider_pos: Vec2, margin: f32) -> Option<Vec2> {
        let rel = point - collider_pos;
        let padded_hw = self.half_width + margin;
        let padded_hh = self.half_height + margin;

        // Check if point is inside padded bounds
        if rel.x.abs() < padded_hw && rel.y.abs() < padded_hh {
            // Find shortest escape direction
            let escape_x = if rel.x >= 0.0 {
                padded_hw - rel.x
            } else {
                -padded_hw - rel.x
            };
            let escape_y = if rel.y >= 0.0 {
                padded_hh - rel.y
            } else {
                -padded_hh - rel.y
            };

            if escape_x.abs() < escape_y.abs() {
                Some(Vec2::new(escape_x, 0.0))
            } else {
                Some(Vec2::new(0.0, escape_y))
            }
        } else {
            None
        }
    }
}

// ============================================================================
// Combat Components
// ============================================================================

/// Type of damage dealt by attacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DamageType {
    /// Standard kinetic damage (bullets, melee).
    #[default]
    Kinetic,
    /// Energy-based damage (lasers, plasma).
    Energy,
    /// Explosive damage (rockets, grenades).
    Explosive,
}

/// Type of armor protecting a unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ArmorType {
    /// No armor (soft targets like infantry).
    #[default]
    Unarmored,
    /// Light armor (scouts, light vehicles).
    Light,
    /// Heavy armor (tanks, mechs).
    Heavy,
    /// Building armor.
    Structure,
}

impl ArmorType {
    /// Get damage multiplier against this armor type.
    #[must_use]
    pub fn damage_modifier(&self, damage_type: DamageType) -> f32 {
        match (damage_type, self) {
            // Kinetic: good vs unarmored, poor vs heavy
            (DamageType::Kinetic, ArmorType::Unarmored) => 1.25,
            (DamageType::Kinetic, ArmorType::Light) => 1.0,
            (DamageType::Kinetic, ArmorType::Heavy) => 0.5,
            (DamageType::Kinetic, ArmorType::Structure) => 0.5, // Was 0.25, now 0.5 for better balance

            // Energy: consistent across targets
            (DamageType::Energy, ArmorType::Unarmored) => 1.0,
            (DamageType::Energy, ArmorType::Light) => 1.0,
            (DamageType::Energy, ArmorType::Heavy) => 0.75,
            (DamageType::Energy, ArmorType::Structure) => 0.75,

            // Explosive: good vs everything, great vs structures
            (DamageType::Explosive, ArmorType::Unarmored) => 1.0,
            (DamageType::Explosive, ArmorType::Light) => 1.25,
            (DamageType::Explosive, ArmorType::Heavy) => 1.0,
            (DamageType::Explosive, ArmorType::Structure) => 1.5,
        }
    }

    /// Get default armor value for this type.
    #[must_use]
    pub const fn base_armor(&self) -> u32 {
        match self {
            Self::Unarmored => 0,
            Self::Light => 5,
            Self::Heavy => 15,
            Self::Structure => 20,
        }
    }
}

/// Component for units that can attack.
#[derive(Component, Debug, Clone, Copy)]
pub struct CombatStats {
    /// Base damage per attack.
    pub damage: u32,
    /// Type of damage dealt.
    pub damage_type: DamageType,
    /// Attack range in world units.
    pub range: f32,
    /// Seconds between attacks.
    pub attack_cooldown: f32,
    /// Current cooldown timer (counts down to 0).
    pub cooldown_timer: f32,
}

impl CombatStats {
    /// Create new combat stats.
    #[must_use]
    pub const fn new(
        damage: u32,
        damage_type: DamageType,
        range: f32,
        attack_cooldown: f32,
    ) -> Self {
        Self {
            damage,
            damage_type,
            range,
            attack_cooldown,
            cooldown_timer: 0.0,
        }
    }

    /// Check if the unit can attack (cooldown ready).
    #[must_use]
    pub fn can_attack(&self) -> bool {
        self.cooldown_timer <= 0.0
    }

    /// Start the attack cooldown.
    pub fn start_cooldown(&mut self) {
        self.cooldown_timer = self.attack_cooldown;
    }

    /// Update cooldown timer.
    pub fn tick_cooldown(&mut self, dt: f32) {
        self.cooldown_timer = (self.cooldown_timer - dt).max(0.0);
    }
}

/// Component for units that can be damaged (armor type and value).
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Armor {
    /// Type of armor this unit has.
    pub armor_type: ArmorType,
    /// Flat armor value that reduces incoming damage.
    pub value: u32,
}

impl Armor {
    /// Create a new armor component with default value for the type.
    #[must_use]
    pub const fn new(armor_type: ArmorType) -> Self {
        Self {
            armor_type,
            value: armor_type.base_armor(),
        }
    }

    /// Create a new armor component with specific value.
    #[must_use]
    pub const fn new_with_value(armor_type: ArmorType, value: u32) -> Self {
        Self { armor_type, value }
    }
}

/// Component tracking the current attack target.
#[derive(Component, Debug, Clone, Copy)]
pub struct AttackTarget {
    /// The entity being attacked.
    pub target: Entity,
}

/// Marker component for dead entities awaiting cleanup.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Dead;

// ============================================================================
// Building Components
// ============================================================================

/// Type of building that can be constructed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildingType {
    /// Main base / HQ - produces harvesters, provides supply.
    Depot,
    /// Barracks - produces infantry and rangers.
    Barracks,
    /// Supply Depot - provides additional supply capacity.
    SupplyDepot,
    /// Tech Lab - unlocks advanced units and upgrades.
    TechLab,
    /// Turret - defensive structure.
    Turret,
}

impl BuildingType {
    /// Get the feedstock cost for this building type.
    #[must_use]
    pub const fn cost(&self) -> i32 {
        match self {
            Self::Depot => 400,
            Self::Barracks => 150,
            Self::SupplyDepot => 100,
            Self::TechLab => 200,
            Self::Turret => 75,
        }
    }

    /// Get the construction time in seconds.
    #[must_use]
    pub const fn build_time(&self) -> f32 {
        match self {
            Self::Depot => 60.0,
            Self::Barracks => 30.0,
            Self::SupplyDepot => 20.0,
            Self::TechLab => 45.0,
            Self::Turret => 15.0,
        }
    }

    /// Get the display name for this building type.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Depot => "Depot",
            Self::Barracks => "Barracks",
            Self::SupplyDepot => "Supply Depot",
            Self::TechLab => "Tech Lab",
            Self::Turret => "Turret",
        }
    }

    /// Get the supply provided by this building.
    #[must_use]
    pub const fn supply_provided(&self) -> i32 {
        match self {
            Self::Depot => 10,
            Self::SupplyDepot => 8,
            _ => 0,
        }
    }

    /// Get the health of this building type.
    #[must_use]
    pub const fn health(&self) -> u32 {
        match self {
            Self::Depot => 500,
            Self::Barracks => 350,
            Self::SupplyDepot => 200,
            Self::TechLab => 250,
            Self::Turret => 150,
        }
    }

    /// Get the required building (tech requirement) to construct this.
    #[must_use]
    pub const fn requires(&self) -> Option<BuildingType> {
        match self {
            Self::Depot => None,                   // Can always build depot (expansion)
            Self::Barracks => None,                // Basic building
            Self::SupplyDepot => None,             // Basic building
            Self::TechLab => Some(Self::Barracks), // Requires Barracks
            Self::Turret => Some(Self::Barracks),  // Requires Barracks
        }
    }

    /// Get the size of this building (for placement).
    #[must_use]
    pub const fn size(&self) -> (f32, f32) {
        match self {
            Self::Depot => (64.0, 64.0),
            Self::Barracks => (48.0, 48.0),
            Self::SupplyDepot => (32.0, 32.0),
            Self::TechLab => (40.0, 40.0),
            Self::Turret => (24.0, 24.0),
        }
    }

    /// Check if this building can produce units.
    #[must_use]
    pub const fn can_produce_units(&self) -> bool {
        matches!(self, Self::Depot | Self::Barracks)
    }
}

/// Component for buildings under construction.
#[derive(Component, Debug, Clone)]
pub struct UnderConstruction {
    /// Type of building being built.
    pub building_type: BuildingType,
    /// Progress toward completion (0.0 to 1.0).
    pub progress: f32,
    /// Total construction time.
    pub total_time: f32,
}

impl UnderConstruction {
    /// Create a new under-construction marker.
    #[must_use]
    pub fn new(building_type: BuildingType) -> Self {
        Self {
            building_type,
            progress: 0.0,
            total_time: building_type.build_time(),
        }
    }

    /// Check if construction is complete.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.progress >= 1.0
    }

    /// Advance construction by delta time.
    pub fn advance(&mut self, dt: f32) {
        self.progress = (self.progress + dt / self.total_time).min(1.0);
    }
}

/// Component marking what type of building this is.
#[derive(Component, Debug, Clone, Copy)]
pub struct Building {
    /// The type of building.
    pub building_type: BuildingType,
}

impl Building {
    /// Create a new building component.
    #[must_use]
    pub const fn new(building_type: BuildingType) -> Self {
        Self { building_type }
    }
}

/// Resource tracking owned buildings per faction.
#[derive(Resource, Debug, Clone, Default)]
pub struct FactionBuildings {
    /// Buildings owned by each faction.
    pub buildings: std::collections::HashMap<FactionId, Vec<BuildingType>>,
}

impl FactionBuildings {
    /// Check if a faction has a specific building type.
    #[must_use]
    pub fn has_building(&self, faction: FactionId, building_type: BuildingType) -> bool {
        self.buildings
            .get(&faction)
            .map(|b| b.contains(&building_type))
            .unwrap_or(false)
    }

    /// Add a building to a faction's list.
    pub fn add_building(&mut self, faction: FactionId, building_type: BuildingType) {
        self.buildings
            .entry(faction)
            .or_default()
            .push(building_type);
    }

    /// Remove a building from a faction's list.
    pub fn remove_building(&mut self, faction: FactionId, building_type: BuildingType) {
        if let Some(buildings) = self.buildings.get_mut(&faction) {
            if let Some(pos) = buildings.iter().position(|&b| b == building_type) {
                buildings.remove(pos);
            }
        }
    }

    /// Check if a faction can build a specific building type (tech requirements met).
    #[must_use]
    pub fn can_build(&self, faction: FactionId, building_type: BuildingType) -> bool {
        match building_type.requires() {
            None => true,
            Some(required) => self.has_building(faction, required),
        }
    }
}

// ============================================================================
// Production Components
// ============================================================================

/// Component tracking which unit type from RON data this entity is.
///
/// Stores the string ID matching the UnitData id field in faction RON files.
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnitDataId(pub String);

impl UnitDataId {
    /// Create a new unit data ID.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the ID string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Type of unit that can be produced.
///
/// This enum provides backwards compatibility while we transition to
/// fully data-driven unit spawning. Maps to faction-specific RON unit IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitType {
    /// Basic infantry unit.
    Infantry,
    /// Harvester unit for gathering resources.
    Harvester,
    /// Ranged attack unit.
    Ranger,
}

impl UnitType {
    /// Get the feedstock cost for this unit type.
    #[must_use]
    pub const fn cost(&self) -> i32 {
        match self {
            Self::Infantry => 50,
            Self::Harvester => 75,
            Self::Ranger => 100,
        }
    }

    /// Get the build time in seconds for this unit type.
    #[must_use]
    pub const fn build_time(&self) -> f32 {
        match self {
            Self::Infantry => 5.0,
            Self::Harvester => 8.0,
            Self::Ranger => 10.0,
        }
    }

    /// Get the display name for this unit type.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Infantry => "Infantry",
            Self::Harvester => "Harvester",
            Self::Ranger => "Ranger",
        }
    }

    /// Get the supply cost for this unit type.
    #[must_use]
    pub const fn supply(&self) -> i32 {
        match self {
            Self::Infantry => 1,
            Self::Harvester => 2,
            Self::Ranger => 2,
        }
    }

    /// Get the required building to produce this unit.
    #[must_use]
    pub const fn required_building(&self) -> BuildingType {
        match self {
            Self::Infantry => BuildingType::Barracks,
            Self::Harvester => BuildingType::Depot,
            Self::Ranger => BuildingType::Barracks,
        }
    }

    /// Check if this unit requires a Tech Lab.
    #[must_use]
    pub const fn requires_tech_lab(&self) -> bool {
        matches!(self, Self::Ranger)
    }

    /// Get the faction-specific unit ID from RON data.
    ///
    /// Maps this enum variant to the corresponding unit ID string
    /// for the given faction. Returns the RON-defined unit ID.
    #[must_use]
    pub fn to_unit_id(&self, faction: rts_core::factions::FactionId) -> &'static str {
        use rts_core::factions::FactionId;
        match (self, faction) {
            // Continuity Authority units
            (Self::Infantry, FactionId::Continuity) => "security_team",
            (Self::Harvester, FactionId::Continuity) => "collection_vehicle",
            (Self::Ranger, FactionId::Continuity) => "crowd_management_unit",

            // Collegium units
            (Self::Infantry, FactionId::Collegium) => "attack_drone_squadron",
            (Self::Harvester, FactionId::Collegium) => "constructor_bot",
            (Self::Ranger, FactionId::Collegium) => "hover_tank",

            // Tinkers' Union units
            (Self::Infantry, FactionId::Tinkers) => "field_tech",
            (Self::Harvester, FactionId::Tinkers) => "salvager",
            (Self::Ranger, FactionId::Tinkers) => "modular_mech",

            // Sculptors units
            (Self::Infantry, FactionId::BioSovereigns) => "guardian",
            (Self::Harvester, FactionId::BioSovereigns) => "harvester_organism",
            (Self::Ranger, FactionId::BioSovereigns) => "symbiote",

            // Zephyr Guild units
            (Self::Infantry, FactionId::Zephyr) => "boarding_crew",
            (Self::Harvester, FactionId::Zephyr) => "cargo_skiff",
            (Self::Ranger, FactionId::Zephyr) => "corsair_gunship",
        }
    }
}

/// Component tracking a unit's type for supply management.
#[derive(Component, Debug, Clone, Copy)]
pub struct Unit {
    /// The type of unit.
    pub unit_type: UnitType,
}

impl Unit {
    /// Create a new Unit component.
    #[must_use]
    pub const fn new(unit_type: UnitType) -> Self {
        Self { unit_type }
    }

    /// Get the supply cost for this unit.
    #[must_use]
    pub const fn supply(&self) -> i32 {
        self.unit_type.supply()
    }
}

/// Component identifying a unit's kind and role.
///
/// This is the data-driven replacement for the hardcoded `UnitType` enum.
/// The `id` maps to loaded RON data via `BevyUnitKindRegistry`.
/// Use this for new code instead of the legacy `Unit` component.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitKind {
    /// The numeric ID of this unit kind. Look up in registry for full data.
    pub id: UnitKindId,
    /// Cached role flags for fast queries (is infantry? harvester? etc.)
    pub role: UnitRole,
}

impl UnitKind {
    /// Create a new UnitKind component.
    #[must_use]
    pub const fn new(id: UnitKindId, role: UnitRole) -> Self {
        Self { id, role }
    }

    /// Check if this unit has a specific role.
    #[must_use]
    pub const fn has_role(&self, role: UnitRole) -> bool {
        self.role.contains(role)
    }

    /// Check if this is infantry.
    #[must_use]
    pub const fn is_infantry(&self) -> bool {
        self.role.contains(UnitRole::INFANTRY)
    }

    /// Check if this is a harvester.
    #[must_use]
    pub const fn is_harvester(&self) -> bool {
        self.role.contains(UnitRole::HARVESTER)
    }

    /// Check if this is an air unit.
    #[must_use]
    pub const fn is_air(&self) -> bool {
        self.role.contains(UnitRole::AIR)
    }
}

/// An item in the production queue.
#[derive(Debug, Clone)]
pub struct QueuedUnit {
    /// Type of unit being built.
    pub unit_type: UnitType,
    /// Progress toward completion (0.0 to 1.0).
    pub progress: f32,
}

/// Component for buildings that can produce units.
#[derive(Component, Debug, Clone, Default)]
pub struct GameProductionQueue {
    /// Queue of units being produced.
    pub queue: Vec<QueuedUnit>,
    /// Rally point where produced units spawn.
    pub rally_point: Option<Vec2>,
    /// Maximum queue size.
    pub max_queue_size: usize,
}

impl GameProductionQueue {
    /// Create a new production queue.
    #[must_use]
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: Vec::new(),
            rally_point: None,
            max_queue_size: max_size,
        }
    }

    /// Check if the queue can accept more units.
    #[must_use]
    pub fn can_queue(&self) -> bool {
        self.queue.len() < self.max_queue_size
    }

    /// Add a unit to the production queue.
    pub fn enqueue(&mut self, unit_type: UnitType) {
        if self.can_queue() {
            self.queue.push(QueuedUnit {
                unit_type,
                progress: 0.0,
            });
        }
    }

    /// Cancel the last item in the queue, returning partial refund.
    pub fn cancel_last(&mut self) -> Option<(UnitType, f32)> {
        self.queue.pop().map(|q| (q.unit_type, 1.0 - q.progress))
    }

    /// Get the currently building unit (front of queue).
    #[must_use]
    pub fn current(&self) -> Option<&QueuedUnit> {
        self.queue.first()
    }

    /// Get mutable reference to currently building unit.
    pub fn current_mut(&mut self) -> Option<&mut QueuedUnit> {
        self.queue.first_mut()
    }

    /// Complete the current unit and remove from queue.
    pub fn complete_current(&mut self) -> Option<UnitType> {
        if !self.queue.is_empty() {
            Some(self.queue.remove(0).unit_type)
        } else {
            None
        }
    }
}
