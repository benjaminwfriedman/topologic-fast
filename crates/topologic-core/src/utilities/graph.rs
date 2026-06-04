//! Graph utility class matching topologicpy Graph API
//!
//! Provides graph-based topology analysis and pathfinding.

use crate::geometry::{Point3, Vector3, TOLERANCE};
use crate::topology::{TopologyStore, VertexHandle, EdgeHandle, FaceHandle, CellHandle,
                      CellComplexHandle, ClusterHandle, ShellHandle, Vertex, Edge, Face, Cell, CellComplex,
                      Cluster, TopologyHandle, Wire, WireHandle, Shell};
use hashbrown::{HashMap, HashSet};
use std::collections::VecDeque;

/// Handle to a graph in the topology store
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GraphHandle {
    pub(crate) vertices: ClusterHandle,
    pub(crate) edges: ClusterHandle,
}

/// Graph utility class
pub struct GraphUtil;

impl GraphUtil {
    /// Create a graph from vertices and edges
    pub fn by_vertices_edges(
        store: &TopologyStore,
        vertices: Vec<VertexHandle>,
        edges: Vec<EdgeHandle>,
    ) -> GraphHandle {
        let vertex_cluster = Cluster::by_vertices(store, vertices);
        let edge_cluster = Cluster::by_edges(store, edges);

        GraphHandle {
            vertices: vertex_cluster,
            edges: edge_cluster,
        }
    }

    /// Create a graph from a topology (dual graph)
    ///
    /// For CellComplex: Creates a vertex for each Cell, edges between Cells that share a Face
    /// For Cell: Creates a single vertex at the Cell's centroid
    /// For Shell: Creates a vertex for each Face, edges between Faces that share an Edge
    /// For Face: Creates a single vertex at the Face's centroid
    /// For Wire: Creates a vertex for each Edge, edges between Edges that share a Vertex
    /// For Edge: Creates a single vertex at the Edge's centroid
    /// For Vertex: Creates a single vertex
    pub fn by_topology(
        store: &TopologyStore,
        topology: TopologyHandle,
        direct: bool,
        via_shared_topologies: bool,
        to_exterior_topologies: bool,
        tolerance: f64,
    ) -> GraphHandle {
        match topology {
            TopologyHandle::CellComplex(cc) => Self::from_cell_complex(store, cc, direct, tolerance),
            TopologyHandle::Cell(c) => Self::from_cell_single(store, c),
            TopologyHandle::Shell(s) => Self::from_shell(store, s, direct, tolerance),
            TopologyHandle::Face(f) => Self::from_face_single(store, f),
            TopologyHandle::Wire(w) => Self::from_wire(store, w, direct, tolerance),
            TopologyHandle::Edge(e) => Self::from_edge_single(store, e),
            TopologyHandle::Vertex(v) => Self::from_vertex(store, v),
            TopologyHandle::Cluster(_) => {
                // For cluster, just return empty graph for now
                Self::by_vertices_edges(store, vec![], vec![])
            }
        }
    }

    /// Create a graph from a cell complex (dual graph)
    /// Creates a vertex for each Cell, edges between Cells that share a Face
    fn from_cell_complex(
        store: &TopologyStore,
        cc: CellComplexHandle,
        direct: bool,
        tolerance: f64,
    ) -> GraphHandle {
        let cells = CellComplex::cells(store, cc);
        let mut vertices = Vec::new();
        let mut graph_edges = Vec::new();

        // Create a vertex for each cell centroid
        let mut cell_to_vertex: HashMap<CellHandle, VertexHandle> = HashMap::new();
        for cell in &cells {
            let centroid = Cell::center_of_mass(store, *cell);
            let v = Vertex::by_point(store, centroid);
            cell_to_vertex.insert(*cell, v);
            vertices.push(v);
        }

        // Create edges between adjacent cells
        if direct {
            // Check every pair of cells for adjacency (faces that overlap on same plane)
            let mut seen_pairs: HashSet<(usize, usize)> = HashSet::new();

            for (i, cell1) in cells.iter().enumerate() {
                for (j, cell2) in cells.iter().enumerate() {
                    if i >= j {
                        continue;
                    }

                    // Check if cells share an overlapping face
                    if Self::cells_share_face(store, *cell1, *cell2, tolerance) {
                        let v1 = cell_to_vertex[cell1];
                        let v2 = cell_to_vertex[cell2];

                        let pair = (i, j);
                        if !seen_pairs.contains(&pair) {
                            seen_pairs.insert(pair);
                            graph_edges.push(Edge::by_start_vertex_end_vertex(store, v1, v2));
                        }
                    }
                }
            }
        }

        Self::by_vertices_edges(store, vertices, graph_edges)
    }

    /// Check if two cells share an overlapping face (adjacent on a common plane)
    fn cells_share_face(
        store: &TopologyStore,
        cell1: CellHandle,
        cell2: CellHandle,
        tolerance: f64,
    ) -> bool {
        let faces1 = Cell::faces(store, cell1);
        let faces2 = Cell::faces(store, cell2);

        for face1 in &faces1 {
            for face2 in &faces2 {
                if Self::faces_overlap(store, *face1, *face2, tolerance) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if two faces overlap (are coplanar and share area)
    fn faces_overlap(
        store: &TopologyStore,
        face1: FaceHandle,
        face2: FaceHandle,
        tolerance: f64,
    ) -> bool {
        // Get bounding boxes and plane info for each face
        let (min1, max1, plane1) = Self::face_bounds_and_plane(store, face1);
        let (min2, max2, plane2) = Self::face_bounds_and_plane(store, face2);

        // Must be on the same plane (same axis and same coordinate value)
        if plane1.0 != plane2.0 {
            return false;
        }
        if (plane1.1 - plane2.1).abs() > tolerance {
            return false;
        }

        // Check if 2D bounding boxes overlap (excluding the plane axis)
        // Get the two axes that are NOT the plane axis
        let (ax1, ax2) = match plane1.0 {
            0 => (1, 2), // X plane, check Y and Z
            1 => (0, 2), // Y plane, check X and Z
            _ => (0, 1), // Z plane, check X and Y
        };

        // Check overlap on both axes
        let overlap_ax1 = Self::ranges_overlap(
            min1[ax1], max1[ax1],
            min2[ax1], max2[ax1],
            tolerance,
        );
        let overlap_ax2 = Self::ranges_overlap(
            min1[ax2], max1[ax2],
            min2[ax2], max2[ax2],
            tolerance,
        );

        overlap_ax1 && overlap_ax2
    }

    /// Get face bounding box and which plane it lies on
    /// Returns (min_coords, max_coords, (axis_index, axis_value))
    fn face_bounds_and_plane(
        store: &TopologyStore,
        face: FaceHandle,
    ) -> ([f64; 3], [f64; 3], (usize, f64)) {
        let vertices = Face::vertices(store, face);

        let mut min = [f64::MAX, f64::MAX, f64::MAX];
        let mut max = [f64::MIN, f64::MIN, f64::MIN];

        for v in &vertices {
            let p = Vertex::point(store, *v);
            let coords = [p.x, p.y, p.z];
            for i in 0..3 {
                min[i] = min[i].min(coords[i]);
                max[i] = max[i].max(coords[i]);
            }
        }

        // Determine which axis is constant (the plane)
        let ranges = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
        let plane_axis = if ranges[0] < 0.001 {
            0
        } else if ranges[1] < 0.001 {
            1
        } else {
            2
        };

        (min, max, (plane_axis, min[plane_axis]))
    }

    /// Check if two 1D ranges overlap
    fn ranges_overlap(
        min1: f64, max1: f64,
        min2: f64, max2: f64,
        tolerance: f64,
    ) -> bool {
        // Ranges overlap if one doesn't end before the other starts
        // With tolerance to handle touching edges
        max1 > min2 + tolerance && max2 > min1 + tolerance
    }

    /// Create a graph from a single cell (just the centroid vertex)
    fn from_cell_single(store: &TopologyStore, cell: CellHandle) -> GraphHandle {
        let centroid = Cell::center_of_mass(store, cell);
        let v = Vertex::by_point(store, centroid);
        Self::by_vertices_edges(store, vec![v], vec![])
    }

    /// Create a graph from a shell (dual graph)
    /// Creates a vertex for each Face, edges between Faces that share an Edge
    fn from_shell(
        store: &TopologyStore,
        shell: ShellHandle,
        direct: bool,
        tolerance: f64,
    ) -> GraphHandle {
        let faces = Shell::faces(store, shell);
        let mut vertices = Vec::new();
        let mut graph_edges = Vec::new();

        // Create a vertex for each face centroid
        let mut face_to_vertex: HashMap<FaceHandle, VertexHandle> = HashMap::new();
        for face in &faces {
            let centroid = Face::center_of_mass(store, *face);
            let v = Vertex::by_point(store, centroid);
            face_to_vertex.insert(*face, v);
            vertices.push(v);
        }

        // Create edges between adjacent faces (those that share an edge)
        if direct {
            // Build a map from edge geometry key to the faces that contain it
            let mut edge_to_faces: HashMap<String, Vec<FaceHandle>> = HashMap::new();
            for face in &faces {
                let face_edges = Face::edges(store, *face);
                for edge in face_edges {
                    let key = Self::edge_geometry_key(store, edge, tolerance);
                    edge_to_faces.entry(key).or_default().push(*face);
                }
            }

            // Connect faces that share an edge
            let mut seen_pairs: HashSet<(usize, usize)> = HashSet::new();
            for (_key, sharing_faces) in edge_to_faces.iter() {
                if sharing_faces.len() >= 2 {
                    for i in 0..sharing_faces.len() {
                        for j in (i + 1)..sharing_faces.len() {
                            let f1 = sharing_faces[i];
                            let f2 = sharing_faces[j];
                            let v1 = face_to_vertex[&f1];
                            let v2 = face_to_vertex[&f2];

                            let idx1 = vertices.iter().position(|v| *v == v1).unwrap_or(0);
                            let idx2 = vertices.iter().position(|v| *v == v2).unwrap_or(0);
                            let pair = if idx1 < idx2 { (idx1, idx2) } else { (idx2, idx1) };

                            if !seen_pairs.contains(&pair) {
                                seen_pairs.insert(pair);
                                graph_edges.push(Edge::by_start_vertex_end_vertex(store, v1, v2));
                            }
                        }
                    }
                }
            }
        }

        Self::by_vertices_edges(store, vertices, graph_edges)
    }

    /// Create a graph from a single face (just the centroid vertex)
    fn from_face_single(store: &TopologyStore, face: FaceHandle) -> GraphHandle {
        let centroid = Face::center_of_mass(store, face);
        let v = Vertex::by_point(store, centroid);
        Self::by_vertices_edges(store, vec![v], vec![])
    }

    /// Create a graph from a wire (dual graph)
    /// Creates a vertex for each Edge, edges between Edges that share a Vertex
    fn from_wire(
        store: &TopologyStore,
        wire: WireHandle,
        direct: bool,
        tolerance: f64,
    ) -> GraphHandle {
        let wire_edges = Wire::edges(store, wire);
        let mut vertices = Vec::new();
        let mut graph_edges = Vec::new();

        // Create a vertex for each edge midpoint
        let mut edge_to_vertex: HashMap<EdgeHandle, VertexHandle> = HashMap::new();
        for edge in &wire_edges {
            let start = Vertex::point(store, Edge::start_vertex(store, *edge));
            let end = Vertex::point(store, Edge::end_vertex(store, *edge));
            let midpoint = Point3::new(
                (start.x + end.x) / 2.0,
                (start.y + end.y) / 2.0,
                (start.z + end.z) / 2.0,
            );
            let v = Vertex::by_point(store, midpoint);
            edge_to_vertex.insert(*edge, v);
            vertices.push(v);
        }

        // Create graph edges between edges that share a vertex
        if direct {
            // Build a map from vertex geometry key to the edges that contain it
            let mut vertex_to_edges: HashMap<String, Vec<EdgeHandle>> = HashMap::new();
            for edge in &wire_edges {
                let (start, end) = Edge::vertices(store, *edge);
                let start_key = Self::vertex_geometry_key(store, start, tolerance);
                let end_key = Self::vertex_geometry_key(store, end, tolerance);
                vertex_to_edges.entry(start_key).or_default().push(*edge);
                vertex_to_edges.entry(end_key).or_default().push(*edge);
            }

            // Connect edges that share a vertex
            let mut seen_pairs: HashSet<(usize, usize)> = HashSet::new();
            for (_key, sharing_edges) in vertex_to_edges.iter() {
                if sharing_edges.len() >= 2 {
                    for i in 0..sharing_edges.len() {
                        for j in (i + 1)..sharing_edges.len() {
                            let e1 = sharing_edges[i];
                            let e2 = sharing_edges[j];
                            let v1 = edge_to_vertex[&e1];
                            let v2 = edge_to_vertex[&e2];

                            let idx1 = vertices.iter().position(|v| *v == v1).unwrap_or(0);
                            let idx2 = vertices.iter().position(|v| *v == v2).unwrap_or(0);
                            let pair = if idx1 < idx2 { (idx1, idx2) } else { (idx2, idx1) };

                            if !seen_pairs.contains(&pair) {
                                seen_pairs.insert(pair);
                                graph_edges.push(Edge::by_start_vertex_end_vertex(store, v1, v2));
                            }
                        }
                    }
                }
            }
        }

        Self::by_vertices_edges(store, vertices, graph_edges)
    }

    /// Create a graph from a single edge (just the midpoint vertex)
    fn from_edge_single(store: &TopologyStore, edge: EdgeHandle) -> GraphHandle {
        let start = Vertex::point(store, Edge::start_vertex(store, edge));
        let end = Vertex::point(store, Edge::end_vertex(store, edge));
        let midpoint = Point3::new(
            (start.x + end.x) / 2.0,
            (start.y + end.y) / 2.0,
            (start.z + end.z) / 2.0,
        );
        let v = Vertex::by_point(store, midpoint);
        Self::by_vertices_edges(store, vec![v], vec![])
    }

    /// Create a graph from a single vertex
    fn from_vertex(store: &TopologyStore, vertex: VertexHandle) -> GraphHandle {
        Self::by_vertices_edges(store, vec![vertex], vec![])
    }

    /// Create a geometry key for a face (sorted vertex coordinates)
    fn face_geometry_key(store: &TopologyStore, face: FaceHandle, tolerance: f64) -> String {
        let vertices = Face::vertices(store, face);
        let mut coords: Vec<(i64, i64, i64)> = vertices
            .iter()
            .map(|v| {
                let p = Vertex::point(store, *v);
                let scale = 1.0 / tolerance;
                (
                    (p.x * scale).round() as i64,
                    (p.y * scale).round() as i64,
                    (p.z * scale).round() as i64,
                )
            })
            .collect();
        coords.sort();
        format!("{:?}", coords)
    }

    /// Create a geometry key for an edge (sorted vertex coordinates)
    fn edge_geometry_key(store: &TopologyStore, edge: EdgeHandle, tolerance: f64) -> String {
        let (start, end) = Edge::vertices(store, edge);
        let p1 = Vertex::point(store, start);
        let p2 = Vertex::point(store, end);
        let scale = 1.0 / tolerance;
        let mut coords = vec![
            ((p1.x * scale).round() as i64, (p1.y * scale).round() as i64, (p1.z * scale).round() as i64),
            ((p2.x * scale).round() as i64, (p2.y * scale).round() as i64, (p2.z * scale).round() as i64),
        ];
        coords.sort();
        format!("{:?}", coords)
    }

    /// Create a geometry key for a vertex
    fn vertex_geometry_key(store: &TopologyStore, vertex: VertexHandle, tolerance: f64) -> String {
        let p = Vertex::point(store, vertex);
        let scale = 1.0 / tolerance;
        format!(
            "({},{},{})",
            (p.x * scale).round() as i64,
            (p.y * scale).round() as i64,
            (p.z * scale).round() as i64
        )
    }

    /// Add an edge to the graph
    pub fn add_edge(
        store: &TopologyStore,
        graph: GraphHandle,
        edge: EdgeHandle,
    ) -> GraphHandle {
        let mut edges = Cluster::edges(store, graph.edges);
        edges.push(edge);

        let vertices = Cluster::vertices(store, graph.vertices);
        Self::by_vertices_edges(store, vertices, edges)
    }

    /// Add a vertex to the graph
    pub fn add_vertex(
        store: &TopologyStore,
        graph: GraphHandle,
        vertex: VertexHandle,
    ) -> GraphHandle {
        let edges = Cluster::edges(store, graph.edges);
        let mut vertices = Cluster::vertices(store, graph.vertices);
        vertices.push(vertex);

        Self::by_vertices_edges(store, vertices, edges)
    }

    /// Add multiple vertices to the graph
    pub fn add_vertices(
        store: &TopologyStore,
        graph: GraphHandle,
        new_vertices: Vec<VertexHandle>,
    ) -> GraphHandle {
        let edges = Cluster::edges(store, graph.edges);
        let mut vertices = Cluster::vertices(store, graph.vertices);
        vertices.extend(new_vertices);

        Self::by_vertices_edges(store, vertices, edges)
    }

    /// Get the adjacency list representation
    pub fn adjacency_list(
        store: &TopologyStore,
        graph: GraphHandle,
        tolerance: f64,
    ) -> Vec<Vec<usize>> {
        let vertices = Cluster::vertices(store, graph.vertices);
        let edges = Cluster::edges(store, graph.edges);
        let n = vertices.len();

        let mut adj = vec![vec![]; n];

        // Map vertex handles to indices
        let vertex_to_idx: HashMap<VertexHandle, usize> = vertices
            .iter()
            .enumerate()
            .map(|(i, v)| (*v, i))
            .collect();

        // Build adjacency list from edges
        for edge in edges {
            let start = Edge::start_vertex(store, edge);
            let end = Edge::end_vertex(store, edge);

            // Find matching vertices by proximity
            let start_idx = Self::find_nearest_vertex_idx(store, &vertices, start, tolerance);
            let end_idx = Self::find_nearest_vertex_idx(store, &vertices, end, tolerance);

            if let (Some(s), Some(e)) = (start_idx, end_idx) {
                if s != e {
                    adj[s].push(e);
                    adj[e].push(s); // Undirected graph
                }
            }
        }

        adj
    }

    /// Get the adjacency matrix representation
    pub fn adjacency_matrix(
        store: &TopologyStore,
        graph: GraphHandle,
        tolerance: f64,
    ) -> Vec<Vec<i32>> {
        let adj_list = Self::adjacency_list(store, graph, tolerance);
        let n = adj_list.len();

        let mut matrix = vec![vec![0; n]; n];
        for (i, neighbors) in adj_list.iter().enumerate() {
            for &j in neighbors {
                matrix[i][j] = 1;
            }
        }

        matrix
    }

    /// Get vertices adjacent to a given vertex
    pub fn adjacent_vertices(
        store: &TopologyStore,
        graph: GraphHandle,
        vertex: VertexHandle,
        tolerance: f64,
    ) -> Vec<VertexHandle> {
        let vertices = Cluster::vertices(store, graph.vertices);
        let edges = Cluster::edges(store, graph.edges);
        let target_point = Vertex::point(store, vertex);

        let mut result = Vec::new();

        for edge in edges {
            let start = Edge::start_vertex(store, edge);
            let end = Edge::end_vertex(store, edge);
            let start_point = Vertex::point(store, start);
            let end_point = Vertex::point(store, end);

            if start_point.distance(target_point) < tolerance {
                // Find the matching end vertex in graph
                if let Some(v) = Self::find_nearest_vertex(store, &vertices, end, tolerance) {
                    result.push(v);
                }
            } else if end_point.distance(target_point) < tolerance {
                // Find the matching start vertex in graph
                if let Some(v) = Self::find_nearest_vertex(store, &vertices, start, tolerance) {
                    result.push(v);
                }
            }
        }

        result
    }

    /// Find all paths between two vertices
    pub fn all_paths(
        store: &TopologyStore,
        graph: GraphHandle,
        start: VertexHandle,
        end: VertexHandle,
        tolerance: f64,
        time_limit_secs: u64,
    ) -> Vec<Vec<VertexHandle>> {
        let vertices = Cluster::vertices(store, graph.vertices);
        let adj_list = Self::adjacency_list(store, graph, tolerance);

        let start_idx = Self::find_nearest_vertex_idx(store, &vertices, start, tolerance);
        let end_idx = Self::find_nearest_vertex_idx(store, &vertices, end, tolerance);

        let (start_idx, end_idx) = match (start_idx, end_idx) {
            (Some(s), Some(e)) => (s, e),
            _ => return vec![],
        };

        let mut all_paths = Vec::new();
        let mut current_path = vec![start_idx];
        let mut visited = vec![false; vertices.len()];
        visited[start_idx] = true;

        Self::dfs_all_paths(
            &adj_list,
            start_idx,
            end_idx,
            &mut visited,
            &mut current_path,
            &mut all_paths,
        );

        // Convert indices back to vertex handles
        all_paths
            .into_iter()
            .map(|path| path.into_iter().map(|i| vertices[i]).collect())
            .collect()
    }

    fn dfs_all_paths(
        adj: &[Vec<usize>],
        current: usize,
        target: usize,
        visited: &mut [bool],
        path: &mut Vec<usize>,
        all_paths: &mut Vec<Vec<usize>>,
    ) {
        if current == target {
            all_paths.push(path.clone());
            return;
        }

        for &next in &adj[current] {
            if !visited[next] {
                visited[next] = true;
                path.push(next);
                Self::dfs_all_paths(adj, next, target, visited, path, all_paths);
                path.pop();
                visited[next] = false;
            }
        }
    }

    /// Check if the graph contains an edge
    pub fn contains_edge(
        store: &TopologyStore,
        graph: GraphHandle,
        edge: EdgeHandle,
        tolerance: f64,
    ) -> bool {
        let edges = Cluster::edges(store, graph.edges);
        let target_start = Vertex::point(store, Edge::start_vertex(store, edge));
        let target_end = Vertex::point(store, Edge::end_vertex(store, edge));

        for e in edges {
            let start = Vertex::point(store, Edge::start_vertex(store, e));
            let end = Vertex::point(store, Edge::end_vertex(store, e));

            if (start.distance(target_start) < tolerance && end.distance(target_end) < tolerance)
                || (start.distance(target_end) < tolerance && end.distance(target_start) < tolerance)
            {
                return true;
            }
        }

        false
    }

    /// Check if the graph contains a vertex
    pub fn contains_vertex(
        store: &TopologyStore,
        graph: GraphHandle,
        vertex: VertexHandle,
        tolerance: f64,
    ) -> bool {
        let vertices = Cluster::vertices(store, graph.vertices);
        let target = Vertex::point(store, vertex);

        for v in vertices {
            if Vertex::point(store, v).distance(target) < tolerance {
                return true;
            }
        }

        false
    }

    /// Get the degree sequence of the graph
    pub fn degree_sequence(
        store: &TopologyStore,
        graph: GraphHandle,
        tolerance: f64,
    ) -> Vec<usize> {
        let adj_list = Self::adjacency_list(store, graph, tolerance);
        let mut degrees: Vec<usize> = adj_list.iter().map(|neighbors| neighbors.len()).collect();
        degrees.sort_by(|a, b| b.cmp(a)); // Sort descending
        degrees
    }

    /// Calculate the density of the graph
    pub fn density(store: &TopologyStore, graph: GraphHandle) -> f64 {
        let n = Cluster::size(store, graph.vertices);
        let m = Cluster::size(store, graph.edges);

        if n < 2 {
            return 0.0;
        }

        let max_edges = n * (n - 1) / 2;
        if max_edges == 0 {
            return 0.0;
        }

        m as f64 / max_edges as f64
    }

    /// Get the depth map from a source vertex (BFS distances)
    pub fn depth_map(
        store: &TopologyStore,
        graph: GraphHandle,
        source: VertexHandle,
        tolerance: f64,
    ) -> Vec<i32> {
        let vertices = Cluster::vertices(store, graph.vertices);
        let adj_list = Self::adjacency_list(store, graph, tolerance);
        let n = vertices.len();

        let mut depths = vec![-1i32; n];

        let source_idx = Self::find_nearest_vertex_idx(store, &vertices, source, tolerance);
        let source_idx = match source_idx {
            Some(i) => i,
            None => return depths,
        };

        let mut queue = VecDeque::new();
        queue.push_back(source_idx);
        depths[source_idx] = 0;

        while let Some(current) = queue.pop_front() {
            let current_depth = depths[current];
            for &neighbor in &adj_list[current] {
                if depths[neighbor] < 0 {
                    depths[neighbor] = current_depth + 1;
                    queue.push_back(neighbor);
                }
            }
        }

        depths
    }

    /// Get the diameter of the graph (longest shortest path)
    pub fn diameter(
        store: &TopologyStore,
        graph: GraphHandle,
        tolerance: f64,
    ) -> i32 {
        let vertices = Cluster::vertices(store, graph.vertices);
        let mut max_dist = 0;

        for v in &vertices {
            let depths = Self::depth_map(store, graph, *v, tolerance);
            for &d in &depths {
                if d > max_dist {
                    max_dist = d;
                }
            }
        }

        max_dist
    }

    /// Get the topological distance between two vertices
    pub fn distance(
        store: &TopologyStore,
        graph: GraphHandle,
        v1: VertexHandle,
        v2: VertexHandle,
        tolerance: f64,
    ) -> Option<i32> {
        let vertices = Cluster::vertices(store, graph.vertices);

        let idx1 = Self::find_nearest_vertex_idx(store, &vertices, v1, tolerance)?;
        let idx2 = Self::find_nearest_vertex_idx(store, &vertices, v2, tolerance)?;

        let depths = Self::depth_map(store, graph, v1, tolerance);
        let dist = depths[idx2];

        if dist >= 0 {
            Some(dist)
        } else {
            None
        }
    }

    /// Get an edge between two vertices (if it exists)
    pub fn edge(
        store: &TopologyStore,
        graph: GraphHandle,
        v1: VertexHandle,
        v2: VertexHandle,
        tolerance: f64,
    ) -> Option<EdgeHandle> {
        let edges = Cluster::edges(store, graph.edges);
        let p1 = Vertex::point(store, v1);
        let p2 = Vertex::point(store, v2);

        for e in edges {
            let start = Vertex::point(store, Edge::start_vertex(store, e));
            let end = Vertex::point(store, Edge::end_vertex(store, e));

            if (start.distance(p1) < tolerance && end.distance(p2) < tolerance)
                || (start.distance(p2) < tolerance && end.distance(p1) < tolerance)
            {
                return Some(e);
            }
        }

        None
    }

    /// Get all edges in the graph
    pub fn edges(store: &TopologyStore, graph: GraphHandle) -> Vec<EdgeHandle> {
        Cluster::edges(store, graph.edges)
    }

    /// Get all vertices in the graph
    pub fn vertices(store: &TopologyStore, graph: GraphHandle) -> Vec<VertexHandle> {
        Cluster::vertices(store, graph.vertices)
    }

    /// Check if the graph is bipartite
    pub fn is_bipartite(
        store: &TopologyStore,
        graph: GraphHandle,
        tolerance: f64,
    ) -> bool {
        let adj_list = Self::adjacency_list(store, graph, tolerance);
        let n = adj_list.len();

        if n == 0 {
            return true;
        }

        let mut colors = vec![-1i32; n];
        let mut queue = VecDeque::new();

        for start in 0..n {
            if colors[start] >= 0 {
                continue;
            }

            queue.push_back(start);
            colors[start] = 0;

            while let Some(current) = queue.pop_front() {
                let current_color = colors[current];
                let next_color = 1 - current_color;

                for &neighbor in &adj_list[current] {
                    if colors[neighbor] < 0 {
                        colors[neighbor] = next_color;
                        queue.push_back(neighbor);
                    } else if colors[neighbor] == current_color {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Check if the graph is complete
    pub fn is_complete(store: &TopologyStore, graph: GraphHandle) -> bool {
        let n = Cluster::size(store, graph.vertices);
        let m = Cluster::size(store, graph.edges);

        if n < 2 {
            return true;
        }

        let expected_edges = n * (n - 1) / 2;
        m == expected_edges
    }

    /// Get isolated vertices (vertices with no edges)
    pub fn isolated_vertices(
        store: &TopologyStore,
        graph: GraphHandle,
        tolerance: f64,
    ) -> Vec<VertexHandle> {
        let vertices = Cluster::vertices(store, graph.vertices);
        let adj_list = Self::adjacency_list(store, graph, tolerance);

        vertices
            .into_iter()
            .enumerate()
            .filter(|(i, _)| adj_list[*i].is_empty())
            .map(|(_, v)| v)
            .collect()
    }

    /// Get the maximum degree in the graph
    pub fn maximum_delta(
        store: &TopologyStore,
        graph: GraphHandle,
        tolerance: f64,
    ) -> usize {
        let adj_list = Self::adjacency_list(store, graph, tolerance);
        adj_list.iter().map(|neighbors| neighbors.len()).max().unwrap_or(0)
    }

    /// Get the minimum degree in the graph
    pub fn minimum_delta(
        store: &TopologyStore,
        graph: GraphHandle,
        tolerance: f64,
    ) -> usize {
        let adj_list = Self::adjacency_list(store, graph, tolerance);
        adj_list.iter().map(|neighbors| neighbors.len()).min().unwrap_or(0)
    }

    /// Find the nearest vertex to a target vertex
    pub fn nearest_vertex(
        store: &TopologyStore,
        graph: GraphHandle,
        target: VertexHandle,
    ) -> Option<VertexHandle> {
        let vertices = Cluster::vertices(store, graph.vertices);
        let target_point = Vertex::point(store, target);

        let mut nearest = None;
        let mut min_dist = f64::MAX;

        for v in vertices {
            let dist = Vertex::point(store, v).distance(target_point);
            if dist < min_dist {
                min_dist = dist;
                nearest = Some(v);
            }
        }

        nearest
    }

    /// Get the order (number of vertices) of the graph
    pub fn order(store: &TopologyStore, graph: GraphHandle) -> usize {
        Cluster::size(store, graph.vertices)
    }

    /// Get the size (number of edges) of the graph
    pub fn size(store: &TopologyStore, graph: GraphHandle) -> usize {
        Cluster::size(store, graph.edges)
    }

    /// Find the shortest path between two vertices (as a wire)
    pub fn shortest_path(
        store: &TopologyStore,
        graph: GraphHandle,
        start: VertexHandle,
        end: VertexHandle,
        tolerance: f64,
    ) -> Option<WireHandle> {
        let path_vertices = Self::shortest_path_vertices(store, graph, start, end, tolerance)?;

        if path_vertices.len() < 2 {
            return None;
        }

        Some(Wire::by_vertices(store, path_vertices, false))
    }

    /// Find the shortest path as a list of vertices
    pub fn shortest_path_vertices(
        store: &TopologyStore,
        graph: GraphHandle,
        start: VertexHandle,
        end: VertexHandle,
        tolerance: f64,
    ) -> Option<Vec<VertexHandle>> {
        let vertices = Cluster::vertices(store, graph.vertices);
        let adj_list = Self::adjacency_list(store, graph, tolerance);

        let start_idx = Self::find_nearest_vertex_idx(store, &vertices, start, tolerance)?;
        let end_idx = Self::find_nearest_vertex_idx(store, &vertices, end, tolerance)?;

        // BFS
        let n = vertices.len();
        let mut visited = vec![false; n];
        let mut parent = vec![None; n];
        let mut queue = VecDeque::new();

        queue.push_back(start_idx);
        visited[start_idx] = true;

        while let Some(current) = queue.pop_front() {
            if current == end_idx {
                break;
            }

            for &neighbor in &adj_list[current] {
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    parent[neighbor] = Some(current);
                    queue.push_back(neighbor);
                }
            }
        }

        if !visited[end_idx] {
            return None;
        }

        // Reconstruct path
        let mut path = Vec::new();
        let mut current = end_idx;
        while let Some(p) = parent[current] {
            path.push(vertices[current]);
            current = p;
        }
        path.push(vertices[start_idx]);
        path.reverse();

        Some(path)
    }

    /// Remove an edge from the graph
    pub fn remove_edge(
        store: &TopologyStore,
        graph: GraphHandle,
        edge: EdgeHandle,
        tolerance: f64,
    ) -> GraphHandle {
        let edges = Cluster::edges(store, graph.edges);
        let target_start = Vertex::point(store, Edge::start_vertex(store, edge));
        let target_end = Vertex::point(store, Edge::end_vertex(store, edge));

        let filtered_edges: Vec<_> = edges
            .into_iter()
            .filter(|e| {
                let start = Vertex::point(store, Edge::start_vertex(store, *e));
                let end = Vertex::point(store, Edge::end_vertex(store, *e));

                !((start.distance(target_start) < tolerance && end.distance(target_end) < tolerance)
                    || (start.distance(target_end) < tolerance
                        && end.distance(target_start) < tolerance))
            })
            .collect();

        let vertices = Cluster::vertices(store, graph.vertices);
        Self::by_vertices_edges(store, vertices, filtered_edges)
    }

    /// Remove a vertex from the graph (and its incident edges)
    pub fn remove_vertex(
        store: &TopologyStore,
        graph: GraphHandle,
        vertex: VertexHandle,
        tolerance: f64,
    ) -> GraphHandle {
        let vertices = Cluster::vertices(store, graph.vertices);
        let edges = Cluster::edges(store, graph.edges);
        let target = Vertex::point(store, vertex);

        let filtered_vertices: Vec<_> = vertices
            .into_iter()
            .filter(|v| Vertex::point(store, *v).distance(target) >= tolerance)
            .collect();

        let filtered_edges: Vec<_> = edges
            .into_iter()
            .filter(|e| {
                let start = Vertex::point(store, Edge::start_vertex(store, *e));
                let end = Vertex::point(store, Edge::end_vertex(store, *e));
                start.distance(target) >= tolerance && end.distance(target) >= tolerance
            })
            .collect();

        Self::by_vertices_edges(store, filtered_vertices, filtered_edges)
    }

    /// Get the vertex degree (number of incident edges)
    pub fn vertex_degree(
        store: &TopologyStore,
        graph: GraphHandle,
        vertex: VertexHandle,
        tolerance: f64,
    ) -> usize {
        let vertices = Cluster::vertices(store, graph.vertices);
        let adj_list = Self::adjacency_list(store, graph, tolerance);

        match Self::find_nearest_vertex_idx(store, &vertices, vertex, tolerance) {
            Some(idx) => adj_list[idx].len(),
            None => 0,
        }
    }

    /// Convert the graph to a topology (cluster of vertices and edges)
    pub fn topology(store: &TopologyStore, graph: GraphHandle) -> ClusterHandle {
        let vertices = Cluster::vertices(store, graph.vertices);
        let edges = Cluster::edges(store, graph.edges);

        let mut handles: Vec<TopologyHandle> = vertices
            .into_iter()
            .map(TopologyHandle::Vertex)
            .collect();
        handles.extend(edges.into_iter().map(TopologyHandle::Edge));

        Cluster::by_topologies(store, handles)
    }

    // Helper functions

    fn find_nearest_vertex(
        store: &TopologyStore,
        vertices: &[VertexHandle],
        target: VertexHandle,
        tolerance: f64,
    ) -> Option<VertexHandle> {
        let target_point = Vertex::point(store, target);

        for &v in vertices {
            if Vertex::point(store, v).distance(target_point) < tolerance {
                return Some(v);
            }
        }

        None
    }

    fn find_nearest_vertex_idx(
        store: &TopologyStore,
        vertices: &[VertexHandle],
        target: VertexHandle,
        tolerance: f64,
    ) -> Option<usize> {
        let target_point = Vertex::point(store, target);

        for (i, &v) in vertices.iter().enumerate() {
            if Vertex::point(store, v).distance(target_point) < tolerance {
                return Some(i);
            }
        }

        None
    }

    // ========================================================================
    // NEW METHODS: Graph construction and analysis
    // ========================================================================

    /// Create a graph from an adjacency matrix
    pub fn by_adjacency_matrix(
        store: &TopologyStore,
        matrix: &[Vec<i32>],
        vertices: Option<Vec<VertexHandle>>,
    ) -> GraphHandle {
        let n = matrix.len();
        if n == 0 {
            return Self::by_vertices_edges(store, vec![], vec![]);
        }

        // Create vertices if not provided
        let graph_vertices: Vec<VertexHandle> = match vertices {
            Some(v) if v.len() == n => v,
            _ => {
                // Create vertices in a grid pattern
                (0..n)
                    .map(|i| {
                        let row = i / 10;
                        let col = i % 10;
                        Vertex::by_coordinates(store, col as f64, row as f64, 0.0)
                    })
                    .collect()
            }
        };

        // Create edges based on adjacency matrix
        let mut edges = Vec::new();
        for i in 0..n {
            for j in (i + 1)..n {
                if j < matrix[i].len() && matrix[i][j] != 0 {
                    edges.push(Edge::by_start_vertex_end_vertex(
                        store,
                        graph_vertices[i],
                        graph_vertices[j],
                    ));
                }
            }
        }

        Self::by_vertices_edges(store, graph_vertices, edges)
    }

    /// Calculate betweenness centrality for all vertices using Brandes' algorithm
    pub fn betweenness_centrality(
        store: &TopologyStore,
        graph: GraphHandle,
        tolerance: f64,
        normalized: bool,
    ) -> Vec<f64> {
        let vertices = Cluster::vertices(store, graph.vertices);
        let adj_list = Self::adjacency_list(store, graph, tolerance);
        let n = vertices.len();

        if n <= 1 {
            return vec![0.0; n];
        }

        let mut centrality = vec![0.0; n];

        // Brandes' algorithm
        for s in 0..n {
            // Single-source shortest paths
            let mut stack = Vec::new();
            let mut pred: Vec<Vec<usize>> = vec![vec![]; n];
            let mut sigma = vec![0.0_f64; n];
            sigma[s] = 1.0;
            let mut dist = vec![-1_i32; n];
            dist[s] = 0;

            let mut queue = VecDeque::new();
            queue.push_back(s);

            while let Some(v) = queue.pop_front() {
                stack.push(v);
                for &w in &adj_list[v] {
                    // w found for the first time?
                    if dist[w] < 0 {
                        dist[w] = dist[v] + 1;
                        queue.push_back(w);
                    }
                    // shortest path to w via v?
                    if dist[w] == dist[v] + 1 {
                        sigma[w] += sigma[v];
                        pred[w].push(v);
                    }
                }
            }

            // Accumulation
            let mut delta = vec![0.0_f64; n];
            while let Some(w) = stack.pop() {
                for &v in &pred[w] {
                    delta[v] += (sigma[v] / sigma[w]) * (1.0 + delta[w]);
                }
                if w != s {
                    centrality[w] += delta[w];
                }
            }
        }

        // For undirected graphs, divide by 2
        for c in centrality.iter_mut() {
            *c /= 2.0;
        }

        // Normalize if requested
        if normalized && n > 2 {
            let scale = 1.0 / ((n - 1) * (n - 2)) as f64;
            for c in centrality.iter_mut() {
                *c *= scale * 2.0;
            }
        }

        centrality
    }

    /// Calculate PageRank for all vertices
    pub fn page_rank(
        store: &TopologyStore,
        graph: GraphHandle,
        tolerance: f64,
        damping: f64,
        max_iterations: usize,
        convergence: f64,
    ) -> Vec<f64> {
        let vertices = Cluster::vertices(store, graph.vertices);
        let adj_list = Self::adjacency_list(store, graph, tolerance);
        let n = vertices.len();

        if n == 0 {
            return vec![];
        }

        // Initialize PageRank values
        let mut ranks = vec![1.0 / n as f64; n];
        let mut new_ranks = vec![0.0; n];

        // Get out-degrees
        let out_degrees: Vec<usize> = adj_list.iter().map(|neighbors| neighbors.len()).collect();

        // Power iteration
        for _ in 0..max_iterations {
            // Reset new ranks with damping factor contribution
            for r in new_ranks.iter_mut() {
                *r = (1.0 - damping) / n as f64;
            }

            // Distribute rank from each vertex to its neighbors
            for (i, neighbors) in adj_list.iter().enumerate() {
                if !neighbors.is_empty() {
                    let contrib = damping * ranks[i] / neighbors.len() as f64;
                    for &j in neighbors {
                        new_ranks[j] += contrib;
                    }
                } else {
                    // Dangling node: distribute to all nodes
                    let contrib = damping * ranks[i] / n as f64;
                    for r in new_ranks.iter_mut() {
                        *r += contrib;
                    }
                }
            }

            // Check convergence
            let diff: f64 = ranks
                .iter()
                .zip(new_ranks.iter())
                .map(|(a, b)| (a - b).abs())
                .sum();

            std::mem::swap(&mut ranks, &mut new_ranks);

            if diff < convergence {
                break;
            }
        }

        ranks
    }

    /// Find bridge edges (edges whose removal disconnects the graph)
    pub fn bridges(
        store: &TopologyStore,
        graph: GraphHandle,
        tolerance: f64,
    ) -> Vec<EdgeHandle> {
        let vertices = Cluster::vertices(store, graph.vertices);
        let edges = Cluster::edges(store, graph.edges);
        let adj_list = Self::adjacency_list(store, graph, tolerance);
        let n = vertices.len();

        if n == 0 {
            return vec![];
        }

        let mut visited = vec![false; n];
        let mut disc = vec![0_u32; n];
        let mut low = vec![0_u32; n];
        let mut bridge_indices: Vec<(usize, usize)> = Vec::new();
        let mut time = 0_u32;

        fn dfs(
            u: usize,
            parent: Option<usize>,
            adj: &[Vec<usize>],
            visited: &mut [bool],
            disc: &mut [u32],
            low: &mut [u32],
            time: &mut u32,
            bridges: &mut Vec<(usize, usize)>,
        ) {
            visited[u] = true;
            *time += 1;
            disc[u] = *time;
            low[u] = *time;

            for &v in &adj[u] {
                if !visited[v] {
                    dfs(v, Some(u), adj, visited, disc, low, time, bridges);
                    low[u] = low[u].min(low[v]);

                    if low[v] > disc[u] {
                        bridges.push((u.min(v), u.max(v)));
                    }
                } else if Some(v) != parent {
                    low[u] = low[u].min(disc[v]);
                }
            }
        }

        for i in 0..n {
            if !visited[i] {
                dfs(i, None, &adj_list, &mut visited, &mut disc, &mut low, &mut time, &mut bridge_indices);
            }
        }

        // Convert vertex index pairs to edge handles
        let mut result = Vec::new();
        for (i, j) in bridge_indices {
            let p1 = Vertex::point(store, vertices[i]);
            let p2 = Vertex::point(store, vertices[j]);

            for &e in &edges {
                let start = Vertex::point(store, Edge::start_vertex(store, e));
                let end = Vertex::point(store, Edge::end_vertex(store, e));

                if (start.distance(p1) < tolerance && end.distance(p2) < tolerance)
                    || (start.distance(p2) < tolerance && end.distance(p1) < tolerance)
                {
                    result.push(e);
                    break;
                }
            }
        }

        result
    }

    /// Find cut vertices (articulation points whose removal disconnects the graph)
    pub fn cut_vertices(
        store: &TopologyStore,
        graph: GraphHandle,
        tolerance: f64,
    ) -> Vec<VertexHandle> {
        let vertices = Cluster::vertices(store, graph.vertices);
        let adj_list = Self::adjacency_list(store, graph, tolerance);
        let n = vertices.len();

        if n == 0 {
            return vec![];
        }

        let mut visited = vec![false; n];
        let mut disc = vec![0_u32; n];
        let mut low = vec![0_u32; n];
        let mut parent = vec![None::<usize>; n];
        let mut cut_vertex_flags = vec![false; n];
        let mut time = 0_u32;

        fn dfs(
            u: usize,
            adj: &[Vec<usize>],
            visited: &mut [bool],
            disc: &mut [u32],
            low: &mut [u32],
            parent: &mut [Option<usize>],
            cut_vertex_flags: &mut [bool],
            time: &mut u32,
        ) {
            let mut children = 0;
            visited[u] = true;
            *time += 1;
            disc[u] = *time;
            low[u] = *time;

            for &v in &adj[u] {
                if !visited[v] {
                    children += 1;
                    parent[v] = Some(u);
                    dfs(v, adj, visited, disc, low, parent, cut_vertex_flags, time);
                    low[u] = low[u].min(low[v]);

                    // u is a cut vertex if:
                    // 1) u is root of DFS tree and has two or more children
                    // 2) u is not root and low[v] >= disc[u]
                    if parent[u].is_none() && children > 1 {
                        cut_vertex_flags[u] = true;
                    }
                    if parent[u].is_some() && low[v] >= disc[u] {
                        cut_vertex_flags[u] = true;
                    }
                } else if Some(v) != parent[u] {
                    low[u] = low[u].min(disc[v]);
                }
            }
        }

        for i in 0..n {
            if !visited[i] {
                dfs(i, &adj_list, &mut visited, &mut disc, &mut low, &mut parent, &mut cut_vertex_flags, &mut time);
            }
        }

        vertices
            .into_iter()
            .enumerate()
            .filter(|(i, _)| cut_vertex_flags[*i])
            .map(|(_, v)| v)
            .collect()
    }

    /// Reshape graph vertices using force-directed layout (spring algorithm)
    pub fn reshape(
        store: &TopologyStore,
        graph: GraphHandle,
        tolerance: f64,
        shape: &str,
        k: Option<f64>,
        iterations: Option<usize>,
        root_vertex: Option<VertexHandle>,
    ) -> GraphHandle {
        let vertices = Cluster::vertices(store, graph.vertices);
        let edges = Cluster::edges(store, graph.edges);
        let n = vertices.len();

        if n == 0 {
            return Self::by_vertices_edges(store, vec![], vec![]);
        }

        let adj_list = Self::adjacency_list(store, graph, tolerance);

        // Calculate new positions based on shape
        let new_positions: Vec<Point3> = match shape.to_lowercase().as_str() {
            "spring" | "spring 2d" | "spring2d" => {
                Self::spring_layout_2d(&adj_list, n, k.unwrap_or(0.1), iterations.unwrap_or(50))
            }
            "spring 3d" | "spring3d" => {
                Self::spring_layout_3d(&adj_list, n, k.unwrap_or(0.1), iterations.unwrap_or(50))
            }
            "circle" | "circular" => Self::circular_layout(n),
            "tree" | "tree 2d" | "tree2d" => {
                let root = root_vertex.and_then(|v| {
                    Self::find_nearest_vertex_idx(store, &vertices, v, tolerance)
                }).unwrap_or(0);
                Self::tree_layout_2d(&adj_list, n, root)
            }
            "grid" => Self::grid_layout(n),
            _ => Self::spring_layout_2d(&adj_list, n, k.unwrap_or(0.1), iterations.unwrap_or(50)),
        };

        // Create new vertices at new positions
        let new_vertices: Vec<VertexHandle> = new_positions
            .iter()
            .map(|p| Vertex::by_point(store, *p))
            .collect();

        // Recreate edges connecting the new vertices
        let mut new_edges = Vec::new();
        for i in 0..n {
            for &j in &adj_list[i] {
                if i < j {
                    new_edges.push(Edge::by_start_vertex_end_vertex(
                        store,
                        new_vertices[i],
                        new_vertices[j],
                    ));
                }
            }
        }

        Self::by_vertices_edges(store, new_vertices, new_edges)
    }

    fn spring_layout_2d(adj: &[Vec<usize>], n: usize, k: f64, iterations: usize) -> Vec<Point3> {
        let mut positions: Vec<[f64; 2]> = (0..n)
            .map(|i| {
                let angle = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
                [angle.cos(), angle.sin()]
            })
            .collect();

        let area = 1.0;
        let k_opt = k * (area / n as f64).sqrt();

        for _ in 0..iterations {
            let mut disp = vec![[0.0, 0.0]; n];

            // Repulsive forces
            for i in 0..n {
                for j in (i + 1)..n {
                    let dx = positions[i][0] - positions[j][0];
                    let dy = positions[i][1] - positions[j][1];
                    let dist = (dx * dx + dy * dy).sqrt().max(0.01);
                    let force = k_opt * k_opt / dist;

                    let fx = dx / dist * force;
                    let fy = dy / dist * force;

                    disp[i][0] += fx;
                    disp[i][1] += fy;
                    disp[j][0] -= fx;
                    disp[j][1] -= fy;
                }
            }

            // Attractive forces
            for i in 0..n {
                for &j in &adj[i] {
                    if i < j {
                        let dx = positions[i][0] - positions[j][0];
                        let dy = positions[i][1] - positions[j][1];
                        let dist = (dx * dx + dy * dy).sqrt().max(0.01);
                        let force = dist * dist / k_opt;

                        let fx = dx / dist * force;
                        let fy = dy / dist * force;

                        disp[i][0] -= fx;
                        disp[i][1] -= fy;
                        disp[j][0] += fx;
                        disp[j][1] += fy;
                    }
                }
            }

            // Apply displacements with temperature
            let temp: f64 = 0.1;
            for i in 0..n {
                let mag = (disp[i][0].powi(2) + disp[i][1].powi(2)).sqrt().max(0.01);
                positions[i][0] += disp[i][0] / mag * temp.min(mag);
                positions[i][1] += disp[i][1] / mag * temp.min(mag);
            }
        }

        positions.into_iter().map(|[x, y]| Point3::new(x, y, 0.0)).collect()
    }

    fn spring_layout_3d(adj: &[Vec<usize>], n: usize, k: f64, iterations: usize) -> Vec<Point3> {
        let mut positions: Vec<[f64; 3]> = (0..n)
            .map(|i| {
                let phi = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
                let theta = std::f64::consts::PI * (i as f64 / n as f64);
                [
                    theta.sin() * phi.cos(),
                    theta.sin() * phi.sin(),
                    theta.cos(),
                ]
            })
            .collect();

        let k_opt = k;

        for _ in 0..iterations {
            let mut disp = vec![[0.0, 0.0, 0.0]; n];

            // Repulsive forces
            for i in 0..n {
                for j in (i + 1)..n {
                    let dx = positions[i][0] - positions[j][0];
                    let dy = positions[i][1] - positions[j][1];
                    let dz = positions[i][2] - positions[j][2];
                    let dist = (dx * dx + dy * dy + dz * dz).sqrt().max(0.01);
                    let force = k_opt * k_opt / dist;

                    disp[i][0] += dx / dist * force;
                    disp[i][1] += dy / dist * force;
                    disp[i][2] += dz / dist * force;
                    disp[j][0] -= dx / dist * force;
                    disp[j][1] -= dy / dist * force;
                    disp[j][2] -= dz / dist * force;
                }
            }

            // Attractive forces
            for i in 0..n {
                for &j in &adj[i] {
                    if i < j {
                        let dx = positions[i][0] - positions[j][0];
                        let dy = positions[i][1] - positions[j][1];
                        let dz = positions[i][2] - positions[j][2];
                        let dist = (dx * dx + dy * dy + dz * dz).sqrt().max(0.01);
                        let force = dist * dist / k_opt;

                        disp[i][0] -= dx / dist * force;
                        disp[i][1] -= dy / dist * force;
                        disp[i][2] -= dz / dist * force;
                        disp[j][0] += dx / dist * force;
                        disp[j][1] += dy / dist * force;
                        disp[j][2] += dz / dist * force;
                    }
                }
            }

            // Apply displacements
            let temp: f64 = 0.1;
            for i in 0..n {
                let mag = (disp[i][0].powi(2) + disp[i][1].powi(2) + disp[i][2].powi(2)).sqrt().max(0.01);
                positions[i][0] += disp[i][0] / mag * temp.min(mag);
                positions[i][1] += disp[i][1] / mag * temp.min(mag);
                positions[i][2] += disp[i][2] / mag * temp.min(mag);
            }
        }

        positions.into_iter().map(|[x, y, z]| Point3::new(x, y, z)).collect()
    }

    fn circular_layout(n: usize) -> Vec<Point3> {
        (0..n)
            .map(|i| {
                let angle = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
                Point3::new(angle.cos(), angle.sin(), 0.0)
            })
            .collect()
    }

    fn tree_layout_2d(adj: &[Vec<usize>], n: usize, root: usize) -> Vec<Point3> {
        if n == 0 {
            return vec![];
        }

        let mut positions = vec![Point3::new(0.0, 0.0, 0.0); n];
        let mut visited = vec![false; n];
        let mut levels: Vec<Vec<usize>> = Vec::new();
        let mut queue = VecDeque::new();

        queue.push_back(root);
        visited[root] = true;
        levels.push(vec![root]);

        while !queue.is_empty() {
            let mut next_level = Vec::new();
            while let Some(u) = queue.pop_front() {
                for &v in &adj[u] {
                    if !visited[v] {
                        visited[v] = true;
                        next_level.push(v);
                    }
                }
            }
            if !next_level.is_empty() {
                for &v in &next_level {
                    queue.push_back(v);
                }
                levels.push(next_level);
            }
        }

        // Position nodes by level
        for (level_idx, level) in levels.iter().enumerate() {
            let y = -(level_idx as f64);
            let width = level.len() as f64;
            for (i, &node) in level.iter().enumerate() {
                let x = if width > 1.0 {
                    (i as f64 - (width - 1.0) / 2.0) * (2.0 / width.max(1.0))
                } else {
                    0.0
                };
                positions[node] = Point3::new(x, y, 0.0);
            }
        }

        positions
    }

    fn grid_layout(n: usize) -> Vec<Point3> {
        let cols = (n as f64).sqrt().ceil() as usize;
        (0..n)
            .map(|i| {
                let row = i / cols;
                let col = i % cols;
                Point3::new(col as f64, -(row as f64), 0.0)
            })
            .collect()
    }

    /// Union of two graphs (combine vertices and edges)
    pub fn union(
        store: &TopologyStore,
        graph1: GraphHandle,
        graph2: GraphHandle,
        tolerance: f64,
    ) -> GraphHandle {
        let vertices1 = Cluster::vertices(store, graph1.vertices);
        let vertices2 = Cluster::vertices(store, graph2.vertices);
        let edges1 = Cluster::edges(store, graph1.edges);
        let edges2 = Cluster::edges(store, graph2.edges);

        // Combine vertices, removing duplicates by position
        let mut combined_vertices = vertices1.clone();
        for v2 in vertices2 {
            let p2 = Vertex::point(store, v2);
            let is_duplicate = vertices1.iter().any(|&v1| {
                Vertex::point(store, v1).distance(p2) < tolerance
            });
            if !is_duplicate {
                combined_vertices.push(v2);
            }
        }

        // Combine edges, removing duplicates
        let mut combined_edges = edges1.clone();
        for e2 in edges2 {
            let start2 = Vertex::point(store, Edge::start_vertex(store, e2));
            let end2 = Vertex::point(store, Edge::end_vertex(store, e2));

            let is_duplicate = edges1.iter().any(|&e1| {
                let start1 = Vertex::point(store, Edge::start_vertex(store, e1));
                let end1 = Vertex::point(store, Edge::end_vertex(store, e1));
                (start1.distance(start2) < tolerance && end1.distance(end2) < tolerance)
                    || (start1.distance(end2) < tolerance && end1.distance(start2) < tolerance)
            });
            if !is_duplicate {
                combined_edges.push(e2);
            }
        }

        Self::by_vertices_edges(store, combined_vertices, combined_edges)
    }

    /// Intersection of two graphs (common vertices and edges)
    pub fn intersect(
        store: &TopologyStore,
        graph1: GraphHandle,
        graph2: GraphHandle,
        tolerance: f64,
    ) -> GraphHandle {
        let vertices1 = Cluster::vertices(store, graph1.vertices);
        let vertices2 = Cluster::vertices(store, graph2.vertices);
        let edges1 = Cluster::edges(store, graph1.edges);
        let edges2 = Cluster::edges(store, graph2.edges);

        // Find common vertices
        let common_vertices: Vec<VertexHandle> = vertices1
            .iter()
            .filter(|&&v1| {
                let p1 = Vertex::point(store, v1);
                vertices2.iter().any(|&v2| {
                    Vertex::point(store, v2).distance(p1) < tolerance
                })
            })
            .copied()
            .collect();

        // Find common edges
        let common_edges: Vec<EdgeHandle> = edges1
            .iter()
            .filter(|&&e1| {
                let start1 = Vertex::point(store, Edge::start_vertex(store, e1));
                let end1 = Vertex::point(store, Edge::end_vertex(store, e1));
                edges2.iter().any(|&e2| {
                    let start2 = Vertex::point(store, Edge::start_vertex(store, e2));
                    let end2 = Vertex::point(store, Edge::end_vertex(store, e2));
                    (start1.distance(start2) < tolerance && end1.distance(end2) < tolerance)
                        || (start1.distance(end2) < tolerance && end1.distance(start2) < tolerance)
                })
            })
            .copied()
            .collect();

        Self::by_vertices_edges(store, common_vertices, common_edges)
    }

    /// Difference of two graphs (graph1 - graph2)
    pub fn difference(
        store: &TopologyStore,
        graph1: GraphHandle,
        graph2: GraphHandle,
        tolerance: f64,
    ) -> GraphHandle {
        let vertices1 = Cluster::vertices(store, graph1.vertices);
        let vertices2 = Cluster::vertices(store, graph2.vertices);
        let edges1 = Cluster::edges(store, graph1.edges);
        let edges2 = Cluster::edges(store, graph2.edges);

        // Vertices in graph1 but not in graph2
        let diff_vertices: Vec<VertexHandle> = vertices1
            .iter()
            .filter(|&&v1| {
                let p1 = Vertex::point(store, v1);
                !vertices2.iter().any(|&v2| {
                    Vertex::point(store, v2).distance(p1) < tolerance
                })
            })
            .copied()
            .collect();

        // Edges in graph1 but not in graph2
        let diff_edges: Vec<EdgeHandle> = edges1
            .iter()
            .filter(|&&e1| {
                let start1 = Vertex::point(store, Edge::start_vertex(store, e1));
                let end1 = Vertex::point(store, Edge::end_vertex(store, e1));
                !edges2.iter().any(|&e2| {
                    let start2 = Vertex::point(store, Edge::start_vertex(store, e2));
                    let end2 = Vertex::point(store, Edge::end_vertex(store, e2));
                    (start1.distance(start2) < tolerance && end1.distance(end2) < tolerance)
                        || (start1.distance(end2) < tolerance && end1.distance(start2) < tolerance)
                })
            })
            .copied()
            .collect();

        Self::by_vertices_edges(store, diff_vertices, diff_edges)
    }

    /// Symmetric difference of two graphs (XOR)
    pub fn symmetric_difference(
        store: &TopologyStore,
        graph1: GraphHandle,
        graph2: GraphHandle,
        tolerance: f64,
    ) -> GraphHandle {
        let diff1 = Self::difference(store, graph1, graph2, tolerance);
        let diff2 = Self::difference(store, graph2, graph1, tolerance);
        Self::union(store, diff1, diff2, tolerance)
    }

    /// Create a line graph (edge-to-vertex dual)
    pub fn line_graph(
        store: &TopologyStore,
        graph: GraphHandle,
        tolerance: f64,
    ) -> GraphHandle {
        let edges = Cluster::edges(store, graph.edges);
        let n = edges.len();

        if n == 0 {
            return Self::by_vertices_edges(store, vec![], vec![]);
        }

        // Create a vertex for each edge (at midpoint)
        let new_vertices: Vec<VertexHandle> = edges
            .iter()
            .map(|&e| {
                let start = Vertex::point(store, Edge::start_vertex(store, e));
                let end = Vertex::point(store, Edge::end_vertex(store, e));
                Vertex::by_point(store, Point3::new(
                    (start.x + end.x) / 2.0,
                    (start.y + end.y) / 2.0,
                    (start.z + end.z) / 2.0,
                ))
            })
            .collect();

        // Connect vertices if original edges share a vertex
        let mut new_edges = Vec::new();
        for i in 0..n {
            for j in (i + 1)..n {
                let start_i = Vertex::point(store, Edge::start_vertex(store, edges[i]));
                let end_i = Vertex::point(store, Edge::end_vertex(store, edges[i]));
                let start_j = Vertex::point(store, Edge::start_vertex(store, edges[j]));
                let end_j = Vertex::point(store, Edge::end_vertex(store, edges[j]));

                // Check if edges share a vertex
                if start_i.distance(start_j) < tolerance
                    || start_i.distance(end_j) < tolerance
                    || end_i.distance(start_j) < tolerance
                    || end_i.distance(end_j) < tolerance
                {
                    new_edges.push(Edge::by_start_vertex_end_vertex(
                        store,
                        new_vertices[i],
                        new_vertices[j],
                    ));
                }
            }
        }

        Self::by_vertices_edges(store, new_vertices, new_edges)
    }

    /// Create a quotient graph by contracting vertices in the same partition
    pub fn quotient(
        store: &TopologyStore,
        graph: GraphHandle,
        partition: &[usize],
        tolerance: f64,
    ) -> GraphHandle {
        let vertices = Cluster::vertices(store, graph.vertices);
        let adj_list = Self::adjacency_list(store, graph, tolerance);
        let n = vertices.len();

        if n == 0 || partition.len() != n {
            return Self::by_vertices_edges(store, vec![], vec![]);
        }

        // Find number of partitions
        let num_partitions = partition.iter().copied().max().map(|x| x + 1).unwrap_or(0);
        if num_partitions == 0 {
            return Self::by_vertices_edges(store, vec![], vec![]);
        }

        // Create centroid for each partition
        let mut partition_centers: Vec<Point3> = vec![Point3::new(0.0, 0.0, 0.0); num_partitions];
        let mut partition_counts: Vec<usize> = vec![0; num_partitions];

        for (i, &p) in partition.iter().enumerate() {
            if p < num_partitions {
                let point = Vertex::point(store, vertices[i]);
                partition_centers[p].x += point.x;
                partition_centers[p].y += point.y;
                partition_centers[p].z += point.z;
                partition_counts[p] += 1;
            }
        }

        let new_vertices: Vec<VertexHandle> = (0..num_partitions)
            .map(|p| {
                if partition_counts[p] > 0 {
                    let count = partition_counts[p] as f64;
                    Vertex::by_point(store, Point3::new(
                        partition_centers[p].x / count,
                        partition_centers[p].y / count,
                        partition_centers[p].z / count,
                    ))
                } else {
                    Vertex::by_coordinates(store, p as f64, 0.0, 0.0)
                }
            })
            .collect();

        // Create edges between different partitions
        let mut seen_edges: HashSet<(usize, usize)> = HashSet::new();
        let mut new_edges = Vec::new();

        for i in 0..n {
            for &j in &adj_list[i] {
                let pi = partition[i];
                let pj = partition[j];
                if pi != pj {
                    let edge_key = (pi.min(pj), pi.max(pj));
                    if !seen_edges.contains(&edge_key) {
                        seen_edges.insert(edge_key);
                        new_edges.push(Edge::by_start_vertex_end_vertex(
                            store,
                            new_vertices[pi],
                            new_vertices[pj],
                        ));
                    }
                }
            }
        }

        Self::by_vertices_edges(store, new_vertices, new_edges)
    }

    /// Check if the graph is connected
    pub fn is_connected(
        store: &TopologyStore,
        graph: GraphHandle,
        tolerance: f64,
    ) -> bool {
        let vertices = Cluster::vertices(store, graph.vertices);
        let n = vertices.len();

        if n <= 1 {
            return true;
        }

        let depths = Self::depth_map(store, graph, vertices[0], tolerance);
        depths.iter().all(|&d| d >= 0)
    }

    /// Get connected components
    pub fn connected_components(
        store: &TopologyStore,
        graph: GraphHandle,
        tolerance: f64,
    ) -> Vec<Vec<VertexHandle>> {
        let vertices = Cluster::vertices(store, graph.vertices);
        let adj_list = Self::adjacency_list(store, graph, tolerance);
        let n = vertices.len();

        let mut visited = vec![false; n];
        let mut components = Vec::new();

        for start in 0..n {
            if visited[start] {
                continue;
            }

            let mut component = Vec::new();
            let mut queue = VecDeque::new();
            queue.push_back(start);
            visited[start] = true;

            while let Some(u) = queue.pop_front() {
                component.push(vertices[u]);
                for &v in &adj_list[u] {
                    if !visited[v] {
                        visited[v] = true;
                        queue.push_back(v);
                    }
                }
            }

            components.push(component);
        }

        components
    }

    /// Partition the graph into communities using label propagation algorithm
    pub fn partition(
        store: &TopologyStore,
        graph: GraphHandle,
        tolerance: f64,
        method: &str,
        num_partitions: Option<usize>,
    ) -> Vec<usize> {
        let adj_list = Self::adjacency_list(store, graph, tolerance);
        let n = adj_list.len();

        if n == 0 {
            return vec![];
        }

        match method.to_lowercase().as_str() {
            "community" | "louvain" | "label" => Self::partition_label_propagation(&adj_list, n),
            "spectral" | "fiedler" => {
                let k = num_partitions.unwrap_or(2);
                Self::partition_spectral(&adj_list, n, k)
            }
            "betweenness" | "girvan-newman" => {
                let k = num_partitions.unwrap_or(2);
                Self::partition_edge_betweenness(&adj_list, n, k)
            }
            _ => Self::partition_label_propagation(&adj_list, n),
        }
    }

    /// Label propagation community detection
    fn partition_label_propagation(adj: &[Vec<usize>], n: usize) -> Vec<usize> {
        let mut labels: Vec<usize> = (0..n).collect();
        let mut changed = true;
        let mut iterations = 0;
        let max_iterations = 100;

        while changed && iterations < max_iterations {
            changed = false;
            iterations += 1;

            for i in 0..n {
                if adj[i].is_empty() {
                    continue;
                }

                // Count neighbor labels
                let mut label_counts: HashMap<usize, usize> = HashMap::new();
                for &neighbor in &adj[i] {
                    *label_counts.entry(labels[neighbor]).or_insert(0) += 1;
                }

                // Find most common label
                let best_label = label_counts
                    .into_iter()
                    .max_by_key(|(_, count)| *count)
                    .map(|(label, _)| label)
                    .unwrap_or(labels[i]);

                if best_label != labels[i] {
                    labels[i] = best_label;
                    changed = true;
                }
            }
        }

        // Normalize labels to be 0..num_communities
        let mut label_map: HashMap<usize, usize> = HashMap::new();
        let mut next_label = 0;
        for label in &labels {
            if !label_map.contains_key(label) {
                label_map.insert(*label, next_label);
                next_label += 1;
            }
        }

        labels.iter().map(|l| label_map[l]).collect()
    }

    /// Spectral partitioning using Fiedler vector approximation
    fn partition_spectral(adj: &[Vec<usize>], n: usize, k: usize) -> Vec<usize> {
        if n <= k {
            return (0..n).collect();
        }

        // Simple spectral bisection using power iteration
        // Build Laplacian: L = D - A
        let degrees: Vec<f64> = adj.iter().map(|neighbors| neighbors.len() as f64).collect();

        // Start with random vector orthogonal to all-ones
        let mut v: Vec<f64> = (0..n)
            .map(|i| (i as f64 - n as f64 / 2.0) / n as f64)
            .collect();

        // Power iteration for second smallest eigenvector
        for _ in 0..50 {
            let mut new_v = vec![0.0; n];

            // Multiply by Laplacian: L*v = D*v - A*v
            for i in 0..n {
                new_v[i] = degrees[i] * v[i];
                for &j in &adj[i] {
                    new_v[i] -= v[j];
                }
            }

            // Make orthogonal to constant vector
            let mean: f64 = new_v.iter().sum::<f64>() / n as f64;
            for x in new_v.iter_mut() {
                *x -= mean;
            }

            // Normalize
            let norm: f64 = new_v.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm > 1e-10 {
                for x in new_v.iter_mut() {
                    *x /= norm;
                }
            }

            v = new_v;
        }

        // Partition based on sign of Fiedler vector
        let mut partitions: Vec<usize> = v.iter().map(|&x| if x < 0.0 { 0 } else { 1 }).collect();

        // If more than 2 partitions needed, recursively partition
        if k > 2 {
            // Simple k-means-like assignment
            let step = n / k;
            let mut sorted_indices: Vec<(usize, f64)> = v.iter().copied().enumerate().collect();
            sorted_indices.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            for (rank, (idx, _)) in sorted_indices.iter().enumerate() {
                partitions[*idx] = (rank / step.max(1)).min(k - 1);
            }
        }

        partitions
    }

    /// Edge betweenness-based partitioning (simplified Girvan-Newman)
    fn partition_edge_betweenness(adj: &[Vec<usize>], n: usize, k: usize) -> Vec<usize> {
        // Create mutable adjacency
        let mut adj_mut: Vec<HashSet<usize>> = adj.iter().map(|v| v.iter().copied().collect()).collect();

        // Remove edges until we have k components
        let mut num_components = 1;

        while num_components < k {
            // Calculate edge betweenness
            let mut edge_bc: HashMap<(usize, usize), f64> = HashMap::new();

            for s in 0..n {
                // BFS from s
                let mut pred: Vec<Vec<usize>> = vec![vec![]; n];
                let mut sigma = vec![0.0; n];
                sigma[s] = 1.0;
                let mut dist = vec![-1_i32; n];
                dist[s] = 0;
                let mut stack = Vec::new();
                let mut queue = VecDeque::new();
                queue.push_back(s);

                while let Some(v) = queue.pop_front() {
                    stack.push(v);
                    for &w in &adj_mut[v] {
                        if dist[w] < 0 {
                            dist[w] = dist[v] + 1;
                            queue.push_back(w);
                        }
                        if dist[w] == dist[v] + 1 {
                            sigma[w] += sigma[v];
                            pred[w].push(v);
                        }
                    }
                }

                // Accumulate on edges
                let mut delta = vec![0.0; n];
                while let Some(w) = stack.pop() {
                    for &v in &pred[w] {
                        let c = (sigma[v] / sigma[w]) * (1.0 + delta[w]);
                        let edge = (v.min(w), v.max(w));
                        *edge_bc.entry(edge).or_insert(0.0) += c;
                        delta[v] += c;
                    }
                }
            }

            // Find edge with highest betweenness and remove it
            if let Some((&(u, v), _)) = edge_bc.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()) {
                adj_mut[u].remove(&v);
                adj_mut[v].remove(&u);
            } else {
                break;
            }

            // Count components
            num_components = Self::count_components(&adj_mut, n);
        }

        // Assign partition labels based on connected components
        let adj_final: Vec<Vec<usize>> = adj_mut.iter().map(|s| s.iter().copied().collect()).collect();
        Self::assign_component_labels(&adj_final, n)
    }

    fn count_components(adj: &[HashSet<usize>], n: usize) -> usize {
        let mut visited = vec![false; n];
        let mut count = 0;

        for start in 0..n {
            if visited[start] {
                continue;
            }
            count += 1;

            let mut queue = VecDeque::new();
            queue.push_back(start);
            visited[start] = true;

            while let Some(u) = queue.pop_front() {
                for &v in &adj[u] {
                    if !visited[v] {
                        visited[v] = true;
                        queue.push_back(v);
                    }
                }
            }
        }

        count
    }

    fn assign_component_labels(adj: &[Vec<usize>], n: usize) -> Vec<usize> {
        let mut labels = vec![0; n];
        let mut visited = vec![false; n];
        let mut current_label = 0;

        for start in 0..n {
            if visited[start] {
                continue;
            }

            let mut queue = VecDeque::new();
            queue.push_back(start);
            visited[start] = true;

            while let Some(u) = queue.pop_front() {
                labels[u] = current_label;
                for &v in &adj[u] {
                    if !visited[v] {
                        visited[v] = true;
                        queue.push_back(v);
                    }
                }
            }

            current_label += 1;
        }

        labels
    }

    /// Get the eccentricity of each vertex (max distance to any other vertex)
    pub fn eccentricity(
        store: &TopologyStore,
        graph: GraphHandle,
        tolerance: f64,
    ) -> Vec<i32> {
        let vertices = Cluster::vertices(store, graph.vertices);
        vertices
            .iter()
            .map(|v| {
                let depths = Self::depth_map(store, graph, *v, tolerance);
                depths.into_iter().max().unwrap_or(-1)
            })
            .collect()
    }

    /// Get the radius of the graph (minimum eccentricity)
    pub fn radius(
        store: &TopologyStore,
        graph: GraphHandle,
        tolerance: f64,
    ) -> i32 {
        let eccs = Self::eccentricity(store, graph, tolerance);
        eccs.into_iter().filter(|&e| e >= 0).min().unwrap_or(-1)
    }

    /// Get the center vertices (vertices with eccentricity equal to radius)
    pub fn center(
        store: &TopologyStore,
        graph: GraphHandle,
        tolerance: f64,
    ) -> Vec<VertexHandle> {
        let vertices = Cluster::vertices(store, graph.vertices);
        let eccs = Self::eccentricity(store, graph, tolerance);
        let radius = eccs.iter().filter(|&&e| e >= 0).min().copied().unwrap_or(-1);

        vertices
            .into_iter()
            .enumerate()
            .filter(|(i, _)| eccs[*i] == radius)
            .map(|(_, v)| v)
            .collect()
    }

    /// Get the periphery vertices (vertices with eccentricity equal to diameter)
    pub fn periphery(
        store: &TopologyStore,
        graph: GraphHandle,
        tolerance: f64,
    ) -> Vec<VertexHandle> {
        let vertices = Cluster::vertices(store, graph.vertices);
        let eccs = Self::eccentricity(store, graph, tolerance);
        let diameter = eccs.iter().max().copied().unwrap_or(-1);

        vertices
            .into_iter()
            .enumerate()
            .filter(|(i, _)| eccs[*i] == diameter)
            .map(|(_, v)| v)
            .collect()
    }

    /// Get the closeness centrality of all vertices
    pub fn closeness_centrality(
        store: &TopologyStore,
        graph: GraphHandle,
        tolerance: f64,
    ) -> Vec<f64> {
        let vertices = Cluster::vertices(store, graph.vertices);
        let n = vertices.len();

        if n <= 1 {
            return vec![0.0; n];
        }

        vertices
            .iter()
            .map(|v| {
                let depths = Self::depth_map(store, graph, *v, tolerance);
                let total: i32 = depths.iter().filter(|&&d| d > 0).sum();
                let reachable = depths.iter().filter(|&&d| d > 0).count();

                if total > 0 && reachable > 0 {
                    (reachable as f64 - 1.0) / total as f64 * reachable as f64 / (n as f64 - 1.0)
                } else {
                    0.0
                }
            })
            .collect()
    }

    /// Get the degree centrality of all vertices
    pub fn degree_centrality(
        store: &TopologyStore,
        graph: GraphHandle,
        tolerance: f64,
    ) -> Vec<f64> {
        let adj_list = Self::adjacency_list(store, graph, tolerance);
        let n = adj_list.len();

        if n <= 1 {
            return vec![0.0; n];
        }

        adj_list
            .iter()
            .map(|neighbors| neighbors.len() as f64 / (n - 1) as f64)
            .collect()
    }

    /// Partition the graph into communities and return as groups of vertices
    /// This matches topologicpy's Graph.CommunityPartition() API
    pub fn community_partition(
        store: &TopologyStore,
        graph: GraphHandle,
        tolerance: f64,
        method: Option<&str>,
    ) -> Vec<Vec<VertexHandle>> {
        let vertices = Cluster::vertices(store, graph.vertices);
        let method_str = method.unwrap_or("louvain");
        let partition = Self::partition(store, graph, tolerance, method_str, None);

        // Group vertices by their partition label
        let num_communities = partition.iter().copied().max().map(|x| x + 1).unwrap_or(0);
        let mut communities: Vec<Vec<VertexHandle>> = vec![Vec::new(); num_communities];

        for (i, &label) in partition.iter().enumerate() {
            if label < num_communities {
                communities[label].push(vertices[i]);
            }
        }

        // Remove empty communities
        communities.into_iter().filter(|c| !c.is_empty()).collect()
    }

    /// Louvain community detection algorithm
    /// Returns partition labels for each vertex
    pub fn louvain(
        store: &TopologyStore,
        graph: GraphHandle,
        tolerance: f64,
        resolution: f64,
        max_iterations: usize,
    ) -> Vec<usize> {
        let adj_list = Self::adjacency_list(store, graph, tolerance);
        let n = adj_list.len();

        if n == 0 {
            return vec![];
        }

        // Calculate edge weights (all 1 for unweighted graph)
        let m: f64 = adj_list.iter().map(|neighbors| neighbors.len() as f64).sum::<f64>() / 2.0;
        if m < 1.0 {
            return (0..n).collect();
        }

        // Initialize: each node in its own community
        let mut communities: Vec<usize> = (0..n).collect();
        let degrees: Vec<f64> = adj_list.iter().map(|neighbors| neighbors.len() as f64).collect();

        let mut improved = true;
        let mut iterations = 0;

        while improved && iterations < max_iterations {
            improved = false;
            iterations += 1;

            for i in 0..n {
                let current_community = communities[i];

                // Calculate neighbor community weights
                let mut community_weights: HashMap<usize, f64> = HashMap::new();
                for &j in &adj_list[i] {
                    let c = communities[j];
                    *community_weights.entry(c).or_insert(0.0) += 1.0;
                }

                // Calculate current community's total degree
                let mut community_degrees: HashMap<usize, f64> = HashMap::new();
                for (j, &c) in communities.iter().enumerate() {
                    *community_degrees.entry(c).or_insert(0.0) += degrees[j];
                }

                let ki = degrees[i];
                let mut best_community = current_community;
                let mut best_delta = 0.0;

                // Try moving to each neighbor's community
                for (&c, &weight) in &community_weights {
                    if c == current_community {
                        continue;
                    }

                    let sigma_c = *community_degrees.get(&c).unwrap_or(&0.0);
                    let sigma_current = *community_degrees.get(&current_community).unwrap_or(&0.0) - ki;
                    let ki_c = weight;
                    let ki_current = *community_weights.get(&current_community).unwrap_or(&0.0);

                    // Modularity gain
                    let delta = resolution * (ki_c - ki_current) / m
                        - ki * (sigma_c - sigma_current + ki) / (2.0 * m * m);

                    if delta > best_delta {
                        best_delta = delta;
                        best_community = c;
                    }
                }

                if best_community != current_community {
                    communities[i] = best_community;
                    improved = true;
                }
            }
        }

        // Normalize community labels
        let mut label_map: HashMap<usize, usize> = HashMap::new();
        let mut next_label = 0;
        for c in &communities {
            if !label_map.contains_key(c) {
                label_map.insert(*c, next_label);
                next_label += 1;
            }
        }

        communities.iter().map(|c| label_map[c]).collect()
    }

    /// Get modularity score for a partition
    pub fn modularity(
        store: &TopologyStore,
        graph: GraphHandle,
        partition: &[usize],
        tolerance: f64,
    ) -> f64 {
        let adj_list = Self::adjacency_list(store, graph, tolerance);
        let n = adj_list.len();

        if n == 0 || partition.len() != n {
            return 0.0;
        }

        let m: f64 = adj_list.iter().map(|neighbors| neighbors.len() as f64).sum::<f64>() / 2.0;
        if m < 1.0 {
            return 0.0;
        }

        let degrees: Vec<f64> = adj_list.iter().map(|neighbors| neighbors.len() as f64).collect();

        let mut q = 0.0;
        for i in 0..n {
            for &j in &adj_list[i] {
                if partition[i] == partition[j] {
                    q += 1.0 - (degrees[i] * degrees[j]) / (2.0 * m);
                }
            }
        }

        q / (2.0 * m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_by_vertices_edges() {
        let store = TopologyStore::new();
        let v0 = Vertex::by_coordinates(&store, 0.0, 0.0, 0.0);
        let v1 = Vertex::by_coordinates(&store, 1.0, 0.0, 0.0);
        let v2 = Vertex::by_coordinates(&store, 1.0, 1.0, 0.0);
        let e0 = Edge::by_start_vertex_end_vertex(&store, v0, v1);
        let e1 = Edge::by_start_vertex_end_vertex(&store, v1, v2);

        let graph = GraphUtil::by_vertices_edges(&store, vec![v0, v1, v2], vec![e0, e1]);

        assert_eq!(GraphUtil::order(&store, graph), 3);
        assert_eq!(GraphUtil::size(&store, graph), 2);
    }

    #[test]
    fn test_adjacency_list() {
        let store = TopologyStore::new();
        let v0 = Vertex::by_coordinates(&store, 0.0, 0.0, 0.0);
        let v1 = Vertex::by_coordinates(&store, 1.0, 0.0, 0.0);
        let v2 = Vertex::by_coordinates(&store, 1.0, 1.0, 0.0);
        let e0 = Edge::by_start_vertex_end_vertex(&store, v0, v1);
        let e1 = Edge::by_start_vertex_end_vertex(&store, v1, v2);

        let graph = GraphUtil::by_vertices_edges(&store, vec![v0, v1, v2], vec![e0, e1]);
        let adj = GraphUtil::adjacency_list(&store, graph, 0.001);

        assert_eq!(adj.len(), 3);
        assert_eq!(adj[0].len(), 1); // v0 connected to v1
        assert_eq!(adj[1].len(), 2); // v1 connected to v0 and v2
        assert_eq!(adj[2].len(), 1); // v2 connected to v1
    }

    #[test]
    fn test_shortest_path() {
        let store = TopologyStore::new();
        let v0 = Vertex::by_coordinates(&store, 0.0, 0.0, 0.0);
        let v1 = Vertex::by_coordinates(&store, 1.0, 0.0, 0.0);
        let v2 = Vertex::by_coordinates(&store, 2.0, 0.0, 0.0);
        let e0 = Edge::by_start_vertex_end_vertex(&store, v0, v1);
        let e1 = Edge::by_start_vertex_end_vertex(&store, v1, v2);

        let graph = GraphUtil::by_vertices_edges(&store, vec![v0, v1, v2], vec![e0, e1]);
        let path = GraphUtil::shortest_path_vertices(&store, graph, v0, v2, 0.001);

        assert!(path.is_some());
        assert_eq!(path.unwrap().len(), 3);
    }
}
