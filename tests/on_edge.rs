use naturalneighbor::{Interpolator, Point};
use rand::Rng;

// A macro for comparing floating point values.
macro_rules! assert_approx_eq {
    ($a:expr, $b:expr) => {
        assert!(($a - $b).abs() < 1e-6);
    };
}

#[test]
fn on_edge() {
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
        .map(|i| {
            if i % 2 == 0 {
                Point {
                    x: (rng.gen::<f64>() * ((bound - 3) as f64) + 1f64).floor() + 0.5,
                    y: (rng.gen::<f64>() * ((bound - 2) as f64) + 1f64).floor(),
                }
            } else {
                Point {
                    x: (rng.gen::<f64>() * ((bound - 3) as f64) + 1f64).floor(),
                    y: (rng.gen::<f64>() * ((bound - 2) as f64) + 1f64).floor() + 0.5,
                }
            }
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
            let estimated_floor =
                test_points[i].y.floor() * bound as f64 + test_points[i].x.floor();
            let estimated_ceil = test_points[i].y.ceil() * bound as f64 + test_points[i].x.ceil();
            let estimated = (estimated_ceil + estimated_floor) * 0.5;
            println!(
                "{:?}, {}, {}, {}",
                test_points[i],
                estimated,
                value,
                (value - estimated).abs()
            );
            assert_approx_eq!(value, estimated);
        } else {
            panic!("Failed to interpolate {:?}", test_points[i]);
        }
    }
}
