//! allows grid noise

use bevy_math::{
    DVec2,
    DVec3,
    DVec4,
    I64Vec2,
    I64Vec3,
    I64Vec4,
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

use super::Noise;
use crate::noise::mapping::MapExchange;

/// a noise that converts a vector input to a point in a grid
pub struct GridNoise {
    /// the frequency of the gridlines
    pub frequency: f32,
}

/// a noise that converts a vector input to a point in a grid
pub struct GridNoise64 {
    /// the frequency of the gridlines
    pub frequency: f64,
}

/// a noise that converts an integer vector input to a point in a grid
pub struct GridNoiseIntPow {
    /// grid lines will repeat every 2^x where x is this value
    pub period_power: u32,
}

/// a noise that converts an integer vector input to a point in a grid
pub struct GridNoiseInt {
    /// grid lines will repeat every x where x is this value
    pub period: u32,
}

/// easily creates grid points
macro_rules! make_grid_point {
    ($name:ident, $uint:ty, $int:ty, $f:ty, $fnoise:ty, $f2i:ident, $ui2f:ident, $s:ty, $i:ty) => {
        /// represents a point in a grid
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name {
            /// the least corner of the grid cell the point is in
            pub base: $uint,
            /// the UNorm offset from the [`base`](Self::base)
            pub offset: $f,
        }

        impl Noise<$f> for $fnoise {
            type Output = $name;

            fn sample(&self, input: $f) -> Self::Output {
                let val = input * self.frequency;
                $name {
                    base: MapExchange.sample(val.floor().$f2i()),
                    offset: val.fract_gl(),
                }
            }
        }

        impl Noise<$uint> for GridNoiseIntPow {
            type Output = $name;

            fn sample(&self, input: $uint) -> Self::Output {
                let base = input >> self.period_power;
                $name {
                    offset: (input - base).$ui2f() / 2u32.pow(self.period_power) as $s,
                    base,
                }
            }
        }

        impl Noise<$uint> for GridNoiseInt {
            type Output = $name;

            fn sample(&self, input: $uint) -> Self::Output {
                let base: $uint = input / self.period as $i;
                $name {
                    offset: (input - base).$ui2f() / self.period as $s,
                    base,
                }
            }
        }

        impl Noise<$int> for GridNoiseIntPow {
            type Output = $name;

            fn sample(&self, input: $int) -> Self::Output {
                self.sample(MapExchange.sample(input))
            }
        }

        impl Noise<$int> for GridNoiseInt {
            type Output = $name;

            fn sample(&self, input: $int) -> Self::Output {
                self.sample(MapExchange.sample(input))
            }
        }
    };
}

make_grid_point!(
    GridPoint2, UVec2, IVec2, Vec2, GridNoise, as_ivec2, as_vec2, f32, u32
);
make_grid_point!(
    GridPoint3, UVec3, IVec3, Vec3, GridNoise, as_ivec3, as_vec3, f32, u32
);
make_grid_point!(
    GridPoint4, UVec4, IVec4, Vec4, GridNoise, as_ivec4, as_vec4, f32, u32
);
make_grid_point!(
    GridPointD2,
    U64Vec2,
    I64Vec2,
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
    I64Vec3,
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
    I64Vec4,
    DVec4,
    GridNoise64,
    as_i64vec4,
    as_dvec4,
    f64,
    u64
);
