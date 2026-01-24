//! Faction data loading system for the game client.
//!
//! Loads faction definitions from RON files and makes them available
//! as Bevy resources. All validation happens at load time.

use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use bevy::prelude::*;
use rts_core::data::FactionData;
use rts_core::factions::FactionId;
use thiserror::Error;

/// Errors that can occur during faction data loading.
#[derive(Debug, Error)]
pub enum DataLoadError {
    /// Failed to read file.
    #[error("Failed to read file '{path}': {source}")]
    IoError {
        /// Path to the file.
        path: String,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },

    /// Failed to parse RON file.
    #[error("Failed to parse RON file '{path}': {source}")]
    ParseError {
        /// Path to the file.
        path: String,
        /// Underlying parse error.
        #[source]
        source: ron::error::SpannedError,
    },

    /// Faction data validation failed.
    #[error("Validation failed for faction '{faction}': {errors:?}")]
    ValidationError {
        /// Faction that failed validation.
        faction: String,
        /// List of validation errors.
        errors: Vec<String>,
    },

    /// Duplicate faction ID.
    #[error("Duplicate faction ID: {0:?}")]
    DuplicateFaction(FactionId),

    /// Missing required faction.
    #[error("Required faction not found: {0:?}")]
    MissingFaction(FactionId),

    /// Asset loading error.
    #[error("Asset error: {0}")]
    AssetError(String),
}

/// Result type for data loading operations.
pub type DataLoadResult<T> = Result<T, DataLoadError>;

/// Registry containing all loaded faction data.
///
/// This is a Bevy resource that holds all faction definitions.
/// Insert this into the world during app setup.
#[derive(Resource, Debug, Clone, Default)]
pub struct FactionRegistry {
    /// All loaded faction data, indexed by faction ID.
    factions: HashMap<FactionId, FactionData>,
}

impl FactionRegistry {
    /// Create a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            factions: HashMap::new(),
        }
    }

    /// Register faction data.
    ///
    /// # Errors
    ///
    /// Returns an error if a faction with the same ID is already registered.
    pub fn register(&mut self, data: FactionData) -> DataLoadResult<()> {
        if self.factions.contains_key(&data.id) {
            return Err(DataLoadError::DuplicateFaction(data.id));
        }
        self.factions.insert(data.id, data);
        Ok(())
    }

    /// Get faction data by ID.
    #[must_use]
    pub fn get(&self, id: FactionId) -> Option<&FactionData> {
        self.factions.get(&id)
    }

    /// Get mutable faction data by ID.
    pub fn get_mut(&mut self, id: FactionId) -> Option<&mut FactionData> {
        self.factions.get_mut(&id)
    }

    /// Check if a faction is registered.
    #[must_use]
    pub fn contains(&self, id: FactionId) -> bool {
        self.factions.contains_key(&id)
    }

    /// Get all registered faction IDs.
    pub fn faction_ids(&self) -> impl Iterator<Item = FactionId> + '_ {
        self.factions.keys().copied()
    }

    /// Get all registered faction data.
    pub fn all_factions(&self) -> impl Iterator<Item = &FactionData> {
        self.factions.values()
    }

    /// Get the number of registered factions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.factions.len()
    }

    /// Check if the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.factions.is_empty()
    }

    /// Validate that all required factions are present.
    ///
    /// # Errors
    ///
    /// Returns an error if any required faction is missing.
    pub fn validate_completeness(&self) -> DataLoadResult<()> {
        let required = [
            FactionId::Continuity,
            FactionId::Collegium,
            FactionId::Tinkers,
            FactionId::BioSovereigns,
            FactionId::Zephyr,
        ];

        for faction_id in required {
            if !self.contains(faction_id) {
                return Err(DataLoadError::MissingFaction(faction_id));
            }
        }

        Ok(())
    }
}

/// Load faction data from a RON file.
///
/// # Arguments
///
/// * `path` - Path to the RON file.
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed.
pub fn load_faction_from_file(path: &Path) -> DataLoadResult<FactionData> {
    let path_str = path.display().to_string();

    // Read file contents
    let mut file = std::fs::File::open(path).map_err(|e| DataLoadError::IoError {
        path: path_str.clone(),
        source: e,
    })?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| DataLoadError::IoError {
            path: path_str.clone(),
            source: e,
        })?;

    // Parse RON
    let data: FactionData =
        ron::from_str(&contents).map_err(|e| DataLoadError::ParseError {
            path: path_str.clone(),
            source: e,
        })?;

    // Validate data integrity
    let errors = data.validate();
    if !errors.is_empty() {
        return Err(DataLoadError::ValidationError {
            faction: data.id.short_name().to_string(),
            errors,
        });
    }

    tracing::info!(
        "Loaded faction '{}' with {} units, {} buildings, {} technologies",
        data.id.display_name(),
        data.units.len(),
        data.buildings.len(),
        data.technologies.len()
    );

    Ok(data)
}

/// Load all faction data from a directory.
///
/// Scans the directory for `.ron` files and loads each as faction data.
///
/// # Arguments
///
/// * `dir` - Path to the directory containing faction RON files.
///
/// # Errors
///
/// Returns an error if any file fails to load or validate.
pub fn load_factions_from_directory(dir: &Path) -> DataLoadResult<FactionRegistry> {
    let mut registry = FactionRegistry::new();

    if !dir.exists() {
        tracing::warn!("Faction data directory does not exist: {}", dir.display());
        return Ok(registry);
    }

    let entries = std::fs::read_dir(dir).map_err(|e| DataLoadError::IoError {
        path: dir.display().to_string(),
        source: e,
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| DataLoadError::IoError {
            path: dir.display().to_string(),
            source: e,
        })?;

        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "ron") {
            let data = load_faction_from_file(&path)?;
            registry.register(data)?;
        }
    }

    tracing::info!("Loaded {} factions from {}", registry.len(), dir.display());

    Ok(registry)
}

/// Bevy plugin for loading faction data.
///
/// Loads all faction data from `assets/data/factions/` at startup.
pub struct FactionDataPlugin;

impl Plugin for FactionDataPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FactionRegistry>()
            .add_systems(PreStartup, load_faction_data);
    }
}

/// System that loads faction data at startup.
fn load_faction_data(mut registry: ResMut<FactionRegistry>) {
    // Determine the faction data directory
    let faction_dir = Path::new("assets/data/factions");

    match load_factions_from_directory(faction_dir) {
        Ok(loaded_registry) => {
            *registry = loaded_registry;

            // Log loaded factions
            for faction in registry.all_factions() {
                tracing::debug!(
                    "Faction {} available: {} units, {} buildings",
                    faction.id.short_name(),
                    faction.units.len(),
                    faction.buildings.len()
                );
            }

            // Warn if not all factions are present (expected during development)
            if let Err(e) = registry.validate_completeness() {
                tracing::warn!("Faction data incomplete: {}", e);
            }
        }
        Err(e) => {
            tracing::error!("Failed to load faction data: {}", e);
        }
    }
}

/// Extension trait for accessing faction data from the Bevy world.
pub trait FactionDataExt {
    /// Get faction data for a specific faction.
    fn faction_data(&self, id: FactionId) -> Option<&FactionData>;
}

impl FactionDataExt for World {
    fn faction_data(&self, id: FactionId) -> Option<&FactionData> {
        self.get_resource::<FactionRegistry>()
            .and_then(|reg| reg.get(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rts_core::data::{BuildingData, UnitData};
    use rts_core::math::Fixed;

    fn create_test_faction_data(id: FactionId) -> FactionData {
        FactionData {
            id,
            display_name: format!("faction.{:?}.name", id),
            description: format!("faction.{:?}.desc", id),
            units: vec![UnitData {
                id: "test_unit".to_string(),
                name: "test".to_string(),
                description: "test".to_string(),
                cost: 50,
                build_time: 100,
                health: 100,
                speed: Fixed::from_num(5),
                combat: None,
                tech_required: vec![],
                tier: 1,
                produced_at: vec!["test_building".to_string()],
                tags: vec![],
            }],
            buildings: vec![BuildingData {
                id: "test_building".to_string(),
                name: "test".to_string(),
                description: "test".to_string(),
                cost: 100,
                build_time: 200,
                health: 500,
                produces: vec!["test_unit".to_string()],
                tech_required: vec![],
                provides_tech: vec![],
                tier: 1,
                targetable: true,
                armor: 10,
                vision_range: None,
                tags: vec![],
                is_harvester: false,
                is_main_base: false,
            }],
            technologies: vec![],
            primary_color: [100, 100, 100],
            secondary_color: [200, 200, 200],
            starting_units: vec![],
            starting_buildings: vec![],
            starting_feedstock: 500,
        }
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = FactionRegistry::new();
        let data = create_test_faction_data(FactionId::Continuity);

        registry.register(data.clone()).unwrap();

        assert!(registry.contains(FactionId::Continuity));
        assert!(!registry.contains(FactionId::Collegium));

        let retrieved = registry.get(FactionId::Continuity).unwrap();
        assert_eq!(retrieved.id, FactionId::Continuity);
    }

    #[test]
    fn test_registry_duplicate_error() {
        let mut registry = FactionRegistry::new();
        let data1 = create_test_faction_data(FactionId::Continuity);
        let data2 = create_test_faction_data(FactionId::Continuity);

        registry.register(data1).unwrap();
        let result = registry.register(data2);

        assert!(matches!(result, Err(DataLoadError::DuplicateFaction(_))));
    }

    #[test]
    fn test_registry_validate_completeness() {
        let mut registry = FactionRegistry::new();

        // Incomplete registry should fail validation
        registry
            .register(create_test_faction_data(FactionId::Continuity))
            .unwrap();
        assert!(registry.validate_completeness().is_err());

        // Complete registry should pass
        registry
            .register(create_test_faction_data(FactionId::Collegium))
            .unwrap();
        registry
            .register(create_test_faction_data(FactionId::Tinkers))
            .unwrap();
        registry
            .register(create_test_faction_data(FactionId::BioSovereigns))
            .unwrap();
        registry
            .register(create_test_faction_data(FactionId::Zephyr))
            .unwrap();
        assert!(registry.validate_completeness().is_ok());
    }

    #[test]
    fn test_registry_iteration() {
        let mut registry = FactionRegistry::new();
        registry
            .register(create_test_faction_data(FactionId::Continuity))
            .unwrap();
        registry
            .register(create_test_faction_data(FactionId::Collegium))
            .unwrap();

        assert_eq!(registry.len(), 2);

        let ids: Vec<_> = registry.faction_ids().collect();
        assert!(ids.contains(&FactionId::Continuity));
        assert!(ids.contains(&FactionId::Collegium));
    }
}
