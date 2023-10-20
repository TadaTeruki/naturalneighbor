use rstar::{PointDistance, RTreeObject, AABB};

use crate::Point;

/// Triangle object to be inserted into an RTree
#[derive(Debug, Clone)]
pub(crate) struct Triangle {
    itriangle: usize,
    aabb: AABB<[f64; 2]>,
}

impl Triangle {
    pub fn itriangle(&self) -> usize {
        self.itriangle
    }

    pub fn from_triangle(points: &[Point], triangles: &[usize], t: usize) -> Self {
        let triangle = [triangles[t * 3], triangles[t * 3 + 1], triangles[t * 3 + 2]];

        let min_x = f64::min(
            f64::min(points[triangle[0]].x, points[triangle[1]].x),
            points[triangle[2]].x,
        );
        let min_y = f64::min(
            f64::min(points[triangle[0]].y, points[triangle[1]].y),
            points[triangle[2]].y,
        );
        let max_x = f64::max(
            f64::max(points[triangle[0]].x, points[triangle[1]].x),
            points[triangle[2]].x,
        );
        let max_y = f64::max(
            f64::max(points[triangle[0]].y, points[triangle[1]].y),
            points[triangle[2]].y,
        );

        Self {
            itriangle: t,
            aabb: AABB::from_corners([min_x, min_y], [max_x, max_y]),
        }
    }

    pub fn point_in_triangle(&self, points: &[Point], triangles: &[usize], point: &Point) -> bool {
        let triangle = [
            triangles[self.itriangle * 3],
            triangles[self.itriangle * 3 + 1],
            triangles[self.itriangle * 3 + 2],
        ];
        let p1 = &points[triangle[0]];
        let p2 = &points[triangle[1]];
        let p3 = &points[triangle[2]];

        let area2 = -p2.y * p3.x + p1.y * (-p2.x + p3.x) + p1.x * (p2.y - p3.y) + p2.x * p3.y;

        let s = 1.0 / area2
            * (p1.y * p3.x - p1.x * p3.y + (p3.y - p1.y) * point.x + (p1.x - p3.x) * point.y);
        let t = 1.0 / area2
            * (p1.x * p2.y - p1.y * p2.x + (p1.y - p2.y) * point.x + (p2.x - p1.x) * point.y);

        s >= 0.0 && t >= 0.0 && 1.0 - s - t >= 0.0
    }
}

impl RTreeObject for Triangle {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.aabb
    }
}

impl PointDistance for Triangle {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let min = self.aabb.min_point(point);
        (min[0] - point[0]).powi(2) + (min[1] - point[1]).powi(2)
    }
}
