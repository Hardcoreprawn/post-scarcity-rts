use bevy::prelude::*;

use rts_game::components::{CombatStats, DamageType, Selected};
use rts_game::render::{RangeIndicator, RangeIndicatorPlugin};

#[test]
fn adds_range_indicator_for_selected_units() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RangeIndicatorPlugin);

    let entity = app
        .world_mut()
        .spawn(CombatStats::new(10, DamageType::Kinetic, 75.0, 1.0))
        .insert(Selected)
        .id();

    app.update();

    let indicator = app.world().get::<RangeIndicator>(entity).unwrap();
    assert_eq!(indicator.radius, 75.0);
}

#[test]
fn removes_range_indicator_when_deselected() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RangeIndicatorPlugin);

    let entity = app
        .world_mut()
        .spawn(CombatStats::new(10, DamageType::Kinetic, 60.0, 1.0))
        .insert(Selected)
        .id();

    app.update();

    app.world_mut().entity_mut(entity).remove::<Selected>();
    app.update();

    assert!(app.world().get::<RangeIndicator>(entity).is_none());
}
