//! 2d orthogonal space utilities.

use bevy_math::{
    BVec2,
    IVec2,
};
use flagset::FlagSet;

use super::d1::AxisDirections;
use crate::{
    name_array,
    spatial::named_array::NamedArrayIndices,
};

name_array! {
    /// A 1 to 1 collection for the corners of a square
    pub struct Corners2d,
    /// the corners of a square
    pub enum Corner2d: u8, u8 {
        /// Left Down
        Ld,
        /// Left Up
        Lu,
        /// Right Down
        Rd,
        /// Right Up
        Ru,
    }

    /// A 1 to 1 collection for the sides of a square
    pub struct Sides2d,
    /// the sides of a square
    pub enum Side2d: u8, u8 {
        /// Left
        Left,
        /// Right
        Right,
        /// Down
        Down,
        /// Up
        Up,
    }

    /// A 1 to 1 collection for the axies of 2d orthogonal space
    pub struct Axies2d,
    /// the axies of 2d orthogonal space
    pub enum Axis2d: u8, u8 {
        /// X
        X,
        /// Y
        Y,
    }

    /// A 1 to 1 collection for the edges of a square
    pub struct Edges2d,
    /// the edges of a square
    pub enum Edge2d: u8, u8 {
        /// Left Down to Left Up
        LdToLu,
        /// Right Down to Right Up
        RdToRu,
        /// Left Down to Right Down
        LdToRd,
        /// Left Up to Right Up
        LuToRu,
    }

    /// A 1 to 1 collection for the surroundings of a center square
    pub struct Surroundings2d,
    /// the surroundings of a center square
    pub enum Surrounding2d: u8, u16 {
        /// (-1, 0)
        Nz,
        /// (1, 0)
        Pz,
        /// (0, -1)
        Zn,
        /// (0, -1)
        Zp,
        /// (-1, -1)
        Nn,
        /// (1, 1)
        Pp,
        /// (-1, 1)
        Np,
        /// (1, -1)
        Pn,
        /// (0, 0)
        Zz,
    }
}

/// Converts a side to its corners from least to most positive.
pub const SIDE_CORNERS_2D: Sides2d<[Corner2d; 2]> = Sides2d([
    [Corner2d::Ld, Corner2d::Lu],
    [Corner2d::Rd, Corner2d::Ru],
    [Corner2d::Ld, Corner2d::Rd],
    [Corner2d::Lu, Corner2d::Ru],
]);

/// Converts a corner to its sides in order of axies.
pub const CORNER_SIDES_2D: Corners2d<Axies2d<Side2d>> = Corners2d([
    Axies2d([Side2d::Left, Side2d::Down]),
    Axies2d([Side2d::Left, Side2d::Up]),
    Axies2d([Side2d::Right, Side2d::Down]),
    Axies2d([Side2d::Right, Side2d::Up]),
]);

/// Converts a corner to its neighbors in order of axies.
pub const CORNER_NEIGHBORS_2D: Corners2d<Axies2d<Corner2d>> = Corners2d([
    Axies2d([Corner2d::Rd, Corner2d::Lu]), // ld
    Axies2d([Corner2d::Ru, Corner2d::Ld]), // lu
    Axies2d([Corner2d::Ld, Corner2d::Ru]), // rd
    Axies2d([Corner2d::Lu, Corner2d::Rd]), // ru
]);

/// Walks a corner in the direction of a side, giving its neighbor to that side if it has one
#[rustfmt::skip]
pub const CORNER_WALK_2D: Corners2d<Sides2d<Option<Corner2d>>> = Corners2d([
    //         L          R              D           U
    Sides2d([None, Some(Corner2d::Rd), None, Some(Corner2d::Lu),]), // Ld
    Sides2d([None, Some(Corner2d::Ru), Some(Corner2d::Ld), None,]), // Lu
    Sides2d([Some(Corner2d::Ld), None, None, Some(Corner2d::Ru),]), // Rd
    Sides2d([Some(Corner2d::Lu), None, Some(Corner2d::Rd), None,]), // Ru
]);

/// converts a surrounding to the corners it shares with the central item
pub const ASSOCIATED_CORNERS_2D: Surroundings2d<&'static [Corner2d]> = Surroundings2d([
    &SIDE_CORNERS_2D.0[Side2d::Left as usize],
    &SIDE_CORNERS_2D.0[Side2d::Right as usize],
    &SIDE_CORNERS_2D.0[Side2d::Down as usize],
    &SIDE_CORNERS_2D.0[Side2d::Up as usize],
    &[Corner2d::Ru], // Nn
    &[Corner2d::Ld], // Pp
    &[Corner2d::Rd], // Np
    &[Corner2d::Lu], // Pn
    &Corner2d::IDENTITY.0,
]);

/// The unit corners from 0 to 1
pub const UNIT_CORNERS_IVEC2: Corners2d<IVec2> = Corners2d([
    IVec2::new(0, 0),
    IVec2::new(0, 1),
    IVec2::new(1, 0),
    IVec2::new(1, 1),
]);

/// The unit side directions or normalized orthogonal length
pub const UNIT_SIDES_IVEC2: Sides2d<IVec2> = Sides2d([
    IVec2::new(-1, 0),
    IVec2::new(1, 0),
    IVec2::new(0, -1),
    IVec2::new(0, 1),
]);

/// The unit axies
pub const UNIT_AXIES_IVEC2: Axies2d<IVec2> = Axies2d([IVec2::new(1, 0), IVec2::new(0, 1)]);

/// The unit surroundings from -1 to 1
pub const UNIT_SURROUNDINGS_IVEC2: Surroundings2d<IVec2> = Surroundings2d([
    IVec2::new(-1, 0),  // Nz
    IVec2::new(1, 0),   // Pz
    IVec2::new(0, -1),  // Zn
    IVec2::new(0, 1),   // Zp
    IVec2::new(-1, -1), // Nn
    IVec2::new(1, 1),   // Pp
    IVec2::new(-1, 1),  // Np
    IVec2::new(1, -1),  // Pn
    IVec2::new(0, 0),   // Zz
]);

/// The corners of each edge, arranged in edges order
pub const EDGE_CORNERS_2D: Edges2d<[Corner2d; 2]> = corners_to_edges_2d(Corner2d::IDENTITY);
/// The surroundings identity represented as Corners of corners.
pub const SURROUNDING_CORNERS_IDENTITY_2D: Corners2d<(Corners2d<Surrounding2d>, Corner2d)> =
    surrounding_corners_2d(Surrounding2d::IDENTITY);

/// converts a set of corners to its edges
#[inline]
pub const fn corners_to_edges_2d<T: Copy>(corners: Corners2d<T>) -> Edges2d<[T; 2]> {
    use Corner2d::*;
    Edges2d([
        [corners.0[Ld as usize], corners.0[Lu as usize]],
        [corners.0[Rd as usize], corners.0[Ru as usize]],
        [corners.0[Ld as usize], corners.0[Rd as usize]],
        [corners.0[Lu as usize], corners.0[Ru as usize]],
    ])
}

/// Given some surroundings, it returns each corner of the surroundings
/// where each corner has the 4 surroundings of the corner and the corner index of those 4 that
/// corresponds to the center.
#[inline]
pub const fn surrounding_corners_2d<T: Copy>(
    surroundings: Surroundings2d<T>,
) -> Corners2d<(Corners2d<T>, Corner2d)> {
    Corners2d([
        // Ld
        (
            Corners2d([
                surroundings.0[Surrounding2d::Nn as usize], // ld
                surroundings.0[Surrounding2d::Nz as usize], // lu
                surroundings.0[Surrounding2d::Zn as usize], // rd
                surroundings.0[Surrounding2d::Zz as usize], // ru
            ]),
            Corner2d::Ru,
        ),
        // Lu
        (
            Corners2d([
                surroundings.0[Surrounding2d::Nz as usize], // ld
                surroundings.0[Surrounding2d::Np as usize], // lu
                surroundings.0[Surrounding2d::Zz as usize], // rd
                surroundings.0[Surrounding2d::Zp as usize], // ru
            ]),
            Corner2d::Rd,
        ),
        // Rd
        (
            Corners2d([
                surroundings.0[Surrounding2d::Zn as usize], // ld
                surroundings.0[Surrounding2d::Zz as usize], // lu
                surroundings.0[Surrounding2d::Pn as usize], // rd
                surroundings.0[Surrounding2d::Pz as usize], // ru
            ]),
            Corner2d::Lu,
        ),
        // Ru
        (
            Corners2d([
                surroundings.0[Surrounding2d::Zz as usize], // ld
                surroundings.0[Surrounding2d::Zp as usize], // lu
                surroundings.0[Surrounding2d::Pz as usize], // rd
                surroundings.0[Surrounding2d::Pp as usize], // ru
            ]),
            Corner2d::Ld,
        ),
    ])
}

/// a result of 0 means they are the same. 1 means they are adjacent. 2 means they are opposites
#[inline]
pub const fn corners2d_separation(c1: Corner2d, c2: Corner2d) -> u8 {
    // each bit corresponds to a half of the cube. The xor will have a 1 for everywhere the half is
    // different for that axis.
    let separations = c1 as u8 ^ c2 as u8;
    // sum up the ones. This is branchless and only does the 3 bits needed
    let mut result = 0;
    result += (separations & 1 > 0) as u8;
    result += (separations & 2 > 0) as u8;
    result
}

/// converts an edge to its axis. Edges are always oriented positively
#[inline]
pub const fn edge2d_axis(edge: Edge2d) -> Axis2d {
    let axis = edge as u8 / 2;
    // SAFETY: the index is known to be valid since there are 2 edges for each axis
    unsafe { Axis2d::from_const_index(axis) }
}

/// returns if the `corner` is on the negative half of the `axis`
#[inline]
pub const fn corner2d_is_neg(corner: Corner2d, axis: Axis2d) -> bool {
    corner as u8 & (1 << (Axis2d::MAX - axis as u8)) == 0
}

/// returns if the side if facing negatively
#[inline]
pub const fn side2d_is_neg(side: Side2d) -> bool {
    side as u8 & 1 == 0
}

/// Converts a side to its axis
#[inline]
pub const fn side2d_to_axis2d(side: Side2d) -> Axis2d {
    let side_index = side as u8 / 2;
    // SAFETY: the index is known to be valid
    unsafe { Axis2d::from_const_index(side_index) }
}

/// inverts a side's direction, keeping its axis
#[inline]
pub const fn invert_side2d(side: Side2d) -> Side2d {
    // SAFETY: the index is known to be valid since there are an even number of sides.
    unsafe { Side2d::from_const_index(side as u8 ^ 1) }
}

/// converts a corner to its opposite.
#[inline]
pub const fn invert_corner2d(corner: Corner2d) -> Corner2d {
    let inverted = Corner2d::MAX - corner as u8;
    // SAFETY: the index is known to be valid since we ust subtracted it from the max.
    unsafe { Corner2d::from_const_index(inverted) }
}

/// Flatens a 2d index into a single value losslessly where L is the length of this 2d space.
/// Note that if the only goal is to fit a vector into a number, you may want to instead just merge
/// the bits together. This flattening is special because it keeps the values continuous. (adding
/// any power of `L` to a valid compression gives a position adjacent from the original).
/// See also: [`expand2d`]
#[inline]
pub const fn flatten2d<const L: usize>(x: usize, y: usize) -> usize {
    x + y * L
}

/// expands a single index to its 2d coordinates where L is the length of this 2d space.
/// /// See also: [`flatten2d`]
#[inline]
pub const fn expand2d<const L: usize>(i: usize) -> (usize, usize) {
    let y = i / L;
    let x = i - (y * L);
    (x, y)
}

impl From<FlagSet<Axis2d>> for Corner2d {
    #[inline]
    fn from(value: FlagSet<Axis2d>) -> Self {
        let mut result = 0u8;
        let value = value.bits();
        result |= (value & 1) << 1; // Axis x
        result |= (value & 2) >> 1; // Axis y
        // SAFETY: The total bits are less than 4 no matter what.
        unsafe { Corner2d::from_const_index(result) }
    }
}

impl From<Corner2d> for FlagSet<Axis2d> {
    #[inline]
    fn from(value: Corner2d) -> Self {
        let mut result = 0u8;
        let value = value as u8;
        result |= (value & 2) >> 1; // Axis x
        result |= (value & 1) << 1; // Axis y
        // SAFETY: The total bits are less than 4 no matter what.
        unsafe { Self::new_unchecked(result) }
    }
}

impl From<Side2d> for Surrounding2d {
    #[inline]
    fn from(value: Side2d) -> Self {
        // SAFETY: The indices line up exactly.
        unsafe { Surrounding2d::from_const_index(value.get_index()) }
    }
}

impl From<Axis2d> for AxisDirections<Side2d> {
    #[inline]
    fn from(value: Axis2d) -> Self {
        let negative = value as u8 * 2;
        let positive = negative + 1;
        // SAFETY: the index is known to be valid since there are 2 axies for each side
        unsafe {
            [
                Side2d::from_const_index(negative),
                Side2d::from_const_index(positive),
            ]
            .into()
        }
    }
}

impl Corner2d {
    /// creates a corner given which half it is on of each axis, represented as a BVec
    #[inline]
    pub fn from_signs(positive: BVec2) -> Self {
        let mut result = 0u8;
        result |= (positive.x as u8) << 1;
        result |= positive.y as u8;
        // SAFETY: There are exactly 4 possibilities here, which match exactly the four corners.
        unsafe { Self::from_const_index(result) }
    }
}

// impl<T: Lerpable + Copy> Corners2d<T> {
//     /// performs an interpolation within the square formed by these corners  to the coordinates
// in     /// `by` according to the `curve`
//     #[inline(always)]
//     pub fn interpolate_2d<I: Mixable<T> + Copy>(
//         &self,
//         by: Axies2d<I>,
//         curve: &impl MixerFxn<I>,
//     ) -> T {
//         let lr = by[Axis2d::X].apply_mixer(curve);
//         let du = by[Axis2d::Y].apply_mixer(curve);
//         let left = T::lerp_dirty(self[Corner2d::Ld], self[Corner2d::Lu], du);
//         let right = T::lerp_dirty(self[Corner2d::Rd], self[Corner2d::Ru], du);
//         T::lerp_dirty(left, right, lr)
//     }

//     /// performs an interpolation gradient within the square formed by these corners  to the
//     /// coordinates in `by` according to the `curve`
//     #[inline(always)]
//     pub fn interpolate_gradient_2d<I: Mixable<T> + Copy>(
//         &self,
//         by: Axies2d<I>,
//         curve: &impl MixerFxn<I>,
//     ) -> Axies2d<T> {
//         let gradients = Side2d::IDENTITY.map(|s| {
//             let [c1, c2] = SIDE_CORNERS_2D[s];
//             T::lerp_gradient(self[c1], self[c2])
//         });
//         Axies2d([
//             T::lerp_dirty(
//                 gradients[Side2d::Down],
//                 gradients[Side2d::Up],
//                 by[Axis2d::Y].apply_mixer(curve),
//             ) * by[Axis2d::X].apply_mixer_derivative(curve),
//             T::lerp_dirty(
//                 gradients[Side2d::Left],
//                 gradients[Side2d::Right],
//                 by[Axis2d::X].apply_mixer(curve),
//             ) * by[Axis2d::Y].apply_mixer_derivative(curve),
//         ])
//     }

//     /// performs an interpolation and gradient within the square formed by these corners  to the
//     /// coordinates in `by` according to the `curve`
//     #[inline(always)]
//     pub fn interpolate_and_gradient_2d<I: Mixable<T> + Copy>(
//         &self,
//         by: Axies2d<I>,
//         curve: &impl MixerFxn<I>,
//     ) -> (T, Axies2d<T>) {
//         (
//             self.interpolate_2d(by, curve),
//             self.interpolate_gradient_2d(by, curve),
//         )
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sides_and_axies() {
        for axis in Axis2d::IDENTITY {
            for side in AxisDirections::from(axis) {
                assert_eq!(axis, side2d_to_axis2d(side));
            }
        }
    }

    #[test]
    fn test_inversion() {
        use Corner2d::*;
        assert_eq!(
            Corner2d::IDENTITY.map(invert_corner2d),
            Corners2d([Ru, Rd, Lu, Ld])
        );
        use Side2d::*;
        assert_eq!(
            Side2d::IDENTITY.map(invert_side2d),
            Sides2d([Right, Left, Up, Down])
        );
    }

    #[test]
    fn test_signs() {
        assert_eq!(
            Corner2d::IDENTITY.0.map(|c| corner2d_is_neg(c, Axis2d::X)),
            [true, true, false, false]
        );
        assert_eq!(
            Corner2d::IDENTITY.0.map(|c| corner2d_is_neg(c, Axis2d::Y)),
            [true, false, true, false]
        );
        assert_eq!(
            Side2d::IDENTITY.0.map(side2d_is_neg),
            [true, false, true, false]
        );
    }

    #[test]
    fn test_s() {
        use Corner2d::*;
        assert_eq!(Corner2d::IDENTITY.0.map(Corner2d::from), [Ld, Lu, Rd, Ru,]);
    }

    #[test]
    fn test_corner_axies_conversions() {
        for c in Corner2d::IDENTITY {
            let a = FlagSet::<Axis2d>::from(c);
            let back = Corner2d::from(a);
            assert_eq!(c, back);
        }
        for bits in 0..=Corner2d::MAX {
            let a = FlagSet::<Axis2d>::new_truncated(bits);
            let c = Corner2d::from(a);
            let back = FlagSet::<Axis2d>::from(c);
            assert_eq!(a, back);
        }
    }

    #[test]
    fn test_side_surroundings_conversion() {
        use Surrounding2d::*;
        assert_eq!(
            Side2d::IDENTITY.0.map(Surrounding2d::from),
            [Nz, Pz, Zn, Zp]
        );
    }

    #[test]
    fn test_corner_separation() {
        for c in Corner2d::IDENTITY {
            assert_eq!(corners2d_separation(c, c), 0);
            assert_eq!(corners2d_separation(invert_corner2d(c), c), 2);
        }
        for [c1, c2] in EDGE_CORNERS_2D {
            assert_eq!(corners2d_separation(c1, c2), 1);
        }
    }

    #[test]
    fn corner_from_signs() {
        for c in Corner2d::IDENTITY {
            let x = !corner2d_is_neg(c, Axis2d::X);
            let y = !corner2d_is_neg(c, Axis2d::Y);
            let back = Corner2d::from_signs(BVec2::new(x, y));
            assert_eq!(c, back);
        }
    }
}
