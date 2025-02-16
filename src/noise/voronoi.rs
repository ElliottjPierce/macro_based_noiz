//! This module allows Cellular noise to be created

use super::{
    NoiseOp,
    NoiseType,
    associating::Associated,
    grid::{
        GridPoint2,
        GridPoint3,
        GridPoint4,
    },
    merging::{
        Mergeable,
        Merger,
    },
    nudges::Nudge,
    seeded::Seeded,
};

/// Offsets grid values for distance-based noise
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Voronoi(Nudge);

/// Stores a result of a [`Voronoi`] noise
pub type VoronoiGraph<T> = Associated<T, Voronoi>;

impl Voronoi {
    /// constructs a new [`Voronoi`] with the maximum allowed nudging.
    #[inline]
    pub fn full() -> Self {
        Self(Nudge::full_leashed())
    }

    /// constructs a new [`Voronoi`] with the a particular nudging range.
    /// The range will be forsed into 0..=1.
    #[inline]
    pub fn new(range: f32) -> Self {
        Self(Nudge::new_leashed(range))
    }

    /// Gets the [`Voronoi`]'s [`Nudge`].
    #[inline]
    pub fn get_nudge(&self) -> &Nudge {
        &self.0
    }
}

impl<T: NoiseType, const K: usize> Mergeable for VoronoiGraph<[T; K]> {
    type Meta = Voronoi;
    type Part = T;

    #[inline]
    fn perform_merge<M: Merger<Self::Part, Self::Meta>>(self, merger: &M) -> M::Output {
        merger.merge(self.value, &self.meta)
    }
}

/// easily implements nudging for different types
macro_rules! impl_nudge {
    ($point:path, $d_2:literal, $d_3:literal, $u2f:ident) => {
        impl<const N: usize> NoiseOp<[Seeded<$point>; N]> for Voronoi {
            type Output = VoronoiGraph<[Seeded<$point>; N]>;

            #[inline]
            fn get(&self, mut input: [Seeded<$point>; N]) -> Self::Output {
                for c in &mut input {
                    let grid_shift = self.0.get(c.map_ref(|c| c.base)).value;
                    c.value.offset -= grid_shift;
                }
                VoronoiGraph {
                    meta: *self,
                    value: input,
                }
            }
        }
    };
}

impl_nudge!(GridPoint2, 4, 9, as_vec2);
impl_nudge!(GridPoint3, 8, 27, as_vec3);
impl_nudge!(GridPoint4, 16, 81, as_vec4);
