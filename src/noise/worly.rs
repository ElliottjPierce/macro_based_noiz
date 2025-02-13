//! Allows [`Cellular`] noise to be converted into more useful things.

use super::{
    NoiseOp,
    cellular::Cellular,
    grid::{
        GridPoint2,
        GridPoint3,
        GridPoint4,
    },
    merging::{
        EuclideanDistance,
        ManhatanDistance,
        MergeWithoutSeed,
        Mergeable,
        Merger,
        MinOrder,
    },
    parallel::Parallel,
    seeded::{
        Seeded,
        Seeding,
    },
};

/// Initializes a particular kind of worly noise
pub trait WorlyInitializer<I, T>: Sized {
    /// Creates a new `T` noise based on this [`Cellular`].
    fn init(self, cellular: &Cellular) -> T;
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
    pub fn from_initializer<I>(
        cellular: Cellular,
        seed: u32,
        initializer: impl WorlyInitializer<I, T>,
    ) -> Self {
        Self {
            source: initializer.init(&cellular),
            cellular,
            seeder: Seeding { seed },
        }
    }

    /// creates a new [`Worly`] from [`Cellular`] with a seed
    #[inline]
    pub fn new<I>(cellular: Cellular, seed: u32) -> Self
    where
        (): WorlyInitializer<I, T>,
    {
        Self::from_initializer(cellular, seed, ())
    }
}

/// easily implements worly for different inputs
macro_rules! impl_worly {
    ($point:path, $d:literal) => {
        impl<T: Merger<Seeded<$point>, Cellular>> NoiseOp<$point> for Worly<T> {
            type Output = T::Output;

            #[inline]
            fn get(&self, input: $point) -> Self::Output {
                let corners = Parallel::<$point, Seeding>::new(self.seeder).get(input.corners());
                let cellular = self.cellular.get(corners);
                cellular.perform_merge(&self.source)
            }
        }

        impl WorlyInitializer<$point, MergeWithoutSeed<MinOrder<EuclideanDistance>>> for () {
            #[inline]
            fn init(self, cellular: &Cellular) -> MergeWithoutSeed<MinOrder<EuclideanDistance>> {
                let max_component = cellular.0.max_nudge() + 0.5;
                let distance = EuclideanDistance {
                    inv_max_expected: 1.0 / (max_component * max_component * ($d as f32)).sqrt(),
                };
                MergeWithoutSeed(MinOrder(distance))
            }
        }

        impl WorlyInitializer<$point, MergeWithoutSeed<MinOrder<ManhatanDistance>>> for () {
            #[inline]
            fn init(self, cellular: &Cellular) -> MergeWithoutSeed<MinOrder<ManhatanDistance>> {
                let max_component = cellular.0.max_nudge() + 0.5;
                let distance = ManhatanDistance {
                    inv_max_expected: 1.0 / (max_component * max_component * ($d as f32)),
                };
                MergeWithoutSeed(MinOrder(distance))
            }
        }
    };
}

impl_worly!(GridPoint2, 2);
impl_worly!(GridPoint3, 3);
impl_worly!(GridPoint4, 4);
