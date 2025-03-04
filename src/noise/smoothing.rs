//! This module allows arrays of noise to be combinned into one in various ways

use super::{
    NoiseOp,
    NoiseType,
    associating::{
        Associated,
        AssociationMapping,
    },
};
use crate::spatial::interpolating::{
    Lerpable,
    MixerFxn,
    mix_2d,
    mix_3d,
    mix_4d,
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
pub struct Lerp;

impl<L: LerpLocatable> NoiseOp<L> for Lerp {
    type Output = LerpReady<L::Extents, L::Location>;

    #[inline]
    fn get(&self, input: L) -> Self::Output {
        let preped = input.prepare_lerp();
        Associated {
            value: LerpValues(preped.value),
            meta: LerpLocation(preped.meta),
        }
    }
}

/// Represents the location between some [`LerpValues`].
/// Ex, in 2d, [f32; 2] might be the `T` for a [`LerpLocation`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct LerpLocation<T>(pub T);

impl<T: NoiseType> NoiseType for LerpLocation<T> {}

/// Represents the set of values to pass to [`Smooth`].
/// Ex, in 2d square, [f32; 4] might be the `T` for a [`LerpValues`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct LerpValues<T>(pub T);

impl<T: NoiseType> NoiseType for LerpValues<T> {}

/// A [`NoiseOp`] that gets the [`LerpLocation`] of a
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct LerpLocationOf;

/// Represents the set of values to pass to [`Smooth`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct LerpValuesOf;

/// Represents some data that is ready to be passed to [`Smooth`].
pub type LerpReady<P, L> = Associated<LerpValues<P>, LerpLocation<L>>;

impl<P: NoiseType, L: NoiseType> NoiseOp<LerpReady<P, L>> for LerpLocationOf {
    type Output = L;

    #[inline]
    fn get(&self, input: LerpReady<P, L>) -> Self::Output {
        input.meta.0
    }
}

impl<P: NoiseType, L> NoiseOp<LerpReady<P, L>> for LerpValuesOf {
    type Output = P;

    #[inline]
    fn get(&self, input: LerpReady<P, L>) -> Self::Output {
        input.value.0
    }
}

impl<P: NoiseType, L: NoiseType> AssociationMapping<LerpLocationOf> for LerpReady<P, L> {
    type MapParam = L;

    type Output<O> = LerpReady<P, O>;

    #[inline]
    fn map_association<O>(self, mapper: impl FnOnce(Self::MapParam) -> O) -> Self::Output<O> {
        LerpReady {
            value: self.value,
            meta: LerpLocation(mapper(self.meta.0)),
        }
    }
}

impl<P: NoiseType, L> AssociationMapping<LerpValuesOf> for LerpReady<P, L> {
    type MapParam = P;

    type Output<O> = LerpReady<O, L>;

    #[inline]
    fn map_association<O>(self, mapper: impl FnOnce(Self::MapParam) -> O) -> Self::Output<O> {
        LerpReady {
            value: LerpValues(mapper(self.value.0)),
            meta: self.meta,
        }
    }
}

/// a noise type to smooth out grid noise
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Smooth<C>(pub C);

/// allows implementing easily Shooth for different types
macro_rules! impl_smooth {
    ($mix:ident, $d:literal, $c:literal) => {
        impl<T: NoiseType + Lerpable + Copy, C: MixerFxn<f32, T>>
            NoiseOp<LerpReady<[T; $c], [f32; $d]>> for Smooth<C>
        {
            type Output = T;

            #[inline]
            fn get(&self, input: LerpReady<[T; $c], [f32; $d]>) -> Self::Output {
                let Associated {
                    value: LerpValues(extents),
                    meta: LerpLocation(location),
                } = input;
                $mix(extents, location, &self.0)
            }
        }
    };
}

impl_smooth!(mix_2d, 2, 4);
impl_smooth!(mix_3d, 3, 8);
impl_smooth!(mix_4d, 4, 16);
