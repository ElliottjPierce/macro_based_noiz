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

use super::NoiseType;
use crate::convertible;

/// easily implement mapping for integers
macro_rules! impl_mapper {
    ($s:ty, $u:ty) => {
        convertible!($s = $u, |source| source as $u ^ (1 << (<$u>::BITS - 1)));
    };
}

impl_mapper!(i8, u8);
impl_mapper!(i16, u16);
impl_mapper!(i32, u32);
impl_mapper!(i64, u64);
impl_mapper!(i128, u128);

/// easily implement mapping for integer vecs
macro_rules! impl_mapper_vec {
    ($s:ty, $u:ty) => {
        convertible!($s = $u, |source| <$u>::from_array(
            source.to_array().map(|v| v.adapt())
        ));
    };
}

impl_mapper_vec!(I8Vec2, U8Vec2);
impl_mapper_vec!(I8Vec3, U8Vec3);
impl_mapper_vec!(I8Vec4, U8Vec4);
impl_mapper_vec!(I16Vec2, U16Vec2);
impl_mapper_vec!(I16Vec3, U16Vec3);
impl_mapper_vec!(I16Vec4, U16Vec4);
impl_mapper_vec!(IVec2, UVec2);
impl_mapper_vec!(IVec3, UVec3);
impl_mapper_vec!(IVec4, UVec4);
impl_mapper_vec!(I64Vec2, U64Vec2);
impl_mapper_vec!(I64Vec3, U64Vec3);
impl_mapper_vec!(I64Vec4, U64Vec4);

#[cfg(test)]
mod tests {
    use crate::noise::NoiseType;

    #[test]
    fn check_mapping() {
        assert_eq!(u32::MIN, i32::MIN.adapt::<u32>());
        assert_eq!(u32::MAX / 2 + 1, 0i32.adapt::<u32>());
        assert_eq!(u32::MAX, i32::MAX.adapt::<u32>());
    }
}
