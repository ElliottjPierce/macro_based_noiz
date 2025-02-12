//! This module implements white noise inspiered by the [FxHash](https://crates.io/crates/fxhash)

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

use super::NoiseOp;

/// This creates a white noise implementation
macro_rules! impl_white {
    ($dt:ty, $name:ident, $key:expr, $(($input:ty, $conv:ty)),* $(,),*) => {
        /// A seeded RNG inspired by [FxHash](https://crates.io/crates/fxhash)
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
        pub struct $name(pub $dt);

        impl<const N: usize> NoiseOp< [$dt; N] > for $name {
            type Output = $dt;

            #[inline(always)]
            fn get(&self, input: [$dt; N]) -> Self::Output {
                let slice: &[$dt] = &input;
                self.get(slice)
            }
        }

        impl NoiseOp< Vec<$dt> > for $name {
            type Output = $dt;

            #[inline(always)]
            fn get(&self, input: Vec<$dt>) -> Self::Output {
                let slice: &[$dt] = &input;
                self.get(slice)
            }
        }

        impl NoiseOp< Option<$dt> > for $name {
            type Output = $dt;

            #[inline(always)]
            fn get(&self, input: Option<$dt>) -> Self::Output {
                if let Some(input) = input {
                    self.get([input])
                } else {
                    $key
                }
            }
        }

        impl NoiseOp<$dt> for $name {
            type Output = $dt;

            #[inline(always)]
            fn get(&self, input: $dt) -> Self::Output {
                    (input ^ self.0) // salt with the seed
                    .wrapping_mul($key) // multiply to remove any linear artifacts
                    .rotate_left(5) // multiplying large numbers like this tends to put more entropy on the more significant bits. This pushes that entropy to the least segnificant.
            }
        }

        impl NoiseOp<&'_ [$dt]> for $name {
            type Output = $dt;

            #[inline(always)]
            fn get(&self, input: &[$dt]) -> Self::Output {
                let mut val: $dt = $key;
                for v in input {
                    val = v.wrapping_mul(val) ^ $key // need xor to make it non-commutative to remove diagonal lines and multiplication to put each dimension on separate roders
                }
                self.get(val)
            }
        }

        $(
            impl NoiseOp< $input > for $name {
                type Output = $dt;

                #[inline(always)]
                fn get(&self, input: $input) -> Self::Output {
                    let inner: $conv = input.into();
                    self.get(inner)
                }
            }
        )*
    };
}

// uses some very large primes I found on the internet
impl_white!(
    u8,
    White8,
    97,
    (U8Vec2, [u8; 2]),
    (U8Vec3, [u8; 3]),
    (U8Vec4, [u8; 4]),
);
impl_white!(
    u16,
    White16,
    1777,
    (U16Vec2, [u16; 2]),
    (U16Vec3, [u16; 3]),
    (U16Vec4, [u16; 4]),
);
impl_white!(
    u32,
    White32,
    104_395_303,
    (UVec2, [u32; 2]),
    (UVec3, [u32; 3]),
    (UVec4, [u32; 4]),
);
impl_white!(
    u64,
    White64,
    982_451_653,
    (U64Vec2, [u64; 2]),
    (U64Vec3, [u64; 3]),
    (U64Vec4, [u64; 4]),
);

impl_white!(u128, White128, 982_451_653_011,);

#[cfg(target_pointer_width = "32")]
impl_white!(usize, WhiteUsize, 104_395_303,);
#[cfg(target_pointer_width = "64")]
impl_white!(usize, WhiteUsize, 982_451_653,);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_u32() {
        let rng = White32(5);
        let _tmp = rng.get(8);
        let _tmp = rng.get([8, 2]);
        let _tmp = rng.get([8, 2, 4]);
        let _tmp = rng.get([8, 2, 9, 3]);
        let _tmp = rng.get(vec![1, 2, 3, 4, 5]);
        let _tmp = rng.get(UVec2::new(1, 2));
        let _tmp = rng.get(UVec3::new(1, 2, 3));
        let _tmp = rng.get(UVec4::new(1, 2, 3, 4));
    }
}
