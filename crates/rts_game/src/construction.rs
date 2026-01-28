//! Building construction system.
//!
//! Handles building placement, construction progress, and completion.

use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::bundles::{BarracksBundle, SupplyDepotBundle, TechLabBundle, TurretBundle};
use crate::camera::MainCamera;
use crate::components::{
    Building, BuildingType, FactionBuildings, GameFaction, GameHealth, PlayerFaction,
    UnderConstruction,
};
use crate::economy::PlayerResources;

/// Plugin for building construction.
pub struct ConstructionPlugin;

/// Plugin for placement ghost visuals only (no gizmo dependencies).
pub struct PlacementGhostPlugin;

impl Plugin for ConstructionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FactionBuildings>()
            .init_resource::<BuildingPlacement>()
            .init_resource::<GhostEntity>()
            .add_systems(Update, construction_progress)
            .add_systems(Update, complete_construction.after(construction_progress))
            .add_systems(Update, track_buildings)
            .add_systems(Update, update_placement_preview)
            .add_systems(
                Update,
                handle_placement_input.after(update_placement_preview),
            )
            .add_systems(
                Update,
                update_placement_ghost_sprite.after(update_placement_preview),
            )
            .add_systems(Update, building_hotkeys);
    }
}

impl Plugin for PlacementGhostPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GhostEntity>()
            .add_systems(Update, update_placement_ghost_sprite);
    }
}

/// Resource tracking building placement mode.
#[derive(Resource, Default)]
pub struct BuildingPlacement {
    /// Currently selected building type to place (if any).
    pub placing: Option<BuildingType>,
    /// Whether placement is valid at current cursor position.
    pub valid_placement: bool,
    /// Preview position.
    pub preview_position: Option<Vec2>,
}

/// Tracks the entity used for placement ghost visuals.
#[derive(Resource, Default)]
pub struct GhostEntity {
    /// The current ghost entity, if any.
    pub entity: Option<Entity>,
}

/// Marker component for building placement ghost sprite.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildingGhost {
    /// The building type being previewed.
    pub building_type: BuildingType,
}

/// Advances construction progress on buildings.
fn construction_progress(
    time: Res<Time>,
    mut query: Query<(&mut UnderConstruction, &mut Sprite, &mut GameHealth)>,
) {
    let dt = time.delta_seconds();

    for (mut construction, mut sprite, mut health) in query.iter_mut() {
        construction.advance(dt);

        // Visual feedback: buildings under construction are darker
        let progress = construction.progress;
        sprite.color = sprite.color.with_alpha(0.5 + progress * 0.5);

        // Health scales with construction progress
        let max_health = health.max;
        health.current = (max_health as f32 * progress) as u32;
    }
}

/// Completes construction when progress reaches 100%.
fn complete_construction(
    mut commands: Commands,
    query: Query<(Entity, &UnderConstruction, &GameFaction), With<UnderConstruction>>,
    mut faction_buildings: ResMut<FactionBuildings>,
    mut resources: ResMut<PlayerResources>,
) {
    for (entity, construction, faction) in query.iter() {
        if construction.is_complete() {
            // Remove the under-construction marker
            commands.entity(entity).remove::<UnderConstruction>();

            // Add to faction's building list
            faction_buildings.add_building(faction.faction, construction.building_type);

            // Add supply from building
            let supply_provided = construction.building_type.supply_provided();
            if supply_provided > 0 {
                resources.supply_cap += supply_provided;
            }

            // Increase feedstock storage capacity (all buildings add storage)
            resources.feedstock_cap += 500;

            tracing::info!(
                "Completed {:?} for {:?}",
                construction.building_type,
                faction.faction
            );
        }
    }
}

/// Tracks building destruction and updates faction building lists.
fn track_buildings(
    mut commands: Commands,
    query: Query<(Entity, &Building, &GameFaction, &GameHealth), Without<UnderConstruction>>,
    mut faction_buildings: ResMut<FactionBuildings>,
    mut resources: ResMut<PlayerResources>,
) {
    for (entity, building, faction, health) in query.iter() {
        if health.current == 0 {
            // Building destroyed
            faction_buildings.remove_building(faction.faction, building.building_type);

            // Remove supply from building
            let supply_provided = building.building_type.supply_provided();
            if supply_provided > 0 {
                resources.supply_cap = (resources.supply_cap - supply_provided).max(0);
            }

            // Reduce feedstock storage capacity
            resources.feedstock_cap = (resources.feedstock_cap - 500).max(500);

            commands.entity(entity).despawn_recursive();

            tracing::info!(
                "Destroyed {:?} for {:?}",
                building.building_type,
                faction.faction
            );
        }
    }
}

/// Spawn a building at the given position (starts under construction).
pub fn spawn_building(
    commands: &mut Commands,
    building_type: BuildingType,
    position: Vec2,
    faction: rts_core::factions::FactionId,
) -> Entity {
    match building_type {
        BuildingType::Depot => {
            // Depot uses existing DepotBundle - already instant
            // For a proper implementation, we'd have a DepotUnderConstruction variant
            commands
                .spawn(crate::bundles::DepotBundle::new(position, faction))
                .id()
        }
        BuildingType::Barracks => commands.spawn(BarracksBundle::new(position, faction)).id(),
        BuildingType::SupplyDepot => commands
            .spawn(SupplyDepotBundle::new(position, faction))
            .id(),
        BuildingType::TechLab => commands.spawn(TechLabBundle::new(position, faction)).id(),
        BuildingType::Turret => commands.spawn(TurretBundle::new(position, faction)).id(),
    }
}

/// Updates the placement preview position based on mouse cursor.
fn update_placement_preview(
    mut placement: ResMut<BuildingPlacement>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    if placement.placing.is_none() {
        placement.preview_position = None;
        return;
    }

    let Ok(window) = windows.get_single() else {
        tracing::warn!("Placement: No single window found");
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        tracing::warn!("Placement: No camera found");
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        // Cursor not in window - this is fine, just don't update
        return;
    };

    if let Some(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
        placement.preview_position = Some(world_position);
        placement.valid_placement = true;
    } else {
        tracing::warn!("Placement: viewport_to_world_2d failed");
    }
}

/// Handles mouse input for building placement.
fn handle_placement_input(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut placement: ResMut<BuildingPlacement>,
    player_faction: Res<PlayerFaction>,
    mut resources: ResMut<PlayerResources>,
    mut egui_contexts: EguiContexts,
) {
    // Don't process clicks if egui is using the pointer (e.g., clicking UI)
    if let Some(ctx) = egui_contexts.try_ctx_mut() {
        if ctx.is_pointer_over_area() || ctx.wants_pointer_input() {
            return;
        }
    }

    // Cancel placement on right-click or escape
    if (mouse_button.just_pressed(MouseButton::Right) || keyboard.just_pressed(KeyCode::Escape))
        && placement.placing.is_some()
    {
        placement.placing = None;
        placement.preview_position = None;
        return;
    }

    // Place building on left click
    if mouse_button.just_pressed(MouseButton::Left) {
        tracing::info!(
            "Placement click: placing={:?}, preview={:?}, valid={}",
            placement.placing,
            placement.preview_position,
            placement.valid_placement
        );
        if let (Some(building_type), Some(position), true) = (
            placement.placing,
            placement.preview_position,
            placement.valid_placement,
        ) {
            let cost = building_type.cost();

            // Check if we can afford it
            if resources.feedstock >= cost {
                tracing::info!(
                    "Placing {:?} at {:?} for {} resources",
                    building_type,
                    position,
                    cost
                );
                resources.feedstock -= cost;
                spawn_building(
                    &mut commands,
                    building_type,
                    position,
                    player_faction.faction,
                );

                // Clear placement mode after placing
                placement.placing = None;
                placement.preview_position = None;
            } else {
                tracing::warn!(
                    "Cannot afford {:?}: need {}, have {}",
                    building_type,
                    cost,
                    resources.feedstock
                );
            }
        }
    }
}

/// Updates placement ghost sprite based on current placement state.
fn update_placement_ghost_sprite(
    mut commands: Commands,
    placement: Res<BuildingPlacement>,
    mut ghost_entity: ResMut<GhostEntity>,
    mut ghosts: Query<(&mut Sprite, &mut Transform, &mut BuildingGhost)>,
) {
    let Some(building_type) = placement.placing else {
        if let Some(entity) = ghost_entity.entity.take() {
            commands.entity(entity).despawn_recursive();
        }
        return;
    };

    let Some(position) = placement.preview_position else {
        if let Some(entity) = ghost_entity.entity.take() {
            commands.entity(entity).despawn_recursive();
        }
        return;
    };

    let color = if placement.valid_placement {
        Color::srgba(0.0, 1.0, 0.0, 0.4) // Green for valid
    } else {
        Color::srgba(1.0, 0.0, 0.0, 0.4) // Red for invalid
    };

    let (width, height) = building_type.size();
    let size = Vec2::new(width, height);

    if let Some(entity) = ghost_entity.entity {
        if let Ok((mut sprite, mut transform, mut ghost)) = ghosts.get_mut(entity) {
            sprite.color = color;
            sprite.custom_size = Some(size);
            transform.translation = position.extend(-0.25);
            ghost.building_type = building_type;
            return;
        }
    }

    let entity = commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(position.extend(-0.25)),
            ..default()
        })
        .insert(BuildingGhost { building_type })
        .id();

    ghost_entity.entity = Some(entity);
}

/// Keyboard shortcuts for building placement.
fn building_hotkeys(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut placement: ResMut<BuildingPlacement>,
    resources: Res<PlayerResources>,
) {
    // Only process if not already placing
    if placement.placing.is_some() {
        return;
    }

    // B + number keys for building selection
    if keyboard.pressed(KeyCode::KeyB) {
        if keyboard.just_pressed(KeyCode::Digit1) {
            let building = BuildingType::SupplyDepot;
            if resources.feedstock >= building.cost() {
                placement.placing = Some(building);
            }
        } else if keyboard.just_pressed(KeyCode::Digit2) {
            let building = BuildingType::Barracks;
            if resources.feedstock >= building.cost() {
                placement.placing = Some(building);
            }
        } else if keyboard.just_pressed(KeyCode::Digit3) {
            let building = BuildingType::TechLab;
            if resources.feedstock >= building.cost() {
                placement.placing = Some(building);
            }
        } else if keyboard.just_pressed(KeyCode::Digit4) {
            let building = BuildingType::Turret;
            if resources.feedstock >= building.cost() {
                placement.placing = Some(building);
            }
        }
    }
}
