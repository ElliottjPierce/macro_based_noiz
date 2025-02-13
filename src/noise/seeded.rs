//! Allows noise types to be given a seed.

use super::{
    NoiseOp,
    NoiseType,
};

/// Marks the type as being able to be given aseed. For example, grid points implement this so that
/// each cell in a grid can have a unique seed.
pub trait SeedableNoiseType: NoiseType {
    /// generates the seed for the group of noise values that this value is in. The seed is used to
    /// produce entropy.
    fn generate_seed(&self, seed: u32) -> u32;
}

/// Represents a type that has been given a seed for quick access.
pub struct Seeded<T: NoiseType> {
    /// the value
    pub value: T,
    /// the seed for the value
    pub seed: u32,
}

/// A noise operation that produces a [`Seeded`] version of any value that is passed into it,
/// provided it implements [`SeedableNoiseType`].
pub struct Seeding {
    /// the seed used to produce the seed in each value passed in.
    pub seed: u32,
}

impl<T: SeedableNoiseType> NoiseType for Seeded<T> {}

impl<T: SeedableNoiseType> NoiseOp<T> for Seeding {
    type Output = Seeded<T>;

    fn get(&self, input: T) -> Self::Output {
        Seeded {
            seed: input.generate_seed(self.seed),
            value: input,
        }
    }
}
