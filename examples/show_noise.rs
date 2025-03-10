use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{
        Extent3d,
        TextureDimension,
        TextureFormat,
    },
};
use noiz::{
    noise::{
        Noise,
        NoiseType,
        Period,
        SpatialNoiseSettings,
        associating::ValueOf,
        fbm::{
            OctaveSum,
            StandardFbm,
            StandardOctave,
            WeightedOctave,
        },
        grid::GridNoise,
        merging::{
            EuclideanDistance,
            ManhatanDistance,
        },
        noise_op,
        norm::{
            SNorm,
            UNorm,
        },
        perlin::{
            Perlin,
            RuntimeRand,
        },
        seeded::{
            SeedOf,
            Seeding,
        },
        smoothing::{
            Lerp,
            LerpValuesOf,
            Smooth,
        },
        voronoi::{
            Cellular,
            ExactDistanceToEdge,
            RelativeDistanceToEdge,
            Voronoi,
            Worly,
            worly_mode,
        },
    },
    spatial::interpolating::Cubic,
};

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(
            Startup,
            |mut commands: Commands, mut images: ResMut<Assets<Image>>| {
                let mut image = Image::new_fill(
                    Extent3d {
                        width: 1920,
                        height: 1080,
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    &[255, 255, 255, 255, 255, 255, 255, 255],
                    TextureFormat::Rgba16Unorm,
                    RenderAssetUsages::all(),
                );
                make_noise(&mut image);
                let handle = images.add(image);
                commands.spawn((
                    ImageNode {
                        image: handle,
                        ..Default::default()
                    },
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..Default::default()
                    },
                ));
                commands.spawn(Camera2d);
            },
        )
        .run()
}

type NoiseUsed = WarpedPerlinFbmNoise;

fn make_noise(image: &mut Image) {
    let width = image.width();
    let height = image.height();
    let noise = NoiseUsed::new(SpatialNoiseSettings::new(9202344, 100.0));

    for x in 0..width {
        for y in 0..height {
            let loc = Vec2::new(x as f32 - (x / 2) as f32, -(y as f32 - (y / 2) as f32));
            let out = noise.sample(loc).adapt::<f32>();
            let color = Color::linear_rgb(out, out, out);
            if let Err(err) = image.set_color_at(x, y, color) {
                warn!("Failed to set image color with error: {err:?}");
            }
        }
    }
}

noise_op! {
    pub struct WhiteNoise for Vec2 -> UNorm = SpatialNoiseSettings
    impl
    fn GridNoise = args.period.into();
    fn Seeding = args.seeding();
    fn SeedOf;
    as UNorm
}

noise_op! {
    pub struct ValueNoise for Vec2 -> UNorm = SpatialNoiseSettings
    impl
    fn GridNoise = args.period.into();
    fn Lerp;
    mut LerpValuesOf for fn Seeding = args.seeding();
    mut LerpValuesOf for fn SeedOf;
    mut LerpValuesOf for as UNorm, f32;
    fn Smooth<Cubic>;
    as UNorm
}

noise_op! {
    pub struct PerlinNoise for Vec2 -> UNorm = SpatialNoiseSettings
    impl
    fn GridNoise = args.period.into();
    fn Lerp;
    mut LerpValuesOf for fn Seeding = args.seeding();
    mut LerpValuesOf for mut ValueOf || input.offset;
    mut LerpValuesOf for fn Perlin<RuntimeRand>;
    fn Smooth<Cubic>;
    as SNorm, UNorm
}

noise_op! {
    pub struct Perlin3dNoise for Vec2 -> UNorm = SpatialNoiseSettings
    impl
    || input.extend(445.5);
    fn GridNoise = args.period.into();
    fn Lerp;
    mut LerpValuesOf for fn Seeding = args.seeding();
    mut LerpValuesOf for mut ValueOf || input.offset;
    mut LerpValuesOf for fn Perlin<RuntimeRand>;
    fn Smooth<Cubic>;
    as SNorm, UNorm
}

noise_op! {
    pub struct Value3dNoise for Vec2 -> UNorm = SpatialNoiseSettings
    impl
    || input.extend(72132.5);
    fn GridNoise = args.period.into();
    fn Lerp;
    mut LerpValuesOf for fn Seeding = args.seeding();
    mut LerpValuesOf for fn SeedOf;
    mut LerpValuesOf for as UNorm, f32;
    fn Smooth<Cubic>;
    as UNorm
}

noise_op! {
    pub struct CellularNoise for Vec2 -> UNorm = SpatialNoiseSettings
    impl
    fn GridNoise = args.period.into();
    fn Voronoi<2, Cellular<ManhatanDistance>, true> = Voronoi::new_default(1.0.adapt(), args.rand_32());
    fn SeedOf;
    as UNorm
}

noise_op! {
    pub struct DistanceToEdgeNoise for Vec2 -> UNorm = SpatialNoiseSettings
    impl
    fn GridNoise = args.period.into();
    fn Voronoi<2, RelativeDistanceToEdge, false> = Voronoi::new_default(1.0.adapt(), args.rand_32());
    as UNorm
}

noise_op! {
    pub struct ExactDistanceToEdgeNoise for Vec2 -> UNorm = SpatialNoiseSettings
    impl
    fn GridNoise = args.period.into();
    fn Voronoi<2, ExactDistanceToEdge, false> = Voronoi::new_default(1.0.adapt(), args.rand_32());
    as UNorm
}

noise_op! {
    pub struct WorlyNoise for Vec2 -> UNorm = SpatialNoiseSettings
    impl
    fn GridNoise = args.period.into();
    fn Voronoi<2, Worly<EuclideanDistance, worly_mode::Nearest>, false> = Voronoi::new(1.0, args.rand_32(), Worly::shrunk_by(1.0));
    || input.inverse();
}

noise_op! {
    pub struct PerlinFbmNoise for Vec2 -> UNorm = SpatialNoiseSettings
    impl
    loop OctaveSum where fbm = StandardFbm::new(args.period, 0.5, 0.6) enum [
        8 where octave: WeightedOctave as fbm.gen_octave::<StandardOctave>() impl {
            fn PerlinNoise = args.branch().with_period(octave).into();
        },
    ];
    as UNorm;
}

noise_op! {
    pub struct WarpedPerlinFbmNoise for Vec2 -> UNorm = SpatialNoiseSettings
    impl
    loop OctaveSum where fbm = StandardFbm::new(args.period, 0.5, 0.6) enum [
        8 where octave: WeightedOctave as fbm.gen_octave::<StandardOctave>() impl {
            // || {
            //     fbm.x += 1.0;
            //     input
            // };
            ref warp_x impl { loop OctaveSum where fbm = StandardFbm::new(octave, 0.5, 0.6) enum [
                2 where octave: WeightedOctave as fbm.gen_octave::<StandardOctave>() impl {
                    fn PerlinNoise = args.branch().with_period(octave).into();
                },
            ];};
            ref warp_y impl { loop OctaveSum where fbm = StandardFbm::new(octave, 0.5, 0.6) enum [
                2 where octave: WeightedOctave as fbm.gen_octave::<StandardOctave>() impl {
                    fn PerlinNoise = args.branch().with_period(octave).into();
                },
            ];};
            || {fbm += Vec2::new(warp_x, warp_y) * 10.0; fbm};
            fn PerlinNoise = args.branch().with_period(octave).into();
        },
    ];
    as UNorm;
}

noise_op! {
    pub struct SimpleHeightMapNoise for Vec2 -> UNorm = SpatialNoiseSettings
    impl
    ref mask impl loop OctaveSum where fbm = StandardFbm::new(args.period, 0.5, 0.6) enum [
        4 where octave: WeightedOctave as fbm.gen_octave::<StandardOctave>() impl {
            fn PerlinNoise = args.branch().with_period(octave).into();
        },
    ];
    loop OctaveSum where fbm = StandardFbm::new(args.period, 0.5, 0.6) enum [
        4 where octave: WeightedOctave as fbm.gen_octave::<StandardOctave>() impl {
            fn PerlinNoise = args.branch().with_period(octave).into();
        },
        4 where octave: WeightedOctave as fbm.gen_octave::<StandardOctave>() impl {
            fn ValueNoise = args.branch().with_period(octave).into()
        }
    ];
    || input * mask.powf(2.5);
    as UNorm
}
