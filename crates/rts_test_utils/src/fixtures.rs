//! Test fixtures and helpers.
//!
//! Pre-built game states and entity configurations
//! for consistent testing.

use fixed::types::I32F32;

/// Create a fixed-point number from an integer.
#[must_use]
pub fn fixed(n: i32) -> I32F32 {
    I32F32::from_num(n)
}

/// Create a fixed-point number from a float (for tests only).
///
/// Note: In real simulation code, never use floats.
/// This is only for convenient test setup.
#[must_use]
pub fn fixed_f(n: f64) -> I32F32 {
    I32F32::from_num(n)
}
