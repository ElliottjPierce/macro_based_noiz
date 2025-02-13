//! Allows noise types to be given a seed.

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

use super::{
    NoiseConvert,
    NoiseOp,
    NoiseType,
    white::{
        White8,
        White16,
        White32,
        White64,
        White128,
    },
};

/// Marks the type as being able to be given aseed. For example, grid points implement this so that
/// each cell in a grid can have a unique seed.
pub trait SeedableNoiseType: NoiseType {
    /// generates the seed for the group of noise values that this value is in. The seed is used to
    /// produce entropy.
    fn generate_seed(&self, seed: u32) -> u32;
}

/// Represents a type that has been given a seed for quick access.
pub struct Seeded<T: NoiseType> {
    /// the value
    pub value: T,
    /// the seed for the value
    pub seed: u32,
}

/// A noise operation that produces a [`Seeded`] version of any value that is passed into it,
/// provided it implements [`SeedableNoiseType`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seeding {
    /// the seed used to produce the seed in each value passed in.
    pub seed: u32,
}

impl<T: NoiseType> NoiseType for Seeded<T> {}

impl<T: SeedableNoiseType> NoiseOp<T> for Seeding {
    type Output = Seeded<T>;

    fn get(&self, input: T) -> Self::Output {
        Seeded {
            seed: input.generate_seed(self.seed),
            value: input,
        }
    }
}

impl<T: NoiseType> NoiseConvert<T> for Seeded<T> {
    fn convert(self) -> T {
        self.value
    }
}

impl<T: NoiseType> Seeded<T> {
    /// Maps this value to another, keeping its seed.
    #[inline]
    pub fn map<O: NoiseType>(self, f: impl FnOnce(T) -> O) -> Seeded<O> {
        Seeded {
            value: f(self.value),
            seed: self.seed,
        }
    }

    /// Maps this value to another, keeping its seed.
    #[inline]
    pub fn map_ref<O: NoiseType>(&self, f: impl FnOnce(&T) -> O) -> Seeded<O> {
        Seeded {
            value: f(&self.value),
            seed: self.seed,
        }
    }
}

/// A [`NoiseOp`] that takes only the seed from a [`Seeded`] value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeedOf;

macro_rules! impl_seedable {
    ($dt:path, $white:path, $uint:ident) => {
        impl SeedableNoiseType for $dt {
            fn generate_seed(&self, seed: u32) -> u32 {
                $white(seed as $uint).get(*self) as u32
            }
        }
    };
}

impl_seedable!(u8, White8, u8);
impl_seedable!(u16, White16, u16);
impl_seedable!(u32, White32, u32);
impl_seedable!(u64, White64, u64);
impl_seedable!(u128, White128, u128);
impl_seedable!(U8Vec2, White8, u8);
impl_seedable!(U8Vec3, White8, u8);
impl_seedable!(U8Vec4, White8, u8);
impl_seedable!(U16Vec2, White16, u16);
impl_seedable!(U16Vec3, White16, u16);
impl_seedable!(U16Vec4, White16, u16);
impl_seedable!(UVec2, White32, u32);
impl_seedable!(UVec3, White32, u32);
impl_seedable!(UVec4, White32, u32);
impl_seedable!(U64Vec2, White64, u64);
impl_seedable!(U64Vec3, White64, u64);
impl_seedable!(U64Vec4, White64, u64);
