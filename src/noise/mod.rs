//! This module contains all the noise itself

pub mod grid;
pub mod mapping;
pub mod scalar;
pub mod white;

/// This trait encapsulates what noise is. It takes in an input and outputs the nosie result.
pub trait NoiseOp<Input> {
    /// represents the output of a noise function
    type Output;

    /// Samples the noise at the specific input. This is generally inlined.
    fn get_raw(&self, input: Input) -> Self::Output;

    /// Samples the noise at the input. This is generally inlined.
    #[inline]
    fn get<T: NoiseMapped<Input>>(&self, input: T) -> Self::Output {
        self.get_raw(input.map())
    }

    /// The same as [sample](Self::get), but not inlined.
    fn get_cold<T: NoiseMapped<Input>>(&self, input: T) -> Self::Output {
        self.get(input)
    }
}

/// Allows this type to be mapped to type T for noise calculations.
pub trait NoiseMapped<T> {
    /// maps this value to a noise
    fn map(self) -> T;
}

impl<T> NoiseMapped<T> for T {
    #[inline]
    fn map(self) -> T {
        self
    }
}

/// Allows the chaining of multiple noise types
#[macro_export]
macro_rules! chain {
    ($base:expr) => {$base};
    ($base:expr, $($noise:expr),+) => {
        ($base, chain!($($noise),+))
    };
}
