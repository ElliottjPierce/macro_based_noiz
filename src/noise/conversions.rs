//! Allows easily converting between noise types

use std::marker::PhantomData;

use super::{
    NoiseOp,
    NoiseType,
    convert,
};
pub use crate::__convertible as convertible;

/// A trait to perform conversions
pub trait NoiseConverter<O: NoiseType> {
    /// The input type
    type Input: NoiseType;
    /// performs static conversion between noise types
    fn convert(source: Self::Input) -> O;
}

/// A noise operation that converts one noise type to another
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Adapter<C: NoiseConverter<O>, O: NoiseType>(PhantomData<(C, O)>);

impl<C: NoiseConverter<O>, O: NoiseType> Adapter<C, O> {
    /// Constructs a new [`Adapter`]
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<C: NoiseConverter<O>, O: NoiseType> NoiseOp<C::Input> for Adapter<C, O> {
    type Output = O;

    #[inline]
    fn get(&self, input: C::Input) -> Self::Output {
        C::convert(input)
    }
}

impl<T: NoiseType> NoiseConverter<T> for T {
    type Input = T;

    #[inline]
    fn convert(source: Self::Input) -> T {
        source
    }
}

impl<I: NoiseConverter<O, Input = I> + NoiseType, O: NoiseType> NoiseConverter<O> for (I, O) {
    type Input = I;

    #[inline]
    fn convert(source: Self::Input) -> O {
        I::convert(source)
    }
}

impl<I: NoiseConverter<CF::Input, Input = I> + NoiseType, CF: NoiseConverter<O>, O: NoiseType>
    NoiseConverter<O> for (I, CF, O)
{
    type Input = I;

    #[inline]
    fn convert(source: Self::Input) -> O {
        let source = I::convert(source);
        CF::convert(source)
    }
}

impl<
    I: NoiseConverter<C9::Input, Input = I> + NoiseType,
    C9: NoiseConverter<CF::Input>,
    CF: NoiseConverter<O>,
    O: NoiseType,
> NoiseConverter<O> for (I, C9, CF, O)
{
    type Input = I;

    #[inline]
    fn convert(source: Self::Input) -> O {
        convert!(source => I, C9, CF, O)
    }
}

impl<
    I: NoiseConverter<C8::Input, Input = I> + NoiseType,
    C8: NoiseConverter<C9::Input>,
    C9: NoiseConverter<CF::Input>,
    CF: NoiseConverter<O>,
    O: NoiseType,
> NoiseConverter<O> for (I, C8, C9, CF, O)
{
    type Input = I;

    #[inline]
    fn convert(source: Self::Input) -> O {
        convert!(source => I, C8, C9, CF, O)
    }
}

impl<
    I: NoiseConverter<C7::Input, Input = I> + NoiseType,
    C7: NoiseConverter<C8::Input>,
    C8: NoiseConverter<C9::Input>,
    C9: NoiseConverter<CF::Input>,
    CF: NoiseConverter<O>,
    O: NoiseType,
> NoiseConverter<O> for (I, C7, C8, C9, CF, O)
{
    type Input = I;

    #[inline]
    fn convert(source: Self::Input) -> O {
        convert!(source => I, C7, C8, C9, CF, O)
    }
}

impl<
    I: NoiseConverter<C6::Input, Input = I> + NoiseType,
    C6: NoiseConverter<C7::Input>,
    C7: NoiseConverter<C8::Input>,
    C8: NoiseConverter<C9::Input>,
    C9: NoiseConverter<CF::Input>,
    CF: NoiseConverter<O>,
    O: NoiseType,
> NoiseConverter<O> for (I, C6, C7, C8, C9, CF, O)
{
    type Input = I;

    #[inline]
    fn convert(source: Self::Input) -> O {
        convert!(source => I, C6, C7, C8, C9, CF, O)
    }
}

impl<
    I: NoiseConverter<C5::Input, Input = I> + NoiseType,
    C5: NoiseConverter<C6::Input>,
    C6: NoiseConverter<C7::Input>,
    C7: NoiseConverter<C8::Input>,
    C8: NoiseConverter<C9::Input>,
    C9: NoiseConverter<CF::Input>,
    CF: NoiseConverter<O>,
    O: NoiseType,
> NoiseConverter<O> for (I, C5, C6, C7, C8, C9, CF, O)
{
    type Input = I;

    #[inline]
    fn convert(source: Self::Input) -> O {
        convert!(source => I, C5, C6, C7, C8, C9, CF, O)
    }
}

/// Easily implement [`NoiseConverter`] for a type
#[doc(hidden)]
#[macro_export]
macro_rules! __convertible {
    ($type:path = $out:path, | mut $name:ident | $converter:expr) => {
        $crate::noise::conversions::convertible!($type = $out, |$name| {
            let mut $name = $name;
            $converter
        });
    };

    ($type:path = $out:path, | $name:ident | $converter:expr) => {
        impl $crate::noise::conversions::NoiseConverter<$out> for $type {
            type Input = $type;

            #[inline]
            fn convert(source: Self::Input) -> $out {
                let $name = source;
                $converter
            }
        }
    };
}

/// Easily convert one [`NoiseType`] to another
#[doc(hidden)]
#[macro_export]
macro_rules! __convert {
    ($val:expr => $t:ty $(,)?) => {
        $crate::noise::NoiseType::adapt::<$t>($val)
    };

    ($val:expr => $($next:ty),+) => {
        $crate::noise::convert!($crate::noise::NoiseType::adapt::< $crate::noise::convert!(type $($next),+) >($val) =>| $($next),+ )
    };

    ($val:expr =>| $t:ty, $f:ty $(,)?) => {
        $crate::noise::conversions::noise_convert::<$t, $f, _>($crate::noise::convert!($val => <$t as $crate::noise::conversions::NoiseConverter<$f>>::Input ))
    };

    ($val:expr =>| $c:ty, $n:ty, $($next:ty),+) => {
        $crate::noise::convert!($crate::noise::conversions::noise_convert::<$c, $crate::noise::convert!(type $n, $($next),+), _>($val) => $n, $($next),*)
    };

    (type $n:ty $(,)?) => {
        $n
    };

    (type $n:ty, $f:ty $(,)?) => {
        <$n as $crate::noise::conversions::NoiseConverter<$f>>::Input
    };

    (type $n:ty, $n1:ty, $($next:ty),+) => {
        <$n as $crate::noise::conversions::NoiseConverter< $crate::noise::convert!(type $n1, $($next),+) >>::Input
    };
}

/// Uses `T` to convert a value of `I` to a value of `O`.
pub fn noise_convert<T: NoiseConverter<O, Input = I>, O: NoiseType, I>(val: I) -> O {
    T::convert(val)
}

#[cfg(test)]
mod test {
    use crate::noise::{
        NoiseType,
        conversions::convertible,
        convert,
    };

    struct Foo1;
    struct Foo2;
    struct Foo3;
    struct Foo4;
    struct Foo5;

    impl NoiseType for Foo1 {}
    impl NoiseType for Foo2 {}
    impl NoiseType for Foo3 {}
    impl NoiseType for Foo4 {}
    impl NoiseType for Foo5 {}

    convertible!(Foo1 = Foo2, |mut _tmp| Foo2);
    convertible!(Foo2 = Foo3, |_tmp| Foo3);
    convertible!(Foo3 = Foo4, |_tmp| Foo4);
    convertible!(Foo4 = Foo5, |_tmp| Foo5);
    convertible!(Foo5 = Foo1, |_tmp| Foo1);

    #[test]
    fn macro_tests() {
        let _x = convert!(Foo1 => Foo1, Foo2, Foo3, Foo3, Foo4, Foo5, Foo1, Foo1, Foo2);
    }
}
