//! This module contains all the noise itself

use std::marker::PhantomData;

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

pub mod grid;
pub mod interpolating;
pub mod mapping;
pub mod merging;
pub mod norm;
pub mod smoothing;
pub mod white;

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

/// Signifies that these types are effectively the same as far as noise is concerned.
pub trait NoiseConvert<T: NoiseType>: NoiseType {
    /// maps this value to a noise. Note that you should usually prefer [`NoiseResult::adapt`]
    fn convert(self) -> T;
}

/// A trait to perform conversions
pub trait ConversionChain {
    /// The input type
    type Input: NoiseType;
    /// The output type
    type Output: NoiseType;
    /// performs static conversion between noise types
    fn convert(x: Self::Input) -> Self::Output;
}

/// Marks the type as involved in noise functions as either an input, output or both.
pub trait NoiseType {
    /// converts this value into a different type with a common noise goal.
    fn adapt<T: NoiseType>(self) -> T
    where
        Self: NoiseConvert<T> + Sized,
    {
        self.convert()
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
    fn sample<T: NoiseConvert<Self::Input>>(&self, input: T) -> Self::Output {
        self.get(input.convert())
    }

    /// samples the noise at this input
    fn sample_cold<T: NoiseConvert<Self::Input>>(&self, input: T) -> Self::Output {
        self.sample(input)
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

impl<T: NoiseType> NoiseConvert<T> for T {
    #[inline]
    fn convert(self) -> T {
        self
    }
}

/// Allows chaining noise functions together
#[derive(Default, Clone, PartialEq)]
pub struct Chain<I, N1: NoiseOp<I>, N2: NoiseOp<N1::Output>>(N1, N2, PhantomData<I>);

/// A noise operation that converts one noise type to another
#[derive(Default, Clone, PartialEq)]
pub struct Adapter<I: NoiseType, O: NoiseType>(PhantomData<(I, O)>)
where
    I: NoiseConvert<O>;

/// allows a function to be used as a noise operation
#[derive(Clone, PartialEq)]
pub struct Morph<I, O: NoiseType, D>(fn(I, &D) -> O, D, PhantomData<(I, O)>);

impl<I, N1: NoiseOp<I>, N2: NoiseOp<N1::Output>> Chain<I, N1, N2> {
    /// Constructs a new [`Chain`]
    pub fn new(fist: N1, second: N2) -> Self {
        Self(fist, second, PhantomData)
    }
}

impl<I: NoiseType, O: NoiseType> Adapter<I, O>
where
    I: NoiseConvert<O>,
{
    /// Constructs a new [`Adapter`]
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<I, O: NoiseType, D> Morph<I, O, D> {
    /// Constructs a new [`Morph`]
    pub fn new(function: fn(I, &D) -> O, data: D) -> Self {
        Self(function, data, PhantomData)
    }
}

impl<I, N1: NoiseOp<I>, N2: NoiseOp<N1::Output>> NoiseOp<I> for Chain<I, N1, N2> {
    type Output = N2::Output;

    #[inline(always)]
    fn get(&self, input: I) -> Self::Output {
        self.1.get(self.0.get(input))
    }
}

impl<I: NoiseType, O: NoiseType> NoiseOp<I> for Adapter<I, O>
where
    I: NoiseConvert<O>,
{
    type Output = O;

    #[inline]
    fn get(&self, input: I) -> Self::Output {
        input.convert()
    }
}

impl<I, O: NoiseType, D> NoiseOp<I> for Morph<I, O, D> {
    type Output = O;

    #[inline]
    fn get(&self, input: I) -> Self::Output {
        self.0(input, &self.1)
    }
}

impl<I: NoiseConvert<O>, O: NoiseType> ConversionChain for (I, O) {
    type Input = I;
    type Output = O;

    fn convert(x: Self::Input) -> Self::Output {
        x.convert()
    }
}

impl<I: NoiseConvert<T1>, T1: NoiseConvert<O>, O: NoiseType> ConversionChain for (I, T1, O) {
    type Input = I;
    type Output = O;

    fn convert(x: Self::Input) -> Self::Output {
        x.convert().convert()
    }
}

impl<I: NoiseConvert<T2>, T2: NoiseConvert<T1>, T1: NoiseConvert<O>, O: NoiseType> ConversionChain
    for (I, T2, T1, O)
{
    type Input = I;
    type Output = O;

    fn convert(x: Self::Input) -> Self::Output {
        x.convert().convert().convert()
    }
}

impl<
    I: NoiseConvert<T3>,
    T3: NoiseConvert<T2>,
    T2: NoiseConvert<T1>,
    T1: NoiseConvert<O>,
    O: NoiseType,
> ConversionChain for (I, T3, T2, T1, O)
{
    type Input = I;
    type Output = O;

    fn convert(x: Self::Input) -> Self::Output {
        x.convert().convert().convert().convert()
    }
}

impl<
    I: NoiseConvert<T4>,
    T4: NoiseConvert<T3>,
    T3: NoiseConvert<T2>,
    T2: NoiseConvert<T1>,
    T1: NoiseConvert<O>,
    O: NoiseType,
> ConversionChain for (I, T4, T3, T2, T1, O)
{
    type Input = I;
    type Output = O;

    fn convert(x: Self::Input) -> Self::Output {
        x.convert().convert().convert().convert().convert()
    }
}

impl<
    I: NoiseConvert<T5>,
    T5: NoiseConvert<T4>,
    T4: NoiseConvert<T3>,
    T3: NoiseConvert<T2>,
    T2: NoiseConvert<T1>,
    T1: NoiseConvert<O>,
    O: NoiseType,
> ConversionChain for (I, T5, T4, T3, T2, T1, O)
{
    type Input = I;
    type Output = O;

    fn convert(x: Self::Input) -> Self::Output {
        x.convert()
            .convert()
            .convert()
            .convert()
            .convert()
            .convert()
    }
}

impl<
    I: NoiseConvert<T6>,
    T6: NoiseConvert<T5>,
    T5: NoiseConvert<T4>,
    T4: NoiseConvert<T3>,
    T3: NoiseConvert<T2>,
    T2: NoiseConvert<T1>,
    T1: NoiseConvert<O>,
    O: NoiseType,
> ConversionChain for (I, T6, T5, T4, T3, T2, T1, O)
{
    type Input = I;
    type Output = O;

    fn convert(x: Self::Input) -> Self::Output {
        x.convert()
            .convert()
            .convert()
            .convert()
            .convert()
            .convert()
            .convert()
    }
}

/// Allows a user to construct a new noise type by stringing together noise operations. This simply
/// converts to a type and is intended to be used within [`noise_fn`]
#[macro_export]
macro_rules! noise_type {
    // starts with noise
    (input=$input:path, noise $noise_type:path = $_c:expr, $($next:tt)*) => {
        $crate::noise_type!(input=$input, prev=$noise_type, $($next)*,)
    };

    // starts with empty morph
    (input=$input:path, morph |$_morph_i:ident| -> $out:path $_func:block, $($next:tt)*) => {
        $crate::noise_type!(input=$input, prev=$crate::noise::Morph<$input, $out, ()>, $($next)*,)
    };

    // starts with morph
    (input=$input:path, morph |$_morph_i:ident| { $($data_n:ident: $data_t:path = $data_b:expr),* $(,)? } -> $out:path $_func:block, $($next:tt)*) => {
        $crate::noise_type!(input=$input, prev=$crate::noise::Morph<$input, $out, ($($data_t),*)>, $($next)*,)
    };

    // starts with adapting
    (input=$input:path, into $converted:path, $($next:tt)*) => {
        $crate::noise_type!(input=$input, prev=$crate::noise::Adapter<$input, $converted>, $($next)*,)
    };

    // chains another noise
    (input=$input:path, prev=$prev_t:path, noise $noise_type:path = $_c:expr, $($next:tt)*) => {
        $crate::noise_type!(input=$input, prev=$crate::noise::Chain<$input, $prev_t, $noise_type>, $($next)*)
    };

    // chains another empty morph
    (input=$input:path, prev=$prev_t:path, morph |$_morph_i:ident| -> $out:path $_func:block, $($next:tt)*) => {
        $crate::noise_type!(
            input=$input, prev=$crate::noise::Chain<$input, $prev_t, $crate::noise::Morph<<$prev_t as $crate::noise::NoiseOp<$input>>::Output, $out, ()>>,
            $($next)*
        )
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
    (input=$_input:path, prev=$prev_t:path  $(,)*) => {
        $prev_t
    };
}

/// Allows a user to construct a new noise type by stringing together noise operations. This simply
/// converts to a constructor body and is intended to be used within [`noise_fn`]
#[macro_export]
macro_rules! noise_build {
    // starts with noise
    (input=$input:path, noise $noise_type:path = $creation:expr, $($next:tt)*) => {
        $crate::noise_build!(input=$input, prev=($noise_type, $creation), $($next)*,)
    };

    // starts with empty morph
    (input=$input:path, morph |$morph_i:ident| -> $out:path $func:block, $($next:tt)*) => {
        $crate::noise_build!(
            input=$input,
            prev=(
                $crate::noise::Morph<$input, $out, ()>,
                {
                    $crate::noise::Morph::<$input, $out, ()>::new(
                        |input, _data| {
                            let $morph_i = input;
                            $func
                        },
                        ($($data_b),*),
                        std::marker::PhantomData
                    )
                }
            ),
            $($next)*,
        )
    };

    // starts with morph
    (input=$input:path, morph |$morph_i:ident| { $($data_n:ident: $data_t:path = $data_b:expr),* $(,)? } -> $out:path $func:block, $($next:tt)*) => {
        $crate::noise_build!(
            input=$input,
            prev=(
                $crate::noise::Morph<$input, $out, ($($data_t),*)>,
                {
                    $crate::noise::Morph::<$input, $out, ($($data_t),*)>::new(
                        |input, data| {
                            let (($($data_n),*)) = data;
                            let $morph_i = input;
                            $func
                        },
                        ($($data_b),*)
                    )
                }
            ),
            $($next)*,
        )
    };

    // starts with adapting
    (input=$input:path, into $converted:path, $($next:tt)*) => {
        $crate::noise_build!(input=$input, prev=($crate::noise::Adapter<$input, $converted>, { $crate::noise::Adapter::<$input, $converted>::new() }), $($next)*,)
    };

    // chains another noise
    (input=$input:path, prev=($prev_t:path, $prev_c:expr), noise $noise_type:path = $creation:expr, $($next:tt)*) => {
        $crate::noise_build!(
            input=$input, prev=(
                $crate::noise::Chain<$input, $prev_t, $noise_type>,
                { $crate::noise::Chain::<$input, $prev_t, $noise_type>::new($prev_c, $creation) }
            ),
            $($next)*
        )
    };

    // chains another empty morph
    (input=$input:path, prev=($prev_t:path, $prev_c:expr), morph |$morph_i:ident| -> $out:path $func:block, $($next:tt)*) => {
        $crate::noise_build!(
            input=$input,
            prev=(
                $crate::noise::Chain<$input, $prev_t, $crate::noise::Morph<<$prev_t as $crate::noise::NoiseOp<$input>>::Output, $out, ()>>,
                {
                    $crate::noise::Chain::<$input, $prev_t, _>::new(
                        $prev_c,
                        $crate::noise::Morph::<<$prev_t as $crate::noise::NoiseOp<$input>>::Output, $out, ()>::new(
                            |input, _data| {
                                let $morph_i = input;
                                $func
                            },
                            ()
                        )
                    )
                }
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
                    $crate::noise::Chain::<$input, $prev_t, _>::new(
                        $prev_c,
                        $crate::noise::Morph::<<$prev_t as $crate::noise::NoiseOp<$input>>::Output, $out, ($($data_t),*,)>::new(
                            |input, data| {
                                let ($($data_n),*,) = data;
                                let $morph_i = input;
                                $func
                            },
                            ($($data_b),*,),
                        )
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
                    $crate::noise::Chain::<$input, $prev_t, $crate::noise::Adapter<<$prev_t as $crate::noise::NoiseOp<$input>>::Output, $converted>>::new(
                        $prev_c, $crate::noise::Adapter::new()
                    )
                }
            ),
            $($next)*
        )
    };

    // finish when there are no more tokens
    (input=$_input:path, prev=($_prev_t:path, $prev_c:expr) $(,)*) => {
        $prev_c
    };
}

/// Allows a user to construct a new noise type by stringing together noise operations.
#[macro_export]
macro_rules! noise_fn {
    ($(#[$m:meta])* $v:vis struct $name:ident for $input:path = ($($params:tt)*) { $($body:tt)* }) => {
        $(#[$m])*
        $v struct $name($crate::noise_type!(input=$input, $($body)*));

        impl $name {
            /// constructs a new instance of this noise
            pub fn new($($params)*) -> Self {
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

        impl $crate::noise::Noise for $name {
            type Input = $input;
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
            morph |input| -> u32 {
                input + 2
            }
        }
    }

    #[test]
    fn test_noise_fn() {
        let noise = Test::new(57, 13, 45);
        let _test_res = noise.sample(40);
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
            }
        };
        let _test_res = noise.get(40);
    }
}
