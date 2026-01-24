# Project Conventions

## Asset Organization

### Directory Structure

```text
assets/
├── models/
│   ├── units/
│   │   ├── continuity/
│   │   ├── collegium/
│   │   ├── tinkers/
│   │   ├── sculptors/
│   │   └── zephyr/
│   ├── buildings/
│   │   └── [same faction structure]
│   └── props/
├── textures/
│   ├── units/
│   ├── buildings/
│   ├── terrain/
│   └── effects/
├── audio/
│   ├── sfx/
│   │   ├── units/
│   │   ├── weapons/
│   │   ├── buildings/
│   │   └── ui/
│   ├── music/
│   │   ├── menu/
│   │   ├── battle/
│   │   └── ambient/
│   └── voice/
│       └── [faction structure]
├── ui/
│   ├── icons/
│   ├── portraits/
│   ├── menus/
│   └── hud/
└── data/
    ├── factions/
    ├── units/
    ├── buildings/
    ├── tech/
    └── maps/
```

### Asset Naming

#### Models

```text
[faction]_[category]_[name]_[variant].gltf

Examples:
continuity_unit_heavymech_elite.gltf
collegium_building_factory_t2.gltf
tinkers_vehicle_trike_salvage.gltf
```

#### Textures

```text
[model_name]_[type].png

Types: diffuse, normal, roughness, metallic, emissive

Examples:
continuity_unit_heavymech_diffuse.png
continuity_unit_heavymech_normal.png
```

#### Audio

```text
[category]_[name]_[variant].ogg

Examples:
weapon_laser_fire_01.ogg
unit_mech_footstep_metal_03.ogg
music_battle_intense_loop.ogg
```

## Data File Formats

### Unit Definitions (RON/JSON)

```ron
// data/units/continuity/heavy_mech.ron
UnitDefinition(
    id: "continuity_heavy_mech",
    name: "Enforcer Mech",
    faction: Continuity,
    
    stats: (
        health: 500,
        armor: 50,
        speed: 2.5,
        sight_range: 8.0,
    ),
    
    combat: (
        damage: 45,
        attack_speed: 1.2,
        range: 6.0,
        damage_type: Explosive,
    ),
    
    cost: (
        feedstock: 300,
        build_time: 25.0,
    ),
    
    requirements: [
        "continuity_mech_bay",
        "continuity_tech_heavy_armor",
    ],
    
    abilities: [
        "siege_mode",
    ],
    
    model: "models/units/continuity/heavy_mech.gltf",
    icon: "ui/icons/units/continuity_heavy_mech.png",
)
```

### Building Definitions

```ron
// data/buildings/collegium/fabricator.ron
BuildingDefinition(
    id: "collegium_fabricator",
    name: "Fabricator Array",
    faction: Collegium,
    
    stats: (
        health: 800,
        armor: 20,
    ),
    
    production: (
        queue_size: 5,
        produces: ["collegium_drone_*"],
    ),
    
    cost: (
        feedstock: 200,
        build_time: 20.0,
    ),
    
    footprint: (3, 3),
    
    model: "models/buildings/collegium/fabricator.gltf",
)
```

### Tech Tree Definitions

```ron
// data/tech/continuity_tree.ron
TechTree(
    faction: Continuity,
    
    tiers: [
        Tier(
            level: 1,
            techs: [
                Tech(
                    id: "continuity_tech_basic_armor",
                    name: "Reinforced Plating",
                    description: "Increases infantry armor by 10%",
                    cost: (feedstock: 100, research_time: 30.0),
                    effects: [
                        ModifyUnitStat(category: Infantry, stat: Armor, modifier: Percent(10)),
                    ],
                ),
            ],
        ),
        // More tiers...
    ],
)
```

## Configuration Files

### Game Configuration

```ron
// config/game.ron
GameConfig(
    simulation: (
        tick_rate: 20,          // Ticks per second
        max_units: 1500,        // Per player
    ),
    
    rendering: (
        default_resolution: (1920, 1080),
        vsync: true,
        shadows: High,
    ),
    
    audio: (
        master_volume: 0.8,
        music_volume: 0.6,
        sfx_volume: 0.8,
    ),
    
    network: (
        default_port: 7777,
        max_players: 8,
        turn_delay_ms: 100,
    ),
)
```

## Map Format

### Map Files

```text
maps/
├── 1v1/
│   ├── contested_ridge.map
│   └── contested_ridge.preview.png
├── 2v2/
└── ffa/
```

### Map Definition

```ron
// maps/1v1/contested_ridge.map
MapDefinition(
    name: "Contested Ridge",
    author: "MapMaker",
    size: (256, 256),
    players: (min: 2, max: 2),
    
    spawn_points: [
        SpawnPoint(position: (32, 32), player_slot: 0),
        SpawnPoint(position: (224, 224), player_slot: 1),
    ],
    
    resources: [
        ResourceNode(type: Feedstock, position: (64, 64), amount: 10000),
        ResourceNode(type: Feedstock, position: (192, 192), amount: 10000),
        // Contested center
        ResourceNode(type: Feedstock, position: (128, 128), amount: 15000),
    ],
    
    terrain_file: "contested_ridge_terrain.bin",
)
```

## Localization

### String IDs

```text
faction.[faction_id].name
faction.[faction_id].description
unit.[unit_id].name
unit.[unit_id].description
building.[building_id].name
tech.[tech_id].name
tech.[tech_id].description
ui.[screen].[element]
```

### Localization Files

```ron
// data/localization/en.ron
Localization(
    language: "en",
    strings: {
        "faction.continuity.name": "The Continuity Authority",
        "faction.continuity.description": "Stability through governance...",
        "unit.continuity_heavy_mech.name": "Enforcer Mech",
        "ui.hud.resources": "Resources",
    },
)
```

## Debug and Development

### Debug Flags

```ron
// config/debug.ron (not in release builds)
DebugConfig(
    show_fps: true,
    show_unit_ids: false,
    show_pathfinding: false,
    show_attack_ranges: false,
    show_fog_of_war: true,
    instant_build: false,
    infinite_resources: false,
    ai_disabled: false,
)
```

### Console Commands

```text
/spawn [unit_id] [count]      - Spawn units
/resources [amount]           - Set resources
/reveal                       - Reveal map
/fog                          - Toggle fog of war
/ai [on|off]                  - Toggle AI
/speed [multiplier]           - Game speed
/kill [unit_id|all]           - Destroy units
```

## Related Documents

- [Coding Standards](./coding-standards.md)
- [Architecture Overview](../architecture/overview.md)
- [Asset Pipeline](./asset-pipeline.md)
