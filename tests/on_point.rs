use naturalneighbor::{Interpolator, Point};
use rand::Rng;

#[test]
fn on_point() {
    let mut rng: rand::rngs::StdRng = rand::SeedableRng::from_seed([0; 32]);
    let bound = 100;
    let points = (0..bound)
        .flat_map(|y| {
            (0..bound)
                .map(|x| Point {
                    x: x as f64,
                    y: y as f64,
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let values = (0..bound)
        .flat_map(|y| {
            (0..bound)
                .map(|x| (y * bound + x) as f64)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let interpolator = Interpolator::new(&points);

    let test_n = 10000;
    let test_points = (0..test_n)
        .map(|_| Point {
            x: rng.gen::<f64>() * (bound - 1) as f64,
            y: rng.gen::<f64>() * (bound - 1) as f64,
        })
        .collect::<Vec<_>>();

    for i in 0..test_n {
        let value = interpolator
            .interpolate(
                &values,
                Point {
                    x: test_points[i].x,
                    y: test_points[i].y,
                },
            )
            .unwrap_or_else(|e| {
                panic!(
                    "Failed to interpolate {:?} with error {:?}",
                    test_points[i], e
                )
            });
        if let Some(value) = value {
            let estimated = test_points[i].y * bound as f64 + test_points[i].x;
            assert!((value - estimated).abs() < 1e-8);
        } else {
            panic!("Failed to interpolate {:?}", test_points[i]);
        }
    }
}
