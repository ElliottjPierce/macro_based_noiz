//! This module allows factional brownian motion (fbm) noise.

use std::marker::PhantomData;

use super::{
    NoiseOp,
    NoiseType,
};

/// Signifies that this type can be the result of an fbm octave.
pub trait FbmOctaveResult: NoiseType {
    /// Scales this result by some octave `contribution` in (0,1).
    /// Usually this is just multiplication.
    fn fit_contribution(&mut self, contribution: f32);
}

/// Marks this noise type as being able to be used as an octave of fbm noise.
pub trait FbmOctaveNoise<I: NoiseType, D>: NoiseOp<I, Output: FbmOctaveResult> {
    /// This creates the octave based on some input data.
    fn create_octave(data: D) -> Self;
}

/// Allows this type to generate fbm octaves.
pub trait FbmOctaveGenerator<D> {
    /// Gets the next octave initializer.
    fn get_octave(&self) -> D;
    /// Updates self to prepare the next octave.
    fn progress_octave(&mut self);
}

/// Represents settings that can be used to make fmb.
pub trait FbmSettings<D> {
    /// Uses these settings to construct an [`FbmOctaveGenerator`]
    fn get_generator(&self, octaves: u32) -> impl FbmOctaveGenerator<D>;
}

/// Stores an octave of fbm for some [`FbmOctaveNoise`], `D`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FbmOctave<N> {
    noise: N,
    /// The octave's contribution in (0,1)
    contribution: f32,
}

impl<I: NoiseType, N: NoiseOp<I, Output: FbmOctaveResult>> NoiseOp<I> for FbmOctave<N> {
    type Output = N::Output;

    #[inline]
    fn get(&self, input: I) -> Self::Output {
        let mut result = self.noise.get(input);
        result.fit_contribution(self.contribution);
        result
    }
}

/// Fbm noise itself.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FBM<T, F>(T, PhantomData<F>);

impl<
    D,
    I: NoiseType + Clone,
    N0: FbmOctaveNoise<I, D>,
    N1: FbmOctaveNoise<I, D, Output = N0::Output>,
> NoiseOp<I> for FBM<(FbmOctave<N0>, FbmOctave<N1>), D>
{
    type Output = [N0::Output; 2];

    #[inline]
    fn get(&self, input: I) -> Self::Output {
        [self.0.0.get(input.clone()), self.0.1.get(input.clone())]
    }
}
