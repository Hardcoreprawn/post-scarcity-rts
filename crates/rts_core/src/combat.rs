//! Resistance-based combat system with percentage-based damage reduction.
//!
//! This module implements a combat system with:
//! - Percentage-based damage reduction (not flat armor)
//! - Armor penetration counters resistance
//! - Resistance cap at 75% prevents invulnerability
//! - Size class tracking modifiers
//! - Damage type effectiveness matrix

use serde::{Deserialize, Serialize};

use crate::components::{ArmorType, DamageType};
use crate::math::Fixed;

/// Weapon size class affects tracking vs target size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WeaponSize {
    /// Small weapons - track fast, low damage, poor vs heavy armor.
    Light,
    /// Medium weapons - balanced tracking and damage.
    #[default]
    Medium,
    /// Heavy weapons - slow tracking, high damage, poor vs light targets.
    Heavy,
}

impl WeaponSize {
    /// Get the damage modifier when this weapon size attacks a target armor class.
    ///
    /// Heavy weapons deal reduced damage to Light targets (tracking penalty).
    /// Light weapons deal reduced damage to Heavy armor (penetration penalty).
    #[must_use]
    pub fn tracking_modifier_vs(self, target_armor: ArmorClass) -> Fixed {
        let percent = match (self, target_armor) {
            // Light weapons - fast tracking, weak vs heavy
            (WeaponSize::Light, ArmorClass::Light) => 100,
            (WeaponSize::Light, ArmorClass::Medium) => 75,
            (WeaponSize::Light, ArmorClass::Heavy) => 50,
            (WeaponSize::Light, ArmorClass::Air) => 100,
            (WeaponSize::Light, ArmorClass::Building) => 25,

            // Medium weapons - versatile
            (WeaponSize::Medium, ArmorClass::Light) => 75,
            (WeaponSize::Medium, ArmorClass::Medium) => 100,
            (WeaponSize::Medium, ArmorClass::Heavy) => 100,
            (WeaponSize::Medium, ArmorClass::Air) => 75,
            (WeaponSize::Medium, ArmorClass::Building) => 75,

            // Heavy weapons - slow tracking, can't hit light targets
            (WeaponSize::Heavy, ArmorClass::Light) => 25,
            (WeaponSize::Heavy, ArmorClass::Medium) => 75,
            (WeaponSize::Heavy, ArmorClass::Heavy) => 100,
            (WeaponSize::Heavy, ArmorClass::Air) => 25,
            (WeaponSize::Heavy, ArmorClass::Building) => 150,
        };

        Fixed::from_num(percent) / Fixed::from_num(100)
    }
}

/// Armor class for targets (distinct from damage type interaction).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ArmorClass {
    /// Light armor - infantry, scouts.
    #[default]
    Light,
    /// Medium armor - vehicles, standard units.
    Medium,
    /// Heavy armor - tanks, mechs, heavy units.
    Heavy,
    /// Air units - aircraft, drones.
    Air,
    /// Buildings and structures.
    Building,
}

impl ArmorClass {
    /// Convert from the legacy ArmorType enum.
    #[must_use]
    pub const fn from_armor_type(armor_type: ArmorType) -> Self {
        match armor_type {
            ArmorType::Unarmored => ArmorClass::Light,
            ArmorType::Light => ArmorClass::Light,
            ArmorType::Heavy => ArmorClass::Heavy,
            ArmorType::Building => ArmorClass::Building,
        }
    }

    /// Get the base resistance range for this armor class.
    #[must_use]
    pub const fn base_resistance_range(self) -> (u8, u8) {
        match self {
            ArmorClass::Light => (10, 20),
            ArmorClass::Medium => (25, 40),
            ArmorClass::Heavy => (45, 60),
            ArmorClass::Air => (15, 25),
            ArmorClass::Building => (30, 50),
        }
    }

    /// Get the resistance cap for this armor class.
    #[must_use]
    pub const fn resistance_cap(self) -> u8 {
        match self {
            ArmorClass::Light => 50,
            ArmorClass::Medium => 65,
            ArmorClass::Heavy => 75,
            ArmorClass::Air => 50,
            ArmorClass::Building => 75,
        }
    }
}

/// Extended damage type for resistance-based combat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ExtendedDamageType {
    /// Kinetic damage - bullets, shells. Good vs light, poor vs heavy.
    #[default]
    Kinetic,
    /// Explosive damage - missiles, bombs. Good vs heavy and buildings.
    Explosive,
    /// Energy damage - lasers, plasma. Consistent damage, ignores some resistance.
    Energy,
    /// Bio-Acid damage - Sculptors faction. Strong vs light, useless vs buildings.
    BioAcid,
    /// Fire damage - incendiary weapons. Strong vs bio, poor vs mechanical.
    Fire,
}

impl ExtendedDamageType {
    /// Get the damage modifier for this damage type vs an armor class.
    #[must_use]
    pub fn effectiveness_vs(self, armor_class: ArmorClass) -> Fixed {
        let percent = match (self, armor_class) {
            // Kinetic: good vs light and air, poor vs heavy and buildings
            (Self::Kinetic, ArmorClass::Light) => 100,
            (Self::Kinetic, ArmorClass::Medium) => 75,
            (Self::Kinetic, ArmorClass::Heavy) => 50,
            (Self::Kinetic, ArmorClass::Air) => 75,
            (Self::Kinetic, ArmorClass::Building) => 50,

            // Explosive: weak vs air and spread light units, strong vs heavy and buildings
            (Self::Explosive, ArmorClass::Light) => 75,
            (Self::Explosive, ArmorClass::Medium) => 100,
            (Self::Explosive, ArmorClass::Heavy) => 125,
            (Self::Explosive, ArmorClass::Air) => 50,
            (Self::Explosive, ArmorClass::Building) => 150,

            // Energy: consistent damage across all targets
            (Self::Energy, ArmorClass::Light) => 100,
            (Self::Energy, ArmorClass::Medium) => 100,
            (Self::Energy, ArmorClass::Heavy) => 100,
            (Self::Energy, ArmorClass::Air) => 100,
            (Self::Energy, ArmorClass::Building) => 75,

            // Bio-Acid: devastating vs light, useless vs buildings
            (Self::BioAcid, ArmorClass::Light) => 125,
            (Self::BioAcid, ArmorClass::Medium) => 100,
            (Self::BioAcid, ArmorClass::Heavy) => 75,
            (Self::BioAcid, ArmorClass::Air) => 100,
            (Self::BioAcid, ArmorClass::Building) => 0,

            // Fire: excellent vs light/bio, poor vs heavy/mechanical
            (Self::Fire, ArmorClass::Light) => 125,
            (Self::Fire, ArmorClass::Medium) => 100,
            (Self::Fire, ArmorClass::Heavy) => 75,
            (Self::Fire, ArmorClass::Air) => 100,
            (Self::Fire, ArmorClass::Building) => 125,
        };

        Fixed::from_num(percent) / Fixed::from_num(100)
    }

    /// Convert from legacy DamageType.
    #[must_use]
    pub const fn from_damage_type(damage_type: DamageType) -> Self {
        match damage_type {
            DamageType::Kinetic => Self::Kinetic,
            DamageType::Energy => Self::Energy,
            DamageType::Explosive => Self::Explosive,
            DamageType::Biological => Self::BioAcid,
        }
    }
}

/// Maximum resistance cap (75% damage reduction).
pub const MAX_RESISTANCE: u8 = 75;

/// Minimum damage floor - attacks always deal at least 1 damage (unless immune).
pub const MIN_DAMAGE: u32 = 1;

/// Combat stats for resistance-based damage calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResistanceStats {
    /// Armor class of this unit.
    pub armor_class: ArmorClass,
    /// Base resistance percentage (0-75).
    pub resistance: u8,
    /// Additional resistance from buffs/abilities.
    pub bonus_resistance: u8,
}

impl ResistanceStats {
    /// Create new resistance stats.
    #[must_use]
    pub const fn new(armor_class: ArmorClass, resistance: u8) -> Self {
        Self {
            armor_class,
            resistance,
            bonus_resistance: 0,
        }
    }

    /// Get effective resistance (capped).
    #[must_use]
    pub fn effective_resistance(&self) -> u8 {
        let total = self.resistance.saturating_add(self.bonus_resistance);
        total
            .min(self.armor_class.resistance_cap())
            .min(MAX_RESISTANCE)
    }

    /// Get resistance as a fixed-point fraction (0.0 to 0.75).
    #[must_use]
    pub fn resistance_fraction(&self) -> Fixed {
        Fixed::from_num(self.effective_resistance()) / Fixed::from_num(100)
    }
}

impl Default for ResistanceStats {
    fn default() -> Self {
        Self::new(ArmorClass::Light, 0)
    }
}

/// Weapon stats for armor penetration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeaponStats {
    /// Base damage of the weapon.
    pub damage: u32,
    /// Type of damage dealt.
    pub damage_type: ExtendedDamageType,
    /// Size class of the weapon.
    pub weapon_size: WeaponSize,
    /// Armor penetration percentage (0-100).
    pub armor_penetration: u8,
}

impl WeaponStats {
    /// Create new weapon stats.
    #[must_use]
    pub const fn new(damage: u32, damage_type: ExtendedDamageType) -> Self {
        Self {
            damage,
            damage_type,
            weapon_size: WeaponSize::Medium,
            armor_penetration: 0,
        }
    }

    /// Builder method to set weapon size.
    #[must_use]
    pub const fn with_size(mut self, size: WeaponSize) -> Self {
        self.weapon_size = size;
        self
    }

    /// Builder method to set armor penetration.
    #[must_use]
    pub const fn with_penetration(mut self, penetration: u8) -> Self {
        self.armor_penetration = if penetration > 100 { 100 } else { penetration };
        self
    }

    /// Get armor penetration as a fixed-point fraction.
    #[must_use]
    pub fn penetration_fraction(&self) -> Fixed {
        Fixed::from_num(self.armor_penetration) / Fixed::from_num(100)
    }
}

impl Default for WeaponStats {
    fn default() -> Self {
        Self::new(10, ExtendedDamageType::Kinetic)
    }
}

/// Calculate damage using resistance-based formula.
///
/// Formula:
/// ```text
/// Effective Resistance = Resistance × (1 - Armor Penetration)
/// Damage Reduction = Effective Resistance (capped at 75%)
/// Final Damage = Base Damage × Damage Type Modifier × Size Modifier × (1 - Damage Reduction)
/// Minimum Damage = 1 (unless immune)
/// ```
///
/// # Arguments
/// * `weapon` - Attacker's weapon stats
/// * `target` - Target's resistance stats
///
/// # Returns
/// Final damage to apply (minimum 1 unless immune).
#[must_use]
pub fn calculate_resistance_damage(weapon: &WeaponStats, target: &ResistanceStats) -> u32 {
    // Step 1: Get damage type effectiveness modifier
    let type_modifier = weapon.damage_type.effectiveness_vs(target.armor_class);

    // If damage type is immune (0%), no damage
    if type_modifier == Fixed::ZERO {
        return 0;
    }

    // Step 2: Get weapon size tracking modifier
    let size_modifier = weapon.weapon_size.tracking_modifier_vs(target.armor_class);

    // Step 3: Calculate effective resistance after penetration
    let base_resistance = target.resistance_fraction();
    let penetration = weapon.penetration_fraction();
    let effective_resistance = base_resistance * (Fixed::ONE - penetration);

    // Step 4: Cap resistance at MAX_RESISTANCE
    let capped_resistance =
        effective_resistance.min(Fixed::from_num(MAX_RESISTANCE) / Fixed::from_num(100));

    // Step 5: Calculate damage reduction (1 - resistance)
    let damage_multiplier = Fixed::ONE - capped_resistance;

    // Step 6: Apply all modifiers
    let base_damage = Fixed::from_num(weapon.damage);
    let final_damage = base_damage * type_modifier * size_modifier * damage_multiplier;

    // Step 7: Convert to u32 with minimum damage floor
    let damage_int: u32 = final_damage.to_num::<i32>().max(0) as u32;
    if damage_int == 0 && type_modifier > Fixed::ZERO {
        MIN_DAMAGE
    } else {
        damage_int
    }
}

/// Convert flat armor value to percentage resistance.
///
/// Uses a logarithmic conversion to map old flat armor values to percentages:
/// - 0 armor = 0% resistance
/// - 5 armor ≈ 20% resistance
/// - 10 armor ≈ 35% resistance
/// - 15 armor ≈ 45% resistance
/// - 20+ armor ≈ 55%+ resistance (capped by armor class)
#[must_use]
pub fn convert_flat_armor_to_resistance(flat_armor: u32, armor_class: ArmorClass) -> u8 {
    if flat_armor == 0 {
        return 0;
    }

    // Logarithmic conversion: resistance = 20 * ln(1 + armor/3)
    let armor_f = flat_armor as f64;
    let resistance_f = 20.0 * (1.0 + armor_f / 3.0).ln();
    let resistance = resistance_f.round() as u8;

    // Cap by armor class
    resistance.min(armor_class.resistance_cap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weapon_size_tracking() {
        // Light weapons are effective vs light
        let light_vs_light = WeaponSize::Light.tracking_modifier_vs(ArmorClass::Light);
        assert_eq!(light_vs_light, Fixed::ONE);

        // Light weapons are poor vs heavy
        let light_vs_heavy = WeaponSize::Light.tracking_modifier_vs(ArmorClass::Heavy);
        assert_eq!(light_vs_heavy, Fixed::from_num(50) / Fixed::from_num(100));

        // Heavy weapons are poor vs light (tracking penalty)
        let heavy_vs_light = WeaponSize::Heavy.tracking_modifier_vs(ArmorClass::Light);
        assert_eq!(heavy_vs_light, Fixed::from_num(25) / Fixed::from_num(100));

        // Heavy weapons are good vs buildings
        let heavy_vs_building = WeaponSize::Heavy.tracking_modifier_vs(ArmorClass::Building);
        assert_eq!(
            heavy_vs_building,
            Fixed::from_num(150) / Fixed::from_num(100)
        );
    }

    #[test]
    fn test_damage_type_effectiveness() {
        // Kinetic is full damage vs light
        let kinetic_vs_light = ExtendedDamageType::Kinetic.effectiveness_vs(ArmorClass::Light);
        assert_eq!(kinetic_vs_light, Fixed::ONE);

        // Explosive is strong vs buildings
        let explosive_vs_building =
            ExtendedDamageType::Explosive.effectiveness_vs(ArmorClass::Building);
        assert_eq!(
            explosive_vs_building,
            Fixed::from_num(150) / Fixed::from_num(100)
        );

        // Bio-Acid is immune vs buildings
        let bio_vs_building = ExtendedDamageType::BioAcid.effectiveness_vs(ArmorClass::Building);
        assert_eq!(bio_vs_building, Fixed::ZERO);
    }

    #[test]
    fn test_resistance_stats() {
        let stats = ResistanceStats::new(ArmorClass::Heavy, 50);
        assert_eq!(stats.effective_resistance(), 50);
        assert_eq!(
            stats.resistance_fraction(),
            Fixed::from_num(50) / Fixed::from_num(100)
        );

        // Test capping
        let over_cap = ResistanceStats::new(ArmorClass::Light, 80);
        assert_eq!(over_cap.effective_resistance(), 50); // Light cap is 50%
    }

    #[test]
    fn test_calculate_damage_no_resistance() {
        let weapon = WeaponStats::new(100, ExtendedDamageType::Energy);
        let target = ResistanceStats::new(ArmorClass::Medium, 0);

        let damage = calculate_resistance_damage(&weapon, &target);
        assert_eq!(damage, 100); // Full damage with energy vs medium and size medium
    }

    #[test]
    fn test_calculate_damage_with_resistance() {
        let weapon = WeaponStats::new(100, ExtendedDamageType::Energy);
        let target = ResistanceStats::new(ArmorClass::Medium, 50);

        let damage = calculate_resistance_damage(&weapon, &target);
        assert_eq!(damage, 50); // 50% reduction
    }

    #[test]
    fn test_calculate_damage_with_penetration() {
        let weapon = WeaponStats::new(100, ExtendedDamageType::Energy).with_penetration(50);
        let target = ResistanceStats::new(ArmorClass::Medium, 50);

        // 50% penetration reduces 50% resistance to 25% effective
        // 100 * 1.0 * 1.0 * (1 - 0.25) = 75
        let damage = calculate_resistance_damage(&weapon, &target);
        assert_eq!(damage, 75);
    }

    #[test]
    fn test_calculate_damage_type_modifier() {
        let weapon = WeaponStats::new(100, ExtendedDamageType::Explosive);
        let target = ResistanceStats::new(ArmorClass::Building, 0);

        // Explosive does 150% vs buildings
        // 100 * 1.5 * 1.5 (heavy vs building) * 1.0 = 225
        let weapon_heavy = weapon.with_size(WeaponSize::Heavy);
        let damage = calculate_resistance_damage(&weapon_heavy, &target);
        assert_eq!(damage, 225);
    }

    #[test]
    fn test_calculate_damage_size_penalty() {
        let weapon =
            WeaponStats::new(100, ExtendedDamageType::Kinetic).with_size(WeaponSize::Heavy);
        let target = ResistanceStats::new(ArmorClass::Light, 0);

        // Heavy weapon has 25% tracking vs light
        // 100 * 1.0 (kinetic vs light) * 0.25 (heavy vs light) * 1.0 = 25
        let damage = calculate_resistance_damage(&weapon, &target);
        assert_eq!(damage, 25);
    }

    #[test]
    fn test_calculate_damage_immunity() {
        let weapon = WeaponStats::new(100, ExtendedDamageType::BioAcid);
        let target = ResistanceStats::new(ArmorClass::Building, 0);

        // Bio-Acid is immune vs buildings
        let damage = calculate_resistance_damage(&weapon, &target);
        assert_eq!(damage, 0);
    }

    #[test]
    fn test_calculate_damage_minimum() {
        let weapon = WeaponStats::new(1, ExtendedDamageType::Kinetic).with_size(WeaponSize::Light);
        let target = ResistanceStats::new(ArmorClass::Heavy, 75);

        // Very low damage should still deal minimum 1
        let damage = calculate_resistance_damage(&weapon, &target);
        assert_eq!(damage, MIN_DAMAGE);
    }

    #[test]
    fn test_resistance_cap() {
        let weapon = WeaponStats::new(100, ExtendedDamageType::Energy);

        // Max 75% cap means at least 25% damage gets through
        let max_resist = ResistanceStats::new(ArmorClass::Heavy, 100);
        let damage = calculate_resistance_damage(&weapon, &max_resist);

        // 100 * 1.0 (energy vs heavy) * 1.0 (medium vs heavy) * 0.25 = 25
        assert_eq!(damage, 25);
    }

    #[test]
    fn test_convert_flat_armor() {
        // Test conversion from flat armor to resistance
        assert_eq!(convert_flat_armor_to_resistance(0, ArmorClass::Heavy), 0);
        assert!(convert_flat_armor_to_resistance(5, ArmorClass::Heavy) >= 15);
        assert!(convert_flat_armor_to_resistance(10, ArmorClass::Heavy) >= 25);
        assert!(convert_flat_armor_to_resistance(20, ArmorClass::Heavy) >= 40);

        // Test capping by armor class
        let high_armor = convert_flat_armor_to_resistance(50, ArmorClass::Light);
        assert!(high_armor <= 50); // Light cap is 50%
    }

    #[test]
    fn test_determinism() {
        let weapon = WeaponStats::new(77, ExtendedDamageType::Explosive)
            .with_size(WeaponSize::Heavy)
            .with_penetration(33);
        let target = ResistanceStats::new(ArmorClass::Heavy, 45);

        // Same inputs must always produce same outputs
        for _ in 0..100 {
            let dmg1 = calculate_resistance_damage(&weapon, &target);
            let dmg2 = calculate_resistance_damage(&weapon, &target);
            assert_eq!(dmg1, dmg2);
        }
    }
}
