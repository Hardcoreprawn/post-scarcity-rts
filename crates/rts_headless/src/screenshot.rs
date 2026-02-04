//! Screenshot capture system for visual quality review.
//!
//! Supports two modes:
//! - **GPU mode**: Direct capture via Bevy's screenshot manager (requires display)
//! - **State dump mode**: Serialize visual state for offline rendering (headless)
//!
//! Screenshots are triggered at key game moments (battles, expansions, milestones)
//! and tracked in a manifest for review.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Screenshot capture mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ScreenshotMode {
    /// No screenshots
    #[default]
    Disabled,
    /// State dump for offline rendering (headless-compatible)
    StateDump,
    /// Direct GPU capture (requires display)
    Gpu,
}

impl std::fmt::Display for ScreenshotMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScreenshotMode::Disabled => write!(f, "disabled"),
            ScreenshotMode::StateDump => write!(f, "state_dump"),
            ScreenshotMode::Gpu => write!(f, "gpu"),
        }
    }
}

/// Events that trigger screenshot capture
#[derive(Debug, Clone, Event, Serialize, Deserialize)]
pub enum ScreenshotTrigger {
    /// First combat between factions
    FirstContact,
    /// Large engagement (>10 units)
    MajorBattle { unit_count: u32 },
    /// Enemy attacking command center
    BaseUnderAttack { faction: String },
    /// New base established
    ExpansionComplete { faction: String },
    /// Tech tier unlocked
    TechMilestone { faction: String, tech: String },
    /// Game ended
    Victory { winner: String },
    /// Periodic capture
    TimedSnapshot { tick: u64 },
    /// Manual capture request
    Manual { name: String },
}

impl ScreenshotTrigger {
    /// Get a short name for this trigger type
    pub fn name(&self) -> String {
        match self {
            Self::FirstContact => "first_contact".to_string(),
            Self::MajorBattle { .. } => "major_battle".to_string(),
            Self::BaseUnderAttack { faction } => format!("base_attack_{}", faction),
            Self::ExpansionComplete { faction } => format!("expansion_{}", faction),
            Self::TechMilestone { faction, tech } => format!("tech_{}_{}", faction, tech),
            Self::Victory { winner } => format!("victory_{}", winner),
            Self::TimedSnapshot { tick } => format!("snapshot_{}", tick),
            Self::Manual { name } => name.clone(),
        }
    }

    /// Get review prompts for this screenshot type
    pub fn review_prompts(&self) -> Vec<String> {
        match self {
            Self::FirstContact => vec![
                "Are unit silhouettes distinct during combat?".to_string(),
                "Are faction colors clearly distinguishable?".to_string(),
                "Are attack animations visible and impactful?".to_string(),
            ],
            Self::MajorBattle { .. } => vec![
                "Is the battle readable at this zoom level?".to_string(),
                "Are unit types distinguishable in the melee?".to_string(),
                "Are visual effects (explosions, projectiles) clear?".to_string(),
            ],
            Self::BaseUnderAttack { .. } => vec![
                "Is the threat clearly visible to the player?".to_string(),
                "Are buildings distinct from attacking units?".to_string(),
            ],
            Self::ExpansionComplete { .. } => vec![
                "Does the new base look established?".to_string(),
                "Is building placement logical?".to_string(),
            ],
            Self::TechMilestone { .. } => {
                vec!["Are new units/buildings visually distinct from earlier tiers?".to_string()]
            }
            Self::Victory { .. } => vec![
                "Does the final state look decisive?".to_string(),
                "Is the winner's dominance visually apparent?".to_string(),
            ],
            Self::TimedSnapshot { .. } | Self::Manual { .. } => vec![
                "Is the overall visual quality acceptable?".to_string(),
                "Are there any visual glitches or artifacts?".to_string(),
            ],
        }
    }
}

/// Visual state of a unit for offline rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitVisual {
    pub entity_id: u64,
    pub kind: String,
    pub faction: String,
    pub position: (f32, f32),
    pub rotation: f32,
    pub health_percent: f32,
    pub animation_state: String,
    pub animation_frame: u32,
    pub is_selected: bool,
    pub current_action: Option<String>,
}

/// Visual state of a building for offline rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingVisual {
    pub entity_id: u64,
    pub kind: String,
    pub faction: String,
    pub position: (f32, f32),
    pub health_percent: f32,
    pub construction_progress: Option<f32>,
    pub is_producing: bool,
    pub rally_point: Option<(f32, f32)>,
}

/// Visual state of a projectile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectileVisual {
    pub kind: String,
    pub position: (f32, f32),
    pub velocity: (f32, f32),
    pub rotation: f32,
}

/// Visual state of an effect (explosion, muzzle flash, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectVisual {
    pub kind: String,
    pub position: (f32, f32),
    pub scale: f32,
    pub progress: f32,
}

/// Camera state for reproducing the view
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CameraState {
    pub position: (f32, f32, f32),
    pub zoom: f32,
    pub rotation: f32,
}

/// Complete visual state for offline rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualState {
    pub tick: u64,
    pub game_id: String,
    pub trigger: ScreenshotTrigger,
    pub camera: CameraState,
    pub units: Vec<UnitVisual>,
    pub buildings: Vec<BuildingVisual>,
    pub projectiles: Vec<ProjectileVisual>,
    pub effects: Vec<EffectVisual>,
    pub map_bounds: (u32, u32),
    pub fog_of_war: Option<Vec<Vec<bool>>>,
}

impl VisualState {
    /// Create new empty visual state
    pub fn new(game_id: &str, tick: u64, trigger: ScreenshotTrigger) -> Self {
        Self {
            tick,
            game_id: game_id.to_string(),
            trigger,
            camera: CameraState::default(),
            units: Vec::new(),
            buildings: Vec::new(),
            projectiles: Vec::new(),
            effects: Vec::new(),
            map_bounds: (256, 256),
            fog_of_war: None,
        }
    }

    /// Save state dump to JSON file
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(path, json)
    }
}

/// Entry in the screenshot manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotEntry {
    pub filename: String,
    pub tick: u64,
    pub trigger: String,
    pub mode: String,
    pub camera_position: (f32, f32, f32),
    pub visible_unit_count: u32,
    pub visible_building_count: u32,
    pub review_prompts: Vec<String>,
}

/// Manifest tracking all captured screenshots for a game
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScreenshotManifest {
    pub game_id: String,
    pub output_dir: String,
    pub screenshots: Vec<ScreenshotEntry>,
    pub capture_mode: String,
}

impl ScreenshotManifest {
    /// Create new manifest
    pub fn new(game_id: &str, output_dir: &Path, mode: ScreenshotMode) -> Self {
        Self {
            game_id: game_id.to_string(),
            output_dir: output_dir.display().to_string(),
            screenshots: Vec::new(),
            capture_mode: match mode {
                ScreenshotMode::Disabled => "disabled".to_string(),
                ScreenshotMode::StateDump => "state_dump".to_string(),
                ScreenshotMode::Gpu => "gpu".to_string(),
            },
        }
    }

    /// Add a screenshot entry
    pub fn add_entry(&mut self, entry: ScreenshotEntry) {
        self.screenshots.push(entry);
    }

    /// Save manifest to JSON file
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(path, json)
    }

    /// Load manifest from JSON file
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let json = fs::read_to_string(path)?;
        serde_json::from_str(&json).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

/// Configuration for screenshot capture
#[derive(Debug, Clone, Resource)]
pub struct ScreenshotConfig {
    pub mode: ScreenshotMode,
    pub output_dir: PathBuf,
    pub game_id: String,
    /// Interval between timed snapshots (in ticks, 0 = disabled)
    pub timed_interval: u64,
    /// Track first contact to avoid duplicate triggers
    pub first_contact_captured: bool,
}

impl Default for ScreenshotConfig {
    fn default() -> Self {
        Self {
            mode: ScreenshotMode::Disabled,
            output_dir: PathBuf::from("screenshots"),
            game_id: String::new(),
            timed_interval: 7200, // Every 2 minutes at 60 tps
            first_contact_captured: false,
        }
    }
}

impl ScreenshotConfig {
    /// Create new config with specified mode
    pub fn new(mode: ScreenshotMode, output_dir: PathBuf, game_id: &str) -> Self {
        Self {
            mode,
            output_dir,
            game_id: game_id.to_string(),
            timed_interval: 7200,
            first_contact_captured: false,
        }
    }

    /// Get path for a screenshot
    pub fn screenshot_path(&self, name: &str, extension: &str) -> PathBuf {
        self.output_dir
            .join(&self.game_id)
            .join(format!("{}.{}", name, extension))
    }
}

/// Resource holding the current manifest
#[derive(Debug, Clone, Resource, Default)]
pub struct ScreenshotManifestRes(pub ScreenshotManifest);

/// Manager for capturing screenshots during headless execution.
///
/// This is a standalone struct (not a Bevy resource) for use in
/// headless game runners that don't use the full Bevy ECS.
#[derive(Debug)]
pub struct ScreenshotManager {
    config: ScreenshotConfig,
    manifest: ScreenshotManifest,
    first_contact_captured: bool,
    last_timed_capture: u64,
}

impl ScreenshotManager {
    /// Create a new screenshot manager.
    pub fn new(config: ScreenshotConfig) -> Self {
        let manifest =
            ScreenshotManifest::new(&config.game_id, &config.output_dir, config.mode.clone());
        Self {
            config,
            manifest,
            first_contact_captured: false,
            last_timed_capture: 0,
        }
    }

    /// Check if screenshots are enabled.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        !matches!(self.config.mode, ScreenshotMode::Disabled)
    }

    /// Capture visual state for a trigger.
    pub fn capture(
        &mut self,
        state: VisualState,
        trigger: &ScreenshotTrigger,
    ) -> std::io::Result<()> {
        if !self.is_enabled() {
            return Ok(());
        }

        // Handle first contact deduplication
        if matches!(trigger, ScreenshotTrigger::FirstContact) {
            if self.first_contact_captured {
                return Ok(());
            }
            self.first_contact_captured = true;
        }

        // Create directory if needed
        let screenshot_dir = self.config.output_dir.join(&self.config.game_id);
        if !screenshot_dir.exists() {
            std::fs::create_dir_all(&screenshot_dir)?;
        }

        // Generate filename and save state
        let filename = format!("{}_{}.json", trigger.name(), state.tick);
        let path = screenshot_dir.join(&filename);
        state.save(&path)?;

        // Add entry to manifest
        let entry = ScreenshotEntry {
            filename,
            tick: state.tick,
            trigger: trigger.name(),
            mode: self.config.mode.to_string(),
            camera_position: state.camera.position,
            visible_unit_count: state.units.len() as u32,
            visible_building_count: state.buildings.len() as u32,
            review_prompts: trigger.review_prompts(),
        };
        self.manifest.add_entry(entry);

        Ok(())
    }

    /// Check if a timed capture should occur at this tick.
    #[must_use]
    pub fn should_capture_timed(&self, current_tick: u64) -> bool {
        if self.config.timed_interval == 0 {
            return false;
        }
        current_tick >= self.last_timed_capture + self.config.timed_interval
    }

    /// Record a timed capture occurred.
    pub fn record_timed_capture(&mut self, tick: u64) {
        self.last_timed_capture = tick;
    }

    /// Finalize and save the manifest.
    pub fn finalize(&mut self) -> std::io::Result<PathBuf> {
        let screenshot_dir = self.config.output_dir.join(&self.config.game_id);
        if !screenshot_dir.exists() {
            std::fs::create_dir_all(&screenshot_dir)?;
        }

        let manifest_path = screenshot_dir.join("manifest.json");
        self.manifest.save(&manifest_path)?;
        Ok(manifest_path)
    }

    /// Get the current manifest.
    #[must_use]
    pub fn manifest(&self) -> &ScreenshotManifest {
        &self.manifest
    }
}

/// Plugin for screenshot capture
pub struct ScreenshotPlugin;

impl Plugin for ScreenshotPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ScreenshotTrigger>()
            .init_resource::<ScreenshotConfig>()
            .init_resource::<ScreenshotManifestRes>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_trigger_names() {
        assert_eq!(ScreenshotTrigger::FirstContact.name(), "first_contact");
        assert_eq!(
            ScreenshotTrigger::MajorBattle { unit_count: 15 }.name(),
            "major_battle"
        );
        assert_eq!(
            ScreenshotTrigger::Victory {
                winner: "continuity".to_string()
            }
            .name(),
            "victory_continuity"
        );
        assert_eq!(
            ScreenshotTrigger::TimedSnapshot { tick: 3600 }.name(),
            "snapshot_3600"
        );
    }

    #[test]
    fn test_review_prompts() {
        let prompts = ScreenshotTrigger::FirstContact.review_prompts();
        assert!(!prompts.is_empty());
        assert!(prompts[0].contains("silhouette"));

        let battle_prompts = ScreenshotTrigger::MajorBattle { unit_count: 20 }.review_prompts();
        assert!(!battle_prompts.is_empty());
    }

    #[test]
    fn test_visual_state_creation() {
        let state = VisualState::new("game_001", 1000, ScreenshotTrigger::FirstContact);
        assert_eq!(state.tick, 1000);
        assert_eq!(state.game_id, "game_001");
        assert!(state.units.is_empty());
        assert!(state.buildings.is_empty());
    }

    #[test]
    fn test_visual_state_save_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("state.json");

        let mut state = VisualState::new("test_game", 500, ScreenshotTrigger::FirstContact);
        state.units.push(UnitVisual {
            entity_id: 1,
            kind: "infantry".to_string(),
            faction: "continuity".to_string(),
            position: (100.0, 200.0),
            rotation: 0.5,
            health_percent: 0.8,
            animation_state: "idle".to_string(),
            animation_frame: 0,
            is_selected: false,
            current_action: Some("moving".to_string()),
        });

        state.save(&path).unwrap();
        assert!(path.exists());

        let loaded: VisualState =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(loaded.tick, 500);
        assert_eq!(loaded.units.len(), 1);
        assert_eq!(loaded.units[0].kind, "infantry");
    }

    #[test]
    fn test_manifest_save_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("manifest.json");

        let mut manifest =
            ScreenshotManifest::new("game_001", dir.path(), ScreenshotMode::StateDump);
        manifest.add_entry(ScreenshotEntry {
            filename: "first_contact.json".to_string(),
            tick: 1000,
            trigger: "first_contact".to_string(),
            mode: "state_dump".to_string(),
            camera_position: (128.0, 128.0, 50.0),
            visible_unit_count: 10,
            visible_building_count: 4,
            review_prompts: vec!["Check silhouettes".to_string()],
        });

        manifest.save(&path).unwrap();
        assert!(path.exists());

        let loaded = ScreenshotManifest::load(&path).unwrap();
        assert_eq!(loaded.game_id, "game_001");
        assert_eq!(loaded.screenshots.len(), 1);
        assert_eq!(loaded.screenshots[0].tick, 1000);
    }

    #[test]
    fn test_screenshot_config_paths() {
        let config = ScreenshotConfig::new(
            ScreenshotMode::StateDump,
            PathBuf::from("/output"),
            "game_123",
        );

        let path = config.screenshot_path("battle_001", "json");
        assert!(path.to_string_lossy().contains("game_123"));
        assert!(path.to_string_lossy().contains("battle_001.json"));
    }
}
