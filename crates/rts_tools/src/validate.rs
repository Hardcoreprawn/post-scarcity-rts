//! Data validation utilities.

use rts_core::error::Result;

/// Validate all RON data files in a directory.
///
/// # Errors
///
/// Returns an error if any data file fails validation.
pub fn validate_data_directory(_path: &std::path::Path) -> Result<()> {
    // TODO: Implement data validation
    Ok(())
}
