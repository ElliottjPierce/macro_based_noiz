//! allows grid noise

use bevy_math::{
    DVec2,
    DVec3,
    DVec4,
    IVec2,
    IVec3,
    IVec4,
    U64Vec2,
    U64Vec3,
    U64Vec4,
    UVec2,
    UVec3,
    UVec4,
    Vec2,
    Vec3,
    Vec4,
};

use super::{
    NoiseOp,
    NoiseType,
    Period,
    associating::Associated,
    conversions::convertible,
    norm::make_nonzero_f32,
    seeded::SeedableNoiseType,
    smoothing::LerpLocatable,
};
use crate::spatial::{
    cube::{
        Axies3d,
        Corners3d,
        Surroundings3d,
        UNIT_CORNERS_IVEC3,
        UNIT_SURROUNDINGS_IVEC3,
    },
    hypercube::{
        Axies4d,
        Corners4d,
        Surroundings4d,
        UNIT_CORNERS_IVEC4,
        UNIT_SURROUNDINGS_IVEC4,
    },
    square::{
        Axies2d,
        Corners2d,
        Surroundings2d,
        UNIT_CORNERS_IVEC2,
        UNIT_SURROUNDINGS_IVEC2,
    },
};

/// a noise that converts a vector input to a point in a grid
#[derive(Debug, Clone, PartialEq)]
pub struct GridNoise {
    /// the frequency of the gridlines
    pub frequency: f32,
}

impl GridNoise {
    /// constructs a new [`GridNoise`] of this frequency
    pub fn new_frequency(frequency: f32) -> Self {
        Self { frequency }
    }

    /// constructs a new [`GridNoise`] of this period
    pub fn new_period(period: f32) -> Self {
        Self::new_frequency(1.0 / make_nonzero_f32(period))
    }
}

impl From<Period> for GridNoise {
    fn from(value: Period) -> Self {
        Self::new_period(value.0)
    }
}

/// a noise that converts a vector input to a point in a grid
#[derive(Debug, Clone, PartialEq)]
pub struct GridNoise64 {
    /// the frequency of the gridlines
    pub frequency: f64,
}

impl GridNoise64 {
    /// constructs a new [`GridNoise64`] of this frequency
    pub fn new_frequency(frequency: f64) -> Self {
        Self { frequency }
    }

    /// constructs a new [`GridNoise64`] of this period
    pub fn new_period(period: f64) -> Self {
        Self::new_frequency(1.0 / period)
    }
}

impl From<Period> for GridNoise64 {
    fn from(value: Period) -> Self {
        Self::new_period(value.0 as f64)
    }
}

/// a noise that converts an integer vector input to a point in a grid
#[derive(Debug, Clone, PartialEq)]
pub struct GridNoiseIntPow {
    /// grid lines will repeat every 2^x where x is this value
    pub period_power: u32,
}

impl From<Period> for GridNoiseIntPow {
    fn from(value: Period) -> Self {
        let int = GridNoiseInt::from(value).period;
        Self {
            period_power: int.ilog2() + 1,
        }
    }
}

/// a noise that converts an integer vector input to a point in a grid
#[derive(Debug, Clone, PartialEq)]
pub struct GridNoiseInt {
    /// grid lines will repeat every x where x is this value
    pub period: u32,
}

impl From<Period> for GridNoiseInt {
    fn from(value: Period) -> Self {
        Self {
            period: value.0.abs().ceil() as u32,
        }
    }
}

/// easily creates grid points
macro_rules! make_grid_point {
    (
        $name:ident,
        $uint:ty,
        $f:ty,
        $fnoise:ty,
        $f2i:ident,
        $ui2f:ident,
        $s:ty,
        $i:ty,
        $d:ident,
        $axies:ident,
        $num_d:literal
    ) => {
        make_grid_point!($name, $uint, $f, $fnoise, $f2i, $ui2f, $s, $i, $d, $num_d);

        impl LerpLocatable for $name {
            type Location = $axies<$s>;

            type Extents = $d<Self>;

            #[inline]
            fn prepare_lerp(self) -> Associated<Self::Extents, Self::Location> {
                Associated {
                    value: self.corners(),
                    meta: self.offset.to_array().into(),
                }
            }
        }
    };

    (
        $name:ident,
        $uint:ty,
        $f:ty,
        $fnoise:ty,
        $f2i:ident,
        $ui2f:ident,
        $s:ty,
        $i:ty,
        $d:ident,
        $num_d:literal
    ) => {
        /// represents a point in a grid
        #[derive(Debug, Default, Clone, PartialEq)]
        pub struct $name {
            /// the corner of the grid cell we are anchored to
            pub base: $uint,
            /// the offset from the [`base`](Self::base)
            pub offset: $f,
        }

        impl $name {
            /// pushes the grid point by this offset
            #[inline]
            pub fn pushed(&self, push: $uint) -> Self {
                Self {
                    base: self.base + push,
                    offset: self.offset - push.$ui2f(),
                }
            }
        }

        impl NoiseType for $name {}

        impl SeedableNoiseType for $name {
            #[inline]
            fn generate_seed(&self, seed: u32) -> u32 {
                self.base.generate_seed(seed)
            }
        }

        convertible!($name = $uint, |source| source.base);
        convertible!($name = $f, |source| source.offset);

        impl NoiseOp<$f> for $fnoise {
            type Output = $name;

            #[inline]
            fn get(&self, input: $f) -> Self::Output {
                let val = input * self.frequency;
                $name {
                    base: val.floor().$f2i().adapt::<$uint>(),
                    offset: val.fract_gl(),
                }
            }
        }

        impl NoiseOp<$uint> for GridNoiseIntPow {
            type Output = $name;

            #[inline]
            fn get(&self, input: $uint) -> Self::Output {
                let base = input >> self.period_power;
                $name {
                    offset: (input - base).$ui2f() / 2u32.pow(self.period_power) as $s,
                    base,
                }
            }
        }

        impl NoiseOp<$uint> for GridNoiseInt {
            type Output = $name;

            #[inline]
            fn get(&self, input: $uint) -> Self::Output {
                let base: $uint = input / self.period as $i;
                $name {
                    offset: (input - base).$ui2f() / self.period as $s,
                    base,
                }
            }
        }
    };
}

make_grid_point!(
    GridPoint2, UVec2, Vec2, GridNoise, as_ivec2, as_vec2, f32, u32, Corners2d, Axies2d, 2
);
make_grid_point!(
    GridPoint3, UVec3, Vec3, GridNoise, as_ivec3, as_vec3, f32, u32, Corners3d, Axies3d, 3
);
make_grid_point!(
    GridPoint4, UVec4, Vec4, GridNoise, as_ivec4, as_vec4, f32, u32, Corners4d, Axies4d, 4
);
make_grid_point!(
    GridPointD2,
    U64Vec2,
    DVec2,
    GridNoise64,
    as_i64vec2,
    as_dvec2,
    f64,
    u64,
    Corners2d,
    2
);
make_grid_point!(
    GridPointD3,
    U64Vec3,
    DVec3,
    GridNoise64,
    as_i64vec3,
    as_dvec3,
    f64,
    u64,
    Corners3d,
    3
);
make_grid_point!(
    GridPointD4,
    U64Vec4,
    DVec4,
    GridNoise64,
    as_i64vec4,
    as_dvec4,
    f64,
    u64,
    Corners4d,
    4
);

convertible!(GridPointD2 = GridPoint2, |source| GridPoint2 {
    offset: source.offset.as_vec2(),
    base: source.base.as_uvec2(),
});
convertible!(GridPointD3 = GridPoint3, |source| GridPoint3 {
    offset: source.offset.as_vec3(),
    base: source.base.as_uvec3(),
});
convertible!(GridPointD4 = GridPoint4, |source| GridPoint4 {
    offset: source.offset.as_vec4(),
    base: source.base.as_uvec4(),
});

convertible!(GridPoint2 = GridPointD2, |source| GridPointD2 {
    offset: source.offset.as_dvec2(),
    base: source.base.as_u64vec2(),
});
convertible!(GridPoint3 = GridPointD3, |source| GridPointD3 {
    offset: source.offset.as_dvec3(),
    base: source.base.as_u64vec3(),
});
convertible!(GridPoint4 = GridPointD4, |source| GridPointD4 {
    offset: source.offset.as_dvec4(),
    base: source.base.as_u64vec4(),
});

impl GridPoint2 {
    /// Produces an array of all positive unit offset combinations from the current value.
    #[inline]
    pub fn corners(&self) -> Corners2d<Self> {
        UNIT_CORNERS_IVEC2.map(|d| self.pushed(d.as_uvec2()))
    }

    /// Produces an array of all unit offset combinations from the current value.
    #[inline]
    pub fn surroundings(&self) -> Surroundings2d<Self> {
        let minus_corner = {
            Self {
                base: self.base - UVec2::ONE,
                offset: self.offset + Vec2::ONE,
            }
        };
        UNIT_SURROUNDINGS_IVEC2.map(|d| minus_corner.pushed((d + IVec2::ONE).as_uvec2()))
    }
}

impl GridPoint3 {
    /// Produces an array of all positive unit offset combinations from the current value.
    #[inline]
    pub fn corners(&self) -> Corners3d<Self> {
        UNIT_CORNERS_IVEC3.map(|d| self.pushed(d.as_uvec3()))
    }

    /// Produces an array of all unit offset combinations from the current value.
    #[inline]
    pub fn surroundings(&self) -> Surroundings3d<Self> {
        let minus_corner = {
            Self {
                base: self.base - UVec3::ONE,
                offset: self.offset + Vec3::ONE,
            }
        };
        UNIT_SURROUNDINGS_IVEC3.map(|d| minus_corner.pushed((d + IVec3::ONE).as_uvec3()))
    }
}

impl GridPoint4 {
    /// Produces an array of all positive unit offset combinations from the current value.
    #[inline]
    pub fn corners(&self) -> Corners4d<Self> {
        UNIT_CORNERS_IVEC4.map(|d| self.pushed(d.as_uvec4()))
    }

    /// Produces an array of all unit offset combinations from the current value.
    #[inline]
    pub fn surroundings(&self) -> Surroundings4d<Self> {
        let minus_corner = {
            Self {
                base: self.base - UVec4::ONE,
                offset: self.offset + Vec4::ONE,
            }
        };
        UNIT_SURROUNDINGS_IVEC4.map(|d| minus_corner.pushed((d + IVec4::ONE).as_uvec4()))
    }
}
