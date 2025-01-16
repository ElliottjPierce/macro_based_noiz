//! This module implements white noise inspiered by the [FxHash](https://crates.io/crates/fxhash)

use std::ops::BitXor;

use bevy_math::{
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

use super::Noise;

/// This creates a white noise implementation
macro_rules! impl_white {
    ($dt:ty, $name:ident, $key:expr, $(($input:ty, $conv:ty)),* $(,),*) => {
        /// A seeded RNG inspired by [FxHash](https://crates.io/crates/fxhash)
        pub struct $name(pub $dt);

        $(
            impl Noise< $input > for $name {
                type Output = $dt;

                #[inline(always)]
                fn sample(&self, input: $input) -> Self::Output {
                    let mut val: $dt = self.0;
                    let inner: $conv = input.into();
                    #[allow(for_loops_over_fallibles)] // this lets you use option to work on just one input
                    for v in inner {
                        // multiplication should be pipelined as a vector for v. xor to combine.
                        val = v.wrapping_mul($key).bitxor(val)
                    }
                    val.rotate_left(5) // multiplying large numbers like this tends to put more entorpy on the more significant bits. This pushes that entropy to the least segnificant.
                }
            }
        )*
    };
}

// uses some very large primes I found on the internet
impl_white!(
    u8,
    White8,
    251,
    (u8, Option<u8>),
    (U8Vec2, [u8; 2]),
    (U8Vec3, [u8; 3]),
    (U8Vec4, [u8; 4]),
    ([u8; 2], [u8; 2]),
    ([u8; 3], [u8; 3]),
    ([u8; 4], [u8; 4]),
    (Vec<u8>, Vec<u8>),
);
impl_white!(
    u16,
    White16,
    65521,
    (u16, Option<u16>),
    (U16Vec2, [u16; 2]),
    (U16Vec3, [u16; 3]),
    (U16Vec4, [u16; 4]),
    ([u16; 2], [u16; 2]),
    ([u16; 3], [u16; 3]),
    ([u16; 4], [u16; 4]),
    (Vec<u16>, Vec<u16>),
);
impl_white!(
    u32,
    White32,
    4294967279,
    (u32, Option<u32>),
    (UVec2, [u32; 2]),
    (UVec3, [u32; 3]),
    (UVec4, [u32; 4]),
    ([u32; 2], [u32; 2]),
    ([u32; 3], [u32; 3]),
    ([u32; 4], [u32; 4]),
    (Vec<u32>, Vec<u32>),
);
impl_white!(
    u64,
    White64,
    18446744073709551557,
    (u64, Option<u64>),
    (U64Vec2, [u64; 2]),
    (U64Vec3, [u64; 3]),
    (U64Vec4, [u64; 4]),
    ([u64; 2], [u64; 2]),
    ([u64; 3], [u64; 3]),
    ([u64; 4], [u64; 4]),
    (Vec<u64>, Vec<u64>),
);

#[cfg(target_pointer_width = "32")]
impl_white!(
    usize,
    WhiteUsize,
    4294967279,
    (usize, Option<usize>),
    ([usize; 2], [usize; 2]),
    ([usize; 3], [usize; 3]),
    ([usize; 4], [usize; 4]),
    (Vec<usize>, Vec<usize>),
);
#[cfg(target_pointer_width = "64")]
impl_white!(
    usize,
    WhiteUsize,
    18446744073709551557,
    (usize, Option<usize>),
    ([usize; 2], [usize; 2]),
    ([usize; 3], [usize; 3]),
    ([usize; 4], [usize; 4]),
    (Vec<usize>, Vec<usize>),
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_u32() {
        let rng = White32(5);
        let _tmp = rng.sample(8);
        let _tmp = rng.sample([8, 2]);
        let _tmp = rng.sample([8, 2, 4]);
        let _tmp = rng.sample([8, 2, 9, 3]);
        let _tmp = rng.sample(vec![1, 2, 3, 4, 5]);
        let _tmp = rng.sample(UVec2::new(1, 2));
        let _tmp = rng.sample(UVec3::new(1, 2, 3));
        let _tmp = rng.sample(UVec4::new(1, 2, 3, 4));
    }
}
