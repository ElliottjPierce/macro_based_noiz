//! Allows a noise op to be done to multiple of the same type in sequence.

use std::marker::PhantomData;

use super::NoiseOp;

/// A [`NoiseOp`] that applies a noise operation to sequences of inputs at once.
pub struct Parallel<I, N: NoiseOp<I>> {
    /// The noise being used
    pub noise: N,
    /// marker
    pub phantom: PhantomData<I>,
}

impl<I, N: NoiseOp<I>, const K: usize> NoiseOp<[I; K]> for Parallel<I, N> {
    type Output = [N::Output; K];

    fn get(&self, input: [I; K]) -> Self::Output {
        input.map(|i| self.noise.get(i))
    }
}
