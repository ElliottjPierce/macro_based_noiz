//! This module allows worly noise to be created

use bevy_math::{
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
    ($vec:path, $point:path, $d:literal, $u2f:ident) => {
        impl NoiseOp<$point> for Nudge {
            type Output = $point;

            #[inline]
            fn get(&self, mut input: $point) -> Self::Output {
                let unique = self.seed.get(input.base);
                let additional = input
                    .base
                    .to_array()
                    .map(|v| White32(unique).get(v).adapt::<SNorm>().adapt());
                let shift = <$vec>::from_array(additional) * self.multiplier;
                input.offset += -((input.base % 2).$u2f()) * shift; // we have to flip the offset every other cell.
                input
            }
        }
    };
}

impl_nudge!(Vec2, GridPoint2, 2.0, as_vec2);
impl_nudge!(Vec3, GridPoint3, 3.0, as_vec3);
impl_nudge!(Vec4, GridPoint4, 4.0, as_vec4);
