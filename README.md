# Noiz
A fast and configurable proceedural noise library.

WARNING: This crate is still in development and is not at all production ready yet.

# About
Noiz is designed for game development in rust with [Bevy](https://bevyengine.org/).

### Goals
- Fast. Latency and slim memory footprint is most important.
- Configurable. Making noise of your own is easy.
- Integrated. Features integration with common crates.
- Contextualized. Noise functions and types can be chained together beyond what is possible in most crates.

### Philosophy
Noise interfaces are generic over noise functions, results, randomizers, etc. Noise is implemented for a variety of data types and dimensions. Noise Results can be easily chained together in interesting ways, which can be done automatically with FBM or manually (or some combination).

### Competition
If you just need fast, simple implementations of common noise, you should probably use [noise](https://crates.io/crates/noise). If you are using Bevy, want more or more configurable noise options, or want to easily make some noise functions for yourself, this is the crate for you!

### Dependencies
- We use a FxHash inspired random number generator.
- We use bevy_math (glam) for math
- We use [rand](https://crates.io/crates/rand) for RNG integration
