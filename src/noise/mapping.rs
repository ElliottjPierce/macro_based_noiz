//! This module provides noise that letts you **map**, not convert, between signed and unsigned
//! integers.

use bevy_math::{
    I8Vec2,
    I8Vec3,
    I8Vec4,
    I16Vec2,
    I16Vec3,
    I16Vec4,
    I64Vec2,
    I64Vec3,
    I64Vec4,
    IVec2,
    IVec3,
    IVec4,
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
use crate::noise::white::{
    White8,
    White16,
    White32,
    White64,
    White128,
};

/// A noise that maps integers from signed to unsigned.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct MapExchange;

/// easily implement mapping for integers
macro_rules! impl_mapper {
    ($s:ty, $u:ty, $w:ty) => {
        impl Noise<$s> for MapExchange {
            type Output = $u;

            #[inline]
            fn sample(&self, input: $s) -> Self::Output {
                input as $u ^ (1 << (<$u>::BITS - 1))
            }
        }

        impl Noise<$s> for $w {
            type Output = <$w as Noise<$u>>::Output;

            #[inline]
            fn sample(&self, input: $s) -> Self::Output {
                self.sample(MapExchange.sample(input))
            }
        }
    };
}

impl_mapper!(i8, u8, White8);
impl_mapper!(i16, u16, White16);
impl_mapper!(i32, u32, White32);
impl_mapper!(i64, u64, White64);
impl_mapper!(i128, u128, White128);

/// easily implement mapping for integer vecs
macro_rules! impl_mapper_vec {
    ($s:ty, $u:ty, $w:ty) => {
        impl Noise<$s> for MapExchange {
            type Output = $u;

            #[inline]
            fn sample(&self, input: $s) -> Self::Output {
                <$u>::from_array(input.to_array().map(|v| self.sample(v)))
            }
        }

        impl Noise<$s> for $w {
            type Output = <$w as Noise<$u>>::Output;

            #[inline]
            fn sample(&self, input: $s) -> Self::Output {
                self.sample(MapExchange.sample(input))
            }
        }
    };
}

impl_mapper_vec!(I8Vec2, U8Vec2, White8);
impl_mapper_vec!(I8Vec3, U8Vec3, White8);
impl_mapper_vec!(I8Vec4, U8Vec4, White8);
impl_mapper_vec!(I16Vec2, U16Vec2, White16);
impl_mapper_vec!(I16Vec3, U16Vec3, White16);
impl_mapper_vec!(I16Vec4, U16Vec4, White16);
impl_mapper_vec!(IVec2, UVec2, White32);
impl_mapper_vec!(IVec3, UVec3, White32);
impl_mapper_vec!(IVec4, UVec4, White32);
impl_mapper_vec!(I64Vec2, U64Vec2, White64);
impl_mapper_vec!(I64Vec3, U64Vec3, White64);
impl_mapper_vec!(I64Vec4, U64Vec4, White64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_mapping() {
        assert_eq!(u32::MIN, MapExchange.sample(i32::MIN));
        assert_eq!(u32::MAX / 2 + 1, MapExchange.sample(0));
        assert_eq!(u32::MAX, MapExchange.sample(i32::MAX));
    }
}
