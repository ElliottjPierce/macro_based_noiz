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
    norm::{
        SNorm,
        UNorm,
    },
    white::White32,
};

/// Offsets a grid point randomly
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Nudge {
    /// the amount the grid point can move
    multiplier: f32,
    /// the seed to use
    seed: White32,
}

/// Offsets grid values for future noise
#[derive(Debug, Clone, PartialEq)]
pub struct WorlyOf<N>(pub N, pub Nudge);

/// Offsets grid values for distance-based noise
#[derive(Debug, Clone, PartialEq)]
pub struct Worly(pub Nudge);

impl Worly {
    /// creates a new [`Worly`]
    pub fn new(seed: u32, shift: f32) -> Self {
        Self(Nudge::new(seed, shift))
    }
}

impl Nudge {
    /// creates a new [`Nudge`]
    pub fn new(seed: u32, shift: f32) -> Self {
        Self {
            multiplier: shift.clamp(-1.0, 1.0) * 0.5,
            seed: White32(seed),
        }
    }
}

impl<T> WorlyOf<T> {
    /// creates a new [`WorlyOf`]
    pub fn new(noise: T, seed: u32, shift: f32) -> Self {
        Self(noise, Nudge::new(seed, shift))
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

        impl<N: NoiseOp<$point>> NoiseOp<$point> for WorlyOf<N> {
            type Output = N::Output;

            #[inline]
            fn get(&self, input: $point) -> Self::Output {
                self.0.get(self.1.get(input))
            }
        }

        impl NoiseOp<$point> for Worly {
            type Output = UNorm;

            #[inline]
            fn get(&self, input: $point) -> Self::Output {
                let mut min = f32::INFINITY;

                for c in input.corners() {
                    let v = self.0.get(c);
                    min = min.min(v.offset.length())
                }

                let max_dist_1d = self.0.multiplier + 0.5;
                let max_dist = (max_dist_1d * max_dist_1d * $d).sqrt();

                UNorm::new_clamped(min / max_dist)
            }
        }
    };
}

impl_nudge!(Vec2, GridPoint2, 2.0, as_vec2);
impl_nudge!(Vec3, GridPoint3, 3.0, as_vec3);
impl_nudge!(Vec4, GridPoint4, 4.0, as_vec4);
