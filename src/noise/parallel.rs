//! Allows a noise op to be done to multiple of the same type in sequence.

use super::NoiseOp;

/// A [`NoiseOp`] that applies a noise operation to sequences of inputs at once.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Parallel<N>(pub N);

impl<I, N: NoiseOp<I>, const K: usize> NoiseOp<[I; K]> for Parallel<N> {
    type Output = [N::Output; K];

    fn get(&self, input: [I; K]) -> Self::Output {
        input.map(|i| self.0.get(i))
    }
}
