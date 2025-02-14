//! Allows [`Cellular`] noise to be converted into more useful things.

use bevy_math::{
    Vec2,
    Vec3,
    Vec4,
};

use super::{
    NoiseOp,
    NoiseType,
    cellular::{
        Cellular,
        CellularResult,
    },
    grid::{
        GridPoint2,
        GridPoint3,
        GridPoint4,
    },
    merging::{
        EuclideanDistance,
        ManhatanDistance,
        Mergeable,
        MinOrder,
        Orderer,
    },
    norm::UNorm,
    parallel::Parallel,
    seeded::{
        Seeded,
        Seeding,
    },
};

/// Initializes a particular kind of worly noise. The `I` describes the expected input point type.
pub trait WorlyInitializer<I: NoiseType, T>: Sized {
    /// Creates a new `T` noise based on this [`Cellular`].
    fn init(self, cellular: &Cellular) -> T;
}

/// Describes a source of Worly noise as a [`NoiseOp`] for [`CellularResult`].
pub trait WorlySource<I: NoiseType, const D: usize>:
    NoiseOp<CellularResult<[Seeded<I>; D]>>
{
}

/// Worly noise is defined as any kind of noise derived from [`Cellular`] noise via a
/// [`WorlyNoiseSource`] as `T`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Worly<T> {
    seeder: Seeding,
    cellular: Cellular,
    source: T,
}

impl<T> Worly<T> {
    /// creates a new [`Worly`] from the initializer and seed
    #[inline]
    pub fn from_initializer<I: NoiseType>(
        cellular: Cellular,
        seed: u32,
        initializer: impl WorlyInitializer<I, T>,
    ) -> Self {
        Self {
            source: initializer.init(&cellular),
            cellular,
            seeder: Seeding(seed),
        }
    }

    /// creates a new [`Worly`] from [`Cellular`] with a seed
    #[inline]
    pub fn new<I: NoiseType>(cellular: Cellular, seed: u32) -> Self
    where
        (): WorlyInitializer<I, T>,
    {
        Self::from_initializer(cellular, seed, ())
    }
}

/// A [`WorlySource`] based on an [`Orderer`] that outputs a [`UNorm`]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct DistanceWorly<T>(pub MinOrder<T>);

/// easily implements worly for different inputs
macro_rules! impl_worly {
    ($point:path, $vec:path, $d:literal) => {
        impl<T: WorlySource<$point, { 2 << ($d - 1) }>> NoiseOp<$point> for Worly<T> {
            type Output = T::Output;

            #[inline]
            fn get(&self, input: $point) -> Self::Output {
                let corners = Parallel(self.seeder).get(input.corners());
                let cellular = self.cellular.get(corners);
                self.source.get(cellular)
            }
        }

        impl<T: Orderer<$vec, OrderingOutput = UNorm>> WorlySource<$point, { 2 << ($d - 1) }>
            for DistanceWorly<T>
        {
        }

        impl<T: Orderer<$vec, OrderingOutput = UNorm>>
            NoiseOp<CellularResult<[Seeded<$point>; { 2 << ($d - 1) }]>> for DistanceWorly<T>
        {
            type Output = UNorm;

            #[inline]
            fn get(
                &self,
                input: CellularResult<[Seeded<$point>; { 2 << ($d - 1) }]>,
            ) -> Self::Output {
                input
                    .map(|points| points.map(|point| point.value.offset))
                    .perform_merge(&self.0)
            }
        }

        impl WorlyInitializer<$point, EuclideanDistance> for () {
            #[inline]
            fn init(self, cellular: &Cellular) -> EuclideanDistance {
                let max_component = cellular.0.max_nudge();
                EuclideanDistance {
                    inv_max_expected: 1.0 / (max_component * max_component * ($d as f32)).sqrt(),
                }
            }
        }

        impl WorlyInitializer<$point, ManhatanDistance> for () {
            #[inline]
            fn init(self, cellular: &Cellular) -> ManhatanDistance {
                let max_component = cellular.0.max_nudge();
                ManhatanDistance {
                    inv_max_expected: 1.0 / (max_component * max_component * ($d as f32)),
                }
            }
        }

        impl WorlyInitializer<$point, DistanceWorly<EuclideanDistance>> for () {
            #[inline]
            fn init(self, cellular: &Cellular) -> DistanceWorly<EuclideanDistance> {
                DistanceWorly(MinOrder(<Self as WorlyInitializer<
                    $point,
                    EuclideanDistance,
                >>::init(self, cellular)))
            }
        }

        impl WorlyInitializer<$point, DistanceWorly<ManhatanDistance>> for () {
            #[inline]
            fn init(self, cellular: &Cellular) -> DistanceWorly<ManhatanDistance> {
                DistanceWorly(MinOrder(<Self as WorlyInitializer<
                    $point,
                    ManhatanDistance,
                >>::init(self, cellular)))
            }
        }
    };
}

impl_worly!(GridPoint2, Vec2, 2);
impl_worly!(GridPoint3, Vec3, 3);
impl_worly!(GridPoint4, Vec4, 4);
