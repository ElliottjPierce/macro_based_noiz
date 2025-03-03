//! 3d orthogonal space utilities.

use bevy_math::{
    BVec3,
    IVec3,
};
use flagset::FlagSet;

use super::square::{
    Corners2d,
    expand2d,
    flatten2d,
};
use crate::{
    name_array,
    spatial::named_array::NamedArrayIndices,
};

name_array! {
    /// A 1 to 1 collection for the corners of a cube
    pub struct Corners3d,
    /// the corners of a cube
    pub enum Corner3d: u8, u8 {
        /// Left Down Back
        Ldb,
        /// Left Down Front
        Ldf,
        /// Left Up Back
        Lub,
        /// Left Up Front
        Luf,
        /// Right Down Back
        Rdb,
        /// Right Down Front
        Rdf,
        /// Right Up Back
        Rub,
        /// Right Up Front
        Ruf,
    }

    /// A 1 to 1 collection for the sides of a cube
    pub struct Sides3d,
    /// the sides of a cube
    pub enum Side3d: u8, u8 {
        /// Left
        Left,
        /// Right
        Right,
        /// Down
        Down,
        /// Up
        Up,
        /// Back
        Back,
        /// Front
        Front,
    }

    /// A 1 to 1 collection for the axies of a cube
    pub struct Axies3d,
    /// the axies of a cube
    pub enum Axis3d: u8, u8 {
        /// X
        X,
        /// Y
        Y,
        /// Z
        Z,
    }

    /// A 1 to 1 collection for the edges of a cube
    pub struct Edges3d,
    /// the edges of a cube
    pub enum Edge3d: u8, u16 {
        /// Left Down Back To Right Down Back
        LdbToRdb,
        /// Left Down Front To Right Down Front
        LdfToRdf,
        /// Left Up Back To Right Up Back
        LubToRub,
        /// Left Up Front To Right Up Front
        LufToRuf,
        /// Left Down Back To Left Up Back
        LdbToLub,
        /// Left Down Front To Left Up Front
        LdfToLuf,
        /// Right Down Back To Right Up Back
        RdbToRub,
        /// Right Down Front To Right Up Front
        RdfToRuf,
        /// Left Down Back To Left Down Front
        LdbToLdf,
        /// Left Up Back To Left Up Front
        LubToLuf,
        /// Right Down Back To Right Down Front
        RdbToRdf,
        /// Right Up Back To Right Up Front
        RubToRuf,
    }

    /// A 1 to 1 collection for the surroundings of a center cube
    pub struct Surroundings3d,
    /// the Surroundings of a center cube
    pub enum Surrounding3d: u8, u32 {
        /// (-1, 0, 0)
        Nzz,
        /// (1, 0, 0)
        Pzz,
        /// (0, -1, 0)
        Znz,
        /// (0, 1, 0)
        Zpz,
        /// (0, 0, -1)
        Zzn,
        /// (0, 0, 1)
        Zzp,
        /// (0, -1, -1)
        Znn,
        /// (0, 1, 1)
        Zpp,
        /// (0, -1, 1)
        Znp,
        /// (0, 1, -1)
        Zpn,
        /// (-1, 0, -1)
        Nzn,
        /// (1, 0, 1)
        Pzp,
        /// (-1, 0, 1)
        Nzp,
        /// (1, 0, -1)
        Pzn,
        /// (-1, -1, 0)
        Nnz,
        /// (1, 1, 0)
        Ppz,
        /// (-1, 1, 0)
        Npz,
        /// (1, -1, 0)
        Pnz,
        /// (-1, -1, -1)
        Nnn,
        /// (1, 1, -1)
        Ppn,
        /// (-1, 1, -1)
        Npn,
        /// (1, -1, -1)
        Pnn,
        /// (-1, -1, 1)
        Nnp,
        /// (1, 1, 1)
        Ppp,
        /// (-1, 1, 1)
        Npp,
        /// (1, -1, 1)
        Pnp,
        /// (0, 0, 0)
        Zzz,
    }
}

/// Converts a side to its corners from least to most positive.
pub const SIDE_CORNERS_3D: Sides3d<Corners2d<Corner3d>> = Sides3d([
    Corners2d([Corner3d::Ldb, Corner3d::Ldf, Corner3d::Lub, Corner3d::Luf]), // left
    Corners2d([Corner3d::Rdb, Corner3d::Rdf, Corner3d::Rub, Corner3d::Ruf]), // right
    Corners2d([Corner3d::Ldb, Corner3d::Ldf, Corner3d::Rdb, Corner3d::Rdf]), // down
    Corners2d([Corner3d::Lub, Corner3d::Luf, Corner3d::Rub, Corner3d::Ruf]), // up
    Corners2d([Corner3d::Ldb, Corner3d::Lub, Corner3d::Rdb, Corner3d::Rub]), // back
    Corners2d([Corner3d::Ldf, Corner3d::Luf, Corner3d::Rdf, Corner3d::Ruf]), // front
]);

/// Converts a corner to its sides in order of axies.
pub const CORNER_SIDES_3D: Corners3d<Axies3d<Side3d>> = Corners3d([
    Axies3d([Side3d::Left, Side3d::Down, Side3d::Back]),
    Axies3d([Side3d::Left, Side3d::Down, Side3d::Front]),
    Axies3d([Side3d::Left, Side3d::Up, Side3d::Back]),
    Axies3d([Side3d::Left, Side3d::Up, Side3d::Front]),
    Axies3d([Side3d::Right, Side3d::Down, Side3d::Back]),
    Axies3d([Side3d::Right, Side3d::Down, Side3d::Front]),
    Axies3d([Side3d::Right, Side3d::Up, Side3d::Back]),
    Axies3d([Side3d::Right, Side3d::Up, Side3d::Front]),
]);

/// Converts a corner to its neighbors in order of axies.
pub const CORNER_NEIGHBORS_3D: Corners3d<Axies3d<Corner3d>> = Corners3d([
    Axies3d([Corner3d::Rdb, Corner3d::Lub, Corner3d::Ldf]), // Ldb
    Axies3d([Corner3d::Rdf, Corner3d::Luf, Corner3d::Ldb]), // Ldf
    Axies3d([Corner3d::Rub, Corner3d::Ldb, Corner3d::Luf]), // Lub
    Axies3d([Corner3d::Ruf, Corner3d::Ldf, Corner3d::Lub]), // Luf
    Axies3d([Corner3d::Ldb, Corner3d::Rub, Corner3d::Rdf]), // Rdb
    Axies3d([Corner3d::Ldf, Corner3d::Ruf, Corner3d::Rdb]), // Rdf
    Axies3d([Corner3d::Lub, Corner3d::Rdb, Corner3d::Ruf]), // Rub
    Axies3d([Corner3d::Luf, Corner3d::Rdf, Corner3d::Rub]), // Ruf
]);

/// Converts a corner to the edges in which it appears, arranged by axis
pub const CORNER_EDGES_3D: Corners3d<Axies3d<Edge3d>> = Corners3d([
    Axies3d([Edge3d::LdbToRdb, Edge3d::LdbToLub, Edge3d::LdbToLdf]),
    Axies3d([Edge3d::LdfToRdf, Edge3d::LdfToLuf, Edge3d::LdbToLdf]),
    Axies3d([Edge3d::LubToRub, Edge3d::LdbToLub, Edge3d::LubToLuf]),
    Axies3d([Edge3d::LufToRuf, Edge3d::LdfToLuf, Edge3d::LubToLuf]),
    Axies3d([Edge3d::LdbToRdb, Edge3d::RdbToRub, Edge3d::RdbToRdf]),
    Axies3d([Edge3d::LdfToRdf, Edge3d::RdfToRuf, Edge3d::RdbToRdf]),
    Axies3d([Edge3d::LubToRub, Edge3d::RdbToRub, Edge3d::RubToRuf]),
    Axies3d([Edge3d::LufToRuf, Edge3d::RdfToRuf, Edge3d::RubToRuf]),
]);

/// Lists the combinations of edges on each face
pub const EDGE_FACE_CONNECTIONS_3D: Sides3d<[[Edge3d; 2]; 6]> = Sides3d([
    // LEFT
    [
        [Edge3d::LdbToLub, Edge3d::LdbToLdf],
        [Edge3d::LdbToLub, Edge3d::LubToLuf],
        [Edge3d::LdbToLub, Edge3d::LdfToLuf],
        [Edge3d::LdfToLuf, Edge3d::LubToLuf],
        [Edge3d::LdfToLuf, Edge3d::LdbToLdf],
        [Edge3d::LdbToLdf, Edge3d::LubToLuf],
    ],
    // RIGHT
    [
        [Edge3d::RdbToRub, Edge3d::RdbToRdf],
        [Edge3d::RdbToRub, Edge3d::RubToRuf],
        [Edge3d::RdbToRub, Edge3d::RdfToRuf],
        [Edge3d::RdfToRuf, Edge3d::RubToRuf],
        [Edge3d::RdfToRuf, Edge3d::RdbToRdf],
        [Edge3d::RdbToRdf, Edge3d::RubToRuf],
    ],
    // DOWN
    [
        [Edge3d::LdbToRdb, Edge3d::LdbToLdf],
        [Edge3d::LdbToRdb, Edge3d::RdbToRdf],
        [Edge3d::LdbToRdb, Edge3d::LdfToRdf],
        [Edge3d::LdfToRdf, Edge3d::RdbToRdf],
        [Edge3d::LdfToRdf, Edge3d::LdbToLdf],
        [Edge3d::LdbToLdf, Edge3d::RdbToRdf],
    ],
    // UP
    [
        [Edge3d::LubToRub, Edge3d::LubToLuf],
        [Edge3d::LubToRub, Edge3d::RubToRuf],
        [Edge3d::LubToRub, Edge3d::LufToRuf],
        [Edge3d::LufToRuf, Edge3d::RubToRuf],
        [Edge3d::LufToRuf, Edge3d::LubToLuf],
        [Edge3d::LubToLuf, Edge3d::RubToRuf],
    ],
    // BACK
    [
        [Edge3d::LdbToLub, Edge3d::LdbToRdb],
        [Edge3d::LdbToLub, Edge3d::LubToRub],
        [Edge3d::LdbToLub, Edge3d::RdbToRub],
        [Edge3d::RdbToRub, Edge3d::LubToRub],
        [Edge3d::RdbToRub, Edge3d::LdbToRdb],
        [Edge3d::LdbToRdb, Edge3d::LubToRub],
    ],
    // FRONT
    [
        [Edge3d::LdfToLuf, Edge3d::LdfToRdf],
        [Edge3d::LdfToLuf, Edge3d::LufToRuf],
        [Edge3d::LdfToLuf, Edge3d::RdfToRuf],
        [Edge3d::RdfToRuf, Edge3d::LufToRuf],
        [Edge3d::RdfToRuf, Edge3d::LdfToRdf],
        [Edge3d::LdfToRdf, Edge3d::LufToRuf],
    ],
]);

/// Walks a corner in the direction of a side, giving its neighbor to that side if it has one
#[rustfmt::skip]
pub const CORNER_WALK_3D: Corners3d<Sides3d<Option<Corner3d>>> = Corners3d([
    // L          R    D           U    B           F
    Sides3d([None, Some(Corner3d::Rdb), None, Some(Corner3d::Lub), None, Some(Corner3d::Ldf)]), // Ldb
    Sides3d([None, Some(Corner3d::Rdf), None, Some(Corner3d::Luf), Some(Corner3d::Ldb), None]), // Ldf
    Sides3d([None, Some(Corner3d::Rub), Some(Corner3d::Ldb), None, None, Some(Corner3d::Luf)]), // Lub
    Sides3d([None, Some(Corner3d::Ruf), Some(Corner3d::Ldf), None, Some(Corner3d::Lub), None]), // Luf
    Sides3d([Some(Corner3d::Ldb), None, None, Some(Corner3d::Rub), None, Some(Corner3d::Rdf)]), // Rdb
    Sides3d([Some(Corner3d::Ldf), None, None, Some(Corner3d::Ruf), Some(Corner3d::Rdb), None]), // Rdf
    Sides3d([Some(Corner3d::Lub), None, Some(Corner3d::Rdb), None, None, Some(Corner3d::Ruf)]), // Rub
    Sides3d([Some(Corner3d::Luf), None, Some(Corner3d::Rdf), None, Some(Corner3d::Rub), None]), // Ruf
]);

/// converts a Surrounding to the corners it shares with the central item
pub const ASSOCIATED_CORNERS_3D: Surroundings3d<&'static [Corner3d]> = Surroundings3d([
    &SIDE_CORNERS_3D.0[Side3d::Left as usize].0,
    &SIDE_CORNERS_3D.0[Side3d::Right as usize].0,
    &SIDE_CORNERS_3D.0[Side3d::Down as usize].0,
    &SIDE_CORNERS_3D.0[Side3d::Up as usize].0,
    &SIDE_CORNERS_3D.0[Side3d::Back as usize].0,
    &SIDE_CORNERS_3D.0[Side3d::Front as usize].0,
    &[Corner3d::Ruf, Corner3d::Luf], // ADJ_0NN
    &[Corner3d::Rdb, Corner3d::Ldb], // ADJ_0PP
    &[Corner3d::Rub, Corner3d::Lub], // ADJ_0NP
    &[Corner3d::Rdf, Corner3d::Ldf], // ADJ_0PN
    &[Corner3d::Ruf, Corner3d::Rdf], // ADJ_N0N
    &[Corner3d::Lub, Corner3d::Ldb], // ADJ_P0P
    &[Corner3d::Rub, Corner3d::Rdb], // ADJ_N0P
    &[Corner3d::Luf, Corner3d::Ldf], // ADJ_P0N
    &[Corner3d::Ruf, Corner3d::Rub], // ADJ_NN0
    &[Corner3d::Ldf, Corner3d::Ldb], // ADJ_PP0
    &[Corner3d::Rdf, Corner3d::Rdb], // ADJ_NP0
    &[Corner3d::Luf, Corner3d::Lub], // ADJ_PN0
    &[Corner3d::Ruf],                // ADJ_NNN
    &[Corner3d::Ldf],                // ADJ_PPN
    &[Corner3d::Rdf],                // ADJ_NPN
    &[Corner3d::Luf],                // ADJ_PNN
    &[Corner3d::Rub],                // ADJ_NNP
    &[Corner3d::Ldb],                // ADJ_PPP
    &[Corner3d::Rdb],                // ADJ_NPP
    &[Corner3d::Lub],                // ADJ_PNP
    &Corner3d::IDENTITY.0,           // ADJ_000
]);

/// The unit corners from 0 to 1
pub const UNIT_CORNERS_IVEC3: Corners3d<IVec3> = Corners3d([
    IVec3::new(0, 0, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 1, 0),
    IVec3::new(0, 1, 1),
    IVec3::new(1, 0, 0),
    IVec3::new(1, 0, 1),
    IVec3::new(1, 1, 0),
    IVec3::new(1, 1, 1),
]);

/// The unit side directions or normalized orthogonal length
pub const UNIT_SIDES_IVEC3: Sides3d<IVec3> = Sides3d([
    IVec3::new(-1, 0, 0),
    IVec3::new(1, 0, 0),
    IVec3::new(0, -1, 0),
    IVec3::new(0, 1, 0),
    IVec3::new(0, 0, -1),
    IVec3::new(0, 0, 1),
]);

/// The unit axies
pub const UNIT_AXIES_IVEC3: Axies3d<IVec3> = Axies3d([
    IVec3::new(1, 0, 0),
    IVec3::new(0, 1, 0),
    IVec3::new(0, 0, 1),
]);

/// The unit Surroundings from -1 to 1
pub const UNIT_SURROUNDINGS_IVEC3: Surroundings3d<IVec3> = Surroundings3d([
    IVec3::new(-1, 0, 0),   // Nzz
    IVec3::new(1, 0, 0),    // Pzz
    IVec3::new(0, -1, 0),   // Znz
    IVec3::new(0, 1, 0),    // Zpz
    IVec3::new(0, 0, -1),   // Zzn
    IVec3::new(0, 0, 1),    // Zzp
    IVec3::new(0, -1, -1),  // Znn
    IVec3::new(0, 1, 1),    // Zpp
    IVec3::new(0, -1, 1),   // Znp
    IVec3::new(0, 1, -1),   // Zpn
    IVec3::new(-1, 0, -1),  // Nzn
    IVec3::new(1, 0, 1),    // Pzp
    IVec3::new(-1, 0, 1),   // Nzp
    IVec3::new(1, 0, -1),   // Pzn
    IVec3::new(-1, -1, 0),  // Nnz
    IVec3::new(1, 1, 0),    // Ppz
    IVec3::new(-1, 1, 0),   // Npz
    IVec3::new(1, -1, 0),   // Pnz
    IVec3::new(-1, -1, -1), // Nnn
    IVec3::new(1, 1, -1),   // Ppn
    IVec3::new(-1, 1, -1),  // Npn
    IVec3::new(1, -1, -1),  // Pnn
    IVec3::new(-1, -1, 1),  // Nnp
    IVec3::new(1, 1, 1),    // Ppp
    IVec3::new(-1, 1, 1),   // Npp
    IVec3::new(1, -1, 1),   // Pnp
    IVec3::new(0, 0, 0),    // Zzz
]);

/// The corners of each edge, arranged in edges order
pub const EDGE_CORNERS_3D: Edges3d<[Corner3d; 2]> = corners_to_edges_3d(Corner3d::IDENTITY);
/// The Surroundings identity represented as Corners of corners.
pub const SURROUNDING_CORNERS_IDENTITY_3_D: Corners3d<(Corners3d<Surrounding3d>, Corner3d)> =
    surrounding_corners_3d(Surrounding3d::IDENTITY);

/// converts a set of corners to its edges
#[inline]
pub const fn corners_to_edges_3d<T: Copy>(corners: Corners3d<T>) -> Edges3d<[T; 2]> {
    use Corner3d::*;
    Edges3d([
        [corners.0[Ldb as usize], corners.0[Rdb as usize]],
        [corners.0[Ldf as usize], corners.0[Rdf as usize]],
        [corners.0[Lub as usize], corners.0[Rub as usize]],
        [corners.0[Luf as usize], corners.0[Ruf as usize]],
        [corners.0[Ldb as usize], corners.0[Lub as usize]],
        [corners.0[Ldf as usize], corners.0[Luf as usize]],
        [corners.0[Rdb as usize], corners.0[Rub as usize]],
        [corners.0[Rdf as usize], corners.0[Ruf as usize]],
        [corners.0[Ldb as usize], corners.0[Ldf as usize]],
        [corners.0[Lub as usize], corners.0[Luf as usize]],
        [corners.0[Rdb as usize], corners.0[Rdf as usize]],
        [corners.0[Rub as usize], corners.0[Ruf as usize]],
    ])
}

/// Given some Surroundings, it returns each corner of the Surroundings
/// where each corner has the 8 Surroundings of the corner and the corner index of those 8 that
/// corresponds to the center.
#[inline]
pub const fn surrounding_corners_3d<T: Copy>(
    surroundings: Surroundings3d<T>,
) -> Corners3d<(Corners3d<T>, Corner3d)> {
    Corners3d([
        // Ldb
        (
            Corners3d([
                surroundings.0[Surrounding3d::Nnn as usize], // ldb
                surroundings.0[Surrounding3d::Nnz as usize], // ldf
                surroundings.0[Surrounding3d::Nzn as usize], // lub
                surroundings.0[Surrounding3d::Nzz as usize], // luf
                surroundings.0[Surrounding3d::Znn as usize], // rdb
                surroundings.0[Surrounding3d::Znz as usize], // rdf
                surroundings.0[Surrounding3d::Zzn as usize], // rub
                surroundings.0[Surrounding3d::Zzz as usize], // ruf
            ]),
            Corner3d::Ruf,
        ),
        // Ldf
        (
            Corners3d([
                surroundings.0[Surrounding3d::Nnz as usize], // ldb
                surroundings.0[Surrounding3d::Nnp as usize], // ldf
                surroundings.0[Surrounding3d::Nzz as usize], // lub
                surroundings.0[Surrounding3d::Nzp as usize], // luf
                surroundings.0[Surrounding3d::Znz as usize], // rdb
                surroundings.0[Surrounding3d::Znp as usize], // rdf
                surroundings.0[Surrounding3d::Zzz as usize], // rub
                surroundings.0[Surrounding3d::Zzp as usize], // ruf
            ]),
            Corner3d::Rub,
        ),
        // Lub
        (
            Corners3d([
                surroundings.0[Surrounding3d::Nzn as usize], // ldb
                surroundings.0[Surrounding3d::Nzz as usize], // ldf
                surroundings.0[Surrounding3d::Npn as usize], // lub
                surroundings.0[Surrounding3d::Npz as usize], // luf
                surroundings.0[Surrounding3d::Zzn as usize], // rdb
                surroundings.0[Surrounding3d::Zzz as usize], // rdf
                surroundings.0[Surrounding3d::Zpn as usize], // rub
                surroundings.0[Surrounding3d::Zpz as usize], // ruf
            ]),
            Corner3d::Rdf,
        ),
        // Luf
        (
            Corners3d([
                surroundings.0[Surrounding3d::Nzz as usize], // ldb
                surroundings.0[Surrounding3d::Nzp as usize], // ldf
                surroundings.0[Surrounding3d::Npz as usize], // lub
                surroundings.0[Surrounding3d::Npp as usize], // luf
                surroundings.0[Surrounding3d::Zzz as usize], // rdb
                surroundings.0[Surrounding3d::Zzp as usize], // rdf
                surroundings.0[Surrounding3d::Zpz as usize], // rub
                surroundings.0[Surrounding3d::Zpp as usize], // ruf
            ]),
            Corner3d::Rdb,
        ),
        // Rdb
        (
            Corners3d([
                surroundings.0[Surrounding3d::Znn as usize], // ldb
                surroundings.0[Surrounding3d::Znz as usize], // ldf
                surroundings.0[Surrounding3d::Zzn as usize], // lub
                surroundings.0[Surrounding3d::Zzz as usize], // luf
                surroundings.0[Surrounding3d::Pnn as usize], // rdb
                surroundings.0[Surrounding3d::Pnz as usize], // rdf
                surroundings.0[Surrounding3d::Pzn as usize], // rub
                surroundings.0[Surrounding3d::Pzz as usize], // ruf
            ]),
            Corner3d::Luf,
        ),
        // Rdf
        (
            Corners3d([
                surroundings.0[Surrounding3d::Znz as usize], // ldb
                surroundings.0[Surrounding3d::Znp as usize], // ldf
                surroundings.0[Surrounding3d::Zzz as usize], // lub
                surroundings.0[Surrounding3d::Zzp as usize], // luf
                surroundings.0[Surrounding3d::Pnz as usize], // rdb
                surroundings.0[Surrounding3d::Pnp as usize], // rdf
                surroundings.0[Surrounding3d::Pzz as usize], // rub
                surroundings.0[Surrounding3d::Pzp as usize], // ruf
            ]),
            Corner3d::Lub,
        ),
        // Rub
        (
            Corners3d([
                surroundings.0[Surrounding3d::Zzn as usize], // ldb
                surroundings.0[Surrounding3d::Zzz as usize], // ldf
                surroundings.0[Surrounding3d::Zpn as usize], // lub
                surroundings.0[Surrounding3d::Zpz as usize], // luf
                surroundings.0[Surrounding3d::Pzn as usize], // rdb
                surroundings.0[Surrounding3d::Pzz as usize], // rdf
                surroundings.0[Surrounding3d::Ppn as usize], // rub
                surroundings.0[Surrounding3d::Ppz as usize], // ruf
            ]),
            Corner3d::Ldf,
        ),
        // Ruf
        (
            Corners3d([
                surroundings.0[Surrounding3d::Zzz as usize], // ldb
                surroundings.0[Surrounding3d::Zzp as usize], // ldf
                surroundings.0[Surrounding3d::Zpz as usize], // lub
                surroundings.0[Surrounding3d::Zpp as usize], // luf
                surroundings.0[Surrounding3d::Pzz as usize], // rdb
                surroundings.0[Surrounding3d::Pzp as usize], // rdf
                surroundings.0[Surrounding3d::Ppz as usize], // rub
                surroundings.0[Surrounding3d::Ppp as usize], // ruf
            ]),
            Corner3d::Ldb,
        ),
    ])
}

/// a result of 0 means they are the same. 1 means they are adjacent. 2 means they are a face
/// diagonal. 3 means they are opposites.
#[inline]
pub fn corners3d_separation(c1: Corner3d, c2: Corner3d) -> u8 {
    // each bit corresponds to a half of the cube. The xor will have a 1 for everywhere the half is
    // different for that axis.
    let separations = c1 as u8 ^ c2 as u8;
    // sum up the ones. This is branchless and only does the 3 bits needed
    let mut result = 0;
    result += (separations & 1 > 0) as u8;
    result += (separations & 2 > 0) as u8;
    result += (separations & 4 > 0) as u8;
    result
}

/// converts an edge to its axis. Edges are always oriented positively
#[inline]
pub const fn edge3d_axis(edge: Edge3d) -> Axis3d {
    let axis = edge as u8 / 4;
    // SAFETY: There are exactly 4 edges per axis
    unsafe { Axis3d::from_const_index(axis) }
}

/// returns if the `corner` is on the negative half of the `axis`
#[inline]
pub const fn corner3d_is_neg(corner: Corner3d, axis: Axis3d) -> bool {
    corner as u8 & (1 << (Axis3d::MAX - axis as u8)) == 0
}

/// returns if the side if facing negatively
#[inline]
pub const fn side3d_is_neg(side: Side3d) -> bool {
    side as u8 & 1 == 0
}

/// Converts a side to its axis
#[inline]
pub const fn side3d_to_axis3d(side: Side3d) -> Axis3d {
    let side_index = side as u8 / 2;
    // SAFETY: There are exactly 2 sides per axis.
    unsafe { Axis3d::from_const_index(side_index) }
}

/// converts an axis to its sides, negative first, then positive
#[inline]
pub const fn axis3d_to_side3d(axis: Axis3d) -> [Side3d; 2] {
    let negative = axis as u8 * 2;
    let positive = negative + 1;
    // SAFETY: there are exactly 2 sides per axis.
    unsafe {
        [
            Side3d::from_const_index(negative),
            Side3d::from_const_index(positive),
        ]
    }
}

/// inverts a side's direction, keeping its axis
#[inline]
pub const fn invert_side3d(side: Side3d) -> Side3d {
    // SAFETY: There are an even number of sides.
    unsafe { Side3d::from_const_index(side as u8 ^ 1) }
}

/// converts a corner to its opposite.
#[inline]
pub const fn invert_corner3d(corner: Corner3d) -> Corner3d {
    let inverted = Corner3d::MAX - corner as u8;
    // SAFETY: we just subtracted from the max, so it must be valid.
    unsafe { Corner3d::from_const_index(inverted) }
}

/// Flatens a 3d index into a single value losslessly where L is the length of this 3d space.
/// Note that if the only goal is to fit a vector into a number, you may want to instead just merge
/// the bits together. This flattening is special because it keeps the values continuous. (adding
/// any power of `L` to a valid compression gives a position adjacent from the original).
/// See also: [`expand3d`]
#[inline]
pub const fn flatten3d<const L: usize>(x: usize, y: usize, z: usize) -> usize {
    flatten2d::<L>(x, y) + z * L.pow(2)
}

/// expands a single index to its 3d coordinates where L is the length of this 3d space.
/// /// See also: [`flatten3d`]
#[inline]
pub const fn expand3d<const L: usize>(i: usize) -> (usize, usize, usize) {
    let z = i / L.pow(2);
    let xy = i - z * L.pow(2);
    let (x, y) = expand2d::<L>(xy);
    (x, y, z)
}

impl From<FlagSet<Axis3d>> for Corner3d {
    #[inline]
    fn from(value: FlagSet<Axis3d>) -> Self {
        let mut result = 0u8;
        let value = value.bits();
        result |= (value & 1) << 2; // Axis x
        result |= value & 2; // Axis y
        result |= (value & 4) >> 2; // Axis z
        // SAFETY: There are a total of 3 bits here, we just reverse their order
        unsafe { Corner3d::from_const_index(result) }
    }
}

impl From<Corner3d> for FlagSet<Axis3d> {
    #[inline]
    fn from(value: Corner3d) -> Self {
        let mut result = 0u8;
        let value = value as u8;
        result |= (value & 4) >> 2; // Axis x
        result |= value & 2; // Axis y
        result |= (value & 1) << 2; // Axis z
        // SAFETY: There are a total of 3 bits here, we just reverse their order
        unsafe { Self::new_unchecked(result) }
    }
}

impl From<Side3d> for Surrounding3d {
    #[inline]
    fn from(value: Side3d) -> Self {
        // SAFETY: The sides and surroundings exactly line up.
        unsafe { Surrounding3d::from_const_index(value.get_index()) }
    }
}

impl Corner3d {
    /// creates a corner given which half it is on of each axis, represented as a BVec
    #[inline]
    pub fn from_signs(positive: BVec3) -> Self {
        let mut result = 0u8;
        result |= (positive.x as u8) << 2;
        result |= (positive.y as u8) << 1;
        result |= positive.z as u8;
        // SAFETY: there are exactly 8 possibilities here that map exactly to the corner.
        unsafe { Self::from_const_index(result) }
    }
}

// impl<T: Lerpable + Copy> Corners3d<T> {
//     /// performs an interpolation within the cube formed by these corners  to the coordinates in
// `by`     /// according to the `curve`
//     #[inline(always)]
//     pub fn interpolate_3d<I: Mixable<T> + Copy>(
//         &self,
//         by: Axies3d<I>,
//         curve: &impl MixerFxn<I>,
//     ) -> T {
//         let bf = by[Axis3d::Z].apply_mixer(curve);
//         let back = SIDE_CORNERS_3D[Side3d::Back]
//             .map(|c| self[c])
//             .interpolate_2d([by[Axis3d::X], by[Axis3d::Y]].into(), curve);
//         let front = SIDE_CORNERS_3D[Side3d::Front]
//             .map(|c| self[c])
//             .interpolate_2d([by[Axis3d::X], by[Axis3d::Y]].into(), curve);
//         T::lerp_dirty(back, front, bf)
//     }

//     /// performs an interpolation gradient within the cube formed by these corners  to the
//     /// coordinates in `by` according to the `curve`
//     #[inline(always)]
//     pub fn interpolate_gradient_3d<I: Mixable<T> + Copy>(
//         &self,
//         by: Axies3d<I>,
//         curve: &impl MixerFxn<I>,
//     ) -> Axies3d<T> {
//         let grads = EDGE_CORNERS_3D.map(|[c1, c2]| self[c1].lerp_gradient(self[c2]));
//         let axies = Axis3d::IDENTITY
//             .map(|a| SIDE_CORNERS_3D[axis3d_to_side3d(a)[0]].map(|c|
// grads[CORNER_EDGES_3D[c][a]]));         Axies3d([
//             axies[Axis3d::X].interpolate_2d([by[Axis3d::Y], by[Axis3d::Z]].into(), curve)
//                 * by[Axis3d::X].apply_mixer_derivative(curve),
//             axies[Axis3d::Y].interpolate_2d([by[Axis3d::X], by[Axis3d::Z]].into(), curve)
//                 * by[Axis3d::Y].apply_mixer_derivative(curve),
//             axies[Axis3d::Z].interpolate_2d([by[Axis3d::X], by[Axis3d::Y]].into(), curve)
//                 * by[Axis3d::Z].apply_mixer_derivative(curve),
//         ])
//     }

//     /// performs an interpolation and gradient within the cube formed by these corners  to the
//     /// coordinates in `by` according to the `curve`
//     #[inline(always)]
//     pub fn interpolate_and_gradient_3d<I: Mixable<T> + Copy>(
//         &self,
//         by: Axies3d<I>,
//         curve: &impl MixerFxn<I>,
//     ) -> (T, Axies3d<T>) {
//         (
//             self.interpolate_3d(by, curve),
//             self.interpolate_gradient_3d(by, curve),
//         )
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sides_and_axies() {
        for axis in Axis3d::IDENTITY {
            for side in axis3d_to_side3d(axis) {
                assert_eq!(axis, side3d_to_axis3d(side));
            }
        }
    }

    #[test]
    fn test_inversion() {
        use Corner3d::*;
        assert_eq!(
            Corner3d::IDENTITY.map(invert_corner3d),
            Corners3d([Ruf, Rub, Rdf, Rdb, Luf, Lub, Ldf, Ldb])
        );
        use Side3d::*;
        assert_eq!(
            Side3d::IDENTITY.map(invert_side3d),
            Sides3d([Right, Left, Up, Down, Front, Back])
        );
    }

    #[test]
    fn test_signs() {
        assert_eq!(
            Corner3d::IDENTITY.0.map(|c| corner3d_is_neg(c, Axis3d::X)),
            [true, true, true, true, false, false, false, false]
        );
        assert_eq!(
            Corner3d::IDENTITY.0.map(|c| corner3d_is_neg(c, Axis3d::Y)),
            [true, true, false, false, true, true, false, false]
        );
        assert_eq!(
            Corner3d::IDENTITY.0.map(|c| corner3d_is_neg(c, Axis3d::Z)),
            [true, false, true, false, true, false, true, false]
        );
        assert_eq!(
            Side3d::IDENTITY.0.map(side3d_is_neg),
            [true, false, true, false, true, false]
        );
    }

    #[test]
    fn test_side_surroundings_conversion() {
        use Surrounding3d::*;
        assert_eq!(
            Side3d::IDENTITY.0.map(Surrounding3d::from),
            [Nzz, Pzz, Znz, Zpz, Zzn, Zzp,]
        );
    }

    #[test]
    fn test_corner_separation() {
        use Corner3d::*;
        for c in Corner3d::IDENTITY {
            assert_eq!(corners3d_separation(c, c), 0);
            assert_eq!(corners3d_separation(invert_corner3d(c), c), 3);
        }
        assert_eq!(corners3d_separation(Rdf, Ldf), 1);
        assert_eq!(corners3d_separation(Rdf, Ruf), 1);
        assert_eq!(corners3d_separation(Rdf, Rdb), 1);
        assert_eq!(corners3d_separation(Rdf, Rub), 2);
        assert_eq!(corners3d_separation(Rdf, Ldb), 2);
        assert_eq!(corners3d_separation(Rdf, Luf), 2);
    }

    #[test]
    fn corner_from_signs() {
        for c in Corner3d::IDENTITY {
            let x = !corner3d_is_neg(c, Axis3d::X);
            let y = !corner3d_is_neg(c, Axis3d::Y);
            let z = !corner3d_is_neg(c, Axis3d::Z);
            let back = Corner3d::from_signs(BVec3::new(x, y, z));
            assert_eq!(c, back);
        }
    }
}
