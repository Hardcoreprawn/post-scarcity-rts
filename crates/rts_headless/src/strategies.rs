//! Scripted AI strategies for headless playtesting.
//!
//! Strategies define build orders and tactical decisions for AI players
//! in automated game testing.

use std::collections::VecDeque;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error type for strategy operations.
#[derive(Error, Debug)]
pub enum StrategyError {
    /// File not found.
    #[error("Strategy file not found: {0}")]
    FileNotFound(String),
    /// Failed to read file.
    #[error("Failed to read strategy file: {0}")]
    ReadError(#[from] std::io::Error),
    /// Failed to parse RON.
    #[error("Failed to parse strategy: {0}")]
    ParseError(#[from] ron::error::SpannedError),
}

/// A complete AI strategy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    /// Strategy name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Build order to follow.
    pub build_order: Vec<BuildOrderItem>,
    /// Tick when first attack should happen.
    pub attack_timing: u64,
    /// Re-attack interval after first attack (in ticks).
    pub attack_interval: u64,
    /// Target army composition (unit_type -> fraction 0.0-1.0).
    pub composition: std::collections::HashMap<String, f64>,
    /// Economic targets.
    pub economy: EconomyTargets,
    /// Aggression level (0.0 = passive, 1.0 = hyper-aggressive).
    pub aggression: f64,
}

impl Default for Strategy {
    fn default() -> Self {
        Self {
            name: "Balanced".to_string(),
            description: "Standard balanced gameplay".to_string(),
            build_order: vec![
                BuildOrderItem::Unit("harvester".to_string()),
                BuildOrderItem::Building("barracks".to_string()),
                BuildOrderItem::Unit("infantry".to_string()),
                BuildOrderItem::Unit("infantry".to_string()),
                BuildOrderItem::Unit("harvester".to_string()),
            ],
            attack_timing: 12000,  // 200 seconds (3:20) at 60 tps
            attack_interval: 3600, // 60 seconds between attacks
            composition: [
                ("infantry".to_string(), 0.5),
                ("ranger".to_string(), 0.3),
                ("harvester".to_string(), 0.2),
            ]
            .into_iter()
            .collect(),
            economy: EconomyTargets::default(),
            aggression: 0.5,
        }
    }
}

impl Strategy {
    /// Load a strategy from a RON file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, StrategyError> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(StrategyError::FileNotFound(path.display().to_string()));
        }
        let contents = std::fs::read_to_string(path)?;
        let strategy: Strategy = ron::from_str(&contents)?;
        Ok(strategy)
    }

    /// Load from a RON string.
    pub fn from_ron_str(ron: &str) -> Result<Self, StrategyError> {
        let strategy: Strategy = ron::from_str(ron)?;
        Ok(strategy)
    }

    /// Create a "Rush" strategy (early aggression).
    #[must_use]
    pub fn rush() -> Self {
        Self {
            name: "Rush".to_string(),
            description: "Early aggression with cheap units".to_string(),
            build_order: vec![
                BuildOrderItem::Unit("infantry".to_string()),
                BuildOrderItem::Unit("infantry".to_string()),
                BuildOrderItem::Building("barracks".to_string()),
                BuildOrderItem::Unit("infantry".to_string()),
                BuildOrderItem::Unit("infantry".to_string()),
                BuildOrderItem::Unit("infantry".to_string()),
                BuildOrderItem::WaitForUnits("infantry".to_string(), 5),
            ],
            attack_timing: 7200,   // 120 seconds (balanced from 100s)
            attack_interval: 2100, // 35 seconds between waves
            composition: [
                ("infantry".to_string(), 0.9),
                ("harvester".to_string(), 0.1),
            ]
            .into_iter()
            .collect(),
            economy: EconomyTargets {
                target_harvesters: 1,
                target_supply_depots: 0,
                expand_at_resources: 2000,
            },
            aggression: 0.9,
        }
    }

    /// Create an "Economic" strategy (late game power).
    #[must_use]
    pub fn economic() -> Self {
        Self {
            name: "Economic".to_string(),
            description: "Focus on economy, powerful late game".to_string(),
            build_order: vec![
                BuildOrderItem::Unit("harvester".to_string()),
                BuildOrderItem::Building("supply_depot".to_string()),
                BuildOrderItem::Unit("harvester".to_string()),
                BuildOrderItem::Building("barracks".to_string()),
                BuildOrderItem::Unit("harvester".to_string()),
                BuildOrderItem::WaitForResources(2000),
                BuildOrderItem::Building("tech_lab".to_string()),
            ],
            attack_timing: 18000,  // 300 seconds (5 minutes)
            attack_interval: 6000, // 100 seconds between attacks
            composition: [
                ("infantry".to_string(), 0.3),
                ("ranger".to_string(), 0.4),
                ("harvester".to_string(), 0.3),
            ]
            .into_iter()
            .collect(),
            economy: EconomyTargets {
                target_harvesters: 4,
                target_supply_depots: 2,
                expand_at_resources: 1500,
            },
            aggression: 0.3,
        }
    }

    /// Create a "Turtle" strategy (defensive play).
    #[must_use]
    pub fn turtle() -> Self {
        Self {
            name: "Turtle".to_string(),
            description: "Defensive play with turrets".to_string(),
            build_order: vec![
                BuildOrderItem::Unit("harvester".to_string()),
                BuildOrderItem::Building("barracks".to_string()),
                BuildOrderItem::Building("turret".to_string()),
                BuildOrderItem::Unit("infantry".to_string()),
                BuildOrderItem::Building("turret".to_string()),
                BuildOrderItem::Unit("harvester".to_string()),
                BuildOrderItem::Building("supply_depot".to_string()),
            ],
            attack_timing: 24000,  // 400 seconds
            attack_interval: 9000, // 150 seconds
            composition: [
                ("infantry".to_string(), 0.4),
                ("ranger".to_string(), 0.4),
                ("harvester".to_string(), 0.2),
            ]
            .into_iter()
            .collect(),
            economy: EconomyTargets {
                target_harvesters: 3,
                target_supply_depots: 1,
                expand_at_resources: 3000, // Only expand when very rich
            },
            aggression: 0.1,
        }
    }

    /// Create a "Fast Expand" strategy (two-base economy rush).
    #[must_use]
    pub fn fast_expand() -> Self {
        Self {
            name: "FastExpand".to_string(),
            description: "Quick second base for economic advantage".to_string(),
            build_order: vec![
                BuildOrderItem::Unit("harvester".to_string()),
                BuildOrderItem::Unit("harvester".to_string()),
                BuildOrderItem::Building("supply_depot".to_string()),
                BuildOrderItem::WaitForResources(300),
                BuildOrderItem::Building("command_center".to_string()), // 2nd base
                BuildOrderItem::Unit("harvester".to_string()),
                BuildOrderItem::Building("barracks".to_string()),
                BuildOrderItem::Unit("infantry".to_string()),
                BuildOrderItem::Unit("harvester".to_string()),
            ],
            attack_timing: 15000,  // 250 seconds - medium timing
            attack_interval: 4800, // 80 seconds between attacks
            composition: [
                ("infantry".to_string(), 0.35),
                ("ranger".to_string(), 0.35),
                ("tank".to_string(), 0.15),
                ("harvester".to_string(), 0.15),
            ]
            .into_iter()
            .collect(),
            economy: EconomyTargets {
                target_harvesters: 6,
                target_supply_depots: 3,
                expand_at_resources: 1200,
            },
            aggression: 0.5,
        }
    }

    /// Create a "Harassment" strategy (constant raids and map control).
    #[must_use]
    pub fn harassment() -> Self {
        Self {
            name: "Harassment".to_string(),
            description: "Fast scouts and constant pressure".to_string(),
            build_order: vec![
                BuildOrderItem::Unit("scout".to_string()),
                BuildOrderItem::Unit("scout".to_string()),
                BuildOrderItem::Building("barracks".to_string()),
                BuildOrderItem::Unit("scout".to_string()),
                BuildOrderItem::Unit("ranger".to_string()),
                BuildOrderItem::Unit("scout".to_string()),
                BuildOrderItem::WaitForUnits("scout".to_string(), 4),
            ],
            attack_timing: 4800,   // 80 seconds - very early harassment
            attack_interval: 1200, // 20 seconds - constant pressure
            composition: [
                ("scout".to_string(), 0.5),
                ("ranger".to_string(), 0.35),
                ("infantry".to_string(), 0.15),
            ]
            .into_iter()
            .collect(),
            economy: EconomyTargets {
                target_harvesters: 2,
                target_supply_depots: 1,
                expand_at_resources: 2000,
            },
            aggression: 0.85,
        }
    }

    /// Create an "All-In" strategy (one big attack, no economy).
    #[must_use]
    pub fn all_in() -> Self {
        Self {
            name: "AllIn".to_string(),
            description: "Sacrifice economy for one decisive attack".to_string(),
            build_order: vec![
                BuildOrderItem::Unit("infantry".to_string()),
                BuildOrderItem::Unit("infantry".to_string()),
                BuildOrderItem::Unit("infantry".to_string()),
                BuildOrderItem::Building("barracks".to_string()),
                BuildOrderItem::Unit("infantry".to_string()),
                BuildOrderItem::Unit("infantry".to_string()),
                BuildOrderItem::Unit("infantry".to_string()),
                BuildOrderItem::Unit("infantry".to_string()),
                BuildOrderItem::WaitForUnits("infantry".to_string(), 7),
            ],
            attack_timing: 5400,  // 90 seconds - committed timing
            attack_interval: 600, // 10 seconds - no holding back
            composition: [("infantry".to_string(), 1.0)].into_iter().collect(),
            economy: EconomyTargets {
                target_harvesters: 0,
                target_supply_depots: 0,
                expand_at_resources: 99999, // Never expand
            },
            aggression: 1.0,
        }
    }
}

/// A single item in a build order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildOrderItem {
    /// Produce a unit.
    Unit(String),
    /// Construct a building.
    Building(String),
    /// Wait for a certain amount of resources.
    WaitForResources(i64),
    /// Wait for a certain number of a unit type.
    WaitForUnits(String, u32),
    /// Wait for a specific tick.
    WaitForTick(u64),
}

/// Economic targets for the AI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomyTargets {
    /// Target number of harvesters.
    pub target_harvesters: u32,
    /// Target number of supply depots.
    pub target_supply_depots: u32,
    /// Expand when resources exceed this amount.
    pub expand_at_resources: i64,
}

impl Default for EconomyTargets {
    fn default() -> Self {
        Self {
            target_harvesters: 2,
            target_supply_depots: 1,
            expand_at_resources: 2500,
        }
    }
}

/// Tactical decision types for AI actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TacticalDecision {
    /// Do nothing special.
    Hold,
    /// Attack the enemy.
    Attack,
    /// Rally units to defend base.
    Defend,
    /// Expand to a new location.
    Expand,
    /// Send scouts to explore.
    Scout,
}

/// Tactical rules that trigger decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TacticalRule {
    /// Name of the rule.
    pub name: String,
    /// Priority (higher = checked first).
    pub priority: u32,
    /// Condition expression.
    pub condition: String,
    /// Action to take when condition is met.
    pub action: String,
}

/// Runtime state for executing a strategy.
#[derive(Debug, Clone)]
pub struct StrategyExecutor {
    /// The strategy being executed.
    strategy: Strategy,
    /// Remaining build order items.
    build_queue: VecDeque<BuildOrderItem>,
    /// Current build order index (for tracking progress).
    current_index: usize,
    /// Whether attack has been triggered.
    attack_triggered: bool,
    /// Last attack tick.
    last_attack_tick: u64,
}

impl StrategyExecutor {
    /// Create a new executor for a strategy.
    #[must_use]
    pub fn new(strategy: Strategy) -> Self {
        let build_queue = strategy.build_order.iter().cloned().collect();
        Self {
            strategy,
            build_queue,
            current_index: 0,
            attack_triggered: false,
            last_attack_tick: 0,
        }
    }

    /// Get the next build order item if conditions are met.
    pub fn next_item(
        &mut self,
        current_tick: u64,
        resources: i64,
        unit_counts: &std::collections::HashMap<String, u32>,
    ) -> Option<BuildOrderItem> {
        loop {
            let item = self.build_queue.front()?;

            match item {
                BuildOrderItem::WaitForResources(amount) => {
                    if resources >= *amount {
                        self.build_queue.pop_front();
                        self.current_index += 1;
                        continue;
                    }
                    return None;
                }
                BuildOrderItem::WaitForUnits(unit_type, count) => {
                    let current = unit_counts.get(unit_type).copied().unwrap_or(0);
                    if current >= *count {
                        self.build_queue.pop_front();
                        self.current_index += 1;
                        continue;
                    }
                    return None;
                }
                BuildOrderItem::WaitForTick(tick) => {
                    if current_tick >= *tick {
                        self.build_queue.pop_front();
                        self.current_index += 1;
                        continue;
                    }
                    return None;
                }
                _ => {
                    self.current_index += 1;
                    return self.build_queue.pop_front();
                }
            }
        }
    }

    /// Check if should attack based on timing.
    #[must_use]
    pub fn should_attack(&mut self, current_tick: u64) -> bool {
        if current_tick >= self.strategy.attack_timing && !self.attack_triggered {
            self.attack_triggered = true;
            self.last_attack_tick = current_tick;
            return true;
        }

        if self.attack_triggered
            && current_tick >= self.last_attack_tick + self.strategy.attack_interval
        {
            self.last_attack_tick = current_tick;
            return true;
        }

        false
    }

    /// Get the strategy name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.strategy.name
    }

    /// Get aggression level.
    #[must_use]
    pub fn aggression(&self) -> f64 {
        self.strategy.aggression
    }

    /// Get the target composition.
    #[must_use]
    pub fn composition(&self) -> &std::collections::HashMap<String, f64> {
        &self.strategy.composition
    }

    /// Get economy targets.
    #[must_use]
    pub fn economy(&self) -> &EconomyTargets {
        &self.strategy.economy
    }

    /// Get build order progress as a fraction.
    #[must_use]
    pub fn progress(&self) -> f64 {
        if self.strategy.build_order.is_empty() {
            1.0
        } else {
            self.current_index as f64 / self.strategy.build_order.len() as f64
        }
    }

    /// Decide what tactical action to take based on current game state.
    #[must_use]
    pub fn decide_action(
        &self,
        current_tick: u64,
        army_supply: u32,
        enemy_army_supply: u32,
        base_under_attack: bool,
    ) -> TacticalDecision {
        // If base is under attack, defend
        if base_under_attack {
            return TacticalDecision::Defend;
        }

        // Check if we should attack based on timing
        if current_tick >= self.strategy.attack_timing {
            // Attack if we have army advantage based on aggression
            let threshold = (1.0 - self.strategy.aggression) as u32 * enemy_army_supply;
            if army_supply >= threshold || self.strategy.aggression > 0.7 {
                return TacticalDecision::Attack;
            }
        }

        TacticalDecision::Hold
    }

    /// Get the next build item that should be built (convenience alias for next_item).
    pub fn next_build_item(
        &mut self,
        current_tick: u64,
        resources: i64,
        unit_counts: &std::collections::HashMap<String, u32>,
    ) -> Option<BuildOrderItem> {
        self.next_item(current_tick, resources, unit_counts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_strategy() {
        let strategy = Strategy::default();
        assert_eq!(strategy.name, "Balanced");
        assert!(!strategy.build_order.is_empty());
    }

    #[test]
    fn test_rush_strategy() {
        let strategy = Strategy::rush();
        assert_eq!(strategy.aggression, 0.9);
        assert!(strategy.attack_timing < 10000);
    }

    #[test]
    fn test_economic_strategy() {
        let strategy = Strategy::economic();
        assert!(strategy.economy.target_harvesters > 2);
        assert!(strategy.attack_timing > 15000);
    }

    #[test]
    fn test_executor_next_item() {
        let strategy = Strategy::rush();
        let mut executor = StrategyExecutor::new(strategy);

        let counts = std::collections::HashMap::new();
        let item = executor.next_item(0, 1000, &counts);
        assert!(matches!(item, Some(BuildOrderItem::Unit(_))));
    }

    #[test]
    fn test_executor_should_attack() {
        let mut strategy = Strategy::default();
        strategy.attack_timing = 100;
        strategy.attack_interval = 50;

        let mut executor = StrategyExecutor::new(strategy);

        // Before attack timing
        assert!(!executor.should_attack(50));

        // At attack timing
        assert!(executor.should_attack(100));

        // Right after (in interval)
        assert!(!executor.should_attack(120));

        // After interval
        assert!(executor.should_attack(160));
    }
}
