//! Transformation matrices and operations

use glam::{DMat4, DVec3, DQuat, DAffine3};
use super::{Point3, Vector3, Matrix4};

/// Transformation builder for composing transforms
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    matrix: DMat4,
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

impl Transform {
    /// Create an identity transform
    #[inline]
    pub const fn identity() -> Self {
        Self {
            matrix: DMat4::IDENTITY,
        }
    }

    /// Create from a matrix
    #[inline]
    pub const fn from_matrix(matrix: DMat4) -> Self {
        Self { matrix }
    }

    /// Get the underlying matrix
    #[inline]
    pub const fn matrix(&self) -> DMat4 {
        self.matrix
    }

    /// Create a translation transform
    #[inline]
    pub fn translate(offset: Vector3) -> Self {
        Self {
            matrix: DMat4::from_translation(offset),
        }
    }

    /// Create a rotation transform from axis and angle (radians)
    #[inline]
    pub fn rotate(axis: Vector3, angle: f64) -> Self {
        Self {
            matrix: DMat4::from_axis_angle(axis.normalize_or_zero(), angle),
        }
    }

    /// Create a rotation from quaternion
    #[inline]
    pub fn from_quat(quat: DQuat) -> Self {
        Self {
            matrix: DMat4::from_quat(quat),
        }
    }

    /// Create a uniform scale transform
    #[inline]
    pub fn scale_uniform(factor: f64) -> Self {
        Self {
            matrix: DMat4::from_scale(DVec3::splat(factor)),
        }
    }

    /// Create a non-uniform scale transform
    #[inline]
    pub fn scale(factors: Vector3) -> Self {
        Self {
            matrix: DMat4::from_scale(factors),
        }
    }

    /// Create a transform that looks from `eye` towards `target`
    #[inline]
    pub fn look_at(eye: Point3, target: Point3, up: Vector3) -> Self {
        Self {
            matrix: DMat4::look_at_rh(eye, target, up).inverse(),
        }
    }

    /// Create a mirror transform about a plane
    pub fn mirror(plane_origin: Point3, plane_normal: Vector3) -> Self {
        let n = plane_normal.normalize_or_zero();
        let d = -plane_origin.dot(n);

        // Reflection matrix: I - 2*n*n^T
        let matrix = DMat4::from_cols(
            (DVec3::X - 2.0 * n.x * n).extend(-2.0 * n.x * d),
            (DVec3::Y - 2.0 * n.y * n).extend(-2.0 * n.y * d),
            (DVec3::Z - 2.0 * n.z * n).extend(-2.0 * n.z * d),
            DVec3::ZERO.extend(1.0),
        );

        Self { matrix }
    }

    /// Compose this transform with another (this * other)
    #[inline]
    pub fn then(&self, other: Transform) -> Self {
        Self {
            matrix: self.matrix * other.matrix,
        }
    }

    /// Apply the inverse of another transform
    #[inline]
    pub fn inverse(&self) -> Self {
        Self {
            matrix: self.matrix.inverse(),
        }
    }

    /// Transform a point
    #[inline]
    pub fn transform_point(&self, point: Point3) -> Point3 {
        self.matrix.transform_point3(point)
    }

    /// Transform a vector (ignores translation)
    #[inline]
    pub fn transform_vector(&self, vector: Vector3) -> Vector3 {
        self.matrix.transform_vector3(vector)
    }

    /// Transform a normal (uses inverse transpose)
    #[inline]
    pub fn transform_normal(&self, normal: Vector3) -> Vector3 {
        self.matrix.inverse().transpose().transform_vector3(normal).normalize_or_zero()
    }

    /// Transform multiple points in place
    #[inline]
    pub fn transform_points(&self, points: &mut [Point3]) {
        for point in points.iter_mut() {
            *point = self.transform_point(*point);
        }
    }
}

impl std::ops::Mul for Transform {
    type Output = Transform;

    #[inline]
    fn mul(self, rhs: Transform) -> Transform {
        self.then(rhs)
    }
}

impl std::ops::Mul<Point3> for Transform {
    type Output = Point3;

    #[inline]
    fn mul(self, rhs: Point3) -> Point3 {
        self.transform_point(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::TOLERANCE;

    #[test]
    fn test_translate() {
        let t = Transform::translate(DVec3::new(1.0, 2.0, 3.0));
        let p = DVec3::ZERO;
        let result = t.transform_point(p);

        assert!((result.x - 1.0).abs() < TOLERANCE);
        assert!((result.y - 2.0).abs() < TOLERANCE);
        assert!((result.z - 3.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_scale() {
        let t = Transform::scale_uniform(2.0);
        let p = DVec3::new(1.0, 1.0, 1.0);
        let result = t.transform_point(p);

        assert!((result.x - 2.0).abs() < TOLERANCE);
        assert!((result.y - 2.0).abs() < TOLERANCE);
        assert!((result.z - 2.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_compose() {
        let t1 = Transform::translate(DVec3::new(1.0, 0.0, 0.0));
        let t2 = Transform::scale_uniform(2.0);
        let combined = t1.then(t2);

        let p = DVec3::ZERO;
        let result = combined.transform_point(p);

        // First translate, then scale
        assert!((result.x - 2.0).abs() < TOLERANCE);
    }
}
