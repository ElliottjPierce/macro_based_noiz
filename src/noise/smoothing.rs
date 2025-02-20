//! This module allows arrays of noise to be combinned into one in various ways

use super::{
    NoiseOp,
    NoiseType,
    associating::Associated,
    interpolating::{
        Lerpable,
        MixerFxn,
        mix_2d,
        mix_3d,
        mix_4d,
    },
};

/// A trait that allows this type to have its context of `T` lerped.
pub trait LerpLocatable {
    /// The type storing dimension information. This should usually be a `f32` array the length of
    /// the number of dimensions this will be lerped.
    type Location;
    /// The type storing the contents to be lerped. This should usually be an array with
    /// length 2 ^ number of dimensions this will be lerped.
    type Extents: NoiseType;

    /// prepares this value to be lerped by packaging its extents and location relative to its
    /// dimensions.
    fn prepare_lerp(self) -> Associated<Self::Extents, Self::Location>;
}

/// A noise type that prepares a type to be lerped.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PrepareLerp;

/// a noise type to smooth out grid noise
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Smooth<C>(pub C);

impl<L: LerpLocatable> NoiseOp<L> for PrepareLerp {
    type Output = Associated<L::Extents, L::Location>;

    #[inline]
    fn get(&self, input: L) -> Self::Output {
        input.prepare_lerp()
    }
}

/// allows implementing easily Shooth for different types
macro_rules! impl_smooth {
    ($mix:ident, $d:literal, $c:literal) => {
        impl<T: NoiseType + Lerpable + Copy, C: MixerFxn<f32, T>>
            NoiseOp<Associated<[T; $c], [f32; $d]>> for Smooth<C>
        {
            type Output = T;

            #[inline]
            fn get(&self, input: Associated<[T; $c], [f32; $d]>) -> Self::Output {
                let Associated {
                    value: extents,
                    meta: location,
                } = input;
                $mix(extents, location, &self.0)
            }
        }
    };
}

impl_smooth!(mix_2d, 2, 4);
impl_smooth!(mix_3d, 3, 8);
impl_smooth!(mix_4d, 4, 16);
