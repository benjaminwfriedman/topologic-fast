//! Plane representation and operations

use super::{Point3, Vector3, TOLERANCE};
use serde::{Deserialize, Serialize};

/// A plane in 3D space defined by a point and normal
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Plane {
    pub origin: Point3,
    pub normal: Vector3,
}

impl Plane {
    /// Create a new plane from origin and normal
    #[inline]
    pub fn new(origin: Point3, normal: Vector3) -> Self {
        Self {
            origin,
            normal: normal.normalize_or_zero(),
        }
    }

    /// Create the XY plane
    #[inline]
    pub fn xy() -> Self {
        Self::new(Point3::ZERO, Vector3::Z)
    }

    /// Create the XZ plane
    #[inline]
    pub fn xz() -> Self {
        Self::new(Point3::ZERO, Vector3::Y)
    }

    /// Create the YZ plane
    #[inline]
    pub fn yz() -> Self {
        Self::new(Point3::ZERO, Vector3::X)
    }

    /// Create a plane from three points
    pub fn from_points(a: Point3, b: Point3, c: Point3) -> Option<Self> {
        let v1 = b - a;
        let v2 = c - a;
        let normal = v1.cross(v2);

        if normal.length_squared() < TOLERANCE * TOLERANCE {
            return None; // Points are collinear
        }

        Some(Self::new(a, normal))
    }

    /// Get the signed distance from a point to this plane
    #[inline]
    pub fn signed_distance(&self, point: Point3) -> f64 {
        (point - self.origin).dot(self.normal)
    }

    /// Get the absolute distance from a point to this plane
    #[inline]
    pub fn distance(&self, point: Point3) -> f64 {
        self.signed_distance(point).abs()
    }

    /// Project a point onto this plane
    #[inline]
    pub fn project_point(&self, point: Point3) -> Point3 {
        point - self.normal * self.signed_distance(point)
    }

    /// Check if a point lies on this plane
    #[inline]
    pub fn contains_point(&self, point: Point3) -> bool {
        self.distance(point) < TOLERANCE
    }

    /// Check which side of the plane a point is on
    /// Returns: 1 for positive side, -1 for negative, 0 for on plane
    #[inline]
    pub fn side(&self, point: Point3) -> i8 {
        let d = self.signed_distance(point);
        if d > TOLERANCE {
            1
        } else if d < -TOLERANCE {
            -1
        } else {
            0
        }
    }

    /// Intersect a line with this plane
    /// Returns the intersection point if it exists
    pub fn intersect_line(&self, line_origin: Point3, line_direction: Vector3) -> Option<Point3> {
        let denom = self.normal.dot(line_direction);
        if denom.abs() < TOLERANCE {
            return None; // Line is parallel to plane
        }

        let t = (self.origin - line_origin).dot(self.normal) / denom;
        Some(line_origin + line_direction * t)
    }

    /// Intersect a line segment with this plane
    pub fn intersect_segment(&self, a: Point3, b: Point3) -> Option<Point3> {
        let direction = b - a;
        let point = self.intersect_line(a, direction)?;

        // Check if intersection is within segment
        let t = if direction.x.abs() > TOLERANCE {
            (point.x - a.x) / direction.x
        } else if direction.y.abs() > TOLERANCE {
            (point.y - a.y) / direction.y
        } else {
            (point.z - a.z) / direction.z
        };

        if t >= -TOLERANCE && t <= 1.0 + TOLERANCE {
            Some(point)
        } else {
            None
        }
    }

    /// Intersect two planes, returning the line of intersection
    /// Returns (point on line, direction of line) if they intersect
    pub fn intersect_plane(&self, other: &Plane) -> Option<(Point3, Vector3)> {
        let direction = self.normal.cross(other.normal);

        if direction.length_squared() < TOLERANCE * TOLERANCE {
            return None; // Planes are parallel
        }

        let direction = direction.normalize();

        // Find a point on the line of intersection
        // Solve: p.dot(n1) = d1 and p.dot(n2) = d2
        let d1 = self.origin.dot(self.normal);
        let d2 = other.origin.dot(other.normal);

        let n1_dot_n2 = self.normal.dot(other.normal);
        let denom = 1.0 - n1_dot_n2 * n1_dot_n2;

        let c1 = (d1 - d2 * n1_dot_n2) / denom;
        let c2 = (d2 - d1 * n1_dot_n2) / denom;

        let point = self.normal * c1 + other.normal * c2;

        Some((point, direction))
    }

    /// Get a local coordinate frame for this plane
    /// Returns (u_axis, v_axis) where u cross v = normal
    pub fn local_frame(&self) -> (Vector3, Vector3) {
        let u = if self.normal.x.abs() < 0.9 {
            Vector3::X.cross(self.normal).normalize()
        } else {
            Vector3::Y.cross(self.normal).normalize()
        };
        let v = self.normal.cross(u);
        (u, v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::DVec3;

    #[test]
    fn test_signed_distance() {
        let plane = Plane::xy();
        assert!((plane.signed_distance(DVec3::new(0.0, 0.0, 5.0)) - 5.0).abs() < TOLERANCE);
        assert!((plane.signed_distance(DVec3::new(0.0, 0.0, -3.0)) + 3.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_intersect_line() {
        let plane = Plane::xy();
        let result = plane.intersect_line(DVec3::new(0.0, 0.0, 1.0), DVec3::new(0.0, 0.0, -1.0));
        assert!(result.is_some());
        let p = result.unwrap();
        assert!(p.z.abs() < TOLERANCE);
    }

    #[test]
    fn test_from_points() {
        let plane = Plane::from_points(
            DVec3::ZERO,
            DVec3::X,
            DVec3::Y,
        ).unwrap();
        assert!((plane.normal.z.abs() - 1.0).abs() < TOLERANCE);
    }
}
