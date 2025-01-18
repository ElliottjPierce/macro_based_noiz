//! This module contains all the noise itself

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

/// Allows the chaining of multiple noise types
#[macro_export]
macro_rules! chain {
    ($base:expr) => {$base};
    ($base:expr, $($noise:expr),+) => {
        ($base, chain!($($noise),+))
    };
}
