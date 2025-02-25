//! This module implements perlin noise

use std::marker::PhantomData;

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

impl<S: NoiseOp<u32, Output = Dir2>> NoiseOp<Seeded<Vec2>> for Perlin<S> {
    type Output = f32;

    #[inline]
    fn get(&self, input: Seeded<Vec2>) -> Self::Output {
        let dir = self.0.get(input.meta);
        let dot = dir.dot(input.value);
        const NORMALIZER: f32 = 2.0 / core::f32::consts::SQRT_2;
        (dot * NORMALIZER + 1.0) * 0.5
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
