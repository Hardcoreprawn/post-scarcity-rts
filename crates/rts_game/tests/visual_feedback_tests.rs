use bevy::prelude::*;

use rts_game::components::GameHealth;
use rts_game::render::{DamageFlash, DamageFlashPlugin};

#[test]
fn adds_damage_flash_on_health_drop() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(DamageFlashPlugin);

    let entity = app
        .world_mut()
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.2, 0.4, 0.8),
                ..default()
            },
            ..default()
        })
        .insert(GameHealth {
            current: 100,
            max: 100,
        })
        .id();

    app.update();

    {
        let mut health = app.world_mut().get_mut::<GameHealth>(entity).unwrap();
        health.current = 80;
    }

    app.update();

    assert!(app.world().get::<DamageFlash>(entity).is_some());
}

#[test]
fn clears_damage_flash_when_timer_expires() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(DamageFlashPlugin);

    let base_color = Color::srgb(0.2, 0.4, 0.8);
    let flash_color = base_color.lighter(0.6);

    let entity = app
        .world_mut()
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: flash_color,
                ..default()
            },
            ..default()
        })
        .insert(GameHealth {
            current: 100,
            max: 100,
        })
        .insert(DamageFlash {
            timer: 0.0,
            base_color,
        })
        .id();

    app.update();

    assert!(app.world().get::<DamageFlash>(entity).is_none());
    let sprite = app.world().get::<Sprite>(entity).unwrap();
    assert_eq!(sprite.color, base_color);
}
