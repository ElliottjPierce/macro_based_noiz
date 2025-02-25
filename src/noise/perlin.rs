//! This module implements perlin noise

use std::{
    hint::unreachable_unchecked,
    marker::PhantomData,
};

use bevy_math::{
    Dir2,
    Vec2,
};

use super::{
    NoiseOp,
    convert,
    norm::SNorm,
    seeded::Seeded,
    white::White32,
};

/// A simple perlin noise implementation where `S` is the source of the direction vectors.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Perlin<S>(pub S);

/// A simple perlin noise source that uses vectors with elemental values of only -1, 0, or 1.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Cardinal;

impl NoiseOp<Seeded<Vec2>> for Perlin<Cardinal> {
    type Output = f32;

    #[inline]
    fn get(&self, input: Seeded<Vec2>) -> Self::Output {
        let v = input.value;
        let dot = match input.meta & 0b111 {
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
        };

        const NORMALIZER: f32 = 2.0 / core::f32::consts::SQRT_2;
        dot * NORMALIZER
    }
}

impl<S: NoiseOp<u32, Output = Dir2>> NoiseOp<Seeded<Vec2>> for Perlin<S> {
    type Output = f32;

    #[inline]
    fn get(&self, input: Seeded<Vec2>) -> Self::Output {
        let dir = self.0.get(input.meta);
        let dot = dir.dot(input.value);
        const NORMALIZER: f32 = 2.0 / core::f32::consts::SQRT_2;
        dot * NORMALIZER
    }
}

/// A simple perlin noise source from uniquely random values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeRand<V>(pub PhantomData<V>);

impl<V> Default for RuntimeRand<V> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl NoiseOp<u32> for RuntimeRand<Dir2> {
    type Output = Dir2;

    #[inline]
    fn get(&self, input: u32) -> Self::Output {
        Dir2::new_unchecked(
            (Vec2::new(
                convert!(White32(input).get(0) => SNorm, f32),
                convert!(White32(input).get(1) => SNorm, f32),
            ) * 2.0)
                .normalize(),
        )
    }
}
