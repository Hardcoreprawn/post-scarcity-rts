//! # RTS Test Utilities
//!
//! Shared testing utilities for all crates:
//! - Determinism test harness
//! - Fixture spawning helpers
//! - Benchmark scenarios
//! - Property-based testing strategies

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]

pub mod determinism;
pub mod fixtures;

/// Re-export proptest for convenience.
pub use proptest;
