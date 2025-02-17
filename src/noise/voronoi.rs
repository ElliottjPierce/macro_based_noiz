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
    fn build_noise(self, voronoi: &Nudge) -> Self::Noise;
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
    nudge: Nudge,
    source: S::Noise,
}

/// Stores a result of a [`Voronoi`] noise
pub type VoronoiGraph<T> = Associated<T, Nudge>;

impl<const DIMENSIONS: u8, const APPROX: bool, S: VoronoiSource<DIMENSIONS, APPROX>>
    Voronoi<DIMENSIONS, S, APPROX>
{
    /// creates a new [`Worly`] from [`Voronoi`] with a seed and a noise source.
    #[inline]
    pub fn new_with_noise(voronoi: Nudge, seed: u32, noise: S) -> Self {
        Self {
            seeder: Seeding(seed),
            source: noise.build_noise(&voronoi),
            nudge: voronoi,
        }
    }
}

impl<const DIMENSIONS: u8, const APPROX: bool, T>
    Voronoi<DIMENSIONS, ImplicitWorlySource<T>, APPROX>
where
    ImplicitWorlySource<T>: VoronoiSource<DIMENSIONS, APPROX>,
{
    /// creates a new [`Worly`] from [`Voronoi`] with a seed.
    #[inline]
    pub fn new(voronoi: Nudge, seed: u32) -> Self {
        Self {
            seeder: Seeding(seed),
            source: ImplicitWorlySource::<T>(PhantomData).build_noise(&voronoi),
            nudge: voronoi,
        }
    }
}

/// Allows for standard, distance-based worly noise.
pub struct DistanceWorly<T>(T);

/// A general purpose [`WorlySource`] that doesn't have any fields.
pub struct ImplicitWorlySource<T>(pub PhantomData<T>);

/// easily implements worly for different inputs
macro_rules! impl_voronoi {
    ($point:path, $vec:path, $d:literal, $d_2:literal, $d_3:literal) => {
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

        impl<O: Orderer<$vec, OrderingOutput = UNorm>> NoiseOp<VoronoiGraph<[Seeded<$point>; $d_2]>>
            for DistanceWorly<O>
        {
            type Output = UNorm;

            #[inline]
            fn get(&self, input: VoronoiGraph<[Seeded<$point>; $d_2]>) -> Self::Output {
                let points = input.value.map(|point| point.value.offset);
                MinOrder(&self.0).merge(points, &())
            }
        }

        impl<O: Orderer<$vec, OrderingOutput = UNorm>> NoiseOp<VoronoiGraph<[Seeded<$point>; $d_3]>>
            for DistanceWorly<O>
        {
            type Output = UNorm;

            #[inline]
            fn get(&self, input: VoronoiGraph<[Seeded<$point>; $d_3]>) -> Self::Output {
                let points = input.value.map(|point| point.value.offset);
                MinOrder(&self.0).merge(points, &())
            }
        }

        impl VoronoiSource<$d, true> for ImplicitWorlySource<EuclideanDistance> {
            type Noise = DistanceWorly<EuclideanDistance>;

            fn build_noise(self, voronoi: &Nudge) -> Self::Noise {
                let max_displacement = voronoi.max_nudge();
                let max_dist = (max_displacement * max_displacement * ($d as f32)).sqrt();
                DistanceWorly(EuclideanDistance {
                    inv_max_expected: 1.0 / max_dist,
                })
            }
        }

        impl VoronoiSource<$d, false> for ImplicitWorlySource<EuclideanDistance> {
            type Noise = DistanceWorly<EuclideanDistance>;

            fn build_noise(self, voronoi: &Nudge) -> Self::Noise {
                let max_displacement = voronoi.max_nudge() + 0.5;
                let max_dist = (max_displacement * max_displacement * ($d as f32)).sqrt();
                DistanceWorly(EuclideanDistance {
                    inv_max_expected: 1.0 / max_dist,
                })
            }
        }

        impl VoronoiSource<$d, true> for ImplicitWorlySource<ManhatanDistance> {
            type Noise = DistanceWorly<ManhatanDistance>;

            fn build_noise(self, voronoi: &Nudge) -> Self::Noise {
                let max_displacement = voronoi.max_nudge();
                let max_dist = max_displacement * ($d as f32);
                DistanceWorly(ManhatanDistance {
                    inv_max_expected: 1.0 / max_dist,
                })
            }
        }

        impl VoronoiSource<$d, false> for ImplicitWorlySource<ManhatanDistance> {
            type Noise = DistanceWorly<ManhatanDistance>;

            fn build_noise(self, voronoi: &Nudge) -> Self::Noise {
                let max_displacement = voronoi.max_nudge() + 0.5;
                let max_dist = max_displacement * ($d as f32);
                DistanceWorly(ManhatanDistance {
                    inv_max_expected: 1.0 / max_dist,
                })
            }
        }
    };
}

impl_voronoi!(GridPoint2, Vec2, 2, 4, 9);
impl_voronoi!(GridPoint3, Vec3, 3, 8, 27);
impl_voronoi!(GridPoint4, Vec4, 4, 16, 81);
