use primitives::Triangle;
use util::{circumcenter, circumcircle_with_radius_2, next_harfedge};

mod primitives;
mod util;
pub type Point = delaunator::Point;

pub trait Lerpable: Clone {
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
    // ebase: harfedge starting from base point
    // eprev: previous harfedge of ebase
    // enext: next harfedge of ebase
    fn calculate_area(&self, ptarget: &Point, eprev: usize, ebase: usize, enext: usize) -> f64 {
        let point_prev: &Point = &self.points[self.triangles[eprev]];
        let point_base = &self.points[self.triangles[ebase]];
        let point_next: &Point = &self.points[self.triangles[enext]];

        let mprev = &Point {
            x: (point_base.x + point_prev.x) / 2.,
            y: (point_base.y + point_prev.y) / 2.,
        };
        let mnext = &Point {
            x: (point_base.x + point_next.x) / 2.,
            y: (point_base.y + point_next.y) / 2.,
        };

        let mut ce = eprev;

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
                if ebase == next {
                    break;
                }
                ce = self.harfedges[next];
            }
            pre + (cs1.x - mnext.x) * (cs1.y + mnext.y) + (mnext.x - mprev.x) * (mnext.y + mprev.y)
        };

        let gprev = circumcenter(&[ptarget, point_base, point_prev]);
        let gnext = circumcenter(&[ptarget, point_base, point_next]);

        let post = (mprev.x - gprev.x) * (mprev.y + gprev.y)
            + (gprev.x - gnext.x) * (gprev.y + gnext.y)
            + (gnext.x - mnext.x) * (gnext.y + mnext.y)
            + (mnext.x - mprev.x) * (mnext.y + mprev.y);

        pre - post
    }

    fn add_to_value(
        &self,
        value: Option<V>,
        area_sum: f64,
        ptarget: &Point,
        eprev: usize,
        ebase: usize,
        enext: usize,
    ) -> (Option<V>, f64) {
        let v2 = &self.items[self.triangles[ebase]];

        let area = self.calculate_area(ptarget, eprev, ebase, enext);
        let area_sum = area_sum + area;

        if let Some(value) = value {
            (Some(value.lerp(v2, area / area_sum)), area_sum)
        } else {
            (Some(v2.clone()), area_sum)
        }
    }

    pub fn interpolate(&self, ptarget: Point) -> Option<V> {
        let start: usize = {
            let triangles = self
                .tree
                .locate_all_at_point(&[ptarget.x, ptarget.y])
                .filter(|circle| circle.point_in_triangle(self.points, &self.triangles, &ptarget))
                .collect::<Vec<_>>();
            if let Some(t) = triangles.get(0) {
                t.itriangle() * 3
            } else {
                return None;
            }
        };

        let mut value: Option<V> = None;

        let mut epool = (self.harfedges.len(), self.harfedges.len(), start);
        let mut efirst2 = None;
        let mut area_sum = 0.;

        loop {
            let opposite = self.harfedges[epool.2];

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
                    epool.2 = next_harfedge(opposite);
                    continue;
                }
            }

            if epool.0 < self.harfedges.len() {
                if efirst2.is_none() {
                    efirst2 = Some((epool.0, epool.1));
                }
                (value, area_sum) =
                    self.add_to_value(value, area_sum, &ptarget, epool.0, epool.1, epool.2);
            }

            epool = (epool.1, epool.2, next_harfedge(epool.2));

            if self.triangles[start] == self.triangles[epool.2] {
                (value, area_sum) = self.add_to_value(
                    value,
                    area_sum,
                    &ptarget,
                    epool.0,
                    epool.1,
                    efirst2.unwrap().0,
                );
                (value, _) = self.add_to_value(
                    value,
                    area_sum,
                    &ptarget,
                    epool.1,
                    efirst2.unwrap().0,
                    efirst2.unwrap().1,
                );
                break;
            }
        }

        value
    }
}
