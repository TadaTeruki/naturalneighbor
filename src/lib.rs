use circle::CircumCircle;
use util::{circumcircle, area_of_triangle};

mod util;
mod circle;
pub type Point = delaunator::Point;

pub struct InterpolatorBuilder<'a, V> where V: Copy {
    points: Option<&'a [Point]>,
    items: Option<&'a [V]>,
}

impl<'a, V> InterpolatorBuilder<'a, V> where V: Copy {
    pub fn new() -> Self {
        Self {
            points: None,
            items: None,
        }
    }

    pub fn set_points(&self, points: &'a [Point]) -> Self {
        Self {
            points: Some(points),
            ..*self
        }
    }

    pub fn set_items(&self, items: &'a [V]) -> Self {
        Self {
            items: Some(items),
            ..*self
        }
    }

    pub fn build(&self) -> Option<Interpolator<'a, V>> where V: Copy {
        if let (Some(points), Some(items)) = (self.points, self.items) {
            if points.len() != items.len() {
                return None;
            }

            let triangulation = delaunator::triangulate(points);
            
            let circumcircles = triangulation.triangles.chunks_exact(3).enumerate().map(|(t, _)| {
                CircumCircle::from_triangle(&points, &triangulation.triangles, t)
                //CircumCircle::from_triangle(&points, [triangle[0], triangle[1], triangle[2]])
            }).collect::<Vec<_>>();
            

            let rtree = rstar::RTree::bulk_load(circumcircles);

            let interpolator = Interpolator {
                points,
                items,
                triangles: triangulation.triangles,
                harfedges: triangulation.halfedges,
                tree: rtree,
            };

            return Some(interpolator);
        };

        None
    }
}

pub struct Interpolator<'a, V> where V: Copy {
    points: &'a [Point],
    items: &'a [V],
    triangles: Vec<usize>,
    harfedges: Vec<usize>,
    tree: rstar::RTree<CircumCircle>,
}

impl<'a, V> Interpolator<'a, V> where V: Copy {
    pub fn interpolate(&self, p: Point, add: impl Fn(&V, &V) -> V, mul: impl Fn(&V, f64) -> V) -> Option<V> {
        let triangles = self.tree.locate_all_at_point(&[p.x, p.y]).filter(|circle| {
            circle.point_in_triangle(&self.points, &self.triangles, &p)
        }).collect::<Vec<_>>();

        let it: usize = {
            if let Some(t) = triangles.get(0) {
                t.itriangle()
            } else {
                return None;
            }
        };

        let next_harfedge = |e: usize| -> usize {
            if e % 3 == 2 {
                e - 2
            } else {
                e + 1
            }
        };

        let prev_harfedge = |e: usize| -> usize {
            if e % 3 == 0 {
                e + 2
            } else {
                e - 1
            }
        };

        // start point of each harfedges
        let starts = [self.triangles[it * 3], self.triangles[it * 3 + 1], self.triangles[it * 3 + 2]];
        // circumcenter of the triangle
        let c2 = circumcircle(&[&self.points[starts[0]], &self.points[starts[1]], &self.points[starts[2]]]).0;
        
        // start point of opposite harfedges
        let start_opposites = [self.harfedges[it*3], self.harfedges[it * 3 + 1], self.harfedges[it * 3 + 2]];
        let opposite_triangles = [start_opposites[0]/3, start_opposites[1]/3, start_opposites[2]/3];

        let envelope = start_opposites.iter().enumerate().map(|(i, start_opposite)| {
            if *start_opposite >= self.harfedges.len() {
                return vec![];
            }

            let next = next_harfedge(*start_opposite);
            let prev = prev_harfedge(*start_opposite);
            // 0: index of the target point, 1: order of the opposite triangle
            vec![(self.triangles[next], opposite_triangles[i]), (self.triangles[prev], opposite_triangles[i])]
        }).flatten().collect::<Vec<_>>();

        let envelope_points = envelope.iter().map(|e| self.triangles[e.0]).collect::<Vec<_>>();

        let areas = envelope.iter().enumerate().map(|(i, envl)| {
            let ibp = envl.0;
            let bp = &self.points[ibp];
            let inext = (i + 1) % envelope.len();
            let iprev = (i + envelope.len() - 1) % envelope.len();
            let point_next: &Point = &self.points[envelope[inext].0];
            let point_prev: &Point = &self.points[envelope[iprev].0];

            let m1 = Point { x: (bp.x + point_next.x) / 2., y: (bp.y + point_next.y) / 2. };
            let m2 = Point { x: (bp.x + point_prev.x) / 2., y: (bp.y + point_prev.y) / 2. };

            let itriangle_next = envl.1;
            let itriangle_prev = {
                if i%2 == 0 {
                    envelope[iprev].1
                } else {
                    itriangle_next
                }
            };

            let triangle_next =  [&self.points[self.triangles[it * 3]], &self.points[self.triangles[it * 3 + 1]], &self.points[self.triangles[it * 3 + 2]]];
            let triangle_prev = [&self.points[self.triangles[it * 3]], &self.points[self.triangles[it * 3 + 1]], &self.points[self.triangles[it * 3 + 2]]];

            
            //let triangle_next = [self.triangles[itriangle_next * 3], self.triangles[itriangle_next * 3 + 1], self.triangles[itriangle_next * 3 + 2]];
            //let triangle_prev = [self.triangles[itriangle_prev * 3], self.triangles[itriangle_prev * 3 + 1], self.triangles[itriangle_prev * 3 + 2]];
            

            //let c1 = circumcircle(&self.points, &triangle_next).0;
            //let c3 = circumcircle(&self.points, &triangle_prev).0;
            let c1 = circumcircle(&triangle_next).0;
            let c3 = circumcircle(&triangle_prev).0;

            // calculate area of the polygon formed by bp->m1->c1->c2->c3->m2->bp
            let area_positive =
                area_of_triangle(&[bp, &m1, &c1]) +
                area_of_triangle(&[bp, &c1, &c2]) +
                area_of_triangle(&[bp, &c2, &c3]) +
                area_of_triangle(&[bp, &c3, &m2]);

            let g1 = circumcircle(&[&p, bp, point_next]).0;
            let g2 = circumcircle(&[&p, bp, point_prev]).0;

            // calculate area of the polygon formed by bp->m1->g1->g2->m2->bp
            let area_negative =
                area_of_triangle(&[bp, &m1, &g1]) +
                area_of_triangle(&[bp, &g1, &g2]) +
                area_of_triangle(&[bp, &g2, &m2]);

            
            //println!("area_positive - area_negative: {}", area_positive - area_negative);

            area_positive - area_negative

        }).collect::<Vec<_>>();

        let asum = areas.iter().sum::<f64>();
        let weights = areas.iter().map(|a| a / asum).collect::<Vec<_>>();

        let vs = envelope.iter().enumerate().map(|(i,envl)| {
            let ibp = envl.0;
            let v = &self.items[ibp];
            mul(v, weights[i])
        }).collect::<Vec<_>>();
        let v = vs.iter().fold(vs[0], |acc, v| add(&acc, v));
        
        Some(v)
    }
}
