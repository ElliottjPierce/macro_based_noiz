//! Enables noise to have associated metadata

use std::marker::PhantomData;

use super::{
    NoiseOp,
    NoiseType,
    conversions::NoiseConverter,
    merging::{
        Mergeable,
        Merger,
    },
};

/// Represents a type that has been given a some metadata `M`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Associated<T, M> {
    /// the value
    pub value: T,
    /// the metadata for the value
    pub meta: M,
}

/// A trait that allows part of a noise type to be mapped, keeping the rest of it.
pub trait AssociationMapping<T> {
    /// The input type of the mapping function.
    type MapParam;
    /// The output of the mapping operation, where `O` is the output of the mapping function.
    type Output<O>;

    /// Performs the mapping, consuming `self`, and producing the output.
    fn map_association<O>(self, mapper: impl FnOnce(Self::MapParam) -> O) -> Self::Output<O>;
}

impl<T: NoiseType, E, const K: usize> Mergeable for Associated<[T; K], E> {
    type Meta = E;
    type Part = T;

    #[inline]
    fn perform_merge<M: Merger<Self::Part, Self::Meta>>(self, merger: &M) -> M::Output {
        merger.merge(self.value, &self.meta)
    }
}

impl<T: NoiseType, M> NoiseType for Associated<T, M> {}

impl<T: NoiseType, M> NoiseConverter<T> for Associated<T, M> {
    type Input = Associated<T, M>;

    #[inline]
    fn convert(source: Self::Input) -> T {
        source.value
    }
}

impl<T: NoiseType, M> Associated<T, M> {
    /// Maps this value to another, keeping its metadata.
    #[inline]
    pub fn map<O: NoiseType>(self, f: impl FnOnce(T) -> O) -> Associated<O, M> {
        Associated {
            value: f(self.value),
            meta: self.meta,
        }
    }

    /// Maps this value to another, keeping its metadata.
    #[inline]
    pub fn map_ref<O: NoiseType>(&self, f: impl FnOnce(&T) -> O) -> Associated<O, M>
    where
        M: Clone,
    {
        Associated {
            value: f(&self.value),
            meta: self.meta.clone(),
        }
    }

    /// Maps this metadata to another, keeping its value.
    #[inline]
    pub fn map_meta<O: NoiseType>(self, f: impl FnOnce(M) -> O) -> Associated<T, O> {
        Associated {
            meta: f(self.meta),
            value: self.value,
        }
    }

    /// Maps this metadata to another, keeping its value.
    #[inline]
    pub fn map_meta_ref<O: NoiseType>(&self, f: impl FnOnce(&M) -> O) -> Associated<T, O>
    where
        T: Clone,
    {
        Associated {
            meta: f(&self.meta),
            value: self.value.clone(),
        }
    }
}

/// A [`NoiseOp`] that takes only the meta from a [`Associated`] value.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MetaOf;

impl<T: NoiseType, M> AssociationMapping<MetaOf> for Associated<T, M> {
    type MapParam = M;

    type Output<O> = Associated<T, O>;

    fn map_association<O>(self, mapper: impl FnOnce(Self::MapParam) -> O) -> Self::Output<O> {
        Associated {
            value: self.value,
            meta: mapper(self.meta),
        }
    }
}

impl<T: NoiseType, M: NoiseType> NoiseOp<Associated<T, M>> for MetaOf {
    type Output = M;

    #[inline]
    fn get(&self, input: Associated<T, M>) -> Self::Output {
        input.meta
    }
}

/// A [`NoiseOp`] that takes only the value from a [`Associated`] value.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ValueOf;

impl<T: NoiseType, M> AssociationMapping<ValueOf> for Associated<T, M> {
    type MapParam = T;

    type Output<O> = Associated<O, M>;

    fn map_association<O>(self, mapper: impl FnOnce(Self::MapParam) -> O) -> Self::Output<O> {
        Associated {
            meta: self.meta,
            value: mapper(self.value),
        }
    }
}

impl<T: NoiseType, M> NoiseOp<Associated<T, M>> for ValueOf {
    type Output = T;

    #[inline]
    fn get(&self, input: Associated<T, M>) -> Self::Output {
        input.value
    }
}

/// A [`NoiseOp`] that maps an anything implementing [`AssociationMapping`] for `M` with noise.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Mapped<M, N>(pub N, pub PhantomData<M>);

impl<M, N> Mapped<M, N> {
    /// Constructs a new [`Mapped`].
    pub fn new(noise: N) -> Self {
        Self(noise, PhantomData)
    }
}

impl<M, N: NoiseOp<T::MapParam>, T: AssociationMapping<M>> NoiseOp<T> for Mapped<M, N>
where
    T::Output<N::Output>: NoiseType,
{
    type Output = T::Output<N::Output>;

    fn get(&self, input: T) -> Self::Output {
        input.map_association(|input| self.0.get(input))
    }
}
