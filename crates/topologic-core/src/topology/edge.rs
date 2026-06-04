//! Edge topology (1-dimensional)

use super::*;
use crate::geometry::{Point3, Vector3, TOLERANCE};
use smallvec::SmallVec;

/// Handle to an edge in the topology store
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeHandle {
    pub(crate) index: ArenaIndex,
    pub(crate) id: TopologyId,
}

impl EdgeHandle {
    pub fn id(&self) -> TopologyId {
        self.id
    }
}

/// Internal edge data
pub(crate) struct EdgeData {
    pub id: TopologyId,
    pub start: VertexHandle,
    pub end: VertexHandle,
    pub wires: SmallVec<[WireHandle; 2]>,
    pub faces: SmallVec<[FaceHandle; 2]>,
    pub dictionary: Dictionary,
}

/// Edge operations
pub struct Edge;

impl Edge {
    /// Create an edge from start and end vertices
    pub fn by_start_vertex_end_vertex(
        store: &TopologyStore,
        start: VertexHandle,
        end: VertexHandle,
    ) -> EdgeHandle {
        store.add_edge(start, end)
    }

    /// Create an edge from start and end coordinates
    pub fn by_coordinates(
        store: &TopologyStore,
        x1: f64, y1: f64, z1: f64,
        x2: f64, y2: f64, z2: f64,
    ) -> EdgeHandle {
        let start = Vertex::by_coordinates(store, x1, y1, z1);
        let end = Vertex::by_coordinates(store, x2, y2, z2);
        store.add_edge(start, end)
    }

    /// Get the start vertex
    pub fn start_vertex(store: &TopologyStore, handle: EdgeHandle) -> VertexHandle {
        store.edges.read()[handle.index.index()].start
    }

    /// Get the end vertex
    pub fn end_vertex(store: &TopologyStore, handle: EdgeHandle) -> VertexHandle {
        store.edges.read()[handle.index.index()].end
    }

    /// Get both vertices
    pub fn vertices(store: &TopologyStore, handle: EdgeHandle) -> (VertexHandle, VertexHandle) {
        let edges = store.edges.read();
        let edge = &edges[handle.index.index()];
        (edge.start, edge.end)
    }

    /// Get the start point
    pub fn start_point(store: &TopologyStore, handle: EdgeHandle) -> Point3 {
        let start = Self::start_vertex(store, handle);
        Vertex::point(store, start)
    }

    /// Get the end point
    pub fn end_point(store: &TopologyStore, handle: EdgeHandle) -> Point3 {
        let end = Self::end_vertex(store, handle);
        Vertex::point(store, end)
    }

    /// Get the edge direction (normalized)
    pub fn direction(store: &TopologyStore, handle: EdgeHandle) -> Vector3 {
        let start = Self::start_point(store, handle);
        let end = Self::end_point(store, handle);
        (end - start).normalize_or_zero()
    }

    /// Get the edge length
    pub fn length(store: &TopologyStore, handle: EdgeHandle) -> f64 {
        let start = Self::start_point(store, handle);
        let end = Self::end_point(store, handle);
        start.distance(end)
    }

    /// Get the midpoint of the edge
    pub fn midpoint(store: &TopologyStore, handle: EdgeHandle) -> Point3 {
        let start = Self::start_point(store, handle);
        let end = Self::end_point(store, handle);
        (start + end) * 0.5
    }

    /// Get the center of mass (same as midpoint for straight edges)
    pub fn center_of_mass(store: &TopologyStore, handle: EdgeHandle) -> Point3 {
        Self::midpoint(store, handle)
    }

    /// Get all wires containing this edge
    pub fn wires(store: &TopologyStore, handle: EdgeHandle) -> Vec<WireHandle> {
        store.edges.read()[handle.index.index()].wires.to_vec()
    }

    /// Get all faces containing this edge
    pub fn faces(store: &TopologyStore, handle: EdgeHandle) -> Vec<FaceHandle> {
        store.edges.read()[handle.index.index()].faces.to_vec()
    }

    /// Get adjacent edges (sharing a vertex)
    pub fn adjacent_edges(store: &TopologyStore, handle: EdgeHandle) -> Vec<EdgeHandle> {
        let (start, end) = Self::vertices(store, handle);
        let mut adjacent = Vec::new();

        for e in Vertex::edges(store, start) {
            if e != handle && !adjacent.contains(&e) {
                adjacent.push(e);
            }
        }

        for e in Vertex::edges(store, end) {
            if e != handle && !adjacent.contains(&e) {
                adjacent.push(e);
            }
        }

        adjacent
    }

    /// Get shared vertices between two edges
    pub fn shared_vertices(
        store: &TopologyStore,
        e1: EdgeHandle,
        e2: EdgeHandle,
    ) -> Vec<VertexHandle> {
        let (s1, e1_end) = Self::vertices(store, e1);
        let (s2, e2_end) = Self::vertices(store, e2);

        let mut shared = Vec::new();
        if s1 == s2 || s1 == e2_end {
            shared.push(s1);
        }
        if e1_end == s2 || e1_end == e2_end {
            if !shared.contains(&e1_end) {
                shared.push(e1_end);
            }
        }
        shared
    }

    /// Check if the edge is manifold (used by exactly 2 faces)
    pub fn is_manifold(store: &TopologyStore, handle: EdgeHandle) -> bool {
        Self::faces(store, handle).len() == 2
    }

    /// Evaluate a point along the edge at parameter t ∈ [0, 1]
    pub fn evaluate(store: &TopologyStore, handle: EdgeHandle, t: f64) -> Point3 {
        let start = Self::start_point(store, handle);
        let end = Self::end_point(store, handle);
        start.lerp(end, t)
    }

    /// Get the parameter of the closest point on the edge to a given point
    pub fn closest_parameter(store: &TopologyStore, handle: EdgeHandle, point: Point3) -> f64 {
        let start = Self::start_point(store, handle);
        let end = Self::end_point(store, handle);

        let v = end - start;
        let w = point - start;

        let c1 = w.dot(v);
        if c1 <= 0.0 {
            return 0.0;
        }

        let c2 = v.dot(v);
        if c2 <= c1 {
            return 1.0;
        }

        c1 / c2
    }

    /// Get the closest point on the edge to a given point
    pub fn closest_point(store: &TopologyStore, handle: EdgeHandle, point: Point3) -> Point3 {
        let t = Self::closest_parameter(store, handle, point);
        Self::evaluate(store, handle, t)
    }

    /// Get the distance from a point to the edge
    pub fn distance_to_point(store: &TopologyStore, handle: EdgeHandle, point: Point3) -> f64 {
        let closest = Self::closest_point(store, handle, point);
        point.distance(closest)
    }

    /// Reverse the edge direction (swap start and end)
    /// Creates a new edge without edge sharing to preserve the reversed direction
    pub fn reverse(store: &TopologyStore, handle: EdgeHandle) -> EdgeHandle {
        let (start, end) = Self::vertices(store, handle);
        store.add_edge_no_share(end, start)
    }

    /// Split the edge at parameter t
    pub fn split_at(
        store: &TopologyStore,
        handle: EdgeHandle,
        t: f64,
    ) -> (EdgeHandle, EdgeHandle) {
        let start = Self::start_vertex(store, handle);
        let end = Self::end_vertex(store, handle);
        let mid_point = Self::evaluate(store, handle, t);
        let mid = Vertex::by_point(store, mid_point);

        let e1 = Self::by_start_vertex_end_vertex(store, start, mid);
        let e2 = Self::by_start_vertex_end_vertex(store, mid, end);

        (e1, e2)
    }

    /// Create an edge by origin, direction, and length
    pub fn by_origin_direction_length(
        store: &TopologyStore,
        origin: Point3,
        direction: Vector3,
        length: f64,
    ) -> EdgeHandle {
        let dir = direction.normalize_or_zero();
        let end_point = origin + dir * length;
        let start = Vertex::by_point(store, origin);
        let end = Vertex::by_point(store, end_point);
        Self::by_start_vertex_end_vertex(store, start, end)
    }

    /// Get the angle between two edges in degrees
    pub fn angle(store: &TopologyStore, e1: EdgeHandle, e2: EdgeHandle) -> f64 {
        let d1 = Self::direction(store, e1);
        let d2 = Self::direction(store, e2);

        let dot = d1.dot(d2).clamp(-1.0, 1.0);
        dot.acos().to_degrees()
    }

    /// Bisect the edge at a given parameter
    pub fn bisect(store: &TopologyStore, handle: EdgeHandle, t: f64) -> VertexHandle {
        let point = Self::evaluate(store, handle, t);
        Vertex::by_point(store, point)
    }

    /// Check if two edges are collinear
    pub fn is_collinear(store: &TopologyStore, e1: EdgeHandle, e2: EdgeHandle, tolerance: f64) -> bool {
        let d1 = Self::direction(store, e1);
        let d2 = Self::direction(store, e2);
        d1.cross(d2).length_squared() < tolerance * tolerance
    }

    /// Check if two edges are parallel
    pub fn is_parallel(store: &TopologyStore, e1: EdgeHandle, e2: EdgeHandle, tolerance: f64) -> bool {
        Self::is_collinear(store, e1, e2, tolerance)
    }

    /// Check if two edges are coplanar
    pub fn is_coplanar(store: &TopologyStore, e1: EdgeHandle, e2: EdgeHandle, tolerance: f64) -> bool {
        let p1 = Self::start_point(store, e1);
        let p2 = Self::end_point(store, e1);
        let p3 = Self::start_point(store, e2);
        let p4 = Self::end_point(store, e2);

        // Four points are coplanar if the scalar triple product is zero
        let v1 = p2 - p1;
        let v2 = p3 - p1;
        let v3 = p4 - p1;

        v1.dot(v2.cross(v3)).abs() < tolerance
    }

    /// Extend the edge by a distance at the start (-) or end (+)
    pub fn extend(
        store: &TopologyStore,
        handle: EdgeHandle,
        distance: f64,
        both_sides: bool,
    ) -> EdgeHandle {
        let start = Self::start_point(store, handle);
        let end = Self::end_point(store, handle);
        let dir = Self::direction(store, handle);

        let new_start = if both_sides || distance < 0.0 {
            start - dir * distance.abs()
        } else {
            start
        };

        let new_end = if both_sides || distance > 0.0 {
            end + dir * distance.abs()
        } else {
            end
        };

        let v_start = Vertex::by_point(store, new_start);
        let v_end = Vertex::by_point(store, new_end);
        Self::by_start_vertex_end_vertex(store, v_start, v_end)
    }

    /// Normalize the edge to unit length
    pub fn normalize(store: &TopologyStore, handle: EdgeHandle) -> EdgeHandle {
        let start = Self::start_point(store, handle);
        let dir = Self::direction(store, handle);
        let end_point = start + dir;

        let v_start = Vertex::by_point(store, start);
        let v_end = Vertex::by_point(store, end_point);
        Self::by_start_vertex_end_vertex(store, v_start, v_end)
    }

    /// Set the length of the edge (keeping start vertex fixed)
    pub fn set_length(store: &TopologyStore, handle: EdgeHandle, new_length: f64) -> EdgeHandle {
        let start = Self::start_point(store, handle);
        let dir = Self::direction(store, handle);
        let end_point = start + dir * new_length;

        let v_start = Vertex::by_point(store, start);
        let v_end = Vertex::by_point(store, end_point);
        Self::by_start_vertex_end_vertex(store, v_start, v_end)
    }

    /// Get parameter value at a vertex on the edge
    pub fn parameter_at_vertex(
        store: &TopologyStore,
        handle: EdgeHandle,
        vertex: VertexHandle,
    ) -> f64 {
        let point = Vertex::point(store, vertex);
        Self::closest_parameter(store, handle, point)
    }

    /// Get vertex by distance along the edge
    pub fn vertex_by_distance(
        store: &TopologyStore,
        handle: EdgeHandle,
        distance: f64,
    ) -> VertexHandle {
        let length = Self::length(store, handle);
        let t = if length > TOLERANCE {
            (distance / length).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let point = Self::evaluate(store, handle, t);
        Vertex::by_point(store, point)
    }

    /// Get vertex by parameter t in [0, 1]
    pub fn vertex_by_parameter(
        store: &TopologyStore,
        handle: EdgeHandle,
        t: f64,
    ) -> VertexHandle {
        let point = Self::evaluate(store, handle, t.clamp(0.0, 1.0));
        Vertex::by_point(store, point)
    }

    /// Trim the edge to a new parameter range [t1, t2]
    pub fn trim(
        store: &TopologyStore,
        handle: EdgeHandle,
        t1: f64,
        t2: f64,
    ) -> EdgeHandle {
        let p1 = Self::evaluate(store, handle, t1.clamp(0.0, 1.0));
        let p2 = Self::evaluate(store, handle, t2.clamp(0.0, 1.0));

        let v1 = Vertex::by_point(store, p1);
        let v2 = Vertex::by_point(store, p2);
        Self::by_start_vertex_end_vertex(store, v1, v2)
    }

    /// Create an offset edge in 2D (XY plane)
    pub fn by_offset_2d(
        store: &TopologyStore,
        handle: EdgeHandle,
        offset: f64,
    ) -> EdgeHandle {
        let start = Self::start_point(store, handle);
        let end = Self::end_point(store, handle);
        let dir = Self::direction(store, handle);

        // Perpendicular in XY plane: rotate 90 degrees
        let perp = Vector3::new(-dir.y, dir.x, 0.0).normalize_or_zero();

        let new_start = start + perp * offset;
        let new_end = end + perp * offset;

        let v_start = Vertex::by_point(store, new_start);
        let v_end = Vertex::by_point(store, new_end);
        Self::by_start_vertex_end_vertex(store, v_start, v_end)
    }

    /// Get the normal to the edge (perpendicular in XY plane)
    pub fn normal(store: &TopologyStore, handle: EdgeHandle) -> Vector3 {
        let dir = Self::direction(store, handle);
        Vector3::new(-dir.y, dir.x, 0.0).normalize_or_zero()
    }

    /// Get the quadrance (squared length) of the edge
    pub fn quadrance(store: &TopologyStore, handle: EdgeHandle) -> f64 {
        let start = Self::start_point(store, handle);
        let end = Self::end_point(store, handle);
        start.distance_squared(end)
    }

    /// Get the spread (squared sine of angle) between two edges
    pub fn spread(store: &TopologyStore, e1: EdgeHandle, e2: EdgeHandle) -> f64 {
        let d1 = Self::direction(store, e1);
        let d2 = Self::direction(store, e2);
        let cross = d1.cross(d2);
        cross.length_squared()
    }

    /// Find intersection point of two edges in 2D (XY plane)
    pub fn intersect_2d(
        store: &TopologyStore,
        e1: EdgeHandle,
        e2: EdgeHandle,
        tolerance: f64,
    ) -> Option<Point3> {
        let p1 = Self::start_point(store, e1);
        let p2 = Self::end_point(store, e1);
        let p3 = Self::start_point(store, e2);
        let p4 = Self::end_point(store, e2);

        let d1 = p2 - p1;
        let d2 = p4 - p3;
        let d3 = p1 - p3;

        let cross = d1.x * d2.y - d1.y * d2.x;
        if cross.abs() < tolerance {
            return None; // Parallel lines
        }

        let t1 = (d2.x * d3.y - d2.y * d3.x) / cross;
        let t2 = (d1.x * d3.y - d1.y * d3.x) / cross;

        // Check if intersection is within both edge segments
        if t1 >= -tolerance && t1 <= 1.0 + tolerance
            && t2 >= -tolerance && t2 <= 1.0 + tolerance {
            let intersection = p1 + d1 * t1;
            Some(intersection)
        } else {
            None
        }
    }

    /// Get the 2D line equation coefficients (ax + by + c = 0)
    pub fn equation_2d(store: &TopologyStore, handle: EdgeHandle) -> (f64, f64, f64) {
        let start = Self::start_point(store, handle);
        let end = Self::end_point(store, handle);

        let a = end.y - start.y;
        let b = start.x - end.x;
        let c = end.x * start.y - start.x * end.y;

        // Normalize coefficients
        let len = (a * a + b * b).sqrt();
        if len > TOLERANCE {
            (a / len, b / len, c / len)
        } else {
            (0.0, 0.0, 0.0)
        }
    }

    /// Extend edge to intersect another edge
    pub fn extend_to_edge(
        store: &TopologyStore,
        edge: EdgeHandle,
        other: EdgeHandle,
        tolerance: f64,
    ) -> Option<EdgeHandle> {
        // Get intersection of the infinite lines
        let p1 = Self::start_point(store, edge);
        let p2 = Self::end_point(store, edge);
        let p3 = Self::start_point(store, other);
        let p4 = Self::end_point(store, other);

        let d1 = p2 - p1;
        let d2 = p4 - p3;
        let d3 = p1 - p3;

        let cross = d1.x * d2.y - d1.y * d2.x;
        if cross.abs() < tolerance {
            return None; // Parallel lines
        }

        let t1 = (d2.x * d3.y - d2.y * d3.x) / cross;
        let intersection = p1 + d1 * t1;

        // Extend to intersection
        let v_start = Self::start_vertex(store, edge);
        let v_end = Vertex::by_point(store, intersection);
        Some(Self::by_start_vertex_end_vertex(store, v_start, v_end))
    }

    /// Trim edge by another edge (at intersection point)
    pub fn trim_by_edge(
        store: &TopologyStore,
        edge: EdgeHandle,
        cutter: EdgeHandle,
        tolerance: f64,
    ) -> Option<EdgeHandle> {
        if let Some(intersection) = Self::intersect_2d(store, edge, cutter, tolerance) {
            let start = Self::start_vertex(store, edge);
            let v_end = Vertex::by_point(store, intersection);
            Some(Self::by_start_vertex_end_vertex(store, start, v_end))
        } else {
            None
        }
    }

    /// Create an edge along the normal of a face
    pub fn by_face_normal(
        store: &TopologyStore,
        face: FaceHandle,
        length: f64,
    ) -> Option<EdgeHandle> {
        let center = Face::center_of_mass(store, face);
        let normal = Face::normal(store, face)?;
        Some(Self::by_origin_direction_length(store, center, normal, length))
    }

    /// Align the edge to 2D (project to XY plane)
    pub fn align_2d(store: &TopologyStore, handle: EdgeHandle) -> EdgeHandle {
        let start = Self::start_point(store, handle);
        let end = Self::end_point(store, handle);

        let v_start = Vertex::by_coordinates(store, start.x, start.y, 0.0);
        let v_end = Vertex::by_coordinates(store, end.x, end.y, 0.0);
        Self::by_start_vertex_end_vertex(store, v_start, v_end)
    }

    /// Get the dictionary attached to this edge
    pub fn get_dictionary(store: &TopologyStore, handle: EdgeHandle) -> Dictionary {
        store.edges.read()[handle.index.index()].dictionary.clone()
    }

    /// Set the dictionary for this edge
    pub fn set_dictionary(store: &TopologyStore, handle: EdgeHandle, dict: Dictionary) {
        store.edges.write()[handle.index.index()].dictionary = dict;
    }

    /// Set a single key-value pair in the edge's dictionary
    pub fn set_dictionary_value(store: &TopologyStore, handle: EdgeHandle, key: &str, value: DictionaryValue) {
        store.edges.write()[handle.index.index()].dictionary.set(key, value);
    }

    /// Get a value from the edge's dictionary
    pub fn get_dictionary_value(store: &TopologyStore, handle: EdgeHandle, key: &str) -> Option<DictionaryValue> {
        store.edges.read()[handle.index.index()].dictionary.get(key).cloned()
    }
}

impl<'a> EdgeRef<'a> {
    pub fn start_vertex(&self) -> VertexHandle {
        Edge::start_vertex(self.store, self.handle)
    }

    pub fn end_vertex(&self) -> VertexHandle {
        Edge::end_vertex(self.store, self.handle)
    }

    pub fn length(&self) -> f64 {
        Edge::length(self.store, self.handle)
    }

    pub fn direction(&self) -> Vector3 {
        Edge::direction(self.store, self.handle)
    }

    pub fn midpoint(&self) -> Point3 {
        Edge::midpoint(self.store, self.handle)
    }

    pub fn wires(&self) -> Vec<WireHandle> {
        Edge::wires(self.store, self.handle)
    }

    pub fn faces(&self) -> Vec<FaceHandle> {
        Edge::faces(self.store, self.handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_creation() {
        let store = TopologyStore::new();
        let e = Edge::by_coordinates(&store, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0);

        assert!((Edge::length(&store, e) - 1.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_edge_midpoint() {
        let store = TopologyStore::new();
        let e = Edge::by_coordinates(&store, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0);

        let mid = Edge::midpoint(&store, e);
        assert!((mid.x - 1.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_edge_evaluate() {
        let store = TopologyStore::new();
        let e = Edge::by_coordinates(&store, 0.0, 0.0, 0.0, 4.0, 0.0, 0.0);

        let p = Edge::evaluate(&store, e, 0.25);
        assert!((p.x - 1.0).abs() < TOLERANCE);
    }
}
