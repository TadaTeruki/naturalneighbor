
use criterion::{criterion_group, criterion_main, Criterion};
use naturalneighbor::{Point, Interpolator};
use rand::Rng;

fn benchmark(c: &mut Criterion) {
    let mut rng: rand::rngs::StdRng = rand::SeedableRng::from_seed([0; 32]);
    let n = 1000;
    let points = (0..n)
        .map(|_| Point {
            x: rng.gen::<f64>() * 1000.0,
            y: rng.gen::<f64>() * 1000.0,
        })
        .collect::<Vec<_>>();

    let weights = (0..n)
        .map(|_| rng.gen::<f64>())
        .collect::<Vec<_>>();
    let interpolator = Interpolator::new(&points);

    c.bench_function("interpolate", |b| {
        b.iter(|| {
            for x in 0..100 {
                for y in 0..100 {
                    let _ = interpolator.interpolate(
                        &weights,
                        Point {
                            x: x as f64,
                            y: y as f64,
                        },
                    );
                }
            }
        })
    });

}

criterion_group!(benches, benchmark);
criterion_main!(benches);

