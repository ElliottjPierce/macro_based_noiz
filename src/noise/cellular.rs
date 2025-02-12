//! This module allows Cellular noise to be created

use super::{
    NoiseOp,
    NoiseType,
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
};

/// Offsets grid values for distance-based noise
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cellular(pub Nudge);

/// Stores a result of a [`Cellular`] noise
#[derive(Debug, Clone, PartialEq)]
pub struct CellularResult<T> {
    /// The original [`Cellular`] noise.
    pub source: Cellular,
    /// The points around which this sample is roughly centered.
    pub points: T,
}

impl Cellular {
    /// creates a new [`Cellular`].
    pub fn new(seed: u32, shift: f32) -> Self {
        Self::from_nudge(Nudge::new(seed, shift))
    }

    /// constructs a new [`Cellular`] that shifts maximially.
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
        impl NoiseOp<$point> for Cellular {
            type Output = CellularResult<[$point; $d]>;

            #[inline]
            fn get(&self, input: $point) -> Self::Output {
                let mut points = input.corners();
                for c in &mut points {
                    let grid_shift = self.0.get(c.base);
                    let relative_shift = -((c.base % 2).$u2f()) * grid_shift; // we have to flip the offset every other cell.
                    c.offset += relative_shift;
                }
                CellularResult {
                    source: *self,
                    points,
                }
            }
        }

        impl NoiseType for CellularResult<[$point; $d]> {}

        impl Mergeable for CellularResult<[$point; $d]> {
            type Meta = Cellular;
            type Part = $point;

            #[inline]
            fn perform_merge<M: Merger<Self::Part, Self::Meta>>(self, merger: &M) -> M::Output {
                merger.merge(self.points, &self.source)
            }
        }
    };
}

impl_nudge!(Vec2, GridPoint2, 4, as_vec2);
impl_nudge!(Vec3, GridPoint3, 8, as_vec3);
impl_nudge!(Vec4, GridPoint4, 16, as_vec4);
