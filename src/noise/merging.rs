//! THis module allows noise types to be merged together

use std::ops::{
    Add,
    AddAssign,
    Mul,
};

use bevy_math::{
    Vec2,
    Vec3,
    Vec4,
};

use super::{
    NoiseOp,
    NoiseType,
    norm::UNorm,
};

/// Allows the noise type to be merged
pub trait Merger<I, M> {
    /// the merged output
    type Output: NoiseType;

    /// merges any number of the input type into an output
    fn merge(&self, vals: impl IntoIterator<Item = I>, meta: &M) -> Self::Output;
}

/// Marks a type as being able to be merged.
pub trait Mergeable {
    /// the kind of metadata given.
    type Meta;
    /// the kind of part given, the item type in an array.
    type Part;

    /// performs merging on a with a given compatible merger.
    fn perform_merge<M: Merger<Self::Part, Self::Meta>>(self, merger: &M) -> M::Output;
}

/// Defines a type that is able to order a particular type by mapping it to a number.
pub trait Orderer<I> {
    /// Value's ordering can be mapped to this value
    type OrderingOutput: NoiseType;

    /// Maps the value to a number for its ordering.
    fn ordering_of(&self, value: &I) -> f32;

    /// Maps this particular ordering number to an output. This is useful when calculating a final
    /// order is slower than calculating one that maintains the same ordering.
    fn relative_ordering(&self, ordering: f32) -> Self::OrderingOutput;
}

/// Defines a type that is able to weigh a given type of value relative to other weights
pub trait WeightFactorer<I> {
    /// The type that the weighing results in
    type Output: AddAssign + NoiseType;

    /// Calculates the weight of the given value.
    fn weight_of(&self, value: &I) -> f32;

    /// Given a value and it's relative weight in 0..=1 convert the value to the output
    fn weigh_value(&self, value: I, relative_weight: f32) -> Self::Output;
}

impl<I, T: WeightFactorer<I>> WeightFactorer<I> for &T {
    type Output = T::Output;

    #[inline]
    fn weight_of(&self, value: &I) -> f32 {
        T::weight_of(self, value)
    }

    #[inline]
    fn weigh_value(&self, value: I, relative_weight: f32) -> Self::Output {
        T::weigh_value(self, value, relative_weight)
    }
}

impl<I, T: Orderer<I>> Orderer<I> for &T {
    type OrderingOutput = T::OrderingOutput;

    #[inline]
    fn ordering_of(&self, value: &I) -> f32 {
        T::ordering_of(self, value)
    }

    #[inline]
    fn relative_ordering(&self, ordering: f32) -> Self::OrderingOutput {
        T::relative_ordering(self, ordering)
    }
}

/// A merger that selects the value with the least weight.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct Min<T>(pub T);

impl<I: NoiseType + Default, M, T: Orderer<I>> Merger<I, M> for Min<T> {
    type Output = I;

    #[inline]
    fn merge(&self, vals: impl IntoIterator<Item = I>, _meta: &M) -> Self::Output {
        let mut ordering_number = f32::INFINITY;
        let mut result = I::default();
        for val in vals {
            let weight = self.0.ordering_of(&val);
            if weight < ordering_number {
                ordering_number = weight;
                result = val;
            }
        }

        result
    }
}

/// A merger that selects the index of the value with the least weight.
/// If you try to merge on an empty array, this will return zero.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct MinIndex<T>(pub T);

impl<I: NoiseType, M, T: Orderer<I>> Merger<I, M> for MinIndex<T> {
    type Output = usize;

    #[inline]
    fn merge(&self, vals: impl IntoIterator<Item = I>, _meta: &M) -> Self::Output {
        let mut ordering_number = f32::INFINITY;
        let mut result = 0;
        for (index, val) in vals.into_iter().enumerate() {
            let weight = self.0.ordering_of(&val);
            if weight < ordering_number {
                ordering_number = weight;
                result = index;
            }
        }

        result
    }
}

/// A merger that selects the indices of the 2 values with the least weights.
/// If you try to merge on an array shorter than 2, this will return zeros, where data is missing.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct MinIndices<T>(pub T);

impl<I: NoiseType, M, T: Orderer<I>> Merger<I, M> for MinIndices<T> {
    type Output = [usize; 2];

    #[inline]
    fn merge(&self, vals: impl IntoIterator<Item = I>, _meta: &M) -> Self::Output {
        let mut ordering_numbers = (f32::INFINITY, f32::INFINITY);
        let mut results = (0, 0);

        for (index, val) in vals.into_iter().enumerate() {
            let weight = self.0.ordering_of(&val);

            if weight < ordering_numbers.0 {
                ordering_numbers.1 = ordering_numbers.0;
                results.1 = results.0;
                ordering_numbers.0 = weight;
                results.0 = index;
            } else if weight < ordering_numbers.1 {
                ordering_numbers.1 = weight;
                results.1 = index;
            }
        }

        [results.0, results.1]
    }
}

/// A merger that selects the weights of the 2 values with the least weights.
/// If you try to merge on an array shorter than 2, this will return [`f32::INFINITY`], where data
/// is missing.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct MinOrders<T>(pub T);

impl<I: NoiseType, M, T: Orderer<I>> Merger<I, M> for MinOrders<T> {
    type Output = [T::OrderingOutput; 2];

    #[inline]
    fn merge(&self, vals: impl IntoIterator<Item = I>, _meta: &M) -> Self::Output {
        let mut ordering_numbers = (f32::INFINITY, f32::INFINITY);

        for val in vals {
            let weight = self.0.ordering_of(&val);

            if weight < ordering_numbers.0 {
                ordering_numbers.1 = ordering_numbers.0;
                ordering_numbers.0 = weight;
            } else if weight < ordering_numbers.1 {
                ordering_numbers.1 = weight;
            }
        }

        [ordering_numbers.0, ordering_numbers.1].map(|v| self.0.relative_ordering(v))
    }
}

/// A merger that selects the weight of the value with the least weight.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct MinOrder<T>(pub T);

impl<I: NoiseType, M, T: Orderer<I>> Merger<I, M> for MinOrder<T> {
    type Output = T::OrderingOutput;

    #[inline]
    fn merge(&self, vals: impl IntoIterator<Item = I>, _meta: &M) -> Self::Output {
        let mut ordering_number = f32::INFINITY;
        for val in vals {
            let weight = self.0.ordering_of(&val);
            if weight < ordering_number {
                ordering_number = weight;
            }
        }

        self.0.relative_ordering(ordering_number)
    }
}

/// A merger that selects the value with the greatest weight.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct Max<T>(pub T);

impl<I: NoiseType + Default, M, T: Orderer<I>> Merger<I, M> for Max<T> {
    type Output = I;

    #[inline]
    fn merge(&self, vals: impl IntoIterator<Item = I>, _meta: &M) -> Self::Output {
        let mut ordering_number = f32::NEG_INFINITY;
        let mut result = I::default();
        for val in vals {
            let weight = self.0.ordering_of(&val);
            if weight > ordering_number {
                ordering_number = weight;
                result = val;
            }
        }

        result
    }
}

/// A merger that selects the index of the value with the greatest weight.
/// If you try to merge on an empty array, this will return zero.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct MaxIndex<T>(pub T);

impl<I: NoiseType, M, T: Orderer<I>> Merger<I, M> for MaxIndex<T> {
    type Output = usize;

    #[inline]
    fn merge(&self, vals: impl IntoIterator<Item = I>, _meta: &M) -> Self::Output {
        let mut ordering_number = f32::NEG_INFINITY;
        let mut result = 0;
        for (index, val) in vals.into_iter().enumerate() {
            let weight = self.0.ordering_of(&val);
            if weight > ordering_number {
                ordering_number = weight;
                result = index;
            }
        }

        result
    }
}

/// A merger that selects the indices of the 2 values with the greatest weights.
/// If you try to merge on an array shorter than 2, this will return zeros, where data is missing.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct MaxIndices<T>(pub T);

impl<I: NoiseType, M, T: Orderer<I>> Merger<I, M> for MaxIndices<T> {
    type Output = [usize; 2];

    #[inline]
    fn merge(&self, vals: impl IntoIterator<Item = I>, _meta: &M) -> Self::Output {
        let mut ordering_numbers = (f32::NEG_INFINITY, f32::NEG_INFINITY);
        let mut results = (0, 0);

        for (index, val) in vals.into_iter().enumerate() {
            let weight = self.0.ordering_of(&val);

            if weight > ordering_numbers.0 {
                ordering_numbers.1 = ordering_numbers.0;
                results.1 = results.0;
                ordering_numbers.0 = weight;
                results.0 = index;
            } else if weight > ordering_numbers.1 {
                ordering_numbers.1 = weight;
                results.1 = index;
            }
        }

        [results.0, results.1]
    }
}

/// A merger that selects the weights of the 2 values with the greatest weights.
/// If you try to merge on an array shorter than 2, this will return [`f32::NEG_INFINITY`], where
/// data is missing.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct MaxOrders<T>(pub T);

impl<I: NoiseType, M, T: Orderer<I>> Merger<I, M> for MaxOrders<T> {
    type Output = [T::OrderingOutput; 2];

    #[inline]
    fn merge(&self, vals: impl IntoIterator<Item = I>, _meta: &M) -> Self::Output {
        let mut ordering_numbers = (f32::NEG_INFINITY, f32::NEG_INFINITY);

        for val in vals {
            let weight = self.0.ordering_of(&val);

            if weight > ordering_numbers.0 {
                ordering_numbers.1 = ordering_numbers.0;
                ordering_numbers.0 = weight;
            } else if weight > ordering_numbers.1 {
                ordering_numbers.1 = weight;
            }
        }

        [ordering_numbers.0, ordering_numbers.1].map(|v| self.0.relative_ordering(v))
    }
}

/// A merger that selects the weight of the value with the greatest weight.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct MaxOrder<T>(pub T);

impl<I: NoiseType, M, T: Orderer<I>> Merger<I, M> for MaxOrder<T> {
    type Output = T::OrderingOutput;

    #[inline]
    fn merge(&self, vals: impl IntoIterator<Item = I>, _meta: &M) -> Self::Output {
        let mut ordering_number = f32::NEG_INFINITY;
        for val in vals {
            let weight = self.0.ordering_of(&val);
            if weight > ordering_number {
                ordering_number = weight;
            }
        }

        self.0.relative_ordering(ordering_number)
    }
}

/// A merger that averages the weights of all values. This will return 0 if there are no values
/// being merged.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct AverageOrders<T>(pub T);

impl<I: NoiseType, M, T: Orderer<I>> Merger<I, M> for AverageOrders<T> {
    type Output = T::OrderingOutput;

    #[inline]
    fn merge(&self, vals: impl IntoIterator<Item = I>, _meta: &M) -> Self::Output {
        let mut total = 0.0;
        let mut len = 0u32;
        for val in vals {
            len += 1;
            total += self.0.ordering_of(&val);
        }

        if len == 0 {
            self.0.relative_ordering(0.0)
        } else {
            self.0.relative_ordering(total / (len as f32))
        }
    }
}

/// A merger that merges values by assigning them weights.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Weighted<T>(pub T);

impl<I: NoiseType + Default, M, T: WeightFactorer<I>> Merger<I, M> for Weighted<T> {
    type Output = T::Output;

    #[inline]
    fn merge(&self, vals: impl IntoIterator<Item = I>, _meta: &M) -> Self::Output {

        // if vals.is_empty() {
        //     return self.0.weigh_value(I::default(), 1.0);
        // }

        // let mut total = 0f32;
        // for value in &vals {
        //     total += self.0.weight_of(value);
        // }
        // let inverse_total = if total == 0f32 { 0f32 } else { 1.0 / total };

        // let mut result = None;
        // for v in vals {
        //     let relative_weight = self.0.weight_of(&v) * inverse_total;
        //     let local = self.0.weigh_value(v, relative_weight);
        //     if let Some(result) = &mut result {
        //         *result += local;
        //     } else {
        //         result = Some(local)
        //     }
        // }

        // // SAFETY: we know vals is non-empty and that therefore on the first iteration and
        // // thereafter, result will be some.
        // unsafe { result.unwrap_unchecked() }
    }
}

/// A [`WeightFactorer`] that leverages an [`Orderer`] of type `O` to weigh values based on their
/// order. Those weights are then applied to the noise output of the [`NoiseOp`] `N`.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct OrderingWeight<O, N, const INVERTED: bool = true> {
    /// The [`Orderer`]
    pub orderer: O,
    /// The [`NoiseOp`]
    pub noise: N,
}

impl<I, O: Orderer<I, OrderingOutput = UNorm>, N: NoiseOp<I>, const INVERTED: bool>
    WeightFactorer<I> for OrderingWeight<O, N, INVERTED>
where
    N::Output: Mul<f32>,
    <N::Output as Mul<f32>>::Output: NoiseType + AddAssign,
{
    type Output = <N::Output as Mul<f32>>::Output;

    #[inline]
    fn weight_of(&self, value: &I) -> f32 {
        let standard = self
            .orderer
            .relative_ordering(self.orderer.ordering_of(value));
        if INVERTED {
            standard.inverse()
        } else {
            standard
        }
        .adapt()
    }

    #[inline]
    fn weigh_value(&self, value: I, relative_weight: f32) -> Self::Output {
        self.noise.get(value) * relative_weight
    }
}

/// A [`Merger`] that sums together values.
#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub struct Total;

impl<I: NoiseType + Default + Add<Output = I>, M> Merger<I, M> for Total {
    type Output = I;

    #[inline]
    fn merge(&self, vals: impl IntoIterator<Item = I>, _meta: &M) -> Self::Output {
        let mut vals = vals.into_iter();
        let Some(mut total) = vals.next() else {
            return I::default();
        };

        for v in vals {
            total = total + v;
        }

        total
    }
}

/// A [`Merger`] that multiplies together values.
#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub struct Product;

impl<I: NoiseType + Default + Mul<Output = I>, M> Merger<I, M> for Product {
    type Output = I;

    #[inline]
    fn merge(&self, vals: impl IntoIterator<Item = I>, _meta: &M) -> Self::Output {
        let mut vals = vals.into_iter();
        let Some(mut total) = vals.next() else {
            return I::default();
        };

        for v in vals {
            total = total * v;
        }

        total
    }
}

/// A noise operation that uses [`Merger`] `M` to merge any [`Mergeable`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Merged<M>(pub M);

impl<I: Mergeable, M: Merger<I::Part, I::Meta>> NoiseOp<I> for Merged<M> {
    type Output = M::Output;

    #[inline]
    fn get(&self, input: I) -> Self::Output {
        input.perform_merge(&self.0)
    }
}

impl<const N: usize, I: NoiseType, M: Merger<I, ()>> NoiseOp<[I; N]> for Merged<M> {
    type Output = M::Output;

    #[inline]
    fn get(&self, input: [I; N]) -> Self::Output {
        self.0.merge(input, &())
    }
}

impl Orderer<f32> for () {
    type OrderingOutput = f32;

    #[inline]
    fn ordering_of(&self, value: &f32) -> f32 {
        *value
    }

    #[inline]
    fn relative_ordering(&self, ordering: f32) -> Self::OrderingOutput {
        ordering
    }
}

impl WeightFactorer<f32> for () {
    type Output = f32;

    #[inline]
    fn weight_of(&self, value: &f32) -> f32 {
        *value
    }

    #[inline]
    fn weigh_value(&self, _value: f32, relative_weight: f32) -> Self::Output {
        relative_weight
    }
}

/// A [`Orderer`] and for "as the crow flyies" distance
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EuclideanDistance {
    /// represents the inverse of the maximum expected evaluation of this distance.
    pub inv_max_expected: f32,
}

/// A [`Orderer`] and for "manhatan" or diagonal distance
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ManhatanDistance {
    /// represents the inverse of the maximum expected evaluation of this distance.
    pub inv_max_expected: f32,
}

/// A [`Orderer`] that evenly combines [`EuclideanDistance`] and [`ManhatanDistance`]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HybridDistance {
    /// represents the inverse of the maximum expected evaluation of this distance.
    pub inv_max_expected: f32,
}

macro_rules! impl_distances {
    ($t:path) => {
        impl Orderer<$t> for EuclideanDistance {
            type OrderingOutput = UNorm;

            #[inline]
            fn ordering_of(&self, value: &$t) -> f32 {
                value.length_squared()
            }

            #[inline]
            fn relative_ordering(&self, ordering: f32) -> Self::OrderingOutput {
                UNorm::new_clamped(ordering.sqrt() * self.inv_max_expected)
            }
        }

        impl Orderer<$t> for ManhatanDistance {
            type OrderingOutput = UNorm;

            #[inline]
            fn ordering_of(&self, value: &$t) -> f32 {
                value.abs().element_sum()
            }

            #[inline]
            fn relative_ordering(&self, ordering: f32) -> Self::OrderingOutput {
                UNorm::new_clamped(ordering * self.inv_max_expected)
            }
        }

        // inspired by https://github.com/Auburn/FastNoiseLite/blob/master/Rust/src/lib.rs#L1825
        impl Orderer<$t> for HybridDistance {
            type OrderingOutput = UNorm;

            #[inline]
            fn ordering_of(&self, value: &$t) -> f32 {
                value.length_squared() + value.abs().element_sum()
            }

            #[inline]
            fn relative_ordering(&self, ordering: f32) -> Self::OrderingOutput {
                UNorm::new_clamped(ordering * self.inv_max_expected)
            }
        }
    };
}

impl_distances!(Vec2);
impl_distances!(Vec3);
impl_distances!(Vec4);
