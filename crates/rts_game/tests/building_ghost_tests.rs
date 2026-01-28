use bevy::prelude::*;

use rts_game::components::BuildingType;
use rts_game::construction::{BuildingGhost, BuildingPlacement, PlacementGhostPlugin};

#[test]
fn spawns_ghost_when_placing_building() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(BuildingPlacement {
        placing: Some(BuildingType::Barracks),
        valid_placement: true,
        preview_position: Some(Vec2::new(10.0, 20.0)),
    });
    app.add_plugins(PlacementGhostPlugin);

    app.update();

    let world = app.world_mut();
    let mut ghosts = world.query::<(&BuildingGhost, &Sprite, &Transform)>();
    let (ghost, sprite, transform) = ghosts.single(world);

    assert_eq!(ghost.building_type, BuildingType::Barracks);
    assert_eq!(transform.translation.truncate(), Vec2::new(10.0, 20.0));
    assert_eq!(sprite.color, Color::srgba(0.0, 1.0, 0.0, 0.4));
}

#[test]
fn updates_ghost_color_when_invalid() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(BuildingPlacement {
        placing: Some(BuildingType::TechLab),
        valid_placement: false,
        preview_position: Some(Vec2::new(-5.0, 5.0)),
    });
    app.add_plugins(PlacementGhostPlugin);

    app.update();

    let world = app.world_mut();
    let mut ghosts = world.query::<(&BuildingGhost, &Sprite)>();
    let (_ghost, sprite) = ghosts.single(world);
    assert_eq!(sprite.color, Color::srgba(1.0, 0.0, 0.0, 0.4));
}

#[test]
fn despawns_ghost_when_not_placing() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(BuildingPlacement {
        placing: Some(BuildingType::SupplyDepot),
        valid_placement: true,
        preview_position: Some(Vec2::new(0.0, 0.0)),
    });
    app.add_plugins(PlacementGhostPlugin);

    app.update();

    {
        let mut placement = app.world_mut().resource_mut::<BuildingPlacement>();
        placement.placing = None;
        placement.preview_position = None;
    }

    app.update();

    let world = app.world_mut();
    let mut ghosts = world.query::<&BuildingGhost>();
    assert_eq!(ghosts.iter(world).count(), 0);
}
