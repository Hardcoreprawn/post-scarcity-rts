//! Utility functions for working with unit data.

/// Check if unit data indicates a ranged unit type.
///
/// Ranged units are identified by having either "ranged" or "ranger" tags
/// in their unit data.
pub fn is_ranged_unit(unit_data: &rts_core::data::UnitData) -> bool {
    unit_data.has_tag("ranged") || unit_data.has_tag("ranger")
}
