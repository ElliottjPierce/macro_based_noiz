//! This module allows worly noise to be created

use std::marker::PhantomData;

use bevy_math::{
    Vec2,
    Vec3,
    Vec4,
};

use super::{
    NoiseOp,
    NoiseType,
    grid::{
        GridPoint2,
        GridPoint3,
        GridPoint4,
    },
    norm::UNorm,
    nudges::Nudge,
};

/// Offsets grid values for distance-based noise
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Worly(pub Nudge);

/// Stores a result of a [`Worly`] noise
pub struct WorlyResult<T> {
    /// The original [`Worly`] noise.
    pub source: Worly,
    /// The points around which this sample is roughly centered.
    pub points: T,
}

/// Tracks the length of a type of vector `T`.
pub trait DistanceType<T> {
    /// precomputes the inverse maximum distance of a vector with this maximum component
    fn inverse_max(&self, max_component: f32) -> f32;
    /// computes the length of the given vector
    fn distance(&self, vec: T) -> f32;
}

/// A [`DistanceType`] for "as the crow flyies"
pub struct Euclidean;
/// A [`DistanceType`] for "manhatan" or diagonal distance
pub struct Manhatan;

/// Represents taking the nearest result of a [`WorlyResult`]
pub struct Nearest<T, D: DistanceType<T>> {
    distance: D,
    cached_inv_max_distance: f32,
    marker: PhantomData<T>,
}

impl<T, D: DistanceType<T>> Nearest<T, D> {
    /// constructs a new [`Nearest`] based on the passed `D` and `max_component` for
    /// [`DistanceType::inverse_max`]
    pub fn new(distance: D, max_component: f32) -> Self {
        Self {
            cached_inv_max_distance: distance.inverse_max(max_component),
            distance,
            marker: PhantomData,
        }
    }
}

impl Worly {
    /// creates a new [`Worly`].
    pub fn new(seed: u32, shift: f32) -> Self {
        Self::from_nudge(Nudge::new(seed, shift))
    }

    /// constructs a new [`Worly`] that shifts maximially.
    pub fn full(seed: u32) -> Self {
        Self::new(seed, 1.0)
    }

    /// constructs a new [`World`] based on its nudge.
    #[inline]
    pub fn from_nudge(nudge: Nudge) -> Self {
        Self(nudge)
    }
}

/// easily implements nudging for different types
macro_rules! impl_nudge {
    ($vec:path, $point:path, $d:literal, $u2f:ident) => {
        impl NoiseOp<$point> for Worly {
            type Output = WorlyResult<[$point; $d]>;

            #[inline]
            fn get(&self, input: $point) -> Self::Output {
                let mut points = input.corners();
                for c in &mut points {
                    let grid_shift = self.0.get(c.base);
                    let relative_shift = -((c.base % 2).$u2f()) * grid_shift; // we have to flip the offset every other cell.
                    c.offset += relative_shift;
                }
                WorlyResult {
                    source: *self,
                    points,
                }
            }
        }

        impl NoiseType for WorlyResult<[$point; $d]> {}

        impl DistanceType<$vec> for Euclidean {
            fn inverse_max(&self, max_component: f32) -> f32 {
                1.0 / self.distance(<$vec>::splat(max_component))
            }

            fn distance(&self, vec: $vec) -> f32 {
                vec.length()
            }
        }

        impl DistanceType<$vec> for Manhatan {
            fn inverse_max(&self, max_component: f32) -> f32 {
                1.0 / self.distance(<$vec>::splat(max_component))
            }

            fn distance(&self, vec: $vec) -> f32 {
                vec.length_squared()
            }
        }

        impl<D: DistanceType<$vec>> NoiseOp<WorlyResult<[$point; $d]>> for Nearest<$vec, D> {
            type Output = UNorm;

            #[inline]
            fn get(&self, input: WorlyResult<[$point; $d]>) -> Self::Output {
                let mut min = f32::INFINITY;
                for p in input.points {
                    min = min.min(self.distance.distance(p.offset));
                }
                UNorm::new_clamped(min * self.cached_inv_max_distance)
            }
        }
    };
}

impl_nudge!(Vec2, GridPoint2, 4, as_vec2);
impl_nudge!(Vec3, GridPoint3, 8, as_vec3);
impl_nudge!(Vec4, GridPoint4, 16, as_vec4);
