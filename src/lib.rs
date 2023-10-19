use primitives::Triangle;
use util::{circumcenter, circumcircle_with_radius_2, next_harfedge};

mod primitives;
mod util;
pub type Point = delaunator::Point;

pub trait Lerpable: Copy {
    fn lerp(&self, other: &Self, weight: f64) -> Self;
}

pub struct InterpolatorBuilder<'a, V>
where
    V: Lerpable,
{
    points: Option<&'a [Point]>,
    items: Option<&'a [V]>,
}

impl<V> Default for InterpolatorBuilder<'_, V>
where
    V: Lerpable,
{
    fn default() -> Self {
        Self {
            points: None,
            items: None,
        }
    }
}

impl<'a, V> InterpolatorBuilder<'a, V>
where
    V: Lerpable,
{
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

    pub fn build(&self) -> Option<Interpolator<'a, V>>
    where
        V: Lerpable,
    {
        if let (Some(points), Some(items)) = (self.points, self.items) {
            if points.len() != items.len() {
                return None;
            }

            let triangulation = delaunator::triangulate(points);

            let circumcircles = triangulation
                .triangles
                .chunks_exact(3)
                .enumerate()
                .map(|(t, _)| Triangle::from_triangle(points, &triangulation.triangles, t))
                .collect::<Vec<_>>();

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

pub struct Interpolator<'a, V>
where
    V: Lerpable,
{
    points: &'a [Point],
    items: &'a [V],
    triangles: Vec<usize>,
    harfedges: Vec<usize>,
    tree: rstar::RTree<Triangle>,
}

impl<'a, V> Interpolator<'a, V>
where
    V: Lerpable,
{
    pub fn interpolate(&self, ptarget: Point) -> Option<V> {
        let triangles = self
            .tree
            .locate_all_at_point(&[ptarget.x, ptarget.y])
            .filter(|circle| circle.point_in_triangle(self.points, &self.triangles, &ptarget))
            .collect::<Vec<_>>();

        let it: usize = {
            if let Some(t) = triangles.get(0) {
                t.itriangle()
            } else {
                return None;
            }
        };

        let start = it * 3;
        let mut current = start;
        let mut envelope = vec![];

        // create boyer-watson envelope
        loop {
            let opposite = self.harfedges[current];

            if opposite < self.harfedges.len() {
                // triangle of the opposite harfedge
                let oit = opposite / 3;
                let triangle = [
                    self.triangles[oit * 3],
                    self.triangles[oit * 3 + 1],
                    self.triangles[oit * 3 + 2],
                ];
                // circumcicle of the triangle
                let (c, r2) = circumcircle_with_radius_2(&[
                    &self.points[triangle[0]],
                    &self.points[triangle[1]],
                    &self.points[triangle[2]],
                ]);
                // check if the point is in the circumcircle
                let dist2 = (c.x - ptarget.x).powi(2) + (c.y - ptarget.y).powi(2);

                if dist2 <= r2 {
                    current = next_harfedge(opposite);
                    continue;
                }
            }

            envelope.push(current);
            current = next_harfedge(current);

            if self.triangles[start] == self.triangles[current] {
                break;
            }
        }

        let areas = envelope
            .iter()
            .enumerate()
            .map(|(i, eref)| {
                let e = *eref;
                let ibp = self.triangles[e];
                let inext = (i + 1) % envelope.len();
                let iprev = (i + envelope.len() - 1) % envelope.len();
                let bp = &self.points[ibp];
                let point_next: &Point = &self.points[self.triangles[envelope[inext]]];
                let point_prev: &Point = &self.points[self.triangles[envelope[iprev]]];

                let mprev = &Point {
                    x: (bp.x + point_prev.x) / 2.,
                    y: (bp.y + point_prev.y) / 2.,
                };
                let mnext = &Point {
                    x: (bp.x + point_next.x) / 2.,
                    y: (bp.y + point_next.y) / 2.,
                };

                let mut ce = envelope[iprev];

                let pre = {
                    let mut pre = 0.;
                    let mut cs1 = mprev.clone();
                    loop {
                        let cit = ce / 3;
                        let triangle = [
                            &self.points[self.triangles[cit * 3]],
                            &self.points[self.triangles[cit * 3 + 1]],
                            &self.points[self.triangles[cit * 3 + 2]],
                        ];
                        let c = circumcenter(&triangle);
                        pre += (cs1.x - c.x) * (cs1.y + c.y);
                        cs1 = c;
                        let next = next_harfedge(ce);
                        if e == next {
                            break;
                        }
                        ce = self.harfedges[next];
                    }
                    pre + (cs1.x - mnext.x) * (cs1.y + mnext.y)
                        + (mnext.x - mprev.x) * (mnext.y + mprev.y)
                };

                let gprev = circumcenter(&[&ptarget, bp, point_prev]);
                let gnext = circumcenter(&[&ptarget, bp, point_next]);

                let post = (mprev.x - gprev.x) * (mprev.y + gprev.y)
                    + (gprev.x - gnext.x) * (gprev.y + gnext.y)
                    + (gnext.x - mnext.x) * (gnext.y + mnext.y)
                    + (mnext.x - mprev.x) * (mnext.y + mprev.y);

                pre - post
            })
            .collect::<Vec<_>>();

        let area_sum = areas.iter().sum::<f64>();
        let weights = areas.iter().map(|a| a / area_sum).collect::<Vec<_>>();

        let v = {
            let mut v = self.items[self.triangles[envelope[0]]];
            let mut weight_sum = weights[0];
            for i in 1..envelope.len() {
                let ibp = self.triangles[envelope[i]];
                let v2 = self.items[ibp];
                weight_sum += weights[i];
                v = v.lerp(&v2, weights[i] / weight_sum);
            }
            v
        };

        Some(v)
    }
}
