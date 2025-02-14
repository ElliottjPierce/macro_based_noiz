//! Enables noise to have associated metadata

use super::{
    NoiseOp,
    NoiseType,
    conversions::NoiseConverter,
};

/// Represents a type that has been given a some metadata `M`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Associated<T: NoiseType, M> {
    /// the value
    pub value: T,
    /// the metadata for the value
    pub meta: M,
}

impl<T: NoiseType, M> NoiseType for Associated<T, M> {}

impl<T: NoiseType, M> NoiseConverter<T> for Associated<T, M> {
    type Input = Associated<T, M>;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetaOf;

impl<T: NoiseType, M: NoiseType> NoiseOp<Associated<T, M>> for MetaOf {
    type Output = M;

    fn get(&self, input: Associated<T, M>) -> Self::Output {
        input.meta
    }
}

/// A [`NoiseOp`] that maps an [`Associated`] value by its value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapValue<N>(pub N);

impl<T: NoiseType, M: NoiseType, N: NoiseOp<T>> NoiseOp<Associated<T, M>> for MapValue<N> {
    type Output = Associated<N::Output, M>;

    fn get(&self, input: Associated<T, M>) -> Self::Output {
        input.map(|value| self.0.get(value))
    }
}

/// A [`NoiseOp`] that maps an [`Associated`] value by its metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapMeta<N>(pub N);

impl<T: NoiseType, M: NoiseType, N: NoiseOp<M>> NoiseOp<Associated<T, M>> for MapMeta<N> {
    type Output = Associated<T, N::Output>;

    fn get(&self, input: Associated<T, M>) -> Self::Output {
        input.map_meta(|meta| self.0.get(meta))
    }
}
