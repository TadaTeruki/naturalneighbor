use naturalneighbor::{Interpolator, Point};
use rand::Rng;

/// check the result of `interpolate` is same as the result of `query_weights`
#[test]
fn query_weights() {
    let mut rng: rand::rngs::StdRng = rand::SeedableRng::from_seed([0; 32]);
    let n = 10000;
    let bound = 1000.0;
    let points = (0..n)
        .map(|_| Point {
            x: rng.gen::<f64>() * bound,
            y: rng.gen::<f64>() * bound,
        })
        .collect::<Vec<_>>();

    let values = (0..n).map(|_| rng.gen::<f64>()).collect::<Vec<_>>();

    let interpolator = Interpolator::new(&points);

    let test_points = (0..n)
        .map(|_| Point {
            x: rng.gen::<f64>() * bound,
            y: rng.gen::<f64>() * bound,
        })
        .collect::<Vec<_>>();

    for i in 0..100 {
        let value1 = interpolator.interpolate(
            &values,
            Point {
                x: test_points[i].x,
                y: test_points[i].y,
            },
        );

        let queried_weights = interpolator.query_weights(Point {
            x: test_points[i].x,
            y: test_points[i].y,
        });

        if let Some(weights) = queried_weights {
            let value2 = weights.iter().map(|(i, w)| values[*i] * w).sum::<f64>();

            assert!(value1.is_some());

            assert!((value1.unwrap() - value2).abs() < 1e-6);
        } else {
            assert!(value1.is_none());
        }
    }
}
