use crate::Point;

pub(crate) fn circumcenter(triangle: &[&Point; 3]) -> Point {
    let p1 = triangle[0];
    let p2 = triangle[1];
    let p3 = triangle[2];
    let d = 2.0 * (p1.x * (p2.y - p3.y) + p2.x * (p3.y - p1.y) + p3.x * (p1.y - p2.y));
    let ux = ((p1.x * p1.x + p1.y * p1.y) * (p2.y - p3.y)
        + (p2.x * p2.x + p2.y * p2.y) * (p3.y - p1.y)
        + (p3.x * p3.x + p3.y * p3.y) * (p1.y - p2.y))
        / d;
    let uy = ((p1.x * p1.x + p1.y * p1.y) * (p3.x - p2.x)
        + (p2.x * p2.x + p2.y * p2.y) * (p1.x - p3.x)
        + (p3.x * p3.x + p3.y * p3.y) * (p2.x - p1.x))
        / d;

    Point { x: ux, y: uy }
}

pub(crate) fn circumcircle_with_radius_2(triangle: &[&Point; 3]) -> (Point, f64) {
    let p1 = triangle[0];
    let circumcenter = circumcenter(triangle);
    let circumradius2 = (p1.x - circumcenter.x).powi(2) + (p1.y - circumcenter.y).powi(2);

    (circumcenter, circumradius2)
}

pub(crate) fn next_harfedge(e: usize) -> usize {
    if e % 3 == 2 {
        e - 2
    } else {
        e + 1
    }
}
