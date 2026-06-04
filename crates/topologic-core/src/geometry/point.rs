//! Point operations

use super::{Point3, Vector3, TOLERANCE};

/// Extended point operations
pub trait PointOps {
    /// Project this point onto a plane defined by origin and normal
    fn project_to_plane(&self, plane_origin: Point3, plane_normal: Vector3) -> Point3;

    /// Project this point onto a line defined by origin and direction
    fn project_to_line(&self, line_origin: Point3, line_direction: Vector3) -> Point3;

    /// Check if this point lies on a line segment
    fn on_segment(&self, a: Point3, b: Point3) -> bool;

    /// Get the parameter t where this point lies on line a->b
    fn parameter_on_line(&self, a: Point3, b: Point3) -> Option<f64>;
}

impl PointOps for Point3 {
    #[inline]
    fn project_to_plane(&self, plane_origin: Point3, plane_normal: Vector3) -> Point3 {
        let d = (*self - plane_origin).dot(plane_normal);
        *self - plane_normal * d
    }

    #[inline]
    fn project_to_line(&self, line_origin: Point3, line_direction: Vector3) -> Point3 {
        let v = *self - line_origin;
        let t = v.dot(line_direction) / line_direction.length_squared();
        line_origin + line_direction * t
    }

    #[inline]
    fn on_segment(&self, a: Point3, b: Point3) -> bool {
        let ab = b - a;
        let ap = *self - a;

        // Check if collinear
        let cross = ab.cross(ap);
        if cross.length_squared() > TOLERANCE * TOLERANCE {
            return false;
        }

        // Check if within segment bounds
        let t = ap.dot(ab) / ab.length_squared();
        t >= -TOLERANCE && t <= 1.0 + TOLERANCE
    }

    #[inline]
    fn parameter_on_line(&self, a: Point3, b: Point3) -> Option<f64> {
        let ab = b - a;
        let len_sq = ab.length_squared();
        if len_sq < TOLERANCE * TOLERANCE {
            return None;
        }
        Some((*self - a).dot(ab) / len_sq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::DVec3;

    #[test]
    fn test_project_to_plane() {
        let p = DVec3::new(1.0, 1.0, 1.0);
        let origin = DVec3::ZERO;
        let normal = DVec3::Z;

        let projected = p.project_to_plane(origin, normal);
        assert!((projected.z).abs() < TOLERANCE);
        assert!((projected.x - 1.0).abs() < TOLERANCE);
        assert!((projected.y - 1.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_on_segment() {
        let a = DVec3::ZERO;
        let b = DVec3::new(2.0, 0.0, 0.0);

        assert!(DVec3::new(1.0, 0.0, 0.0).on_segment(a, b));
        assert!(!DVec3::new(3.0, 0.0, 0.0).on_segment(a, b));
        assert!(!DVec3::new(1.0, 1.0, 0.0).on_segment(a, b));
    }
}
