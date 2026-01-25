//! Entity bundles for spawning game objects.
//!
//! These bundles combine multiple components for easy entity creation.

use bevy::prelude::*;
use rts_core::factions::FactionId;
use rts_core::math::{Fixed, Vec2Fixed};

use crate::components::{
    Armor, ArmorType, Building, BuildingType, Collider, CombatStats, DamageType, GameCommandQueue,
    GameDepot, GameFaction, GameHarvester, GameHealth, GamePosition, GameProductionQueue,
    GameResourceNode, Regeneration, Selectable, Stationary, UnderConstruction, UnitDataId,
};

/// Returns the faction color for rendering.
#[must_use]
pub fn faction_color(faction: FactionId) -> Color {
    match faction {
        FactionId::Continuity => Color::srgb(0.2, 0.4, 0.8), // Blue
        FactionId::Collegium => Color::srgb(0.8, 0.6, 0.2),  // Gold
        FactionId::Tinkers => Color::srgb(0.6, 0.4, 0.2),    // Brown
        FactionId::BioSovereigns => Color::srgb(0.2, 0.7, 0.3), // Green
        FactionId::Zephyr => Color::srgb(0.6, 0.8, 0.9),     // Sky blue
    }
}

/// Bundle for spawning a unit with all required game components.
///
/// This creates a visible unit with placeholder sprite rendering,
/// selection capability, and command queue.
#[derive(Bundle)]
pub struct UnitBundle {
    /// The sprite bundle for the unit.
    pub sprite: SpriteBundle,
    /// Whether this unit can be selected.
    pub selectable: Selectable,
    /// The faction this unit belongs to.
    pub faction: GameFaction,
    /// The command queue for this unit.
    pub command_queue: GameCommandQueue,
    /// The game position (synced to transform).
    pub position: GamePosition,
    /// The unit's health.
    pub health: GameHealth,
    /// Combat statistics for attacking.
    pub combat: CombatStats,
    /// Armor for damage reduction.
    pub armor: Armor,
    /// The unit type ID from RON data.
    pub unit_data_id: UnitDataId,
}

impl UnitBundle {
    /// Creates a new unit bundle with faction-colored sprite.
    ///
    /// **Deprecated:** Use `from_data` for data-driven unit spawning.
    #[must_use]
    pub fn new(position: Vec2, faction: FactionId, max_health: u32) -> Self {
        Self {
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: faction_color(faction),
                    custom_size: Some(Vec2::new(32.0, 32.0)),
                    ..default()
                },
                transform: Transform::from_translation(position.extend(0.0)),
                ..default()
            },
            selectable: Selectable,
            faction: GameFaction { faction },
            command_queue: GameCommandQueue::new(),
            position: GamePosition::new(Vec2Fixed::new(
                Fixed::from_num(position.x),
                Fixed::from_num(position.y),
            )),
            health: GameHealth::new(max_health),
            combat: CombatStats::new(15, DamageType::Kinetic, 50.0, 1.0),
            armor: Armor::new(ArmorType::Light),
            unit_data_id: UnitDataId::new("unknown"),
        }
    }

    /// Creates a unit bundle from RON `UnitData`.
    ///
    /// Uses the data-driven stats from faction configuration files.
    #[must_use]
    pub fn from_data(
        position: Vec2,
        faction: FactionId,
        unit_data: &rts_core::data::UnitData,
    ) -> Self {
        // Convert combat stats from RON data
        let (damage, range, cooldown, armor_value) = if let Some(combat) = &unit_data.combat {
            (
                combat.damage,
                combat.range.to_num::<f32>(),
                1.0 / (combat.attack_cooldown as f32 / 60.0), // Convert ticks to attacks/sec
                combat.armor,
            )
        } else {
            (0, 0.0, 0.0, 0) // Non-combat unit
        };

        // Determine armor type from tags
        let armor_type = if unit_data.has_tag("vehicle") || unit_data.has_tag("mech") {
            ArmorType::Heavy
        } else if unit_data.has_tag("structure") {
            ArmorType::Structure
        } else {
            ArmorType::Light
        };

        // Determine sprite size based on unit type for visual differentiation
        // Rangers are tall and thin, infantry are square, harvesters are larger
        let sprite_size = if unit_data.has_tag("ranged")
            || unit_data.id.contains("sniper")
            || unit_data.id.contains("ranger")
        {
            Vec2::new(20.0, 28.0) // Tall and thin for ranged units
        } else if unit_data.has_tag("worker") || unit_data.id.contains("harvester") {
            Vec2::new(36.0, 36.0) // Larger for harvesters (HarvesterBundle handles this separately)
        } else {
            Vec2::new(28.0, 28.0) // Slightly smaller square for infantry
        };

        Self {
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: faction_color(faction),
                    custom_size: Some(sprite_size),
                    ..default()
                },
                transform: Transform::from_translation(position.extend(0.0)),
                ..default()
            },
            selectable: Selectable,
            faction: GameFaction { faction },
            command_queue: GameCommandQueue::new(),
            position: GamePosition::new(Vec2Fixed::new(
                Fixed::from_num(position.x),
                Fixed::from_num(position.y),
            )),
            health: GameHealth::new(unit_data.health),
            combat: CombatStats::new(damage, DamageType::Kinetic, range, cooldown),
            armor: Armor::new_with_value(armor_type, armor_value),
            unit_data_id: UnitDataId::new(&unit_data.id),
        }
    }
}

/// Bundle for spawning a resource node.
#[derive(Bundle)]
pub struct ResourceNodeBundle {
    /// The sprite bundle for the node.
    pub sprite: SpriteBundle,
    /// The game position.
    pub position: GamePosition,
    /// Resource node data.
    pub node: GameResourceNode,
    /// Marker to exclude from unit separation.
    pub stationary: Stationary,
}

impl ResourceNodeBundle {
    /// Create a temporary (depletable) resource node.
    #[must_use]
    pub fn temporary(position: Vec2, remaining: i32) -> Self {
        Self {
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.9, 0.7, 0.2), // Golden color for temporary
                    custom_size: Some(Vec2::new(40.0, 40.0)),
                    ..default()
                },
                transform: Transform::from_translation(position.extend(-0.5)), // Behind units
                ..default()
            },
            position: GamePosition::new(Vec2Fixed::new(
                Fixed::from_num(position.x),
                Fixed::from_num(position.y),
            )),
            node: GameResourceNode::temporary(remaining, 10),
            stationary: Stationary,
        }
    }

    /// Create a permanent (infinite, degrading yield) resource node near a base.
    #[must_use]
    pub fn permanent(position: Vec2, optimal_harvesters: u8) -> Self {
        Self {
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.3, 0.8, 0.4), // Green tint for permanent
                    custom_size: Some(Vec2::new(56.0, 56.0)), // Larger than temporary
                    ..default()
                },
                transform: Transform::from_translation(position.extend(-0.5)),
                ..default()
            },
            position: GamePosition::new(Vec2Fixed::new(
                Fixed::from_num(position.x),
                Fixed::from_num(position.y),
            )),
            node: GameResourceNode::permanent(15, optimal_harvesters), // Higher base rate
            stationary: Stationary,
        }
    }
}

/// Bundle for spawning a depot building.
#[derive(Bundle)]
pub struct DepotBundle {
    /// The sprite bundle for the depot.
    pub sprite: SpriteBundle,
    /// Whether this depot can be selected.
    pub selectable: Selectable,
    /// The faction this depot belongs to.
    pub faction: GameFaction,
    /// The game position.
    pub position: GamePosition,
    /// The depot's health.
    pub health: GameHealth,
    /// Health regeneration.
    pub regen: Regeneration,
    /// Depot marker.
    pub depot: GameDepot,
    /// Production queue for building units.
    pub production: GameProductionQueue,
    /// Armor for damage reduction.
    pub armor: Armor,
    /// Collision shape.
    pub collider: Collider,
    /// Marker to exclude from unit separation.
    pub stationary: Stationary,
}

impl DepotBundle {
    /// Create a new depot at the given position.
    #[must_use]
    pub fn new(position: Vec2, faction: FactionId) -> Self {
        Self {
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: faction_color(faction).lighter(0.3),
                    custom_size: Some(Vec2::new(64.0, 64.0)),
                    ..default()
                },
                transform: Transform::from_translation(position.extend(-0.3)),
                ..default()
            },
            selectable: Selectable,
            faction: GameFaction { faction },
            position: GamePosition::new(Vec2Fixed::new(
                Fixed::from_num(position.x),
                Fixed::from_num(position.y),
            )),
            health: GameHealth::new(1500),  // Tough main building
            regen: Regeneration::new(10.0), // 10 HP/sec passive regen
            depot: GameDepot,
            production: GameProductionQueue::new(5), // Max 5 units in queue
            armor: Armor::new(ArmorType::Structure),
            collider: Collider::new(64.0, 64.0),
            stationary: Stationary,
        }
    }
}

/// Bundle for spawning a harvester unit.
#[derive(Bundle)]
pub struct HarvesterBundle {
    /// The sprite bundle for the harvester.
    pub sprite: SpriteBundle,
    /// Whether this harvester can be selected.
    pub selectable: Selectable,
    /// The faction this harvester belongs to.
    pub faction: GameFaction,
    /// The command queue for this harvester.
    pub command_queue: GameCommandQueue,
    /// The game position.
    pub position: GamePosition,
    /// The harvester's health.
    pub health: GameHealth,
    /// Harvester component.
    pub harvester: GameHarvester,
}

impl HarvesterBundle {
    /// Create a new harvester at the given position.
    #[must_use]
    pub fn new(position: Vec2, faction: FactionId) -> Self {
        Self {
            sprite: SpriteBundle {
                sprite: Sprite {
                    // Harvesters are darker with a cargo indicator
                    color: faction_color(faction).darker(0.2),
                    custom_size: Some(Vec2::new(36.0, 36.0)), // Slightly larger than combat units
                    ..default()
                },
                transform: Transform::from_translation(position.extend(0.0)),
                ..default()
            },
            selectable: Selectable,
            faction: GameFaction { faction },
            command_queue: GameCommandQueue::new(),
            position: GamePosition::new(Vec2Fixed::new(
                Fixed::from_num(position.x),
                Fixed::from_num(position.y),
            )),
            health: GameHealth::new(80),
            harvester: GameHarvester::new(100, 10), // 100 capacity, 10 per tick
        }
    }
}

/// Bundle for spawning a barracks building.
#[derive(Bundle)]
pub struct BarracksBundle {
    /// The sprite bundle.
    pub sprite: SpriteBundle,
    /// Selectable marker.
    pub selectable: Selectable,
    /// Faction ownership.
    pub faction: GameFaction,
    /// World position.
    pub position: GamePosition,
    /// Building health.
    pub health: GameHealth,
    /// Building type marker.
    pub building: Building,
    /// Production queue for units.
    pub production: GameProductionQueue,
    /// Under construction marker.
    pub under_construction: UnderConstruction,
    /// Armor for damage reduction.
    pub armor: Armor,
    /// Collision shape.
    pub collider: Collider,
    /// Stationary marker.
    pub stationary: Stationary,
}

impl BarracksBundle {
    /// Create a new barracks at the given position (starts under construction).
    #[must_use]
    pub fn new(position: Vec2, faction: FactionId) -> Self {
        Self {
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: faction_color(faction).lighter(0.2),
                    custom_size: Some(Vec2::new(48.0, 48.0)),
                    ..default()
                },
                transform: Transform::from_translation(position.extend(-0.3)),
                ..default()
            },
            selectable: Selectable,
            faction: GameFaction { faction },
            position: GamePosition::new(Vec2Fixed::new(
                Fixed::from_num(position.x),
                Fixed::from_num(position.y),
            )),
            health: GameHealth::new(350),
            building: Building::new(BuildingType::Barracks),
            production: GameProductionQueue::new(5),
            under_construction: UnderConstruction::new(BuildingType::Barracks),
            armor: Armor::new(ArmorType::Structure),
            collider: Collider::new(48.0, 48.0),
            stationary: Stationary,
        }
    }
}

/// Bundle for spawning a supply depot building.
#[derive(Bundle)]
pub struct SupplyDepotBundle {
    /// The sprite bundle.
    pub sprite: SpriteBundle,
    /// Selectable marker.
    pub selectable: Selectable,
    /// Faction ownership.
    pub faction: GameFaction,
    /// World position.
    pub position: GamePosition,
    /// Building health.
    pub health: GameHealth,
    /// Building type marker.
    pub building: Building,
    /// Under construction marker.
    pub under_construction: UnderConstruction,
    /// Armor for damage reduction.
    pub armor: Armor,
    /// Collision shape.
    pub collider: Collider,
    /// Stationary marker.
    pub stationary: Stationary,
}

impl SupplyDepotBundle {
    /// Create a new supply depot (starts under construction).
    #[must_use]
    pub fn new(position: Vec2, faction: FactionId) -> Self {
        Self {
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: faction_color(faction).lighter(0.1),
                    custom_size: Some(Vec2::new(32.0, 32.0)),
                    ..default()
                },
                transform: Transform::from_translation(position.extend(-0.3)),
                ..default()
            },
            selectable: Selectable,
            faction: GameFaction { faction },
            position: GamePosition::new(Vec2Fixed::new(
                Fixed::from_num(position.x),
                Fixed::from_num(position.y),
            )),
            health: GameHealth::new(200),
            building: Building::new(BuildingType::SupplyDepot),
            under_construction: UnderConstruction::new(BuildingType::SupplyDepot),
            armor: Armor::new(ArmorType::Structure),
            collider: Collider::new(32.0, 32.0),
            stationary: Stationary,
        }
    }
}

/// Bundle for spawning a tech lab building.
#[derive(Bundle)]
pub struct TechLabBundle {
    /// The sprite bundle.
    pub sprite: SpriteBundle,
    /// Selectable marker.
    pub selectable: Selectable,
    /// Faction ownership.
    pub faction: GameFaction,
    /// World position.
    pub position: GamePosition,
    /// Building health.
    pub health: GameHealth,
    /// Building type marker.
    pub building: Building,
    /// Under construction marker.
    pub under_construction: UnderConstruction,
    /// Armor for damage reduction.
    pub armor: Armor,
    /// Collision shape.
    pub collider: Collider,
    /// Stationary marker.
    pub stationary: Stationary,
}

impl TechLabBundle {
    /// Create a new tech lab (starts under construction).
    #[must_use]
    pub fn new(position: Vec2, faction: FactionId) -> Self {
        Self {
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.6, 0.3, 0.8), // Purple for tech
                    custom_size: Some(Vec2::new(40.0, 40.0)),
                    ..default()
                },
                transform: Transform::from_translation(position.extend(-0.3)),
                ..default()
            },
            selectable: Selectable,
            faction: GameFaction { faction },
            position: GamePosition::new(Vec2Fixed::new(
                Fixed::from_num(position.x),
                Fixed::from_num(position.y),
            )),
            health: GameHealth::new(250),
            building: Building::new(BuildingType::TechLab),
            under_construction: UnderConstruction::new(BuildingType::TechLab),
            armor: Armor::new(ArmorType::Structure),
            collider: Collider::new(40.0, 40.0),
            stationary: Stationary,
        }
    }
}

/// Bundle for spawning a turret building.
#[derive(Bundle)]
pub struct TurretBundle {
    /// The sprite bundle.
    pub sprite: SpriteBundle,
    /// Selectable marker.
    pub selectable: Selectable,
    /// Faction ownership.
    pub faction: GameFaction,
    /// World position.
    pub position: GamePosition,
    /// Building health.
    pub health: GameHealth,
    /// Building type marker.
    pub building: Building,
    /// Combat stats for attacking.
    pub combat: CombatStats,
    /// Under construction marker.
    pub under_construction: UnderConstruction,
    /// Armor for damage reduction.
    pub armor: Armor,
    /// Collision shape.
    pub collider: Collider,
    /// Stationary marker.
    pub stationary: Stationary,
}

impl TurretBundle {
    /// Create a new turret (starts under construction).
    #[must_use]
    pub fn new(position: Vec2, faction: FactionId) -> Self {
        Self {
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: faction_color(faction).darker(0.1),
                    custom_size: Some(Vec2::new(24.0, 24.0)),
                    ..default()
                },
                transform: Transform::from_translation(position.extend(-0.2)),
                ..default()
            },
            selectable: Selectable,
            faction: GameFaction { faction },
            position: GamePosition::new(Vec2Fixed::new(
                Fixed::from_num(position.x),
                Fixed::from_num(position.y),
            )),
            health: GameHealth::new(150),
            building: Building::new(BuildingType::Turret),
            combat: CombatStats::new(20, DamageType::Kinetic, 200.0, 0.8), // High damage, long range
            under_construction: UnderConstruction::new(BuildingType::Turret),
            armor: Armor::new(ArmorType::Structure),
            collider: Collider::new(24.0, 24.0),
            stationary: Stationary,
        }
    }
}
