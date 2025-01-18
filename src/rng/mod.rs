//! This module allows noise driven randomness

use rand::{
    Error,
    RngCore,
};
use rand_core::impls;

use crate::noise::NoiseOp;

/// A 64-bit version of [`NoiseRng`]. Use this when you are working primarily with 64-bit numbers.
/// You may use this to generate seeds, etc. In general, [`NoiseRng`] is a better pick.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct NoiseRng64<N: NoiseOp<u64, Output = u64>>(N, u64);

impl<N: NoiseOp<u64, Output = u64>> RngCore for NoiseRng64<N> {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        let res = self.1;
        self.1 = self.0.get(self.1);
        res
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        impls::fill_bytes_via_next(self, dest)
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        #[allow(clippy::unit_arg)]
        Ok(self.fill_bytes(dest))
    }
}

impl<N: NoiseOp<u64, Output = u64>> NoiseRng64<N> {
    /// constructs a new rng with this noise and seed
    pub fn new_with(noise: N, seed: u64) -> Self {
        Self(noise, seed)
    }
}

impl<N: NoiseOp<u64, Output = u64> + Clone> NoiseRng64<N> {
    /// creates a new version of Self from this one
    pub fn break_off(&mut self) -> Self {
        let start = self.next_u64();
        Self(self.0.clone(), start.rotate_left(24)) // rotation just to desync the two generators
    }
}

/// A rng that uses a noise function as its randomizer. This operates on 32 bit noise, so it is a
/// good default RNG.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct NoiseRng<N: NoiseOp<u32, Output = u32>>(N, u32);

impl<N: NoiseOp<u32, Output = u32>> RngCore for NoiseRng<N> {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        let res = self.1;
        self.1 = self.0.get(self.1);
        res
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        ((self.next_u32() as u64) << 32) | self.next_u32() as u64
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        impls::fill_bytes_via_next(self, dest)
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        #[allow(clippy::unit_arg)]
        Ok(self.fill_bytes(dest))
    }
}

impl<N: NoiseOp<u32, Output = u32>> NoiseRng<N> {
    /// constructs a new rng with this noise and seed
    pub fn new_with(noise: N, seed: u32) -> Self {
        Self(noise, seed)
    }
}

impl<N: NoiseOp<u32, Output = u32> + Clone> NoiseRng<N> {
    /// creates a new version of Self from this one
    pub fn break_off(&mut self) -> Self {
        let start = self.next_u32();
        Self(self.0.clone(), start.rotate_left(12)) // rotation just to desync the two generators
    }
}
