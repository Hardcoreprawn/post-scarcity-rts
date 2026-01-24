# ADR-003: Fixed-Point Math for Determinism

## Status

Accepted

## Context

Lockstep multiplayer requires **bit-perfect determinism**: given identical inputs, all clients must produce identical simulation states. If simulations diverge ("desync"), the game becomes unplayable.

**Floating-point arithmetic is not deterministic across platforms:**

- Different CPUs may use different precision (x87 vs SSE vs AVX)
- Compiler optimizations may reorder operations
- Transcendental functions (sin, cos, sqrt) vary by implementation
- Same operations may produce different results on Intel vs AMD vs ARM

Even tiny differences (1 ULP) compound over thousands of ticks into massive desyncs.

Options considered:

1. **IEEE 754 strict mode**: Force consistent FP behavior via compiler flags
   - Unreliable across compilers, platforms, and libraries
2. **Software floating-point**: Implement FP in software
   - Slow, complex, still need to handle edge cases
3. **Fixed-point arithmetic**: Use integers with implicit decimal point
   - Deterministic by definition, fast, predictable
4. **Integer-only**: Avoid fractions entirely
   - Too limiting for smooth movement and physics

## Decision

We will use **fixed-point arithmetic** for all simulation math via the `fixed` crate.

### Specification

```rust
use fixed::types::I32F32;

// 32 bits integer, 32 bits fractional
// Range: ~-2 billion to ~+2 billion
// Precision: ~0.00000000023 (more than enough for game units)
pub type Fixed = I32F32;
```

### Scope

| Domain | Math Type |
| ------ | --------- |
| Simulation positions | Fixed-point |
| Movement calculations | Fixed-point |
| Damage/health | Integer |
| Resource counts | Integer |
| Pathfinding costs | Fixed-point |
| Rendering transforms | Floating-point (display only) |
| UI layouts | Floating-point (display only) |

### Conversion Rules

```rust
// Simulation → Rendering (one-way, at render time)
let render_x: f32 = simulation_pos.x.to_num();

// NEVER convert float → fixed in simulation code
// All simulation constants must be fixed from the start
const UNIT_SPEED: Fixed = Fixed::from_bits(0x00010000); // 1.0
```

## Consequences

### Positive

- **Guaranteed determinism**: Integer operations are always identical
- **Performance**: Fixed-point is faster than software FP, comparable to hardware FP
- **Simplicity**: No need to track FP modes or worry about compiler flags
- **Debugging**: Easy to compare exact values across clients

### Negative

- **Overflow risk**: Must check for overflow in pathological cases
- **Limited range**: Can't represent extremely large or small values
- **Sqrt/trig complexity**: Need lookup tables or approximations
- **Learning curve**: Team must think in fixed-point

### Mitigations

- Use `I32F32` for generous range (±2 billion with 32 bits of fraction)
- Add overflow checks in debug builds
- Pre-compute lookup tables for common trig operations
- Avoid sqrt where possible (compare squared distances instead)
- Document fixed-point patterns in coding standards

## Implementation

The `rts_core` crate provides:

```rust
// crates/rts_core/src/math.rs
pub type Fixed = fixed::types::I32F32;

pub struct Vec2Fixed {
    pub x: Fixed,
    pub y: Fixed,
}

impl Vec2Fixed {
    // Use squared distance to avoid sqrt
    pub fn distance_squared(self, other: Self) -> Fixed { ... }
}
```

## Related

- [ADR-001: ECS Architecture](./ADR-001-ecs-architecture.md)
- [Coding Standards - Determinism](../../standards/coding-standards.md#determinism)
