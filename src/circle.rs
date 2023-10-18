use rstar::{RTreeObject, AABB, PointDistance};

use crate::Point;

#[derive(Debug, Clone)]
pub(crate) struct CircumCircle {
    itriangle: usize,
    origin: Point,
    radius: f64,
}

impl CircumCircle {
    pub fn itriangle(&self) -> usize {
        self.itriangle
    }

    pub fn origin(&self) -> &Point {
        &self.origin
    }

    pub fn radius(&self) -> f64 {
        self.radius
    }

    pub fn from_triangle(points: &[Point], triangles: &[usize], t: usize) -> Self {
        let triangle = [triangles[t * 3], triangles[t * 3 + 1], triangles[t * 3 + 2]];

        let (x1, y1) = (points[triangle[0]].x, points[triangle[0]].y);
        let (x2, y2) = (points[triangle[1]].x, points[triangle[1]].y);
        let (x3, y3) = (points[triangle[2]].x, points[triangle[2]].y);
        
        let d = 2.0 * (x1 * (y2 - y3) + x2 * (y3 - y1) + x3 * (y1 - y2));
        
        let ux = ((x1*x1 + y1*y1) * (y2 - y3) + (x2*x2 + y2*y2) * (y3 - y1) + (x3*x3 + y3*y3) * (y1 - y2)) / d;
        let uy = ((x1*x1 + y1*y1) * (x3 - x2) + (x2*x2 + y2*y2) * (x1 - x3) + (x3*x3 + y3*y3) * (x2 - x1)) / d;
        
        let origin = Point { x: ux, y: uy };
        
        let radius = ((origin.x - x1).powi(2) + (origin.y - y1).powi(2)).sqrt();
        
        Self {
            itriangle: t,
            origin,
            radius,
        }
    }

    pub fn point_in_triangle(&self, points: &[Point], triangles: &[usize], point: &Point) -> bool {
        let triangle = [triangles[self.itriangle * 3], triangles[self.itriangle * 3 + 1], triangles[self.itriangle * 3 + 2]];
        let p1 = &points[triangle[0]];
        let p2 = &points[triangle[1]];
        let p3 = &points[triangle[2]];
    
        let area = 0.5 * (-p2.y * p3.x + p1.y * (-p2.x + p3.x) + p1.x * (p2.y - p3.y) + p2.x * p3.y);
    
        let s = 1.0 / (2.0 * area) * (p1.y * p3.x - p1.x * p3.y + (p3.y - p1.y) * point.x + (p1.x - p3.x) * point.y);
        let t = 1.0 / (2.0 * area) * (p1.x * p2.y - p1.y * p2.x + (p1.y - p2.y) * point.x + (p2.x - p1.x) * point.y);
    
        s > 0.0 && t > 0.0 && 1.0 - s - t > 0.0
    }

    pub fn point_in_circle(&self, point: &Point) -> bool {
        let d_x = self.origin.x - point.x;
        let d_y = self.origin.y - point.y;
        let distance_to_origin_2 = d_x * d_x + d_y * d_y;
        let radius_2 = self.radius * self.radius;
        distance_to_origin_2 <= radius_2
    }
}

impl RTreeObject for CircumCircle {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let corner_1 = [self.origin.x - self.radius, self.origin.y - self.radius];
        let corner_2 = [self.origin.x + self.radius, self.origin.y + self.radius];
        AABB::from_corners(corner_1, corner_2)
    }
}

impl PointDistance for CircumCircle
{
    fn distance_2(&self, point: &[f64; 2]) -> f64
    {
        let d_x = self.origin.x - point[0];
        let d_y = self.origin.y - point[1];
        let distance_to_origin = (d_x * d_x + d_y * d_y).sqrt();
        let distance_to_ring = distance_to_origin - self.radius;
        let distance_to_circle = f64::max(0.0, distance_to_ring);
        // We must return the squared distance!
        distance_to_circle * distance_to_circle
    }
    /* 
    fn contains_point(&self, point: &[f64; 2]) -> bool
    {
        let d_x = self.origin.x - point[0];
        let d_y = self.origin.y - point[1];
        let distance_to_origin_2 = d_x * d_x + d_y * d_y;
        let radius_2 = self.radius * self.radius;
        distance_to_origin_2 <= radius_2
    }
    */
}