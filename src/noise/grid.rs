//! allows grid noise

use bevy_math::{
    DVec2,
    DVec3,
    DVec4,
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
    NoiseConvert,
    NoiseOp,
    NoiseType,
    norm::make_nonzero_f32,
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

/// a noise that converts an integer vector input to a point in a grid
#[derive(Debug, Clone, PartialEq)]
pub struct GridNoiseIntPow {
    /// grid lines will repeat every 2^x where x is this value
    pub period_power: u32,
}

/// a noise that converts an integer vector input to a point in a grid
#[derive(Debug, Clone, PartialEq)]
pub struct GridNoiseInt {
    /// grid lines will repeat every x where x is this value
    pub period: u32,
}

/// easily creates grid points
macro_rules! make_grid_point {
    ($name:ident, $uint:ty, $f:ty, $fnoise:ty, $f2i:ident, $ui2f:ident, $s:ty, $i:ty) => {
        /// represents a point in a grid
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name {
            /// the least corner of the grid cell the point is in
            pub base: $uint,
            /// the UNorm offset from the [`base`](Self::base)
            pub offset: $f,
        }

        impl NoiseType for $name {}

        impl NoiseConvert<$uint> for $name {
            fn convert(self) -> $uint {
                self.base
            }
        }

        impl NoiseConvert<$f> for $name {
            fn convert(self) -> $f {
                self.offset
            }
        }

        impl NoiseOp<$f> for $fnoise {
            type Output = $name;

            #[inline]
            fn get(&self, input: $f) -> Self::Output {
                let val = input * self.frequency;
                $name {
                    base: NoiseConvert::<$uint>::convert(val.floor().$f2i()),
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
    GridPoint2, UVec2, Vec2, GridNoise, as_ivec2, as_vec2, f32, u32
);
make_grid_point!(
    GridPoint3, UVec3, Vec3, GridNoise, as_ivec3, as_vec3, f32, u32
);
make_grid_point!(
    GridPoint4, UVec4, Vec4, GridNoise, as_ivec4, as_vec4, f32, u32
);
make_grid_point!(
    GridPointD2,
    U64Vec2,
    DVec2,
    GridNoise64,
    as_i64vec2,
    as_dvec2,
    f64,
    u64
);
make_grid_point!(
    GridPointD3,
    U64Vec3,
    DVec3,
    GridNoise64,
    as_i64vec3,
    as_dvec3,
    f64,
    u64
);
make_grid_point!(
    GridPointD4,
    U64Vec4,
    DVec4,
    GridNoise64,
    as_i64vec4,
    as_dvec4,
    f64,
    u64
);

impl NoiseConvert<GridPoint2> for GridPointD2 {
    fn convert(self) -> GridPoint2 {
        GridPoint2 {
            offset: self.offset.as_vec2(),
            base: self.base.as_uvec2(),
        }
    }
}

impl NoiseConvert<GridPointD2> for GridPoint2 {
    fn convert(self) -> GridPointD2 {
        GridPointD2 {
            offset: self.offset.as_dvec2(),
            base: self.base.as_u64vec2(),
        }
    }
}

impl NoiseConvert<GridPoint3> for GridPointD3 {
    fn convert(self) -> GridPoint3 {
        GridPoint3 {
            offset: self.offset.as_vec3(),
            base: self.base.as_uvec3(),
        }
    }
}

impl NoiseConvert<GridPointD3> for GridPoint3 {
    fn convert(self) -> GridPointD3 {
        GridPointD3 {
            offset: self.offset.as_dvec3(),
            base: self.base.as_u64vec3(),
        }
    }
}

impl NoiseConvert<GridPoint4> for GridPointD4 {
    fn convert(self) -> GridPoint4 {
        GridPoint4 {
            offset: self.offset.as_vec4(),
            base: self.base.as_uvec4(),
        }
    }
}

impl NoiseConvert<GridPointD4> for GridPoint4 {
    fn convert(self) -> GridPointD4 {
        GridPointD4 {
            offset: self.offset.as_dvec4(),
            base: self.base.as_u64vec4(),
        }
    }
}
