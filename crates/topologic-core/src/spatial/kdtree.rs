//! K-D Tree for fast nearest neighbor queries

use super::*;
use crate::topology::*;

/// K-D Tree node
pub struct KdNode<T> {
    pub item: T,
    pub point: Point3,
    pub axis: usize,
    pub left: Option<Box<KdNode<T>>>,
    pub right: Option<Box<KdNode<T>>>,
}

impl<T: Clone> KdNode<T> {
    /// Build a k-d tree from points
    pub fn build(items: &[(T, Point3)], depth: usize) -> Option<Self> {
        if items.is_empty() {
            return None;
        }

        let axis = depth % 3;

        // Sort by axis
        let mut sorted: Vec<_> = items.to_vec();
        sorted.sort_by(|a, b| {
            let va = match axis {
                0 => a.1.x,
                1 => a.1.y,
                _ => a.1.z,
            };
            let vb = match axis {
                0 => b.1.x,
                1 => b.1.y,
                _ => b.1.z,
            };
            va.partial_cmp(&vb).unwrap()
        });

        let mid = sorted.len() / 2;
        let (item, point) = sorted[mid].clone();

        Some(Self {
            item,
            point,
            axis,
            left: Self::build(&sorted[..mid], depth + 1).map(Box::new),
            right: Self::build(&sorted[mid + 1..], depth + 1).map(Box::new),
        })
    }

    /// Find the nearest neighbor
    pub fn nearest(&self, query: Point3) -> (&T, f64) {
        let mut best = (&self.item, self.point.distance_squared(query));
        self.nearest_recursive(query, &mut best);
        (best.0, best.1.sqrt())
    }

    fn nearest_recursive<'a>(&'a self, query: Point3, best: &mut (&'a T, f64)) {
        let dist_sq = self.point.distance_squared(query);
        if dist_sq < best.1 {
            *best = (&self.item, dist_sq);
        }

        let query_val = match self.axis {
            0 => query.x,
            1 => query.y,
            _ => query.z,
        };
        let self_val = match self.axis {
            0 => self.point.x,
            1 => self.point.y,
            _ => self.point.z,
        };

        let (first, second) = if query_val < self_val {
            (&self.left, &self.right)
        } else {
            (&self.right, &self.left)
        };

        if let Some(ref node) = first {
            node.nearest_recursive(query, best);
        }

        // Check if we need to search the other subtree
        let axis_dist_sq = (query_val - self_val).powi(2);
        if axis_dist_sq < best.1 {
            if let Some(ref node) = second {
                node.nearest_recursive(query, best);
            }
        }
    }

    /// Find all neighbors within a radius
    pub fn range(&self, query: Point3, radius: f64) -> Vec<&T> {
        let mut results = Vec::new();
        self.range_recursive(query, radius * radius, &mut results);
        results
    }

    fn range_recursive<'a>(&'a self, query: Point3, radius_sq: f64, results: &mut Vec<&'a T>) {
        let dist_sq = self.point.distance_squared(query);
        if dist_sq <= radius_sq {
            results.push(&self.item);
        }

        let query_val = match self.axis {
            0 => query.x,
            1 => query.y,
            _ => query.z,
        };
        let self_val = match self.axis {
            0 => self.point.x,
            1 => self.point.y,
            _ => self.point.z,
        };

        let axis_dist_sq = (query_val - self_val).powi(2);

        if query_val - radius_sq.sqrt() <= self_val {
            if let Some(ref left) = self.left {
                left.range_recursive(query, radius_sq, results);
            }
        }
        if query_val + radius_sq.sqrt() >= self_val {
            if let Some(ref right) = self.right {
                right.range_recursive(query, radius_sq, results);
            }
        }
    }

    /// Find k nearest neighbors
    pub fn k_nearest(&self, query: Point3, k: usize) -> Vec<(&T, f64)> {
        let mut heap: Vec<(&T, f64)> = Vec::with_capacity(k + 1);
        self.k_nearest_recursive(query, k, &mut heap);
        heap.into_iter().map(|(t, d)| (t, d.sqrt())).collect()
    }

    fn k_nearest_recursive<'a>(&'a self, query: Point3, k: usize, heap: &mut Vec<(&'a T, f64)>) {
        let dist_sq = self.point.distance_squared(query);

        // Insert into heap
        heap.push((&self.item, dist_sq));
        heap.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        if heap.len() > k {
            heap.pop();
        }

        let query_val = match self.axis {
            0 => query.x,
            1 => query.y,
            _ => query.z,
        };
        let self_val = match self.axis {
            0 => self.point.x,
            1 => self.point.y,
            _ => self.point.z,
        };

        let (first, second) = if query_val < self_val {
            (&self.left, &self.right)
        } else {
            (&self.right, &self.left)
        };

        if let Some(ref node) = first {
            node.k_nearest_recursive(query, k, heap);
        }

        let axis_dist_sq = (query_val - self_val).powi(2);
        let worst_dist = heap.last().map(|(_, d)| *d).unwrap_or(f64::MAX);
        if heap.len() < k || axis_dist_sq < worst_dist {
            if let Some(ref node) = second {
                node.k_nearest_recursive(query, k, heap);
            }
        }
    }
}

/// K-D Tree for vertices
pub struct VertexKdTree {
    root: Option<KdNode<VertexHandle>>,
}

impl VertexKdTree {
    /// Build a k-d tree from vertices
    pub fn build(store: &TopologyStore, vertices: &[VertexHandle]) -> Self {
        let items: Vec<_> = vertices
            .iter()
            .map(|v| (*v, Vertex::point(store, *v)))
            .collect();

        Self {
            root: KdNode::build(&items, 0),
        }
    }

    /// Find the nearest vertex
    pub fn nearest(&self, query: Point3) -> Option<(VertexHandle, f64)> {
        self.root.as_ref().map(|node| {
            let (v, d) = node.nearest(query);
            (*v, d)
        })
    }

    /// Find all vertices within a radius
    pub fn range(&self, query: Point3, radius: f64) -> Vec<VertexHandle> {
        match &self.root {
            Some(node) => node.range(query, radius).into_iter().copied().collect(),
            None => Vec::new(),
        }
    }

    /// Find k nearest vertices
    pub fn k_nearest(&self, query: Point3, k: usize) -> Vec<(VertexHandle, f64)> {
        match &self.root {
            Some(node) => node.k_nearest(query, k).into_iter().map(|(v, d)| (*v, d)).collect(),
            None => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kdtree_nearest() {
        let store = TopologyStore::new();

        let v1 = Vertex::by_coordinates(&store, 0.0, 0.0, 0.0);
        let v2 = Vertex::by_coordinates(&store, 1.0, 0.0, 0.0);
        let v3 = Vertex::by_coordinates(&store, 5.0, 0.0, 0.0);

        let kdtree = VertexKdTree::build(&store, &[v1, v2, v3]);

        let result = kdtree.nearest(Point3::new(0.4, 0.0, 0.0));
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, v1);
    }

    #[test]
    fn test_kdtree_range() {
        let store = TopologyStore::new();

        let v1 = Vertex::by_coordinates(&store, 0.0, 0.0, 0.0);
        let v2 = Vertex::by_coordinates(&store, 1.0, 0.0, 0.0);
        let v3 = Vertex::by_coordinates(&store, 5.0, 0.0, 0.0);

        let kdtree = VertexKdTree::build(&store, &[v1, v2, v3]);

        let results = kdtree.range(Point3::ZERO, 1.5);
        assert_eq!(results.len(), 2);
        assert!(results.contains(&v1));
        assert!(results.contains(&v2));
    }

    #[test]
    fn test_kdtree_k_nearest() {
        let store = TopologyStore::new();

        let v1 = Vertex::by_coordinates(&store, 0.0, 0.0, 0.0);
        let v2 = Vertex::by_coordinates(&store, 1.0, 0.0, 0.0);
        let v3 = Vertex::by_coordinates(&store, 2.0, 0.0, 0.0);
        let v4 = Vertex::by_coordinates(&store, 5.0, 0.0, 0.0);

        let kdtree = VertexKdTree::build(&store, &[v1, v2, v3, v4]);

        let results = kdtree.k_nearest(Point3::ZERO, 2);
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|(v, _)| *v == v1));
        assert!(results.iter().any(|(v, _)| *v == v2));
    }
}
