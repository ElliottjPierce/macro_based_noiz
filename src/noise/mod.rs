//! This module contains all the noise itself

use std::marker::PhantomData;

use bevy_math::{
    I8Vec2,
    I8Vec3,
    I8Vec4,
    I16Vec2,
    I16Vec3,
    I16Vec4,
    I64Vec2,
    I64Vec3,
    I64Vec4,
    IVec2,
    IVec3,
    IVec4,
    U8Vec2,
    U8Vec3,
    U8Vec4,
    U16Vec2,
    U16Vec3,
    U16Vec4,
    U64Vec2,
    U64Vec3,
    U64Vec4,
    UVec2,
    UVec3,
    UVec4,
};

pub mod grid;
pub mod mapping;
pub mod scalar;
pub mod white;

/// This trait encapsulates what noise is. It takes in an input and outputs the nosie result.
pub trait NoiseOp<I> {
    /// represents the output of a noise function
    type Output: NoiseResult;

    /// Samples the noise at the specific input. This is generally inlined.
    fn get(&self, input: I) -> Self::Output;

    /// The same as [sample](Self::get), but not inlined.
    fn get_cold(&self, input: I) -> Self::Output {
        self.get(input)
    }
}

/// Signifies that these types are effectively the same as far as noise is concerned.
pub trait NoiseConvert<T: NoiseResult>: NoiseResult {
    /// maps this value to a noise
    fn convert(self) -> T;
}

/// marks this type as the potential result of some noise function.
pub trait NoiseResult {
    /// converts this value into a different type with a common noise goal.
    fn adapt<T: NoiseResult>(self) -> T
    where
        Self: NoiseConvert<T> + Sized,
    {
        self.convert()
    }
}

// built in
impl NoiseResult for u8 {}
impl NoiseResult for u16 {}
impl NoiseResult for u32 {}
impl NoiseResult for u64 {}
impl NoiseResult for u128 {}
impl NoiseResult for usize {}
impl NoiseResult for i8 {}
impl NoiseResult for i16 {}
impl NoiseResult for i32 {}
impl NoiseResult for i64 {}
impl NoiseResult for i128 {}
impl NoiseResult for isize {}
// bevy
impl NoiseResult for I8Vec2 {}
impl NoiseResult for I8Vec3 {}
impl NoiseResult for I8Vec4 {}
impl NoiseResult for I16Vec2 {}
impl NoiseResult for I16Vec3 {}
impl NoiseResult for I16Vec4 {}
impl NoiseResult for I64Vec2 {}
impl NoiseResult for I64Vec3 {}
impl NoiseResult for I64Vec4 {}
impl NoiseResult for IVec2 {}
impl NoiseResult for IVec3 {}
impl NoiseResult for IVec4 {}
impl NoiseResult for U8Vec2 {}
impl NoiseResult for U8Vec3 {}
impl NoiseResult for U8Vec4 {}
impl NoiseResult for U16Vec2 {}
impl NoiseResult for U16Vec3 {}
impl NoiseResult for U16Vec4 {}
impl NoiseResult for U64Vec2 {}
impl NoiseResult for U64Vec3 {}
impl NoiseResult for U64Vec4 {}
impl NoiseResult for UVec2 {}
impl NoiseResult for UVec3 {}
impl NoiseResult for UVec4 {}

impl<T: NoiseResult> NoiseConvert<T> for T {
    #[inline]
    fn convert(self) -> T {
        self
    }
}

/// Allows chaining noise functions together
struct Chain<I, N1: NoiseOp<I>, N2: NoiseOp<N1::Output>>(N1, N2, PhantomData<I>);

/// A noise operation that converts one noise type to another
struct Adapter<I: NoiseResult, O: NoiseResult>(PhantomData<(I, O)>)
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

impl<I: NoiseResult, O: NoiseResult> NoiseOp<I> for Adapter<I, O>
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
