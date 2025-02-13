//! This module facilatites scalar noise results

use bevy_math::Curve;

use super::NoiseType;
use crate::convertible;

/// A value that stores an f32 in range (-1, 0)∪(0, 1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SNorm(f32);

/// A value that stores an f32 in range (0, 1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UNorm(f32);

impl SNorm {
    /// constructs an arbetrary but valid value using these bits. Returns an additional byte of
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
        Self(take_sign(value.abs().clamp(UNorm::MIN, UNorm::MAX), value))
    }

    /// creates a new [`SNorm`] by rolling the input by fractions. Values an integer apart will be
    /// the same.
    #[inline]
    pub fn new_rolling(value: f32) -> Self {
        // SAFETY: could be zero if its an integer number
        unsafe { Self::new_in_bounds(value.fract()) }
    }

    /// the higher the value is, the closer to zero this gets. At `value == half_life` this will be
    /// 0.5.
    #[inline]
    pub fn new_decay(value: f32, half_life: f32) -> Self {
        // SAFETY: bounds satisfied by UNorm
        unsafe { Self::new_unchecked(take_sign(UNorm::new_decay(value, half_life).0, value)) }
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
        Self(make_nonzero_f32(value))
    }

    /// constructs a new SNorm assuming value is in bounds.
    ///
    /// # Safety
    /// value MUST be guaranteed to be in (-1, 1) and not be 0
    #[inline]
    pub unsafe fn new_unchecked(value: f32) -> Self {
        Self(value)
    }

    /// inverts the value. Equivalent to -value
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

    /// creates a sharp, mirrored division at 0
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

    /// constructs an arbetrary but valid value using these bits. Returns an additional byte of
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

    /// creates a new [`UNorm`] by rolling the input by fractions. Values an integer apart will be
    /// the same.
    #[inline]
    pub fn new_rolling(value: f32) -> Self {
        // SAFETY: could be zero but x - floor(x) is always positive
        unsafe { Self::new_positive(value - value.floor()) }
    }

    /// the higher the value is, the closer to zero this gets. At `value == half_life` this will be
    /// 0.5
    #[inline]
    pub fn new_decay(value: f32, half_life: f32) -> Self {
        let decay = half_life.abs();
        Self::new_clamped(decay / (value.abs() + decay))
    }

    /// constructs a new UNorm assuming value is in bounds.
    ///
    /// # Safety
    /// value MUST be guaranteed to be in (0, 1)
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
        Self(make_nonzero_f32(value))
    }

    /// inverts the value. Equivalent to 1 - value
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

    /// creates a sharp, mirrored division at 0.5
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

convertible!(u32 = UNorm, |source| UNorm::from_bits(source));
convertible!(u32 = SNorm, |source| SNorm::from_bits(source));

convertible!(f32 = UNorm, |source| UNorm::new_rolling(source));
convertible!(f32 = SNorm, |source| SNorm::new_rolling(source));

convertible!(UNorm = f32, |source| source.0);
convertible!(SNorm = f32, |source| source.0);

convertible!(SNorm = UNorm, |source| SNorm::map_to_unorm(source));
convertible!(UNorm = SNorm, |source| UNorm::map_to_snorm(source));

impl NoiseType for SNorm {}
impl NoiseType for UNorm {}

/// forces the f32 to be nonzero by forcing on the least significant bit.
#[inline]
pub const fn make_nonzero_f32(v: f32) -> f32 {
    f32::from_bits(v.to_bits() | 1)
}

/// forces the value to take on the sign of another
#[inline]
pub const fn take_sign(v: f32, sign: f32) -> f32 {
    f32::from_bits(v.to_bits() | (sign.to_bits() & (1 << 31)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_non_zero() {
        assert_ne!(0f32, make_nonzero_f32(0.0));
    }
}
