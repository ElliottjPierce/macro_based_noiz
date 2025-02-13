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
        cellular::Cellular,
        grid::{
            GridNoise,
            GridPoint2,
        },
        interpolating::Cubic,
        merging::{
            EuclideanDistance,
            MergeWithoutSeed,
            MinOrder,
        },
        norm::UNorm,
        nudges::Nudge,
        seeded::{
            SeedOf,
            Seeding,
        },
        smoothing::Smooth,
        white::White32,
        worly::Worly,
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
    let noise = NoiseUsed::new(2345, 20.0);

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
        noise SeedOf = SeedOf,
        into UNorm
    }
}

noise_fn! {
    pub struct ValueNoise for Vec2 = (seed: u32, period: f32) {
        noise GridNoise = GridNoise::new_period(period),
        noise Smooth<Cubic, (GridPoint2, UVec2), White32, (u32, UNorm, f32)> = Smooth::new_vec2(Cubic, White32(seed)),
        into UNorm
    }
}

noise_fn! {
    pub struct WorlyNoise for Vec2 = (seed: u32, period: f32) {
        noise GridNoise = GridNoise::new_period(period),
        noise Worly<MergeWithoutSeed<MinOrder<EuclideanDistance>>> = Worly::new::<GridPoint2>(Cellular(Nudge::full()), seed),
    }
}
