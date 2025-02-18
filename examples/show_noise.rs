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
        associating::{
            MapValue,
            MetaOf,
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
    },
    noise_fn,
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

fn make_noise(image: &mut Image) {
    let width = image.width();
    let height = image.height();
    let noise = NoiseUsed::new(2345, 100.0);

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

noise_fn! {
    pub struct WhiteNoise for Vec2 = (seed: u32, period: f32) {
        noise GridNoise = GridNoise::new_period(period),
        noise Seeding = Seeding(seed),
        noise MetaOf = MetaOf,
        into UNorm
    }
}

noise_fn! {
    pub struct ValueNoise for Vec2 = (seed: u32, period: f32) {
        noise GridNoise = GridNoise::new_period(period),
        noise Lerp = Lerp,
        noise MapValue<Parallel<Seeding>> = MapValue(Parallel(Seeding(seed))),
        noise MapValue<Parallel<MetaOf>> = MapValue(Parallel(MetaOf)),
        noise MapValue<Parallel<Adapter<(u32, UNorm, f32), f32>>> = MapValue(Parallel(Adapter::new())),
        noise Smooth<Cubic> = Smooth(Cubic),
    }
}

noise_fn! {
    pub struct CellularNoise for Vec2 = (seed: u32, period: f32) {
        noise GridNoise = GridNoise::new_period(period),
        noise Voronoi<2, Cellular<ManhatanDistance>, true> = Voronoi::new(1.0.adapt(), seed, Cellular::default()),
        noise MetaOf = MetaOf,
        noise Adapter<(u32, UNorm, f32), f32> = Adapter::new(),
    }
}

noise_fn! {
    pub struct WorlyNoise for Vec2 = (seed: u32, period: f32) {
        noise GridNoise = GridNoise::new_period(period),
        noise Voronoi<2, Worly<EuclideanDistance>, false> = Voronoi::new(1.0, seed, Worly::shrunk_by(0.75).with_mode(WorlyMode::Ratio)),
        morph |input| -> UNorm {
            input.inverse()
        }
    }
}
