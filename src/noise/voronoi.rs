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
///
/// ## FAST_APPROX
///
/// If this is true, the noise will be approximated instead of being fully calculated.
///
/// ### When False (default)
///
/// Normal Voronoi noise samples all the surrounding cells around the cell where the point falls.
/// In 1d that's 3 tiles, 2d = 9 tiles, kd = 3^k tiles. This allows complete freedom with where each
/// point is shifted. However, it can leave some performance on the table.
///
/// ### When True
///
/// When true this restricts the cells checked to only the cell's corners, so kd = 2^k. Much nicer!
/// However, to prevent ugly seams from the blind spots this introduces, we have to be a bit more
/// aggressive with the noise. This can produce sharper, less appealing results, but it can do so
/// more quickly. Further, this limited [`VoronoiGraph`] has less functionality because of the blind
/// spots, so while it can work with most distance-based noise, it is limited in other aspects.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Voronoi<const FAST_APPROX: bool = false>(Nudge<false>);

/// Stores a result of a [`Voronoi`] noise
pub type VoronoiGraph<T, const FAST_APPROX: bool = false> = Associated<T, Voronoi<FAST_APPROX>>;

impl<const FAST_APPROX: bool> Voronoi<FAST_APPROX> {
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
        impl NoiseOp<[Seeded<$point>; $d]> for Voronoi<true> {
            type Output = VoronoiGraph<[Seeded<$point>; $d], true>;

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
