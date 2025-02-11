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
    white::White32,
};

/// Offsets a grid point randomly, with respect to its surroundings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Nudge {
    /// the amount the grid point can move
    multiplier: f32,
    /// the seed to use
    seed: White32,
}

impl Nudge {
    /// creates a new [`Nudge`]
    pub fn new(seed: u32, shift: f32) -> Self {
        Self {
            multiplier: shift.clamp(-1.0, 1.0) * 0.5,
            seed: White32(seed),
        }
    }

    /// the maximum amount a point will be nudged
    pub fn max_nudge(&self) -> f32 {
        self.multiplier.abs()
    }
}

/// easily implements nudging for different types
macro_rules! impl_nudge {
    ($vec:path, $uvec:path, $point:path, $d:literal, $u2f:ident) => {
        impl NoiseOp<$point> for Nudge {
            type Output = $point;

            #[inline]
            fn get(&self, mut input: $point) -> Self::Output {
                input.offset += self.get(input.base);
                input
            }
        }

        impl NoiseOp<$uvec> for Nudge {
            type Output = $vec;

            #[inline]
            fn get(&self, input: $uvec) -> Self::Output {
                let unique = self.seed.get(input);
                let raw_shift = input
                    .to_array()
                    .map(|v| White32(unique).get(v).adapt::<SNorm>().adapt());
                let shift = <$vec>::from_array(raw_shift) * self.multiplier;
                shift
            }
        }
    };
}

impl_nudge!(Vec2, UVec2, GridPoint2, 2.0, as_vec2);
impl_nudge!(Vec3, UVec3, GridPoint3, 3.0, as_vec3);
impl_nudge!(Vec4, UVec4, GridPoint4, 4.0, as_vec4);
