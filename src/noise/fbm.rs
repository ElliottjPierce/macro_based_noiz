//! This module allows factional brownian motion (fbm) noise.

use super::norm::UNorm;

/// Represents the settings of a fbm.
pub trait Settings: Sized {
    /// Progresses the settings for the next octave.
    fn progress(&mut self);

    /// Generates another octave with a particular progression `f`.
    fn gen_octave_with<O: Octave<Self>>(&mut self, f: impl FnOnce(&mut Self)) -> O {
        f(self);
        O::new(self)
    }

    /// Generates another octave with the default rpgression.
    fn gen_octave<O: Octave<Self>>(&mut self) -> O {
        self.progress();
        O::new(self)
    }
}

/// Represents an octave of fbm.
pub trait Octave<S: Settings> {
    /// The information stored and made available to the [`Accumulator`].
    type Stored;
    /// The information made available to the octave's operations' construction.
    type View;

    /// Finalizes the octave based on its settings.
    fn finalize(self, settings: &S) -> (Self::Stored, Self::View);
    /// Constructs the octave from its settings.
    fn new(settings: &S) -> Self;
}

/// Starts a corresponding [`Accumulator`] for `N` octaves.
pub trait PreAccumulator<R, O, const N: usize> {
    /// The corresponding [`FbmAccumulator`].
    type Accumulator: Accumulator<R, O>;

    /// Begins the accumulation.
    fn start_accumulate(self, octave_result: R, octave: &O) -> Self::Accumulator;
}

/// Represents the accumulation of some result `R` from some octavs `O`.
pub trait Accumulator<R, O> {
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

/// Fbm that is completely unchecked. This lets you do whatever you want, including custom octaves
/// and layers, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UncheckedFbm;

impl Settings for UncheckedFbm {
    #[inline]
    fn progress(&mut self) {}
}

impl Octave<UncheckedFbm> for () {
    type Stored = Self;

    type View = Self;

    fn finalize(self, _settings: &UncheckedFbm) -> (Self::Stored, Self::View) {
        ((), ())
    }

    fn new(_settings: &UncheckedFbm) -> Self {}
}

// /// An octave defined by some settings `T` and a weight.
// #[derive(Debug, Clone, Copy, PartialEq)]
// pub struct Weighted<T> {
//     /// The settings of the octave.
//     pub settings: T,
//     /// The weight of the octave. The higher the weight, the more pronounced this octave will be
//     /// relative to others.
//     pub weight: f32,
// }

// /// Stores the final, normalized contribution of a [`Weighted`] octave.
// #[derive(Debug, Clone, Copy, PartialEq)]
// pub struct WeightedOctaveStorage(pub UNorm);

// impl<T> Octave for Weighted<T> {
//     type Stored = WeightedOctaveStorage;

//     type View = T;

//     fn split(self) -> (Self::Stored, Self::Settings) {
//         todo!()
//     }
// }
