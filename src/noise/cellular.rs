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
    seeded::Seeded,
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
    /// constructs a new [`Cellular`] based on its [`Nudge`].
    #[inline]
    pub fn new(nudge: Nudge) -> Self {
        Self(nudge)
    }
}

impl<T> CellularResult<T> {
    /// Maps this value to another, keeping its source.
    #[inline]
    pub fn map<O: NoiseType>(self, f: impl FnOnce(T) -> O) -> CellularResult<O> {
        CellularResult {
            points: f(self.points),
            source: self.source,
        }
    }

    /// Maps this value to another, keeping its source.
    #[inline]
    pub fn map_ref<O: NoiseType>(&self, f: impl FnOnce(&T) -> O) -> CellularResult<O> {
        CellularResult {
            points: f(&self.points),
            source: self.source,
        }
    }
}

impl<T: NoiseType> NoiseType for CellularResult<T> {}

impl<T: NoiseType, const K: usize> Mergeable for CellularResult<[T; K]> {
    type Meta = Cellular;
    type Part = T;

    #[inline]
    fn perform_merge<M: Merger<Self::Part, Self::Meta>>(self, merger: &M) -> M::Output {
        merger.merge(self.points, &self.source)
    }
}

/// easily implements nudging for different types
macro_rules! impl_nudge {
    ($vec:path, $point:path, $d:literal, $u2f:ident) => {
        impl NoiseOp<[Seeded<$point>; $d]> for Cellular {
            type Output = CellularResult<[Seeded<$point>; $d]>;

            #[inline]
            fn get(&self, mut input: [Seeded<$point>; $d]) -> Self::Output {
                for c in &mut input {
                    let grid_shift = self.0.get(c.map_ref(|c| c.base)).value;
                    let relative_shift = -((c.value.base % 2).$u2f()) * grid_shift; // we have to flip the offset every other cell.
                    c.value.offset += relative_shift;
                }
                CellularResult {
                    source: *self,
                    points: input,
                }
            }
        }
    };
}

impl_nudge!(Vec2, GridPoint2, 4, as_vec2);
impl_nudge!(Vec3, GridPoint3, 8, as_vec3);
impl_nudge!(Vec4, GridPoint4, 16, as_vec4);
