//! # naturalneighbor
//!
//! `naturalneighbor` is a library to provide 2D Natural Neighbor Interpolation (NNI) for Rust.
//!
//! The implementation of this library is based on '[A Fast and Accurate Algorithm for Natural Neighbor Interpolation](https://gwlucastrig.github.io/TinfourDocs/NaturalNeighborTinfourAlgorithm/index.html)' by G.W. Lucas.
//!
//! ## Documentation
//!
//! See the [Interpolator] struct for the main documentation of this crate.
//!
use primitives::Triangle;
use thiserror::Error;
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
/// # Example
/// ```
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
/// Use `query_weights(&self, ptarget: P)` to query the result of the interpolation as a list of indices of sites to be weighted.
///
/// # Example
///
/// ```
/// use naturalneighbor::{Point, Interpolator};
///
/// // A macro for comparing floating point values.
/// macro_rules! assert_approx_eq {
///    ($a:expr, $b:expr) => {
///     assert!(($a - $b).abs() < 1e-6);
///   };
/// }
///
/// // Pairs of points and values to be interpolated.
/// let points = [
///     Point { x: 0.0, y: 0.0 },
///     Point { x: 100.0, y: 0.0 },
///     Point { x: 100.0, y: 100.0 },
///     Point { x: 0.0, y: 100.0 },
/// ];
///
/// let values = [
///     1.0, 0.0, 1.0, 0.0
/// ];
///
/// // Create an interpolator from the points.
/// let interpolator = Interpolator::new(&points);
///
/// // Calculate the value at the point.
/// let value = interpolator.interpolate(&values, Point {
///     x: 50.0,
///     y: 50.0,
/// }).unwrap().unwrap();
///
/// assert_approx_eq!(value, 0.5);
///
/// // Query the result of the interpolation as a list of indices of sites to be weighted.
/// let mut value_and_weight = interpolator.query_weights(Point {
///    x: 50.0,
///    y: 50.0,
/// }).unwrap().unwrap();
///
/// for (i, w) in value_and_weight.iter() {
///    assert_approx_eq!(*w, 0.25);
/// }
/// assert_approx_eq!(value_and_weight.iter().map(|(i, w)| values[*i] * w).sum::<f64>(), 0.5);
/// ```
#[derive(Clone)]
pub struct Interpolator {
    points: Vec<Point>,
    triangles: Vec<usize>,
    harfedges: Vec<usize>,
    tree: rstar::RTree<Triangle>,
    degree_limitation: usize,
}

// The epsiron value for the interpolator.
// This is used to move the point slightly when the point is on the edge of the triangulation.
// because calculating the weight of the point on the edge is not stable.
// This value must be greater than primitives::EPS_TRIANGLE.
static EPS_INTERPOLATOR: f64 = 1e-12;

// The default degree limitation of the interpolator.
static DEFAULT_DEGREE_LIMITATION: usize = 30;

#[derive(Error, Debug)]
pub enum InterpolatorError {
    /// This error occurs when the number of neighbors of the point is higher than the degree limitation of the interpolator.
    /// This error is for preventing the interpolator from running infinitely.
    /// You can customize the degree limitation by using Interpolator::new_with_curtom_degree_limitation instead of Interpolator::new.
    #[error("A site with too many neighbors is detected. The number of neighbors the is higher than the degree limitation of the interpolator({0}).")]
    TooManyNeighbors(usize),
    #[error("The number of points and values are not the same.")]
    DifferentNumberOfPointsAndValues,
}

impl Interpolator {
    /// Create a new Interpolator from a slice of points.
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
            degree_limitation: DEFAULT_DEGREE_LIMITATION,
        }
    }

    /// Create a new Interpolator from a slice of points with degree limitation.
    pub fn new_with_curtom_degree_limitation<P>(points: &[P], degree_limitation: usize) -> Self
    where
        P: Into<Point> + Clone,
    {
        let mut interpolator = Self::new(points);
        interpolator.degree_limitation = degree_limitation;
        interpolator
    }

    fn detect_too_large_degree(&self, dct: usize) -> bool {
        dct >= self.degree_limitation - 1
    }

    // edges.0 -> edges.1 -> edges.2
    fn calculate_weight_area(
        &self,
        ptarget: &Point,
        edges: (usize, usize, usize),
    ) -> Result<f64, InterpolatorError> {
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
            for dcount in 0..self.degree_limitation {
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

                if self.detect_too_large_degree(dcount) {
                    return Err(InterpolatorError::TooManyNeighbors(self.degree_limitation));
                }
            }
            pre + (cs1.x - mnext.x) * (cs1.y + mnext.y) + (mnext.x - mprev.x) * (mnext.y + mprev.y)
        };

        let gprev = circumcenter(&[ptarget, point_base, point_prev]);
        let gnext = circumcenter(&[ptarget, point_base, point_next]);

        let post = (mprev.x - gprev.x) * (mprev.y + gprev.y)
            + (gprev.x - gnext.x) * (gprev.y + gnext.y)
            + (gnext.x - mnext.x) * (gnext.y + mnext.y)
            + (mnext.x - mprev.x) * (mnext.y + mprev.y);

        Ok(pre - post)
    }

    fn fit_in_triangle(&self, ptarget: &Point, check_around: bool) -> Option<(usize, Point)> {
        let triangles = self
            .tree
            .locate_all_at_point(&[ptarget.x, ptarget.y])
            .filter(|circle| circle.point_in_triangle(&self.points, &self.triangles, ptarget))
            .collect::<Vec<_>>();

        if triangles.len() >= 2 {
            if !check_around {
                return None;
            }
            let eps = EPS_INTERPOLATOR;

            // random (mannually selected) points around the target point
            let check_angles = [
                Point {
                    x: eps * 1.415,
                    y: eps * 1.339,
                },
                Point {
                    x: eps * 1.335,
                    y: -eps * 1.483,
                },
                Point {
                    x: -eps * 1.421,
                    y: -eps * 1.384,
                },
                Point {
                    x: -eps * 1.498,
                    y: eps * 1.322,
                },
            ];

            for angle in check_angles {
                let check_point = Point {
                    x: ptarget.x + angle.x,
                    y: ptarget.y + angle.y,
                };
                if let Some(t) = self.fit_in_triangle(&check_point, false) {
                    return Some(t);
                }
            }

            return None;
        }

        triangles
            .get(0)
            .map(|t| (t.itriangle() * 3, ptarget.clone()))
    }

    /// Perform natural neighbor interpolation.
    ///
    /// The 'apply_weight' function is called if the point is iterated as one of the natural neighbors.
    /// The first argument is the index of the point, the second argument is the weight of the point, and the third argument is the tentative sum of the weight.
    /// See the implementation of `Interpolator::interpolate` as an example.
    fn perform_interpoation<P>(
        &self,
        ptarget: P,
        apply_weight: &mut impl FnMut(usize, f64, f64),
    ) -> Result<(), InterpolatorError>
    where
        P: Into<Point> + Clone,
    {
        let ptarget = ptarget.into();

        // initial edge
        let (start, ptarget) = if let Some(t) = self.fit_in_triangle(&ptarget, true) {
            t
        } else {
            return Ok(());
        };

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
        let mut tmp_weight_sum = 0.;

        // apply the weight.
        let mut apply =
            |edges: (usize, usize, usize), tmp_weight_sum: f64| -> Result<f64, InterpolatorError> {
                let weight = self.calculate_weight_area(&ptarget, edges)?;
                let tmp_weight_sum: f64 = tmp_weight_sum + weight;
                apply_weight(self.triangles[edges.1], weight, tmp_weight_sum);
                Ok(tmp_weight_sum)
            };

        for dcount in 0..self.degree_limitation {
            edges.2 = {
                let mut edge2 = edges.2;
                for dcount in 0..self.degree_limitation {
                    let opposite = self.harfedges[edge2];

                    // if the opposite is not found (the triangle is on the edge of the triangulation), break the loop.
                    if opposite >= self.harfedges.len() {
                        break;
                    }

                    let oit = opposite / 3;
                    let triangle_points = [
                        &self.points[self.triangles[oit * 3]],
                        &self.points[self.triangles[oit * 3 + 1]],
                        &self.points[self.triangles[oit * 3 + 2]],
                    ];

                    // circumcicle of the triangle
                    let (c, r2) = circumcircle_with_radius_2(&triangle_points);

                    // check if the point is in the circumcircle
                    let dist2 = (c.x - ptarget.x).powi(2) + (c.y - ptarget.y).powi(2);
                    if dist2 < r2 {
                        edge2 = next_harfedge(opposite);
                    } else {
                        break;
                    }

                    if self.detect_too_large_degree(dcount) {
                        return Err(InterpolatorError::TooManyNeighbors(self.degree_limitation));
                    }
                }
                edge2
            };

            // if it is in the first iteration after all elements of edges are on the envelope
            if edges.0 < self.harfedges.len() {
                if efirst2.is_none() {
                    efirst2 = Some((edges.0, edges.1));
                }
                tmp_weight_sum = apply((edges.0, edges.1, edges.2), tmp_weight_sum)?;
            }

            // update edges
            edges = (edges.1, edges.2, next_harfedge(edges.2));

            // if the envelope is closed
            if self.triangles[start] == self.triangles[edges.2] {
                tmp_weight_sum = apply((edges.0, edges.1, efirst2.unwrap().0), tmp_weight_sum)?;
                apply(
                    (edges.1, efirst2.unwrap().0, efirst2.unwrap().1),
                    tmp_weight_sum,
                )?;
                break;
            }

            if self.detect_too_large_degree(dcount) {
                return Err(InterpolatorError::TooManyNeighbors(self.degree_limitation));
            }
        }
        Ok(())
    }

    /// Interpolate the value at the point.
    /// If the point is outside the triangulation, None is returned.
    pub fn interpolate<P, V>(
        &self,
        values: &[V],
        ptarget: P,
    ) -> Result<Option<V>, InterpolatorError>
    where
        P: Into<Point> + Clone,
        V: Lerpable,
    {
        if self.points.len() != values.len() {
            return Err(InterpolatorError::DifferentNumberOfPointsAndValues);
        }

        let mut value: Option<V> = None;
        self.perform_interpoation::<P>(ptarget, &mut |i, weight, tmp_weight_sum| {
            let vbase = &values[i];
            let new_value = if let Some(value) = &value {
                Some(value.lerp(vbase, weight / tmp_weight_sum))
            } else {
                Some(vbase.clone())
            };
            value = new_value;
        })?;

        Ok(value)
    }

    /// Query the result of the interpolation as a list of indices of sites to be weighted.
    /// If the point is outside the triangulation, None is returned.
    pub fn query_weights<P>(
        &self,
        ptarget: P,
    ) -> Result<Option<Vec<(usize, f64)>>, InterpolatorError>
    where
        P: Into<Point> + Clone,
    {
        let mut weights = Vec::new();
        let mut weight_sum = 0.;
        self.perform_interpoation::<P>(ptarget, &mut |i, weight, _| {
            weight_sum += weight;
            weights.push((i, weight));
        })?;

        if weight_sum == 0. {
            Ok(None)
        } else {
            Ok(Some(
                weights.iter().map(|(i, w)| (*i, w / weight_sum)).collect(),
            ))
        }
    }
}
