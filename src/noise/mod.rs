//! This module contains all the noise itself

use bevy_math::{
    DVec2,
    DVec3,
    DVec4,
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
    Vec2,
    Vec3,
    Vec4,
};
use conversions::NoiseConverter;

pub mod associating;
pub mod conversions;
pub mod grid;
pub mod interpolating;
pub mod mapping;
pub mod merging;
pub mod norm;
pub mod nudges;
pub mod seeded;
pub mod smoothing;
pub mod voronoi;
pub mod white;

pub use macros::noise_op;

pub use crate::__convert as convert;

/// This trait encapsulates what noise is. It takes in an input and outputs the nosie result.
pub trait NoiseOp<I> {
    /// represents the output of a noise function
    type Output: NoiseType;

    /// Samples the noise at the specific input. This is generally inlined.
    fn get(&self, input: I) -> Self::Output;

    /// The same as [sample](Self::get), but not inlined.
    fn get_cold(&self, input: I) -> Self::Output {
        self.get(input)
    }
}

/// Marks the type as involved in noise functions as either an input, output or both.
pub trait NoiseType {
    /// converts this value into a different type with a common noise goal.
    /// This exists to prevent the user from having to qualify the trait and the using
    /// [`NoiseConvert::convert`]
    fn adapt<T: NoiseType>(self) -> T
    where
        Self: NoiseConverter<T, Input = Self> + Sized,
    {
        Self::convert(self)
    }
}

/// Signifies that this type is a noise endpoint.
pub trait Noise
where
    Self: NoiseOp<Self::Input>,
{
    /// the primary input type used for this noise
    type Input: NoiseType;

    /// samples the noise at this input
    #[inline]
    fn sample<C: NoiseConverter<Self::Input, Input = C>>(&self, input: C) -> Self::Output {
        self.get(C::convert(input))
    }

    /// samples the noise at this input
    fn sample_cold<C: NoiseConverter<Self::Input, Input = C>>(&self, input: C) -> Self::Output {
        self.sample::<C>(input)
    }
}

// built in
impl NoiseType for f32 {}
impl NoiseType for f64 {}
impl NoiseType for u8 {}
impl NoiseType for u16 {}
impl NoiseType for u32 {}
impl NoiseType for u64 {}
impl NoiseType for u128 {}
impl NoiseType for usize {}
impl NoiseType for i8 {}
impl NoiseType for i16 {}
impl NoiseType for i32 {}
impl NoiseType for i64 {}
impl NoiseType for i128 {}
impl NoiseType for isize {}
// bevy
impl NoiseType for Vec2 {}
impl NoiseType for DVec2 {}
impl NoiseType for Vec3 {}
impl NoiseType for DVec3 {}
impl NoiseType for Vec4 {}
impl NoiseType for DVec4 {}
impl NoiseType for I8Vec2 {}
impl NoiseType for I8Vec3 {}
impl NoiseType for I8Vec4 {}
impl NoiseType for I16Vec2 {}
impl NoiseType for I16Vec3 {}
impl NoiseType for I16Vec4 {}
impl NoiseType for I64Vec2 {}
impl NoiseType for I64Vec3 {}
impl NoiseType for I64Vec4 {}
impl NoiseType for IVec2 {}
impl NoiseType for IVec3 {}
impl NoiseType for IVec4 {}
impl NoiseType for U8Vec2 {}
impl NoiseType for U8Vec3 {}
impl NoiseType for U8Vec4 {}
impl NoiseType for U16Vec2 {}
impl NoiseType for U16Vec3 {}
impl NoiseType for U16Vec4 {}
impl NoiseType for U64Vec2 {}
impl NoiseType for U64Vec3 {}
impl NoiseType for U64Vec4 {}
impl NoiseType for UVec2 {}
impl NoiseType for UVec3 {}
impl NoiseType for UVec4 {}

impl<T: NoiseType, const N: usize> NoiseType for [T; N] {}

#[cfg(test)]
mod tests {

    use super::{
        associating::MetaOf,
        grid::{
            GridNoise,
            GridPoint2,
        },
        norm::UNorm,
        seeded::Seeding,
        *,
    };
    use crate as noiz;

    // this is taken from the docs for noise_op.
    noise_op! {
        /// Attributes work!
        pub struct MyNoise for Vec2 -> UNorm = // declare the name of the noise and what type it is for
        /// Attributes work!
        pub(crate) struct MyNoiseArgs {seed: u32, period: f32,} // declare the data that is used to make the noise operation
        impl // specifies the start of the noise implementation.
        // const let creates a local variable diring construction.
        const #[allow(unused)] let another_seed = seed + 1;
        /// Attributes work!
        #[allow(unused)]
        pub use custom_data: f32 = period; // `use` adds custom data to the noise struct. Visibility works too.
        pub fn fist_noise: GridNoise = GridNoise::new_period(period); // 'do' is the same as 'use', but the value participates as a noise operation.
        /// Attributes work!
        fn Seeding = Seeding(seed); // If you don't give a 'do' a name, it will make one for you.
        #[allow(unused)]
        let GridPoint2{ base, offset } = input.value; // 'let' holds a temporary value during the noise calculation.
        fn MetaOf; // If you don't provide a constructor for a 'do' value, the default will be used.
        as UNorm, f32, UNorm; // 'as' performs a conversion chain through the types listed.
        |mut x: UNorm| { // 'fn' performs a custom noise function. You must name the return type.
            // You can name the parameter and its type if you want.
            x = UNorm::new_clamped(*custom_data * offset.x); // You can use the values of 'use' 'do' 'let' operations here.
            [x, x, x] // You can't use return, but whatever value is left here is passed out as the result.
        }
        for as f32; // 'for' operates on inner values of an array for this operation only. The next operation will be on the resulting mapped array.
        || input[2]; // 'fn' operations don't need to specify their type, and if they don't specify a name, `input` is the default
        // whatever value is left here is returned for the noise operation.
        as UNorm
    }

    #[test]
    fn test_noise_fn() {
        let noise = MyNoise::from(MyNoiseArgs {
            seed: 12,
            period: 10.0,
        });
        let _test_res = noise.sample(Vec2::ONE);
    }
}
