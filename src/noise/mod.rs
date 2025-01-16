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

/// A value that stores an f32 in range (-1, 0)∪(0, 1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SNorm(f32);

/// A value that stores an f32 in range (0, 1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UNorm(f32);

impl SNorm {
    /// constructs an arbetrary but valid value using these bits
    pub fn from_bits(bits: u32) -> Self {
        // the following is inspired by rand's Open01 implementation:

        use rand::distributions::hidden_export::IntoFloat;
        // Transmute-based method; 23/52 random bits; (0, 1) interval.
        // We use the most significant bits because for simple RNGs
        // those are usually more random.
        let raw = bits.into_float_with_exponent(0) - (1.0 - f32::EPSILON / 2.0);
        Self(raw)
    }
}

impl UNorm {
    /// constructs an arbetrary but valid value using these bits
    pub fn from_bits(mut bits: u32) -> Self {
        bits &= !(1 << 31); // ensures float's sign bit isn't on
        Self(SNorm::from_bits(bits).0)
    }
}
