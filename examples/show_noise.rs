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
    associating::{
        Mapped,
        MetaOf,
        ValueOf,
    },
    conversions::Adapter,
    grid::GridNoise,
    interpolating::Cubic,
    merging::{
        EuclideanDistance,
        ManhatanDistance,
    },
    noise_op,
    norm::UNorm,
    parallel::Parallel,
    seeded::Seeding,
    smoothing::{
        Lerp,
        Smooth,
    },
    voronoi::{
        Cellular,
        Voronoi,
        Worly,
        WorlyMode,
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

type NoiseUsed = WorlyNoise;

pub struct TestingNoiseInput {
    pub seed: u32,
    pub period: f32,
}

fn make_noise(image: &mut Image) {
    let width = image.width();
    let height = image.height();
    let noise = NoiseUsed::new(TestingNoiseInput {
        seed: 9283740,
        period: 100.0,
    });

    for x in 0..width {
        for y in 0..height {
            let loc = Vec2::new(x as f32 - (x / 2) as f32, y as f32 - (y / 2) as f32);
            let out = noise.sample(loc).adapt::<f32>();
            let color = Color::linear_rgb(out, out, out);
            if let Err(err) = image.set_color_at(x, y, color) {
                warn!("Failed to set image color with error: {err:?}");
            }
        }
    }
}

noise_op! {
    pub struct WhiteNoise for Vec2 -> UNorm = TestingNoiseInput
    impl
    fn GridNoise = GridNoise::new_period(args.period);
    fn Seeding = Seeding(args.seed);
    fn MetaOf;
    as UNorm
}

noise_op! {
    pub struct ValueNoise for Vec2 -> UNorm = TestingNoiseInput
    impl
    fn GridNoise = GridNoise::new_period(args.period);
    fn Lerp = Lerp;
    mut ValueOf for fn Seeding = Seeding(args.seed);
    mut ValueOf for fn MetaOf;
    mut ValueOf for as u32, UNorm, f32;
    fn Smooth<Cubic> = Smooth(Cubic);
    as UNorm
}

noise_op! {
    pub struct CellularNoise for Vec2 -> UNorm = TestingNoiseInput
    impl
    fn GridNoise = GridNoise::new_period(args.period);
    fn Voronoi<2, Cellular<ManhatanDistance>, true> = Voronoi::new(1.0.adapt(), args.seed, Cellular::default());
    fn MetaOf;
    as UNorm
}

noise_op! {
    pub struct WorlyNoise for Vec2 -> UNorm = TestingNoiseInput
    impl
    fn GridNoise = GridNoise::new_period(args.period);
    fn Voronoi<2, Worly<EuclideanDistance>, false> = Voronoi::new(1.0, args.seed, Worly::shrunk_by(0.75).with_mode(WorlyMode::Ratio));
    || input.inverse();
}
