//! This module allows factional brownian motion (fbm) noise.

use super::{
    NoiseType,
    Period,
    conversions::NoiseConverter,
    norm::UNorm,
};

/// Represents the settings of a fbm.
pub trait Settings: Sized {
    /// Progresses the settings for the next octave.
    fn progress(&mut self);

    /// Generates another octave with a particular progression `f`.
    #[inline]
    fn gen_octave_with<O: Octave<Self>>(&mut self, f: impl FnOnce(&mut Self)) -> O {
        let res = O::new(self);
        f(self);
        res
    }

    /// Generates another octave with the default rpgression.
    #[inline]
    fn gen_octave<O: Octave<Self>>(&mut self) -> O {
        self.gen_octave_with(Settings::progress)
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
    fn new(settings: &mut S) -> Self;
    /// Based on the final value of this octave, modify settings.
    fn post_construction(&self, settings: &mut S);
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

    fn new(_settings: &mut UncheckedFbm) -> Self {}

    fn post_construction(&self, _settings: &mut UncheckedFbm) {}
}

/// Traditional fbm settings.
pub struct StandardFbm {
    /// The period of the next octave.
    pub next_period: f64,
    /// The weight of the next octave.
    pub next_weight: f32,
    /// The amount tby which the period is scaled between octaves by default.
    pub octave_scaling: f64,
    /// The amount tby which the weight is scaled between octaves by default.
    pub octave_fall_off: f32,
    total_weight: f32,
}

impl Settings for StandardFbm {
    fn progress(&mut self) {
        self.next_period *= self.octave_scaling;
        self.next_weight *= self.octave_fall_off;
    }
}

impl StandardFbm {
    /// Adds the [`next_weight`](Self::next_weight) to the total.
    /// This is intended to be used to create custom [`Octave`]s for this setting.
    pub fn tally_weight(&mut self) {
        self.tally_weight_manual(self.next_weight);
    }

    /// Adds the `weight` to the total. If the weight does not represent an octave, this can have
    /// unintened consequencess.
    pub fn tally_weight_manual(&mut self, weight: f32) {
        self.total_weight += weight;
    }

    /// Gets the total of weights from [`tally_weight`](Self::tally_weight).
    pub fn tallied_weight(&self) -> f32 {
        self.total_weight
    }

    /// Constructs a new [`StandardFbm`].
    pub fn new(period: Period, octave_scaling: f64, octave_fall_off: f32) -> Self {
        Self {
            next_period: period.0,
            next_weight: 1_000.0,
            octave_scaling,
            octave_fall_off,
            total_weight: 0.0,
        }
    }
}

/// An octave defined by a period and a weight.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StandardOctave {
    /// The period of the octave.
    pub period: Period,
    /// The weight of the octave. The higher the weight, the more pronounced this octave will be
    /// relative to others.
    pub weight: f32,
}

/// Stores the final, normalized contribution of a [`Weighted`] octave.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeightedOctave(pub UNorm);

impl Octave<StandardFbm> for StandardOctave {
    type Stored = WeightedOctave;

    type View = Period;

    fn finalize(self, settings: &StandardFbm) -> (Self::Stored, Self::View) {
        (
            WeightedOctave(UNorm::new_clamped(self.weight / settings.tallied_weight())),
            self.period,
        )
    }

    fn new(settings: &mut StandardFbm) -> Self {
        Self {
            period: Period(settings.next_period),
            weight: settings.next_weight,
        }
    }

    fn post_construction(&self, settings: &mut StandardFbm) {
        settings.tally_weight_manual(self.weight);
    }
}

macro_rules! impl_weighted_accumulator {
    ($pre:ty, $acc:ty, $t:ty, $def:expr, $cmb:ident) => {
        impl<const N: usize, T: NoiseConverter<$t, Input = T>> PreAccumulator<T, WeightedOctave, N>
            for $pre
        {
            type Accumulator = $acc;

            #[inline]
            fn start_accumulate(
                self,
                octave_result: T,
                octave: &WeightedOctave,
            ) -> Self::Accumulator {
                let mut acc = $def;
                acc.accumulate(octave_result, octave);
                acc
            }
        }

        impl<T: NoiseConverter<$t, Input = T>> Accumulator<T, WeightedOctave> for $acc {
            type Final = $t;

            #[inline]
            fn accumulate(&mut self, octave_result: T, octave: &WeightedOctave) {
                let acc = &mut self.0;
                let val = T::convert(octave_result) * octave.0.adapt::<f32>();
                $cmb(acc, val);
            }

            #[inline]
            fn finish(self) -> Self::Final {
                self.0
            }
        }
    };
}

/// A [`PreAccumulator`] that sums together all the octaves, normalized by their weights.
pub struct OctaveSum;

/// The [`Accumulator`] for [`OctaveSum`].
pub struct OctaveSumAccumulator(pub f32);

fn sum(acc: &mut f32, val: f32) {
    *acc += val;
}

impl_weighted_accumulator!(
    OctaveSum,
    OctaveSumAccumulator,
    f32,
    OctaveSumAccumulator(0.0),
    sum
);

/// A [`PreAccumulator`] that multiplies together all the octaves, normalized by their weights.
pub struct OctaveProduct;

/// The [`Accumulator`] for [`OctaveProduct`].
pub struct OctaveProductAccumulator(pub f32);

fn mul(acc: &mut f32, val: f32) {
    *acc *= val;
}

impl_weighted_accumulator!(
    OctaveProduct,
    OctaveProductAccumulator,
    f32,
    OctaveProductAccumulator(1.0),
    mul
);
