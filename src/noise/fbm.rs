//! This module allows factional brownian motion (fbm) noise.

use super::{
    NoiseOp,
    NoiseType,
};

/// Signifies that this type can be the result of an fbm octave.
pub trait FbmOctaveResult: NoiseType {
    /// The final result type of the octave
    type Result: NoiseType;

    /// Converts this type to [`Result`](FbmOctaveResult::Result) by some `contribution` in (0,1).
    /// Usually this is just multiplication.
    fn fit_contribution(self, contribution: f32) -> Self::Result;
}

/// Marks this noise type as being able to be used as an octave of fbm noise.
pub trait FbmOctaveNoise<I: NoiseType, D>: NoiseOp<I>
where
    Self::Output: FbmOctaveResult,
{
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
pub struct FmbOctave<N> {
    noise: N,
    /// The octave's contribution in (0,1)
    contribution: f32,
}
