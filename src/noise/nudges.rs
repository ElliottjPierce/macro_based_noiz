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
    norm::{
        SNorm,
        UNorm,
    },
    seeded::Seeded,
    white::White32,
};

/// Offsets a grid point randomly, with respect to its surroundings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Nudge<const RESTRICT_POSITIVE: bool> {
    /// the amount the grid point can move
    multiplier: f32,
}

impl<const RESTRICT_POSITIVE: bool> Nudge<RESTRICT_POSITIVE> {
    /// Creates a new [`Nudge`] with this range. Each point will be shifted by half this range.
    /// Points that are nudged will still be in the same order. For example, if integer points a and
    /// b are nudged and a > b, a' > b' (just by a different amount).
    pub fn new_leashed(range: f32) -> Self {
        Self::new_magnitude(range.clamp(0.0, 1.0) * 0.5)
    }

    /// Creates a new [`Nudge`] with this range. Each point will be shifted by this range directly.
    /// Points that are nudged may not still be in the same order. For example, if integer points a
    /// and b are nudged and a > b, a' < b' may be true.
    pub fn new_raw(range: f32) -> Self {
        Self::new_magnitude(range.clamp(0.0, 1.0))
    }

    /// Creates a new leashed [`Nudge`] with this range. Each point will be shifted by up to this
    /// amount with no checks. Use this carefully.
    pub fn new_magnitude(range: f32) -> Self {
        Self { multiplier: range }
    }

    /// Creates a new leashed [`Nudge`] with this full 1.0 range.
    pub fn full_leashed() -> Self {
        Self::new_leashed(1.0)
    }

    /// Creates a new raw [`Nudge`] with this full 1.0 range.
    pub fn full_raw() -> Self {
        Self::new_raw(1.0)
    }

    /// the maximum amount a point will be nudged
    pub fn max_nudge(&self) -> f32 {
        self.multiplier
    }
}

/// easily implements nudging for different types
macro_rules! impl_nudge {
    ($vec:path, $uvec:path, $point:path, $d:literal, $u2f:ident) => {
        impl<const RESTRICT_POSITIVE: bool> NoiseOp<Seeded<$point>> for Nudge<RESTRICT_POSITIVE> {
            type Output = Seeded<$point>;

            #[inline]
            fn get(&self, mut input: Seeded<$point>) -> Self::Output {
                input.value.offset += self.get(input.map_ref(|v| v.base)).value;
                input
            }
        }

        impl<const RESTRICT_POSITIVE: bool> NoiseOp<Seeded<$uvec>> for Nudge<RESTRICT_POSITIVE> {
            type Output = Seeded<$vec>;

            #[inline]
            fn get(&self, input: Seeded<$uvec>) -> Self::Output {
                let raw_shift = input.value.to_array().map(|v| {
                    let seed = White32(input.meta).get(v);
                    if RESTRICT_POSITIVE {
                        seed.adapt::<UNorm>().adapt()
                    } else {
                        seed.adapt::<SNorm>().adapt()
                    }
                });
                let shift = <$vec>::from_array(raw_shift) * self.multiplier;
                Seeded {
                    value: shift,
                    meta: input.meta,
                }
            }
        }
    };
}

impl_nudge!(Vec2, UVec2, GridPoint2, 2.0, as_vec2);
impl_nudge!(Vec3, UVec3, GridPoint3, 3.0, as_vec3);
impl_nudge!(Vec4, UVec4, GridPoint4, 4.0, as_vec4);
