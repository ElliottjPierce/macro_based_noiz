//! This module allows worly noise to be created

use bevy_math::{
    UVec2,
    UVec3,
    UVec4,
    Vec2,
    Vec3,
    Vec4,
};

use super::{
    NoiseOp,
    NoiseType,
    grid::{
        GridPoint2,
        GridPoint3,
        GridPoint4,
    },
    norm::SNorm,
    seeded::Seeded,
    white::White32,
};

/// Offsets a grid point randomly, with respect to its surroundings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Nudge {
    /// the amount the grid point can move
    multiplier: f32,
}

impl Nudge {
    /// Creates a new [`Nudge`] with this range. Each point will be shifted by ± half this range.
    pub fn new(range: f32) -> Self {
        Self {
            multiplier: range.clamp(-1.0, 1.0) * 0.5,
        }
    }

    /// Creates a new [`Nudge`] with this full 1.0 range. Each point will be shifted by ±0.5.
    pub fn full() -> Self {
        Self::new(1.0)
    }

    /// the maximum amount a point will be nudged
    pub fn max_nudge(&self) -> f32 {
        self.multiplier.abs()
    }
}

/// easily implements nudging for different types
macro_rules! impl_nudge {
    ($vec:path, $uvec:path, $point:path, $d:literal, $u2f:ident) => {
        impl NoiseOp<Seeded<$point>> for Nudge {
            type Output = Seeded<$point>;

            #[inline]
            fn get(&self, mut input: Seeded<$point>) -> Self::Output {
                input.value.offset += self.get(input.map_ref(|v| v.base)).value;
                input
            }
        }

        impl NoiseOp<Seeded<$uvec>> for Nudge {
            type Output = Seeded<$vec>;

            #[inline]
            fn get(&self, input: Seeded<$uvec>) -> Self::Output {
                let raw_shift = input
                    .value
                    .to_array()
                    .map(|v| White32(input.seed).get(v).adapt::<SNorm>().adapt());
                let shift = <$vec>::from_array(raw_shift) * self.multiplier;
                Seeded {
                    value: shift,
                    seed: input.seed,
                }
            }
        }
    };
}

impl_nudge!(Vec2, UVec2, GridPoint2, 2.0, as_vec2);
impl_nudge!(Vec3, UVec3, GridPoint3, 3.0, as_vec3);
impl_nudge!(Vec4, UVec4, GridPoint4, 4.0, as_vec4);
