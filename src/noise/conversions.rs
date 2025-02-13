//! Allows easily converting between noise types

use std::marker::PhantomData;

use super::{
    NoiseOp,
    NoiseType,
};

/// A trait to perform conversions
pub trait ConversionChain<O: NoiseType> {
    /// The input type
    type Input: NoiseType;
    /// performs static conversion between noise types
    fn convert(source: Self::Input) -> O;
}

/// A noise operation that converts one noise type to another
#[derive(Default, Clone, PartialEq)]
pub struct Adapter<C: ConversionChain<O>, O: NoiseType>(PhantomData<(C, O)>);

impl<C: ConversionChain<O>, O: NoiseType> Adapter<C, O> {
    /// Constructs a new [`Adapter`]
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<C: ConversionChain<O>, O: NoiseType> NoiseOp<C::Input> for Adapter<C, O> {
    type Output = O;

    #[inline]
    fn get(&self, input: C::Input) -> Self::Output {
        C::convert(input)
    }
}

impl<T: NoiseType> ConversionChain<T> for T {
    type Input = T;

    fn convert(source: Self::Input) -> T {
        source
    }
}

impl<I: ConversionChain<O, Input = I> + NoiseType, O: NoiseType> ConversionChain<O> for (I, O) {
    type Input = I;

    fn convert(source: Self::Input) -> O {
        I::convert(source)
    }
}

impl<I: ConversionChain<CF::Input, Input = I> + NoiseType, CF: ConversionChain<O>, O: NoiseType>
    ConversionChain<O> for (I, CF, O)
{
    type Input = I;

    fn convert(source: Self::Input) -> O {
        let source = I::convert(source);
        CF::convert(source)
    }
}

impl<
    I: ConversionChain<C9::Input, Input = I> + NoiseType,
    C9: ConversionChain<CF::Input>,
    CF: ConversionChain<O>,
    O: NoiseType,
> ConversionChain<O> for (I, C9, CF, O)
{
    type Input = I;

    fn convert(source: Self::Input) -> O {
        let source = I::convert(source);
        let source = C9::convert(source);
        CF::convert(source)
    }
}

impl<
    I: ConversionChain<C8::Input, Input = I> + NoiseType,
    C8: ConversionChain<C9::Input>,
    C9: ConversionChain<CF::Input>,
    CF: ConversionChain<O>,
    O: NoiseType,
> ConversionChain<O> for (I, C8, C9, CF, O)
{
    type Input = I;

    fn convert(source: Self::Input) -> O {
        let source = I::convert(source);
        let source = C8::convert(source);
        let source = C9::convert(source);
        CF::convert(source)
    }
}

impl<
    I: ConversionChain<C7::Input, Input = I> + NoiseType,
    C7: ConversionChain<C8::Input>,
    C8: ConversionChain<C9::Input>,
    C9: ConversionChain<CF::Input>,
    CF: ConversionChain<O>,
    O: NoiseType,
> ConversionChain<O> for (I, C7, C8, C9, CF, O)
{
    type Input = I;

    fn convert(source: Self::Input) -> O {
        let source = I::convert(source);
        let source = C7::convert(source);
        let source = C8::convert(source);
        let source = C9::convert(source);
        CF::convert(source)
    }
}

impl<
    I: ConversionChain<C6::Input, Input = I> + NoiseType,
    C6: ConversionChain<C7::Input>,
    C7: ConversionChain<C8::Input>,
    C8: ConversionChain<C9::Input>,
    C9: ConversionChain<CF::Input>,
    CF: ConversionChain<O>,
    O: NoiseType,
> ConversionChain<O> for (I, C6, C7, C8, C9, CF, O)
{
    type Input = I;

    fn convert(source: Self::Input) -> O {
        let source = I::convert(source);
        let source = C6::convert(source);
        let source = C7::convert(source);
        let source = C8::convert(source);
        let source = C9::convert(source);
        CF::convert(source)
    }
}

impl<
    I: ConversionChain<C5::Input, Input = I> + NoiseType,
    C5: ConversionChain<C6::Input>,
    C6: ConversionChain<C7::Input>,
    C7: ConversionChain<C8::Input>,
    C8: ConversionChain<C9::Input>,
    C9: ConversionChain<CF::Input>,
    CF: ConversionChain<O>,
    O: NoiseType,
> ConversionChain<O> for (I, C5, C6, C7, C8, C9, CF, O)
{
    type Input = I;

    fn convert(source: Self::Input) -> O {
        let source = I::convert(source);
        let source = C5::convert(source);
        let source = C6::convert(source);
        let source = C7::convert(source);
        let source = C8::convert(source);
        let source = C9::convert(source);
        CF::convert(source)
    }
}

/// Easily implement [`ConversionChain`] for a type
#[macro_export]
macro_rules! convertible {
    ($type:path = $out:path, | mut $name:ident | $converter:expr) => {
        $crate::convertible!($type = $out, |$name| {
            let mut $name = $name;
            $converter
        });
    };

    ($type:path = $out:path, | $name:ident | $converter:expr) => {
        impl $crate::noise::conversions::ConversionChain<$out> for $type {
            type Input = $type;

            #[inline]
            fn convert(source: Self::Input) -> $out {
                let $name = source;
                $converter
            }
        }
    };
}

#[cfg(test)]
mod test {
    use crate::noise::NoiseType;

    struct Foo1;
    struct Foo2;

    impl NoiseType for Foo1 {}
    impl NoiseType for Foo2 {}

    convertible!(Foo1 = Foo2, |mut _tmp| Foo2);
    convertible!(Foo2 = Foo1, |_tmp| Foo1);
}
