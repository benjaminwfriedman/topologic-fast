//! Bounding Volume Hierarchy for fast spatial queries

use super::*;
use crate::topology::*;
use rayon::prelude::*;

/// BVH tree node
pub struct BvhNode<T> {
    pub aabb: Aabb,
    pub left: Option<Box<BvhNode<T>>>,
    pub right: Option<Box<BvhNode<T>>>,
    pub items: Vec<T>,
}

impl<T: Clone + Send + Sync> BvhNode<T> {
    /// Build a BVH from items with their AABBs
    pub fn build<F>(items: &[(T, Aabb)], get_centroid: F, max_leaf_size: usize) -> Option<Self>
    where
        F: Fn(&T) -> Point3 + Send + Sync + Copy,
    {
        if items.is_empty() {
            return None;
        }

        // Compute bounding box of all items
        let aabb = items
            .iter()
            .map(|(_, aabb)| aabb)
            .fold(items[0].1, |acc, aabb| acc.merge(aabb));

        // If we have few enough items, make a leaf
        if items.len() <= max_leaf_size {
            return Some(Self {
                aabb,
                left: None,
                right: None,
                items: items.iter().map(|(item, _)| item.clone()).collect(),
            });
        }

        // Choose split axis (longest)
        let axis = aabb.longest_axis();

        // Sort items by centroid along the axis
        let mut sorted_items: Vec<_> = items.to_vec();
        sorted_items.sort_by(|a, b| {
            let ca = get_centroid(&a.0);
            let cb = get_centroid(&b.0);
            let va = match axis {
                0 => ca.x,
                1 => ca.y,
                _ => ca.z,
            };
            let vb = match axis {
                0 => cb.x,
                1 => cb.y,
                _ => cb.z,
            };
            va.partial_cmp(&vb).unwrap()
        });

        // Split in the middle
        let mid = sorted_items.len() / 2;
        let (left_items, right_items) = sorted_items.split_at(mid);

        Some(Self {
            aabb,
            left: Self::build(left_items, get_centroid, max_leaf_size).map(Box::new),
            right: Self::build(right_items, get_centroid, max_leaf_size).map(Box::new),
            items: Vec::new(),
        })
    }

    /// Query items that might intersect with an AABB
    pub fn query_aabb(&self, query: &Aabb) -> Vec<&T> {
        let mut results = Vec::new();
        self.query_aabb_recursive(query, &mut results);
        results
    }

    fn query_aabb_recursive<'a>(&'a self, query: &Aabb, results: &mut Vec<&'a T>) {
        if !self.aabb.intersects(query) {
            return;
        }

        // Add leaf items
        for item in &self.items {
            results.push(item);
        }

        // Recurse into children
        if let Some(ref left) = self.left {
            left.query_aabb_recursive(query, results);
        }
        if let Some(ref right) = self.right {
            right.query_aabb_recursive(query, results);
        }
    }

    /// Query items that might be hit by a ray
    pub fn query_ray(&self, origin: Point3, direction: Vector3) -> Vec<&T> {
        let mut results = Vec::new();
        self.query_ray_recursive(origin, direction, &mut results);
        results
    }

    fn query_ray_recursive<'a>(&'a self, origin: Point3, direction: Vector3, results: &mut Vec<&'a T>) {
        if self.aabb.intersect_ray(origin, direction).is_none() {
            return;
        }

        // Add leaf items
        for item in &self.items {
            results.push(item);
        }

        // Recurse into children
        if let Some(ref left) = self.left {
            left.query_ray_recursive(origin, direction, results);
        }
        if let Some(ref right) = self.right {
            right.query_ray_recursive(origin, direction, results);
        }
    }

    /// Query the nearest item to a point
    pub fn query_nearest(&self, point: Point3) -> Option<(&T, f64)> {
        let mut best: Option<(&T, f64)> = None;
        self.query_nearest_recursive(point, &mut best);
        best
    }

    fn query_nearest_recursive<'a>(&'a self, point: Point3, best: &mut Option<(&'a T, f64)>) {
        // Early exit if this node's AABB is farther than best
        let node_dist = self.aabb.distance_to_point(point);
        if let Some((_, best_dist)) = best {
            if node_dist > *best_dist {
                return;
            }
        }

        // Check leaf items
        for item in &self.items {
            // Note: The actual distance calculation depends on the item type
            // For now, we use the AABB distance as a proxy
            let dist = node_dist;
            match best {
                Some((_, best_dist)) if dist < *best_dist => {
                    *best = Some((item, dist));
                }
                None => {
                    *best = Some((item, dist));
                }
                _ => {}
            }
        }

        // Recurse into children
        if let Some(ref left) = self.left {
            left.query_nearest_recursive(point, best);
        }
        if let Some(ref right) = self.right {
            right.query_nearest_recursive(point, best);
        }
    }
}

/// BVH tree specifically for faces
pub struct BvhTree {
    root: Option<BvhNode<FaceHandle>>,
}

impl BvhTree {
    /// Build a BVH from faces in a topology store
    pub fn build(store: &TopologyStore, faces: &[FaceHandle]) -> Self {
        let items: Vec<_> = faces
            .iter()
            .map(|face| {
                let (min, max) = Face::bounding_box(store, *face);
                (*face, Aabb::new(min, max))
            })
            .collect();

        let get_centroid = |face: &FaceHandle| Face::center_of_mass(store, *face);

        Self {
            root: BvhNode::build(&items, get_centroid, 4),
        }
    }

    /// Query faces that might intersect an AABB
    pub fn query_aabb(&self, query: &Aabb) -> Vec<FaceHandle> {
        match &self.root {
            Some(root) => root.query_aabb(query).into_iter().copied().collect(),
            None => Vec::new(),
        }
    }

    /// Query faces that might be hit by a ray
    pub fn query_ray(&self, origin: Point3, direction: Vector3) -> Vec<FaceHandle> {
        match &self.root {
            Some(root) => root.query_ray(origin, direction).into_iter().copied().collect(),
            None => Vec::new(),
        }
    }

    /// Ray cast and find the closest hit
    pub fn ray_cast(
        &self,
        store: &TopologyStore,
        origin: Point3,
        direction: Vector3,
    ) -> Option<(FaceHandle, Point3, f64)> {
        let candidates = self.query_ray(origin, direction);
        SpatialQuery::ray_cast(store, origin, direction, &candidates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bvh_query() {
        let store = TopologyStore::new();

        // Create a few faces
        let face1 = Face::rectangle(&store, Point3::ZERO, 1.0, 1.0);
        let face2 = Face::rectangle(&store, Point3::new(2.0, 0.0, 0.0), 1.0, 1.0);
        let face3 = Face::rectangle(&store, Point3::new(4.0, 0.0, 0.0), 1.0, 1.0);

        let bvh = BvhTree::build(&store, &[face1, face2, face3]);

        // Query AABB that should only hit face1 and face2
        let query = Aabb::new(Point3::ZERO, Point3::new(2.5, 1.0, 0.0));
        let hits = bvh.query_aabb(&query);

        assert!(hits.contains(&face1));
        assert!(hits.contains(&face2));
    }

    #[test]
    fn test_bvh_ray_cast() {
        let store = TopologyStore::new();

        // Create a face at z=0
        let face = Face::rectangle(&store, Point3::new(-0.5, -0.5, 0.0), 1.0, 1.0);

        let bvh = BvhTree::build(&store, &[face]);

        // Ray from above pointing down
        let result = bvh.ray_cast(&store, Point3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 0.0, -1.0));

        assert!(result.is_some());
        let (hit_face, _, t) = result.unwrap();
        assert_eq!(hit_face, face);
        assert!((t - 1.0).abs() < TOLERANCE);
    }
}
