use circle::Triangle;
use util::{circumcircle, next_harfedge};

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
                Triangle::from_triangle(&points, &triangulation.triangles, t)
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
    tree: rstar::RTree<Triangle>,
}

impl<'a, V> Interpolator<'a, V> where V: Copy {
    pub fn interpolate(&self, ptarget: Point, add: impl Fn(&V, &V) -> V, mul: impl Fn(&V, f64) -> V) -> Option<V>{

        let triangles = self.tree.locate_all_at_point(&[ptarget.x, ptarget.y]).filter(|circle| {
            circle.point_in_triangle(&self.points, &self.triangles, &ptarget)
        }).collect::<Vec<_>>();

        let it: usize = {
            if let Some(t) = triangles.get(0) {
                t.itriangle()
            } else {
                return None;
            }
        };

        let start = it*3;
        let mut current = start;
        let mut envelope = vec![];

        // create boyer-watson envelope
        loop {
            let opposite = self.harfedges[current];
            // triangle of the opposite harfedge
            let oit = opposite / 3;
            let triangle = [self.triangles[oit * 3], self.triangles[oit * 3 + 1], self.triangles[oit * 3 + 2]];
            // circumcicle of the triangle
            let (c, r) = circumcircle(&[&self.points[triangle[0]], &self.points[triangle[1]], &self.points[triangle[2]]]);
            // check if the point is in the circumcircle
            let dist2 = (c.x - ptarget.x).powi(2) + (c.y - ptarget.y).powi(2);

            if dist2 <= r.powi(2) {
                current = next_harfedge(opposite);
                continue;
            }

            envelope.push(current);
            current = next_harfedge(current);

            if self.triangles[start] == self.triangles[current] {
                break;
            }
        };

        let areas = envelope.iter().enumerate().map(|(i, e)| {
            let ibp = self.triangles[*e];
            let bp = &self.points[ibp];
            let inext = (i + 1) % envelope.len();
            let iprev = (i + envelope.len() - 1) % envelope.len();
            let point_next: &Point = &self.points[self.triangles[envelope[inext]]];
            let point_prev: &Point = &self.points[self.triangles[envelope[iprev]]];

            let mprev = &Point { x: (bp.x + point_prev.x) / 2., y: (bp.y + point_prev.y) / 2. };
            let mnext = &Point { x: (bp.x + point_next.x) / 2., y: (bp.y + point_next.y) / 2. };
            
            let gprev = circumcircle(&[&ptarget, bp, point_prev]).0;
            let gnext = circumcircle(&[&ptarget, bp, point_next]).0;
            
            let mut ce = envelope[iprev];
            let mut cs = vec![mprev.clone()];

            loop {
                let cit = ce/3;
                let triangle = [&self.points[self.triangles[cit * 3]], &self.points[self.triangles[cit * 3 + 1]], &self.points[self.triangles[cit * 3 + 2]]];
                let c = circumcircle(&triangle).0;
                cs.push(c);
                let next = next_harfedge(ce);
                if *e == next {
                    break;
                }
                ce = self.harfedges[next];
            }

            cs.push(mnext.clone());

            let pre ={
                let mut pre = 0.;
                for i in 0..cs.len() {
                    let inx = (i + 1) % cs.len();
                    pre += (cs[i].x-cs[inx].x)*(cs[i].y+cs[inx].y);
                }
                pre
            };

            let post = 
                (mprev.x-gprev.x)*(mprev.y+gprev.y) +
                (gprev.x-gnext.x)*(gprev.y+gnext.y) +
                (gnext.x-mnext.x)*(gnext.y+mnext.y) +
                (mnext.x-mprev.x)*(mnext.y+mprev.y);

            let area = pre - post;
            area
        }).collect::<Vec<_>>();

        let area_sum = areas.iter().sum::<f64>();
        let weights = areas.iter().map(|a| a / area_sum).collect::<Vec<_>>();

        let vs = envelope.iter().enumerate().map(|(i, e)| {
            let ibp = self.triangles[*e];
            let v = &self.items[ibp];
            mul(v, weights[i])
        }).collect::<Vec<_>>();

        let v = {
            let mut v = vs[0];
            for i in 1..vs.len() {
                v = add(&v, &vs[i]);
            }
            v
        };

        Some(v)
        
    }
}
