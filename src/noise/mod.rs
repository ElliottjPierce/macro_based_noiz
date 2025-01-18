//! This module contains all the noise itself

use std::marker::PhantomData;

pub mod grid;
pub mod mapping;
pub mod scalar;
pub mod white;

/// This trait encapsulates what noise is. It takes in an input and outputs the nosie result.
pub trait NoiseOp<I> {
    /// represents the output of a noise function
    type Output;

    /// Samples the noise at the specific input. This is generally inlined.
    fn get(&self, input: I) -> Self::Output;

    /// The same as [sample](Self::get), but not inlined.
    fn get_cold(&self, input: I) -> Self::Output {
        self.get(input)
    }
}

/// Signifies that these types are effectively the same as far as noise is concerned.
pub trait NoiseConvert<T> {
    /// maps this value to a noise
    fn convert(self) -> T;
}

/// marks this type as the potential result of some noise function.
pub trait NoiseResult {
    /// converts this value into a different type with a common noise goal.
    fn adapt<T>(self) -> T
    where
        Self: NoiseConvert<T> + Sized,
    {
        self.convert()
    }
}

impl<T> NoiseConvert<T> for T {
    #[inline]
    fn convert(self) -> T {
        self
    }
}

impl NoiseResult for u8 {}
impl NoiseResult for u16 {}
impl NoiseResult for u32 {}
impl NoiseResult for u64 {}
impl NoiseResult for u128 {}

/// Allows chaining noise functions together
struct Chain<I, N1: NoiseOp<I>, N2: NoiseOp<N1::Output>>(N1, N2, PhantomData<I>);

/// A noise operation that converts one noise type to another
struct Adapter<I, O>(PhantomData<(I, O)>)
where
    I: NoiseConvert<O>;

/// allows a function to be used as a noise operation
pub struct NoiseFn<I, O: NoiseResult, F: Fn(I) -> O>(F, PhantomData<(I, O)>);

impl<I, N1: NoiseOp<I>, N2: NoiseOp<N1::Output>> NoiseOp<I> for Chain<I, N1, N2> {
    type Output = N2::Output;

    #[inline]
    fn get(&self, input: I) -> Self::Output {
        self.1.get(self.0.get(input))
    }
}

impl<I, O> NoiseOp<I> for Adapter<I, O>
where
    I: NoiseConvert<O>,
{
    type Output = O;

    #[inline]
    fn get(&self, input: I) -> Self::Output {
        input.convert()
    }
}

impl<I, O: NoiseResult, F: Fn(I) -> O> NoiseOp<I> for NoiseFn<I, O, F> {
    type Output = O;

    #[inline]
    fn get(&self, input: I) -> Self::Output {
        self.0(input)
    }
}

/// Allows the chaining of multiple noise types
#[macro_export]
macro_rules! noise_fn {
    ($base:expr) => {$base};
    ($base:expr, $($noise:expr),+) => {
        ($base, chain!($($noise),+))
    };
}
