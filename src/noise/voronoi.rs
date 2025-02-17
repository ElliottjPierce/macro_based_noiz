//! This module allows Cellular noise to be created

use std::marker::PhantomData;

use bevy_math::{
    Vec2,
    Vec3,
    Vec4,
};

use super::{
    NoiseOp,
    associating::Associated,
    grid::{
        GridPoint2,
        GridPoint3,
        GridPoint4,
    },
    merging::{
        EuclideanDistance,
        ManhatanDistance,
        Merger,
        MinIndex,
        MinOrder,
        Orderer,
    },
    norm::UNorm,
    nudges::Nudge,
    seeded::{
        Seeded,
        Seeding,
    },
};

/// Describes a source of Worly noise with a [`NoiseOp`] for [`VoronoiGraph`].
pub trait VoronoiSource<const DIMENSIONS: u8, const APPROX: bool> {
    /// The type of noise
    type Noise;

    /// Creates the noise itself
    fn build_noise(self, max_nudge: f32) -> Self::Noise;
}

/// Worly noise is defined as any kind of noise derived from [`Cellular`] noise via a
/// [`WorlyNoiseSource`] as `T`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Voronoi<
    const DIMENSIONS: u8,
    S: VoronoiSource<DIMENSIONS, APPROX>,
    const APPROX: bool = false,
> {
    seeder: Seeding,
    nudge: Nudge<true>,
    source: S::Noise,
}

/// Stores a result of a [`Voronoi`] noise
pub type VoronoiGraph<T> = Associated<T, Nudge<true>>;

impl<const DIMENSIONS: u8, const APPROX: bool, S: VoronoiSource<DIMENSIONS, APPROX>>
    Voronoi<DIMENSIONS, S, APPROX>
{
    /// creates a new [`Voronoi`] from nudge range with a seed and a noise source.
    #[inline]
    pub fn new(range: f32, seed: u32, noise: S) -> Self {
        let mut real_range = range.abs().min(1.0);
        if APPROX {
            real_range *= 0.5;
        }
        Self {
            seeder: Seeding(seed),
            source: noise.build_noise(real_range),
            nudge: Nudge::new_magnitude(real_range),
        }
    }

    /// creates a new [`Voronoi`] from a seed and a noise source.
    #[inline]
    pub fn full(seed: u32, noise: S) -> Self {
        Self::new(1.0, seed, noise)
    }
}

/// Allows for standard, distance-based worly noise.
#[derive(Debug, Clone, Copy, Default)]
pub struct WorlyNoise<T>(T);

/// A [`VoronoiSource`] for [`WorlyNoise`].
#[derive(Debug, Clone, Copy)]
pub struct Worly<T> {
    /// marker data
    pub marker: PhantomData<T>,
    /// This a a multiplier for the expected maximum length of a voronoi sphere.
    /// 1.0 is the default. Infreasing this too much can lead to articacts.
    /// Decreasing this can mave the voronoi spheres more issolated.
    pub expected_length_multiplier: f32,
}

/// Allows simple, nearest neighbor cellular noise
#[derive(Debug, Clone, Copy, Default)]
pub struct CellularNoise<T>(T);

/// A [`VoronoiSource`] for [`CellularNoise`].
#[derive(Debug, Clone, Copy)]
pub struct Cellular<T>(pub PhantomData<T>);

impl<T> Default for Cellular<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T> Default for Worly<T> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
            expected_length_multiplier: 1.0,
        }
    }
}

impl<T> Worly<T> {
    /// Clams the absolute value of this factor as [`WorlySource::expected_length_multiplier`].
    pub fn shrunk_by(srkinging_factor: f32) -> Self {
        Self {
            marker: PhantomData,
            expected_length_multiplier: srkinging_factor.abs().clamp(0.0, 1.0),
        }
    }

    /// Maxes the absolute value of this factor as [`WorlySource::expected_length_multiplier`].
    ///
    /// # Warning. This can lead to artifacts. Use this carefully.
    pub fn expanded_by(expansion_factor: f32) -> Self {
        Self {
            marker: PhantomData,
            expected_length_multiplier: expansion_factor.abs().max(0.0),
        }
    }
}

/// easily implements worly for different inputs
macro_rules! impl_voronoi {
    ($point:path, $vec:path, $d:literal, $d_2:literal, $d_3:literal) => {
        // worly

        impl<S: VoronoiSource<$d, true>> NoiseOp<$point> for Voronoi<$d, S, true>
        where
            S::Noise: NoiseOp<VoronoiGraph<[Seeded<$point>; $d_2]>>,
        {
            type Output = <S::Noise as NoiseOp<VoronoiGraph<[Seeded<$point>; $d_2]>>>::Output;

            #[inline]
            fn get(&self, input: $point) -> Self::Output {
                let points = input.corners().map(|point| {
                    let mut seeded = self.seeder.get(point);
                    let grid_shift = self.nudge.get(seeded.map_ref(|p| p.base)).value;
                    seeded.value.offset -= grid_shift;
                    seeded
                });
                let voronoi = VoronoiGraph {
                    value: points,
                    meta: self.nudge,
                };
                self.source.get(voronoi)
            }
        }

        impl<S: VoronoiSource<$d, false>> NoiseOp<$point> for Voronoi<$d, S, false>
        where
            S::Noise: NoiseOp<VoronoiGraph<[Seeded<$point>; $d_3]>>,
        {
            type Output = <S::Noise as NoiseOp<VoronoiGraph<[Seeded<$point>; $d_3]>>>::Output;

            #[inline]
            fn get(&self, input: $point) -> Self::Output {
                let points = input.surroundings().map(|point| {
                    let mut seeded = self.seeder.get(point);
                    let grid_shift = self.nudge.get(seeded.map_ref(|p| p.base)).value;
                    seeded.value.offset -= grid_shift;
                    seeded
                });
                let voronoi = VoronoiGraph {
                    value: points,
                    meta: self.nudge,
                };
                self.source.get(voronoi)
            }
        }

        // worly

        impl<O: Orderer<$vec, OrderingOutput = UNorm>, const K: usize>
            NoiseOp<VoronoiGraph<[Seeded<$point>; K]>> for WorlyNoise<O>
        {
            type Output = UNorm;

            #[inline]
            fn get(&self, input: VoronoiGraph<[Seeded<$point>; K]>) -> Self::Output {
                let points = input.value.map(|point| point.value.offset);
                MinOrder(&self.0).merge(points, &())
            }
        }

        impl<const APPROX: bool> VoronoiSource<$d, APPROX> for Worly<EuclideanDistance> {
            type Noise = WorlyNoise<EuclideanDistance>;

            fn build_noise(self, max_nudge: f32) -> Self::Noise {
                let max_displacement = max_nudge * self.expected_length_multiplier;
                let max_dist = if APPROX {
                    // a negative cell could be at the same spot on all axies but the cell's offset.
                    (max_displacement * max_displacement).sqrt()
                } else {
                    (max_displacement * max_displacement * ($d as f32)).sqrt()
                };
                WorlyNoise(EuclideanDistance {
                    inv_max_expected: 1.0 / max_dist,
                })
            }
        }

        impl<const APPROX: bool> VoronoiSource<$d, APPROX> for Worly<ManhatanDistance> {
            type Noise = WorlyNoise<ManhatanDistance>;

            fn build_noise(self, max_nudge: f32) -> Self::Noise {
                let max_displacement = max_nudge * self.expected_length_multiplier;
                let max_dist = if APPROX {
                    // a negative cell could be at the same spot on all axies but the cell's offset.
                    max_displacement
                } else {
                    max_displacement * ($d as f32)
                };
                WorlyNoise(ManhatanDistance {
                    inv_max_expected: 1.0 / max_dist,
                })
            }
        }

        // cellular

        // we can't generalize CellularNoise's array length since length of 0 is unsafe.
        impl<O: Orderer<$vec, OrderingOutput = UNorm>> NoiseOp<VoronoiGraph<[Seeded<$point>; $d_2]>>
            for CellularNoise<O>
        {
            type Output = Seeded<$point>;

            #[inline]
            fn get(&self, input: VoronoiGraph<[Seeded<$point>; $d_2]>) -> Self::Output {
                let points = input.value.clone().map(|point| point.value.offset);
                let index = MinIndex(&self.0).merge(points, &());
                input.value[index].clone()
            }
        }

        impl<O: Orderer<$vec, OrderingOutput = UNorm>> NoiseOp<VoronoiGraph<[Seeded<$point>; $d_3]>>
            for CellularNoise<O>
        {
            type Output = Seeded<$point>;

            #[inline]
            fn get(&self, input: VoronoiGraph<[Seeded<$point>; $d_3]>) -> Self::Output {
                let points = input.value.clone().map(|point| point.value.offset);
                let index = MinIndex(&self.0).merge(points, &());
                input.value[index].clone()
            }
        }

        impl<const APPROX: bool> VoronoiSource<$d, APPROX> for Cellular<EuclideanDistance> {
            type Noise = CellularNoise<EuclideanDistance>;

            fn build_noise(self, _max_nudge: f32) -> Self::Noise {
                CellularNoise(EuclideanDistance {
                    inv_max_expected: 0.0,
                })
            }
        }

        impl<const APPROX: bool> VoronoiSource<$d, APPROX> for Cellular<ManhatanDistance> {
            type Noise = CellularNoise<ManhatanDistance>;

            fn build_noise(self, _max_nudge: f32) -> Self::Noise {
                CellularNoise(ManhatanDistance {
                    inv_max_expected: 0.0,
                })
            }
        }
    };
}

impl_voronoi!(GridPoint2, Vec2, 2, 4, 9);
impl_voronoi!(GridPoint3, Vec3, 3, 8, 27);
impl_voronoi!(GridPoint4, Vec4, 4, 16, 81);
