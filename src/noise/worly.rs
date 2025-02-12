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
        Mergeable,
        Merger,
        MinOrder,
    },
};

/// Initializes a particular kind of worly noise
pub trait WorlyInitializer<I, T>: Sized {
    /// Creates a new `T` noise based on this [`Cellular`].
    fn init(self, cellular: &Cellular) -> T;
}

/// Worly noise is defined as any kind of noise derived from [`Cellular`] noise via a
/// [`WorlyNoiseSource`] as `T`.
pub struct Worly<T> {
    cellular: Cellular,
    source: T,
}

impl<T> Worly<T> {
    /// creates a new [`Worly`] from the initializer
    #[inline]
    pub fn from_initializer<I>(
        cellular: Cellular,
        initializer: impl WorlyInitializer<I, T>,
    ) -> Self {
        Self {
            source: initializer.init(&cellular),
            cellular,
        }
    }

    /// creates a new [`Worly`] from [`Cellular`]
    #[inline]
    pub fn new<I>(cellular: Cellular) -> Self
    where
        (): WorlyInitializer<I, T>,
    {
        Self {
            source: ().init(&cellular),
            cellular,
        }
    }
}

/// easily implements worly for different inputs
macro_rules! impl_worly {
    ($point:path, $d:literal) => {
        impl<T: Merger<$point, Cellular>> NoiseOp<$point> for Worly<T> {
            type Output = T::Output;

            #[inline]
            fn get(&self, input: $point) -> Self::Output {
                let cellular = self.cellular.get(input);
                cellular.perform_merge(&self.source)
            }
        }

        impl WorlyInitializer<$point, MinOrder<EuclideanDistance>> for () {
            fn init(self, cellular: &Cellular) -> MinOrder<EuclideanDistance> {
                let max_component = cellular.0.max_nudge() + 0.5;
                let distance = EuclideanDistance {
                    inv_max_expected: 1.0 / (max_component * max_component * ($d as f32)).sqrt(),
                };
                MinOrder(distance)
            }
        }

        impl WorlyInitializer<$point, MinOrder<ManhatanDistance>> for () {
            fn init(self, cellular: &Cellular) -> MinOrder<ManhatanDistance> {
                let max_component = cellular.0.max_nudge() + 0.5;
                let distance = ManhatanDistance {
                    inv_max_expected: 1.0 / (max_component * max_component * ($d as f32)).sqrt(),
                };
                MinOrder(distance)
            }
        }
    };
}

impl_worly!(GridPoint2, 2);
impl_worly!(GridPoint3, 3);
impl_worly!(GridPoint4, 4);
