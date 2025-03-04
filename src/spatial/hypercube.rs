//! 4d orthogonal space utilities.

use bevy_math::IVec4;

use super::{
    cube::{
        expand3d,
        flatten3d,
    },
    interpolating::{
        Lerpable,
        MixerFxn,
    },
};
use crate::{
    name_array,
    spatial::cube::Corners3d,
};

name_array! {
    /// A 1 to 1 collection for the corners of a hypercube
    pub struct Corners4d,
    /// the corners of a hypercube
    pub enum Corner4d: u8, u16 {
        /// Left Down Back X
        Ldbx,
        /// Left Down Back Y
        Ldby,
        /// Left Down Front X
        Ldfx,
        /// Left Down Front Y
        Ldfy,
        /// Left Up Back X
        Lubx,
        /// Left Up Back Y
        Luby,
        /// Left Up Front X
        Lufx,
        /// Left Up Front Y
        Lufy,
        /// Right Down Back X
        Rdbx,
        /// Right Down Back Y
        Rdby,
        /// Right Down Front X
        Rdfx,
        /// Right Down Front Y
        Rdfy,
        /// Right Up Back X
        Rubx,
        /// Right Up Back Y
        Ruby,
        /// Right Up Front X
        Rufx,
        /// Right Up Front Y
        Rufy,
    }

    /// A 1 to 1 collection for the axies of 4d orthogonal space
    pub struct Axies4d,
    /// the axies of 4d orthogonal space
    pub enum Axis4d: u8, u8 {
        /// X
        X,
        /// Y
        Y,
        /// Z
        Z,
        /// W (The made up 4th one)
        W,
    }


    /// A 1 to 1 collection for the sides of a cube
    pub struct Sides4d,
    /// the sides of a cube
    pub enum Side4d: u8, u8 {
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
        /// negative w
        WNeg,
        /// positive w
        WPos,
    }

    /// A 1 to 1 collection for the surroundings of a center hypercube
    pub struct Surroundings4d,
    /// the Surroundings of a center hypercube
    pub enum Surrounding4d: u8, u128 {
        // sides
        Nzzz,
        Pzzz,
        Znzz,
        Zpzz,
        Zznz,
        Zzpz,
        Zzzn,
        Zzzp,
        // rest of x=z
        Zznn,
        Zzpp,
        Zznp,
        Zzpn,
        Znzn,
        Zpzp,
        Znzp,
        Zpzn,
        Znnz,
        Zppz,
        Znpz,
        Zpnz,
        Znnn,
        Zppn,
        Znpn,
        Zpnn,
        Znnp,
        Zppp,
        Znpp,
        Zpnp,
        // rest of x=n
        Nnzz,
        Npzz,
        Nznz,
        Nzpz,
        Nzzn,
        Nzzp,
        Nznn,
        Nzpp,
        Nznp,
        Nzpn,
        Nnzn,
        Npzp,
        Nnzp,
        Npzn,
        Nnnz,
        Nppz,
        Nnpz,
        Npnz,
        Nnnn,
        Nppn,
        Nnpn,
        Npnn,
        Nnnp,
        Nppp,
        Nnpp,
        Npnp,
        // rest of x=p
        Pnzz,
        Ppzz,
        Pznz,
        Pzpz,
        Pzzn,
        Pzzp,
        Pznn,
        Pzpp,
        Pznp,
        Pzpn,
        Pnzn,
        Ppzp,
        Pnzp,
        Ppzn,
        Pnnz,
        Pppz,
        Pnpz,
        Ppnz,
        Pnnn,
        Pppn,
        Pnpn,
        Ppnn,
        Pnnp,
        Pppp,
        Pnpp,
        Ppnp,
        // center
        Zzzz,
   }
}

/// The unit corners from 0 to 1
pub const UNIT_CORNERS_IVEC4: Corners4d<IVec4> = Corners4d([
    IVec4::new(0, 0, 0, 0),
    IVec4::new(0, 0, 0, 1),
    IVec4::new(0, 0, 1, 0),
    IVec4::new(0, 0, 1, 1),
    IVec4::new(0, 1, 0, 0),
    IVec4::new(0, 1, 0, 1),
    IVec4::new(0, 1, 1, 0),
    IVec4::new(0, 1, 1, 1),
    IVec4::new(1, 0, 0, 0),
    IVec4::new(1, 0, 0, 1),
    IVec4::new(1, 0, 1, 0),
    IVec4::new(1, 0, 1, 1),
    IVec4::new(1, 1, 0, 0),
    IVec4::new(1, 1, 0, 1),
    IVec4::new(1, 1, 1, 0),
    IVec4::new(1, 1, 1, 1),
]);

/// The unit side directions or normalized orthogonal length
pub const UNIT_SIDES_IVEC4: Sides4d<IVec4> = Sides4d([
    IVec4::new(-1, 0, 0, 0),
    IVec4::new(1, 0, 0, 0),
    IVec4::new(0, -1, 0, 0),
    IVec4::new(0, 1, 0, 0),
    IVec4::new(0, 0, -1, 0),
    IVec4::new(0, 0, 1, 0),
    IVec4::new(0, 0, 0, -1),
    IVec4::new(0, 0, 0, 1),
]);

/// The unit axies
pub const UNIT_AXIES_IVEC4: Axies4d<IVec4> = Axies4d([
    IVec4::new(1, 0, 0, 0),
    IVec4::new(0, 1, 0, 0),
    IVec4::new(0, 0, 1, 0),
    IVec4::new(0, 0, 0, 1),
]);

/// The unit Surroundings from -1 to 1
pub const UNIT_SURROUNDINGS_IVEC4: Surroundings4d<IVec4> = Surroundings4d([
    IVec4::new(-1, 0, 0, 0),
    IVec4::new(1, 0, 0, 0),
    IVec4::new(0, -1, 0, 0),
    IVec4::new(0, 1, 0, 0),
    IVec4::new(0, 0, -1, 0),
    IVec4::new(0, 0, 1, 0),
    IVec4::new(0, 0, 0, -1),
    IVec4::new(0, 0, 0, 1),
    IVec4::new(0, 0, -1, -1),
    IVec4::new(0, 0, 1, 1),
    IVec4::new(0, 0, -1, 1),
    IVec4::new(0, 0, 1, -1),
    IVec4::new(0, -1, 0, -1),
    IVec4::new(0, 1, 0, 1),
    IVec4::new(0, -1, 0, 1),
    IVec4::new(0, 1, 0, -1),
    IVec4::new(0, -1, -1, 0),
    IVec4::new(0, 1, 1, 0),
    IVec4::new(0, -1, 1, 0),
    IVec4::new(0, 1, -1, 0),
    IVec4::new(0, -1, -1, -1),
    IVec4::new(0, 1, 1, -1),
    IVec4::new(0, -1, 1, -1),
    IVec4::new(0, 1, -1, -1),
    IVec4::new(0, -1, -1, 1),
    IVec4::new(0, 1, 1, 1),
    IVec4::new(0, -1, 1, 1),
    IVec4::new(0, 1, -1, 1),
    IVec4::new(-1, -1, 0, 0),
    IVec4::new(-1, 1, 0, 0),
    IVec4::new(-1, 0, -1, 0),
    IVec4::new(-1, 0, 1, 0),
    IVec4::new(-1, 0, 0, -1),
    IVec4::new(-1, 0, 0, 1),
    IVec4::new(-1, 0, -1, -1),
    IVec4::new(-1, 0, 1, 1),
    IVec4::new(-1, 0, -1, 1),
    IVec4::new(-1, 0, 1, -1),
    IVec4::new(-1, -1, 0, -1),
    IVec4::new(-1, 1, 0, 1),
    IVec4::new(-1, -1, 0, 1),
    IVec4::new(-1, 1, 0, -1),
    IVec4::new(-1, -1, -1, 0),
    IVec4::new(-1, 1, 1, 0),
    IVec4::new(-1, -1, 1, 0),
    IVec4::new(-1, 1, -1, 0),
    IVec4::new(-1, -1, -1, -1),
    IVec4::new(-1, 1, 1, -1),
    IVec4::new(-1, -1, 1, -1),
    IVec4::new(-1, 1, -1, -1),
    IVec4::new(-1, -1, -1, 1),
    IVec4::new(-1, 1, 1, 1),
    IVec4::new(-1, -1, 1, 1),
    IVec4::new(-1, 1, -1, 1),
    IVec4::new(1, -1, 0, 0),
    IVec4::new(1, 1, 0, 0),
    IVec4::new(1, 0, -1, 0),
    IVec4::new(1, 0, 1, 0),
    IVec4::new(1, 0, 0, -1),
    IVec4::new(1, 0, 0, 1),
    IVec4::new(1, 0, -1, -1),
    IVec4::new(1, 0, 1, 1),
    IVec4::new(1, 0, -1, 1),
    IVec4::new(1, 0, 1, -1),
    IVec4::new(1, -1, 0, -1),
    IVec4::new(1, 1, 0, 1),
    IVec4::new(1, -1, 0, 1),
    IVec4::new(1, 1, 0, -1),
    IVec4::new(1, -1, -1, 0),
    IVec4::new(1, 1, 1, 0),
    IVec4::new(1, -1, 1, 0),
    IVec4::new(1, 1, -1, 0),
    IVec4::new(1, -1, -1, -1),
    IVec4::new(1, 1, 1, -1),
    IVec4::new(1, -1, 1, -1),
    IVec4::new(1, 1, -1, -1),
    IVec4::new(1, -1, -1, 1),
    IVec4::new(1, 1, 1, 1),
    IVec4::new(1, -1, 1, 1),
    IVec4::new(1, 1, -1, 1),
    IVec4::new(0, 0, 0, 0),
]);

/// Flatens a 4d index into a single value losslessly where L is the length of this 4d space.
/// Note that if the only goal is to fit a vector into a number, you may want to instead just merge
/// the bits together. This flattening is special because it keeps the values continuous. (adding
/// any power of `L` to a valid compression gives a position adjacent from the original).
/// See also: [`expand4d`]
#[inline]
pub const fn flatten4d<const L: usize>(x: usize, y: usize, z: usize, w: usize) -> usize {
    flatten3d::<L>(x, y, z) + w * L.pow(3)
}

/// expands a single index to its 4d coordinates where L is the length of this 4d space.
/// /// See also: [`flatten4d`]
#[inline]
pub const fn expand4d<const L: usize>(i: usize) -> (usize, usize, usize, usize) {
    let w = i / L.pow(3);
    let xyz = i - w * L.pow(3);
    let (x, y, z) = expand3d::<L>(xyz);
    (x, y, z, w)
}

impl<T: Lerpable + Copy> Corners4d<T> {
    /// performs an interpolation within the hypercube formed by these corners  to the coordinates
    /// in `by` according to the `curve`
    #[inline(always)]
    pub fn interpolate_4d<I: Copy>(&self, by: Axies4d<I>, curve: &impl MixerFxn<I, T>) -> T {
        use Axis4d::*;
        use Corner4d::*;
        let x = Corners3d([Ldbx, Ldfx, Lubx, Lufx, Rdbx, Rdfx, Rubx, Rufx]);
        let y = Corners3d([Ldby, Ldfy, Luby, Lufy, Rdby, Rdfy, Ruby, Rufy]);
        let x = x
            .map(|c| self[c])
            .interpolate_3d([by[X], by[Y], by[Z]].into(), curve);
        let y = y
            .map(|c| self[c])
            .interpolate_3d([by[X], by[Y], by[Z]].into(), curve);
        x.mix_dirty(y, by[W], curve)
    }

    /// performs an interpolation gradient within the hypercube formed by these corners  to the
    /// coordinates in `by` according to the `curve`
    #[inline(always)]
    pub fn interpolate_gradient_4d<I: Copy>(
        &self,
        by: Axies4d<I>,
        curve: &impl MixerFxn<I, T>,
    ) -> Axies4d<T> {
        use Axis4d::*;
        use Corner4d::*;
        Axies4d([
            Corners3d([
                self[Ldbx].lerp_gradient(self[Rdbx]),
                self[Ldby].lerp_gradient(self[Rdby]),
                self[Lubx].lerp_gradient(self[Rubx]),
                self[Luby].lerp_gradient(self[Ruby]),
                self[Ldfx].lerp_gradient(self[Rdfx]),
                self[Ldfy].lerp_gradient(self[Rdfy]),
                self[Lufx].lerp_gradient(self[Rufx]),
                self[Lufy].lerp_gradient(self[Rufy]),
            ])
            .interpolate_3d([by[W], by[Y], by[Z]].into(), curve)
                * curve.derivative(by[X]),
            Corners3d([
                self[Ldbx].lerp_gradient(self[Lubx]),
                self[Rdbx].lerp_gradient(self[Rubx]),
                self[Ldby].lerp_gradient(self[Luby]),
                self[Rdby].lerp_gradient(self[Ruby]),
                self[Ldfx].lerp_gradient(self[Lufx]),
                self[Rdfx].lerp_gradient(self[Rufx]),
                self[Ldfy].lerp_gradient(self[Lufy]),
                self[Rdfy].lerp_gradient(self[Rufy]),
            ])
            .interpolate_3d([by[X], by[W], by[Z]].into(), curve)
                * curve.derivative(by[Y]),
            Corners3d([
                self[Ldbx].lerp_gradient(self[Ldfx]),
                self[Rdbx].lerp_gradient(self[Rdfx]),
                self[Lubx].lerp_gradient(self[Lufx]),
                self[Rubx].lerp_gradient(self[Rufx]),
                self[Ldby].lerp_gradient(self[Ldfy]),
                self[Rdby].lerp_gradient(self[Rdfy]),
                self[Luby].lerp_gradient(self[Lufy]),
                self[Ruby].lerp_gradient(self[Rufy]),
            ])
            .interpolate_3d([by[X], by[Y], by[W]].into(), curve)
                * curve.derivative(by[Z]),
            Corners3d([
                self[Ldbx].lerp_gradient(self[Ldby]),
                self[Rdbx].lerp_gradient(self[Rdby]),
                self[Lubx].lerp_gradient(self[Luby]),
                self[Rubx].lerp_gradient(self[Ruby]),
                self[Ldfx].lerp_gradient(self[Ldfy]),
                self[Rdfx].lerp_gradient(self[Rdfy]),
                self[Lufx].lerp_gradient(self[Lufy]),
                self[Rufx].lerp_gradient(self[Rufy]),
            ])
            .interpolate_3d([by[X], by[Y], by[Z]].into(), curve)
                * curve.derivative(by[W]),
        ])
    }

    /// performs an interpolation and gradient within the hypercube formed by these corners  to the
    /// coordinates in `by` according to the `curve`
    #[inline(always)]
    pub fn interpolate_and_gradient_4d<I: Copy>(
        &self,
        by: Axies4d<I>,
        curve: &impl MixerFxn<I, T>,
    ) -> (T, Axies4d<T>) {
        (
            self.interpolate_4d(by, curve),
            self.interpolate_gradient_4d(by, curve),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expanding_and_flattening() {
        for test in [
            (8usize, 2usize, 5usize, 0usize),
            (1usize, 0usize, 3usize, 8usize),
            (4usize, 6usize, 0usize, 7usize),
            (5usize, 5usize, 5usize, 5usize),
        ] {
            let (x, y, z, w) = test;
            let compressed = flatten4d::<9>(x, y, z, w);
            let expanded = expand4d::<9>(compressed);
            assert_eq!(test, expanded);
            let compressed = flatten4d::<826>(x, y, z, w);
            let expanded = expand4d::<826>(compressed);
            assert_eq!(test, expanded);
            let compressed = flatten4d::<16>(x, y, z, w);
            let expanded = expand4d::<16>(compressed);
            assert_eq!(test, expanded);
            let compressed = flatten4d::<20>(x, y, z, w);
            let expanded = expand4d::<20>(compressed);
            assert_eq!(test, expanded);
        }
    }
}
