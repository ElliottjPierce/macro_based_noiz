//! Allows [`Cellular`] noise to be converted into more useful things.

use std::marker::PhantomData;

use bevy_math::{
    Vec2,
    Vec3,
    Vec4,
};

use super::{
    NoiseOp,
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
    parallel::Parallel,
    seeded::{
        Seeded,
        Seeding,
    },
    voronoi::{
        Voronoi,
        VoronoiGraph,
    },
};

/// Describes a source of Worly noise with a [`NoiseOp`] for [`VoronoiGraph`].
pub trait WorlySource<const DIMENSIONS: u8, const APPROX: bool> {
    /// The type of noise
    type Noise;

    /// Creates the noise itself
    fn build_noise(self, voronoi: &Voronoi) -> Self::Noise;
}

/// Worly noise is defined as any kind of noise derived from [`Cellular`] noise via a
/// [`WorlyNoiseSource`] as `T`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Worly<
    const DIMENSIONS: u8,
    S: WorlySource<DIMENSIONS, APPROX>,
    const APPROX: bool = false,
> {
    seeder: Seeding,
    voronoi: Voronoi,
    source: S::Noise,
}

impl<const DIMENSIONS: u8, const APPROX: bool, S: WorlySource<DIMENSIONS, APPROX>>
    Worly<DIMENSIONS, S, APPROX>
{
    /// creates a new [`Worly`] from [`Voronoi`] with a seed and a noise source.
    #[inline]
    pub fn new_with_noise(voronoi: Voronoi, seed: u32, noise: S) -> Self {
        Self {
            seeder: Seeding(seed),
            source: noise.build_noise(&voronoi),
            voronoi,
        }
    }
}

impl<const DIMENSIONS: u8, const APPROX: bool, T> Worly<DIMENSIONS, ImplicitWorlySource<T>, APPROX>
where
    ImplicitWorlySource<T>: WorlySource<DIMENSIONS, APPROX>,
{
    /// creates a new [`Worly`] from [`Voronoi`] with a seed.
    #[inline]
    pub fn new(voronoi: Voronoi, seed: u32) -> Self {
        Self {
            seeder: Seeding(seed),
            source: ImplicitWorlySource::<T>(PhantomData).build_noise(&voronoi),
            voronoi,
        }
    }
}

/// Allows for standard, distance-based worly noise.
pub struct DistanceWorly<T>(T);

/// A general purpose [`WorlySource`] that doesn't have any fields.
pub struct ImplicitWorlySource<T>(pub PhantomData<T>);

/// easily implements worly for different inputs
macro_rules! impl_worly {
    ($point:path, $vec:path, $d:literal, $d_2:literal, $d_3:literal) => {
        impl<S: WorlySource<$d, true>> NoiseOp<$point> for Worly<$d, S, true>
        where
            S::Noise: NoiseOp<VoronoiGraph<[Seeded<$point>; $d_2]>>,
        {
            type Output = <S::Noise as NoiseOp<VoronoiGraph<[Seeded<$point>; $d_2]>>>::Output;

            #[inline]
            fn get(&self, input: $point) -> Self::Output {
                let points = Parallel(self.seeder).get(input.corners());
                let voronoi = self.voronoi.get(points);
                self.source.get(voronoi)
            }
        }

        impl<S: WorlySource<$d, false>> NoiseOp<$point> for Worly<$d, S, false>
        where
            S::Noise: NoiseOp<VoronoiGraph<[Seeded<$point>; $d_3]>>,
        {
            type Output = <S::Noise as NoiseOp<VoronoiGraph<[Seeded<$point>; $d_3]>>>::Output;

            #[inline]
            fn get(&self, input: $point) -> Self::Output {
                let points = Parallel(self.seeder).get(input.surroundings());
                let voronoi = self.voronoi.get(points);
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

        impl WorlySource<$d, true> for ImplicitWorlySource<EuclideanDistance> {
            type Noise = DistanceWorly<EuclideanDistance>;

            fn build_noise(self, voronoi: &Voronoi) -> Self::Noise {
                let max_displacement = voronoi.get_nudge().max_nudge();
                let max_dist = (max_displacement * max_displacement * ($d as f32)).sqrt();
                DistanceWorly(EuclideanDistance {
                    inv_max_expected: 1.0 / max_dist,
                })
            }
        }

        impl WorlySource<$d, false> for ImplicitWorlySource<EuclideanDistance> {
            type Noise = DistanceWorly<EuclideanDistance>;

            fn build_noise(self, voronoi: &Voronoi) -> Self::Noise {
                let max_displacement = voronoi.get_nudge().max_nudge() + 0.5;
                let max_dist = (max_displacement * max_displacement * ($d as f32)).sqrt();
                DistanceWorly(EuclideanDistance {
                    inv_max_expected: 1.0 / max_dist,
                })
            }
        }

        impl WorlySource<$d, true> for ImplicitWorlySource<ManhatanDistance> {
            type Noise = DistanceWorly<ManhatanDistance>;

            fn build_noise(self, voronoi: &Voronoi) -> Self::Noise {
                let max_displacement = voronoi.get_nudge().max_nudge();
                let max_dist = max_displacement * ($d as f32);
                DistanceWorly(ManhatanDistance {
                    inv_max_expected: 1.0 / max_dist,
                })
            }
        }

        impl WorlySource<$d, false> for ImplicitWorlySource<ManhatanDistance> {
            type Noise = DistanceWorly<ManhatanDistance>;

            fn build_noise(self, voronoi: &Voronoi) -> Self::Noise {
                let max_displacement = voronoi.get_nudge().max_nudge() + 0.5;
                let max_dist = max_displacement * ($d as f32);
                DistanceWorly(ManhatanDistance {
                    inv_max_expected: 1.0 / max_dist,
                })
            }
        }
    };
}

impl_worly!(GridPoint2, Vec2, 2, 4, 9);
impl_worly!(GridPoint3, Vec3, 3, 8, 27);
impl_worly!(GridPoint4, Vec4, 4, 16, 81);
