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

/// Serde support for `Option<Fixed>`.
///
/// Serializes optional fixed-point numbers via their raw bit representation,
/// preserving `None` as a serialized `None` value.
pub mod option_fixed_serde {
    use super::Fixed;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Serialize an optional fixed-point number.
    pub fn serialize<S>(value: &Option<Fixed>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(v) => v.to_bits().serialize(serializer),
            None => serializer.serialize_none(),
        }
    }

    /// Deserialize an optional fixed-point number.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Fixed>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<i64>::deserialize(deserializer)?;
        Ok(opt.map(Fixed::from_bits))
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

    /// Dot product of two vectors.
    #[must_use]
    pub fn dot(self, other: Self) -> Fixed {
        self.x * other.x + self.y * other.y
    }

    /// Linearly interpolate between two vectors.
    #[must_use]
    pub fn lerp(self, other: Self, t: Fixed) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }

    /// Normalize vector using fixed-point math.
    #[must_use]
    pub fn normalize(self) -> Self {
        let len_sq = self.dot(self);

        if len_sq == Fixed::ZERO {
            return Self::ZERO;
        }

        let len = fixed_sqrt(len_sq);
        if len == Fixed::ZERO {
            return Self::ZERO;
        }

        Self::new(self.x / len, self.y / len)
    }
}

/// Computes the square root of a fixed-point number using binary search.
fn fixed_sqrt(value: Fixed) -> Fixed {
    if value <= Fixed::ZERO {
        return Fixed::ZERO;
    }

    let mut low = Fixed::ZERO;
    let mut high = if value > Fixed::from_num(1) {
        value
    } else {
        Fixed::from_num(1)
    };

    for _ in 0..32 {
        let mid = (low + high) / Fixed::from_num(2);
        let mid_sq = mid.saturating_mul(mid);

        if mid_sq <= value {
            low = mid;
        } else {
            high = mid;
        }
    }

    low
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

    #[test]
    fn test_vec2_dot() {
        let a = Vec2Fixed::new(Fixed::from_num(2), Fixed::from_num(3));
        let b = Vec2Fixed::new(Fixed::from_num(4), Fixed::from_num(-1));
        let dot = a.dot(b);
        assert_eq!(dot, Fixed::from_num(5));
    }

    #[test]
    fn test_vec2_lerp() {
        let a = Vec2Fixed::new(Fixed::from_num(0), Fixed::from_num(0));
        let b = Vec2Fixed::new(Fixed::from_num(10), Fixed::from_num(20));
        let mid = a.lerp(b, Fixed::from_num(0.5));
        assert_eq!(mid, Vec2Fixed::new(Fixed::from_num(5), Fixed::from_num(10)));
    }

    #[test]
    fn test_vec2_normalize() {
        let v = Vec2Fixed::new(Fixed::from_num(3), Fixed::from_num(4));
        let norm = v.normalize();

        // Verify normalization produces unit length (within fixed_sqrt precision)
        // Length squared should be very close to 1
        let len_sq = norm.dot(norm);
        let one = Fixed::from_num(1);
        // Allow tiny epsilon: 1/10000 in fixed-point (no floats!)
        let epsilon = one / Fixed::from_num(10000);
        assert!(
            (len_sq - one).abs() < epsilon,
            "normalized vector length² should be ~1, got {:?}",
            len_sq
        );

        // Verify direction is preserved (x/y ratio matches original 3/4)
        // norm.x * 4 should equal norm.y * 3
        let ratio_diff = (norm.x * Fixed::from_num(4)) - (norm.y * Fixed::from_num(3));
        assert!(
            ratio_diff.abs() < epsilon,
            "direction not preserved: {:?}",
            ratio_diff
        );
    }
}
