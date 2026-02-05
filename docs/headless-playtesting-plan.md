# Headless Playtesting & Balance Tuning Plan

> Created: 2026-02-03  
> Status: Ready for Implementation  
> Focus: Continuity vs Collegium factions

## Overview

This plan establishes an AI-driven headless playtesting system that:

1. Runs 1000+ automated games for balance validation
2. Collects comprehensive metrics (economy, combat, timing, positioning)
3. Captures screenshots at key moments for visual quality review
4. Provides auto-suggestions for balance tuning
5. Maintains determinism for reproducible results

## Visual Quality Bar

Target: **Dark Reign / Supreme Commander / Total Annihilation** quality

- 3D rendered sprite style with strong silhouettes
- Readable at all zoom levels
- Distinct faction identity

### Faction Visual Standards

**Continuity** (Tech/Preservation):

- Blue-silver color palette
- Sleek, shell-like forms with smooth curves
- Glowing blue accents (eyes, vents, weapon tips)
- Clean, minimalist aesthetic
- Preservation pods/stasis elements on buildings

**Collegium** (Knowledge/Geometric):

- Gold-bronze color palette
- Angular, crystalline forms
- Floating geometric elements (orbiting cubes, pyramids)
- Visible energy conduits
- Archive/library motifs on structures

---

## Phase 1: Scenario System (Foundation)

### 1.1 Create `crates/rts_headless/src/scenario.rs`

```rust
use serde::{Deserialize, Serialize};
use rts_core::math::FixedPoint;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub description: String,
    pub map_size: (u32, u32),
    pub factions: Vec<FactionSetup>,
    pub victory_conditions: VictoryConditions,
    pub initial_resources: ResourceSetup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionSetup {
    pub faction_id: String,  // "continuity" or "collegium"
    pub ai_controller: AiController,
    pub starting_units: Vec<UnitPlacement>,
    pub starting_buildings: Vec<BuildingPlacement>,
    pub spawn_position: (i32, i32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiController {
    Sandbox,                    // Full autonomous AI
    Scripted(String),           // Named strategy file
    External,                   // JSON protocol control
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitPlacement {
    pub kind: String,
    pub position: (i32, i32),
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingPlacement {
    pub kind: String,
    pub position: (i32, i32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VictoryConditions {
    pub elimination: bool,
    pub time_limit_ticks: Option<u64>,
    pub resource_threshold: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSetup {
    pub credits: i64,
    pub ore_nodes: Vec<(i32, i32, i64)>,  // x, y, amount
}
```

### 1.2 Create `assets/scenarios/skirmish_1v1.ron`

```ron
Scenario(
    name: "Standard 1v1 Skirmish",
    description: "Balanced starting positions for faction matchup testing",
    map_size: (256, 256),
    factions: [
        FactionSetup(
            faction_id: "continuity",
            ai_controller: Sandbox,
            starting_units: [
                UnitPlacement(kind: "scout", position: (32, 128), count: 2),
                UnitPlacement(kind: "harvester", position: (40, 128), count: 1),
            ],
            starting_buildings: [
                BuildingPlacement(kind: "command_center", position: (24, 128)),
            ],
            spawn_position: (24, 128),
        ),
        FactionSetup(
            faction_id: "collegium",
            ai_controller: Sandbox,
            starting_units: [
                UnitPlacement(kind: "scout", position: (224, 128), count: 2),
                UnitPlacement(kind: "harvester", position: (216, 128), count: 1),
            ],
            starting_buildings: [
                BuildingPlacement(kind: "command_center", position: (232, 128)),
            ],
            spawn_position: (232, 128),
        ),
    ],
    victory_conditions: VictoryConditions(
        elimination: true,
        time_limit_ticks: Some(36000),  // 10 minutes at 60 tps
        resource_threshold: None,
    ),
    initial_resources: ResourceSetup(
        credits: 1000,
        ore_nodes: [
            (64, 100, 5000),
            (64, 156, 5000),
            (192, 100, 5000),
            (192, 156, 5000),
            (128, 128, 10000),  // Contested center
        ],
    ),
)
```

### 1.3 Wire Commands to CoreCommandBuffer

In `runner.rs`, update command processing to actually affect simulation:

```rust
fn process_command(
    cmd: &Command,
    world: &mut World,
    entity_map: &mut EntityIdMap,
) -> Response {
    match cmd {
        Command::Move { entity_id, x, y } => {
            if let Some(&entity) = entity_map.get(entity_id) {
                let pos = FixedPoint::from_i32(*x, *y);
                world.resource_scope(|world, mut commands: Mut<CoreCommandBuffer>| {
                    commands.move_unit(entity, pos);
                });
                Response::Ack
            } else {
                Response::Error { message: format!("Unknown entity: {}", entity_id) }
            }
        }
        Command::Attack { attacker_id, target_id } => {
            // Similar wiring to CoreCommandBuffer::attack()
        }
        Command::Build { builder_id, building_type, x, y } => {
            // Wire to construction system
        }
        // ... etc
    }
}
```

---

## Phase 2: Complete AI Player

### 2.1 Extend `crates/rts_game/src/ai.rs`

The current AI has basic states. Add:

```rust
#[derive(Debug, Clone)]
pub struct AiEconomyState {
    pub target_harvesters: u32,
    pub target_refineries: u32,
    pub income_rate: FixedPoint,
    pub expense_rate: FixedPoint,
}

#[derive(Debug, Clone)]
pub struct AiBuildOrder {
    pub queue: VecDeque<BuildOrderItem>,
    pub current_phase: BuildPhase,
}

#[derive(Debug, Clone)]
pub enum BuildOrderItem {
    Unit(String),
    Building(String),
    Research(String),
    WaitForResources(i64),
    WaitForUnit(String, u32),  // unit type, count
}

#[derive(Debug, Clone, Copy)]
pub enum BuildPhase {
    Opening,
    Expansion,
    Military,
    Endgame,
}
```

### 2.2 AI Construction Logic

```rust
fn ai_construction_system(
    mut ai_query: Query<&mut AiController>,
    economy: Res<FactionEconomy>,
    buildings: Query<&Building>,
    mut commands: ResMut<CoreCommandBuffer>,
) {
    for mut ai in ai_query.iter_mut() {
        let harvester_count = count_units_of_type(&ai.faction, "harvester");
        let refinery_count = count_buildings_of_type(&ai.faction, "refinery");
        
        // Economic scaling
        if harvester_count < ai.economy.target_harvesters {
            if can_afford("harvester", &economy) {
                commands.produce_unit(ai.factory, "harvester");
            }
        }
        
        // Expansion timing
        if should_expand(&ai, &economy) {
            let expansion_site = find_expansion_site(&ai);
            commands.build_structure(ai.builder, "command_center", expansion_site);
        }
    }
}
```

### 2.3 Scripted Strategies

Create `assets/strategies/` directory with RON files:

**rush.ron**:

```ron
Strategy(
    name: "Early Rush",
    build_order: [
        Unit("scout"),
        Unit("scout"),
        Building("barracks"),
        Unit("infantry"),
        Unit("infantry"),
        Unit("infantry"),
        WaitForUnit("infantry", 6),
        // Attack timing: ~2 minutes
    ],
    attack_timing: 7200,  // ticks
    composition: { "infantry": 0.8, "scout": 0.2 },
)
```

**economic.ron**:

```ron
Strategy(
    name: "Economic Boom",
    build_order: [
        Unit("harvester"),
        Building("refinery"),
        Unit("harvester"),
        Building("refinery"),
        WaitForResources(3000),
        Building("factory"),
        // Delayed military
    ],
    attack_timing: 18000,
    composition: { "tank": 0.6, "infantry": 0.3, "artillery": 0.1 },
)
```

---

## Phase 3: Metrics Collection

### 3.1 Create `crates/rts_headless/src/metrics.rs`

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameMetrics {
    pub game_id: String,
    pub scenario: String,
    pub seed: u64,
    pub duration_ticks: u64,
    pub winner: Option<String>,
    pub win_condition: String,
    pub factions: HashMap<String, FactionMetrics>,
    pub events: Vec<TimedEvent>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FactionMetrics {
    pub faction_id: String,
    pub final_score: i64,
    
    // Economy
    pub total_resources_gathered: i64,
    pub total_resources_spent: i64,
    pub peak_income_rate: FixedPoint,
    pub resource_efficiency: f64,  // gathered / potential
    
    // Military
    pub units_produced: HashMap<String, u32>,
    pub units_lost: HashMap<String, u32>,
    pub units_killed: HashMap<String, u32>,
    pub buildings_constructed: HashMap<String, u32>,
    pub buildings_destroyed: HashMap<String, u32>,
    pub buildings_lost: HashMap<String, u32>,
    
    // Combat
    pub total_damage_dealt: i64,
    pub total_damage_taken: i64,
    pub battles_won: u32,
    pub battles_lost: u32,
    
    // Timing
    pub first_attack_tick: Option<u64>,
    pub first_expansion_tick: Option<u64>,
    pub tech_unlock_times: HashMap<String, u64>,
    
    // Positioning
    pub map_control_over_time: Vec<(u64, f64)>,  // tick, percentage
    pub average_army_position: Vec<(u64, i32, i32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimedEvent {
    pub tick: u64,
    pub event_type: EventType,
    pub faction: String,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    UnitProduced,
    UnitKilled,
    BuildingCompleted,
    BuildingDestroyed,
    BattleStarted,
    BattleEnded,
    ExpansionStarted,
    TechUnlocked,
    MajorEngagement,  // >5 units involved
}
```

### 3.2 Metrics Collection System

```rust
fn metrics_collection_system(
    mut metrics: ResMut<GameMetrics>,
    tick: Res<SimulationTick>,
    events: Res<TickEvents>,
    factions: Query<&Faction>,
    units: Query<(&Unit, &Position, &Faction)>,
    buildings: Query<(&Building, &Faction)>,
) {
    // Record events from this tick
    for event in events.iter() {
        match event {
            TickEvent::UnitSpawned { entity, kind, faction } => {
                metrics.factions.entry(faction.clone())
                    .or_default()
                    .units_produced
                    .entry(kind.clone())
                    .and_modify(|c| *c += 1)
                    .or_insert(1);
            }
            TickEvent::UnitDestroyed { entity, kind, faction, killer } => {
                // Record kill/death
            }
            // ... handle all event types
        }
    }
    
    // Periodic snapshots (every 600 ticks = 10 seconds)
    if tick.0 % 600 == 0 {
        for faction in factions.iter() {
            let control = calculate_map_control(&faction, &units);
            metrics.factions.get_mut(&faction.id)
                .map(|m| m.map_control_over_time.push((tick.0, control)));
        }
    }
}
```

---

## Phase 4: Screenshot Capture

### 4.1 Create `crates/rts_headless/src/screenshot.rs`

For headless rendering, we have two options:

#### Option A: Bevy Render to Texture (if GPU available)

```rust
use bevy::render::view::screenshot::ScreenshotManager;

fn screenshot_system(
    mut screenshot_manager: ResMut<ScreenshotManager>,
    trigger_events: EventReader<ScreenshotTrigger>,
) {
    for trigger in trigger_events.read() {
        let path = format!("screenshots/{}/{}.png", trigger.game_id, trigger.name);
        screenshot_manager.save_screenshot_to_disk(path);
    }
}
```

#### Option B: State Dump + Offline Renderer

```rust
#[derive(Serialize)]
pub struct VisualState {
    pub tick: u64,
    pub camera: CameraState,
    pub units: Vec<UnitVisual>,
    pub buildings: Vec<BuildingVisual>,
    pub projectiles: Vec<ProjectileVisual>,
    pub effects: Vec<EffectVisual>,
}

fn dump_visual_state(
    units: Query<(&Unit, &Position, &Faction, &AnimationState)>,
    buildings: Query<(&Building, &Position, &Faction, &Health)>,
) -> VisualState {
    // Serialize complete visual state for offline rendering
}
```

### 4.2 Screenshot Triggers

```rust
pub enum ScreenshotTrigger {
    FirstContact,           // First combat between factions
    MajorBattle,            // >10 units engaged
    BaseUnderAttack,        // Enemy near command center
    ExpansionComplete,      // New base established
    TechMilestone,          // Tier 2/3 unlocked
    Victory,                // Game end
    TimedSnapshot(u64),     // Every N ticks
}

fn screenshot_trigger_system(
    mut triggers: EventWriter<ScreenshotTrigger>,
    tick: Res<SimulationTick>,
    battles: Query<&Battle>,
    // ...
) {
    // Detect major battles
    for battle in battles.iter() {
        if battle.unit_count > 10 && !battle.screenshot_taken {
            triggers.send(ScreenshotTrigger::MajorBattle);
        }
    }
    
    // Timed snapshots every 2 minutes
    if tick.0 % 7200 == 0 {
        triggers.send(ScreenshotTrigger::TimedSnapshot(tick.0));
    }
}
```

### 4.3 Screenshot Manifest

```rust
#[derive(Serialize)]
pub struct ScreenshotManifest {
    pub game_id: String,
    pub screenshots: Vec<ScreenshotEntry>,
}

#[derive(Serialize)]
pub struct ScreenshotEntry {
    pub filename: String,
    pub tick: u64,
    pub trigger: String,
    pub camera_position: (f32, f32, f32),
    pub visible_units: Vec<String>,
    pub review_prompts: Vec<String>,  // Questions for visual review
}
```

---

## Phase 5: Visual Quality Review

### 5.1 Review Criteria Checklist

Create `docs/visual-review-criteria.md`:

```markdown
# Visual Quality Review Criteria

## Readability
- [ ] Units distinguishable at max zoom-out
- [ ] Faction colors clearly different
- [ ] Health bars readable
- [ ] Selection indicators visible
- [ ] Attack animations clear

## Faction Identity
### Continuity
- [ ] Blue-silver palette consistent
- [ ] Smooth, curved forms
- [ ] Glowing blue accents visible
- [ ] Tech/preservation aesthetic clear

### Collegium  
- [ ] Gold-bronze palette consistent
- [ ] Angular, geometric forms
- [ ] Floating elements present
- [ ] Knowledge/archive aesthetic clear

## Animation Quality
- [ ] Idle animations present
- [ ] Move animations smooth
- [ ] Attack animations impactful
- [ ] Death animations clear
- [ ] Building construction visible

## Effects
- [ ] Weapon effects visible
- [ ] Explosion effects readable
- [ ] Projectile trails clear
- [ ] Selection effects distinct

## Consistency
- [ ] Similar units same scale
- [ ] Shadow directions consistent
- [ ] Lighting coherent
- [ ] Ground contact clear
```

### 5.2 Automated Visual Tests

```rust
fn silhouette_test(screenshot: &Image) -> SilhouetteResult {
    // Convert to grayscale
    // Threshold to binary
    // Check distinct shapes are recognizable
    SilhouetteResult {
        unit_count_detected: detected,
        unit_count_expected: expected,
        overlap_percentage: overlap,
        pass: detected == expected && overlap < 0.1,
    }
}

fn color_distinction_test(screenshot: &Image) -> ColorResult {
    // Sample faction unit pixels
    // Verify color distance > threshold
    ColorResult {
        faction_colors: colors,
        min_distance: distance,
        pass: distance > 50.0,  // In LAB color space
    }
}
```

---

## Phase 6: Batch Runner

### 6.1 Create `crates/rts_headless/src/batch.rs`

```rust
use rayon::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};

pub struct BatchConfig {
    pub scenario: String,
    pub game_count: u32,
    pub parallel_games: u32,
    pub output_dir: PathBuf,
    pub screenshot_mode: ScreenshotMode,
    pub seed_start: u64,
}

pub fn run_batch(config: BatchConfig) -> BatchResults {
    let completed = AtomicU32::new(0);
    let total = config.game_count;
    
    let results: Vec<GameMetrics> = (0..config.game_count)
        .into_par_iter()
        .map(|i| {
            let seed = config.seed_start + i as u64;
            let metrics = run_single_game(&config.scenario, seed, &config);
            
            let done = completed.fetch_add(1, Ordering::Relaxed) + 1;
            if done % 10 == 0 {
                eprintln!("Progress: {}/{}", done, total);
            }
            
            metrics
        })
        .collect();
    
    BatchResults {
        config,
        games: results,
        summary: calculate_summary(&results),
    }
}
```

### 6.2 CLI Commands

```rust
#[derive(Parser)]
pub enum Commands {
    /// Run a single interactive game
    Run {
        #[arg(short, long)]
        scenario: Option<String>,
    },
    
    /// Run batch of games for balance testing
    Batch {
        #[arg(short, long, default_value = "skirmish_1v1")]
        scenario: String,
        
        #[arg(short, long, default_value = "1000")]
        count: u32,
        
        #[arg(short, long, default_value = "8")]
        parallel: u32,
        
        #[arg(short, long)]
        output: PathBuf,
    },
    
    /// Analyze batch results and suggest balance changes
    Analyze {
        #[arg(short, long)]
        input: PathBuf,
        
        #[arg(long)]
        suggest: bool,
    },
    
    /// Generate visual review report
    Review {
        #[arg(short, long)]
        screenshots: PathBuf,
        
        #[arg(short, long)]
        output: PathBuf,
    },
}
```

### 6.3 Create `crates/rts_headless/src/analyzer.rs`

```rust
#[derive(Debug, Serialize)]
pub struct BalanceAnalysis {
    pub win_rates: HashMap<String, f64>,
    pub matchup_matrix: HashMap<(String, String), f64>,
    pub outliers: Vec<BalanceOutlier>,
    pub suggestions: Vec<BalanceSuggestion>,
}

#[derive(Debug, Serialize)]
pub struct BalanceOutlier {
    pub category: String,
    pub metric: String,
    pub value: f64,
    pub expected_range: (f64, f64),
    pub severity: Severity,
}

#[derive(Debug, Serialize)]
pub struct BalanceSuggestion {
    pub target: String,   // e.g., "continuity.tank.damage"
    pub current: f64,
    pub suggested: f64,
    pub reasoning: String,
    pub confidence: f64,
}

pub fn analyze_batch(results: &BatchResults) -> BalanceAnalysis {
    let mut analysis = BalanceAnalysis::default();
    
    // Calculate win rates
    for metrics in &results.games {
        if let Some(winner) = &metrics.winner {
            *analysis.win_rates.entry(winner.clone()).or_default() += 1.0;
        }
    }
    
    // Normalize
    let total = results.games.len() as f64;
    for rate in analysis.win_rates.values_mut() {
        *rate /= total;
    }
    
    // Detect imbalances
    for (faction, rate) in &analysis.win_rates {
        if *rate < 0.45 || *rate > 0.55 {
            analysis.outliers.push(BalanceOutlier {
                category: "win_rate".into(),
                metric: faction.clone(),
                value: *rate,
                expected_range: (0.45, 0.55),
                severity: if *rate < 0.40 || *rate > 0.60 { 
                    Severity::High 
                } else { 
                    Severity::Medium 
                },
            });
        }
    }
    
    // Generate suggestions
    generate_balance_suggestions(&mut analysis, results);
    
    analysis
}

fn generate_balance_suggestions(
    analysis: &mut BalanceAnalysis,
    results: &BatchResults,
) {
    // Example: If faction wins too much with rushes
    let early_wins = count_early_victories(results, "continuity");
    if early_wins > results.games.len() as f64 * 0.6 {
        analysis.suggestions.push(BalanceSuggestion {
            target: "continuity.infantry.build_time".into(),
            current: 5.0,
            suggested: 6.0,
            reasoning: "Continuity wins 60%+ of games with early rushes. \
                        Slowing infantry production gives opponents time to respond.".into(),
            confidence: 0.75,
        });
    }
}
```

---

## Phase 7: Integration & Workflow

### 7.1 Determinism Verification

```rust
fn verify_determinism(scenario: &str, seed: u64, runs: u32) -> bool {
    let hashes: Vec<u64> = (0..runs)
        .map(|_| {
            run_game_and_get_final_hash(scenario, seed)
        })
        .collect();
    
    hashes.iter().all(|h| *h == hashes[0])
}
```

### 7.2 Progress Reporting

```rust
pub struct BatchProgress {
    pub total: u32,
    pub completed: u32,
    pub current_game: Option<GameProgress>,
    pub estimated_remaining: Duration,
    pub results_so_far: PartialResults,
}

impl BatchProgress {
    pub fn display(&self) {
        eprintln!("╔════════════════════════════════════╗");
        eprintln!("║ Batch Progress: {:>4}/{:<4} ({:>5.1}%) ║", 
            self.completed, self.total, 
            self.completed as f64 / self.total as f64 * 100.0);
        eprintln!("║ ETA: {:>28} ║", format_duration(self.estimated_remaining));
        eprintln!("╟────────────────────────────────────╢");
        eprintln!("║ Win Rates So Far:                  ║");
        for (faction, rate) in &self.results_so_far.win_rates {
            eprintln!("║   {:<12}: {:>5.1}%              ║", faction, rate * 100.0);
        }
        eprintln!("╚════════════════════════════════════╝");
    }
}
```

### 7.3 Complete Workflow

```bash
# 1. Run batch balance test
cargo run -p rts_headless -- batch \
    --scenario skirmish_1v1 \
    --count 1000 \
    --parallel 8 \
    --output results/batch_001

# 2. Analyze results
cargo run -p rts_headless -- analyze \
    --input results/batch_001 \
    --suggest > balance_suggestions.json

# 3. Review visual quality
cargo run -p rts_headless -- review \
    --screenshots results/batch_001/screenshots \
    --output visual_review.html

# 4. Apply suggestions (manual review recommended)
# Edit faction RON files based on balance_suggestions.json

# 5. Re-run to verify
cargo run -p rts_headless -- batch \
    --scenario skirmish_1v1 \
    --count 100 \
    --output results/batch_002_verify
```

---

## Implementation Order

1. **Phase 1**: Scenario system + command wiring (2-3 hours)
2. **Phase 2**: AI construction + strategies (3-4 hours)
3. **Phase 3**: Metrics collection (2-3 hours)
4. **Phase 4**: Screenshot capture (2-3 hours)
5. **Phase 5**: Visual review criteria (1-2 hours)
6. **Phase 6**: Batch runner + analyzer (3-4 hours)
7. **Phase 7**: Integration + testing (2-3 hours)

## Total Estimated Time: 15-22 hours

---

## Current State (as of 2026-02-03)

### Completed

- [x] `rts_headless` crate structure created
- [x] JSON protocol defined in `protocol.rs`
- [x] Basic CLI with clap
- [x] Runner plugin structure
- [x] Wire commands to CoreCommandBuffer (Move, Attack, Stop work)
- [x] Scenario loader (`scenario.rs` + `skirmish_1v1.ron`)
- [x] Scripted strategies (`strategies.rs` + rush/economic/turtle.ron)
- [x] Metrics collection (`metrics.rs` with full tracking)
- [x] Screenshot capture (`screenshot.rs` with state dumps)
- [x] Visual review system (`visual_review.rs` + criteria doc)
- [x] Batch runner (`batch.rs` with rayon parallelism)
- [x] Balance analyzer (`analyzer.rs` with suggestions)
- [x] Full CLI with subcommands (run, batch, analyze, review, verify)

### Not Started

- [ ] Full AI construction logic (exists in stub form)
- [ ] Replay verification
- [ ] Benchmark mode

---

## Files Created/Modified

| File | Status | Purpose |
| ------- | ------- | -------- |
| `crates/rts_headless/src/scenario.rs` | ✅ Created | Scenario loading (RON format) |
| `crates/rts_headless/src/strategies.rs` | ✅ Created | AI strategy system |
| `crates/rts_headless/src/metrics.rs` | ✅ Created | Metrics collection |
| `crates/rts_headless/src/screenshot.rs` | ✅ Created | Screenshot capture |
| `crates/rts_headless/src/visual_review.rs` | ✅ Created | Visual quality review |
| `crates/rts_headless/src/batch.rs` | ✅ Created | Parallel game runner |
| `crates/rts_headless/src/analyzer.rs` | ✅ Created | Balance analysis |
| `crates/rts_headless/src/runner.rs` | ✅ Modified | Wire commands to sim |
| `crates/rts_headless/src/main.rs` | ✅ Modified | Full CLI with subcommands |
| `crates/rts_game/assets/scenarios/skirmish_1v1.ron` | ✅ Created | Standard 1v1 scenario |
| `crates/rts_game/assets/strategies/*.ron` | ✅ Created | Rush/Economic/Turtle strategies |
| `docs/visual-review-criteria.md` | ✅ Created | Visual quality checklist |

---

## Notes

- All code uses fixed-point math (I32F32) for determinism
- Rayon provides parallelism for batch runs
- Screenshot system can work headless via state dumps
- Metrics are serialized to JSON for analysis
- Balance suggestions are auto-generated but require manual review
- 48 unit tests covering all new modules
