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
pub struct Voronoi(pub Nudge<true>);

/// Stores a result of a [`Cellular`] noise
pub type VoronoiGraph<T> = Associated<T, Voronoi>;

impl Voronoi {
    /// constructs a new [`Cellular`] based on its [`Nudge`].
    #[inline]
    pub fn new(nudge: Nudge<true>) -> Self {
        Self(nudge)
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
    ($point:path, $d:literal, $u2f:ident) => {
        impl NoiseOp<[Seeded<$point>; $d]> for Voronoi {
            type Output = VoronoiGraph<[Seeded<$point>; $d]>;

            #[inline]
            fn get(&self, mut input: [Seeded<$point>; $d]) -> Self::Output {
                for c in &mut input {
                    let grid_shift = self.0.get(c.map_ref(|c| c.base)).value;
                    let relative_shift = -((c.value.base % 2).$u2f()) * grid_shift; // we have to flip the offset every other cell.
                    c.value.offset += relative_shift;
                }
                VoronoiGraph {
                    meta: *self,
                    value: input,
                }
            }
        }
    };
}

impl_nudge!(GridPoint2, 4, as_vec2);
impl_nudge!(GridPoint3, 8, as_vec3);
impl_nudge!(GridPoint4, 16, as_vec4);
