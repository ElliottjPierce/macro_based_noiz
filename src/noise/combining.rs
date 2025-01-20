//! This module allows arrays of noise to be combinned into one in various ways

use super::{
    NoiseOp,
    grid::{
        GridPoint2,
        GridPoint3,
        GridPoint4,
        GridPointD2,
        GridPointD3,
        GridPointD4,
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
pub struct Smooth<C, N> {
    /// the way we are smoothing
    curve: C,
    /// the noise we are smoothing
    noise: N,
}

/// allows implementing easily Shooth for different types
macro_rules! impl_smooth {
    ($t:path, $mix:ident, $f:ident, $new:ident) => {
        impl<C: MixerFxn<$f, N::Output>, N: NoiseOp<$t>> NoiseOp<$t> for Smooth<C, N>
        where
            N::Output: Lerpable + Copy,
        {
            type Output = N::Output;

            #[inline]
            fn get(&self, input: $t) -> N::Output {
                let values = input.corners().map(|c| self.noise.get(c));
                $mix(values, input.offset.to_array(), &self.curve)
            }
        }

        impl<C: MixerFxn<$f, N::Output>, N: NoiseOp<$t>> Smooth<C, N>
        where
            N::Output: Lerpable + Copy,
        {
            /// constructs a new [`Smooth`] with these values
            pub fn $new(curve: C, noise: N) -> Self {
                Self { curve, noise }
            }
        }
    };
}

impl_smooth!(GridPoint2, mix_2d, f32, new_vec2);
impl_smooth!(GridPoint3, mix_3d, f32, new_vec3);
impl_smooth!(GridPoint4, mix_4d, f32, new_vec4);
impl_smooth!(GridPointD2, mix_2d, f64, new_dvec2);
impl_smooth!(GridPointD3, mix_3d, f64, new_dvec3);
impl_smooth!(GridPointD4, mix_4d, f64, new_dvec4);
