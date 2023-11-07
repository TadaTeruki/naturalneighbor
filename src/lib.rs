//! # naturalneighbor
//!
//! `naturalneighbor` is a library to provide 2D Natural Neighbor Interpolation (NNI) for Rust.
//!
//! The implementation of this library is based on '[A Fast and Accurate Algorithm for Natural Neighbor Interpolation](https://gwlucastrig.github.io/TinfourDocs/NaturalNeighborTinfourAlgorithm/index.html)' by G.W. Lucas.
use primitives::Triangle;
use util::{circumcenter, circumcircle_with_radius_2, next_harfedge};

mod primitives;
mod util;

/// Represents a 2D point.
pub type Point = delaunator::Point;

/// Defines objects that can apply linear interpolation.
///
/// The value to be interpolated must implement this trait.
/// `f64` implements this trait by default.
///
/// # Examples
/// ```
///
/// use naturalneighbor::Lerpable;
///
/// #[derive(Copy, Clone, Debug)]
/// pub struct Color {
///     pub r: f64,
///     pub g: f64,
///     pub b: f64,
/// }
///
/// impl Lerpable for Color {
///     fn lerp(&self, other: &Self, weight: f64) -> Self {
///         Self {
///             r: self.r * (1.0 - weight) + other.r * weight,
///             g: self.g * (1.0 - weight) + other.g * weight,
///             b: self.b * (1.0 - weight) + other.b * weight,
///         }
///     }
/// }
/// ```

pub trait Lerpable: Clone {
    /// Apply linear interpolation with weight (0.0-1.0).
    fn lerp(&self, other: &Self, weight: f64) -> Self;
}

// Implementation of Lerpable for all float values that can convert to f64
impl<V> Lerpable for V
where
    V: Into<f64> + From<f64> + Copy,
{
    fn lerp(&self, other: &Self, weight: f64) -> Self {
        let result_f64 = (*self).into() * (1.0 - weight) + (*other).into() * weight;
        result_f64.into()
    }
}

/// Provides method for calculating natural neighbor interpolation.
///
/// This includes:
///  - Cloned point data
///  - RTree structure to find the triangle as the origin of the boyer-watson envelope
///  - Delaunay triangulation to construct the boyer-watson envelope for calculating the weight
///
/// Use `interpolate(&self, values: &[V], ptarget: P)` to interpolate the value at the point.
///
/// # Examples
///
/// ```
///
/// use naturalneighbor::{Point, Interpolator};
///
/// let points = [
///     Point { x: 0.0, y: 0.0 },
///     Point { x: 100.0, y: 0.0 },
///     Point { x: 100.0, y: 100.0 },
///     Point { x: 0.0, y: 100.0 },
/// ];
///
/// // weights of the points to be interpolated
/// let weights = [
///     1.0, 0.0, 1.0, 0.0
/// ];
///
/// let interpolator = Interpolator::new(&points);
///
/// let weight = interpolator.interpolate(&weights, Point {
///     x: 50.0,
///     y: 50.0,
/// }).unwrap();
///
/// assert_eq!(weight, 0.5);
///
/// ```
pub struct Interpolator {
    points: Vec<Point>,
    triangles: Vec<usize>,
    harfedges: Vec<usize>,
    tree: rstar::RTree<Triangle>,
}

impl Interpolator {
    /// Create a new Interpolator from a slice of points.
    ///
    pub fn new<P>(points: &[P]) -> Self
    where
        P: Into<Point> + Clone,
    {
        let points = points
            .iter()
            .map(|p| (*p).clone().into())
            .collect::<Vec<Point>>();

        let triangulation = delaunator::triangulate(&points);

        let circumcircles = triangulation
            .triangles
            .chunks_exact(3)
            .enumerate()
            .map(|(t, _)| Triangle::from_triangle(&points, &triangulation.triangles, t))
            .collect::<Vec<_>>();

        let rtree = rstar::RTree::bulk_load(circumcircles);

        Self {
            points,
            triangles: triangulation.triangles,
            harfedges: triangulation.halfedges,
            tree: rtree,
        }
    }

    // edges.0 -> edges.1 -> edges.2
    fn calculate_weight_area(&self, ptarget: &Point, edges: (usize, usize, usize)) -> f64 {
        let point_prev = &self.points[self.triangles[edges.0]];
        let point_base = &self.points[self.triangles[edges.1]];
        let point_next = &self.points[self.triangles[edges.2]];

        let mprev = &Point {
            x: (point_base.x + point_prev.x) / 2.,
            y: (point_base.y + point_prev.y) / 2.,
        };
        let mnext = &Point {
            x: (point_base.x + point_next.x) / 2.,
            y: (point_base.y + point_next.y) / 2.,
        };

        let mut ce = edges.0;

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
                if edges.1 == next {
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

    // value: the value to be weighted
    // weight_sum: tentative sum of the weight
    // ptarget: the point to be interpolated
    // edges.0 -> edges.1 -> edges.2
    fn apply_weight<V>(
        &self,
        values: &[V],
        value: Option<V>,
        weight_sum: f64,
        ptarget: &Point,
        edges: (usize, usize, usize),
    ) -> (Option<V>, f64)
    where
        V: Lerpable,
    {
        let vbase = &values[self.triangles[edges.1]];

        let weight = self.calculate_weight_area(ptarget, edges);
        let weight_sum = weight_sum + weight;

        if let Some(value) = value {
            (Some(value.lerp(vbase, weight / weight_sum)), weight_sum)
        } else {
            (Some(vbase.clone()), weight_sum)
        }
    }

    /// Interpolate the value at the point.
    /// If the point is outside the triangulation or the number of points and values are not the same, None is returned.
    pub fn interpolate<P, V>(&self, values: &[V], ptarget: P) -> Option<V>
    where
        P: Into<Point> + Clone,
        V: Lerpable,
    {
        if self.points.len() != values.len() {
            return None;
        }

        let ptarget = ptarget.into();

        // initial edge
        let start: usize = {
            let triangles = self
                .tree
                .locate_all_at_point(&[ptarget.x, ptarget.y])
                .filter(|circle| circle.point_in_triangle(&self.points, &self.triangles, &ptarget))
                .collect::<Vec<_>>();

            if triangles.len() >= 3 {
                let mut result = None;
                triangles.iter().for_each(|t| {
                    let it = t.itriangle();
                    let triangle = [
                        self.triangles[it * 3],
                        self.triangles[it * 3 + 1],
                        self.triangles[it * 3 + 2],
                    ];

                    triangle.iter().for_each(|i| {
                        if self.points[*i].x == ptarget.x && self.points[*i].y == ptarget.y {
                            result = Some(values[*i].clone());
                        }
                    });
                });
                if let Some(result) = result {
                    return Some(result);
                }
            }

            if let Some(t) = triangles.get(0) {
                t.itriangle() * 3
            } else {
                return None;
            }
        };

        // The value to be returned.
        let mut value: Option<V> = None;

        // Stream of edges on the boyer-watson envelope.
        // edges.0 -> edges.1 -> edges.2
        // The result value is updated when all elements of edges are on the envelope.
        let mut edges = (self.harfedges.len(), self.harfedges.len(), start);

        // the first and second edge of the envelope.
        // efirst2.0 -> efirst2.1
        // In the first and second iteration, the result value is not calculated because some elements of edges is not on the envelope.
        // After the envelope is closed, the rest of the process is processed using efirst2.
        let mut efirst2 = None;

        // the tentative sum of the weight.
        let mut weight_sum = 0.;

        loop {
            let opposite = self.harfedges[edges.2];

            // if the opposite harfedge of the earlist harfedge in the stream exists
            if opposite < self.harfedges.len() {
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

                if dist2 < r2 {
                    edges.2 = next_harfedge(opposite);
                    continue;
                }
            }

            // if it is in the first iteration after all elements of edges are on the envelope
            if edges.0 < self.harfedges.len() {
                if efirst2.is_none() {
                    efirst2 = Some((edges.0, edges.1));
                }
                (value, weight_sum) = self.apply_weight(values, value, weight_sum, &ptarget, edges);
            }

            // update edges
            edges = (edges.1, edges.2, next_harfedge(edges.2));

            // if the envelope is closed
            if self.triangles[start] == self.triangles[edges.2] {
                (value, weight_sum) = self.apply_weight(
                    values,
                    value,
                    weight_sum,
                    &ptarget,
                    (edges.0, edges.1, efirst2.unwrap().0),
                );
                (value, _) = self.apply_weight(
                    values,
                    value,
                    weight_sum,
                    &ptarget,
                    (edges.1, efirst2.unwrap().0, efirst2.unwrap().1),
                );
                break;
            }
        }

        value
    }
}
