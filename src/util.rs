use crate::Point;
/*
fn point_in_triangle(points:&[Point], p: &Point, t: &[usize; 3]) -> bool {
    let p1 = &points[t[0]];
    let p2 = &points[t[1]];
    let p3 = &points[t[2]];

    let area = 0.5 * (-p2.y * p3.x + p1.y * (-p2.x + p3.x) + p1.x * (p2.y - p3.y) + p2.x * p3.y);

    let s = 1.0 / (2.0 * area) * (p1.y * p3.x - p1.x * p3.y + (p3.y - p1.y) * p.x + (p1.x - p3.x) * p.y);
    let t = 1.0 / (2.0 * area) * (p1.x * p2.y - p1.y * p2.x + (p1.y - p2.y) * p.x + (p2.x - p1.x) * p.y);

    s > 0.0 && t > 0.0 && 1.0 - s - t > 0.0
}*/

pub fn area_of_triangle(triangle: &[&Point; 3]) -> f64 {
    let p1 = &triangle[0];
    let p2 = &triangle[1];
    let p3 = &triangle[2];

    let area = 0.5 * (
        p1.x * (p2.y - p3.y) +
        p2.x * (p3.y - p1.y) +
        p3.x * (p1.y - p2.y)
    ).abs();

    area
}

pub fn circumcircle(triangle: &[&Point; 3]) -> (Point, f64) {
    let p1 = &triangle[0];
    let p2 = &triangle[1];
    let p3 = &triangle[2];

    let d = 2.0 * (p1.x * (p2.y - p3.y) + p2.x * (p3.y - p1.y) + p3.x * (p1.y - p2.y));
    let ux = ((p1.x * p1.x + p1.y * p1.y) * (p2.y - p3.y) + (p2.x * p2.x + p2.y * p2.y) * (p3.y - p1.y) + (p3.x * p3.x + p3.y * p3.y) * (p1.y - p2.y)) / d;
    let uy = ((p1.x * p1.x + p1.y * p1.y) * (p3.x - p2.x) + (p2.x * p2.x + p2.y * p2.y) * (p1.x - p3.x) + (p3.x * p3.x + p3.y * p3.y) * (p2.x - p1.x)) / d;

    let circumcenter = Point { x: ux, y: uy };
    let circumradius = ((p1.x - ux).powi(2) + (p1.y - uy).powi(2)).sqrt();

    (circumcenter, circumradius)
}