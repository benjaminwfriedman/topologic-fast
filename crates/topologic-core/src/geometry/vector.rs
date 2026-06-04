//! Vector operations

use super::{Vector3, TOLERANCE};

/// Extended vector operations
pub trait VectorOps {
    /// Check if this vector is approximately zero
    fn is_zero(&self) -> bool;

    /// Check if two vectors are parallel
    fn is_parallel(&self, other: Vector3) -> bool;

    /// Check if two vectors are perpendicular
    fn is_perpendicular(&self, other: Vector3) -> bool;

    /// Get the angle between two vectors in radians
    fn angle_to(&self, other: Vector3) -> f64;

    /// Get the signed angle between two vectors around an axis
    fn signed_angle(&self, other: Vector3, axis: Vector3) -> f64;

    /// Reflect this vector about a normal
    fn reflect(&self, normal: Vector3) -> Vector3;

    /// Rotate this vector around an axis by an angle (radians)
    fn rotate_around(&self, axis: Vector3, angle: f64) -> Vector3;
}

impl VectorOps for Vector3 {
    #[inline]
    fn is_zero(&self) -> bool {
        self.length_squared() < TOLERANCE * TOLERANCE
    }

    #[inline]
    fn is_parallel(&self, other: Vector3) -> bool {
        self.cross(other).length_squared() < TOLERANCE * TOLERANCE
    }

    #[inline]
    fn is_perpendicular(&self, other: Vector3) -> bool {
        self.dot(other).abs() < TOLERANCE
    }

    #[inline]
    fn angle_to(&self, other: Vector3) -> f64 {
        let denom = (self.length_squared() * other.length_squared()).sqrt();
        if denom < TOLERANCE {
            return 0.0;
        }
        let cos_angle = (self.dot(other) / denom).clamp(-1.0, 1.0);
        cos_angle.acos()
    }

    #[inline]
    fn signed_angle(&self, other: Vector3, axis: Vector3) -> f64 {
        let cross = self.cross(other);
        let dot = self.dot(other);
        let angle = cross.length().atan2(dot);

        if cross.dot(axis) < 0.0 {
            -angle
        } else {
            angle
        }
    }

    #[inline]
    fn reflect(&self, normal: Vector3) -> Vector3 {
        *self - normal * 2.0 * self.dot(normal)
    }

    fn rotate_around(&self, axis: Vector3, angle: f64) -> Vector3 {
        let axis = axis.normalize_or_zero();
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // Rodrigues' rotation formula
        *self * cos_a + axis.cross(*self) * sin_a + axis * axis.dot(*self) * (1.0 - cos_a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::DVec3;
    use std::f64::consts::PI;

    #[test]
    fn test_is_parallel() {
        let v1 = DVec3::new(1.0, 0.0, 0.0);
        let v2 = DVec3::new(2.0, 0.0, 0.0);
        let v3 = DVec3::new(0.0, 1.0, 0.0);

        assert!(v1.is_parallel(v2));
        assert!(!v1.is_parallel(v3));
    }

    #[test]
    fn test_angle_to() {
        let v1 = DVec3::X;
        let v2 = DVec3::Y;

        let angle = v1.angle_to(v2);
        assert!((angle - PI / 2.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_rotate_around() {
        let v = DVec3::X;
        let rotated = v.rotate_around(DVec3::Z, PI / 2.0);

        assert!((rotated.x).abs() < TOLERANCE);
        assert!((rotated.y - 1.0).abs() < TOLERANCE);
    }
}
