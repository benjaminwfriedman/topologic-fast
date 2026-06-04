//! Geometry primitives with SIMD acceleration
//!
//! Uses glam for SIMD-accelerated vector math and nalgebra for advanced
//! linear algebra operations.

mod point;
mod vector;
mod transform;
mod plane;
mod curve;
mod surface;

pub use point::*;
pub use vector::*;
pub use transform::*;
pub use plane::*;
pub use curve::*;
pub use surface::*;

use glam::{DVec3, DMat4, DAffine3};

/// Tolerance for geometric comparisons
pub const TOLERANCE: f64 = 1e-10;

/// Point in 3D space (SIMD-optimized)
pub type Point3 = DVec3;

/// Vector in 3D space (SIMD-optimized)
pub type Vector3 = DVec3;

/// 4x4 transformation matrix
pub type Matrix4 = DMat4;

/// Check if two values are approximately equal
#[inline(always)]
pub fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < TOLERANCE
}

/// Check if a value is approximately zero
#[inline(always)]
pub fn approx_zero(a: f64) -> bool {
    a.abs() < TOLERANCE
}

/// Compute the distance between two points
#[inline(always)]
pub fn distance(a: Point3, b: Point3) -> f64 {
    a.distance(b)
}

/// Compute the squared distance between two points (faster, avoids sqrt)
#[inline(always)]
pub fn distance_squared(a: Point3, b: Point3) -> f64 {
    a.distance_squared(b)
}

/// Linear interpolation between two points
#[inline(always)]
pub fn lerp(a: Point3, b: Point3, t: f64) -> Point3 {
    a.lerp(b, t)
}

/// Compute the centroid of a set of points
pub fn centroid(points: &[Point3]) -> Point3 {
    if points.is_empty() {
        return Point3::ZERO;
    }
    let sum: Point3 = points.iter().copied().sum();
    sum / points.len() as f64
}

/// Compute the bounding box of a set of points
pub fn bounding_box(points: &[Point3]) -> Option<(Point3, Point3)> {
    if points.is_empty() {
        return None;
    }

    let mut min = points[0];
    let mut max = points[0];

    for &p in &points[1..] {
        min = min.min(p);
        max = max.max(p);
    }

    Some((min, max))
}

/// Compute the normal of a triangle
#[inline(always)]
pub fn triangle_normal(a: Point3, b: Point3, c: Point3) -> Vector3 {
    (b - a).cross(c - a).normalize_or_zero()
}

/// Compute the area of a triangle
#[inline(always)]
pub fn triangle_area(a: Point3, b: Point3, c: Point3) -> f64 {
    (b - a).cross(c - a).length() * 0.5
}

/// Check if three points are collinear
#[inline(always)]
pub fn collinear(a: Point3, b: Point3, c: Point3) -> bool {
    approx_zero((b - a).cross(c - a).length_squared())
}

/// Check if four points are coplanar
pub fn coplanar(a: Point3, b: Point3, c: Point3, d: Point3) -> bool {
    let v1 = b - a;
    let v2 = c - a;
    let v3 = d - a;
    approx_zero(v1.dot(v2.cross(v3)))
}
