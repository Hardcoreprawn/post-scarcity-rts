//! Faction data loading for headless testing.
//!
//! Loads faction definitions from RON files for use in automated testing.
//! This allows tests to use real faction-specific units instead of generic hardcoded units.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use rts_core::data::{BuildingData, FactionData, UnitData};
use rts_core::factions::FactionId;

/// Registry holding loaded faction data for headless testing.
#[derive(Debug, Clone, Default)]
pub struct FactionRegistry {
    factions: HashMap<FactionId, FactionData>,
}

impl FactionRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            factions: HashMap::new(),
        }
    }

    /// Load faction data from a RON file.
    pub fn load_from_file(&mut self, path: &Path) -> Result<FactionId, FactionLoadError> {
        let content = fs::read_to_string(path)
            .map_err(|e| FactionLoadError::IoError(path.display().to_string(), e.to_string()))?;

        let data: FactionData = ron::from_str(&content)
            .map_err(|e| FactionLoadError::ParseError(path.display().to_string(), e.to_string()))?;

        let id = data.id;
        self.factions.insert(id, data);
        Ok(id)
    }

    /// Load all factions from a directory.
    pub fn load_from_directory(&mut self, dir: &Path) -> Result<Vec<FactionId>, FactionLoadError> {
        let mut loaded = Vec::new();

        if !dir.exists() {
            return Err(FactionLoadError::DirectoryNotFound(
                dir.display().to_string(),
            ));
        }

        for entry in fs::read_dir(dir)
            .map_err(|e| FactionLoadError::IoError(dir.display().to_string(), e.to_string()))?
        {
            let entry = entry
                .map_err(|e| FactionLoadError::IoError(dir.display().to_string(), e.to_string()))?;
            let path = entry.path();

            if path.extension().map(|e| e == "ron").unwrap_or(false) {
                match self.load_from_file(&path) {
                    Ok(id) => loaded.push(id),
                    Err(e) => {
                        tracing::warn!("Failed to load faction from {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(loaded)
    }

    /// Get faction data by ID.
    pub fn get(&self, id: FactionId) -> Option<&FactionData> {
        self.factions.get(&id)
    }

    /// Get a unit definition from a faction.
    pub fn get_unit(&self, faction: FactionId, unit_id: &str) -> Option<&UnitData> {
        self.factions
            .get(&faction)
            .and_then(|f| f.get_unit(unit_id))
    }

    /// Get a building definition from a faction.
    pub fn get_building(&self, faction: FactionId, building_id: &str) -> Option<&BuildingData> {
        self.factions
            .get(&faction)
            .and_then(|f| f.get_building(building_id))
    }

    /// Get all units for a faction at a specific tier.
    pub fn units_at_tier(&self, faction: FactionId, tier: u8) -> Vec<&UnitData> {
        self.factions
            .get(&faction)
            .map(|f| f.units_at_tier(tier).collect())
            .unwrap_or_default()
    }

    /// Get all combatant units for a faction (units with combat stats).
    pub fn combatant_units(&self, faction: FactionId) -> Vec<&UnitData> {
        self.factions
            .get(&faction)
            .map(|f| f.units.iter().filter(|u| u.is_combatant()).collect())
            .unwrap_or_default()
    }

    /// Get harvester units for a faction.
    pub fn harvester_units(&self, faction: FactionId) -> Vec<&UnitData> {
        self.factions
            .get(&faction)
            .map(|f| f.units.iter().filter(|u| u.has_tag("harvester")).collect())
            .unwrap_or_default()
    }

    /// Get a unit by its role tag (e.g., "infantry", "harvester", "tank").
    /// This allows strategies to use generic role names that map to faction-specific units.
    /// Returns the first matching unit at the lowest tier if multiple units have the same tag.
    pub fn get_unit_by_role(&self, faction: FactionId, role: &str) -> Option<&UnitData> {
        self.factions.get(&faction).and_then(|f| {
            f.units
                .iter()
                .filter(|u| u.has_tag(role))
                .min_by_key(|u| u.tier)
        })
    }

    /// Check if a faction is loaded.
    pub fn has_faction(&self, id: FactionId) -> bool {
        self.factions.contains_key(&id)
    }

    /// Get all loaded faction IDs.
    pub fn loaded_factions(&self) -> Vec<FactionId> {
        self.factions.keys().copied().collect()
    }

    /// Get the number of loaded factions.
    pub fn faction_count(&self) -> usize {
        self.factions.len()
    }
}

/// Errors that can occur during faction loading.
#[derive(Debug, Clone)]
pub enum FactionLoadError {
    /// Failed to read file.
    IoError(String, String),
    /// Failed to parse RON.
    ParseError(String, String),
    /// Directory not found.
    DirectoryNotFound(String),
}

impl std::fmt::Display for FactionLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(path, msg) => write!(f, "IO error reading '{}': {}", path, msg),
            Self::ParseError(path, msg) => write!(f, "Parse error in '{}': {}", path, msg),
            Self::DirectoryNotFound(path) => write!(f, "Directory not found: {}", path),
        }
    }
}

impl std::error::Error for FactionLoadError {}

/// Resolve the default faction data directory.
///
/// Looks for faction RON files in standard locations:
/// 1. `./crates/rts_game/assets/data/factions/` (repo root)
/// 2. `./assets/data/factions/` (running from rts_game)
/// 3. Environment variable `RTS_FACTION_DATA_DIR`
pub fn default_faction_data_dir() -> Option<std::path::PathBuf> {
    // Check environment variable first
    if let Ok(dir) = std::env::var("RTS_FACTION_DATA_DIR") {
        let path = std::path::PathBuf::from(dir);
        if path.exists() {
            return Some(path);
        }
    }

    // Check standard locations
    let candidates = [
        "crates/rts_game/assets/data/factions",
        "assets/data/factions",
        "../rts_game/assets/data/factions",
    ];

    for candidate in &candidates {
        let path = std::path::PathBuf::from(candidate);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

/// Load all available factions from the default directory.
pub fn load_all_factions() -> Result<FactionRegistry, FactionLoadError> {
    let dir = default_faction_data_dir()
        .ok_or_else(|| FactionLoadError::DirectoryNotFound("faction data directory".to_string()))?;

    let mut registry = FactionRegistry::new();
    registry.load_from_directory(&dir)?;
    Ok(registry)
}

/// Load all factions from a specified directory path.
pub fn load_factions_from_path<P: AsRef<Path>>(
    path: P,
) -> Result<FactionRegistry, FactionLoadError> {
    let mut registry = FactionRegistry::new();
    registry.load_from_directory(path.as_ref())?;
    Ok(registry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_faction_registry_new() {
        let registry = FactionRegistry::new();
        assert!(registry.loaded_factions().is_empty());
    }

    #[test]
    fn test_default_faction_dir_resolution() {
        // This test may pass or fail depending on working directory
        let dir = default_faction_data_dir();
        // Just check it doesn't panic
        if let Some(path) = dir {
            assert!(path.exists());
        }
    }
}
