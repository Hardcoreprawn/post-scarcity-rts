//! Production and building system for constructing units.
//!
//! Handles unit blueprints, building blueprints, production queues,
//! and the production system that advances construction each tick.
//!
//! All calculations use integer/fixed-point math for deterministic simulation.

use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

use crate::components::{EntityId, Position};
use crate::math::{fixed_serde, option_fixed_serde, Fixed, Vec2Fixed};

/// Unique identifier for unit types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnitTypeId(pub u32);

impl UnitTypeId {
    /// Create a new unit type ID.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Unique identifier for building types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BuildingTypeId(pub u32);

impl BuildingTypeId {
    /// Create a new building type ID.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Unique identifier for technology requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TechId(pub u32);

impl TechId {
    /// Create a new tech ID.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Blueprint defining a unit type's properties.
///
/// Unit blueprints are data-driven definitions loaded from configuration.
/// They define all the stats and costs for a unit type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitBlueprint {
    /// Unique identifier for this unit type.
    pub id: UnitTypeId,
    /// Display name of the unit.
    pub name: String,
    /// Feedstock cost to produce this unit.
    pub cost: i32,
    /// Time in ticks to build this unit.
    pub build_time: u32,
    /// Maximum health points.
    pub health: i32,
    /// Movement speed (fixed-point).
    #[serde(with = "fixed_serde")]
    pub speed: Fixed,
    /// Attack damage (None if unit cannot attack).
    pub attack_damage: Option<i32>,
    /// Attack range (None if unit cannot attack).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "option_fixed_serde"
    )]
    pub attack_range: Option<Fixed>,
}

impl UnitBlueprint {
    /// Create a new unit blueprint.
    #[must_use]
    pub fn new(
        id: UnitTypeId,
        name: impl Into<String>,
        cost: i32,
        build_time: u32,
        health: i32,
        speed: Fixed,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            cost,
            build_time,
            health,
            speed,
            attack_damage: None,
            attack_range: None,
        }
    }

    /// Create a unit blueprint with combat stats.
    #[must_use]
    pub fn with_combat(mut self, damage: i32, range: Fixed) -> Self {
        self.attack_damage = Some(damage);
        self.attack_range = Some(range);
        self
    }
}

/// Blueprint defining a building type's properties.
///
/// Building blueprints define construction costs, times, and what
/// units a building can produce.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingBlueprint {
    /// Unique identifier for this building type.
    pub id: BuildingTypeId,
    /// Display name of the building.
    pub name: String,
    /// Feedstock cost to construct this building.
    pub cost: i32,
    /// Time in ticks to construct this building.
    pub build_time: u32,
    /// Maximum health points.
    pub health: i32,
    /// Unit types this building can produce.
    pub produces: Vec<UnitTypeId>,
    /// Technologies required to build this building.
    pub tech_required: Vec<TechId>,
}

impl BuildingBlueprint {
    /// Create a new building blueprint.
    #[must_use]
    pub fn new(
        id: BuildingTypeId,
        name: impl Into<String>,
        cost: i32,
        build_time: u32,
        health: i32,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            cost,
            build_time,
            health,
            produces: Vec::new(),
            tech_required: Vec::new(),
        }
    }

    /// Add unit types this building can produce.
    #[must_use]
    pub fn with_produces(mut self, units: Vec<UnitTypeId>) -> Self {
        self.produces = units;
        self
    }

    /// Add technology requirements.
    #[must_use]
    pub fn with_tech_required(mut self, techs: Vec<TechId>) -> Self {
        self.tech_required = techs;
        self
    }

    /// Check if this building can produce a given unit type.
    #[must_use]
    pub fn can_produce(&self, unit_type: UnitTypeId) -> bool {
        self.produces.contains(&unit_type)
    }
}

/// An item currently in a production queue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionItem {
    /// The type of unit being produced.
    pub unit_type: UnitTypeId,
    /// Current production progress in ticks.
    pub progress: u32,
    /// Total build time in ticks.
    pub total_time: u32,
}

impl ProductionItem {
    /// Create a new production item.
    #[must_use]
    pub const fn new(unit_type: UnitTypeId, total_time: u32) -> Self {
        Self {
            unit_type,
            progress: 0,
            total_time,
        }
    }

    /// Check if production is complete.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.progress >= self.total_time
    }

    /// Get progress as a percentage (0-100).
    #[must_use]
    pub fn percentage(&self) -> u32 {
        if self.total_time == 0 {
            100
        } else {
            (self.progress * 100) / self.total_time
        }
    }

    /// Advance production by one tick.
    pub fn tick(&mut self) {
        if self.progress < self.total_time {
            self.progress += 1;
        }
    }
}

/// Production queue component for buildings.
///
/// Manages a queue of units being produced. The first item in the queue
/// is actively being produced, advancing each tick.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ProductionQueue {
    /// Queue of items being produced.
    pub queue: VecDeque<ProductionItem>,
    /// Maximum number of items allowed in the queue.
    pub max_queue_size: usize,
}

impl ProductionQueue {
    /// Default maximum queue size.
    pub const DEFAULT_MAX_QUEUE_SIZE: usize = 5;

    /// Create a new empty production queue.
    #[must_use]
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            max_queue_size: Self::DEFAULT_MAX_QUEUE_SIZE,
        }
    }

    /// Create a production queue with a specific max size.
    #[must_use]
    pub fn with_max_size(max_queue_size: usize) -> Self {
        Self {
            queue: VecDeque::new(),
            max_queue_size,
        }
    }

    /// Check if the queue is full.
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.queue.len() >= self.max_queue_size
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

    /// Add an item to the queue.
    ///
    /// Returns `Err` if the queue is full.
    pub fn add(&mut self, unit_type: UnitTypeId, build_time: u32) -> Result<(), ProductionError> {
        if self.is_full() {
            return Err(ProductionError::QueueFull);
        }
        self.queue
            .push_back(ProductionItem::new(unit_type, build_time));
        Ok(())
    }

    /// Cancel and remove an item at the given index.
    ///
    /// Returns the cancelled item if found.
    pub fn cancel(&mut self, index: usize) -> Option<ProductionItem> {
        if index < self.queue.len() {
            self.queue.remove(index)
        } else {
            None
        }
    }

    /// Cancel the first item matching the unit type.
    ///
    /// Searches from the back of the queue (most recently added).
    /// Returns the cancelled item if found.
    pub fn cancel_unit_type(&mut self, unit_type: UnitTypeId) -> Option<ProductionItem> {
        // Find the last occurrence (most recently queued)
        if let Some(pos) = self
            .queue
            .iter()
            .rposition(|item| item.unit_type == unit_type)
        {
            self.queue.remove(pos)
        } else {
            None
        }
    }

    /// Get the currently producing item.
    #[must_use]
    pub fn current(&self) -> Option<&ProductionItem> {
        self.queue.front()
    }

    /// Get the currently producing item mutably.
    pub fn current_mut(&mut self) -> Option<&mut ProductionItem> {
        self.queue.front_mut()
    }

    /// Complete and remove the current item.
    ///
    /// Returns the completed item if production is done.
    pub fn complete(&mut self) -> Option<ProductionItem> {
        if self.queue.front().is_some_and(ProductionItem::is_complete) {
            self.queue.pop_front()
        } else {
            None
        }
    }

    /// Clear all items from the queue.
    pub fn clear(&mut self) {
        self.queue.clear();
    }
}

/// Building component with construction and rally point state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Building {
    /// The type of this building.
    pub building_type: BuildingTypeId,
    /// Whether the building is fully constructed.
    pub is_constructed: bool,
    /// Current construction progress in ticks.
    pub construction_progress: u32,
    /// Total construction time in ticks.
    pub construction_total: u32,
    /// Rally point where produced units spawn/move to.
    pub rally_point: Option<Vec2Fixed>,
}

impl Building {
    /// Create a new building under construction.
    #[must_use]
    pub fn new(building_type: BuildingTypeId, construction_total: u32) -> Self {
        Self {
            building_type,
            is_constructed: false,
            construction_progress: 0,
            construction_total,
            rally_point: None,
        }
    }

    /// Create a fully constructed building.
    #[must_use]
    pub fn constructed(building_type: BuildingTypeId) -> Self {
        Self {
            building_type,
            is_constructed: true,
            construction_progress: 0,
            construction_total: 0,
            rally_point: None,
        }
    }

    /// Set the rally point.
    pub fn set_rally_point(&mut self, point: Vec2Fixed) {
        self.rally_point = Some(point);
    }

    /// Clear the rally point.
    pub fn clear_rally_point(&mut self) {
        self.rally_point = None;
    }

    /// Check if construction is complete.
    #[must_use]
    pub const fn is_construction_complete(&self) -> bool {
        self.is_constructed || self.construction_progress >= self.construction_total
    }

    /// Get construction progress as a percentage (0-100).
    #[must_use]
    pub fn construction_percentage(&self) -> u32 {
        if self.is_constructed || self.construction_total == 0 {
            100
        } else {
            (self.construction_progress * 100) / self.construction_total
        }
    }

    /// Advance construction by one tick.
    ///
    /// Returns `true` if construction just completed.
    pub fn tick_construction(&mut self) -> bool {
        if self.is_constructed {
            return false;
        }
        if self.construction_progress < self.construction_total {
            self.construction_progress += 1;
            if self.construction_progress >= self.construction_total {
                self.is_constructed = true;
                return true;
            }
        }
        false
    }
}

/// Errors that can occur during production operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionError {
    /// The production queue is full.
    QueueFull,
    /// Cannot afford the unit cost.
    InsufficientResources,
    /// The building cannot produce this unit type.
    CannotProduceUnit,
    /// The building is not yet constructed.
    BuildingNotConstructed,
    /// The requested unit or building type was not found.
    BlueprintNotFound,
}

impl std::fmt::Display for ProductionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::QueueFull => write!(f, "Production queue is full"),
            Self::InsufficientResources => write!(f, "Insufficient resources"),
            Self::CannotProduceUnit => write!(f, "Building cannot produce this unit type"),
            Self::BuildingNotConstructed => write!(f, "Building is not yet constructed"),
            Self::BlueprintNotFound => write!(f, "Blueprint not found"),
        }
    }
}

impl std::error::Error for ProductionError {}

/// Events generated by the production system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductionEvent {
    /// Production of a unit has started.
    ProductionStarted {
        /// The building producing the unit.
        building: EntityId,
        /// The type of unit being produced.
        unit_type: UnitTypeId,
    },
    /// Production progress update.
    ProductionProgress {
        /// The building producing the unit.
        building: EntityId,
        /// The type of unit being produced.
        unit_type: UnitTypeId,
        /// Current progress in ticks.
        progress: u32,
        /// Total build time in ticks.
        total: u32,
    },
    /// A unit has finished production.
    ProductionComplete {
        /// The building that produced the unit.
        building: EntityId,
        /// The type of unit produced.
        unit_type: UnitTypeId,
        /// Position where the unit should spawn.
        spawn_position: Vec2Fixed,
    },
    /// Production was cancelled.
    ProductionCancelled {
        /// The building where production was cancelled.
        building: EntityId,
        /// The type of unit that was cancelled.
        unit_type: UnitTypeId,
        /// Amount of resources refunded.
        refund: i32,
    },
}

/// Registry containing all unit and building blueprints.
///
/// Provides lookup by ID for game data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlueprintRegistry {
    /// Unit blueprints indexed by ID.
    units: HashMap<UnitTypeId, UnitBlueprint>,
    /// Building blueprints indexed by ID.
    buildings: HashMap<BuildingTypeId, BuildingBlueprint>,
}

impl BlueprintRegistry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            units: HashMap::new(),
            buildings: HashMap::new(),
        }
    }

    /// Register a unit blueprint.
    pub fn register_unit(&mut self, blueprint: UnitBlueprint) {
        self.units.insert(blueprint.id, blueprint);
    }

    /// Register a building blueprint.
    pub fn register_building(&mut self, blueprint: BuildingBlueprint) {
        self.buildings.insert(blueprint.id, blueprint);
    }

    /// Get a unit blueprint by ID.
    #[must_use]
    pub fn get_unit(&self, id: UnitTypeId) -> Option<&UnitBlueprint> {
        self.units.get(&id)
    }

    /// Get a building blueprint by ID.
    #[must_use]
    pub fn get_building(&self, id: BuildingTypeId) -> Option<&BuildingBlueprint> {
        self.buildings.get(&id)
    }

    /// Get all registered unit blueprints.
    pub fn all_units(&self) -> impl Iterator<Item = &UnitBlueprint> {
        self.units.values()
    }

    /// Get all registered building blueprints.
    pub fn all_buildings(&self) -> impl Iterator<Item = &BuildingBlueprint> {
        self.buildings.values()
    }

    /// Check if a building can produce a unit type.
    #[must_use]
    pub fn can_building_produce(&self, building_id: BuildingTypeId, unit_id: UnitTypeId) -> bool {
        self.buildings
            .get(&building_id)
            .is_some_and(|b| b.can_produce(unit_id))
    }
}

/// Default spawn offset from building position when no rally point is set.
const DEFAULT_SPAWN_OFFSET: i32 = 2;

/// Process production for all buildings with production queues.
///
/// Advances production progress by one tick for each building and
/// generates events for completed units.
///
/// # Arguments
///
/// * `buildings` - All building entities with their production queues and positions
/// * `blueprints` - Registry of all unit/building blueprints
/// * `tick` - Current simulation tick (for event ordering/logging)
///
/// # Returns
///
/// A vector of production events that occurred this tick.
pub fn production_system(
    buildings: &mut [(EntityId, &mut ProductionQueue, &Building, &Position)],
    _blueprints: &BlueprintRegistry,
    _tick: u64,
) -> Vec<ProductionEvent> {
    let mut events = Vec::new();

    for (entity_id, queue, building, position) in buildings.iter_mut() {
        // Only process production for fully constructed buildings
        if !building.is_constructed {
            continue;
        }

        // Check if there's something in the queue
        if queue.is_empty() {
            continue;
        }

        // Get the current item and advance progress
        if let Some(item) = queue.current_mut() {
            let was_zero = item.progress == 0;
            item.tick();

            // Emit started event on first tick
            if was_zero && item.progress == 1 {
                events.push(ProductionEvent::ProductionStarted {
                    building: *entity_id,
                    unit_type: item.unit_type,
                });
            }

            // Emit progress event (could be throttled in a real game)
            events.push(ProductionEvent::ProductionProgress {
                building: *entity_id,
                unit_type: item.unit_type,
                progress: item.progress,
                total: item.total_time,
            });
        }

        // Check if production is complete
        if let Some(completed) = queue.complete() {
            // Determine spawn position
            let spawn_position = building.rally_point.unwrap_or_else(|| {
                // Default: spawn slightly offset from building
                Vec2Fixed::new(
                    position.value.x + Fixed::from_num(DEFAULT_SPAWN_OFFSET),
                    position.value.y + Fixed::from_num(DEFAULT_SPAWN_OFFSET),
                )
            });

            events.push(ProductionEvent::ProductionComplete {
                building: *entity_id,
                unit_type: completed.unit_type,
                spawn_position,
            });
        }
    }

    events
}

/// Queue a unit for production at a building.
///
/// Validates that the building can produce the unit and that resources are available.
///
/// # Arguments
///
/// * `queue` - The building's production queue
/// * `building` - The building component
/// * `unit_type` - The type of unit to produce
/// * `blueprints` - Registry of blueprints
/// * `player_feedstock` - Player's current feedstock (will be deducted on success)
///
/// # Returns
///
/// `Ok(())` on success, or a `ProductionError` on failure.
pub fn queue_production(
    queue: &mut ProductionQueue,
    building: &Building,
    unit_type: UnitTypeId,
    blueprints: &BlueprintRegistry,
    player_feedstock: &mut i32,
) -> Result<(), ProductionError> {
    // Check building is constructed
    if !building.is_constructed {
        return Err(ProductionError::BuildingNotConstructed);
    }

    // Get building blueprint to check if it can produce this unit
    let building_blueprint = blueprints
        .get_building(building.building_type)
        .ok_or(ProductionError::BlueprintNotFound)?;

    if !building_blueprint.can_produce(unit_type) {
        return Err(ProductionError::CannotProduceUnit);
    }

    // Get unit blueprint for cost and build time
    let unit_blueprint = blueprints
        .get_unit(unit_type)
        .ok_or(ProductionError::BlueprintNotFound)?;

    // Check resources
    if *player_feedstock < unit_blueprint.cost {
        return Err(ProductionError::InsufficientResources);
    }

    // Try to add to queue
    queue.add(unit_type, unit_blueprint.build_time)?;

    // Deduct cost
    *player_feedstock -= unit_blueprint.cost;

    Ok(())
}

/// Cancel production of a unit at a specific queue index.
///
/// Refunds a portion of the cost based on progress.
///
/// # Arguments
///
/// * `queue` - The building's production queue
/// * `index` - Index of the item to cancel
/// * `blueprints` - Registry of blueprints
/// * `player_feedstock` - Player's feedstock (refund will be added)
/// * `refund_percentage` - Percentage of cost to refund (0-100)
///
/// # Returns
///
/// The cancelled `ProductionItem` and refund amount if successful.
pub fn cancel_production(
    queue: &mut ProductionQueue,
    index: usize,
    blueprints: &BlueprintRegistry,
    player_feedstock: &mut i32,
    refund_percentage: i32,
) -> Option<(ProductionItem, i32)> {
    let item = queue.cancel(index)?;

    // Calculate refund
    let unit_blueprint = blueprints.get_unit(item.unit_type)?;
    let base_refund = (unit_blueprint.cost * refund_percentage) / 100;

    // If production has started, reduce refund based on progress
    let progress_factor = if item.total_time > 0 {
        100 - ((item.progress * 100) / item.total_time) as i32
    } else {
        100
    };
    let refund = (base_refund * progress_factor) / 100;

    *player_feedstock += refund;

    Some((item, refund))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_blueprints() -> BlueprintRegistry {
        let mut registry = BlueprintRegistry::new();

        // Register a basic unit
        registry.register_unit(UnitBlueprint::new(
            UnitTypeId(1),
            "Infantry",
            100,
            60,
            50,
            Fixed::from_num(1),
        ));

        // Register a combat unit
        registry.register_unit(
            UnitBlueprint::new(UnitTypeId(2), "Tank", 300, 120, 200, Fixed::from_num(2))
                .with_combat(50, Fixed::from_num(5)),
        );

        // Register a barracks
        registry.register_building(
            BuildingBlueprint::new(BuildingTypeId(1), "Barracks", 200, 90, 500)
                .with_produces(vec![UnitTypeId(1)]),
        );

        // Register a factory
        registry.register_building(
            BuildingBlueprint::new(BuildingTypeId(2), "Factory", 400, 120, 800)
                .with_produces(vec![UnitTypeId(1), UnitTypeId(2)]),
        );

        registry
    }

    #[test]
    fn test_unit_type_id() {
        let id = UnitTypeId::new(42);
        assert_eq!(id.0, 42);
    }

    #[test]
    fn test_building_type_id() {
        let id = BuildingTypeId::new(7);
        assert_eq!(id.0, 7);
    }

    #[test]
    fn test_unit_blueprint_creation() {
        let blueprint =
            UnitBlueprint::new(UnitTypeId(1), "Infantry", 100, 60, 50, Fixed::from_num(1));

        assert_eq!(blueprint.id, UnitTypeId(1));
        assert_eq!(blueprint.name, "Infantry");
        assert_eq!(blueprint.cost, 100);
        assert_eq!(blueprint.build_time, 60);
        assert_eq!(blueprint.health, 50);
        assert_eq!(blueprint.speed, Fixed::from_num(1));
        assert!(blueprint.attack_damage.is_none());
        assert!(blueprint.attack_range.is_none());
    }

    #[test]
    fn test_unit_blueprint_with_combat() {
        let blueprint =
            UnitBlueprint::new(UnitTypeId(1), "Tank", 300, 120, 200, Fixed::from_num(2))
                .with_combat(50, Fixed::from_num(5));

        assert_eq!(blueprint.attack_damage, Some(50));
        assert_eq!(blueprint.attack_range, Some(Fixed::from_num(5)));
    }

    #[test]
    fn test_building_blueprint_creation() {
        let blueprint = BuildingBlueprint::new(BuildingTypeId(1), "Barracks", 200, 90, 500)
            .with_produces(vec![UnitTypeId(1), UnitTypeId(2)])
            .with_tech_required(vec![TechId(1)]);

        assert_eq!(blueprint.id, BuildingTypeId(1));
        assert_eq!(blueprint.name, "Barracks");
        assert_eq!(blueprint.cost, 200);
        assert_eq!(blueprint.build_time, 90);
        assert_eq!(blueprint.health, 500);
        assert_eq!(blueprint.produces.len(), 2);
        assert_eq!(blueprint.tech_required.len(), 1);
        assert!(blueprint.can_produce(UnitTypeId(1)));
        assert!(blueprint.can_produce(UnitTypeId(2)));
        assert!(!blueprint.can_produce(UnitTypeId(3)));
    }

    #[test]
    fn test_production_item() {
        let mut item = ProductionItem::new(UnitTypeId(1), 100);

        assert_eq!(item.progress, 0);
        assert_eq!(item.total_time, 100);
        assert!(!item.is_complete());
        assert_eq!(item.percentage(), 0);

        // Tick 50 times
        for _ in 0..50 {
            item.tick();
        }
        assert_eq!(item.progress, 50);
        assert_eq!(item.percentage(), 50);
        assert!(!item.is_complete());

        // Tick remaining 50
        for _ in 0..50 {
            item.tick();
        }
        assert_eq!(item.progress, 100);
        assert_eq!(item.percentage(), 100);
        assert!(item.is_complete());

        // Extra ticks don't go past total
        item.tick();
        assert_eq!(item.progress, 100);
    }

    #[test]
    fn test_production_queue_basic() {
        let mut queue = ProductionQueue::new();

        assert!(queue.is_empty());
        assert!(!queue.is_full());
        assert_eq!(queue.len(), 0);

        // Add items
        queue.add(UnitTypeId(1), 60).unwrap();
        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());

        queue.add(UnitTypeId(2), 120).unwrap();
        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn test_production_queue_full() {
        let mut queue = ProductionQueue::with_max_size(2);

        queue.add(UnitTypeId(1), 60).unwrap();
        queue.add(UnitTypeId(2), 60).unwrap();
        assert!(queue.is_full());

        let result = queue.add(UnitTypeId(3), 60);
        assert!(matches!(result, Err(ProductionError::QueueFull)));
    }

    #[test]
    fn test_production_queue_cancel() {
        let mut queue = ProductionQueue::new();

        queue.add(UnitTypeId(1), 60).unwrap();
        queue.add(UnitTypeId(2), 120).unwrap();
        queue.add(UnitTypeId(3), 90).unwrap();

        // Cancel middle item
        let cancelled = queue.cancel(1);
        assert!(cancelled.is_some());
        assert_eq!(cancelled.unwrap().unit_type, UnitTypeId(2));
        assert_eq!(queue.len(), 2);

        // First and third remain
        assert_eq!(queue.current().unwrap().unit_type, UnitTypeId(1));
    }

    #[test]
    fn test_production_queue_cancel_by_type() {
        let mut queue = ProductionQueue::new();

        queue.add(UnitTypeId(1), 60).unwrap();
        queue.add(UnitTypeId(1), 60).unwrap(); // Duplicate
        queue.add(UnitTypeId(2), 120).unwrap();

        // Cancel last of type 1 (most recently added)
        let cancelled = queue.cancel_unit_type(UnitTypeId(1));
        assert!(cancelled.is_some());
        assert_eq!(queue.len(), 2);

        // First item is still type 1
        assert_eq!(queue.current().unwrap().unit_type, UnitTypeId(1));
    }

    #[test]
    fn test_building_construction() {
        let mut building = Building::new(BuildingTypeId(1), 90);

        assert!(!building.is_constructed);
        assert!(!building.is_construction_complete());
        assert_eq!(building.construction_percentage(), 0);

        // Tick 45 times (half done)
        for _ in 0..45 {
            assert!(!building.tick_construction());
        }
        assert_eq!(building.construction_percentage(), 50);
        assert!(!building.is_constructed);

        // Tick remaining 45
        for i in 0..45 {
            let completed = building.tick_construction();
            if i == 44 {
                assert!(completed);
            } else {
                assert!(!completed);
            }
        }
        assert!(building.is_constructed);
        assert_eq!(building.construction_percentage(), 100);
    }

    #[test]
    fn test_building_rally_point() {
        let mut building = Building::constructed(BuildingTypeId(1));

        assert!(building.rally_point.is_none());

        let rally = Vec2Fixed::new(Fixed::from_num(10), Fixed::from_num(20));
        building.set_rally_point(rally);
        assert_eq!(building.rally_point, Some(rally));

        building.clear_rally_point();
        assert!(building.rally_point.is_none());
    }

    #[test]
    fn test_blueprint_registry() {
        let registry = create_test_blueprints();

        // Unit lookups
        let infantry = registry.get_unit(UnitTypeId(1));
        assert!(infantry.is_some());
        assert_eq!(infantry.unwrap().name, "Infantry");

        let tank = registry.get_unit(UnitTypeId(2));
        assert!(tank.is_some());
        assert_eq!(tank.unwrap().name, "Tank");

        assert!(registry.get_unit(UnitTypeId(999)).is_none());

        // Building lookups
        let barracks = registry.get_building(BuildingTypeId(1));
        assert!(barracks.is_some());
        assert_eq!(barracks.unwrap().name, "Barracks");

        // Can produce checks
        assert!(registry.can_building_produce(BuildingTypeId(1), UnitTypeId(1)));
        assert!(!registry.can_building_produce(BuildingTypeId(1), UnitTypeId(2)));
        assert!(registry.can_building_produce(BuildingTypeId(2), UnitTypeId(1)));
        assert!(registry.can_building_produce(BuildingTypeId(2), UnitTypeId(2)));
    }

    #[test]
    fn test_production_system_basic() {
        let blueprints = create_test_blueprints();

        let mut queue = ProductionQueue::new();
        queue.add(UnitTypeId(1), 3).unwrap(); // 3 tick build time

        let building = Building::constructed(BuildingTypeId(1));
        let position = Position::new(Vec2Fixed::new(Fixed::from_num(0), Fixed::from_num(0)));

        let mut buildings = vec![(1u64, &mut queue, &building, &position)];

        // Tick 1: production starts
        let events = production_system(&mut buildings, &blueprints, 1);
        assert!(events.iter().any(|e| matches!(e, ProductionEvent::ProductionStarted { building: 1, unit_type } if *unit_type == UnitTypeId(1))));
        assert!(events.iter().any(|e| matches!(
            e,
            ProductionEvent::ProductionProgress {
                progress: 1,
                total: 3,
                ..
            }
        )));

        // Tick 2: in progress
        let events = production_system(&mut buildings, &blueprints, 2);
        assert!(events.iter().any(|e| matches!(
            e,
            ProductionEvent::ProductionProgress {
                progress: 2,
                total: 3,
                ..
            }
        )));

        // Tick 3: complete
        let events = production_system(&mut buildings, &blueprints, 3);
        assert!(events.iter().any(|e| matches!(e, ProductionEvent::ProductionComplete { building: 1, unit_type, .. } if *unit_type == UnitTypeId(1))));
    }

    #[test]
    fn test_production_system_rally_point() {
        let blueprints = create_test_blueprints();

        let mut queue = ProductionQueue::new();
        queue.add(UnitTypeId(1), 1).unwrap(); // 1 tick build time

        let mut building = Building::constructed(BuildingTypeId(1));
        let rally = Vec2Fixed::new(Fixed::from_num(50), Fixed::from_num(50));
        building.set_rally_point(rally);

        let position = Position::new(Vec2Fixed::new(Fixed::from_num(0), Fixed::from_num(0)));

        let mut buildings = vec![(1u64, &mut queue, &building, &position)];

        let events = production_system(&mut buildings, &blueprints, 1);

        // Spawn position should be rally point
        let complete_event = events
            .iter()
            .find(|e| matches!(e, ProductionEvent::ProductionComplete { .. }));
        assert!(complete_event.is_some());
        if let ProductionEvent::ProductionComplete { spawn_position, .. } = complete_event.unwrap()
        {
            assert_eq!(*spawn_position, rally);
        }
    }

    #[test]
    fn test_production_system_unconstructed_building() {
        let blueprints = create_test_blueprints();

        let mut queue = ProductionQueue::new();
        queue.add(UnitTypeId(1), 1).unwrap();

        // Building not yet constructed
        let building = Building::new(BuildingTypeId(1), 90);
        let position = Position::new(Vec2Fixed::ZERO);

        let mut buildings = vec![(1u64, &mut queue, &building, &position)];

        let events = production_system(&mut buildings, &blueprints, 1);

        // No production events for unconstructed building
        assert!(events.is_empty());
        // Queue item progress unchanged
        assert_eq!(queue.current().unwrap().progress, 0);
    }

    #[test]
    fn test_queue_production() {
        let blueprints = create_test_blueprints();

        let mut queue = ProductionQueue::new();
        let building = Building::constructed(BuildingTypeId(2)); // Factory can build both
        let mut feedstock = 500;

        // Queue infantry (costs 100)
        let result = queue_production(
            &mut queue,
            &building,
            UnitTypeId(1),
            &blueprints,
            &mut feedstock,
        );
        assert!(result.is_ok());
        assert_eq!(feedstock, 400);
        assert_eq!(queue.len(), 1);

        // Queue tank (costs 300)
        let result = queue_production(
            &mut queue,
            &building,
            UnitTypeId(2),
            &blueprints,
            &mut feedstock,
        );
        assert!(result.is_ok());
        assert_eq!(feedstock, 100);
        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn test_queue_production_insufficient_resources() {
        let blueprints = create_test_blueprints();

        let mut queue = ProductionQueue::new();
        let building = Building::constructed(BuildingTypeId(2));
        let mut feedstock = 50; // Not enough for infantry (100)

        let result = queue_production(
            &mut queue,
            &building,
            UnitTypeId(1),
            &blueprints,
            &mut feedstock,
        );
        assert!(matches!(
            result,
            Err(ProductionError::InsufficientResources)
        ));
        assert_eq!(feedstock, 50); // Unchanged
        assert!(queue.is_empty());
    }

    #[test]
    fn test_queue_production_cannot_produce() {
        let blueprints = create_test_blueprints();

        let mut queue = ProductionQueue::new();
        let building = Building::constructed(BuildingTypeId(1)); // Barracks can't build tanks
        let mut feedstock = 500;

        let result = queue_production(
            &mut queue,
            &building,
            UnitTypeId(2),
            &blueprints,
            &mut feedstock,
        );
        assert!(matches!(result, Err(ProductionError::CannotProduceUnit)));
        assert_eq!(feedstock, 500); // Unchanged
    }

    #[test]
    fn test_queue_production_not_constructed() {
        let blueprints = create_test_blueprints();

        let mut queue = ProductionQueue::new();
        let building = Building::new(BuildingTypeId(1), 90); // Not constructed
        let mut feedstock = 500;

        let result = queue_production(
            &mut queue,
            &building,
            UnitTypeId(1),
            &blueprints,
            &mut feedstock,
        );
        assert!(matches!(
            result,
            Err(ProductionError::BuildingNotConstructed)
        ));
    }

    #[test]
    fn test_cancel_production_full_refund() {
        let blueprints = create_test_blueprints();

        let mut queue = ProductionQueue::new();
        queue.add(UnitTypeId(1), 60).unwrap(); // Infantry, no progress yet
        let mut feedstock = 0;

        let result = cancel_production(&mut queue, 0, &blueprints, &mut feedstock, 100);
        assert!(result.is_some());

        let (item, refund) = result.unwrap();
        assert_eq!(item.unit_type, UnitTypeId(1));
        assert_eq!(refund, 100); // Full refund (100% of 100 cost)
        assert_eq!(feedstock, 100);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_cancel_production_partial_progress_refund() {
        let blueprints = create_test_blueprints();

        let mut queue = ProductionQueue::new();
        queue.add(UnitTypeId(1), 100).unwrap();

        // Simulate 50% progress
        queue.current_mut().unwrap().progress = 50;

        let mut feedstock = 0;

        // 100% refund rate, but 50% complete = 50% actual refund
        let result = cancel_production(&mut queue, 0, &blueprints, &mut feedstock, 100);
        assert!(result.is_some());

        let (_, refund) = result.unwrap();
        assert_eq!(refund, 50); // 50% of 100 cost
        assert_eq!(feedstock, 50);
    }

    #[test]
    fn test_cancel_production_reduced_refund_rate() {
        let blueprints = create_test_blueprints();

        let mut queue = ProductionQueue::new();
        queue.add(UnitTypeId(1), 100).unwrap(); // No progress
        let mut feedstock = 0;

        // 50% refund rate
        let result = cancel_production(&mut queue, 0, &blueprints, &mut feedstock, 50);
        assert!(result.is_some());

        let (_, refund) = result.unwrap();
        assert_eq!(refund, 50); // 50% of 100 cost
        assert_eq!(feedstock, 50);
    }

    #[test]
    fn test_multiple_production_queues() {
        let blueprints = create_test_blueprints();

        let mut queue1 = ProductionQueue::new();
        queue1.add(UnitTypeId(1), 2).unwrap();

        let mut queue2 = ProductionQueue::new();
        queue2.add(UnitTypeId(2), 2).unwrap();

        let building1 = Building::constructed(BuildingTypeId(1));
        let building2 = Building::constructed(BuildingTypeId(2));

        let pos1 = Position::new(Vec2Fixed::new(Fixed::from_num(0), Fixed::from_num(0)));
        let pos2 = Position::new(Vec2Fixed::new(Fixed::from_num(10), Fixed::from_num(10)));

        let mut buildings = vec![
            (1u64, &mut queue1, &building1, &pos1),
            (2u64, &mut queue2, &building2, &pos2),
        ];

        // Both should produce on first tick
        let events = production_system(&mut buildings, &blueprints, 1);

        // Should have started events for both
        assert!(events
            .iter()
            .any(|e| matches!(e, ProductionEvent::ProductionStarted { building: 1, .. })));
        assert!(events
            .iter()
            .any(|e| matches!(e, ProductionEvent::ProductionStarted { building: 2, .. })));
    }

    #[test]
    fn test_production_error_display() {
        assert_eq!(
            ProductionError::QueueFull.to_string(),
            "Production queue is full"
        );
        assert_eq!(
            ProductionError::InsufficientResources.to_string(),
            "Insufficient resources"
        );
        assert_eq!(
            ProductionError::CannotProduceUnit.to_string(),
            "Building cannot produce this unit type"
        );
        assert_eq!(
            ProductionError::BuildingNotConstructed.to_string(),
            "Building is not yet constructed"
        );
        assert_eq!(
            ProductionError::BlueprintNotFound.to_string(),
            "Blueprint not found"
        );
    }
}
