//! Production plugin for unit production from buildings.
//!
//! Handles production queues and unit spawning.

use bevy::prelude::*;

use crate::bundles::{HarvesterBundle, UnitBundle};
use crate::components::{
    GameFaction, GamePosition, GameProductionQueue, PlayerFaction, Unit, UnitType,
};
use crate::data_loader::FactionRegistry;
use crate::economy::PlayerResources;

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
    player_faction: Res<PlayerFaction>,
) {
    let dt = time.delta_seconds();

    for (pos, faction, mut production) in buildings.iter_mut() {
        // Only process if there's something in the queue
        if let Some(current) = production.current_mut() {
            let build_time = current.unit_type.build_time();
            current.progress += dt / build_time;

            // Check if complete
            if current.progress >= 1.0 {
                let unit_type = production.complete_current().unwrap();
                let spawn_pos: Vec2 = production.rally_point.unwrap_or_else(|| {
                    Vec2::new(
                        pos.value.x.to_num::<f32>() + 80.0,
                        pos.value.y.to_num::<f32>(),
                    )
                });

                // Look up faction-specific unit data
                let unit_id = unit_type.to_unit_id(faction.faction);
                let faction_data = faction_registry.get(faction.faction);

                // Try to spawn with data-driven stats, fall back to legacy if data missing
                match (faction_data, unit_type) {
                    (Some(_data), UnitType::Harvester) => {
                        // Harvesters use special bundle with harvester component
                        commands
                            .spawn(HarvesterBundle::new(spawn_pos, faction.faction))
                            .insert(Unit::new(unit_type));
                        // Supply was already reserved at queue time
                    }
                    (Some(data), _) => {
                        // Combat units use data-driven stats
                        if let Some(unit_data) = data.get_unit(unit_id) {
                            commands
                                .spawn(UnitBundle::from_data(spawn_pos, faction.faction, unit_data))
                                .insert(Unit::new(unit_type));
                            // Supply was already reserved at queue time
                        } else {
                            // Unit not found in data, use legacy spawn
                            tracing::warn!(
                                "Unit '{}' not found in faction {:?} data, using legacy spawn",
                                unit_id,
                                faction.faction
                            );
                            let is_player = faction.faction == player_faction.faction;
                            spawn_legacy_unit(
                                &mut commands,
                                unit_type,
                                spawn_pos,
                                faction.faction,
                                &mut resources,
                                is_player,
                            );
                        }
                    }
                    (None, _) => {
                        // No faction data loaded, use legacy spawn
                        tracing::warn!(
                            "No faction data for {:?}, using legacy spawn",
                            faction.faction
                        );
                        let is_player = faction.faction == player_faction.faction;
                        spawn_legacy_unit(
                            &mut commands,
                            unit_type,
                            spawn_pos,
                            faction.faction,
                            &mut resources,
                            is_player,
                        );
                    }
                }

                tracing::info!(
                    "Produced {:?} ({}) for {:?}",
                    unit_type,
                    unit_id,
                    faction.faction
                );
            }
        }
    }
}

/// Fallback spawning with hardcoded values (for backwards compatibility).
fn spawn_legacy_unit(
    commands: &mut Commands,
    unit_type: UnitType,
    spawn_pos: Vec2,
    faction: rts_core::factions::FactionId,
    _resources: &mut ResMut<PlayerResources>,
    _is_player: bool,
) {
    // Supply is now reserved at queue time in UI, not at spawn time
    match unit_type {
        UnitType::Infantry => {
            commands
                .spawn(UnitBundle::new(spawn_pos, faction, 100))
                .insert(Unit::new(unit_type));
        }
        UnitType::Harvester => {
            commands
                .spawn(HarvesterBundle::new(spawn_pos, faction))
                .insert(Unit::new(unit_type));
        }
        UnitType::Ranger => {
            commands
                .spawn(UnitBundle::new(spawn_pos, faction, 75))
                .insert(Unit::new(unit_type));
        }
    }
}
