//! This module allows Cellular noise to be created

use std::marker::PhantomData;

use bevy_math::{
    Vec2,
    Vec3,
    Vec4,
};

use super::{
    NoiseOp,
    NoiseType,
    associating::Associated,
    grid::{
        GridPoint2,
        GridPoint3,
        GridPoint4,
    },
    merging::{
        EuclideanDistance,
        HybridDistance,
        ManhatanDistance,
        Merger,
        MinIndex,
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

    /// creates a new [`Voronoi`] from nudge range with a seed with a default noise source.
    #[inline]
    pub fn new_default(range: f32, seed: u32) -> Self
    where
        S: Default,
    {
        Self::new(range, seed, S::default())
    }

    /// creates a new [`Voronoi`] from a seed and a noise source.
    #[inline]
    pub fn full(seed: u32, noise: S) -> Self {
        Self::new(1.0, seed, noise)
    }

    /// creates a new [`Voronoi`] from a seed with a default noise source.
    #[inline]
    pub fn full_default(seed: u32) -> Self
    where
        S: Default,
    {
        Self::full(seed, S::default())
    }
}

/// Defines a particular mode for `Worly` to operate in.
pub trait WorlyMode {
    /// Computes the actual worly result given an orderer and the points.
    fn compute_worly<const N: usize, T: NoiseType>(
        &self,
        orderer: &impl Orderer<T, OrderingOutput = UNorm>,
        points: [T; N],
    ) -> UNorm;
}

/// Allows for standard, distance-based worly noise.
#[derive(Debug, Clone, Copy, Default)]
pub struct WorlyNoise<T, M>(T, M);

/// A [`VoronoiSource`] for [`WorlyNoise`].
#[derive(Debug, Clone, Copy)]
pub struct Worly<T, M> {
    /// marker data
    pub marker: PhantomData<T>,
    /// This a a multiplier for the expected maximum length of a voronoi sphere.
    /// 1.0 is the default. Infreasing this too much can lead to articacts.
    /// Decreasing this can mave the voronoi spheres more issolated.
    pub expected_length_multiplier: f32,
    /// Defines the [`WorlyMode`] this noise will use.
    pub mode: M,
}

/// Contains some common [`WorlyMode`]s.
pub mod worly_mode {
    use super::WorlyMode;
    use crate::noise::{
        NoiseType,
        merging::{
            Merger,
            MinOrder,
            MinOrders,
            Orderer,
        },
        norm::UNorm,
    };

    /// A [`WorlyMode`] that uses the nearst distance.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct Nearest;

    impl WorlyMode for Nearest {
        fn compute_worly<const N: usize, T: NoiseType>(
            &self,
            orderer: &impl Orderer<T, OrderingOutput = UNorm>,
            points: [T; N],
        ) -> UNorm {
            MinOrder(orderer).merge(points, &())
        }
    }

    /// A [`WorlyMode`] that uses the second nearst distance.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct NextNearest;

    impl WorlyMode for NextNearest {
        fn compute_worly<const N: usize, T: NoiseType>(
            &self,
            orderer: &impl Orderer<T, OrderingOutput = UNorm>,
            points: [T; N],
        ) -> UNorm {
            MinOrders(orderer).merge(points, &())[1]
        }
    }

    /// A [`WorlyMode`] that subtracts the nearst distance from the second nearest.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct Difference;

    impl WorlyMode for Difference {
        fn compute_worly<const N: usize, T: NoiseType>(
            &self,
            orderer: &impl Orderer<T, OrderingOutput = UNorm>,
            points: [T; N],
        ) -> UNorm {
            let [nearest, next_nearest] = MinOrders(orderer)
                .merge(points, &())
                .map(|v| v.adapt::<f32>());
            UNorm::new_clamped(next_nearest - nearest)
        }
    }

    /// A [`WorlyMode`] that averages the two nearst distances.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct Average;

    impl WorlyMode for Average {
        fn compute_worly<const N: usize, T: NoiseType>(
            &self,
            orderer: &impl Orderer<T, OrderingOutput = UNorm>,
            points: [T; N],
        ) -> UNorm {
            let [nearest, next_nearest] = MinOrders(orderer)
                .merge(points, &())
                .map(|v| v.adapt::<f32>());
            UNorm::new_clamped((next_nearest + nearest) * 0.5)
        }
    }

    /// A [`WorlyMode`] that multiplies the nearst distance from the second nearest.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct Product;

    impl WorlyMode for Product {
        fn compute_worly<const N: usize, T: NoiseType>(
            &self,
            orderer: &impl Orderer<T, OrderingOutput = UNorm>,
            points: [T; N],
        ) -> UNorm {
            let [nearest, next_nearest] = MinOrders(orderer)
                .merge(points, &())
                .map(|v| v.adapt::<f32>());
            UNorm::new_clamped(next_nearest * nearest)
        }
    }

    /// A [`WorlyMode`] that divides the nearst distance by the second nearest.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct Ratio;

    impl WorlyMode for Ratio {
        fn compute_worly<const N: usize, T: NoiseType>(
            &self,
            orderer: &impl Orderer<T, OrderingOutput = UNorm>,
            points: [T; N],
        ) -> UNorm {
            let [nearest, next_nearest] = MinOrders(orderer)
                .merge(points, &())
                .map(|v| v.adapt::<f32>());
            UNorm::new_clamped(nearest / next_nearest)
        }
    }
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

impl<T, M: Default> Default for Worly<T, M> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
            expected_length_multiplier: 1.0,
            mode: M::default(),
        }
    }
}

impl<T, M> Worly<T, M> {
    /// A version of [`shrunk_by`](Self::shrunk_by) that supplies a mode.
    pub fn new_shrunk_by(srkinging_factor: f32, mode: M) -> Self {
        Self {
            marker: PhantomData,
            expected_length_multiplier: srkinging_factor.abs().clamp(0.0, 1.0),
            mode,
        }
    }

    /// A version of [`expanded_by`](Self::expanded_by) that supplies a mode.
    pub fn new_expanded_by(expansion_factor: f32, mode: M) -> Self {
        Self {
            marker: PhantomData,
            expected_length_multiplier: expansion_factor.abs().max(0.0),
            mode,
        }
    }
}

impl<T, M: Default> Worly<T, M> {
    /// Clams the absolute value of this factor as [`WorlySource::expected_length_multiplier`].
    pub fn shrunk_by(srkinging_factor: f32) -> Self {
        Self {
            marker: PhantomData,
            expected_length_multiplier: srkinging_factor.abs().clamp(0.0, 1.0),
            mode: M::default(),
        }
    }

    /// Maxes the absolute value of this factor as [`WorlySource::expected_length_multiplier`].
    ///
    /// # Warning. This can lead to artifacts. Use this carefully.
    pub fn expanded_by(expansion_factor: f32) -> Self {
        Self {
            marker: PhantomData,
            expected_length_multiplier: expansion_factor.abs().max(0.0),
            mode: M::default(),
        }
    }

    /// Sets the [`WorlyMode`] for this noise
    pub fn with_mode(mut self, mode: M) -> Self {
        self.mode = mode;
        self
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

        impl<O: Orderer<$vec, OrderingOutput = UNorm>, M: WorlyMode, const K: usize>
            NoiseOp<VoronoiGraph<[Seeded<$point>; K]>> for WorlyNoise<O, M>
        {
            type Output = UNorm;

            #[inline]
            fn get(&self, input: VoronoiGraph<[Seeded<$point>; K]>) -> Self::Output {
                let points = input.value.map(|point| point.value.offset);
                self.1.compute_worly(&self.0, points)
            }
        }

        impl<const APPROX: bool, M> VoronoiSource<$d, APPROX> for Worly<EuclideanDistance, M> {
            type Noise = WorlyNoise<EuclideanDistance, M>;

            fn build_noise(self, max_nudge: f32) -> Self::Noise {
                let max_displacement = max_nudge * self.expected_length_multiplier;
                let max_dist = if APPROX {
                    // a negative cell could be at the same spot on all axies but the cell's offset.
                    (max_displacement * max_displacement).sqrt()
                } else {
                    (max_displacement * max_displacement * ($d as f32)).sqrt()
                };
                WorlyNoise(
                    EuclideanDistance {
                        inv_max_expected: 1.0 / max_dist,
                    },
                    self.mode,
                )
            }
        }

        impl<const APPROX: bool, M> VoronoiSource<$d, APPROX> for Worly<ManhatanDistance, M> {
            type Noise = WorlyNoise<ManhatanDistance, M>;

            fn build_noise(self, max_nudge: f32) -> Self::Noise {
                let max_displacement = max_nudge * self.expected_length_multiplier;
                let max_dist = if APPROX {
                    // a negative cell could be at the same spot on all axies but the cell's offset.
                    max_displacement
                } else {
                    max_displacement * ($d as f32)
                };
                WorlyNoise(
                    ManhatanDistance {
                        inv_max_expected: 1.0 / max_dist,
                    },
                    self.mode,
                )
            }
        }

        impl<const APPROX: bool, M> VoronoiSource<$d, APPROX> for Worly<HybridDistance, M> {
            type Noise = WorlyNoise<HybridDistance, M>;

            fn build_noise(self, max_nudge: f32) -> Self::Noise {
                let max_displacement = max_nudge * self.expected_length_multiplier;
                let max_dist = if APPROX {
                    // a negative cell could be at the same spot on all axies but the cell's offset.
                    max_displacement * max_displacement + max_displacement
                } else {
                    (max_displacement * max_displacement + max_displacement) * ($d as f32)
                };
                WorlyNoise(
                    HybridDistance {
                        inv_max_expected: 1.0 / max_dist,
                    },
                    self.mode,
                )
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

        impl<const APPROX: bool> VoronoiSource<$d, APPROX> for Cellular<HybridDistance> {
            type Noise = CellularNoise<HybridDistance>;

            fn build_noise(self, _max_nudge: f32) -> Self::Noise {
                CellularNoise(HybridDistance {
                    inv_max_expected: 0.0,
                })
            }
        }
    };
}

impl_voronoi!(GridPoint2, Vec2, 2, 4, 9);
impl_voronoi!(GridPoint3, Vec3, 3, 8, 27);
impl_voronoi!(GridPoint4, Vec4, 4, 16, 81);
