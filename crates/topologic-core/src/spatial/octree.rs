//! Octree for spatial partitioning

use super::*;
use crate::topology::*;

/// Octree node
pub struct OctreeNode<T> {
    pub center: Point3,
    pub half_size: f64,
    pub children: Option<Box<[OctreeNode<T>; 8]>>,
    pub items: Vec<T>,
}

impl<T: Clone> OctreeNode<T> {
    /// Create a new octree node
    pub fn new(center: Point3, half_size: f64) -> Self {
        Self {
            center,
            half_size,
            children: None,
            items: Vec::new(),
        }
    }

    /// Get the AABB of this node
    pub fn aabb(&self) -> Aabb {
        Aabb::new(
            self.center - Vector3::splat(self.half_size),
            self.center + Vector3::splat(self.half_size),
        )
    }

    /// Check if this node contains a point
    pub fn contains(&self, point: Point3) -> bool {
        (point.x - self.center.x).abs() <= self.half_size
            && (point.y - self.center.y).abs() <= self.half_size
            && (point.z - self.center.z).abs() <= self.half_size
    }

    /// Get the child index for a point
    fn child_index(&self, point: Point3) -> usize {
        let mut index = 0;
        if point.x > self.center.x {
            index |= 1;
        }
        if point.y > self.center.y {
            index |= 2;
        }
        if point.z > self.center.z {
            index |= 4;
        }
        index
    }

    /// Get the center of a child node
    fn child_center(&self, index: usize) -> Point3 {
        let quarter = self.half_size / 2.0;
        Point3::new(
            self.center.x + if index & 1 != 0 { quarter } else { -quarter },
            self.center.y + if index & 2 != 0 { quarter } else { -quarter },
            self.center.z + if index & 4 != 0 { quarter } else { -quarter },
        )
    }

    /// Subdivide this node
    pub fn subdivide(&mut self) {
        if self.children.is_some() {
            return;
        }

        let child_half = self.half_size / 2.0;
        let children: [OctreeNode<T>; 8] = std::array::from_fn(|i| {
            OctreeNode::new(self.child_center(i), child_half)
        });

        self.children = Some(Box::new(children));
    }

    /// Insert an item with its position
    pub fn insert(&mut self, item: T, position: Point3, max_depth: usize, max_items: usize) {
        if max_depth == 0 || self.items.len() < max_items && self.children.is_none() {
            self.items.push(item);
            return;
        }

        if self.children.is_none() {
            self.subdivide();

            // Move existing items to children
            let items = std::mem::take(&mut self.items);
            // Note: We'd need the positions of existing items to redistribute them
            // For now, keep them in this node
            self.items = items;
        }

        let index = self.child_index(position);
        if let Some(ref mut children) = self.children {
            children[index].insert(item, position, max_depth - 1, max_items);
        }
    }

    /// Query items in a sphere
    pub fn query_sphere(&self, center: Point3, radius: f64) -> Vec<&T> {
        let mut results = Vec::new();
        self.query_sphere_recursive(center, radius, &mut results);
        results
    }

    fn query_sphere_recursive<'a>(&'a self, center: Point3, radius: f64, results: &mut Vec<&'a T>) {
        // Check if sphere intersects this node's AABB
        let aabb = self.aabb();
        if aabb.distance_to_point(center) > radius {
            return;
        }

        // Add items from this node
        for item in &self.items {
            results.push(item);
        }

        // Recurse into children
        if let Some(ref children) = self.children {
            for child in children.iter() {
                child.query_sphere_recursive(center, radius, results);
            }
        }
    }

    /// Query items in an AABB
    pub fn query_aabb(&self, query: &Aabb) -> Vec<&T> {
        let mut results = Vec::new();
        self.query_aabb_recursive(query, &mut results);
        results
    }

    fn query_aabb_recursive<'a>(&'a self, query: &Aabb, results: &mut Vec<&'a T>) {
        if !self.aabb().intersects(query) {
            return;
        }

        for item in &self.items {
            results.push(item);
        }

        if let Some(ref children) = self.children {
            for child in children.iter() {
                child.query_aabb_recursive(query, results);
            }
        }
    }
}

/// Octree for vertices
pub struct VertexOctree {
    root: OctreeNode<VertexHandle>,
    max_depth: usize,
    max_items: usize,
}

impl VertexOctree {
    /// Build an octree from vertices
    pub fn build(store: &TopologyStore, vertices: &[VertexHandle], max_depth: usize) -> Self {
        if vertices.is_empty() {
            return Self {
                root: OctreeNode::new(Point3::ZERO, 1.0),
                max_depth,
                max_items: 8,
            };
        }

        // Find bounding box
        let points: Vec<_> = vertices.iter().map(|v| Vertex::point(store, *v)).collect();
        let aabb = Aabb::from_points(&points).unwrap();

        let center = aabb.center();
        let half_size = aabb.extents().max_element() * 1.1; // Add margin

        let mut root = OctreeNode::new(center, half_size);

        for (vertex, point) in vertices.iter().zip(points.iter()) {
            root.insert(*vertex, *point, max_depth, 8);
        }

        Self {
            root,
            max_depth,
            max_items: 8,
        }
    }

    /// Query vertices in a sphere
    pub fn query_sphere(&self, center: Point3, radius: f64) -> Vec<VertexHandle> {
        self.root.query_sphere(center, radius).into_iter().copied().collect()
    }

    /// Query vertices in an AABB
    pub fn query_aabb(&self, query: &Aabb) -> Vec<VertexHandle> {
        self.root.query_aabb(query).into_iter().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_octree_query() {
        let store = TopologyStore::new();

        let v1 = Vertex::by_coordinates(&store, 0.0, 0.0, 0.0);
        let v2 = Vertex::by_coordinates(&store, 1.0, 0.0, 0.0);
        let v3 = Vertex::by_coordinates(&store, 5.0, 0.0, 0.0);

        let octree = VertexOctree::build(&store, &[v1, v2, v3], 5);

        // Query sphere around origin
        let results = octree.query_sphere(Point3::ZERO, 2.0);

        assert!(results.contains(&v1));
        assert!(results.contains(&v2));
        // v3 might or might not be in results depending on octree granularity
    }
}
