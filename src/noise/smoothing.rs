//! This module allows arrays of noise to be combinned into one in various ways

use std::marker::PhantomData;

use super::{
    NoiseOp,
    NoiseType,
    associating::Associated,
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

/// A trait that allows this type to have its context of `T` lerped.
///
/// `Extents` is the type storing the contents to be lerped. This should usually be an array with
/// length 2 ^ number of dimensions this will be lerped.
pub trait LerpLocatable<Extents: NoiseType> {
    /// The type storing dimension information. This should usually be a `f32` array the length of
    /// the number of dimensions this will be lerped.
    type Location;

    /// prepares this value to be lerped by packaging its extents and location relative to its
    /// dimensions.
    fn prepare_lerp(self) -> Associated<Extents, Self::Location>;
}

/// a noise type to smooth out grid noise
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Smooth<C, T> {
    pub curve: C,
    pub marker: PhantomData<T>,
}

/// allows implementing easily Shooth for different types
macro_rules! impl_smooth {
    ($mix:ident, $d:literal, $c:literal) => {
        impl<
            T: NoiseType + Lerpable + Copy,
            L: LerpLocatable<[T; $c], Location = [f32; $d]>,
            C: MixerFxn<f32, T>,
        > NoiseOp<L> for Smooth<C, [T; $d]>
        {
            type Output = T;

            #[inline]
            fn get(&self, input: L) -> Self::Output {
                let Associated {
                    value: extents,
                    meta: location,
                } = input.prepare_lerp();
                $mix(extents, location, &self.curve)
            }
        }
    };
}

impl_smooth!(mix_2d, 2, 4);
impl_smooth!(mix_3d, 3, 8);
impl_smooth!(mix_4d, 4, 16);
