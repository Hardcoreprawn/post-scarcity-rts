//! Fixed-point math utilities for deterministic simulation.
//!
//! All game simulation uses fixed-point arithmetic to ensure
//! deterministic behavior across platforms. Floating-point
//! operations can produce different results on different CPUs.

use fixed::types::I32F32;
use serde::{Deserialize, Serialize};

/// Fixed-point number type for all simulation math.
///
/// Uses 32 bits for integer part and 32 bits for fractional part.
/// Range: approximately -2,147,483,648 to 2,147,483,647
/// Precision: approximately 0.00000000023
pub type Fixed = I32F32;

/// Fixed-point 2D vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Vec2Fixed {
    /// X coordinate.
    #[serde(with = "fixed_serde")]
    pub x: Fixed,
    /// Y coordinate.
    #[serde(with = "fixed_serde")]
    pub y: Fixed,
}

/// Serde support for fixed-point numbers.
///
/// Serializes fixed-point numbers as their raw bit representation (i64)
/// to preserve exact precision across serialization boundaries.
pub mod fixed_serde {
    use super::Fixed;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Serialize a fixed-point number as its raw bit representation.
    pub fn serialize<S>(value: &Fixed, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as raw bits to preserve exact value
        value.to_bits().serialize(serializer)
    }

    /// Deserialize a fixed-point number from its raw bit representation.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Fixed, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bits = i64::deserialize(deserializer)?;
        Ok(Fixed::from_bits(bits))
    }
}

impl Vec2Fixed {
    /// Create a new fixed-point vector.
    #[must_use]
    pub const fn new(x: Fixed, y: Fixed) -> Self {
        Self { x, y }
    }

    /// Zero vector.
    pub const ZERO: Self = Self {
        x: Fixed::ZERO,
        y: Fixed::ZERO,
    };

    /// Calculate squared distance (avoids sqrt for comparisons).
    #[must_use]
    pub fn distance_squared(self, other: Self) -> Fixed {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    /// Calculate Manhattan distance (faster, good for pathfinding heuristics).
    #[must_use]
    pub fn manhattan_distance(self, other: Self) -> Fixed {
        let dx = if self.x > other.x {
            self.x - other.x
        } else {
            other.x - self.x
        };
        let dy = if self.y > other.y {
            self.y - other.y
        } else {
            other.y - self.y
        };
        dx + dy
    }
}

impl std::ops::Add for Vec2Fixed {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub for Vec2Fixed {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec2_distance_squared() {
        let a = Vec2Fixed::new(Fixed::from_num(3), Fixed::from_num(0));
        let b = Vec2Fixed::new(Fixed::from_num(0), Fixed::from_num(4));
        let dist_sq = a.distance_squared(b);
        // 3² + 4² = 25
        assert_eq!(dist_sq, Fixed::from_num(25));
    }

    #[test]
    fn test_fixed_determinism() {
        // Same operations must produce identical results
        let a = Fixed::from_num(1) / Fixed::from_num(3);
        let b = Fixed::from_num(1) / Fixed::from_num(3);
        assert_eq!(a, b);

        // Multiplication must be deterministic
        let result1 = a * Fixed::from_num(7);
        let result2 = b * Fixed::from_num(7);
        assert_eq!(result1, result2);
    }
}
