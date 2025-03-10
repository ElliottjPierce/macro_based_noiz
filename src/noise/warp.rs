//! Allows domain warping.

use std::ops::{
    Add,
    AddAssign,
    Mul,
};

use super::{
    NoiseOp,
    NoiseType,
};

/// A mode for `Warp` that only warps the current input.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Single;

/// A mode for `Warp` that warps the current input as a reference, such that future warps build on
/// it.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Compounding;

/// Warps its input via a [`NoiseOp`] of type `T`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Warp<T, M = Single> {
    /// The noise that does the warping.
    pub noise: T,
    /// The strength of the warp.
    pub strength: f32,
    /// The warp mode.
    pub mode: M,
}

impl<T: Default, M: Default> Default for Warp<T, M> {
    fn default() -> Self {
        Self {
            noise: T::default(),
            strength: 1.0,
            mode: M::default(),
        }
    }
}

impl<I: NoiseType + Copy + Add<Output = I> + Mul<f32, Output = I>, N: NoiseOp<I, Output = I>>
    NoiseOp<I> for Warp<N, Single>
{
    type Output = I;

    #[inline]
    fn get(&self, input: I) -> Self::Output {
        input + self.noise.get(input) * self.strength
    }
}

impl<'a, I: NoiseType + Copy + AddAssign + Mul<f32, Output = I>, N: NoiseOp<I, Output = I>>
    NoiseOp<&'a mut I> for Warp<N, Compounding>
{
    type Output = &'a mut I;

    #[inline]
    fn get(&self, input: &'a mut I) -> Self::Output {
        *input += self.noise.get(*input) * self.strength;
        input
    }
}
