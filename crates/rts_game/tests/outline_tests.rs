use bevy::prelude::*;

use rts_core::factions::FactionId;
use rts_game::bundles::faction_color;
use rts_game::components::{GameFaction, Selectable};
use rts_game::render::{OutlinePlugin, UnitOutline};

#[test]
fn adds_outline_for_selectable_units() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(OutlinePlugin);

    let entity = app
        .world_mut()
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(40.0, 20.0)),
                ..default()
            },
            ..default()
        })
        .insert(Selectable)
        .insert(GameFaction {
            faction: FactionId::Continuity,
        })
        .id();

    app.update();

    let outline = app.world().get::<UnitOutline>(entity).unwrap();
    assert_eq!(outline.radius, 22.0);
    assert_eq!(outline.color, faction_color(FactionId::Continuity));
}

#[test]
fn removes_outline_when_not_selectable() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(OutlinePlugin);

    let entity = app
        .world_mut()
        .spawn(SpriteBundle::default())
        .insert(Selectable)
        .insert(GameFaction {
            faction: FactionId::Continuity,
        })
        .id();

    app.update();

    app.world_mut().entity_mut(entity).remove::<Selectable>();
    app.update();

    assert!(app.world().get::<UnitOutline>(entity).is_none());
}
