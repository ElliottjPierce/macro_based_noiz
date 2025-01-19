use noiz::{
    noise::{
        Noise,
        scalar::{
            SNorm,
            UNorm,
        },
        white::White32,
    },
    noise_fn,
    rng::NoiseRng,
};
use rand::prelude::*;

noise_fn! {
    /// white noise expressed as SNorms
    pub struct WhiteSNorm for u32 = (gen: &mut impl Rng) {
        noise White32 = White32(gen.next_u32()),
        into SNorm
    }
}

noise_fn! {
    /// white noise expressed as UNorms
    pub struct WhiteUNorm for u32 = (gen: &mut impl Rng) {
        noise White32 = White32(gen.next_u32()),
        into UNorm
    }
}

noise_fn! {
    /// white noise expressed as UNorms
    pub struct CrazyWhite for u32 = (gen: &mut impl Rng, key: u32) {
        noise White32 = White32(gen.next_u32()),
        noise White32 = White32(gen.next_u32()),
        morph |input| -> u32 {input.wrapping_mul(234085)},
        morph |input| {key: u32 = key} -> u32 {input.wrapping_mul(*key)},
        noise White32 = White32(gen.next_u32())
    }
}

fn main() {
    let mut seeds = NoiseRng::new_with(White32(9823475), 1024375);

    println!("SNorms");
    let noise = WhiteSNorm::new(&mut seeds);
    for i in 0..100 {
        let val = noise.sample(i);
        println!("\tSnorm: {val:?}");
    }

    println!("UNorms");
    let noise = WhiteUNorm::new(&mut seeds);
    for i in 0..100 {
        let val = noise.sample(i);
        println!("\tUnorm: {val:?}");
    }

    println!("Chaining white");
    let noise = CrazyWhite::new(&mut seeds, 389576);
    for i in 0..100 {
        let val = noise.sample(i);
        println!("\tChained white: {val:?}");
    }
}
