//! This module allows factional brownian motion (fbm) noise.

use bevy_math::{
    Vec2,
    Vec3,
    Vec4,
};
use rand::{
    Rng,
    RngCore,
};

use super::{
    NoiseOp,
    NoiseType,
    norm::{
        SNorm,
        UNorm,
    },
    seeded::Seeding,
    white::White32,
};
use crate::rng::NoiseRng;

/// Signifies that this type can be the result of an fbm octave.
pub trait FbmOctaveResult: NoiseType {
    /// Scales this result by some octave `contribution` in (0,1).
    /// Usually this is just multiplication.
    ///
    /// # Safety
    ///
    /// `contribution` must be in (0,1)
    unsafe fn fit_contribution(&mut self, contribution: f32);
}

/// Allows this type to generate fbm octaves.
pub trait FbmOctaveGenerator<D> {
    /// Gets the next octave initializer.
    fn get_octave(&mut self) -> D;
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
        // SAFETY: Self can only be constructed via internal methods which always satisfy safety.
        unsafe { result.fit_contribution(self.contribution) };
        result
    }
}

impl<N> FbmOctave<N> {
    /// constructs a new [`FbmOctave`] where the `contribution` has not yet been normalized.
    #[inline]
    fn new_octave_partial<const _N: usize, D>(
        // the unused _N lets us use repetition in the macro.
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

    /// Normalizes contribution given a total.
    #[inline]
    fn finalize(&mut self, total_contribution: f32) {
        self.contribution = UNorm::new_clamped(self.contribution / total_contribution).adapt();
    }
}

/// Fbm noise itself.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Fbm<T>(T);

macro_rules! impl_fbm {
    ($octaves:expr, $($n:tt = $t:ident),+) => {
        impl<
            I: NoiseType + Clone,
            N0: NoiseOp<I, Output: FbmOctaveResult>,
            $($t: NoiseOp<I, Output = N0::Output>),+
        > NoiseOp<I> for Fbm<(FbmOctave<N0>, $(FbmOctave<$t>),+ )>
        {
            type Output = [N0::Output; $octaves + 1];

            #[inline]
            fn get(&self, input: I) -> Self::Output {
                [$(self.0.$n.get(input.clone())),+ , self.0.0.get(input)]
            }
        }

        impl<N0, $($t),+> Fbm<(FbmOctave<N0>, $(FbmOctave<$t>),+ )> {
            /// Constructs a new [`FBM`] given these settings.
            pub fn new_fbm<D, G: FbmOctaveGenerator<D>>(settings: &impl FbmSettings<D>) -> Self
            where
                N0: From<D>,
                $($t: From<D>),+
            {
                let mut generator = settings.get_generator($octaves + 1);
                let mut total_contribution = 0.0;
                let mut result = Self((
                    FbmOctave::new_octave_partial::<0, _>(&mut generator, &mut total_contribution),
                    $(FbmOctave::new_octave_partial::<$n, _>(&mut generator, &mut total_contribution)),+
                ));
                result.0.0.finalize(total_contribution);
                $(result.0.$n.finalize(total_contribution);)+
                result
            }
        }
    };
}

macro_rules! impl_octave_result_with_mul {
    ($($t:ty),*) => {
        $(impl FbmOctaveResult for $t {
            #[inline]
            unsafe fn fit_contribution(&mut self, contribution: f32) {
                *self *= contribution;
            }
        })*
    };
}

impl_octave_result_with_mul!(f32, Vec2, Vec3, Vec4);

impl<T: FbmOctaveResult, const N: usize> FbmOctaveResult for [T; N] {
    #[inline]
    unsafe fn fit_contribution(&mut self, contribution: f32) {
        for v in self {
            // SAFETY: Caller ensures contribution is in range.
            unsafe {
                v.fit_contribution(contribution);
            }
        }
    }
}

impl FbmOctaveResult for UNorm {
    #[inline]
    unsafe fn fit_contribution(&mut self, contribution: f32) {
        // SAFETY: Caller ensures contribution is in the same range.
        unsafe { *self.get_mut() *= contribution };
    }
}

impl FbmOctaveResult for SNorm {
    #[inline]
    unsafe fn fit_contribution(&mut self, contribution: f32) {
        // SAFETY: Caller ensures contribution is in the same range.
        unsafe { *self.get_mut() *= contribution };
    }
}

#[rustfmt::skip]
mod unformatted {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    impl_fbm!(1, 1=N1);
    impl_fbm!(2, 1=N1, 2=N2);
    impl_fbm!(3, 1=N1, 2=N2, 3=N3);
    impl_fbm!(4, 1=N1, 2=N2, 3=N3, 4=N4);
    impl_fbm!(5, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5);
    impl_fbm!(6, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5, 6=N6);
    impl_fbm!(7, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5, 6=N6, 7=N7);
    impl_fbm!(8, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5, 6=N6, 7=N7, 8=N8);
    impl_fbm!(9, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5, 6=N6, 7=N7, 8=N8, 9=N9);
    impl_fbm!(10, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5, 6=N6, 7=N7, 8=N8, 9=N9, 10=N10);
    impl_fbm!(11, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5, 6=N6, 7=N7, 8=N8, 9=N9, 10=N10, 11=N11);
    impl_fbm!(12, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5, 6=N6, 7=N7, 8=N8, 9=N9, 10=N10, 11=N11, 12=N12);
    impl_fbm!(13, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5, 6=N6, 7=N7, 8=N8, 9=N9, 10=N10, 11=N11, 12=N12, 13=N13);
    impl_fbm!(14, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5, 6=N6, 7=N7, 8=N8, 9=N9, 10=N10, 11=N11, 12=N12, 13=N13, 14=N14);
    impl_fbm!(15, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5, 6=N6, 7=N7, 8=N8, 9=N9, 10=N10, 11=N11, 12=N12, 13=N13, 14=N14, 15=N15);
    impl_fbm!(16, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5, 6=N6, 7=N7, 8=N8, 9=N9, 10=N10, 11=N11, 12=N12, 13=N13, 14=N14, 15=N15, 16=N16);
    impl_fbm!(17, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5, 6=N6, 7=N7, 8=N8, 9=N9, 10=N10, 11=N11, 12=N12, 13=N13, 14=N14, 15=N15, 16=N16, 17=N17);
    impl_fbm!(18, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5, 6=N6, 7=N7, 8=N8, 9=N9, 10=N10, 11=N11, 12=N12, 13=N13, 14=N14, 15=N15, 16=N16, 17=N17, 18=N18);
    impl_fbm!(19, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5, 6=N6, 7=N7, 8=N8, 9=N9, 10=N10, 11=N11, 12=N12, 13=N13, 14=N14, 15=N15, 16=N16, 17=N17, 18=N18, 19=N19);
    impl_fbm!(20, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5, 6=N6, 7=N7, 8=N8, 9=N9, 10=N10, 11=N11, 12=N12, 13=N13, 14=N14, 15=N15, 16=N16, 17=N17, 18=N18, 19=N19, 20=N20);
    impl_fbm!(21, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5, 6=N6, 7=N7, 8=N8, 9=N9, 10=N10, 11=N11, 12=N12, 13=N13, 14=N14, 15=N15, 16=N16, 17=N17, 18=N18, 19=N19, 20=N20, 21=N21);
    impl_fbm!(22, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5, 6=N6, 7=N7, 8=N8, 9=N9, 10=N10, 11=N11, 12=N12, 13=N13, 14=N14, 15=N15, 16=N16, 17=N17, 18=N18, 19=N19, 20=N20, 21=N21, 22=N22);
    impl_fbm!(23, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5, 6=N6, 7=N7, 8=N8, 9=N9, 10=N10, 11=N11, 12=N12, 13=N13, 14=N14, 15=N15, 16=N16, 17=N17, 18=N18, 19=N19, 20=N20, 21=N21, 22=N22, 23=N23);
    impl_fbm!(24, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5, 6=N6, 7=N7, 8=N8, 9=N9, 10=N10, 11=N11, 12=N12, 13=N13, 14=N14, 15=N15, 16=N16, 17=N17, 18=N18, 19=N19, 20=N20, 21=N21, 22=N22, 23=N23, 24=N24);
    impl_fbm!(25, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5, 6=N6, 7=N7, 8=N8, 9=N9, 10=N10, 11=N11, 12=N12, 13=N13, 14=N14, 15=N15, 16=N16, 17=N17, 18=N18, 19=N19, 20=N20, 21=N21, 22=N22, 23=N23, 24=N24, 25=N25);
    impl_fbm!(26, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5, 6=N6, 7=N7, 8=N8, 9=N9, 10=N10, 11=N11, 12=N12, 13=N13, 14=N14, 15=N15, 16=N16, 17=N17, 18=N18, 19=N19, 20=N20, 21=N21, 22=N22, 23=N23, 24=N24, 25=N25, 26=N26);
    impl_fbm!(27, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5, 6=N6, 7=N7, 8=N8, 9=N9, 10=N10, 11=N11, 12=N12, 13=N13, 14=N14, 15=N15, 16=N16, 17=N17, 18=N18, 19=N19, 20=N20, 21=N21, 22=N22, 23=N23, 24=N24, 25=N25, 26=N26, 27=N27);
    impl_fbm!(28, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5, 6=N6, 7=N7, 8=N8, 9=N9, 10=N10, 11=N11, 12=N12, 13=N13, 14=N14, 15=N15, 16=N16, 17=N17, 18=N18, 19=N19, 20=N20, 21=N21, 22=N22, 23=N23, 24=N24, 25=N25, 26=N26, 27=N27, 28=N28);
    impl_fbm!(29, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5, 6=N6, 7=N7, 8=N8, 9=N9, 10=N10, 11=N11, 12=N12, 13=N13, 14=N14, 15=N15, 16=N16, 17=N17, 18=N18, 19=N19, 20=N20, 21=N21, 22=N22, 23=N23, 24=N24, 25=N25, 26=N26, 27=N27, 28=N28, 29=N29);
    impl_fbm!(30, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5, 6=N6, 7=N7, 8=N8, 9=N9, 10=N10, 11=N11, 12=N12, 13=N13, 14=N14, 15=N15, 16=N16, 17=N17, 18=N18, 19=N19, 20=N20, 21=N21, 22=N22, 23=N23, 24=N24, 25=N25, 26=N26, 27=N27, 28=N28, 29=N29, 30=N30);
    impl_fbm!(31, 1=N1, 2=N2, 3=N3, 4=N4, 5=N5, 6=N6, 7=N7, 8=N8, 9=N9, 10=N10, 11=N11, 12=N12, 13=N13, 14=N14, 15=N15, 16=N16, 17=N17, 18=N18, 19=N19, 20=N20, 21=N21, 22=N22, 23=N23, 24=N24, 25=N25, 26=N26, 27=N27, 28=N28, 29=N29, 30=N30, 31=N31);
}

/// This describes typical settings for a [`NoiseOp`] that creates noise based purely on space and
/// seeds.
pub struct SpatialNoiseSettings {
    /// The period at which the noise may repeat, the inverse of frequency.
    /// You can think of this like the scale of the noise.
    pub period: f32,
    rng: NoiseRng<White32>,
}

impl SpatialNoiseSettings {
    /// Gets the [`Rng`] for this [`SpatialNoiseSettings`].
    pub fn rng(&mut self) -> &mut impl Rng {
        &mut self.rng
    }

    /// Gets a unique [`Seeding`].
    pub fn seeding(&mut self) -> Seeding {
        Seeding(self.rng.next_u32())
    }
}

/// A [`FbmOctaveGenerator`] for [`SpatialNoiseSettings`].
pub struct SpatialFbmGenerator {
    period: f32,
    rng: NoiseRng<White32>,
    weight: f32,
    octave_fall_off: f32,
    octave_scaling: f32,
}

/// [`FbmSettings`] for [`SpatialFbmGenerator`] using [`NoiseRng`] of [`White32`].
pub struct SpatialFbmSettings {
    /// The period at which the noise may repeat, the inverse of frequency.
    /// You can think of this like the scale of the noise.
    pub period: f32,
    /// A 64-bit seed value.
    pub seed: u64,
    /// A multiplier that determines how the weight of each octave changes.
    pub octave_fall_off: f32,
    /// A multiplier that determines how the period of each octave changes.
    pub octave_scaling: f32,
}

impl FbmOctaveGenerator<SpatialNoiseSettings> for SpatialFbmGenerator {
    #[inline]
    fn get_octave(&mut self) -> SpatialNoiseSettings {
        SpatialNoiseSettings {
            period: self.period,
            rng: self.rng.break_off(),
        }
    }

    #[inline]
    fn get_weight(&self) -> f32 {
        self.weight
    }

    #[inline]
    fn progress_octave(&mut self) {
        self.period *= self.octave_scaling;
        self.weight *= self.octave_fall_off;
    }
}

impl FbmSettings<SpatialNoiseSettings> for SpatialFbmSettings {
    #[inline]
    fn get_generator(&self, _octaves: u8) -> impl FbmOctaveGenerator<SpatialNoiseSettings> {
        SpatialFbmGenerator {
            period: self.period,
            rng: NoiseRng::new_seed(self.seed),
            octave_fall_off: self.octave_fall_off,
            octave_scaling: self.octave_scaling,
            weight: 100.0,
        }
    }
}
