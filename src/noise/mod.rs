//! This module contains all the noise itself

pub mod white;

/// This trait encapsulates what noise is. It takes in an input and outputs the nosie result.
pub trait Noise<Input> {
    /// represents the output of a noise function
    type Output;

    /// Samples the noise at the specific input. This is generally inlined.
    fn sample(&self, input: Input) -> Self::Output;

    /// The same as [sample](Self::sample), but not inlined.
    fn sample_cold(&self, input: Input) -> Self::Output {
        self.sample(input)
    }
}
