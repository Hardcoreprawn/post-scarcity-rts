//! Sprite asset loading and management.
//!
//! Loads sprite textures and provides handles for rendering.

use bevy::prelude::*;
use rts_core::factions::FactionId;
use std::collections::HashMap;

use crate::components::{
    Building, BuildingType, GameDepot, GameFaction, GameHarvester, GameResourceNode,
    ResourceNodeType, Unit, UnitType,
};

/// Plugin for loading sprite assets.
pub struct SpriteLoaderPlugin;

impl Plugin for SpriteLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteAssets>()
            .init_resource::<DefaultImage>()
            .add_systems(Startup, (setup_default_image, load_sprite_assets).chain())
            .add_systems(
                PostUpdate,
                (
                    apply_unit_sprites,
                    apply_harvester_sprites,
                    apply_building_sprites,
                    apply_depot_sprites,
                    apply_resource_sprites,
                ),
            );
    }
}

/// Resource holding the default image handle for comparison.
#[derive(Resource, Default)]
pub struct DefaultImage(pub Handle<Image>);

/// System to capture the default image handle.
fn setup_default_image(mut default_image: ResMut<DefaultImage>) {
    default_image.0 = Handle::default();
}

/// Resource containing handles to all sprite textures.
#[derive(Resource, Default)]
pub struct SpriteAssets {
    /// Unit sprites by faction and unit type.
    pub units: HashMap<(FactionId, UnitSprite), Handle<Image>>,
    /// Building sprites by faction and building type.
    pub buildings: HashMap<(FactionId, BuildingSprite), Handle<Image>>,
    /// Neutral sprites (resources, terrain).
    pub neutral: HashMap<NeutralSprite, Handle<Image>>,
    /// Whether assets have been loaded.
    pub loaded: bool,
}

/// Unit sprite types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitSprite {
    /// Infantry unit sprite.
    Infantry,
    /// Ranger unit sprite.
    Ranger,
    /// Harvester unit sprite.
    Harvester,
}

/// Building sprite types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildingSprite {
    /// Depot building sprite.
    Depot,
    /// Barracks building sprite.
    Barracks,
    /// Supply depot building sprite.
    SupplyDepot,
    /// Tech lab building sprite.
    TechLab,
    /// Turret building sprite.
    Turret,
}

/// Neutral (non-faction) sprite types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NeutralSprite {
    /// Temporary resource sprite.
    ResourceTemp,
    /// Permanent resource sprite.
    ResourcePerm,
    /// Terrain sprite.
    Terrain,
}

impl SpriteAssets {
    /// Get the sprite handle for a unit.
    pub fn get_unit(&self, faction: FactionId, sprite: UnitSprite) -> Option<Handle<Image>> {
        self.units.get(&(faction, sprite)).cloned()
    }

    /// Get the sprite handle for a building.
    pub fn get_building(
        &self,
        faction: FactionId,
        sprite: BuildingSprite,
    ) -> Option<Handle<Image>> {
        self.buildings.get(&(faction, sprite)).cloned()
    }

    /// Get a neutral sprite handle.
    pub fn get_neutral(&self, sprite: NeutralSprite) -> Option<Handle<Image>> {
        self.neutral.get(&sprite).cloned()
    }
}

/// System to load all sprite assets at startup.
fn load_sprite_assets(mut sprites: ResMut<SpriteAssets>, asset_server: Res<AssetServer>) {
    // Load faction-specific unit sprites
    let factions = [
        (FactionId::Continuity, "continuity"),
        (FactionId::Collegium, "collegium"),
    ];

    let unit_types = [
        (UnitSprite::Infantry, "infantry"),
        (UnitSprite::Ranger, "ranger"),
        (UnitSprite::Harvester, "harvester"),
    ];

    let building_types = [
        (BuildingSprite::Depot, "depot"),
        (BuildingSprite::Barracks, "barracks"),
        (BuildingSprite::SupplyDepot, "supply_depot"),
        (BuildingSprite::TechLab, "tech_lab"),
        (BuildingSprite::Turret, "turret"),
    ];

    for (faction_id, faction_name) in &factions {
        for (sprite_type, sprite_name) in &unit_types {
            let path = format!("textures/sprites/{}/{}.png", faction_name, sprite_name);
            let handle = asset_server.load(&path);
            sprites.units.insert((*faction_id, *sprite_type), handle);
        }

        for (sprite_type, sprite_name) in &building_types {
            let path = format!("textures/sprites/{}/{}.png", faction_name, sprite_name);
            let handle = asset_server.load(&path);
            sprites
                .buildings
                .insert((*faction_id, *sprite_type), handle);
        }
    }

    // Load neutral sprites
    let neutral_types = [
        (NeutralSprite::ResourceTemp, "resource_temp"),
        (NeutralSprite::ResourcePerm, "resource_perm"),
        (NeutralSprite::Terrain, "terrain"),
    ];

    for (sprite_type, sprite_name) in &neutral_types {
        let path = format!("textures/sprites/{}.png", sprite_name);
        let handle = asset_server.load(&path);
        sprites.neutral.insert(*sprite_type, handle);
    }

    sprites.loaded = true;
    tracing::info!("Loaded sprite assets");
}

// ============================================================================
// Sprite Application Systems
// ============================================================================

/// System to apply textures to combat units (Infantry, Ranger).
/// Runs every frame to catch entities that were spawned before assets loaded.
fn apply_unit_sprites(
    sprite_assets: Res<SpriteAssets>,
    default_image: Res<DefaultImage>,
    mut units: Query<(&Unit, &GameFaction, &mut Handle<Image>), Without<GameHarvester>>,
) {
    if !sprite_assets.loaded {
        return;
    }
    for (unit, faction, mut texture) in units.iter_mut() {
        // Skip if already has a real texture
        if *texture != default_image.0 {
            continue;
        }
        let sprite_type = match unit.unit_type {
            UnitType::Infantry => UnitSprite::Infantry,
            UnitType::Ranger => UnitSprite::Ranger,
            UnitType::Harvester => UnitSprite::Harvester,
        };
        if let Some(handle) = sprite_assets.get_unit(faction.faction, sprite_type) {
            *texture = handle;
        }
    }
}

/// System to apply textures to harvesters.
fn apply_harvester_sprites(
    sprite_assets: Res<SpriteAssets>,
    default_image: Res<DefaultImage>,
    mut harvesters: Query<(&GameFaction, &mut Handle<Image>), With<GameHarvester>>,
) {
    if !sprite_assets.loaded {
        return;
    }
    for (faction, mut texture) in harvesters.iter_mut() {
        if *texture != default_image.0 {
            continue;
        }
        if let Some(handle) = sprite_assets.get_unit(faction.faction, UnitSprite::Harvester) {
            *texture = handle;
        }
    }
}

/// System to apply textures to buildings with Building component.
fn apply_building_sprites(
    sprite_assets: Res<SpriteAssets>,
    default_image: Res<DefaultImage>,
    mut buildings: Query<(&Building, &GameFaction, &mut Handle<Image>)>,
) {
    if !sprite_assets.loaded {
        return;
    }
    for (building, faction, mut texture) in buildings.iter_mut() {
        if *texture != default_image.0 {
            continue;
        }
        let sprite_type = match building.building_type {
            BuildingType::Depot => BuildingSprite::Depot,
            BuildingType::Barracks => BuildingSprite::Barracks,
            BuildingType::SupplyDepot => BuildingSprite::SupplyDepot,
            BuildingType::TechLab => BuildingSprite::TechLab,
            BuildingType::Turret => BuildingSprite::Turret,
        };
        if let Some(handle) = sprite_assets.get_building(faction.faction, sprite_type) {
            *texture = handle;
        }
    }
}

/// System to apply textures to depots (main base) without Building component.
fn apply_depot_sprites(
    sprite_assets: Res<SpriteAssets>,
    default_image: Res<DefaultImage>,
    mut depots: Query<(&GameFaction, &mut Handle<Image>), (With<GameDepot>, Without<Building>)>,
) {
    if !sprite_assets.loaded {
        return;
    }
    for (faction, mut texture) in depots.iter_mut() {
        if *texture != default_image.0 {
            continue;
        }
        if let Some(handle) = sprite_assets.get_building(faction.faction, BuildingSprite::Depot) {
            *texture = handle;
        }
    }
}

/// System to apply textures to resource nodes.
fn apply_resource_sprites(
    sprite_assets: Res<SpriteAssets>,
    default_image: Res<DefaultImage>,
    mut resources: Query<(&GameResourceNode, &mut Handle<Image>)>,
) {
    if !sprite_assets.loaded {
        return;
    }
    for (node, mut texture) in resources.iter_mut() {
        if *texture != default_image.0 {
            continue;
        }
        let sprite_type = if node.node_type == ResourceNodeType::Permanent {
            NeutralSprite::ResourcePerm
        } else {
            NeutralSprite::ResourceTemp
        };
        if let Some(handle) = sprite_assets.get_neutral(sprite_type) {
            *texture = handle;
        }
    }
}
