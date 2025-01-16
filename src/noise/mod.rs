//! This module contains all the noise itself

mod white;

/// This trait encapsulates what noise is. It takes in an input and outputs the nosie result.
pub trait Noise {
    /// represents the input to a noise function
    type Input;
    /// represents the output of a noise function
    type Output;

    /// Samples the noise at the specific input. This is generally inlined.
    fn sample(&self, input: Self::Input) -> Self::Output;

    /// The same as [sample](Self::sample), but not inlined.
    fn sample_cold(&self, input: Self::Input) -> Self::Output {
        self.sample(input)
    }
}
