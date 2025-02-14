//! This module allows Cellular noise to be created

use bevy_math::{
    Mat2,
    Vec2,
    VectorSpace,
};

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

fn norm_length_of_a_along_opposite(a: Vec2, b: Vec2) -> f32 {
    let opposite = b - a;
    b.dot(opposite).max(0.0) * opposite.length_recip()

    // let opposite = a - b;
    // if a.dot(opposite) < 0.0 {
    //     return 0.0;
    // }

    // a.length() / (a.length() + b.length())

    // let opposite = (a + b).normalize();
    // a.dot(opposite) / (a.dot(opposite) + b.dot(opposite))
}

impl LerpLocatable for CellularResult<[Vec2; 4]> {
    type Location = [f32; 2];

    type Extents = [Vec2; 4];

    #[inline]
    fn prepare_lerp(self) -> Associated<Self::Extents, Self::Location> {
        // derived from  https://math.stackexchange.com/questions/169176/2d-transformation-matrix-to-make-a-trapezoid-out-of-a-rectangle/863702#863702

        // the quadralateral
        let p = self.value[0];
        let i = self.value[0] - self.value[2];
        let j = self.value[0] - self.value[1];
        let corner_if_parallel = i + j;
        let quad_to_prallel = corner_if_parallel - p + self.value[3]; // shifts the corner of the quadralateral to make it a parallelagram
        let corner = p - self.value[3];

        // parallelagram
        let square_to_parallelagram = Mat2::from_cols(i, j);
        let parallelagram_to_square = square_to_parallelagram.inverse();
        let p_in_square = parallelagram_to_square * p;
        let furthest_outside_of_square = parallelagram_to_square * corner;

        // the unit square
        let re_bounder = Vec2::ONE - furthest_outside_of_square;
        let p_final = p_in_square / furthest_outside_of_square;
        // p_in_square + re_bounder * (p_in_square / furthest_outside_of_square).element_product();
        // let [x, y] = p_final.to_array();
        let [x, y] = p_in_square.clamp(Vec2::ZERO, Vec2::ONE).to_array();

        // let nx = norm_length_of_a_along_opposite(self.value[0], self.value[1]);
        // let px = norm_length_of_a_along_opposite(self.value[2], self.value[3]);
        // let ny = norm_length_of_a_along_opposite(self.value[2], self.value[0]);
        // let py = norm_length_of_a_along_opposite(self.value[3], self.value[1]);

        // let dx = px - nx;
        // let dy = py - ny;
        // let skew = 1.0 / (1.0 - dy * dx);
        // let x = (ny + dy * nx) * skew;
        // let y = (nx + dx * ny) * skew;

        Associated {
            value: self.value,
            meta: [x, y],
        }
    }
}

impl LerpLocatable for CellularResult<[Seeded<GridPoint2>; 4]> {
    type Location = [f32; 2];

    type Extents = [Seeded<GridPoint2>; 4];

    #[inline]
    fn prepare_lerp(self) -> Associated<Self::Extents, Self::Location> {
        let lerp_loc = self
            .map_ref(|points| points.clone().map(|point| point.value.offset))
            .prepare_lerp()
            .meta;
        self.map_meta(|_| lerp_loc)
    }
}
