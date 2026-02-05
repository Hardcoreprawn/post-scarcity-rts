//! Production plugin for unit production from buildings.
//!
//! Handles production queues and unit spawning.

use bevy::prelude::*;

use crate::bundles::{HarvesterBundle, UnitBundle};
use crate::components::{
    GameFaction, GamePosition, GameProductionQueue, PlayerFaction, Unit, UnitType,
};
use crate::data_loader::{BevyUnitKindRegistry, FactionRegistry};
use crate::economy::PlayerResources;

/// Ticks per second - used to convert build_time from ticks (stored in data) to seconds (used in calculations).
const TICKS_PER_SECOND: f32 = 60.0;

/// Default build time in seconds when unit data is not found.
const DEFAULT_BUILD_TIME_SECONDS: f32 = 5.0;

/// Check if unit data indicates a ranged unit type.
fn is_ranged_unit(unit_data: &rts_core::data::UnitData) -> bool {
    unit_data.has_tag("ranged") || unit_data.has_tag("ranger")
}

/// Plugin that handles unit production from buildings.
pub struct ProductionPlugin;

impl Plugin for ProductionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, production_system);
    }
}

/// Event sent when a unit should be spawned from production.
#[derive(Event)]
pub struct SpawnUnitEvent {
    /// Type of unit to spawn.
    pub unit_type: UnitType,
    /// Position to spawn at.
    pub position: Vec2,
    /// Faction the unit belongs to.
    pub faction: rts_core::factions::FactionId,
}

/// Advances production queues and spawns completed units.
fn production_system(
    time: Res<Time>,
    mut commands: Commands,
    mut buildings: Query<(&GamePosition, &GameFaction, &mut GameProductionQueue)>,
    mut resources: ResMut<PlayerResources>,
    faction_registry: Res<FactionRegistry>,
    unit_kind_registry: Res<BevyUnitKindRegistry>,
    player_faction: Res<PlayerFaction>,
) {
    let dt = time.delta_seconds();

    for (pos, faction, mut production) in buildings.iter_mut() {
        // Only process if there's something in the queue
        if let Some(current) = production.current_mut() {
            // Look up build time from faction data
            let faction_data = faction_registry.get(faction.faction);
            let build_time = if let Some(data) = faction_data {
                if let Some(unit_data) = data.get_unit(&current.unit_id) {
                    // Convert ticks to seconds
                    unit_data.build_time as f32 / TICKS_PER_SECOND
                } else {
                    // Fallback if unit not found in data
                    tracing::warn!(
                        "Unit '{}' not found in faction {:?} data, using default build time",
                        current.unit_id,
                        faction.faction
                    );
                    DEFAULT_BUILD_TIME_SECONDS
                }
            } else {
                DEFAULT_BUILD_TIME_SECONDS
            };

            current.progress += dt / build_time;

            // Check if complete
            if current.progress >= 1.0 {
                let unit_id = production.complete_current().unwrap();
                let spawn_pos: Vec2 = production.rally_point.unwrap_or_else(|| {
                    Vec2::new(
                        pos.value.x.to_num::<f32>() + 80.0,
                        pos.value.y.to_num::<f32>(),
                    )
                });

                // Look up faction-specific unit data
                let faction_data = faction_registry.get(faction.faction);

                // Try to spawn with data-driven stats
                if let Some(data) = faction_data {
                    if let Some(unit_data) = data.get_unit(&unit_id) {
                        // Look up the UnitKindId from the registry
                        let unit_kind_id = unit_kind_registry
                            .find(faction.faction, &unit_id)
                            .unwrap_or(rts_core::unit_kind::UnitKindId::NONE);

                        // Check if this is a harvester by looking at tags
                        if unit_data.has_tag("harvester") || unit_data.has_tag("worker") {
                            // Harvesters use special bundle with harvester component
                            commands
                                .spawn(HarvesterBundle::new(spawn_pos, faction.faction))
                                .insert(Unit::new(UnitType::Harvester));
                            // Supply was already reserved at queue time
                        } else {
                            // Combat units use data-driven stats
                            // Determine appropriate UnitType for backward compatibility
                            let unit_type = if is_ranged_unit(unit_data) {
                                UnitType::Ranger
                            } else {
                                UnitType::Infantry
                            };
                            
                            commands
                                .spawn(UnitBundle::from_data(
                                    spawn_pos,
                                    faction.faction,
                                    unit_data,
                                    unit_kind_id,
                                ))
                                .insert(Unit::new(UnitType::Infantry)); // Legacy component for supply tracking
                            // Supply was already reserved at queue time
                        }

                        tracing::info!(
                            "Produced {} ({}) for {:?}",
                            unit_data.name,
                            unit_id,
                            faction.faction
                        );
                    } else {
                        // Unit not found in data, log error
                        tracing::error!(
                            "Unit '{}' not found in faction {:?} data, cannot spawn",
                            unit_id,
                            faction.faction
                        );
                    }
                } else {
                    // No faction data loaded, log error
                    tracing::error!(
                        "No faction data for {:?}, cannot spawn unit '{}'",
                        faction.faction,
                        unit_id
                    );
                }
            }
        }
    }
}

