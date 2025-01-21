//! THis module allows noise types to be merged together

use std::{
    marker::PhantomData,
    ops::{
        AddAssign,
        Div,
        Mul,
    },
};

use super::{
    ConversionChain,
    NoiseOp,
    NoiseType,
    grid::{
        GridPoint2,
        GridPoint3,
        GridPoint4,
        GridPointD2,
        GridPointD3,
        GridPointD4,
    },
};

/// Allows the noise type to be merged
pub trait Merger<I> {
    /// the merged output
    type Output: NoiseType;

    /// merges any number of the input type into an output
    fn merge<const N: usize>(&self, vals: [I; N]) -> Self::Output;
}

/// a noise type to smooth out grid noise
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Merged<
    M,
    I: ConversionChain,
    N: NoiseOp<I::Output>,
    O: ConversionChain<Input = N::Output>,
> {
    /// the way we are merging
    merger: M,
    /// the noise we are merging
    noise: N,
    /// phantom data
    marker: PhantomData<(I, O)>,
}

/// A merger that selects the least value.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Nearest;

impl<C: PartialOrd, T: NoiseType + Default> Merger<(C, T)> for Nearest
where
    (C, T): Clone,
{
    type Output = T;

    #[inline]
    fn merge<const N: usize>(&self, vals: [(C, T); N]) -> Self::Output {
        if vals.is_empty() {
            return T::default();
        }

        let (mut least, mut result) = vals[0].clone();
        for (c, t) in vals {
            if c < least {
                least = c;
                result = t;
            }
        }

        result
    }
}

/// A merger that selects the least value.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Weighted;

impl<
    C: AddAssign + Div<Output = C> + Default + Copy,
    T: NoiseType + Default + AddAssign + Mul<C, Output = T> + Copy,
> Merger<(C, T)> for Weighted
{
    type Output = T;

    #[inline]
    fn merge<const N: usize>(&self, vals: [(C, T); N]) -> Self::Output {
        if vals.is_empty() {
            return T::default();
        }

        let mut total = C::default();

        for (c, _t) in &vals {
            total += *c
        }

        let first = vals[0];
        let mut result = first.1 * (first.0 / total);
        for (c, t) in &vals[1..] {
            result += *t * (*c / total);
        }

        result
    }
}

/// allows implementing easily merge for different types
macro_rules! impl_merge {
    ($t:path, $f:path, $new:ident) => {
        impl<
            M: Merger<($f, O::Output)>,
            I: ConversionChain<Input = $t>,
            N: NoiseOp<I::Output>,
            O: ConversionChain<Input = N::Output>,
        > NoiseOp<$t> for Merged<M, I, N, O>
        {
            type Output = M::Output;

            #[inline]
            fn get(&self, input: $t) -> Self::Output {
                let values = input
                    .corners()
                    .map(|c| (c.offset.length(), O::convert(self.noise.get(I::convert(c)))));
                self.merger.merge(values)
            }
        }

        impl<
            M: Merger<($f, O::Output)>,
            I: ConversionChain<Input = $t>,
            N: NoiseOp<I::Output>,
            O: ConversionChain<Input = N::Output>,
        > Merged<M, I, N, O>
        {
            /// constructs a new [`Merged`]
            pub fn $new(merger: M, noise: N) -> Self {
                Self {
                    merger,
                    noise,
                    marker: PhantomData,
                }
            }
        }
    };
}

impl_merge!(GridPoint2, f32, new_vec2);
impl_merge!(GridPoint3, f32, new_vec3);
impl_merge!(GridPoint4, f32, new_vec4);
impl_merge!(GridPointD2, f64, new_dvec2);
impl_merge!(GridPointD3, f64, new_dvec3);
impl_merge!(GridPointD4, f64, new_dvec4);
