//! Curve representations (lines, arcs, NURBS)

use super::{Point3, Vector3, TOLERANCE};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

/// A parametric curve trait
pub trait Curve: Send + Sync {
    /// Evaluate the curve at parameter t ∈ [0, 1]
    fn evaluate(&self, t: f64) -> Point3;

    /// Evaluate the tangent at parameter t
    fn tangent(&self, t: f64) -> Vector3;

    /// Get the length of the curve
    fn length(&self) -> f64;

    /// Get the parameter closest to a point
    fn closest_parameter(&self, point: Point3) -> f64;

    /// Split the curve at parameter t
    fn split(&self, t: f64) -> (Box<dyn Curve>, Box<dyn Curve>);
}

/// A line segment
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub start: Point3,
    pub end: Point3,
}

impl Line {
    #[inline]
    pub fn new(start: Point3, end: Point3) -> Self {
        Self { start, end }
    }

    #[inline]
    pub fn direction(&self) -> Vector3 {
        (self.end - self.start).normalize_or_zero()
    }

    #[inline]
    pub fn midpoint(&self) -> Point3 {
        (self.start + self.end) * 0.5
    }

    /// Find the closest point on this line segment to the given point
    pub fn closest_point(&self, point: Point3) -> Point3 {
        let v = self.end - self.start;
        let w = point - self.start;

        let c1 = w.dot(v);
        if c1 <= 0.0 {
            return self.start;
        }

        let c2 = v.dot(v);
        if c2 <= c1 {
            return self.end;
        }

        let t = c1 / c2;
        self.start + v * t
    }

    /// Distance from a point to this line segment
    #[inline]
    pub fn distance_to_point(&self, point: Point3) -> f64 {
        point.distance(self.closest_point(point))
    }
}

impl Curve for Line {
    #[inline]
    fn evaluate(&self, t: f64) -> Point3 {
        self.start.lerp(self.end, t)
    }

    #[inline]
    fn tangent(&self, _t: f64) -> Vector3 {
        self.direction()
    }

    #[inline]
    fn length(&self) -> f64 {
        self.start.distance(self.end)
    }

    fn closest_parameter(&self, point: Point3) -> f64 {
        let v = self.end - self.start;
        let len_sq = v.length_squared();
        if len_sq < TOLERANCE * TOLERANCE {
            return 0.0;
        }
        ((point - self.start).dot(v) / len_sq).clamp(0.0, 1.0)
    }

    fn split(&self, t: f64) -> (Box<dyn Curve>, Box<dyn Curve>) {
        let mid = self.evaluate(t);
        (
            Box::new(Line::new(self.start, mid)),
            Box::new(Line::new(mid, self.end)),
        )
    }
}

/// A circular arc
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Arc {
    pub center: Point3,
    pub radius: f64,
    pub start_angle: f64,
    pub end_angle: f64,
    pub normal: Vector3,
}

impl Arc {
    pub fn new(center: Point3, radius: f64, start_angle: f64, end_angle: f64, normal: Vector3) -> Self {
        Self {
            center,
            radius,
            start_angle,
            end_angle,
            normal: normal.normalize_or_zero(),
        }
    }

    /// Create an arc from three points
    pub fn from_three_points(start: Point3, mid: Point3, end: Point3) -> Option<Self> {
        // Compute the circumcenter
        let a = start;
        let b = mid;
        let c = end;

        let ab = b - a;
        let ac = c - a;
        let normal = ab.cross(ac);

        if normal.length_squared() < TOLERANCE * TOLERANCE {
            return None; // Points are collinear
        }

        let normal = normal.normalize();

        // Find circumcenter using perpendicular bisectors
        let ab_mid = (a + b) * 0.5;
        let ac_mid = (a + c) * 0.5;

        let ab_perp = normal.cross(ab);
        let ac_perp = normal.cross(ac);

        // Solve for intersection
        let denom = ab_perp.cross(ac_perp).dot(normal);
        if denom.abs() < TOLERANCE {
            return None;
        }

        let t = (ac_mid - ab_mid).cross(ac_perp).dot(normal) / denom;
        let center = ab_mid + ab_perp * t;
        let radius = center.distance(a);

        // Calculate angles
        let u = if normal.x.abs() < 0.9 {
            Vector3::X.cross(normal).normalize()
        } else {
            Vector3::Y.cross(normal).normalize()
        };
        let v = normal.cross(u);

        let to_angle = |p: Point3| {
            let d = p - center;
            d.dot(v).atan2(d.dot(u))
        };

        let start_angle = to_angle(start);
        let end_angle = to_angle(end);

        Some(Self::new(center, radius, start_angle, end_angle, normal))
    }

    fn local_frame(&self) -> (Vector3, Vector3) {
        let u = if self.normal.x.abs() < 0.9 {
            Vector3::X.cross(self.normal).normalize()
        } else {
            Vector3::Y.cross(self.normal).normalize()
        };
        let v = self.normal.cross(u);
        (u, v)
    }
}

impl Curve for Arc {
    fn evaluate(&self, t: f64) -> Point3 {
        let angle = self.start_angle + t * (self.end_angle - self.start_angle);
        let (u, v) = self.local_frame();
        self.center + (u * angle.cos() + v * angle.sin()) * self.radius
    }

    fn tangent(&self, t: f64) -> Vector3 {
        let angle = self.start_angle + t * (self.end_angle - self.start_angle);
        let (u, v) = self.local_frame();
        (-u * angle.sin() + v * angle.cos()).normalize()
    }

    fn length(&self) -> f64 {
        self.radius * (self.end_angle - self.start_angle).abs()
    }

    fn closest_parameter(&self, point: Point3) -> f64 {
        let (u, v) = self.local_frame();
        let d = point - self.center;
        let angle = d.dot(v).atan2(d.dot(u));

        let angle_range = self.end_angle - self.start_angle;
        let mut t = (angle - self.start_angle) / angle_range;
        t = t.clamp(0.0, 1.0);
        t
    }

    fn split(&self, t: f64) -> (Box<dyn Curve>, Box<dyn Curve>) {
        let mid_angle = self.start_angle + t * (self.end_angle - self.start_angle);
        (
            Box::new(Arc::new(self.center, self.radius, self.start_angle, mid_angle, self.normal)),
            Box::new(Arc::new(self.center, self.radius, mid_angle, self.end_angle, self.normal)),
        )
    }
}

/// A NURBS curve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NurbsCurve {
    pub degree: usize,
    pub control_points: Vec<Point3>,
    pub weights: Vec<f64>,
    pub knots: Vec<f64>,
}

impl NurbsCurve {
    pub fn new(degree: usize, control_points: Vec<Point3>, weights: Vec<f64>, knots: Vec<f64>) -> Self {
        Self {
            degree,
            control_points,
            weights,
            knots,
        }
    }

    /// Create a B-spline curve (uniform weights)
    pub fn bspline(degree: usize, control_points: Vec<Point3>, knots: Vec<f64>) -> Self {
        let weights = vec![1.0; control_points.len()];
        Self::new(degree, control_points, weights, knots)
    }

    /// Create a Bezier curve
    pub fn bezier(control_points: Vec<Point3>) -> Self {
        let n = control_points.len();
        let degree = n - 1;
        let mut knots = vec![0.0; n];
        knots.extend(vec![1.0; n]);
        let weights = vec![1.0; n];
        Self::new(degree, control_points, weights, knots)
    }

    /// Evaluate using de Boor's algorithm
    fn de_boor(&self, t: f64) -> Point3 {
        let n = self.control_points.len();
        let p = self.degree;

        // Find the knot span
        let mut k = p;
        for i in p..n {
            if t < self.knots[i + 1] {
                k = i;
                break;
            }
        }

        // Initialize with control points in homogeneous coordinates
        let mut d: SmallVec<[(Point3, f64); 8]> = SmallVec::new();
        for i in 0..=p {
            let idx = k - p + i;
            d.push((self.control_points[idx] * self.weights[idx], self.weights[idx]));
        }

        // de Boor iterations
        for r in 1..=p {
            for j in (r..=p).rev() {
                let i = k - p + j;
                let alpha = (t - self.knots[i]) / (self.knots[i + p + 1 - r] - self.knots[i]);

                let (prev_pt, prev_w) = d[j - 1];
                let (curr_pt, curr_w) = d[j];

                let new_pt = prev_pt * (1.0 - alpha) + curr_pt * alpha;
                let new_w = prev_w * (1.0 - alpha) + curr_w * alpha;

                d[j] = (new_pt, new_w);
            }
        }

        let (pt, w) = d[p];
        pt / w
    }
}

impl Curve for NurbsCurve {
    fn evaluate(&self, t: f64) -> Point3 {
        let t_mapped = self.knots[self.degree] + t * (self.knots[self.knots.len() - self.degree - 1] - self.knots[self.degree]);
        self.de_boor(t_mapped)
    }

    fn tangent(&self, t: f64) -> Vector3 {
        // Numerical differentiation
        let h = 0.0001;
        let t0 = (t - h).max(0.0);
        let t1 = (t + h).min(1.0);
        let p0 = self.evaluate(t0);
        let p1 = self.evaluate(t1);
        (p1 - p0).normalize_or_zero()
    }

    fn length(&self) -> f64 {
        // Numerical integration using adaptive Simpson's rule
        const SEGMENTS: usize = 100;
        let mut length = 0.0;
        let mut prev = self.evaluate(0.0);

        for i in 1..=SEGMENTS {
            let t = i as f64 / SEGMENTS as f64;
            let curr = self.evaluate(t);
            length += prev.distance(curr);
            prev = curr;
        }

        length
    }

    fn closest_parameter(&self, point: Point3) -> f64 {
        // Newton-Raphson iteration with initial search
        const SAMPLES: usize = 20;
        let mut best_t = 0.0;
        let mut best_dist = f64::MAX;

        for i in 0..=SAMPLES {
            let t = i as f64 / SAMPLES as f64;
            let p = self.evaluate(t);
            let dist = p.distance_squared(point);
            if dist < best_dist {
                best_dist = dist;
                best_t = t;
            }
        }

        // Refine with numerical optimization (simplified gradient descent)
        for _ in 0..10 {
            let h = 0.0001;
            let f = self.evaluate(best_t).distance_squared(point);
            let f_plus = self.evaluate((best_t + h).min(1.0)).distance_squared(point);
            let f_minus = self.evaluate((best_t - h).max(0.0)).distance_squared(point);

            let gradient = (f_plus - f_minus) / (2.0 * h);
            best_t -= 0.01 * gradient;
            best_t = best_t.clamp(0.0, 1.0);

            if gradient.abs() < TOLERANCE {
                break;
            }
        }

        best_t
    }

    fn split(&self, _t: f64) -> (Box<dyn Curve>, Box<dyn Curve>) {
        // For simplicity, approximate with line segments
        // A full implementation would use knot insertion
        let mid = self.evaluate(_t);
        let start = self.evaluate(0.0);
        let end = self.evaluate(1.0);
        (
            Box::new(Line::new(start, mid)),
            Box::new(Line::new(mid, end)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::DVec3;

    #[test]
    fn test_line() {
        let line = Line::new(DVec3::ZERO, DVec3::new(2.0, 0.0, 0.0));
        assert!((line.length() - 2.0).abs() < TOLERANCE);
        assert!((line.evaluate(0.5) - DVec3::new(1.0, 0.0, 0.0)).length() < TOLERANCE);
    }

    #[test]
    fn test_bezier() {
        let curve = NurbsCurve::bezier(vec![
            DVec3::ZERO,
            DVec3::new(1.0, 1.0, 0.0),
            DVec3::new(2.0, 0.0, 0.0),
        ]);
        let start = curve.evaluate(0.0);
        let end = curve.evaluate(1.0);
        assert!((start - DVec3::ZERO).length() < TOLERANCE);
        assert!((end - DVec3::new(2.0, 0.0, 0.0)).length() < TOLERANCE);
    }
}
