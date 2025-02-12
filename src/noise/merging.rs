//! THis module allows noise types to be merged together

use std::ops::AddAssign;

use super::{
    NoiseOp,
    NoiseType,
};

/// Allows the noise type to be merged
pub trait Merger<I, M> {
    /// the merged output
    type Output: NoiseType;

    /// merges any number of the input type into an output
    fn merge<const N: usize>(&self, vals: [I; N], meta: &M) -> Self::Output;
}

/// Defines a type that is able to give weights to a particular kind of value.
pub trait WeightGiver<I> {
    /// Calculates the weight of the given value.
    fn weight_of(&self, value: &I) -> f32;
}

/// Defines a type that is able to weigh a given type of value relative to other weights
pub trait WeightFactorer<I>: WeightGiver<I> {
    /// The type that the weighing results in
    type Output: AddAssign + NoiseType;

    /// Given a value and it's relative weight in 0..=1 convert the value to the output
    fn weigh_value(&self, value: I, relative_weight: f32) -> Self::Output;
}

/// A merger that selects the value with the least weight.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Min;

impl<I: NoiseType + Default, M: WeightGiver<I>> Merger<I, M> for Min {
    type Output = I;

    #[inline]
    fn merge<const N: usize>(&self, vals: [I; N], meta: &M) -> Self::Output {
        let mut least_weight = f32::INFINITY;
        let mut result = I::default();
        for val in vals {
            let weight = meta.weight_of(&val);
            if weight < least_weight {
                least_weight = weight;
                result = val;
            }
        }

        result
    }
}

/// A merger that selects the weight of the value with the least weight.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct MinWeight;

impl<I: NoiseType + Default, M: WeightGiver<I>> Merger<I, M> for MinWeight {
    type Output = f32;

    #[inline]
    fn merge<const N: usize>(&self, vals: [I; N], meta: &M) -> Self::Output {
        let mut least_weight = f32::INFINITY;
        for val in vals {
            let weight = meta.weight_of(&val);
            if weight < least_weight {
                least_weight = weight;
            }
        }

        least_weight
    }
}

/// A merger that selects the value with the greatest weight.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Max;

impl<I: NoiseType + Default, M: WeightGiver<I>> Merger<I, M> for Max {
    type Output = I;

    #[inline]
    fn merge<const N: usize>(&self, vals: [I; N], meta: &M) -> Self::Output {
        let mut least_weight = f32::NEG_INFINITY;
        let mut result = I::default();
        for val in vals {
            let weight = meta.weight_of(&val);
            if weight > least_weight {
                least_weight = weight;
                result = val;
            }
        }

        result
    }
}

/// A merger that selects the weight of the value with the greatest weight.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct MaxWeight;

impl<I: NoiseType + Default, M: WeightGiver<I>> Merger<I, M> for MaxWeight {
    type Output = f32;

    #[inline]
    fn merge<const N: usize>(&self, vals: [I; N], meta: &M) -> Self::Output {
        let mut least_weight = f32::NEG_INFINITY;
        for val in vals {
            let weight = meta.weight_of(&val);
            if weight > least_weight {
                least_weight = weight;
            }
        }

        least_weight
    }
}

/// A merger that selects the least value.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Weighted;

impl<I: NoiseType + Default, M: WeightFactorer<I>> Merger<I, M> for Weighted {
    type Output = M::Output;

    #[inline]
    fn merge<const N: usize>(&self, vals: [I; N], meta: &M) -> Self::Output {
        if vals.is_empty() {
            return meta.weigh_value(I::default(), 1.0);
        }

        let mut total = 0f32;
        for value in &vals {
            total += meta.weight_of(value);
        }
        let inverse_total = if total == 0f32 { 0f32 } else { 1.0 / total };

        let mut result = None;
        for v in vals {
            let relative_weight = meta.weight_of(&v) * inverse_total;
            let local = meta.weigh_value(v, relative_weight);
            if let Some(result) = &mut result {
                *result += local;
            } else {
                result = Some(local)
            }
        }

        // SAFETY: we know vals is non-empty and that therefore on the first iteration and
        // thereafter, result will be some.
        unsafe { result.unwrap_unchecked() }
    }
}
