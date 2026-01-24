//! Economy and resource management system.
//!
//! Implements the Feedstock-based economy where harvesters gather
//! resources from nodes and deposit them at depots.
//!
//! All calculations use integer math for deterministic simulation.

use serde::{Deserialize, Serialize};

use crate::components::EntityId;
use crate::math::Vec2Fixed;

/// Feedstock - the primary resource gathered by harvesters.
///
/// Raw material that matter replicators use to construct units and buildings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Feedstock(pub i32);

impl Feedstock {
    /// Create a new feedstock amount.
    #[must_use]
    pub const fn new(amount: i32) -> Self {
        Self(amount)
    }

    /// Zero feedstock.
    pub const ZERO: Self = Self(0);
}

impl std::ops::Add for Feedstock {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub for Feedstock {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl std::ops::AddAssign for Feedstock {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl std::ops::SubAssign for Feedstock {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

/// A resource node that harvesters can gather from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceNode {
    /// Position in world space.
    pub position: Vec2Fixed,
    /// Remaining feedstock in this node.
    pub remaining: i32,
    /// Amount gathered per harvest action.
    pub gather_rate: i32,
}

impl ResourceNode {
    /// Create a new resource node.
    #[must_use]
    pub const fn new(position: Vec2Fixed, remaining: i32, gather_rate: i32) -> Self {
        Self {
            position,
            remaining,
            gather_rate,
        }
    }

    /// Check if this node is depleted.
    #[must_use]
    pub const fn is_depleted(&self) -> bool {
        self.remaining <= 0
    }

    /// Extract resources from this node.
    ///
    /// Returns the actual amount extracted (may be less than requested if node is nearly depleted).
    pub fn extract(&mut self, requested: i32) -> i32 {
        let extracted = requested.min(self.remaining);
        self.remaining -= extracted;
        extracted
    }
}

/// Player economy state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PlayerEconomy {
    /// Current feedstock stockpile.
    pub feedstock: i32,
    /// Maximum feedstock storage capacity.
    pub storage_capacity: i32,
    /// Calculated income rate from active harvesters (per tick estimation).
    pub income_rate: i32,
}

impl PlayerEconomy {
    /// Create a new player economy with initial values.
    #[must_use]
    pub const fn new(feedstock: i32, storage_capacity: i32) -> Self {
        Self {
            feedstock,
            storage_capacity,
            income_rate: 0,
        }
    }

    /// Check available storage space.
    #[must_use]
    pub const fn available_storage(&self) -> i32 {
        self.storage_capacity - self.feedstock
    }

    /// Deposit feedstock, respecting storage limits.
    ///
    /// Returns the actual amount deposited.
    pub fn deposit(&mut self, amount: i32) -> i32 {
        let space = self.available_storage();
        let deposited = amount.min(space);
        self.feedstock += deposited;
        deposited
    }

    /// Spend feedstock if available.
    ///
    /// Returns true if the transaction succeeded.
    pub fn spend(&mut self, amount: i32) -> bool {
        if self.feedstock >= amount {
            self.feedstock -= amount;
            true
        } else {
            false
        }
    }

    /// Check if player can afford a cost.
    #[must_use]
    pub const fn can_afford(&self, cost: i32) -> bool {
        self.feedstock >= cost
    }
}

/// State of a harvester unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HarvesterState {
    /// Idle, waiting for orders or auto-assignment.
    Idle,
    /// Moving to a resource node.
    MovingToNode(EntityId),
    /// Actively gathering from a node.
    Gathering(EntityId),
    /// Returning to a depot to deposit resources.
    Returning(EntityId),
    /// Depositing resources at a depot.
    Depositing,
}

impl Default for HarvesterState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Harvester component for units that gather resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Harvester {
    /// Maximum feedstock this harvester can carry.
    pub capacity: i32,
    /// Current load of feedstock being carried.
    pub current_load: i32,
    /// Amount gathered per harvest action.
    pub gather_rate: i32,
    /// Current state of the harvester.
    pub state: HarvesterState,
}

impl Harvester {
    /// Create a new harvester with the given capacity and gather rate.
    #[must_use]
    pub const fn new(capacity: i32, gather_rate: i32) -> Self {
        Self {
            capacity,
            current_load: 0,
            gather_rate,
            state: HarvesterState::Idle,
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
    ///
    /// Returns actual amount loaded (may be less if capacity is reached).
    pub fn load(&mut self, amount: i32) -> i32 {
        let space = self.available_capacity();
        let loaded = amount.min(space);
        self.current_load += loaded;
        loaded
    }

    /// Unload all resources from the harvester.
    ///
    /// Returns the amount unloaded.
    pub fn unload(&mut self) -> i32 {
        let amount = self.current_load;
        self.current_load = 0;
        amount
    }
}

/// Marker component for depot buildings that accept resource deposits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Depot;

/// Events generated by the economy system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EconomyEvent {
    /// A harvester gathered resources from a node.
    ResourceGathered {
        /// The harvester entity.
        harvester: EntityId,
        /// The resource node entity.
        node: EntityId,
        /// Amount gathered.
        amount: i32,
    },
    /// A harvester deposited resources at a depot.
    ResourceDeposited {
        /// The harvester entity.
        harvester: EntityId,
        /// The depot entity.
        depot: EntityId,
        /// Amount deposited.
        amount: i32,
    },
    /// A resource node has been fully depleted.
    NodeDepleted {
        /// The depleted node entity.
        node: EntityId,
    },
}

/// Distance threshold for interaction (squared, to avoid sqrt).
/// Units within this distance can gather/deposit.
const INTERACTION_DISTANCE_SQ: i64 = 4; // 2 units squared

/// Process the economy system for one tick.
///
/// Handles harvester state machines, resource gathering, and deposits.
/// Returns events for anything significant that happened.
///
/// # Arguments
///
/// * `harvesters` - All harvester entities with their components and positions
/// * `nodes` - All resource node entities with their components and positions
/// * `depots` - All depot entities with their positions
/// * `player_economy` - The player's economy state to update
///
/// # Returns
///
/// A vector of economy events that occurred this tick.
pub fn economy_system(
    harvesters: &mut [(EntityId, &mut Harvester, &Vec2Fixed)],
    nodes: &mut [(EntityId, &mut ResourceNode, &Vec2Fixed)],
    depots: &[(EntityId, &Vec2Fixed)],
    player_economy: &mut PlayerEconomy,
) -> Vec<EconomyEvent> {
    let mut events = Vec::new();

    for (harvester_id, harvester, harvester_pos) in harvesters.iter_mut() {
        match harvester.state {
            HarvesterState::Idle => {
                // Auto-assign to nearest non-depleted node if empty
                if harvester.is_empty() {
                    if let Some(node_id) = find_nearest_node(**harvester_pos, nodes) {
                        harvester.state = HarvesterState::MovingToNode(node_id);
                    }
                } else {
                    // Has resources, find depot
                    if let Some(depot_id) = find_nearest_depot(**harvester_pos, depots) {
                        harvester.state = HarvesterState::Returning(depot_id);
                    }
                }
            }

            HarvesterState::MovingToNode(node_id) => {
                // Check if we've arrived at the node
                if let Some((_, node, node_pos)) = nodes.iter().find(|(id, _, _)| *id == node_id) {
                    if is_within_range(**harvester_pos, **node_pos) {
                        if node.is_depleted() {
                            // Node depleted, find another
                            harvester.state = HarvesterState::Idle;
                        } else {
                            harvester.state = HarvesterState::Gathering(node_id);
                        }
                    }
                    // Otherwise, keep moving (movement handled by movement system)
                } else {
                    // Node no longer exists
                    harvester.state = HarvesterState::Idle;
                }
            }

            HarvesterState::Gathering(node_id) => {
                // Try to gather from the node
                if let Some((_, node, _)) =
                    nodes.iter_mut().find(|(id, _, _)| *id == node_id)
                {
                    if node.is_depleted() {
                        events.push(EconomyEvent::NodeDepleted { node: node_id });
                        harvester.state = HarvesterState::Idle;
                    } else if harvester.is_full() {
                        // Full, need to return to depot
                        if let Some(depot_id) = find_nearest_depot(**harvester_pos, depots) {
                            harvester.state = HarvesterState::Returning(depot_id);
                        } else {
                            harvester.state = HarvesterState::Idle;
                        }
                    } else {
                        // Gather resources
                        let to_gather = harvester.gather_rate.min(harvester.available_capacity());
                        let gathered = node.extract(to_gather);
                        harvester.load(gathered);

                        if gathered > 0 {
                            events.push(EconomyEvent::ResourceGathered {
                                harvester: *harvester_id,
                                node: node_id,
                                amount: gathered,
                            });
                        }

                        // Check if node depleted after gathering
                        if node.is_depleted() {
                            events.push(EconomyEvent::NodeDepleted { node: node_id });
                        }

                        // Check if full after gathering
                        if harvester.is_full() {
                            if let Some(depot_id) = find_nearest_depot(**harvester_pos, depots) {
                                harvester.state = HarvesterState::Returning(depot_id);
                            } else {
                                harvester.state = HarvesterState::Idle;
                            }
                        }
                    }
                } else {
                    // Node no longer exists
                    harvester.state = HarvesterState::Idle;
                }
            }

            HarvesterState::Returning(depot_id) => {
                // Check if we've arrived at the depot
                if let Some((_, depot_pos)) = depots.iter().find(|(id, _)| *id == depot_id) {
                    if is_within_range(**harvester_pos, **depot_pos) {
                        harvester.state = HarvesterState::Depositing;
                    }
                    // Otherwise, keep moving
                } else {
                    // Depot no longer exists, find another
                    if let Some(new_depot_id) = find_nearest_depot(**harvester_pos, depots) {
                        harvester.state = HarvesterState::Returning(new_depot_id);
                    } else {
                        harvester.state = HarvesterState::Idle;
                    }
                }
            }

            HarvesterState::Depositing => {
                // Find the depot we're at
                if let Some((depot_id, _)) = depots
                    .iter()
                    .find(|(_, pos)| is_within_range(**harvester_pos, **pos))
                {
                    let load = harvester.unload();
                    let deposited = player_economy.deposit(load);

                    if deposited > 0 {
                        events.push(EconomyEvent::ResourceDeposited {
                            harvester: *harvester_id,
                            depot: *depot_id,
                            amount: deposited,
                        });
                    }

                    // If we couldn't deposit everything (full storage), the rest is lost
                    // This encourages building more storage

                    // Go back to gathering
                    if let Some(node_id) = find_nearest_node(**harvester_pos, nodes) {
                        harvester.state = HarvesterState::MovingToNode(node_id);
                    } else {
                        harvester.state = HarvesterState::Idle;
                    }
                } else {
                    // Not at a depot anymore, find one
                    if let Some(depot_id) = find_nearest_depot(**harvester_pos, depots) {
                        harvester.state = HarvesterState::Returning(depot_id);
                    } else {
                        harvester.state = HarvesterState::Idle;
                    }
                }
            }
        }
    }

    // Update income rate estimation (based on active harvesters)
    let active_harvesters: i32 = harvesters
        .iter()
        .filter(|(_, h, _)| matches!(h.state, HarvesterState::Gathering(_)))
        .map(|(_, h, _)| h.gather_rate)
        .sum();
    player_economy.income_rate = active_harvesters;

    events
}

/// Find the nearest non-depleted resource node.
fn find_nearest_node(
    pos: Vec2Fixed,
    nodes: &[(EntityId, &mut ResourceNode, &Vec2Fixed)],
) -> Option<EntityId> {
    nodes
        .iter()
        .filter(|(_, node, _)| !node.is_depleted())
        .min_by_key(|(_, _, node_pos)| {
            let dist = pos.distance_squared(**node_pos);
            dist.to_bits()
        })
        .map(|(id, _, _)| *id)
}

/// Find the nearest depot.
fn find_nearest_depot(pos: Vec2Fixed, depots: &[(EntityId, &Vec2Fixed)]) -> Option<EntityId> {
    depots
        .iter()
        .min_by_key(|(_, depot_pos)| {
            let dist = pos.distance_squared(**depot_pos);
            dist.to_bits()
        })
        .map(|(id, _)| *id)
}

/// Check if two positions are within interaction range.
fn is_within_range(a: Vec2Fixed, b: Vec2Fixed) -> bool {
    let dist_sq = a.distance_squared(b);
    dist_sq.to_bits() <= INTERACTION_DISTANCE_SQ * (1i64 << 32) // Account for Fixed scaling
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Fixed;

    fn pos(x: i32, y: i32) -> Vec2Fixed {
        Vec2Fixed::new(Fixed::from_num(x), Fixed::from_num(y))
    }

    #[test]
    fn test_feedstock_arithmetic() {
        let a = Feedstock(100);
        let b = Feedstock(30);

        assert_eq!(a + b, Feedstock(130));
        assert_eq!(a - b, Feedstock(70));

        let mut c = Feedstock(50);
        c += Feedstock(10);
        assert_eq!(c, Feedstock(60));

        c -= Feedstock(20);
        assert_eq!(c, Feedstock(40));
    }

    #[test]
    fn test_resource_node_extraction() {
        let mut node = ResourceNode::new(pos(0, 0), 100, 10);

        assert!(!node.is_depleted());

        let extracted = node.extract(10);
        assert_eq!(extracted, 10);
        assert_eq!(node.remaining, 90);

        // Extract more than remaining
        let extracted = node.extract(100);
        assert_eq!(extracted, 90);
        assert_eq!(node.remaining, 0);
        assert!(node.is_depleted());
    }

    #[test]
    fn test_player_economy_deposit() {
        let mut economy = PlayerEconomy::new(0, 100);

        let deposited = economy.deposit(50);
        assert_eq!(deposited, 50);
        assert_eq!(economy.feedstock, 50);

        // Deposit more than capacity allows
        let deposited = economy.deposit(60);
        assert_eq!(deposited, 50); // Only 50 space left
        assert_eq!(economy.feedstock, 100);
    }

    #[test]
    fn test_player_economy_spend() {
        let mut economy = PlayerEconomy::new(100, 200);

        assert!(economy.can_afford(50));
        assert!(economy.spend(50));
        assert_eq!(economy.feedstock, 50);

        assert!(!economy.can_afford(100));
        assert!(!economy.spend(100));
        assert_eq!(economy.feedstock, 50); // Unchanged
    }

    #[test]
    fn test_harvester_load_unload() {
        let mut harvester = Harvester::new(100, 10);

        assert!(harvester.is_empty());
        assert!(!harvester.is_full());

        let loaded = harvester.load(50);
        assert_eq!(loaded, 50);
        assert_eq!(harvester.current_load, 50);

        // Try to overload
        let loaded = harvester.load(60);
        assert_eq!(loaded, 50); // Only 50 space left
        assert!(harvester.is_full());

        let unloaded = harvester.unload();
        assert_eq!(unloaded, 100);
        assert!(harvester.is_empty());
    }

    #[test]
    fn test_economy_system_gathering() {
        let mut harvester = Harvester::new(100, 10);
        harvester.state = HarvesterState::Gathering(1);

        let mut node = ResourceNode::new(pos(0, 0), 50, 10);
        let mut economy = PlayerEconomy::new(0, 1000);

        let harvester_pos = pos(0, 0);
        let node_pos = pos(0, 0);

        let mut harvesters = vec![(0u64, &mut harvester, &harvester_pos)];
        let mut nodes = vec![(1u64, &mut node, &node_pos)];
        let depots: Vec<(EntityId, &Vec2Fixed)> = vec![];

        let events = economy_system(&mut harvesters, &mut nodes, &depots, &mut economy);

        // Should have gathered resources
        assert!(events.iter().any(|e| matches!(
            e,
            EconomyEvent::ResourceGathered {
                harvester: 0,
                node: 1,
                amount: 10
            }
        )));

        assert_eq!(harvesters[0].1.current_load, 10);
        assert_eq!(nodes[0].1.remaining, 40);
    }

    #[test]
    fn test_economy_system_depositing() {
        let mut harvester = Harvester::new(100, 10);
        harvester.current_load = 50;
        harvester.state = HarvesterState::Depositing;

        let mut economy = PlayerEconomy::new(0, 1000);

        let harvester_pos = pos(0, 0);
        let depot_pos = pos(0, 0);

        let mut harvesters = vec![(0u64, &mut harvester, &harvester_pos)];
        let mut nodes: Vec<(EntityId, &mut ResourceNode, &Vec2Fixed)> = vec![];
        let depots = vec![(1u64, &depot_pos)];

        let events = economy_system(&mut harvesters, &mut nodes, &depots, &mut economy);

        // Should have deposited resources
        assert!(events.iter().any(|e| matches!(
            e,
            EconomyEvent::ResourceDeposited {
                harvester: 0,
                depot: 1,
                amount: 50
            }
        )));

        assert_eq!(harvesters[0].1.current_load, 0);
        assert_eq!(economy.feedstock, 50);
    }

    #[test]
    fn test_economy_system_node_depleted() {
        let mut harvester = Harvester::new(100, 10);
        harvester.state = HarvesterState::Gathering(1);

        let mut node = ResourceNode::new(pos(0, 0), 5, 10); // Only 5 left
        let mut economy = PlayerEconomy::new(0, 1000);

        let harvester_pos = pos(0, 0);
        let node_pos = pos(0, 0);

        let mut harvesters = vec![(0u64, &mut harvester, &harvester_pos)];
        let mut nodes = vec![(1u64, &mut node, &node_pos)];
        let depots: Vec<(EntityId, &Vec2Fixed)> = vec![];

        let events = economy_system(&mut harvesters, &mut nodes, &depots, &mut economy);

        // Should have node depleted event
        assert!(events
            .iter()
            .any(|e| matches!(e, EconomyEvent::NodeDepleted { node: 1 })));

        assert!(nodes[0].1.is_depleted());
    }

    #[test]
    fn test_economy_system_storage_limit() {
        let mut harvester = Harvester::new(100, 10);
        harvester.current_load = 50;
        harvester.state = HarvesterState::Depositing;

        // Only 20 storage space left
        let mut economy = PlayerEconomy::new(80, 100);

        let harvester_pos = pos(0, 0);
        let depot_pos = pos(0, 0);

        let mut harvesters = vec![(0u64, &mut harvester, &harvester_pos)];
        let mut nodes: Vec<(EntityId, &mut ResourceNode, &Vec2Fixed)> = vec![];
        let depots = vec![(1u64, &depot_pos)];

        let events = economy_system(&mut harvesters, &mut nodes, &depots, &mut economy);

        // Should only deposit 20 (storage limit)
        assert!(events.iter().any(|e| matches!(
            e,
            EconomyEvent::ResourceDeposited {
                harvester: 0,
                depot: 1,
                amount: 20
            }
        )));

        assert_eq!(economy.feedstock, 100);
    }

    #[test]
    fn test_harvester_auto_return_when_full() {
        let mut harvester = Harvester::new(20, 10);
        harvester.state = HarvesterState::Gathering(1);

        let mut node = ResourceNode::new(pos(0, 0), 100, 10);
        let mut economy = PlayerEconomy::new(0, 1000);

        let harvester_pos = pos(0, 0);
        let node_pos = pos(0, 0);
        let depot_pos = pos(10, 10);

        let mut harvesters = vec![(0u64, &mut harvester, &harvester_pos)];
        let mut nodes = vec![(1u64, &mut node, &node_pos)];
        let depots = vec![(2u64, &depot_pos)];

        // First tick: gather 10
        let _ = economy_system(&mut harvesters, &mut nodes, &depots, &mut economy);
        assert_eq!(harvesters[0].1.current_load, 10);

        // Second tick: gather 10 more, now full -> should switch to returning
        let _ = economy_system(&mut harvesters, &mut nodes, &depots, &mut economy);
        assert_eq!(harvesters[0].1.current_load, 20);
        assert!(matches!(
            harvesters[0].1.state,
            HarvesterState::Returning(2)
        ));
    }

    #[test]
    fn test_income_rate_calculation() {
        let mut harvester1 = Harvester::new(100, 10);
        harvester1.state = HarvesterState::Gathering(3); // Node entity id is 3

        let mut harvester2 = Harvester::new(100, 15);
        harvester2.state = HarvesterState::Gathering(3); // Node entity id is 3

        let mut harvester3 = Harvester::new(100, 20);
        harvester3.state = HarvesterState::Idle; // Not gathering

        let mut node = ResourceNode::new(pos(0, 0), 1000, 10);
        let mut economy = PlayerEconomy::new(0, 1000);

        let pos1 = pos(0, 0);
        let pos2 = pos(0, 0);
        let pos3 = pos(0, 0);
        let node_pos = pos(0, 0);

        let mut harvesters = vec![
            (0u64, &mut harvester1, &pos1),
            (1u64, &mut harvester2, &pos2),
            (2u64, &mut harvester3, &pos3),
        ];
        let mut nodes = vec![(3u64, &mut node, &node_pos)];
        let depots: Vec<(EntityId, &Vec2Fixed)> = vec![];

        let _ = economy_system(&mut harvesters, &mut nodes, &depots, &mut economy);

        // Income rate should be sum of gather rates for actively gathering harvesters
        // harvester1 (10) + harvester2 (15) = 25
        assert_eq!(economy.income_rate, 25);
    }
}
