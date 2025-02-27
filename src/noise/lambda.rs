//! This module allows unnamed noise types.

use std::marker::PhantomData;

use super::{
    NoiseOp,
    NoiseType,
};

/// A noise operation that has no named type associated with it.
/// `I` and `O` are input and outputs respectively.
/// `D` is any additional data.
/// `T` and `N` are identifiers to differentiate between different `LambdaNoise`.
pub struct LambdaNoise<D, I, O, const N: usize = 0, T = ()> {
    data: D,
    op: fn(&D, I) -> O,
    marker: PhantomData<T>,
}

impl<D, I, O, const N: usize, T> LambdaNoise<D, I, O, N, T> {
    /// Constructs a new [`LambdaNoise`] given its data and operation.
    pub fn new(data: D, op: fn(&D, I) -> O) -> Self {
        Self {
            data,
            op,
            marker: PhantomData,
        }
    }
}

impl<D, I, O: NoiseType, const N: usize, T> NoiseOp<I> for LambdaNoise<D, I, O, N, T> {
    type Output = O;

    #[inline]
    fn get(&self, input: I) -> Self::Output {
        (self.op)(&self.data, input)
    }
}
