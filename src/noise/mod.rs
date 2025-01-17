//! This module contains all the noise itself

use bevy_math::Curve;
use rand_core::{
    Error,
    RngCore,
    impls,
};

pub mod mapping;
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

impl<I, N1: Noise<I>, N2: Noise<N1::Output>> Noise<I> for (N1, N2) {
    type Output = N2::Output;

    #[inline]
    fn sample(&self, input: I) -> Self::Output {
        self.1.sample(self.0.sample(input))
    }
}

impl<I, N1: Noise<I>, N2: Noise<N1::Output>, N3: Noise<N2::Output>> Noise<I> for (N1, N2, N3) {
    type Output = N3::Output;

    #[inline]
    fn sample(&self, input: I) -> Self::Output {
        self.2.sample(self.1.sample(self.0.sample(input)))
    }
}

impl<I, N1: Noise<I>, N2: Noise<N1::Output>, N3: Noise<N2::Output>, N4: Noise<N3::Output>> Noise<I>
    for (N1, N2, N3, N4)
{
    type Output = N4::Output;

    #[inline]
    fn sample(&self, input: I) -> Self::Output {
        self.3
            .sample(self.2.sample(self.1.sample(self.0.sample(input))))
    }
}

/// A value that stores an f32 in range (-1, 0)∪(0, 1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SNorm(f32);

/// A value that stores an f32 in range (0, 1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UNorm(f32);

impl SNorm {
    /// constructs an arbetrary but valid value using these bits. Retruns an additional byte of
    /// leftover bits not used in the calculation.
    #[inline]
    pub fn from_bits_with_entropy(bits: u32) -> (Self, u8) {
        let entropy = bits as u8; // takes the least significant 8 bits

        // the following is inspired by rand's Open01 implementation:
        use rand::distributions::hidden_export::IntoFloat;
        // Transmute-based method; 23/52 random bits; (0, 1) interval.
        // We use the most significant bits because for simple RNGs
        // those are usually more random.

        let mut raw = (bits >> 9).into_float_with_exponent(0) - (1.0 - f32::EPSILON / 2.0); // loose the 8 least significant
        raw = f32::from_bits(raw.to_bits() | ((bits & (1 << 8)) << 23)); // should lower to just moving the bit over.
        // to summarize, the 8 least bits went to entropy. The next least became the sign. The most
        // significant 23 bits became the float.
        (Self(raw), entropy)
    }

    /// conststructs an arbetrary but valid value from these bits.
    #[inline]
    pub fn from_bits(bits: u32) -> Self {
        Self::from_bits_with_entropy(bits).0
    }

    /// clamps the value into a valid SNorm
    #[inline]
    pub fn new_clamped(value: f32) -> Self {
        let unorm = value.abs().clamp(UNorm::MIN, UNorm::MAX);
        Self(f32::from_bits(
            unorm.to_bits() | (value.to_bits() & (1 << 31)),
        ))
    }

    /// constructs a new SNorm assuming the value is not zero.
    ///
    /// # Safety
    /// value MUST not be zero
    #[inline]
    pub unsafe fn new_non_zero(value: f32) -> Self {
        Self(value.clamp(-UNorm::MAX, UNorm::MAX))
    }

    /// constructs a new SNorm  assuming the value is in (-1, 1).
    ///
    /// # Safety
    /// value MUST be in (-1, 1)
    #[inline]
    pub unsafe fn new_in_bounds(value: f32) -> Self {
        Self(f32::from_bits(value.to_bits() | 1)) // technically causes a minute change to the value, but saves an instruction
    }

    /// constructs a new SNorm assuming value is in bounds.
    ///
    /// # Safety
    /// value MUST be garenteed to be in (-1, 1) and not be 0
    #[inline]
    pub unsafe fn new_unchecked(value: f32) -> Self {
        Self(value)
    }

    /// inverts the value. Equivilent to -value
    #[inline]
    pub fn inverse(self) -> Self {
        Self(-self.0)
    }

    /// constructs a new value after passing through the curve
    pub fn remap(self, curve: &impl Curve<f32>) -> Self {
        Self::new_clamped(curve.sample_clamped(self.0))
    }

    /// interprets this value an a new scale by multiplication
    #[inline]
    pub fn scale(self, scale: f32) -> f32 {
        self.0 * scale
    }

    /// smoothly maps this value onto a UNorm
    #[inline]
    pub fn map_to_unorm(self) -> UNorm {
        // SAFETY: we know it is in bounds, but a value of -1 could create a zero
        unsafe { UNorm::new_in_bounds(self.0 * 0.5 + 0.5) }
    }

    /// splits the value, converting it to a UNorm,
    #[inline]
    pub fn split_to_unorm(self) -> UNorm {
        // SAFETY: there is no way the value can change. The sign just becomes positive.
        unsafe { UNorm::new_unchecked(self.0.abs()) }
    }

    /// creates a sharp, mirrord division at 0
    #[inline]
    pub fn split_even(self) -> Self {
        self.split_to_unorm().map_to_snorm()
    }

    /// creates sharp jumps
    #[inline]
    pub fn jump(self, jumps: f32) -> Self {
        Self::new_clamped((self.0 * jumps).fract())
    }
}

impl UNorm {
    /// The smallest valid value
    const MIN: f32 = f32::MIN_POSITIVE;
    /// The greatest valid value
    const MAX: f32 = 1.0 - f32::EPSILON;

    /// constructs an arbetrary but valid value using these bits. Retruns an additional byte of
    /// leftover bits not used in the calculation.
    #[inline]
    pub fn from_bits_with_entropy(bits: u32) -> (Self, u8) {
        let (signed, entropy) = SNorm::from_bits_with_entropy(bits);
        (Self(signed.0.abs()), entropy)
    }

    /// conststructs an arbetrary but valid value from these bits.
    #[inline]
    pub fn from_bits(bits: u32) -> Self {
        Self::from_bits_with_entropy(bits).0
    }

    /// clamps the value into a valid UNorm
    #[inline]
    pub fn new_clamped(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    /// constructs a new UNorm assuming value is in bounds.
    ///
    /// # Safety
    /// value MUST be garenteed to be in (0, 1)
    #[inline]
    pub unsafe fn new_unchecked(value: f32) -> Self {
        Self(value)
    }

    /// constructs a new UNorm assuming value is in positive.
    ///
    /// # Safety
    /// value MUST be > 0
    #[inline]
    pub unsafe fn new_positive(value: f32) -> Self {
        Self(value.min(Self::MAX))
    }

    /// constructs a new UNorm assuming value is in less than 1.
    ///
    /// # Safety
    /// value MUST be < 1
    #[inline]
    pub unsafe fn new_small(value: f32) -> Self {
        Self(value.max(Self::MIN))
    }

    /// constructs a SNorm assuming the value is in [0, 1).
    ///
    /// # Safety
    /// value MUST be in [0, 1)
    #[inline]
    pub unsafe fn new_in_bounds(value: f32) -> Self {
        Self(f32::from_bits(value.to_bits() | 1)) // technically causes a minute change to the value, but saves an instruction
    }

    /// inverts the value. Equivilent to 1 - value
    #[inline]
    pub fn inverse(self) -> Self {
        Self(1.0 - self.0)
    }

    /// constructs a new value after passing through the curve
    pub fn remap(self, curve: &impl Curve<f32>) -> Self {
        Self::new_clamped(curve.sample_clamped(self.0))
    }

    /// interprets this value an a new scale by multiplication
    #[inline]
    pub fn scale(self, scale: f32) -> f32 {
        self.0 * scale
    }

    /// smoothly maps this value onto a SNorm
    #[inline]
    pub fn map_to_snorm(self) -> SNorm {
        // SAFETY: we know it is in bounds, but a value of .5 could create a zero
        unsafe { SNorm::new_in_bounds(self.0 * 2.0 - 1.0) }
    }

    /// creates a sharp, mirrord division at 0.5
    #[inline]
    pub fn split_even(self) -> Self {
        // SAFETY: the could produce a value of zero
        unsafe { Self::new_small((self.0 - 0.5).abs() * 2.0) }
    }

    /// creates sharp jumps
    #[inline]
    pub fn jump(self, jumps: f32) -> Self {
        Self::new_clamped((self.0 * jumps).fract())
    }

    /// populates a u8 based on this value
    #[inline]
    pub fn fill_u8(self) -> u8 {
        let val = self.scale(256.0).floor();
        val as u8
    }

    /// populates a u16 based on this value
    #[inline]
    pub fn fill_u16(self) -> u16 {
        let val = self.scale(u16::MAX as f32 + 1.0).floor();
        val as u16
    }

    /// constructs a valid UNorm from this value
    #[inline]
    pub fn from_u8(value: u8) -> Self {
        // SAFETY: this may be 1 if value was 255, so we need to clamp it
        unsafe { Self::new_positive((value as f32 + 1.0) / 256.0) }
    }

    /// constructs a valid UNorm from this value
    #[inline]
    pub fn from_u16(value: u16) -> Self {
        // SAFETY: this may be 1 if value was u16 max, so we need to clamp it
        unsafe { Self::new_positive((value as f32 + 1.0) / u16::MAX as f32) }
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

/// A 64-bit version of [`NoiseRng`]. Use this when you are working primarily with 64-bit numbers.
/// You may use this to generate seeds, etc. In general, [`NoiseRng`] is a better pick.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct NoiseRng64<N: Noise<u64, Output = u64>>(N, u64);

impl<N: Noise<u64, Output = u64>> RngCore for NoiseRng64<N> {
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_u64(&mut self) -> u64 {
        let res = self.1;
        self.1 = self.0.sample(self.1);
        res
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        impls::fill_bytes_via_next(self, dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        #[allow(clippy::unit_arg)]
        Ok(self.fill_bytes(dest))
    }
}

impl<N: Noise<u64, Output = u64>> NoiseRng64<N> {
    /// constructs a new rng with this noise and seed
    pub fn new_with(noise: N, seed: u64) -> Self {
        Self(noise, seed)
    }
}

impl<N: Noise<u64, Output = u64> + Clone> NoiseRng64<N> {
    /// creates a new version of Self from this one
    pub fn break_off(&mut self) -> Self {
        let start = self.next_u64();
        Self(self.0.clone(), start.rotate_left(24)) // rotation just to desync the two generators
    }
}

/// A rng that uses a noise function as its randomizer. This operates on 32 bit noise, so it is a
/// good default RNG.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct NoiseRng<N: Noise<u32, Output = u32>>(N, u32);

impl<N: Noise<u32, Output = u32>> RngCore for NoiseRng<N> {
    fn next_u32(&mut self) -> u32 {
        let res = self.1;
        self.1 = self.0.sample(self.1);
        res
    }

    fn next_u64(&mut self) -> u64 {
        ((self.next_u32() as u64) << 32) | self.next_u32() as u64
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        impls::fill_bytes_via_next(self, dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        #[allow(clippy::unit_arg)]
        Ok(self.fill_bytes(dest))
    }
}

impl<N: Noise<u32, Output = u32>> NoiseRng<N> {
    /// constructs a new rng with this noise and seed
    pub fn new_with(noise: N, seed: u32) -> Self {
        Self(noise, seed)
    }
}

impl<N: Noise<u32, Output = u32> + Clone> NoiseRng<N> {
    /// creates a new version of Self from this one
    pub fn break_off(&mut self) -> Self {
        let start = self.next_u32();
        Self(self.0.clone(), start.rotate_left(12)) // rotation just to desync the two generators
    }
}
