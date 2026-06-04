//! Vertex topology (0-dimensional)

use super::*;
use crate::geometry::{Point3, Vector3, TOLERANCE};
use smallvec::SmallVec;

/// Handle to a vertex in the topology store
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexHandle {
    pub(crate) index: ArenaIndex,
    pub(crate) id: TopologyId,
}

impl VertexHandle {
    pub fn id(&self) -> TopologyId {
        self.id
    }
}

/// Internal vertex data
pub(crate) struct VertexData {
    pub id: TopologyId,
    pub point: Point3,
    pub edges: SmallVec<[EdgeHandle; 4]>,
    pub dictionary: Dictionary,
}

/// Vertex operations
pub struct Vertex;

impl Vertex {
    /// Create a vertex at the origin
    pub fn origin(store: &TopologyStore) -> VertexHandle {
        store.add_vertex(Point3::ZERO)
    }

    /// Create a vertex from coordinates
    pub fn by_coordinates(store: &TopologyStore, x: f64, y: f64, z: f64) -> VertexHandle {
        store.add_vertex(Point3::new(x, y, z))
    }

    /// Create a vertex from a point
    pub fn by_point(store: &TopologyStore, point: Point3) -> VertexHandle {
        store.add_vertex(point)
    }

    /// Get the X coordinate
    pub fn x(store: &TopologyStore, handle: VertexHandle) -> f64 {
        store.vertices.read()[handle.index.index()].point.x
    }

    /// Get the Y coordinate
    pub fn y(store: &TopologyStore, handle: VertexHandle) -> f64 {
        store.vertices.read()[handle.index.index()].point.y
    }

    /// Get the Z coordinate
    pub fn z(store: &TopologyStore, handle: VertexHandle) -> f64 {
        store.vertices.read()[handle.index.index()].point.z
    }

    /// Get all coordinates as a tuple
    pub fn coordinates(store: &TopologyStore, handle: VertexHandle) -> (f64, f64, f64) {
        let p = store.vertices.read()[handle.index.index()].point;
        (p.x, p.y, p.z)
    }

    /// Get the point
    pub fn point(store: &TopologyStore, handle: VertexHandle) -> Point3 {
        store.vertices.read()[handle.index.index()].point
    }

    /// Get all edges connected to this vertex
    pub fn edges(store: &TopologyStore, handle: VertexHandle) -> Vec<EdgeHandle> {
        store.vertices.read()[handle.index.index()].edges.to_vec()
    }

    /// Get all adjacent vertices (connected by an edge)
    pub fn adjacent_vertices(store: &TopologyStore, handle: VertexHandle) -> Vec<VertexHandle> {
        let edges = Self::edges(store, handle);
        let edge_data = store.edges.read();

        edges
            .iter()
            .filter_map(|e| {
                let edge = &edge_data[e.index.index()];
                if edge.start == handle {
                    Some(edge.end)
                } else if edge.end == handle {
                    Some(edge.start)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Calculate distance to another vertex
    pub fn distance(store: &TopologyStore, v1: VertexHandle, v2: VertexHandle) -> f64 {
        let vertices = store.vertices.read();
        let p1 = vertices[v1.index.index()].point;
        let p2 = vertices[v2.index.index()].point;
        p1.distance(p2)
    }

    /// Check if two vertices are at the same location (within tolerance)
    pub fn is_coincident(store: &TopologyStore, v1: VertexHandle, v2: VertexHandle) -> bool {
        Self::distance(store, v1, v2) < TOLERANCE
    }

    /// Project vertex onto a face
    pub fn project_to_face(
        store: &TopologyStore,
        vertex: VertexHandle,
        face: FaceHandle,
    ) -> Option<Point3> {
        let point = Self::point(store, vertex);
        let normal = Face::normal(store, face)?;
        let center = Face::center_of_mass(store, face);

        // Project onto the face's plane
        let d = (point - center).dot(normal);
        Some(point - normal * d)
    }

    /// Find the nearest vertex in a collection
    pub fn nearest(
        store: &TopologyStore,
        point: Point3,
        vertices: &[VertexHandle],
    ) -> Option<VertexHandle> {
        if vertices.is_empty() {
            return None;
        }

        let vertex_data = store.vertices.read();
        vertices
            .iter()
            .min_by(|a, b| {
                let da = vertex_data[a.index.index()].point.distance_squared(point);
                let db = vertex_data[b.index.index()].point.distance_squared(point);
                da.partial_cmp(&db).unwrap()
            })
            .copied()
    }

    /// Check if a vertex is internal to a cell (inside the cell's volume)
    pub fn is_internal(
        store: &TopologyStore,
        vertex: VertexHandle,
        cell: CellHandle,
    ) -> bool {
        let point = Self::point(store, vertex);

        // Ray casting algorithm: count intersections with cell faces
        let faces = Cell::faces(store, cell);
        let ray_dir = Vector3::new(1.0, 0.0, 0.0);

        let mut intersections = 0;

        for face_handle in faces {
            if let Some(normal) = Face::normal(store, face_handle) {
                let center = Face::center_of_mass(store, face_handle);

                // Check ray-plane intersection
                let denom = ray_dir.dot(normal);
                if denom.abs() > TOLERANCE {
                    let t = (center - point).dot(normal) / denom;
                    if t > 0.0 {
                        // Check if intersection is within face bounds (simplified)
                        let intersection = point + ray_dir * t;
                        if Face::contains_point(store, face_handle, intersection) {
                            intersections += 1;
                        }
                    }
                }
            }
        }

        // Odd number of intersections means inside
        intersections % 2 == 1
    }

    /// Get the center of mass (same as point for a vertex)
    pub fn center_of_mass(store: &TopologyStore, handle: VertexHandle) -> Point3 {
        Self::point(store, handle)
    }

    /// Get the dictionary attached to this vertex
    pub fn get_dictionary(store: &TopologyStore, handle: VertexHandle) -> Dictionary {
        store.vertices.read()[handle.index.index()].dictionary.clone()
    }

    /// Set the dictionary for this vertex
    pub fn set_dictionary(store: &TopologyStore, handle: VertexHandle, dict: Dictionary) {
        store.vertices.write()[handle.index.index()].dictionary = dict;
    }

    /// Set a single key-value pair in the vertex's dictionary
    pub fn set_dictionary_value(store: &TopologyStore, handle: VertexHandle, key: &str, value: DictionaryValue) {
        store.vertices.write()[handle.index.index()].dictionary.set(key, value);
    }

    /// Get a value from the vertex's dictionary
    pub fn get_dictionary_value(store: &TopologyStore, handle: VertexHandle, key: &str) -> Option<DictionaryValue> {
        store.vertices.read()[handle.index.index()].dictionary.get(key).cloned()
    }
}

impl<'a> VertexRef<'a> {
    pub fn point(&self) -> Point3 {
        Vertex::point(self.store, self.handle)
    }

    pub fn x(&self) -> f64 {
        Vertex::x(self.store, self.handle)
    }

    pub fn y(&self) -> f64 {
        Vertex::y(self.store, self.handle)
    }

    pub fn z(&self) -> f64 {
        Vertex::z(self.store, self.handle)
    }

    pub fn coordinates(&self) -> (f64, f64, f64) {
        Vertex::coordinates(self.store, self.handle)
    }

    pub fn edges(&self) -> Vec<EdgeHandle> {
        Vertex::edges(self.store, self.handle)
    }

    pub fn adjacent_vertices(&self) -> Vec<VertexHandle> {
        Vertex::adjacent_vertices(self.store, self.handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_creation() {
        let store = TopologyStore::new();
        let v = Vertex::by_coordinates(&store, 1.0, 2.0, 3.0);

        assert!((Vertex::x(&store, v) - 1.0).abs() < TOLERANCE);
        assert!((Vertex::y(&store, v) - 2.0).abs() < TOLERANCE);
        assert!((Vertex::z(&store, v) - 3.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_vertex_distance() {
        let store = TopologyStore::new();
        let v1 = Vertex::by_coordinates(&store, 0.0, 0.0, 0.0);
        let v2 = Vertex::by_coordinates(&store, 3.0, 4.0, 0.0);

        assert!((Vertex::distance(&store, v1, v2) - 5.0).abs() < TOLERANCE);
    }
}
