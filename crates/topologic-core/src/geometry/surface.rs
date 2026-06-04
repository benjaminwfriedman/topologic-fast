//! Surface representations (planes, NURBS surfaces)

use super::{Point3, Vector3, Plane, TOLERANCE};
use serde::{Deserialize, Serialize};

/// A parametric surface trait
pub trait Surface: Send + Sync {
    /// Evaluate the surface at parameters (u, v) ∈ [0, 1]²
    fn evaluate(&self, u: f64, v: f64) -> Point3;

    /// Evaluate the normal at parameters (u, v)
    fn normal(&self, u: f64, v: f64) -> Vector3;

    /// Get the partial derivative with respect to u
    fn du(&self, u: f64, v: f64) -> Vector3;

    /// Get the partial derivative with respect to v
    fn dv(&self, u: f64, v: f64) -> Vector3;

    /// Get the surface area (approximation)
    fn area(&self) -> f64;

    /// Find the closest parameters to a point
    fn closest_parameters(&self, point: Point3) -> (f64, f64);
}

/// A planar surface
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlanarSurface {
    pub plane: Plane,
    pub u_axis: Vector3,
    pub v_axis: Vector3,
    pub u_range: (f64, f64),
    pub v_range: (f64, f64),
}

impl PlanarSurface {
    pub fn new(plane: Plane, u_range: (f64, f64), v_range: (f64, f64)) -> Self {
        let (u_axis, v_axis) = plane.local_frame();
        Self {
            plane,
            u_axis,
            v_axis,
            u_range,
            v_range,
        }
    }

    pub fn from_rectangle(origin: Point3, u_dir: Vector3, v_dir: Vector3, width: f64, height: f64) -> Self {
        let normal = u_dir.cross(v_dir).normalize_or_zero();
        let plane = Plane::new(origin, normal);
        Self {
            plane,
            u_axis: u_dir.normalize_or_zero(),
            v_axis: v_dir.normalize_or_zero(),
            u_range: (0.0, width),
            v_range: (0.0, height),
        }
    }
}

impl Surface for PlanarSurface {
    fn evaluate(&self, u: f64, v: f64) -> Point3 {
        let u_val = self.u_range.0 + u * (self.u_range.1 - self.u_range.0);
        let v_val = self.v_range.0 + v * (self.v_range.1 - self.v_range.0);
        self.plane.origin + self.u_axis * u_val + self.v_axis * v_val
    }

    fn normal(&self, _u: f64, _v: f64) -> Vector3 {
        self.plane.normal
    }

    fn du(&self, _u: f64, _v: f64) -> Vector3 {
        self.u_axis * (self.u_range.1 - self.u_range.0)
    }

    fn dv(&self, _u: f64, _v: f64) -> Vector3 {
        self.v_axis * (self.v_range.1 - self.v_range.0)
    }

    fn area(&self) -> f64 {
        (self.u_range.1 - self.u_range.0) * (self.v_range.1 - self.v_range.0)
    }

    fn closest_parameters(&self, point: Point3) -> (f64, f64) {
        let projected = self.plane.project_point(point);
        let d = projected - self.plane.origin;

        let u_val = d.dot(self.u_axis);
        let v_val = d.dot(self.v_axis);

        let u = (u_val - self.u_range.0) / (self.u_range.1 - self.u_range.0);
        let v = (v_val - self.v_range.0) / (self.v_range.1 - self.v_range.0);

        (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
    }
}

/// A spherical surface
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SphericalSurface {
    pub center: Point3,
    pub radius: f64,
}

impl SphericalSurface {
    pub fn new(center: Point3, radius: f64) -> Self {
        Self { center, radius }
    }
}

impl Surface for SphericalSurface {
    fn evaluate(&self, u: f64, v: f64) -> Point3 {
        let theta = u * std::f64::consts::TAU; // Azimuthal angle
        let phi = v * std::f64::consts::PI;    // Polar angle

        let x = self.radius * phi.sin() * theta.cos();
        let y = self.radius * phi.sin() * theta.sin();
        let z = self.radius * phi.cos();

        self.center + Vector3::new(x, y, z)
    }

    fn normal(&self, u: f64, v: f64) -> Vector3 {
        let p = self.evaluate(u, v);
        (p - self.center).normalize_or_zero()
    }

    fn du(&self, u: f64, v: f64) -> Vector3 {
        let theta = u * std::f64::consts::TAU;
        let phi = v * std::f64::consts::PI;

        let dx = -self.radius * phi.sin() * theta.sin() * std::f64::consts::TAU;
        let dy = self.radius * phi.sin() * theta.cos() * std::f64::consts::TAU;
        let dz = 0.0;

        Vector3::new(dx, dy, dz)
    }

    fn dv(&self, u: f64, v: f64) -> Vector3 {
        let theta = u * std::f64::consts::TAU;
        let phi = v * std::f64::consts::PI;

        let dx = self.radius * phi.cos() * theta.cos() * std::f64::consts::PI;
        let dy = self.radius * phi.cos() * theta.sin() * std::f64::consts::PI;
        let dz = -self.radius * phi.sin() * std::f64::consts::PI;

        Vector3::new(dx, dy, dz)
    }

    fn area(&self) -> f64 {
        4.0 * std::f64::consts::PI * self.radius * self.radius
    }

    fn closest_parameters(&self, point: Point3) -> (f64, f64) {
        let d = (point - self.center).normalize_or_zero();

        let phi = d.z.acos();
        let theta = d.y.atan2(d.x);

        let u = (theta + std::f64::consts::PI) / std::f64::consts::TAU;
        let v = phi / std::f64::consts::PI;

        (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
    }
}

/// A cylindrical surface
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CylindricalSurface {
    pub center: Point3,
    pub axis: Vector3,
    pub radius: f64,
    pub height: f64,
}

impl CylindricalSurface {
    pub fn new(center: Point3, axis: Vector3, radius: f64, height: f64) -> Self {
        Self {
            center,
            axis: axis.normalize_or_zero(),
            radius,
            height,
        }
    }

    fn local_frame(&self) -> (Vector3, Vector3) {
        let u = if self.axis.x.abs() < 0.9 {
            Vector3::X.cross(self.axis).normalize()
        } else {
            Vector3::Y.cross(self.axis).normalize()
        };
        let v = self.axis.cross(u);
        (u, v)
    }
}

impl Surface for CylindricalSurface {
    fn evaluate(&self, u: f64, v: f64) -> Point3 {
        let theta = u * std::f64::consts::TAU;
        let h = v * self.height;

        let (x_axis, y_axis) = self.local_frame();
        let radial = x_axis * theta.cos() + y_axis * theta.sin();

        self.center + radial * self.radius + self.axis * h
    }

    fn normal(&self, u: f64, _v: f64) -> Vector3 {
        let theta = u * std::f64::consts::TAU;
        let (x_axis, y_axis) = self.local_frame();
        x_axis * theta.cos() + y_axis * theta.sin()
    }

    fn du(&self, u: f64, _v: f64) -> Vector3 {
        let theta = u * std::f64::consts::TAU;
        let (x_axis, y_axis) = self.local_frame();
        (-x_axis * theta.sin() + y_axis * theta.cos()) * self.radius * std::f64::consts::TAU
    }

    fn dv(&self, _u: f64, _v: f64) -> Vector3 {
        self.axis * self.height
    }

    fn area(&self) -> f64 {
        std::f64::consts::TAU * self.radius * self.height
    }

    fn closest_parameters(&self, point: Point3) -> (f64, f64) {
        let d = point - self.center;
        let h = d.dot(self.axis);
        let radial = d - self.axis * h;

        let (x_axis, y_axis) = self.local_frame();
        let theta = radial.dot(y_axis).atan2(radial.dot(x_axis));

        let u = (theta + std::f64::consts::PI) / std::f64::consts::TAU;
        let v = h / self.height;

        (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
    }
}

/// A NURBS surface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NurbsSurface {
    pub degree_u: usize,
    pub degree_v: usize,
    pub control_points: Vec<Vec<Point3>>,
    pub weights: Vec<Vec<f64>>,
    pub knots_u: Vec<f64>,
    pub knots_v: Vec<f64>,
}

impl NurbsSurface {
    pub fn new(
        degree_u: usize,
        degree_v: usize,
        control_points: Vec<Vec<Point3>>,
        weights: Vec<Vec<f64>>,
        knots_u: Vec<f64>,
        knots_v: Vec<f64>,
    ) -> Self {
        Self {
            degree_u,
            degree_v,
            control_points,
            weights,
            knots_u,
            knots_v,
        }
    }

    /// Evaluate basis function using de Boor
    fn basis(&self, span: usize, t: f64, degree: usize, knots: &[f64]) -> Vec<f64> {
        let mut n = vec![0.0; degree + 1];
        n[0] = 1.0;

        for j in 1..=degree {
            let mut saved = 0.0;
            for r in 0..j {
                let left = t - knots[span + 1 - j + r];
                let right = knots[span + 1 + r] - t;
                let temp = n[r] / (right + left);
                n[r] = saved + right * temp;
                saved = left * temp;
            }
            n[j] = saved;
        }

        n
    }

    fn find_span(&self, t: f64, n_ctrl: usize, degree: usize, knots: &[f64]) -> usize {
        if t >= knots[n_ctrl] {
            return n_ctrl - 1;
        }
        if t <= knots[degree] {
            return degree;
        }

        let mut low = degree;
        let mut high = n_ctrl;
        let mut mid = (low + high) / 2;

        while t < knots[mid] || t >= knots[mid + 1] {
            if t < knots[mid] {
                high = mid;
            } else {
                low = mid;
            }
            mid = (low + high) / 2;
        }

        mid
    }
}

impl Surface for NurbsSurface {
    fn evaluate(&self, u: f64, v: f64) -> Point3 {
        let n_u = self.control_points.len();
        let n_v = self.control_points[0].len();

        let u_mapped = self.knots_u[self.degree_u] + u * (self.knots_u[n_u] - self.knots_u[self.degree_u]);
        let v_mapped = self.knots_v[self.degree_v] + v * (self.knots_v[n_v] - self.knots_v[self.degree_v]);

        let span_u = self.find_span(u_mapped, n_u, self.degree_u, &self.knots_u);
        let span_v = self.find_span(v_mapped, n_v, self.degree_v, &self.knots_v);

        let basis_u = self.basis(span_u, u_mapped, self.degree_u, &self.knots_u);
        let basis_v = self.basis(span_v, v_mapped, self.degree_v, &self.knots_v);

        let mut point = Point3::ZERO;
        let mut weight = 0.0;

        for i in 0..=self.degree_u {
            for j in 0..=self.degree_v {
                let idx_u = span_u - self.degree_u + i;
                let idx_v = span_v - self.degree_v + j;

                let w = self.weights[idx_u][idx_v] * basis_u[i] * basis_v[j];
                point += self.control_points[idx_u][idx_v] * w;
                weight += w;
            }
        }

        point / weight
    }

    fn normal(&self, u: f64, v: f64) -> Vector3 {
        let du = self.du(u, v);
        let dv = self.dv(u, v);
        du.cross(dv).normalize_or_zero()
    }

    fn du(&self, u: f64, v: f64) -> Vector3 {
        let h = 0.0001;
        let p0 = self.evaluate((u - h).max(0.0), v);
        let p1 = self.evaluate((u + h).min(1.0), v);
        (p1 - p0) / (2.0 * h)
    }

    fn dv(&self, u: f64, v: f64) -> Vector3 {
        let h = 0.0001;
        let p0 = self.evaluate(u, (v - h).max(0.0));
        let p1 = self.evaluate(u, (v + h).min(1.0));
        (p1 - p0) / (2.0 * h)
    }

    fn area(&self) -> f64 {
        // Numerical integration
        const SAMPLES: usize = 20;
        let mut area = 0.0;
        let step = 1.0 / SAMPLES as f64;

        for i in 0..SAMPLES {
            for j in 0..SAMPLES {
                let u = (i as f64 + 0.5) * step;
                let v = (j as f64 + 0.5) * step;

                let du = self.du(u, v);
                let dv = self.dv(u, v);
                area += du.cross(dv).length() * step * step;
            }
        }

        area
    }

    fn closest_parameters(&self, point: Point3) -> (f64, f64) {
        // Grid search followed by refinement
        const SAMPLES: usize = 10;
        let mut best_u = 0.5;
        let mut best_v = 0.5;
        let mut best_dist = f64::MAX;

        for i in 0..=SAMPLES {
            for j in 0..=SAMPLES {
                let u = i as f64 / SAMPLES as f64;
                let v = j as f64 / SAMPLES as f64;
                let p = self.evaluate(u, v);
                let dist = p.distance_squared(point);
                if dist < best_dist {
                    best_dist = dist;
                    best_u = u;
                    best_v = v;
                }
            }
        }

        (best_u, best_v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::DVec3;

    #[test]
    fn test_planar_surface() {
        let plane = Plane::xy();
        let surface = PlanarSurface::new(plane, (0.0, 1.0), (0.0, 1.0));

        let p = surface.evaluate(0.5, 0.5);
        assert!((p.x - 0.5).abs() < TOLERANCE);
        assert!((p.y - 0.5).abs() < TOLERANCE);
        assert!(p.z.abs() < TOLERANCE);
    }

    #[test]
    fn test_sphere_surface() {
        let surface = SphericalSurface::new(DVec3::ZERO, 1.0);
        let area = surface.area();
        let expected = 4.0 * std::f64::consts::PI;
        assert!((area - expected).abs() < 0.01);
    }
}
