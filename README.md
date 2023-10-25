# naturalneighbor

2D Natural Neighbor Interpolation (NNI) library for Rust.

The implementation of this library is based on '[A Fast and Accurate Algorithm for Natural Neighbor Interpolation](
https://gwlucastrig.github.io/TinfourDocs/NaturalNeighborTinfourAlgorithm/index.html)' by G.W. Lucas.

This libary is designed to be fast and memory efficient.
 - [delaunator](https://crates.io/crates/delaunator) for delaunay triangulation
 - [rstar](https://crates.io/crates/rstar) for spatial indexing
 - Dynamic memory allocation is never used in the calculation

*TODO: Add benchmark results*

This is a subproject for the [terrain-rs](https://github.com/TadaTeruki/terrain-rs).

## Installation

```
[dependencies]
naturalneighbor = "1.1"
```

## Usage

See [API documentation](https://docs.rs/naturalneighbor) for details.

Some examples are provided which are useful to understand how to use this library in the `examples` directory. 

## Preview

```
$ cargo run --example color
```

![color](https://github.com/TadaTeruki/naturalneighbor/assets/69315285/0b8f7bc6-a15f-470b-bad3-7852eee55dcd)

## Dependencies

 - [rstar](https://crates.io/crates/rstar)
 - [delaunator](https://crates.io/crates/delaunator)

## Contributing

Contributions are welcome. 

The author is not a native English speaker. Please let me know if you find any grammatical errors in the documentation.

Not only for that, please open an issue or a pull request if you have any problems or improvements.

## License

MIT

Copyright (c) 2023 Teruki TADA
