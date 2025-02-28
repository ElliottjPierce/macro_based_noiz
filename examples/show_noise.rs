use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{
        Extent3d,
        TextureDimension,
        TextureFormat,
    },
};
use noiz::noise::{
    Noise,
    NoiseType,
    associating::ValueOf,
    fbm::{
        SpatialFbmSettings,
        SpatialNoiseSettings,
    },
    grid::GridNoise,
    interpolating::Cubic,
    merging::{
        EuclideanDistance,
        ManhatanDistance,
        Merged,
        Total,
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
        Voronoi,
        Worly,
        worly_mode,
    },
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

type NoiseUsed = CustomNoise;

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
    fn GridNoise = GridNoise::new_period(args.period);
    fn Seeding = args.seeding();
    fn SeedOf;
    as UNorm
}

noise_op! {
    pub struct ValueNoise for Vec2 -> UNorm = SpatialNoiseSettings
    impl
    fn GridNoise = GridNoise::new_period(args.period);
    fn Lerp = Lerp;
    mut LerpValuesOf for fn Seeding = args.seeding();
    mut LerpValuesOf for fn SeedOf;
    mut LerpValuesOf for as UNorm, f32;
    fn Smooth<Cubic>;
    as UNorm
}

noise_op! {
    pub struct PerlinNoise for Vec2 -> UNorm = SpatialNoiseSettings
    impl
    fn GridNoise = GridNoise::new_period(args.period);
    fn Lerp = Lerp;
    mut LerpValuesOf for fn Seeding = args.seeding();
    mut LerpValuesOf for mut ValueOf || input.offset;
    mut LerpValuesOf for fn Perlin<RuntimeRand>;
    fn Smooth<Cubic>;
    as SNorm, UNorm
}

noise_op! {
    pub struct CellularNoise for Vec2 -> UNorm = SpatialNoiseSettings
    impl
    fn GridNoise = GridNoise::new_period(args.period);
    fn Voronoi<2, Cellular<ManhatanDistance>, true> = Voronoi::new_default(1.0.adapt(), args.rand_32());
    fn SeedOf;
    as UNorm
}

noise_op! {
    pub struct WorlyNoise for Vec2 -> UNorm = SpatialNoiseSettings
    impl
    fn GridNoise = GridNoise::new_period(args.period);
    fn Voronoi<2, Worly<EuclideanDistance, worly_mode::Ratio>, false> = Voronoi::new(1.0, args.rand_32(), Worly::shrunk_by(0.75));
    || input.inverse();
}

noise_op! {
    pub struct PerlinFbmNoise for Vec2 -> UNorm = SpatialNoiseSettings
    impl
    loop &SpatialFbmSettings::from_spatial(&mut args, 0.5, 0.3) enum [8 PerlinNoise];
    for as f32;
    fn Merged<Total>;
    as UNorm;
}

noise_op! {
    pub struct TestLambda for Vec2 -> UNorm = SpatialNoiseSettings
    impl
    fn type Vec2 -> UNorm = SpatialNoiseSettings impl {
        loop &SpatialFbmSettings::from_spatial(&mut args, 0.5, 0.3) enum [8 PerlinNoise];
        for as f32;
        fn Merged<Total>;
        as UNorm;
    } = args.branch().into();
}

noise_op! {
    pub struct CustomNoise for Vec2 -> UNorm = SpatialNoiseSettings
    impl
    ref worly_res impl {
        fn WorlyNoise = WorlyNoise::from(args.branch());
    };
    loop &SpatialFbmSettings::from_spatial(&mut args, 0.8, 0.7) enum [
        3 PerlinNoise,
        WorlyNoise,
        CellularNoise,
        5 type Vec2 -> UNorm = SpatialNoiseSettings impl {
            fn GridNoise = GridNoise::new_period(args.period);
            fn Lerp = Lerp;
            mut LerpValuesOf for fn Seeding = args.seeding();
            mut LerpValuesOf for fn SeedOf;
            mut LerpValuesOf for as UNorm, f32;
            fn Smooth<Cubic>;
            as UNorm;
        },
    ];
    for as f32;
    fn Merged<Total>;
    || input * worly_res.adapt::<f32>();
    as UNorm;
}
