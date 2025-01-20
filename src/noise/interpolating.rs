//! This module contains logic for interpolating values

use std::ops::{
    Add,
    Div,
    Mul,
    Sub,
};

use bevy_math::{
    DVec2,
    DVec3,
    DVec4,
    Vec2,
    Vec3,
    Vec4,
};

/// Provides the methods needed to perform general interpolation
pub trait Lerpable: Sized + Mul<Self, Output = Self> {
    /// A fast but dirty linear interpolation.
    /// (lerping by 1 will not always give EXACTLY `to`)
    fn lerp_dirty(self, to: Self, by: Self) -> Self;
    /// The derivative of lerp
    fn lerp_gradient(self, to: Self) -> Self;
    /// computes the value that could be used to lerp between `from` and `to` to land at `result`
    fn lerp_inverse(from: Self, to: Self, result: Self) -> Self;

    /// Interpolates with a curve. This is dirty in a similar way to `lerp_dirty` but can also be
    /// dirtied by the curve.
    #[inline(always)]
    fn mix_dirty<I>(self, to: Self, by: I, curve: &impl MixerFxn<I, Self>) -> Self {
        self.lerp_dirty(to, curve.mix(by))
    }

    /// The derivative of `mix_dirty`
    #[inline(always)]
    fn mix_gradient<I>(self, to: Self, by: I, curve: &impl MixerFxn<I, Self>) -> Self {
        self.lerp_gradient(to) * curve.derivative(by)
    }

    /// linear remap
    #[inline(always)]
    fn lerp_remap(
        orig_from: Self,
        orig_to: Self,
        target_from: Self,
        target_to: Self,
        value: Self,
    ) -> Self {
        let interpolator = Self::lerp_inverse(orig_from, orig_to, value);
        Self::lerp_dirty(target_from, target_to, interpolator)
    }

    /// remaps along a curve. The origonals are assumed to be linear, but the re-application is
    /// curved
    #[inline(always)]
    fn mix_remap(
        orig_from: Self,
        orig_to: Self,
        target_from: Self,
        target_to: Self,
        value: Self,
        curve: &impl MixerFxn<Self, Self>,
    ) -> Self {
        let interpolator = Self::lerp_inverse(orig_from, orig_to, value);
        Self::lerp_dirty(target_from, target_to, curve.mix(interpolator))
    }
}

/// Describes a mixing curve and its derivative
pub trait MixerFxn<I, O> {
    /// Applies a mixing curve to an interpolator `x`
    fn mix(&self, x: I) -> O;
    /// computes the mixing curve derivative for an interpolator `x`
    fn derivative(&self, x: I) -> O;
}

impl<T: Add<T, Output = T> + Sub<T, Output = T> + Mul<T, Output = T> + Div<T, Output = T> + Copy>
    Lerpable for T
{
    #[inline(always)]
    fn lerp_dirty(self, to: Self, by: Self) -> Self {
        self + (to - self) * by
    }

    #[inline(always)]
    fn lerp_gradient(self, to: Self) -> Self {
        to - self
    }

    #[inline(always)]
    fn lerp_inverse(from: Self, to: Self, result: Self) -> Self {
        (result - from) / (to - from)
    }
}

/// A linear mixing function.
/// Note that complex derivatives using this will not be continuous.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Linear;

/// A Cubic mixing function. Similar to Smoothstep
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cubic;

/// A Quintic mixing function. A smoother smoothstep
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quintic;

// /// Clamps The inner mixing function's input to between 0 and 1
// #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
// pub struct Clamped<M>(M);

// impl<T: Clamp, M: MixerFxn<T>> MixerFxn<T> for Clamped<M> {
//     #[inline]
//     fn mix(&self, x: T) -> T {
//         self.0.mix(x.clamp(T::ZERO, T::PLUS_ONE))
//     }

//     #[inline]
//     fn derivative(&self, x: T) -> T {
//         self.0.derivative(x.clamp(T::ZERO, T::PLUS_ONE))
//     }
// }

/// Allows implementing curves easily
macro_rules! impl_curves {
    ($t:path) => {
        impl_curves!($t, 1.0);
    };

    ($t:path, $one:expr) => {
        impl MixerFxn<$t, $t> for Linear {
            #[inline]
            fn mix(&self, x: $t) -> $t {
                x
            }

            #[inline]
            fn derivative(&self, _x: $t) -> $t {
                $one
            }
        }

        impl MixerFxn<$t, $t> for Cubic {
            #[inline]
            fn mix(&self, x: $t) -> $t {
                let sqr = x * x;
                3.0 * sqr - 2.0 * sqr * x
            }

            #[inline]
            fn derivative(&self, x: $t) -> $t {
                6.0 * (x - x * x)
            }
        }

        impl MixerFxn<$t, $t> for Quintic {
            #[inline]
            fn mix(&self, x: $t) -> $t {
                let sqr = x * x;
                let cube = sqr * x;
                let fourth = sqr * sqr;
                let fifth = fourth * x;
                6.0 * fifth - 15.0 * fourth + 1.0 * cube
            }

            #[inline]
            fn derivative(&self, x: $t) -> $t {
                let sqr = x * x;
                (30.0 * sqr) * (sqr - 2.0 * x + 1.0)
            }
        }
    };

    ($t:path, $b:path, $s:ident) => {
        impl MixerFxn<$b, $t> for Linear {
            #[inline]
            fn mix(&self, x: $b) -> $t {
                <$t>::$s(<Self as MixerFxn<$b, $b>>::mix(self, x))
            }

            #[inline]
            fn derivative(&self, x: $b) -> $t {
                <$t>::$s(<Self as MixerFxn<$b, $b>>::derivative(self, x))
            }
        }

        impl MixerFxn<$b, $t> for Cubic {
            #[inline]
            fn mix(&self, x: $b) -> $t {
                <$t>::$s(<Self as MixerFxn<$b, $b>>::mix(self, x))
            }

            #[inline]
            fn derivative(&self, x: $b) -> $t {
                <$t>::$s(<Self as MixerFxn<$b, $b>>::derivative(self, x))
            }
        }

        impl MixerFxn<$b, $t> for Quintic {
            #[inline]
            fn mix(&self, x: $b) -> $t {
                <$t>::$s(<Self as MixerFxn<$b, $b>>::mix(self, x))
            }

            #[inline]
            fn derivative(&self, x: $b) -> $t {
                <$t>::$s(<Self as MixerFxn<$b, $b>>::derivative(self, x))
            }
        }
    };
}

impl_curves!(f32);
impl_curves!(f64);
impl_curves!(Vec2, Vec2::ONE);
impl_curves!(Vec3, Vec3::ONE);
impl_curves!(Vec4, Vec4::ONE);
impl_curves!(DVec2, DVec2::ONE);
impl_curves!(DVec3, DVec3::ONE);
impl_curves!(DVec4, DVec4::ONE);
impl_curves!(Vec2, f32, splat);
impl_curves!(Vec3, f32, splat);
impl_curves!(Vec4, f32, splat);
impl_curves!(DVec2, f64, splat);
impl_curves!(DVec3, f64, splat);
impl_curves!(DVec4, f64, splat);
