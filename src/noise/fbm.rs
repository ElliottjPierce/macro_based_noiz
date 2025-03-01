//! This module allows factional brownian motion (fbm) noise.

/// Represents the settings of fbm of a certain type of octave.
pub trait FbmSettings {
    /// The type of octave used.
    type Octave;

    /// For `N` octaves, gets each octave.
    fn get_octaves<const N: usize>(self) -> [Self::Octave; N];
}

/// Starts a corresponding [`FbmAccumulator`].
pub trait FbmPreAccumulator<R, O> {
    /// The corresponding [`FbmAccumulator`].
    type Accumulator: FbmAccumulator<R, O>;

    /// Begins the accumulation.
    fn start_accumulate(self, octave_result: R, octave: &O) -> Self::Accumulator;
}

/// Represents the accumulation of some result `R` from some octavs `O`.
pub trait FbmAccumulator<R, O> {
    /// Brings together an octave and its result.
    fn accumulate(&mut self, octave_result: R, octave: &O);
}
