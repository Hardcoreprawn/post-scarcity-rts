use bevy::prelude::Color;

use rts_game::combat::weapon_fire_color;
use rts_game::components::DamageType;

#[test]
fn weapon_fire_color_is_distinct_by_damage_type() {
    let kinetic = weapon_fire_color(DamageType::Kinetic);
    let energy = weapon_fire_color(DamageType::Energy);
    let explosive = weapon_fire_color(DamageType::Explosive);

    assert_ne!(kinetic, energy);
    assert_ne!(kinetic, explosive);
    assert_ne!(energy, explosive);
}

#[test]
fn weapon_fire_color_matches_expected_palette() {
    assert_eq!(
        weapon_fire_color(DamageType::Kinetic),
        Color::srgb(1.0, 0.8, 0.0)
    );
    assert_eq!(
        weapon_fire_color(DamageType::Energy),
        Color::srgb(0.3, 0.8, 1.0)
    );
    assert_eq!(
        weapon_fire_color(DamageType::Explosive),
        Color::srgb(1.0, 0.4, 0.2)
    );
}
