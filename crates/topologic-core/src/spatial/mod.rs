//! Spatial indexing and queries
//!
//! Provides BVH (Bounding Volume Hierarchy) and other spatial data structures
//! for fast geometric queries.

mod bvh;
mod octree;
mod kdtree;

pub use bvh::*;
pub use octree::*;
pub use kdtree::*;

use crate::topology::*;
use crate::geometry::{Point3, Vector3, TOLERANCE};
use rayon::prelude::*;

/// Axis-aligned bounding box
#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min: Point3,
    pub max: Point3,
}

impl Aabb {
    /// Create a new AABB
    pub fn new(min: Point3, max: Point3) -> Self {
        Self { min, max }
    }

    /// Create an AABB from a point
    pub fn from_point(point: Point3) -> Self {
        Self {
            min: point,
            max: point,
        }
    }

    /// Create an AABB from multiple points
    pub fn from_points(points: &[Point3]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let mut min = points[0];
        let mut max = points[0];

        for p in &points[1..] {
            min = min.min(*p);
            max = max.max(*p);
        }

        Some(Self { min, max })
    }

    /// Get the center of the AABB
    pub fn center(&self) -> Point3 {
        (self.min + self.max) * 0.5
    }

    /// Get the extents (half-widths) of the AABB
    pub fn extents(&self) -> Vector3 {
        (self.max - self.min) * 0.5
    }

    /// Get the size of the AABB
    pub fn size(&self) -> Vector3 {
        self.max - self.min
    }

    /// Get the surface area of the AABB
    pub fn surface_area(&self) -> f64 {
        let s = self.size();
        2.0 * (s.x * s.y + s.y * s.z + s.z * s.x)
    }

    /// Get the volume of the AABB
    pub fn volume(&self) -> f64 {
        let s = self.size();
        s.x * s.y * s.z
    }

    /// Check if this AABB contains a point
    pub fn contains_point(&self, point: Point3) -> bool {
        point.x >= self.min.x - TOLERANCE
            && point.x <= self.max.x + TOLERANCE
            && point.y >= self.min.y - TOLERANCE
            && point.y <= self.max.y + TOLERANCE
            && point.z >= self.min.z - TOLERANCE
            && point.z <= self.max.z + TOLERANCE
    }

    /// Check if this AABB intersects another
    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min.x <= other.max.x + TOLERANCE
            && self.max.x >= other.min.x - TOLERANCE
            && self.min.y <= other.max.y + TOLERANCE
            && self.max.y >= other.min.y - TOLERANCE
            && self.min.z <= other.max.z + TOLERANCE
            && self.max.z >= other.min.z - TOLERANCE
    }

    /// Merge this AABB with another
    pub fn merge(&self, other: &Aabb) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    /// Expand this AABB by a margin
    pub fn expand(&self, margin: f64) -> Self {
        Self {
            min: self.min - Vector3::splat(margin),
            max: self.max + Vector3::splat(margin),
        }
    }

    /// Get the longest axis (0=x, 1=y, 2=z)
    pub fn longest_axis(&self) -> usize {
        let s = self.size();
        if s.x >= s.y && s.x >= s.z {
            0
        } else if s.y >= s.z {
            1
        } else {
            2
        }
    }

    /// Compute the distance from a point to this AABB
    pub fn distance_to_point(&self, point: Point3) -> f64 {
        let clamped = point.clamp(self.min, self.max);
        point.distance(clamped)
    }

    /// Ray-AABB intersection test
    pub fn intersect_ray(&self, origin: Point3, direction: Vector3) -> Option<f64> {
        let inv_dir = Vector3::new(1.0 / direction.x, 1.0 / direction.y, 1.0 / direction.z);

        let t1 = (self.min.x - origin.x) * inv_dir.x;
        let t2 = (self.max.x - origin.x) * inv_dir.x;
        let t3 = (self.min.y - origin.y) * inv_dir.y;
        let t4 = (self.max.y - origin.y) * inv_dir.y;
        let t5 = (self.min.z - origin.z) * inv_dir.z;
        let t6 = (self.max.z - origin.z) * inv_dir.z;

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        if tmax >= tmin && tmax >= 0.0 {
            Some(if tmin > 0.0 { tmin } else { tmax })
        } else {
            None
        }
    }
}

/// Spatial query interface for topology
pub struct SpatialQuery;

impl SpatialQuery {
    /// Find all vertices within a sphere
    pub fn vertices_in_sphere(
        store: &TopologyStore,
        center: Point3,
        radius: f64,
    ) -> Vec<VertexHandle> {
        let vertices = store.vertices.read();
        let radius_sq = radius * radius;

        vertices
            .iter()
            .enumerate()
            .filter(|(_, v)| v.point.distance_squared(center) <= radius_sq)
            .map(|(i, v)| VertexHandle {
                index: ArenaIndex::new(i as u32),
                id: v.id,
            })
            .collect()
    }

    /// Find all faces intersecting a bounding box
    pub fn faces_in_aabb(
        store: &TopologyStore,
        aabb: &Aabb,
    ) -> Vec<FaceHandle> {
        let faces = store.faces.read();

        faces
            .iter()
            .enumerate()
            .filter(|(i, _)| {
                let handle = FaceHandle {
                    index: ArenaIndex::new(*i as u32),
                    id: faces[*i].id,
                };
                let (min, max) = Face::bounding_box(store, handle);
                let face_aabb = Aabb::new(min, max);
                aabb.intersects(&face_aabb)
            })
            .map(|(i, f)| FaceHandle {
                index: ArenaIndex::new(i as u32),
                id: f.id,
            })
            .collect()
    }

    /// Find the nearest vertex to a point
    pub fn nearest_vertex(
        store: &TopologyStore,
        point: Point3,
    ) -> Option<VertexHandle> {
        let vertices = store.vertices.read();
        if vertices.is_empty() {
            return None;
        }

        let (idx, _) = vertices
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let da = a.point.distance_squared(point);
                let db = b.point.distance_squared(point);
                da.partial_cmp(&db).unwrap()
            })?;

        Some(VertexHandle {
            index: ArenaIndex::new(idx as u32),
            id: vertices[idx].id,
        })
    }

    /// Find the nearest point on any face
    pub fn nearest_point_on_faces(
        store: &TopologyStore,
        point: Point3,
        faces: &[FaceHandle],
    ) -> Option<(FaceHandle, Point3)> {
        if faces.is_empty() {
            return None;
        }

        faces
            .par_iter()
            .map(|face| {
                // Project point onto face plane
                let normal = Face::normal(store, *face).unwrap_or(Vector3::Z);
                let center = Face::center_of_mass(store, *face);
                let projected = point - normal * (point - center).dot(normal);

                (*face, projected, point.distance_squared(projected))
            })
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
            .map(|(face, projected, _)| (face, projected))
    }

    /// Ray cast against faces
    pub fn ray_cast(
        store: &TopologyStore,
        origin: Point3,
        direction: Vector3,
        faces: &[FaceHandle],
    ) -> Option<(FaceHandle, Point3, f64)> {
        let direction = direction.normalize_or_zero();

        faces
            .par_iter()
            .filter_map(|face| {
                let normal = Face::normal(store, *face)?;
                let center = Face::center_of_mass(store, *face);

                // Ray-plane intersection
                let denom = direction.dot(normal);
                if denom.abs() < TOLERANCE {
                    return None;
                }

                let t = (center - origin).dot(normal) / denom;
                if t < 0.0 {
                    return None;
                }

                let hit_point = origin + direction * t;

                // Check if hit point is inside face
                if Face::contains_point(store, *face, hit_point) {
                    Some((*face, hit_point, t))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_contains() {
        let aabb = Aabb::new(Point3::ZERO, Point3::new(1.0, 1.0, 1.0));

        assert!(aabb.contains_point(Point3::new(0.5, 0.5, 0.5)));
        assert!(!aabb.contains_point(Point3::new(2.0, 0.5, 0.5)));
    }

    #[test]
    fn test_aabb_intersects() {
        let aabb1 = Aabb::new(Point3::ZERO, Point3::new(1.0, 1.0, 1.0));
        let aabb2 = Aabb::new(Point3::new(0.5, 0.5, 0.5), Point3::new(1.5, 1.5, 1.5));
        let aabb3 = Aabb::new(Point3::new(2.0, 2.0, 2.0), Point3::new(3.0, 3.0, 3.0));

        assert!(aabb1.intersects(&aabb2));
        assert!(!aabb1.intersects(&aabb3));
    }

    #[test]
    fn test_aabb_ray_intersect() {
        let aabb = Aabb::new(Point3::ZERO, Point3::new(1.0, 1.0, 1.0));

        let hit = aabb.intersect_ray(Point3::new(-1.0, 0.5, 0.5), Vector3::X);
        assert!(hit.is_some());
        assert!((hit.unwrap() - 1.0).abs() < TOLERANCE);

        let miss = aabb.intersect_ray(Point3::new(-1.0, 2.0, 0.5), Vector3::X);
        assert!(miss.is_none());
    }
}
