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
pub trait Lerpable<T>: Sized {
    /// A fast but dirty linear interpolation.
    /// (lerping by 1 will not always give EXACTLY `to`)
    fn lerp_dirty(self, to: Self, by: T) -> Self;

    /// The derivative of lerp
    fn lerp_gradient(self, to: Self) -> Self;

    /// Interpolates with a curve. This is dirty in a similar way to `lerp_dirty` but can also be
    /// dirtied by the curve.
    #[inline(always)]
    fn mix_dirty<I>(self, to: Self, by: I, curve: &impl MixerFxn<I, T>) -> Self {
        self.lerp_dirty(to, curve.mix(by))
    }

    /// The derivative of `mix_dirty`
    fn mix_gradient<I>(self, to: Self, by: I, curve: &impl MixerFxn<I, T>) -> Self;
}

/// Provides methods needed to perform general inverse interpolation, including remapping
pub trait LerpableInverse: Lerpable<Self> {
    /// computes the value that could be used to lerp between `from` and `to` to land at `result`
    fn lerp_inverse(from: Self, to: Self, result: Self) -> Self;

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

impl<L, T: Add<T, Output = T> + Sub<T, Output = T> + Mul<L, Output = T> + Div<T, Output = T> + Copy>
    Lerpable<L> for T
{
    #[inline(always)]
    fn lerp_dirty(self, to: Self, by: L) -> Self {
        self + (to - self) * by
    }

    #[inline(always)]
    fn lerp_gradient(self, to: Self) -> Self {
        to - self
    }

    #[inline(always)]
    fn mix_gradient<I>(self, to: Self, by: I, curve: &impl MixerFxn<I, L>) -> Self {
        self.lerp_gradient(to) * curve.derivative(by)
    }
}

impl<T: Sub<T, Output = T> + Div<T, Output = T> + Copy + Lerpable<T>> LerpableInverse for T {
    #[inline(always)]
    fn lerp_inverse(from: Self, to: Self, result: Self) -> Self {
        (result - from) / (to - from)
    }
}

/// A linear mixing function.
/// Note that complex derivatives using this will not be continuous.
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Linear;

/// A Cubic mixing function. Similar to Smoothstep
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Cubic;

/// Allows implementing curves easily
macro_rules! impl_curves {
    ($t:ty) => {
        impl MixerFxn<$t, $t> for Linear {
            #[inline]
            fn mix(&self, x: $t) -> $t {
                x
            }

            #[inline]
            fn derivative(&self, _x: $t) -> $t {
                1.0
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
    };

    ($f:ty, $v:ty) => {
        impl<T: MixerFxn<$f, $f>> MixerFxn<$f, $v> for T {
            fn mix(&self, x: $f) -> $v {
                <$v>::splat(<Self as MixerFxn<$f, $f>>::mix(self, x))
            }

            fn derivative(&self, x: $f) -> $v {
                <$v>::splat(<Self as MixerFxn<$f, $f>>::derivative(self, x))
            }
        }

        impl<T: MixerFxn<$f, $f>> MixerFxn<$v, $v> for T {
            fn mix(&self, x: $v) -> $v {
                <$v>::from_array(
                    x.to_array()
                        .map(|x| <Self as MixerFxn<$f, $f>>::mix(self, x)),
                )
            }

            fn derivative(&self, x: $v) -> $v {
                <$v>::from_array(
                    x.to_array()
                        .map(|x| <Self as MixerFxn<$f, $f>>::derivative(self, x)),
                )
            }
        }
    };
}

impl_curves!(f32);
impl_curves!(f64);
impl_curves!(f32, Vec2);
impl_curves!(f32, Vec3);
impl_curves!(f32, Vec4);
impl_curves!(f64, DVec2);
impl_curves!(f64, DVec3);
impl_curves!(f64, DVec4);
