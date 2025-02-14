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
    smoothing::LerpLocatable,
};

/// Offsets grid values for distance-based noise
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cellular(pub Nudge);

/// Stores a result of a [`Cellular`] noise
pub type CellularResult<T> = Associated<T, Cellular>;

impl Cellular {
    /// constructs a new [`Cellular`] based on its [`Nudge`].
    #[inline]
    pub fn new(nudge: Nudge) -> Self {
        Self(nudge)
    }
}

impl<T: NoiseType, const K: usize> Mergeable for CellularResult<[T; K]> {
    type Meta = Cellular;
    type Part = T;

    #[inline]
    fn perform_merge<M: Merger<Self::Part, Self::Meta>>(self, merger: &M) -> M::Output {
        merger.merge(self.value, &self.meta)
    }
}

/// easily implements nudging for different types
macro_rules! impl_nudge {
    ($point:path, $d:literal, $u2f:ident) => {
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

impl LerpLocatable for CellularResult<[Seeded<GridPoint2>; 4]> {
    type Location = [f32; 2];

    type Extents = [Seeded<GridPoint2>; 4];

    #[inline]
    fn prepare_lerp(self) -> Associated<Self::Extents, Self::Location> {
        let nx = (self.value[1].value.offset + self.value[0].value.offset).length();
        let px = (self.value[3].value.offset + self.value[2].value.offset).length();
        let ny = (self.value[2].value.offset + self.value[0].value.offset).length();
        let py = (self.value[3].value.offset + self.value[1].value.offset).length();

        let loc = [nx / (nx + px), ny / (ny + py)];

        Associated {
            value: self.value,
            meta: loc,
        }
    }
}
