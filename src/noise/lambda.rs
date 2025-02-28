//! This module allows unnamed noise types.

use std::marker::PhantomData;

use super::{
    NoiseOp,
    NoiseType,
};

/// A noise operation that has no named type associated with it.
/// `I` and `O` are input and outputs respectively.
/// `D` is any additional data.
/// `S` is the source of the data.
/// `T` and `N` are identifiers to differentiate between different `LambdaNoise`.
pub struct LambdaNoise<S, D, I, O, const N: usize = 0, T = ()> {
    data: D,
    op: fn(&D, I) -> O,
    marker: PhantomData<(T, S)>,
}

/// A generic trait for constructing [`LambdaNoise`].
/// This lets us dodge orphan rules!
pub trait LambdaConstructor<S, D, I, O, const N: usize = 0, T = ()> {
    /// Constructs a version of this [`LambdaNoise`]
    fn construct(value: S) -> Self;
}

impl<S, D, I, O, const N: usize, T> From<S> for LambdaNoise<S, D, I, O, N, T>
where
    Self: LambdaConstructor<S, D, I, O, N, T>,
{
    #[inline]
    fn from(value: S) -> Self {
        Self::construct(value)
    }
}

impl<S, D, I, O, const N: usize, T> LambdaNoise<S, D, I, O, N, T> {
    /// Constructs a new [`LambdaNoise`] given its data and operation.
    pub fn new(data: D, op: fn(&D, I) -> O) -> Self {
        Self {
            data,
            op,
            marker: PhantomData,
        }
    }
}

impl<S, D, I, O: NoiseType, const N: usize, T> NoiseOp<I> for LambdaNoise<S, D, I, O, N, T> {
    type Output = O;

    #[inline]
    fn get(&self, input: I) -> Self::Output {
        (self.op)(&self.data, input)
    }
}
