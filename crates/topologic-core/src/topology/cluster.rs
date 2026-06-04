//! Cluster topology (mixed collection of any topology types)

use super::*;
use crate::geometry::Point3;

/// Handle to a cluster in the topology store
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClusterHandle {
    pub(crate) index: ArenaIndex,
    pub(crate) id: TopologyId,
}

impl ClusterHandle {
    pub fn id(&self) -> TopologyId {
        self.id
    }
}

/// Internal cluster data
pub(crate) struct ClusterData {
    pub id: TopologyId,
    pub members: Vec<TopologyHandle>,
    pub dictionary: Dictionary,
}

/// Cluster operations
pub struct Cluster;

impl Cluster {
    /// Create a cluster from mixed topology handles
    pub fn by_topologies(store: &TopologyStore, members: Vec<TopologyHandle>) -> ClusterHandle {
        store.add_cluster(members)
    }

    /// Create a cluster from vertices
    pub fn by_vertices(store: &TopologyStore, vertices: Vec<VertexHandle>) -> ClusterHandle {
        let handles: Vec<_> = vertices.into_iter().map(TopologyHandle::Vertex).collect();
        store.add_cluster(handles)
    }

    /// Create a cluster from edges
    pub fn by_edges(store: &TopologyStore, edges: Vec<EdgeHandle>) -> ClusterHandle {
        let handles: Vec<_> = edges.into_iter().map(TopologyHandle::Edge).collect();
        store.add_cluster(handles)
    }

    /// Create a cluster from faces
    pub fn by_faces(store: &TopologyStore, faces: Vec<FaceHandle>) -> ClusterHandle {
        let handles: Vec<_> = faces.into_iter().map(TopologyHandle::Face).collect();
        store.add_cluster(handles)
    }

    /// Create a cluster from cells
    pub fn by_cells(store: &TopologyStore, cells: Vec<CellHandle>) -> ClusterHandle {
        let handles: Vec<_> = cells.into_iter().map(TopologyHandle::Cell).collect();
        store.add_cluster(handles)
    }

    /// Get all members
    pub fn members(store: &TopologyStore, handle: ClusterHandle) -> Vec<TopologyHandle> {
        store.clusters.read()[handle.index.index()].members.clone()
    }

    /// Get members of a specific type
    pub fn members_of_type(
        store: &TopologyStore,
        handle: ClusterHandle,
        topology_type: TopologyType,
    ) -> Vec<TopologyHandle> {
        Self::members(store, handle)
            .into_iter()
            .filter(|m| m.topology_type() == topology_type)
            .collect()
    }

    /// Get all vertices in the cluster
    pub fn vertices(store: &TopologyStore, handle: ClusterHandle) -> Vec<VertexHandle> {
        let mut result = Vec::new();

        for member in Self::members(store, handle) {
            match member {
                TopologyHandle::Vertex(v) => result.push(v),
                TopologyHandle::Edge(e) => {
                    let (v1, v2) = Edge::vertices(store, e);
                    result.push(v1);
                    result.push(v2);
                }
                TopologyHandle::Wire(w) => result.extend(Wire::vertices(store, w)),
                TopologyHandle::Face(f) => result.extend(Face::vertices(store, f)),
                TopologyHandle::Shell(s) => result.extend(Shell::vertices(store, s)),
                TopologyHandle::Cell(c) => result.extend(Cell::vertices(store, c)),
                TopologyHandle::CellComplex(cc) => result.extend(CellComplex::vertices(store, cc)),
                TopologyHandle::Cluster(cl) => result.extend(Self::vertices(store, cl)),
            }
        }

        // Remove duplicates
        let mut seen = hashbrown::HashSet::new();
        result.retain(|v| seen.insert(*v));
        result
    }

    /// Get all edges in the cluster
    pub fn edges(store: &TopologyStore, handle: ClusterHandle) -> Vec<EdgeHandle> {
        let mut result = Vec::new();

        for member in Self::members(store, handle) {
            match member {
                TopologyHandle::Vertex(_) => {}
                TopologyHandle::Edge(e) => result.push(e),
                TopologyHandle::Wire(w) => result.extend(Wire::edges(store, w)),
                TopologyHandle::Face(f) => result.extend(Face::edges(store, f)),
                TopologyHandle::Shell(s) => result.extend(Shell::edges(store, s)),
                TopologyHandle::Cell(c) => result.extend(Cell::edges(store, c)),
                TopologyHandle::CellComplex(cc) => result.extend(CellComplex::edges(store, cc)),
                TopologyHandle::Cluster(cl) => result.extend(Self::edges(store, cl)),
            }
        }

        let mut seen = hashbrown::HashSet::new();
        result.retain(|e| seen.insert(*e));
        result
    }

    /// Get all faces in the cluster
    pub fn faces(store: &TopologyStore, handle: ClusterHandle) -> Vec<FaceHandle> {
        let mut result = Vec::new();

        for member in Self::members(store, handle) {
            match member {
                TopologyHandle::Face(f) => result.push(f),
                TopologyHandle::Shell(s) => result.extend(Shell::faces(store, s)),
                TopologyHandle::Cell(c) => result.extend(Cell::faces(store, c)),
                TopologyHandle::CellComplex(cc) => result.extend(CellComplex::faces(store, cc)),
                TopologyHandle::Cluster(cl) => result.extend(Self::faces(store, cl)),
                _ => {}
            }
        }

        let mut seen = hashbrown::HashSet::new();
        result.retain(|f| seen.insert(*f));
        result
    }

    /// Get all cells in the cluster
    pub fn cells(store: &TopologyStore, handle: ClusterHandle) -> Vec<CellHandle> {
        let mut result = Vec::new();

        for member in Self::members(store, handle) {
            match member {
                TopologyHandle::Cell(c) => result.push(c),
                TopologyHandle::CellComplex(cc) => result.extend(CellComplex::cells(store, cc)),
                TopologyHandle::Cluster(cl) => result.extend(Self::cells(store, cl)),
                _ => {}
            }
        }

        let mut seen = hashbrown::HashSet::new();
        result.retain(|c| seen.insert(*c));
        result
    }

    /// Get the number of members
    pub fn size(store: &TopologyStore, handle: ClusterHandle) -> usize {
        Self::members(store, handle).len()
    }

    /// Check if the cluster is empty
    pub fn is_empty(store: &TopologyStore, handle: ClusterHandle) -> bool {
        Self::members(store, handle).is_empty()
    }

    /// Get the center of mass of all vertices
    pub fn center_of_mass(store: &TopologyStore, handle: ClusterHandle) -> Point3 {
        let vertices = Self::vertices(store, handle);
        if vertices.is_empty() {
            return Point3::ZERO;
        }

        let points: Vec<_> = vertices.iter().map(|v| Vertex::point(store, *v)).collect();
        crate::geometry::centroid(&points)
    }

    /// Get the bounding box
    pub fn bounding_box(store: &TopologyStore, handle: ClusterHandle) -> (Point3, Point3) {
        let vertices = Self::vertices(store, handle);
        if vertices.is_empty() {
            return (Point3::ZERO, Point3::ZERO);
        }

        let points: Vec<_> = vertices.iter().map(|v| Vertex::point(store, *v)).collect();
        crate::geometry::bounding_box(&points).unwrap_or((Point3::ZERO, Point3::ZERO))
    }

    /// Add a member to the cluster
    pub fn add_member(store: &TopologyStore, handle: ClusterHandle, member: TopologyHandle) -> ClusterHandle {
        let mut members = Self::members(store, handle);
        members.push(member);
        store.add_cluster(members)
    }

    /// Merge two clusters
    pub fn merge(store: &TopologyStore, c1: ClusterHandle, c2: ClusterHandle) -> ClusterHandle {
        let mut members = Self::members(store, c1);
        members.extend(Self::members(store, c2));
        store.add_cluster(members)
    }

    /// Filter members by predicate
    pub fn filter<F>(store: &TopologyStore, handle: ClusterHandle, predicate: F) -> ClusterHandle
    where
        F: Fn(&TopologyHandle) -> bool,
    {
        let members: Vec<_> = Self::members(store, handle)
            .into_iter()
            .filter(|m| predicate(m))
            .collect();
        store.add_cluster(members)
    }

    /// Get the highest dimensional member
    pub fn highest_dimension(store: &TopologyStore, handle: ClusterHandle) -> Option<u8> {
        Self::members(store, handle)
            .iter()
            .map(|m| m.topology_type().dimensionality())
            .max()
    }

    // ========== Clustering Algorithms ==========

    /// K-Means clustering on vertex positions
    /// Returns cluster labels for each vertex (0 to k-1)
    pub fn kmeans(
        store: &TopologyStore,
        vertices: &[VertexHandle],
        k: usize,
        max_iterations: usize,
        tolerance: f64,
    ) -> Vec<usize> {
        if vertices.is_empty() || k == 0 {
            return vec![];
        }

        let k = k.min(vertices.len());
        let points: Vec<Point3> = vertices.iter().map(|v| Vertex::point(store, *v)).collect();
        let n = points.len();

        // Initialize centroids using k-means++ initialization
        let mut centroids = Self::kmeans_plus_plus_init(&points, k);
        let mut labels = vec![0usize; n];
        let mut old_labels = vec![usize::MAX; n];

        for _iteration in 0..max_iterations {
            // Assign points to nearest centroid
            for (i, point) in points.iter().enumerate() {
                let mut min_dist = f64::MAX;
                let mut best_cluster = 0;
                for (j, centroid) in centroids.iter().enumerate() {
                    let dist = point.distance_squared(*centroid);
                    if dist < min_dist {
                        min_dist = dist;
                        best_cluster = j;
                    }
                }
                labels[i] = best_cluster;
            }

            // Check for convergence
            if labels == old_labels {
                break;
            }
            old_labels = labels.clone();

            // Update centroids
            let mut cluster_sums = vec![Point3::ZERO; k];
            let mut cluster_counts = vec![0usize; k];

            for (i, &label) in labels.iter().enumerate() {
                cluster_sums[label] = Point3::new(
                    cluster_sums[label].x + points[i].x,
                    cluster_sums[label].y + points[i].y,
                    cluster_sums[label].z + points[i].z,
                );
                cluster_counts[label] += 1;
            }

            let mut max_shift: f64 = 0.0;
            for j in 0..k {
                if cluster_counts[j] > 0 {
                    let new_centroid = Point3::new(
                        cluster_sums[j].x / cluster_counts[j] as f64,
                        cluster_sums[j].y / cluster_counts[j] as f64,
                        cluster_sums[j].z / cluster_counts[j] as f64,
                    );
                    max_shift = max_shift.max(centroids[j].distance_squared(new_centroid).sqrt());
                    centroids[j] = new_centroid;
                }
            }

            if max_shift < tolerance {
                break;
            }
        }

        labels
    }

    /// K-means++ initialization for better initial centroids
    fn kmeans_plus_plus_init(points: &[Point3], k: usize) -> Vec<Point3> {
        use std::collections::HashSet;

        let n = points.len();
        if n == 0 || k == 0 {
            return vec![];
        }

        let mut centroids = Vec::with_capacity(k);
        let mut selected = HashSet::new();

        // Choose first centroid randomly (use deterministic selection for reproducibility)
        let first_idx = 0;
        centroids.push(points[first_idx]);
        selected.insert(first_idx);

        // Choose remaining centroids
        for _ in 1..k {
            // Calculate distance squared to nearest centroid for each point
            let mut distances: Vec<f64> = Vec::with_capacity(n);
            let mut total_dist = 0.0;

            for (i, point) in points.iter().enumerate() {
                if selected.contains(&i) {
                    distances.push(0.0);
                    continue;
                }

                let min_dist = centroids
                    .iter()
                    .map(|c| point.distance_squared(*c))
                    .fold(f64::MAX, |a, b| a.min(b));
                distances.push(min_dist);
                total_dist += min_dist;
            }

            // Choose next centroid proportional to distance squared
            // For determinism, choose the point with maximum distance
            let mut max_dist = 0.0;
            let mut next_idx = 0;
            for (i, &dist) in distances.iter().enumerate() {
                if !selected.contains(&i) && dist > max_dist {
                    max_dist = dist;
                    next_idx = i;
                }
            }

            centroids.push(points[next_idx]);
            selected.insert(next_idx);
        }

        centroids
    }

    /// DBSCAN clustering on vertex positions
    /// Returns cluster labels (-1 for noise, 0+ for clusters)
    pub fn dbscan(
        store: &TopologyStore,
        vertices: &[VertexHandle],
        eps: f64,
        min_samples: usize,
    ) -> Vec<i32> {
        if vertices.is_empty() {
            return vec![];
        }

        let points: Vec<Point3> = vertices.iter().map(|v| Vertex::point(store, *v)).collect();
        let n = points.len();
        let eps_sq = eps * eps;

        // -2 = unvisited, -1 = noise, 0+ = cluster
        let mut labels = vec![-2i32; n];
        let mut cluster_id = 0i32;

        // Build neighborhood lists
        let get_neighbors = |idx: usize| -> Vec<usize> {
            let point = &points[idx];
            points
                .iter()
                .enumerate()
                .filter(|(i, p)| *i != idx && point.distance_squared(**p) <= eps_sq)
                .map(|(i, _)| i)
                .collect()
        };

        for i in 0..n {
            if labels[i] != -2 {
                continue; // Already processed
            }

            let neighbors = get_neighbors(i);
            if neighbors.len() < min_samples {
                labels[i] = -1; // Noise
                continue;
            }

            // Start a new cluster
            labels[i] = cluster_id;
            let mut seed_set: Vec<usize> = neighbors;
            let mut j = 0;

            while j < seed_set.len() {
                let q = seed_set[j];

                if labels[q] == -1 {
                    labels[q] = cluster_id; // Change noise to border point
                }

                if labels[q] != -2 {
                    j += 1;
                    continue; // Already processed
                }

                labels[q] = cluster_id;

                let q_neighbors = get_neighbors(q);
                if q_neighbors.len() >= min_samples {
                    // q is a core point, add its neighbors to seed set
                    for &neighbor in &q_neighbors {
                        if labels[neighbor] == -2 || labels[neighbor] == -1 {
                            if labels[neighbor] == -2 {
                                seed_set.push(neighbor);
                            }
                        }
                    }
                }

                j += 1;
            }

            cluster_id += 1;
        }

        labels
    }

    /// HDBSCAN clustering on vertex positions
    /// Simplified implementation using DBSCAN with multiple epsilon values
    /// Returns cluster labels (-1 for noise, 0+ for clusters)
    pub fn hdbscan(
        store: &TopologyStore,
        vertices: &[VertexHandle],
        min_cluster_size: usize,
        min_samples: usize,
    ) -> Vec<i32> {
        if vertices.is_empty() {
            return vec![];
        }

        let points: Vec<Point3> = vertices.iter().map(|v| Vertex::point(store, *v)).collect();
        let n = points.len();

        // Compute pairwise distances and core distances
        let mut distances: Vec<Vec<f64>> = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in (i + 1)..n {
                let dist = points[i].distance_squared(points[j]).sqrt();
                distances[i][j] = dist;
                distances[j][i] = dist;
            }
        }

        // Compute core distances (distance to min_samples-th nearest neighbor)
        let min_samples = min_samples.max(1);
        let mut core_distances = vec![0.0; n];
        for i in 0..n {
            let mut dists: Vec<f64> = distances[i].clone();
            dists.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            core_distances[i] = dists.get(min_samples.min(n - 1)).copied().unwrap_or(f64::MAX);
        }

        // Compute mutual reachability distances
        let mut mrd: Vec<Vec<f64>> = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in (i + 1)..n {
                let d = distances[i][j]
                    .max(core_distances[i])
                    .max(core_distances[j]);
                mrd[i][j] = d;
                mrd[j][i] = d;
            }
        }

        // Build minimum spanning tree using Prim's algorithm
        let mut in_tree = vec![false; n];
        let mut min_dist = vec![f64::MAX; n];
        let mut mst_edges: Vec<(usize, usize, f64)> = Vec::new();

        in_tree[0] = true;
        for j in 1..n {
            min_dist[j] = mrd[0][j];
        }

        for _ in 1..n {
            // Find minimum distance vertex not in tree
            let mut min_val = f64::MAX;
            let mut min_idx = 0;
            for j in 0..n {
                if !in_tree[j] && min_dist[j] < min_val {
                    min_val = min_dist[j];
                    min_idx = j;
                }
            }

            if min_val == f64::MAX {
                break;
            }

            in_tree[min_idx] = true;

            // Find which vertex it connects to
            for i in 0..n {
                if in_tree[i] && i != min_idx && mrd[i][min_idx] == min_val {
                    mst_edges.push((i, min_idx, min_val));
                    break;
                }
            }

            // Update distances
            for j in 0..n {
                if !in_tree[j] {
                    min_dist[j] = min_dist[j].min(mrd[min_idx][j]);
                }
            }
        }

        // Sort MST edges by weight (descending) and extract clusters
        mst_edges.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        // Use Union-Find to extract clusters
        let mut parent: Vec<usize> = (0..n).collect();
        let mut rank = vec![0usize; n];

        fn find(parent: &mut [usize], i: usize) -> usize {
            if parent[i] != i {
                parent[i] = find(parent, parent[i]);
            }
            parent[i]
        }

        fn union(parent: &mut [usize], rank: &mut [usize], x: usize, y: usize) {
            let xroot = find(parent, x);
            let yroot = find(parent, y);
            if rank[xroot] < rank[yroot] {
                parent[xroot] = yroot;
            } else if rank[xroot] > rank[yroot] {
                parent[yroot] = xroot;
            } else {
                parent[yroot] = xroot;
                rank[xroot] += 1;
            }
        }

        // Connect edges until we have clusters of min_cluster_size
        for (u, v, _weight) in &mst_edges {
            union(&mut parent, &mut rank, *u, *v);
        }

        // Assign cluster labels based on connected components
        let mut component_map: hashbrown::HashMap<usize, i32> = hashbrown::HashMap::new();
        let mut labels = vec![-1i32; n];
        let mut next_cluster = 0i32;

        // Count component sizes
        let mut component_sizes: hashbrown::HashMap<usize, usize> = hashbrown::HashMap::new();
        for i in 0..n {
            let root = find(&mut parent, i);
            *component_sizes.entry(root).or_insert(0) += 1;
        }

        // Assign labels (only to components >= min_cluster_size)
        for i in 0..n {
            let root = find(&mut parent, i);
            let size = component_sizes.get(&root).copied().unwrap_or(0);

            if size >= min_cluster_size {
                let label = *component_map.entry(root).or_insert_with(|| {
                    let l = next_cluster;
                    next_cluster += 1;
                    l
                });
                labels[i] = label;
            }
        }

        labels
    }

    /// Get dictionary for a cluster
    pub fn get_dictionary(store: &TopologyStore, handle: ClusterHandle) -> Dictionary {
        store.clusters.read()[handle.index.index()].dictionary.clone()
    }

    /// Set dictionary for a cluster
    pub fn set_dictionary(store: &TopologyStore, handle: ClusterHandle, dict: Dictionary) {
        store.clusters.write()[handle.index.index()].dictionary = dict;
    }

    /// Set a single key-value pair in the cluster's dictionary
    pub fn set_dictionary_value(store: &TopologyStore, handle: ClusterHandle, key: &str, value: DictionaryValue) {
        store.clusters.write()[handle.index.index()].dictionary.set(key, value);
    }

    /// Get a value from the cluster's dictionary
    pub fn get_dictionary_value(store: &TopologyStore, handle: ClusterHandle, key: &str) -> Option<DictionaryValue> {
        store.clusters.read()[handle.index.index()].dictionary.get(key).cloned()
    }
}

impl<'a> ClusterRef<'a> {
    pub fn members(&self) -> Vec<TopologyHandle> {
        Cluster::members(self.store, self.handle)
    }

    pub fn vertices(&self) -> Vec<VertexHandle> {
        Cluster::vertices(self.store, self.handle)
    }

    pub fn edges(&self) -> Vec<EdgeHandle> {
        Cluster::edges(self.store, self.handle)
    }

    pub fn faces(&self) -> Vec<FaceHandle> {
        Cluster::faces(self.store, self.handle)
    }

    pub fn cells(&self) -> Vec<CellHandle> {
        Cluster::cells(self.store, self.handle)
    }

    pub fn size(&self) -> usize {
        Cluster::size(self.store, self.handle)
    }

    pub fn center_of_mass(&self) -> Point3 {
        Cluster::center_of_mass(self.store, self.handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_vertices() {
        let store = TopologyStore::new();
        let v1 = Vertex::by_coordinates(&store, 0.0, 0.0, 0.0);
        let v2 = Vertex::by_coordinates(&store, 1.0, 0.0, 0.0);
        let v3 = Vertex::by_coordinates(&store, 2.0, 0.0, 0.0);

        let cluster = Cluster::by_vertices(&store, vec![v1, v2, v3]);
        assert_eq!(Cluster::size(&store, cluster), 3);
        assert_eq!(Cluster::vertices(&store, cluster).len(), 3);
    }

    #[test]
    fn test_cluster_mixed() {
        let store = TopologyStore::new();
        let v = Vertex::by_coordinates(&store, 0.0, 0.0, 0.0);
        let e = Edge::by_coordinates(&store, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let f = Face::rectangle(&store, Point3::ZERO, 1.0, 1.0);

        let cluster = Cluster::by_topologies(
            &store,
            vec![
                TopologyHandle::Vertex(v),
                TopologyHandle::Edge(e),
                TopologyHandle::Face(f),
            ],
        );

        assert_eq!(Cluster::size(&store, cluster), 3);
    }
}
