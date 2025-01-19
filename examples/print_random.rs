use noiz::noise::{
    NoiseOp,
    NoiseType,
    scalar::{
        SNorm,
        UNorm,
    },
    white::White32,
};

fn main() {
    println!("SNorms");
    for i in 0..100u32 {
        let val = White32(i).get(i).adapt::<UNorm>();
        println!("\tSnorm: {val:?}");
    }

    println!("SNorms");
    for i in 100..200u32 {
        let val = White32(i).get(i).adapt::<SNorm>();
        println!("\tSnorm: {val:?}");
    }

    println!("UNorms");
    for i in 200..300u32 {
        let val = White32(i).get(i);
        println!("\tUnorm: {val:?}");
    }

    println!("Chaining white");
    // let noise = chain!(White32(0), White32(1), White32(1), White32(1), White32(1));
    // for i in 200..300u32 {
    //     let val = noise.sample(i);
    //     println!("\tChained white: {val:?}");
    // }
}
