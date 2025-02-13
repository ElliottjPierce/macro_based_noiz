//! This module allows arrays of noise to be combinned into one in various ways

use std::marker::PhantomData;

use super::{
    NoiseOp,
    NoiseType,
    grid::{
        GridPoint2,
        GridPoint3,
        GridPoint4,
    },
    interpolating::{
        Lerpable,
        MixerFxn,
        mix_2d,
        mix_3d,
        mix_4d,
    },
};

/// a noise type to smooth out grid noise
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Smooth<I: NoiseType, N: NoiseOp<I>, C: MixerFxn<f32, N::Output>> {
    /// the way we are smoothing
    curve: C,
    /// the noise we are smoothing
    noise: N,
    marker: PhantomData<I>,
}

impl<I: NoiseType, N: NoiseOp<I>, C: MixerFxn<f32, N::Output>> Smooth<I, N, C> {
    /// constructs a new [`Smooth`] with these values
    pub fn new(curve: C, noise: N) -> Self {
        Self {
            curve,
            noise,
            marker: PhantomData,
        }
    }
}

/// allows implementing easily Shooth for different types
macro_rules! impl_smooth {
    ($t:path, $mix:ident, $new:ident) => {
        impl<N: NoiseOp<$t>, C: MixerFxn<f32, N::Output>> NoiseOp<$t> for Smooth<$t, N, C>
        where
            N::Output: Lerpable + Copy,
        {
            type Output = N::Output;

            #[inline]
            fn get(&self, input: $t) -> Self::Output {
                let values = input.corners().map(|c| self.noise.get(c));
                $mix(values, input.offset.to_array(), &self.curve)
            }
        }
    };
}

impl_smooth!(GridPoint2, mix_2d, new_vec2);
impl_smooth!(GridPoint3, mix_3d, new_vec3);
impl_smooth!(GridPoint4, mix_4d, new_vec4);
