//! This module implements perlin noise

use std::hint::unreachable_unchecked;

use bevy_math::{
    Vec2,
    Vec3,
    Vec4,
};

use super::{
    NoiseOp,
    NoiseType,
    convert,
    norm::SNorm,
    seeded::Seeded,
    white::White32,
};

/// This trait allows for use as `S` in [`Perlin`].
///
/// # Safety
///
/// For `offset` values where each element is in ±1,
/// [`get_perlin_dot`](PerlinSource::get_perlin_dot) must return a value x such that x *
/// [`NORMALIZING_FACTOR`](PerlinSource::NORMALIZING_FACTOR) / √d is within ±1, where d is the
/// number od dimensions in `I`.
pub unsafe trait PerlinSource<I: NoiseType> {
    /// See [`PerlinSource`]'s safety comment for info.
    const NORMALIZING_FACTOR: f32;

    /// Gets the perlin value for this seed and vector offset.
    /// For use in actual perlin noise, each element of `offset` can be assumed to be in -1..=1.
    fn get_perlin_dot(&self, seed: u32, offset: I) -> f32;
}

/// A simple perlin noise implementation where `S` is the source of the direction vectors.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Perlin<S>(pub S);

macro_rules! impl_perlin {
    ($vec:ty, $sqrt_d:expr) => {
        impl<S: PerlinSource<$vec>> NoiseOp<Seeded<$vec>> for Perlin<S> {
            type Output = f32;

            #[inline]
            fn get(&self, input: Seeded<$vec>) -> Self::Output {
                let dot = self.0.get_perlin_dot(input.meta, input.value);
                dot * S::NORMALIZING_FACTOR / $sqrt_d
            }
        }
    };
}

impl_perlin!(Vec2, core::f32::consts::SQRT_2);
impl_perlin!(Vec3, 1.7320508); // sqrt 3
impl_perlin!(Vec4, 2.0); // sqrt 4

/// A simple perlin noise source from uniquely random values.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeRand;

// SAFETY: The dot product can not be grater than the product of the
// lengths, and one length is normalized and the other one is taken care of by setting
// `NORMALIZING_FACTOR` to 2.0.
unsafe impl PerlinSource<Vec2> for RuntimeRand {
    const NORMALIZING_FACTOR: f32 = 2.0;

    #[inline]
    fn get_perlin_dot(&self, seed: u32, offset: Vec2) -> f32 {
        let vec = Vec2::new(
            convert!(White32(seed).get(0) => SNorm, f32),
            convert!(White32(seed).get(1) => SNorm, f32),
        ) * 128.0; // extra multiplication prevenst len from being Nan because of an approx zero length.
        vec.normalize().dot(offset)
    }
}

// SAFETY: See impl PerlinSource<Vec2> for RuntimeRand
unsafe impl PerlinSource<Vec3> for RuntimeRand {
    const NORMALIZING_FACTOR: f32 = 2.0;

    #[inline]
    fn get_perlin_dot(&self, seed: u32, offset: Vec3) -> f32 {
        let vec = Vec3::new(
            convert!(White32(seed).get(0) => SNorm, f32),
            convert!(White32(seed).get(1) => SNorm, f32),
            convert!(White32(seed).get(2) => SNorm, f32),
        ) * 128.0; // extra multiplication prevenst len from being Nan because of an approx zero length.
        vec.normalize().dot(offset)
    }
}

// SAFETY: See impl PerlinSource<Vec2> for RuntimeRand
unsafe impl PerlinSource<Vec4> for RuntimeRand {
    const NORMALIZING_FACTOR: f32 = 2.0;

    #[inline]
    fn get_perlin_dot(&self, seed: u32, offset: Vec4) -> f32 {
        let vec = Vec4::new(
            convert!(White32(seed).get(0) => SNorm, f32),
            convert!(White32(seed).get(1) => SNorm, f32),
            convert!(White32(seed).get(2) => SNorm, f32),
            convert!(White32(seed).get(3) => SNorm, f32),
        ) * 128.0; // extra multiplication prevenst len from being Nan because of an approx zero length.
        vec.normalize().dot(offset)
    }
}

/// A simple perlin noise source that uses vectors with elemental values of only -1, 0, or 1.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Hashed;

// SAFETY: The dot product can not be grater than the product of the
// lengths, and one length is within √d. So their product is normalized by setting
// `NORMALIZING_FACTOR` to 1.0.
unsafe impl PerlinSource<Vec2> for Hashed {
    const NORMALIZING_FACTOR: f32 = 1.0;

    #[inline]
    fn get_perlin_dot(&self, seed: u32, offset: Vec2) -> f32 {
        let v = offset;
        match seed & 7 {
            0 => v.x + v.y,
            1 => v.x - v.y,
            2 => -v.x + v.y,
            3 => -v.x - v.y,
            4 => v.x,
            5 => -v.x,
            6 => v.y,
            7 => -v.y,
            // SAFETY: We did & 7 above, so there is no way for the value to be > 7.
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

// SAFETY: impl PerlinSource<Vec2> for Cardinal.
unsafe impl PerlinSource<Vec3> for Hashed {
    const NORMALIZING_FACTOR: f32 = 1.0;

    #[inline]
    fn get_perlin_dot(&self, seed: u32, offset: Vec3) -> f32 {
        let mut result = 0.0;
        if seed & 1 > 0 {
            result += offset.x;
        }
        if seed & 2 > 0 {
            result -= offset.x;
        }
        if seed & 4 > 0 {
            result += offset.y;
        }
        if seed & 8 > 0 {
            result -= offset.y;
        }
        if seed & 16 > 0 {
            result += offset.z;
        }
        if seed & 32 > 0 {
            result -= offset.z;
        }
        result
    }
}

// SAFETY: impl PerlinSource<Vec2> for Cardinal.
unsafe impl PerlinSource<Vec4> for Hashed {
    const NORMALIZING_FACTOR: f32 = 1.0;

    #[inline]
    fn get_perlin_dot(&self, seed: u32, offset: Vec4) -> f32 {
        let mut result = 0.0;
        if seed & 1 > 0 {
            result += offset.x;
        }
        if seed & 2 > 0 {
            result -= offset.x;
        }
        if seed & 4 > 0 {
            result += offset.y;
        }
        if seed & 8 > 0 {
            result -= offset.y;
        }
        if seed & 16 > 0 {
            result += offset.z;
        }
        if seed & 32 > 0 {
            result -= offset.z;
        }
        if seed & 64 > 0 {
            result += offset.w;
        }
        if seed & 128 > 0 {
            result -= offset.w;
        }
        result
    }
}
