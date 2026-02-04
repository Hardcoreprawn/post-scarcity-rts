//! ASCII battle visualizer for screenshot analysis.
//!
//! Renders game state screenshots as ASCII art for quick terminal review.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// A unit captured in a screenshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitSnapshot {
    pub entity_id: u32,
    pub kind: String,
    pub faction: String,
    pub position: [f32; 2],
    pub rotation: f32,
    pub health_percent: f32,
    pub animation_state: String,
    pub animation_frame: u32,
    pub is_selected: bool,
    pub current_action: Option<String>,
}

/// A building captured in a screenshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingSnapshot {
    pub entity_id: u32,
    pub kind: String,
    pub faction: String,
    pub position: [f32; 2],
    pub health_percent: f32,
}

/// Screenshot state data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotState {
    pub tick: u64,
    pub game_id: String,
    pub trigger: serde_json::Value,
    pub camera: serde_json::Value,
    pub units: Vec<UnitSnapshot>,
    pub buildings: Vec<BuildingSnapshot>,
    pub projectiles: Vec<serde_json::Value>,
    pub effects: Vec<serde_json::Value>,
    pub map_bounds: [u32; 2],
    pub fog_of_war: Option<serde_json::Value>,
}

impl ScreenshotState {
    /// Load from JSON file.
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

/// ASCII visualization configuration.
#[derive(Debug, Clone)]
pub struct AsciiConfig {
    /// Width of the ASCII viewport.
    pub width: usize,
    /// Height of the ASCII viewport.
    pub height: usize,
    /// Show health bars.
    pub show_health: bool,
    /// Show unit counts legend.
    pub show_legend: bool,
    /// Use colored output (ANSI).
    pub use_color: bool,
}

impl Default for AsciiConfig {
    fn default() -> Self {
        Self {
            width: 80,
            height: 24,
            show_health: true,
            show_legend: true,
            use_color: true,
        }
    }
}

/// Character representation for entities.
fn unit_char(kind: &str, faction: &str) -> char {
    let base = match kind.to_lowercase().as_str() {
        "infantry" | "unit" => 'I',
        "scout" => 's',
        "ranger" => 'R',
        "tank" => 'T',
        "harvester" => 'H',
        "depot" | "command_center" => '#',
        "barracks" => 'B',
        "turret" => '^',
        "supply_depot" => 'S',
        "tech_lab" => 'L',
        _ => 'o',
    };

    // Use lowercase for one faction, uppercase for the other
    if faction.to_lowercase().starts_with("cont") || faction == "continuity" {
        base.to_lowercase().next().unwrap_or(base)
    } else {
        base.to_uppercase().next().unwrap_or(base)
    }
}

/// ANSI color codes.
#[allow(dead_code)]
mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";

    pub const BLUE: &str = "\x1b[34m";
    pub const CYAN: &str = "\x1b[36m";
    pub const RED: &str = "\x1b[31m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const GREEN: &str = "\x1b[32m";
    pub const WHITE: &str = "\x1b[37m";
    pub const GRAY: &str = "\x1b[90m";

    pub const BG_GRAY: &str = "\x1b[100m";
}

fn faction_color(faction: &str) -> &'static str {
    if faction.to_lowercase().starts_with("cont") || faction == "continuity" {
        colors::BLUE
    } else if faction.to_lowercase().starts_with("coll") || faction == "collegium" {
        colors::YELLOW
    } else {
        colors::WHITE
    }
}

fn health_color(health_percent: f32) -> &'static str {
    if health_percent > 0.66 {
        colors::GREEN
    } else if health_percent > 0.33 {
        colors::YELLOW
    } else {
        colors::RED
    }
}

/// Render a screenshot as ASCII art.
pub fn render_ascii(state: &ScreenshotState, config: &AsciiConfig) -> String {
    let mut output = String::new();

    // Create grid
    let mut grid: Vec<Vec<(char, String)>> =
        vec![vec![('.', String::new()); config.width]; config.height];

    // Map bounds
    let map_w = state.map_bounds[0].max(1) as f32;
    let map_h = state.map_bounds[1].max(1) as f32;

    // Calculate scale
    let scale_x = (config.width - 2) as f32 / map_w;
    let scale_y = (config.height - 4) as f32 / map_h;

    // Track faction counts
    let mut faction_counts: HashMap<String, (u32, u32, u32)> = HashMap::new(); // (units, damaged, dead)

    // Place buildings first (they're larger)
    for bld in &state.buildings {
        let x = ((bld.position[0] * scale_x) as usize).min(config.width - 1);
        let y = ((bld.position[1] * scale_y) as usize).min(config.height - 5);

        let ch = unit_char(&bld.kind, &bld.faction);
        let color = if config.use_color {
            format!("{}{}", colors::BOLD, faction_color(&bld.faction))
        } else {
            String::new()
        };

        // Place building (2x2 for HQ)
        if bld.kind.contains("depot") || bld.kind.contains("command") {
            for dy in 0..2 {
                for dx in 0..2 {
                    if y + dy < config.height - 4 && x + dx < config.width - 1 {
                        grid[y + dy][x + dx] = (ch, color.clone());
                    }
                }
            }
        } else {
            grid[y][x] = (ch, color);
        }
    }

    // Place units
    for unit in &state.units {
        let x = ((unit.position[0] * scale_x) as usize).min(config.width - 1);
        let y = ((unit.position[1] * scale_y) as usize).min(config.height - 5);

        let ch = unit_char(&unit.kind, &unit.faction);

        // Color based on faction and health
        let color = if config.use_color {
            if unit.health_percent < 1.0 {
                format!(
                    "{}{}",
                    health_color(unit.health_percent),
                    faction_color(&unit.faction)
                )
            } else {
                faction_color(&unit.faction).to_string()
            }
        } else {
            String::new()
        };

        grid[y][x] = (ch, color);

        // Track counts
        let entry = faction_counts
            .entry(unit.faction.clone())
            .or_insert((0, 0, 0));
        entry.0 += 1;
        if unit.health_percent < 1.0 {
            entry.1 += 1;
        }
    }

    // Build output
    // Header
    let trigger_str = match &state.trigger {
        serde_json::Value::Object(m) => m.keys().next().unwrap_or(&"unknown".to_string()).clone(),
        _ => "snapshot".to_string(),
    };

    output.push_str(&format!(
        "{}╔══ Game: {} │ Tick: {} │ Trigger: {} ══╗{}\n",
        if config.use_color { colors::BOLD } else { "" },
        state.game_id,
        state.tick,
        trigger_str,
        if config.use_color { colors::RESET } else { "" }
    ));

    // Top border
    output.push_str("║");
    for _ in 0..config.width {
        output.push('═');
    }
    output.push_str("║\n");

    // Grid
    for row in &grid {
        output.push_str("║");
        for (ch, color) in row {
            if config.use_color && !color.is_empty() {
                output.push_str(color);
                output.push(*ch);
                output.push_str(colors::RESET);
            } else {
                output.push(*ch);
            }
        }
        output.push_str("║\n");
    }

    // Bottom border
    output.push_str("║");
    for _ in 0..config.width {
        output.push('═');
    }
    output.push_str("║\n");

    // Legend
    if config.show_legend {
        output.push_str("╠══ LEGEND ");
        for _ in 0..(config.width - 10) {
            output.push('═');
        }
        output.push_str("╣\n");

        // Unit symbols
        output.push_str("║ ");
        if config.use_color {
            output.push_str(&format!("{}i{}=Infantry ", colors::BLUE, colors::RESET));
            output.push_str(&format!("{}r{}=Ranger ", colors::BLUE, colors::RESET));
            output.push_str(&format!("{}s{}=Scout ", colors::BLUE, colors::RESET));
            output.push_str(&format!("{}I{}=Infantry ", colors::YELLOW, colors::RESET));
            output.push_str(&format!("{}R{}=Ranger ", colors::YELLOW, colors::RESET));
            output.push_str(&format!("{}#{}=HQ", colors::BOLD, colors::RESET));
        } else {
            output.push_str(
                "i/I=Infantry r/R=Ranger s/S=Scout #=HQ (lower=Continuity UPPER=Collegium)",
            );
        }
        let legend_len = if config.use_color { 60 } else { 70 };
        for _ in 0..config.width.saturating_sub(legend_len) {
            output.push(' ');
        }
        output.push_str("║\n");

        // Faction counts
        output.push_str("║ ");
        for (faction, (total, damaged, _)) in &faction_counts {
            let color = if config.use_color {
                faction_color(faction)
            } else {
                ""
            };
            let reset = if config.use_color { colors::RESET } else { "" };
            output.push_str(&format!(
                "{}{}{}:{} units ({} damaged) ",
                color, faction, reset, total, damaged
            ));
        }
        for _ in 0..20 {
            output.push(' ');
        }
        output.push_str("║\n");
    }

    // Footer
    output.push_str("╚");
    for _ in 0..config.width {
        output.push('═');
    }
    output.push_str("╝\n");

    output
}

/// Render battle summary comparing two game states.
pub fn render_battle_progress(
    before: &ScreenshotState,
    after: &ScreenshotState,
    config: &AsciiConfig,
) -> String {
    let mut output = String::new();

    // Count units per faction
    let count_units = |state: &ScreenshotState| -> HashMap<String, u32> {
        let mut counts = HashMap::new();
        for unit in &state.units {
            *counts.entry(unit.faction.clone()).or_insert(0) += 1;
        }
        counts
    };

    let before_counts = count_units(before);
    let after_counts = count_units(after);

    output.push_str(&format!(
        "\n{}Battle Progress: Tick {} → {}{}\n",
        if config.use_color { colors::BOLD } else { "" },
        before.tick,
        after.tick,
        if config.use_color { colors::RESET } else { "" }
    ));

    output.push_str("┌────────────────┬──────────┬──────────┬──────────┐\n");
    output.push_str("│ Faction        │ Before   │ After    │ Change   │\n");
    output.push_str("├────────────────┼──────────┼──────────┼──────────┤\n");

    let mut all_factions: Vec<_> = before_counts
        .keys()
        .chain(after_counts.keys())
        .cloned()
        .collect();
    all_factions.sort();
    all_factions.dedup();

    for faction in all_factions {
        let b = before_counts.get(&faction).copied().unwrap_or(0);
        let a = after_counts.get(&faction).copied().unwrap_or(0);
        let change = a as i32 - b as i32;

        let change_str = if change > 0 {
            format!("+{}", change)
        } else {
            change.to_string()
        };

        let change_color = if config.use_color {
            if change > 0 {
                colors::GREEN
            } else if change < 0 {
                colors::RED
            } else {
                colors::GRAY
            }
        } else {
            ""
        };

        let reset = if config.use_color { colors::RESET } else { "" };

        output.push_str(&format!(
            "│ {:<14} │ {:>8} │ {:>8} │ {}{:>8}{} │\n",
            faction, b, a, change_color, change_str, reset
        ));
    }

    output.push_str("└────────────────┴──────────┴──────────┴──────────┘\n");

    output
}

/// Load and visualize screenshots from a directory.
pub fn visualize_game_folder(path: &Path, config: &AsciiConfig) -> std::io::Result<String> {
    let mut output = String::new();

    // Find all JSON files in the directory
    let mut snapshots: Vec<(u64, ScreenshotState)> = Vec::new();

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let file_path = entry.path();

        if file_path.extension().map_or(false, |e| e == "json") {
            if let Ok(state) = ScreenshotState::load(&file_path) {
                snapshots.push((state.tick, state));
            }
        }
    }

    // Sort by tick
    snapshots.sort_by_key(|(tick, _)| *tick);

    if snapshots.is_empty() {
        output.push_str("No screenshots found in directory.\n");
        return Ok(output);
    }

    // Render each snapshot
    for (_, state) in &snapshots {
        output.push_str(&render_ascii(state, config));
        output.push('\n');
    }

    // Show battle progress between first and last
    if snapshots.len() >= 2 {
        let first = &snapshots[0].1;
        let last = &snapshots[snapshots.len() - 1].1;
        output.push_str(&render_battle_progress(first, last, config));
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_char() {
        assert_eq!(unit_char("infantry", "continuity"), 'i');
        assert_eq!(unit_char("infantry", "collegium"), 'I');
        assert_eq!(unit_char("ranger", "continuity"), 'r');
        assert_eq!(unit_char("ranger", "Collegium"), 'R');
    }

    #[test]
    fn test_render_empty_state() {
        let state = ScreenshotState {
            tick: 100,
            game_id: "test".to_string(),
            trigger: serde_json::json!({"test": true}),
            camera: serde_json::json!({}),
            units: vec![],
            buildings: vec![],
            projectiles: vec![],
            effects: vec![],
            map_bounds: [256, 256],
            fog_of_war: None,
        };

        let config = AsciiConfig {
            width: 40,
            height: 12,
            use_color: false,
            ..Default::default()
        };

        let output = render_ascii(&state, &config);
        assert!(output.contains("test"));
        assert!(output.contains("100"));
    }

    #[test]
    fn test_render_with_units() {
        let state = ScreenshotState {
            tick: 500,
            game_id: "battle_test".to_string(),
            trigger: serde_json::json!({"MajorBattle": {"tick": 500}}),
            camera: serde_json::json!({}),
            units: vec![
                UnitSnapshot {
                    entity_id: 1,
                    kind: "infantry".to_string(),
                    faction: "continuity".to_string(),
                    position: [50.0, 128.0],
                    rotation: 0.0,
                    health_percent: 1.0,
                    animation_state: "idle".to_string(),
                    animation_frame: 0,
                    is_selected: false,
                    current_action: None,
                },
                UnitSnapshot {
                    entity_id: 2,
                    kind: "infantry".to_string(),
                    faction: "collegium".to_string(),
                    position: [200.0, 128.0],
                    rotation: 0.0,
                    health_percent: 0.5,
                    animation_state: "attacking".to_string(),
                    animation_frame: 2,
                    is_selected: false,
                    current_action: Some("attack".to_string()),
                },
            ],
            buildings: vec![],
            projectiles: vec![],
            effects: vec![],
            map_bounds: [256, 256],
            fog_of_war: None,
        };

        let config = AsciiConfig {
            width: 60,
            height: 16,
            use_color: false,
            show_legend: true,
            show_health: true,
        };

        let output = render_ascii(&state, &config);
        assert!(output.contains("battle_test"));
        assert!(output.contains("500"));
        // Should contain unit markers
        assert!(output.contains('i') || output.contains('I'));
    }
}
