use naturalneighbor::{Interpolator, Point};
use rand::Rng;

#[test]
fn random_points() {
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

    let test_n = 100000;
    let test_points = (0..test_n)
        .map(|_| Point {
            x: rng.gen::<f64>() * bound,
            y: rng.gen::<f64>() * bound,
        })
        .collect::<Vec<_>>();

    for i in 0..test_n {
        let _ = interpolator.interpolate(
            &values,
            Point {
                x: test_points[i].x,
                y: test_points[i].y,
            },
        );
    }
}
