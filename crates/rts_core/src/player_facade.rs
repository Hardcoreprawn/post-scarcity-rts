//! Unified player interface for fair AI and player interactions.
//!
//! This module defines the `PlayerFacade` trait that both human players and AI
//! use to interact with the simulation. By using the same interface, we ensure:
//!
//! - **Fair play:** AI can only see what a player would see (visibility-filtered)
//! - **Consistent behavior:** Same commands, same information access
//! - **Fast testing:** Direct facade calls without Bevy overhead for batch testing
//! - **Multiplayer-ready:** Same visibility rules for all clients

use crate::components::{Command, EntityId};
use crate::economy::PlayerEconomy;
use crate::error::Result;
use crate::factions::FactionId;
use crate::math::{Fixed, Vec2Fixed};
use crate::simulation::Simulation;

/// Basic information about a unit that can be queried through the facade.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitInfo {
    /// Entity ID.
    pub id: EntityId,
    /// Current position.
    pub position: Vec2Fixed,
    /// Current health (if visible).
    pub health: Option<u32>,
    /// Maximum health (if visible).
    pub max_health: Option<u32>,
    /// Faction this unit belongs to.
    pub faction: FactionId,
    /// Whether this is a depot/base building.
    pub is_depot: bool,
}

/// Position and basic info for an enemy unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisibleEnemy {
    /// Entity ID.
    pub id: EntityId,
    /// Current position.
    pub position: Vec2Fixed,
    /// Whether this is a depot/base (high-value target).
    pub is_depot: bool,
}

/// Trait defining what actions a player (human or AI) can perform.
///
/// This trait ensures that both human players and AI opponents have
/// identical capabilities and information access. The only difference
/// is who (or what) makes the decisions.
///
/// # Visibility Rules
///
/// All queries that return enemy information are filtered by visibility.
/// An entity is visible if it's within the vision range of any friendly unit.
/// This prevents AI from "cheating" by seeing the entire map.
///
/// # Command Flow
///
/// All unit control flows through `issue_command()`. There are no backdoor
/// APIs that bypass the command system. This ensures replay compatibility
/// and deterministic behavior.
pub trait PlayerFacade {
    /// Issue a command to a single unit.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The unit doesn't exist
    /// - The unit doesn't belong to this player's faction
    /// - The unit cannot receive commands
    fn issue_command(&mut self, unit: EntityId, command: Command) -> Result<()>;

    /// Issue the same command to multiple units.
    ///
    /// Convenience method for group orders.
    fn issue_commands(&mut self, units: &[EntityId], command: Command) -> Result<()> {
        for &unit in units {
            self.issue_command(unit, command.clone())?;
        }
        Ok(())
    }

    /// Get all entities owned by this player's faction.
    fn get_own_entities(&self) -> Vec<EntityId>;

    /// Get all enemy entities currently visible to this player.
    ///
    /// Only returns enemies within vision range of any friendly unit.
    fn get_visible_enemies(&self) -> Vec<VisibleEnemy>;

    /// Query information about a specific entity.
    ///
    /// Returns `None` if the entity doesn't exist or isn't visible.
    fn query_entity(&self, id: EntityId) -> Option<UnitInfo>;

    /// Get the player's current economy state.
    fn get_resources(&self) -> PlayerEconomy;

    /// Get the faction ID this facade represents.
    fn faction(&self) -> FactionId;
}

/// Default vision range multiplier when no explicit vision_range is set.
/// Units can see 2× their attack range.
pub const DEFAULT_VISION_MULTIPLIER: i32 = 2;

impl Simulation {
    /// Check if a target entity is visible to a faction.
    ///
    /// An entity is visible if it's within the vision range of any entity
    /// belonging to the viewing faction.
    ///
    /// # Arguments
    /// * `viewer_faction` - The faction trying to see
    /// * `target_id` - The entity to check visibility of
    ///
    /// # Returns
    /// `true` if the target is within vision range of any friendly unit
    #[must_use]
    pub fn is_visible_to(&self, viewer_faction: FactionId, target_id: EntityId) -> bool {
        let Some(target) = self.entities().get(target_id) else {
            return false;
        };
        let Some(target_pos) = target.position.as_ref() else {
            return false;
        };

        // Check all entities belonging to the viewing faction
        for (_, entity) in self.entities().iter() {
            let Some(faction) = entity.faction.as_ref() else {
                continue;
            };
            if faction.faction != viewer_faction {
                continue;
            }
            let Some(own_pos) = entity.position.as_ref() else {
                continue;
            };

            // Calculate vision range: use explicit vision_range, or 2× attack range, or default
            let vision_range = entity
                .vision_range
                .or_else(|| {
                    entity
                        .combat_stats
                        .map(|s| s.range * Fixed::from_num(DEFAULT_VISION_MULTIPLIER))
                })
                .unwrap_or(Fixed::from_num(100)); // Default vision for non-combat units

            let dist_sq = own_pos.value.distance_squared(target_pos.value);
            let vision_range_sq = vision_range * vision_range;

            if dist_sq <= vision_range_sq {
                return true;
            }
        }

        false
    }

    /// Get all enemies visible to a faction.
    ///
    /// Returns position and basic info for each visible enemy unit.
    #[must_use]
    pub fn get_visible_enemies_for(&self, faction: FactionId) -> Vec<VisibleEnemy> {
        let mut visible = Vec::new();

        for entity_id in self.entities().sorted_ids() {
            let Some(entity) = self.entities().get(entity_id) else {
                continue;
            };

            // Skip entities without faction or position
            let Some(entity_faction) = entity.faction.as_ref() else {
                continue;
            };
            let Some(pos) = entity.position.as_ref() else {
                continue;
            };

            // Skip friendlies
            if entity_faction.faction == faction {
                continue;
            }

            // Check visibility
            if !self.is_visible_to(faction, entity_id) {
                continue;
            }

            visible.push(VisibleEnemy {
                id: entity_id,
                position: pos.value,
                is_depot: entity.depot.is_some(),
            });
        }

        visible
    }

    /// Get all entities owned by a faction.
    #[must_use]
    pub fn get_faction_entities(&self, faction: FactionId) -> Vec<EntityId> {
        self.entities()
            .iter()
            .filter_map(|(id, entity)| {
                entity
                    .faction
                    .as_ref()
                    .filter(|f| f.faction == faction)
                    .map(|_| *id)
            })
            .collect()
    }
}

/// A player facade that wraps a `Simulation` for a specific faction.
///
/// This is the primary implementation of `PlayerFacade` for both:
/// - Headless batch testing (AI vs AI)
/// - GUI AI opponents
///
/// The facade restricts all queries to visibility-filtered results and
/// routes all commands through the standard command system.
pub struct SimulationPlayerFacade<'a> {
    /// Reference to the simulation.
    sim: &'a mut Simulation,
    /// The faction this facade represents.
    faction: FactionId,
    /// Cached economy state (optional, for headless where we don't track per-player economy).
    economy: PlayerEconomy,
}

impl<'a> SimulationPlayerFacade<'a> {
    /// Create a new player facade for a specific faction.
    pub fn new(sim: &'a mut Simulation, faction: FactionId) -> Self {
        Self {
            sim,
            faction,
            economy: PlayerEconomy::default(),
        }
    }

    /// Create a facade with specific economy state.
    pub fn with_economy(
        sim: &'a mut Simulation,
        faction: FactionId,
        economy: PlayerEconomy,
    ) -> Self {
        Self {
            sim,
            faction,
            economy,
        }
    }

    /// Get a reference to the underlying simulation.
    ///
    /// Use sparingly - prefer facade methods for AI code to ensure fairness.
    pub fn simulation(&self) -> &Simulation {
        self.sim
    }
}

impl<'a> PlayerFacade for SimulationPlayerFacade<'a> {
    fn issue_command(&mut self, unit: EntityId, command: Command) -> Result<()> {
        // Verify the unit belongs to our faction
        if let Some(entity) = self.sim.entities().get(unit) {
            if let Some(faction) = entity.faction.as_ref() {
                if faction.faction != self.faction {
                    return Err(crate::error::GameError::InvalidState(format!(
                        "Unit {} does not belong to faction {:?}",
                        unit, self.faction
                    )));
                }
            }
        }

        self.sim.apply_command(unit, command)
    }

    fn get_own_entities(&self) -> Vec<EntityId> {
        self.sim.get_faction_entities(self.faction)
    }

    fn get_visible_enemies(&self) -> Vec<VisibleEnemy> {
        self.sim.get_visible_enemies_for(self.faction)
    }

    fn query_entity(&self, id: EntityId) -> Option<UnitInfo> {
        let entity = self.sim.entities().get(id)?;
        let pos = entity.position.as_ref()?;
        let faction_member = entity.faction.as_ref()?;

        // If enemy, check visibility
        if faction_member.faction != self.faction && !self.sim.is_visible_to(self.faction, id) {
            return None;
        }

        Some(UnitInfo {
            id,
            position: pos.value,
            health: entity.health.map(|h| h.current),
            max_health: entity.health.map(|h| h.max),
            faction: faction_member.faction,
            is_depot: entity.depot.is_some(),
        })
    }

    fn get_resources(&self) -> PlayerEconomy {
        self.economy
    }

    fn faction(&self) -> FactionId {
        self.faction
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{CombatStats, FactionMember};
    use crate::simulation::EntitySpawnParams;

    fn spawn_unit_for_faction(
        sim: &mut Simulation,
        faction: FactionId,
        pos: Vec2Fixed,
        attack_range: Fixed,
    ) -> EntityId {
        let faction_member = FactionMember::new(faction, 0);
        sim.spawn_entity(EntitySpawnParams {
            position: Some(pos),
            health: Some(100),
            movement: Some(Fixed::from_num(2)),
            combat_stats: Some(CombatStats::new(10, attack_range, 30)),
            faction: Some(faction_member),
            ..Default::default()
        })
    }

    #[test]
    fn test_visibility_within_range() {
        let mut sim = Simulation::new();

        // Spawn a unit for Continuity at origin with range 50
        let _friendly = spawn_unit_for_faction(
            &mut sim,
            FactionId::Continuity,
            Vec2Fixed::ZERO,
            Fixed::from_num(50),
        );

        // Spawn enemy at distance 80 (within 2× attack range = 100)
        let enemy = spawn_unit_for_faction(
            &mut sim,
            FactionId::Collegium,
            Vec2Fixed::new(Fixed::from_num(80), Fixed::from_num(0)),
            Fixed::from_num(50),
        );

        assert!(sim.is_visible_to(FactionId::Continuity, enemy));
    }

    #[test]
    fn test_visibility_outside_range() {
        let mut sim = Simulation::new();

        // Spawn a unit for Continuity at origin with range 50
        let _friendly = spawn_unit_for_faction(
            &mut sim,
            FactionId::Continuity,
            Vec2Fixed::ZERO,
            Fixed::from_num(50),
        );

        // Spawn enemy at distance 200 (outside 2× attack range = 100)
        let enemy = spawn_unit_for_faction(
            &mut sim,
            FactionId::Collegium,
            Vec2Fixed::new(Fixed::from_num(200), Fixed::from_num(0)),
            Fixed::from_num(50),
        );

        assert!(!sim.is_visible_to(FactionId::Continuity, enemy));
    }

    #[test]
    fn test_facade_only_sees_visible_enemies() {
        let mut sim = Simulation::new();

        // Spawn friendly at origin with range 50 (vision = 100)
        let _friendly = spawn_unit_for_faction(
            &mut sim,
            FactionId::Continuity,
            Vec2Fixed::ZERO,
            Fixed::from_num(50),
        );

        // Spawn two enemies: one visible, one not
        let _visible_enemy = spawn_unit_for_faction(
            &mut sim,
            FactionId::Collegium,
            Vec2Fixed::new(Fixed::from_num(80), Fixed::from_num(0)),
            Fixed::from_num(50),
        );
        let _hidden_enemy = spawn_unit_for_faction(
            &mut sim,
            FactionId::Collegium,
            Vec2Fixed::new(Fixed::from_num(500), Fixed::from_num(0)),
            Fixed::from_num(50),
        );

        let facade = SimulationPlayerFacade::new(&mut sim, FactionId::Continuity);
        let visible = facade.get_visible_enemies();

        assert_eq!(visible.len(), 1);
    }

    #[test]
    fn test_facade_cannot_command_enemy_units() {
        let mut sim = Simulation::new();

        let enemy = spawn_unit_for_faction(
            &mut sim,
            FactionId::Collegium,
            Vec2Fixed::ZERO,
            Fixed::from_num(50),
        );

        let mut facade = SimulationPlayerFacade::new(&mut sim, FactionId::Continuity);
        let result = facade.issue_command(enemy, Command::Stop);

        assert!(result.is_err());
    }

    #[test]
    fn test_facade_can_command_own_units() {
        let mut sim = Simulation::new();

        let friendly = spawn_unit_for_faction(
            &mut sim,
            FactionId::Continuity,
            Vec2Fixed::ZERO,
            Fixed::from_num(50),
        );

        let mut facade = SimulationPlayerFacade::new(&mut sim, FactionId::Continuity);
        let result = facade.issue_command(friendly, Command::Stop);

        assert!(result.is_ok());
    }

    #[test]
    fn test_get_own_entities() {
        let mut sim = Simulation::new();

        // Spawn 3 friendlies and 2 enemies
        let _f1 = spawn_unit_for_faction(
            &mut sim,
            FactionId::Continuity,
            Vec2Fixed::ZERO,
            Fixed::from_num(50),
        );
        let _f2 = spawn_unit_for_faction(
            &mut sim,
            FactionId::Continuity,
            Vec2Fixed::new(Fixed::from_num(10), Fixed::from_num(0)),
            Fixed::from_num(50),
        );
        let _f3 = spawn_unit_for_faction(
            &mut sim,
            FactionId::Continuity,
            Vec2Fixed::new(Fixed::from_num(20), Fixed::from_num(0)),
            Fixed::from_num(50),
        );
        let _e1 = spawn_unit_for_faction(
            &mut sim,
            FactionId::Collegium,
            Vec2Fixed::new(Fixed::from_num(100), Fixed::from_num(0)),
            Fixed::from_num(50),
        );
        let _e2 = spawn_unit_for_faction(
            &mut sim,
            FactionId::Collegium,
            Vec2Fixed::new(Fixed::from_num(200), Fixed::from_num(0)),
            Fixed::from_num(50),
        );

        let facade = SimulationPlayerFacade::new(&mut sim, FactionId::Continuity);
        let own = facade.get_own_entities();

        assert_eq!(own.len(), 3);
    }
}
