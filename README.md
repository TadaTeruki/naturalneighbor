# naturalneighbor

2D Natural Neighbor Interpolation (NNI) library for Rust.

The implementation of this library is based on '[A Fast and Accurate Algorithm for Natural Neighbor Interpolation](
https://gwlucastrig.github.io/TinfourDocs/NaturalNeighborTinfourAlgorithm/index.html)' by G.W. Lucas.

This is a subproject for the [fastlem](https://github.com/TadaTeruki/fastlem).

## Installation

```
[dependencies]
naturalneighbor = "1.1.3"
```

## Benchmark

Benchmarking is available with [criterion](https://crates.io/crates/criterion).
```
$ cargo bench
```

### Result

Estimated time per 1 interpolation for 2D NNI with 1000, 10000, and 100000 sites.

CPU: 11th Gen Intel i7-11390H (8) @ 5.000GHz<br>
Version: 1.1.3

||N=1000|N=10000|N=100000|
|:---|:---:|:---:|:---:|
|Estimated time|613.48 ns|932.48 ns|1.2171 µs|

## Usage

See the [API documentation](https://docs.rs/naturalneighbor) for details.

Some examples are provided in the `examples` directory which are useful for understanding how to use this library.

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

Please also open an issue or pull request if you have any problems or suggestions.

## License

MIT

Copyright (c) 2023 Teruki TADA
