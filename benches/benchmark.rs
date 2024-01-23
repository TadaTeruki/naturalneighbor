use criterion::{criterion_group, criterion_main, Criterion};
use naturalneighbor::{Interpolator, Point};
use rand::Rng;

fn benchmark(c: &mut Criterion) {
    let mut rng: rand::rngs::StdRng = rand::SeedableRng::from_seed([0; 32]);
    let n = 100000;
    let bound = 1000.0;
    let points = (0..n)
        .map(|_| Point {
            x: rng.gen::<f64>() * bound,
            y: rng.gen::<f64>() * bound,
        })
        .collect::<Vec<_>>();

    let weights = (0..n).map(|_| rng.gen::<f64>()).collect::<Vec<_>>();
    let interpolator = Interpolator::new(&points);

    c.bench_function("interpolate", |b| {
        b.iter(|| {
            let _ = interpolator.interpolate(
                &weights,
                Point {
                    x: bound / 2.0,
                    y: bound / 2.0,
                },
            );
        })
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
