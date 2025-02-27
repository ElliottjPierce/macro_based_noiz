//! This module allows factional brownian motion (fbm) noise.

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

/// Allows this type to generate fbm octaves.
pub trait FbmOctaveGenerator<D> {
    /// Gets the next octave initializer.
    fn get_octave(&self) -> D;
    /// Gets the weight/influence of this octave.
    fn get_weight(&self) -> f32;
    /// Updates self to prepare the next octave.
    fn progress_octave(&mut self);
}

/// Represents settings that can be used to make fmb.
pub trait FbmSettings<D> {
    /// Uses these settings to construct an [`FbmOctaveGenerator`]
    fn get_generator(&self, octaves: u8) -> impl FbmOctaveGenerator<D>;
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

impl<N> FbmOctave<N> {
    /// constructs a new [`FbmOctave`] where the `contribution` has not yet been normalized.
    #[inline]
    fn new_octave_partial<D>(
        generator: &mut impl FbmOctaveGenerator<D>,
        total_contribution: &mut f32,
    ) -> Self
    where
        N: From<D>,
    {
        let contribution = generator.get_weight();
        *total_contribution += contribution;
        let result = Self {
            noise: generator.get_octave().into(),
            contribution,
        };
        generator.progress_octave();
        result
    }
}

/// Fbm noise itself.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Fbm<T>(T);

impl<
    I: NoiseType + Clone,
    N0: NoiseOp<I, Output: FbmOctaveResult>,
    N1: NoiseOp<I, Output = N0::Output>,
> NoiseOp<I> for Fbm<(FbmOctave<N0>, FbmOctave<N1>)>
{
    type Output = [N0::Output; 2];

    #[inline]
    fn get(&self, input: I) -> Self::Output {
        [self.0.0.get(input.clone()), self.0.1.get(input.clone())]
    }
}

impl<N0, N1> Fbm<(FbmOctave<N0>, FbmOctave<N1>)> {
    /// Constructs a new [`FBM`] given these settings.
    pub fn new_fbm<D, G: FbmOctaveGenerator<D>>(settings: &impl FbmSettings<D>) -> Self
    where
        N0: From<D>,
        N1: From<D>,
    {
        let mut generator = settings.get_generator(2);
        let mut total_contribution = 0.0;
        let mut result = Self((
            FbmOctave::new_octave_partial(&mut generator, &mut total_contribution),
            FbmOctave::new_octave_partial(&mut generator, &mut total_contribution),
        ));
        result.0.0.contribution /= total_contribution;
        result.0.1.contribution /= total_contribution;
        result
    }
}
