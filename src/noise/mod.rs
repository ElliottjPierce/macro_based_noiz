//! This module contains all the noise itself

use std::marker::PhantomData;

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

pub mod grid;
pub mod mapping;
pub mod scalar;
pub mod white;

/// This trait encapsulates what noise is. It takes in an input and outputs the nosie result.
pub trait NoiseOp<I> {
    /// represents the output of a noise function
    type Output: NoiseResult;

    /// Samples the noise at the specific input. This is generally inlined.
    fn get(&self, input: I) -> Self::Output;

    /// The same as [sample](Self::get), but not inlined.
    fn get_cold(&self, input: I) -> Self::Output {
        self.get(input)
    }
}

/// Signifies that these types are effectively the same as far as noise is concerned.
pub trait NoiseConvert<T: NoiseResult>: NoiseResult {
    /// maps this value to a noise. Note that you should usually prefer [`NoiseResult::adapt`]
    fn convert(self) -> T;
}

/// marks this type as the potential result of some noise function.
pub trait NoiseResult {
    /// converts this value into a different type with a common noise goal.
    fn adapt<T: NoiseResult>(self) -> T
    where
        Self: NoiseConvert<T> + Sized,
    {
        self.convert()
    }
}

// built in
impl NoiseResult for u8 {}
impl NoiseResult for u16 {}
impl NoiseResult for u32 {}
impl NoiseResult for u64 {}
impl NoiseResult for u128 {}
impl NoiseResult for usize {}
impl NoiseResult for i8 {}
impl NoiseResult for i16 {}
impl NoiseResult for i32 {}
impl NoiseResult for i64 {}
impl NoiseResult for i128 {}
impl NoiseResult for isize {}
// bevy
impl NoiseResult for I8Vec2 {}
impl NoiseResult for I8Vec3 {}
impl NoiseResult for I8Vec4 {}
impl NoiseResult for I16Vec2 {}
impl NoiseResult for I16Vec3 {}
impl NoiseResult for I16Vec4 {}
impl NoiseResult for I64Vec2 {}
impl NoiseResult for I64Vec3 {}
impl NoiseResult for I64Vec4 {}
impl NoiseResult for IVec2 {}
impl NoiseResult for IVec3 {}
impl NoiseResult for IVec4 {}
impl NoiseResult for U8Vec2 {}
impl NoiseResult for U8Vec3 {}
impl NoiseResult for U8Vec4 {}
impl NoiseResult for U16Vec2 {}
impl NoiseResult for U16Vec3 {}
impl NoiseResult for U16Vec4 {}
impl NoiseResult for U64Vec2 {}
impl NoiseResult for U64Vec3 {}
impl NoiseResult for U64Vec4 {}
impl NoiseResult for UVec2 {}
impl NoiseResult for UVec3 {}
impl NoiseResult for UVec4 {}

impl<T: NoiseResult> NoiseConvert<T> for T {
    #[inline]
    fn convert(self) -> T {
        self
    }
}

/// Allows chaining noise functions together
pub struct Chain<I, N1: NoiseOp<I>, N2: NoiseOp<N1::Output>>(N1, N2, PhantomData<I>);

/// A noise operation that converts one noise type to another
pub struct Adapter<I: NoiseResult, O: NoiseResult>(PhantomData<(I, O)>)
where
    I: NoiseConvert<O>;

/// allows a function to be used as a noise operation
pub struct Morph<I, O: NoiseResult, D>(fn(I, &D) -> O, D, PhantomData<(I, O)>);

impl<I, N1: NoiseOp<I>, N2: NoiseOp<N1::Output>> NoiseOp<I> for Chain<I, N1, N2> {
    type Output = N2::Output;

    #[inline(always)]
    fn get(&self, input: I) -> Self::Output {
        self.1.get(self.0.get(input))
    }
}

impl<I: NoiseResult, O: NoiseResult> NoiseOp<I> for Adapter<I, O>
where
    I: NoiseConvert<O>,
{
    type Output = O;

    #[inline]
    fn get(&self, input: I) -> Self::Output {
        input.convert()
    }
}

impl<I, O: NoiseResult, D> NoiseOp<I> for Morph<I, O, D> {
    type Output = O;

    #[inline]
    fn get(&self, input: I) -> Self::Output {
        self.0(input, &self.1)
    }
}

/// Allows a user to construct a new noise type by stringing together noise operations. This simply
/// converts to a type and is intended to be used within [`noise_fn`]
#[macro_export]
macro_rules! noise_type {
    // starts with noise
    (input=$input:path, noise $noise_type:path = $_c:expr, $($next:tt)*) => {
        $crate::noise_type!(input=$input, prev=$noise_type, $($next)*)
    };

    // starts with morph
    (input=$input:path, morph |$_morph_i:ident| { $($data_n:ident: $data_t:path = $data_b:expr),* $(,)? } -> $out:path $_func:block, $($next:tt)*) => {
        $crate::noise_type!(input=$input, prev=$crate::noise::Morph<$input, $out, ($($data_t),*)>, $($next)*)
    };

    // starts with adapting
    (input=$input:path, into $converted:path, $($next:tt)*) => {
        $crate::noise_type!(input=$input, prev=$crate::noise::Adapter<$input, $converted>, $($next)*)
    };

    // chains another noise
    (input=$input:path, prev=$prev_t:path, noise $noise_type:path = $_c:expr, $($next:tt)*) => {
        $crate::noise_type!(input=$input, prev=$crate::noise::Chain<$input, $prev_t, $noise_type>, $($next)*)
    };

    // chains another morph
    (input=$input:path, prev=$prev_t:path, morph |$_morph_i:ident| { $($data_n:ident: $data_t:path = $data_b:expr),* $(,)? } -> $out:path $_func:block, $($next:tt)*) => {
        $crate::noise_type!(
            input=$input, prev=$crate::noise::Chain<$input, $prev_t, $crate::noise::Morph<<$prev_t as $crate::noise::NoiseOp<$input>>::Output, $out, ($($data_t),*,)>>,
            $($next)*
        )
    };

    // chains another adaption
    (input=$input:path, prev=$prev_t:path, into $converted:path, $($next:tt)*) => {
        $crate::noise_type!(input=$input, prev=$crate::noise::Chain<$input, $prev_t, $crate::noise::Adapter<<$prev_t as $crate::noise::NoiseOp<$input>>::Output, $converted>>, $($next)*)
    };

    // finishes when there are no more tokens
    (input=$_input:path, prev=$prev_t:path,) => {
        $prev_t
    };
}

/// Allows a user to construct a new noise type by stringing together noise operations. This simply
/// converts to a constructor body and is intended to be used within [`noise_fn`]
#[macro_export]
macro_rules! noise_build {
    // starts with noise
    (input=$input:path, noise $noise_type:path = $creation:expr, $($next:tt)*) => {
        $crate::noise_build!(input=$input, prev=($noise_type, $creation), $($next)*)
    };

    // starts with morph
    (input=$input:path, morph |$morph_i:ident| { $($data_n:ident: $data_t:path = $data_b:expr),* $(,)? } -> $out:path $func:block, $($next:tt)*) => {
        $crate::noise_build!(
            input=$input,
            prev=(
                $crate::noise::Morph<$input, $out, ($($data_t),*)>,
                {
                    $crate::noise::Morph::<$input, $out, ($($data_t),*)>(
                        |input, data| {
                            let (($($data_n),*)) = data;
                            let $morph_i = input;
                            $func
                        },
                        ($($data_b),*),
                        std::marker::PhantomData
                    )
                }
            ),
            $($next)*
        )
    };

    // starts with adapting
    (input=$input:path, into $converted:path, $($next:tt)*) => {
        $crate::noise_build!(input=$input, prev=($crate::noise::Adapter<$input, $converted>, { $crate::noise::Adapter::<$input, $converted>(std::marker::PhantomData) }), $($next)*)
    };

    // chains another noise
    (input=$input:path, prev=($prev_t:path, $prev_c:expr), noise $noise_type:path = $creation:expr, $($next:tt)*) => {
        $crate::noise_build!(
            input=$input, prev=(
                $crate::noise::Chain<$input, $prev_t, $noise_type>,
                { $crate::noise::Chain::<$input, $prev_t, $noise_type>($prev_c, $creation, PhantomData) }
            ),
            $($next)*
        )
    };

    // chains another morph
    (input=$input:path, prev=($prev_t:path, $prev_c:expr), morph |$morph_i:ident| { $($data_n:ident: $data_t:path = $data_b:expr),* $(,)? } -> $out:path $func:block, $($next:tt)*) => {
        $crate::noise_build!(
            input=$input,
            prev=(
                $crate::noise::Chain<$input, $prev_t, $crate::noise::Morph<<$prev_t as $crate::noise::NoiseOp<$input>>::Output, $out, ($($data_t),*,)>>,
                {
                    $crate::noise::Chain::<$input, $prev_t, _>(
                        $prev_c,
                        $crate::noise::Morph::<<$prev_t as $crate::noise::NoiseOp<$input>>::Output, $out, ($($data_t),*,)>(
                            |#[allow(unused_variables)] input, data| {
                                let ($($data_n),*,) = data;
                                let $morph_i = input;
                                $func
                            },
                            ($($data_b),*,),
                            std::marker::PhantomData
                        ),
                        std::marker::PhantomData
                    )
                }
            ),
            $($next)*
        )
    };

    // chains another adaption
    (input=$input:path, prev=($prev_t:path, $prev_c:expr), into $converted:path, $($next:tt)*) => {
        $crate::noise_build!(
            input=$input,
            prev=(
                $crate::noise::Chain<$input, $prev_t, $crate::noise::Adapter<<$prev_t as $crate::noise::NoiseOp<$input>>::Output, $converted>>,
                {
                    $crate::noise::Chain::<$input, $prev_t, $crate::noise::Adapter<<$prev_t as $crate::noise::NoiseOp<$input>>::Output, $converted>>(
                        $prev_c, $crate::noise::Adapter(std::marker::PhantomData), std::marker::PhantomData
                    )
                }
            ),
            $($next)*
        )
    };

    // finish when there are no more tokens
    (input=$_input:path, prev=($_prev_t:path, $prev_c:block),) => {
        $prev_c
    };
}

/// Allows a user to construct a new noise type by stringing together noise operations.
#[macro_export]
macro_rules! noise_fn {
    ($(#[$m:meta])* $v:vis struct $name:ident for $input:path = ($($(#[$pm:meta])* $n:ident: $t:path),*) { $($body:tt)* }) => {
        $(#[$m])*
        $v struct $name($crate::noise_type!(input=$input, $($body)*));

        impl $name {
            /// constructs a new instance of this noise
            pub fn new($($(#[$pm])* $n: $t),*) -> Self {
                Self($crate::noise_build!(input=$input, $($body)*))
            }
        }

        impl $crate::noise::NoiseOp<$input> for $name {
            type Output = <$crate::noise_type!(input=$input, $($body)*) as $crate::noise::NoiseOp<$input>>::Output;

            #[inline]
            fn get(&self, input: $input) -> Self::Output {
                self.0.get(input)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use white::White32;

    use super::*;

    noise_fn! {
        /// docs
        pub struct Test for i32 = (x: u32, y: u32, z: u32) {
            into u32,
            noise White32 = {
                White32(x)
            },
            into u32,
            morph |input| {
                offset: u32 = z,
            } -> u32 {
                let x = *offset;
                input + x
            },
            noise White32 = White32(y),
        }
    }

    #[test]
    fn test_noise_fn() {
        let noise = Test::new(57, 13, 45);
        let _test_res = noise.get(40);
    }

    #[test]
    fn test_noise_build() {
        let outer = 34u32;
        let noise = noise_build! {
            input = i32,
            noise Test = {
                Test::new(4, 12, 12)
            },
            noise White32 = {
                White32(outer)
            },
        };
        let _test_res = noise.get(40);
    }
}
