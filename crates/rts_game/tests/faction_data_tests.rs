//! Tests for data-driven faction spawning.
//!
//! Verifies that units spawn with stats from RON data files,
//! not hardcoded values. Issue #2.

use bevy::prelude::*;
use rts_core::factions::FactionId;
use rts_game::bundles::UnitBundle;
use rts_game::components::{GameHealth, UnitType};
use rts_game::data_loader::{load_factions_from_directory, FactionRegistry};
use std::path::Path;

/// Helper to load faction data from assets directory.
fn load_test_registry() -> FactionRegistry {
    // Try multiple paths (running from workspace root or crate directory)
    let paths = [
        Path::new("crates/rts_game/assets/data/factions"),
        Path::new("assets/data/factions"),
    ];

    for path in &paths {
        if path.exists() {
            match load_factions_from_directory(path) {
                Ok(registry) => return registry,
                Err(e) => eprintln!("Failed to load from {:?}: {}", path, e),
            }
        }
    }

    panic!("Could not find faction data directory. Tried: {:?}", paths);
}

// ==========================================================================
// Faction Data Loading Tests
// ==========================================================================

#[test]
fn test_faction_registry_loads_all_factions() {
    let registry = load_test_registry();

    // All 5 factions should be loaded
    assert!(
        registry.contains(FactionId::Continuity),
        "Continuity not loaded"
    );
    assert!(
        registry.contains(FactionId::Collegium),
        "Collegium not loaded"
    );
    assert!(registry.contains(FactionId::Tinkers), "Tinkers not loaded");
    assert!(
        registry.contains(FactionId::BioSovereigns),
        "BioSovereigns not loaded"
    );
    assert!(registry.contains(FactionId::Zephyr), "Zephyr not loaded");
}

#[test]
fn test_all_factions_define_infantry_unit() {
    let registry = load_test_registry();

    // Use the new role-based system: find units with "infantry" tag
    // instead of relying on hardcoded ID mappings
    let factions = [
        FactionId::Continuity,
        FactionId::Collegium,
        FactionId::Tinkers,
        FactionId::BioSovereigns,
        FactionId::Zephyr,
    ];

    for faction_id in factions {
        let faction_data = registry.get(faction_id).expect("Faction should be loaded");

        // Find any unit with the "infantry" tag (data-driven approach)
        let has_infantry = faction_data
            .units
            .iter()
            .any(|unit| unit.tags.iter().any(|tag| tag == "infantry"));

        assert!(
            has_infantry,
            "Faction {:?} has no unit with 'infantry' tag",
            faction_id,
        );
    }
}

#[test]
fn test_unit_type_to_unit_id_mapping() {
    // Verify UnitType::to_unit_id returns correct RON identifiers
    assert_eq!(
        UnitType::Infantry.to_unit_id(FactionId::Continuity),
        "security_team"
    );
    assert_eq!(
        UnitType::Harvester.to_unit_id(FactionId::Continuity),
        "collection_vehicle"
    );
    assert_eq!(
        UnitType::Ranger.to_unit_id(FactionId::Continuity),
        "crowd_management_unit"
    );

    // Check another faction
    assert_eq!(
        UnitType::Infantry.to_unit_id(FactionId::Collegium),
        "attack_drone_squadron"
    );
}

// ==========================================================================
// RON Data Correctness Tests
// ==========================================================================

#[test]
fn test_continuity_security_team_stats() {
    let registry = load_test_registry();

    let faction_data = registry.get(FactionId::Continuity).unwrap();
    let unit_data = faction_data.get_unit("security_team").unwrap();

    // Verify RON-defined stats (from continuity.ron)
    assert_eq!(unit_data.health, 80, "Security team should have 80 HP");
    assert_eq!(unit_data.cost, 50, "Security team should cost 50");

    let combat = unit_data.combat.as_ref().expect("Should have combat stats");
    assert_eq!(combat.damage, 12, "Security team should deal 12 damage");
    assert_eq!(combat.armor, 5, "Security team should have 5 armor");
}

#[test]
fn test_factions_have_distinct_unit_stats() {
    let registry = load_test_registry();

    // Get infantry stats for two different factions
    let continuity = registry.get(FactionId::Continuity).unwrap();
    let collegium = registry.get(FactionId::Collegium).unwrap();

    let cont_infantry = continuity.get_unit("security_team").unwrap();
    let coll_infantry = collegium.get_unit("attack_drone_squadron").unwrap();

    // Factions should have differentiated stats (not all identical)
    // At minimum, they should have different names
    assert_ne!(
        cont_infantry.name, coll_infantry.name,
        "Faction infantry should have different names"
    );
}

// ==========================================================================
// Integration Tests - Spawning with Correct Stats
// ==========================================================================

#[test]
fn test_spawn_unit_from_data_has_correct_health() {
    let registry = load_test_registry();

    let faction_data = registry.get(FactionId::Continuity).unwrap();
    let unit_data = faction_data.get_unit("security_team").unwrap();

    // Create a minimal app to test spawning
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let entity = app.world_mut().spawn(UnitBundle::from_data(
        Vec2::ZERO,
        FactionId::Continuity,
        unit_data,
    ));
    let entity_id = entity.id();

    // Check spawned entity has correct health from RON (80), not hardcoded (100)
    let health = app.world().get::<GameHealth>(entity_id).unwrap();
    assert_eq!(
        health.max, 80,
        "Unit should have max health 80 from RON, not hardcoded 100"
    );
    assert_eq!(health.current, 80, "Unit should spawn at full health");
}

#[test]
fn test_spawn_unit_from_data_vs_legacy_differs() {
    let registry = load_test_registry();

    let faction_data = registry.get(FactionId::Continuity).unwrap();
    let unit_data = faction_data.get_unit("security_team").unwrap();

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Spawn with data-driven method
    let data_entity = app
        .world_mut()
        .spawn(UnitBundle::from_data(
            Vec2::ZERO,
            FactionId::Continuity,
            unit_data,
        ))
        .id();

    // Spawn with legacy method (hardcoded 100 HP)
    let legacy_entity = app
        .world_mut()
        .spawn(UnitBundle::new(Vec2::ZERO, FactionId::Continuity, 100))
        .id();

    let data_health = app.world().get::<GameHealth>(data_entity).unwrap();
    let legacy_health = app.world().get::<GameHealth>(legacy_entity).unwrap();

    // They should differ because RON defines 80 HP, legacy uses 100
    assert_ne!(
        data_health.max, legacy_health.max,
        "Data-driven (80) should differ from legacy (100)"
    );
    assert_eq!(data_health.max, 80);
    assert_eq!(legacy_health.max, 100);
}

// ==========================================================================
// Starting Units Tests
// ==========================================================================

#[test]
fn test_starting_units_defined_in_ron() {
    let registry = load_test_registry();

    let faction_data = registry.get(FactionId::Continuity).unwrap();

    // Continuity should have starting units defined
    assert!(
        !faction_data.starting_units.is_empty(),
        "Faction should have starting units"
    );

    // First starting unit should be security_team
    assert_eq!(faction_data.starting_units[0].type_id, "security_team");
}

#[test]
fn test_all_factions_have_starting_configuration() {
    let registry = load_test_registry();

    for faction_id in registry.faction_ids() {
        let faction_data = registry.get(faction_id).unwrap();

        // Every faction should have starting resources
        assert!(
            faction_data.starting_feedstock > 0,
            "Faction {:?} should have starting feedstock",
            faction_id
        );

        // Every faction should have at least a main base
        assert!(
            !faction_data.starting_buildings.is_empty(),
            "Faction {:?} should have starting buildings",
            faction_id
        );
    }
}
