//! This module allows factional brownian motion (fbm) noise.

/// Represents the settings of fbm of a certain type of octave.
pub trait FbmSettings {
    /// The type of octave used.
    type Octave;

    /// For `N` octaves, gets each octave.
    fn get_octaves<const N: usize>(self) -> [Self::Octave; N];
}

/// Starts a corresponding [`FbmAccumulator`] for `N` octaves.
pub trait FbmPreAccumulator<R, O, const N: usize> {
    /// The corresponding [`FbmAccumulator`].
    type Accumulator: FbmAccumulator<R, O>;

    /// Begins the accumulation.
    fn start_accumulate(self, octave_result: R, octave: &O) -> Self::Accumulator;
}

/// Represents the accumulation of some result `R` from some octavs `O`.
pub trait FbmAccumulator<R, O> {
    /// The final type of the accumulation.
    type Final;

    /// Brings together an octave and its result.
    fn accumulate(&mut self, octave_result: R, octave: &O);

    /// Completes the accumulationn.
    fn finish(self) -> Self::Final;
}

/// Represents an octave that weights or morphs a value based on its settings.
pub trait ContributoryOctave<T> {
    /// The resulting type.
    type Output;

    /// Fits the value to the output based on the octave's contents.
    fn fit_contribution(&self, value: T) -> Self::Output;
}
