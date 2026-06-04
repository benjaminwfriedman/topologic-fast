//! Vector utility class matching topologicpy Vector API
//!
//! Provides static methods for vector operations commonly used in
//! architectural and engineering applications.

use crate::geometry::{Point3, Vector3, TOLERANCE};
use std::f64::consts::PI;

/// Vector utility class with static methods
pub struct VectorUtil;

impl VectorUtil {
    /// Calculate the angle between two vectors in degrees
    pub fn angle(vec_a: [f64; 3], vec_b: [f64; 3]) -> f64 {
        let a = Vector3::new(vec_a[0], vec_a[1], vec_a[2]);
        let b = Vector3::new(vec_b[0], vec_b[1], vec_b[2]);

        let len_a = a.length();
        let len_b = b.length();

        if len_a < TOLERANCE || len_b < TOLERANCE {
            return 0.0;
        }

        let cos_angle = (a.dot(b) / (len_a * len_b)).clamp(-1.0, 1.0);
        cos_angle.acos().to_degrees()
    }

    /// Get the azimuth and altitude of a vector
    /// Returns a tuple (azimuth, altitude) in degrees
    pub fn azimuth_altitude(vec: [f64; 3], mantissa: u32) -> (f64, f64) {
        let v = Vector3::new(vec[0], vec[1], vec[2]);
        let len = v.length();

        if len < TOLERANCE {
            return (0.0, 0.0);
        }

        // Altitude: angle from XY plane
        let altitude = (v.z / len).asin().to_degrees();

        // Azimuth: angle from North (positive Y) clockwise
        let azimuth = if v.x.abs() < TOLERANCE && v.y.abs() < TOLERANCE {
            0.0
        } else {
            let mut az = v.x.atan2(v.y).to_degrees();
            if az < 0.0 {
                az += 360.0;
            }
            az
        };

        let mult = 10f64.powi(mantissa as i32);
        let azimuth = (azimuth * mult).round() / mult;
        let altitude = (altitude * mult).round() / mult;

        (azimuth, altitude)
    }

    /// Create a vector from azimuth and altitude (in degrees)
    pub fn by_azimuth_altitude(azimuth: f64, altitude: f64) -> [f64; 3] {
        let az_rad = azimuth.to_radians();
        let alt_rad = altitude.to_radians();

        let cos_alt = alt_rad.cos();
        let x = cos_alt * az_rad.sin();
        let y = cos_alt * az_rad.cos();
        let z = alt_rad.sin();

        [x, y, z]
    }

    /// Create a vector from coordinates
    pub fn by_coordinates(x: f64, y: f64, z: f64) -> [f64; 3] {
        [x, y, z]
    }

    /// Create a vector from two vertices (end - start)
    pub fn by_vertices(start: [f64; 3], end: [f64; 3]) -> [f64; 3] {
        [
            end[0] - start[0],
            end[1] - start[1],
            end[2] - start[2],
        ]
    }

    /// Get the compass angle of a vector (angle from North, clockwise)
    pub fn compass_angle(vec: [f64; 3], mantissa: u32) -> f64 {
        let (azimuth, _) = Self::azimuth_altitude(vec, mantissa);
        azimuth
    }

    /// Get the coordinates of a vector
    pub fn coordinates(vec: [f64; 3], output_type: &str) -> Vec<f64> {
        match output_type.to_lowercase().as_str() {
            "xy" => vec![vec[0], vec[1]],
            "xz" => vec![vec[0], vec[2]],
            "yz" => vec![vec[1], vec[2]],
            _ => vec![vec[0], vec[1], vec[2]], // "xyz" or default
        }
    }

    /// Cross product of two vectors
    pub fn cross(vec_a: [f64; 3], vec_b: [f64; 3]) -> [f64; 3] {
        let a = Vector3::new(vec_a[0], vec_a[1], vec_a[2]);
        let b = Vector3::new(vec_b[0], vec_b[1], vec_b[2]);
        let result = a.cross(b);
        [result.x, result.y, result.z]
    }

    /// Dot product of two vectors
    pub fn dot(vec_a: [f64; 3], vec_b: [f64; 3]) -> f64 {
        let a = Vector3::new(vec_a[0], vec_a[1], vec_a[2]);
        let b = Vector3::new(vec_b[0], vec_b[1], vec_b[2]);
        a.dot(b)
    }

    /// Get the Up vector (0, 0, 1)
    pub fn up() -> [f64; 3] {
        [0.0, 0.0, 1.0]
    }

    /// Get the Down vector (0, 0, -1)
    pub fn down() -> [f64; 3] {
        [0.0, 0.0, -1.0]
    }

    /// Get the North vector (0, 1, 0)
    pub fn north() -> [f64; 3] {
        [0.0, 1.0, 0.0]
    }

    /// Get the South vector (0, -1, 0)
    pub fn south() -> [f64; 3] {
        [0.0, -1.0, 0.0]
    }

    /// Get the East vector (1, 0, 0)
    pub fn east() -> [f64; 3] {
        [1.0, 0.0, 0.0]
    }

    /// Get the West vector (-1, 0, 0)
    pub fn west() -> [f64; 3] {
        [-1.0, 0.0, 0.0]
    }

    /// Check if two vectors are collinear
    pub fn is_collinear(vec_a: [f64; 3], vec_b: [f64; 3], tolerance: f64) -> bool {
        let a = Vector3::new(vec_a[0], vec_a[1], vec_a[2]);
        let b = Vector3::new(vec_b[0], vec_b[1], vec_b[2]);
        a.cross(b).length_squared() < tolerance * tolerance
    }

    /// Check if two vectors are parallel (same or opposite direction)
    pub fn is_parallel(vec_a: [f64; 3], vec_b: [f64; 3], tolerance: f64) -> bool {
        Self::is_collinear(vec_a, vec_b, tolerance)
    }

    /// Get the magnitude (length) of a vector
    pub fn magnitude(vec: [f64; 3]) -> f64 {
        let v = Vector3::new(vec[0], vec[1], vec[2]);
        v.length()
    }

    /// Multiply a vector by a scalar
    pub fn multiply(vec: [f64; 3], scalar: f64) -> [f64; 3] {
        [vec[0] * scalar, vec[1] * scalar, vec[2] * scalar]
    }

    /// Normalize a vector to unit length
    pub fn normalize(vec: [f64; 3]) -> [f64; 3] {
        let v = Vector3::new(vec[0], vec[1], vec[2]);
        let normalized = v.normalize_or_zero();
        [normalized.x, normalized.y, normalized.z]
    }

    /// Reverse (negate) a vector
    pub fn reverse(vec: [f64; 3]) -> [f64; 3] {
        [-vec[0], -vec[1], -vec[2]]
    }

    /// Set the magnitude of a vector
    pub fn set_magnitude(vec: [f64; 3], magnitude: f64) -> [f64; 3] {
        let normalized = Self::normalize(vec);
        Self::multiply(normalized, magnitude)
    }

    /// Add two vectors
    pub fn add(vec_a: [f64; 3], vec_b: [f64; 3]) -> [f64; 3] {
        [
            vec_a[0] + vec_b[0],
            vec_a[1] + vec_b[1],
            vec_a[2] + vec_b[2],
        ]
    }

    /// Subtract vec_b from vec_a
    pub fn subtract(vec_a: [f64; 3], vec_b: [f64; 3]) -> [f64; 3] {
        [
            vec_a[0] - vec_b[0],
            vec_a[1] - vec_b[1],
            vec_a[2] - vec_b[2],
        ]
    }

    /// Rotate a vector around an axis by an angle (in degrees)
    pub fn rotate(vec: [f64; 3], axis: [f64; 3], angle: f64) -> [f64; 3] {
        let v = Vector3::new(vec[0], vec[1], vec[2]);
        let ax = Vector3::new(axis[0], axis[1], axis[2]).normalize_or_zero();
        let angle_rad = angle.to_radians();

        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();

        // Rodrigues' rotation formula
        let rotated = v * cos_a + ax.cross(v) * sin_a + ax * ax.dot(v) * (1.0 - cos_a);
        [rotated.x, rotated.y, rotated.z]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_angle() {
        let angle = VectorUtil::angle([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((angle - 90.0).abs() < 0.001);
    }

    #[test]
    fn test_magnitude() {
        let mag = VectorUtil::magnitude([3.0, 4.0, 0.0]);
        assert!((mag - 5.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_normalize() {
        let norm = VectorUtil::normalize([3.0, 0.0, 0.0]);
        assert!((norm[0] - 1.0).abs() < TOLERANCE);
        assert!(norm[1].abs() < TOLERANCE);
        assert!(norm[2].abs() < TOLERANCE);
    }

    #[test]
    fn test_cross() {
        let result = VectorUtil::cross([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(result[0].abs() < TOLERANCE);
        assert!(result[1].abs() < TOLERANCE);
        assert!((result[2] - 1.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_azimuth_altitude() {
        let (az, alt) = VectorUtil::azimuth_altitude([1.0, 0.0, 0.0], 6);
        assert!((az - 90.0).abs() < 0.001);
        assert!(alt.abs() < 0.001);
    }
}
