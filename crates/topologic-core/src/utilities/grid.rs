//! Grid utility class matching topologicpy Grid API
//!
//! Provides grid generation methods for faces.

use crate::geometry::{Point3, Vector3, TOLERANCE};
use crate::topology::{TopologyStore, FaceHandle, VertexHandle, EdgeHandle, Face, Vertex, Edge, Cluster, ClusterHandle};

/// Grid utility class with static methods
pub struct GridUtil;

impl GridUtil {
    /// Generate grid edges on a face by parameter ranges
    pub fn edges_by_parameters(
        store: &TopologyStore,
        face: FaceHandle,
        u_range: &[f64],
        v_range: &[f64],
        clip: bool,
    ) -> ClusterHandle {
        let mut edges = Vec::new();

        // Get face vertices and compute basis
        let vertices = Face::vertices(store, face);
        if vertices.len() < 3 {
            return Cluster::by_edges(store, edges);
        }

        // Get face bounds
        let points: Vec<Point3> = vertices.iter().map(|v| Vertex::point(store, *v)).collect();
        let center = Face::center_of_mass(store, face);

        // Compute local coordinate system for the face
        let (origin, u_dir, v_dir) = Self::compute_face_basis(store, face);

        // Compute face extent in UV space
        let (min_u, max_u, min_v, max_v) = Self::compute_uv_extent(&points, origin, u_dir, v_dir);

        // Generate U-lines (varying V, constant U)
        for &u in u_range {
            let u_pos = min_u + u * (max_u - min_u);
            let p1 = origin + u_dir * u_pos + v_dir * min_v;
            let p2 = origin + u_dir * u_pos + v_dir * max_v;

            let v1 = Vertex::by_point(store, p1);
            let v2 = Vertex::by_point(store, p2);
            edges.push(Edge::by_start_vertex_end_vertex(store, v1, v2));
        }

        // Generate V-lines (varying U, constant V)
        for &v in v_range {
            let v_pos = min_v + v * (max_v - min_v);
            let p1 = origin + u_dir * min_u + v_dir * v_pos;
            let p2 = origin + u_dir * max_u + v_dir * v_pos;

            let v1 = Vertex::by_point(store, p1);
            let v2 = Vertex::by_point(store, p2);
            edges.push(Edge::by_start_vertex_end_vertex(store, v1, v2));
        }

        Cluster::by_edges(store, edges)
    }

    /// Generate grid edges on a face by distance ranges
    pub fn edges_by_distances(
        store: &TopologyStore,
        face: FaceHandle,
        u_origin: Option<VertexHandle>,
        v_origin: Option<VertexHandle>,
        u_range: &[f64],
        v_range: &[f64],
        clip: bool,
        tolerance: f64,
    ) -> ClusterHandle {
        let mut edges = Vec::new();

        // Get face bounds
        let (origin, u_dir, v_dir) = Self::compute_face_basis(store, face);

        // Use provided origins or default to face center
        let u_start = u_origin
            .map(|v| Vertex::point(store, v))
            .unwrap_or(origin);
        let v_start = v_origin
            .map(|v| Vertex::point(store, v))
            .unwrap_or(origin);

        // Generate U-lines at specified distances
        for &u_dist in u_range {
            let p1 = u_start + u_dir * u_dist + v_dir * v_range.first().copied().unwrap_or(-10.0);
            let p2 = u_start + u_dir * u_dist + v_dir * v_range.last().copied().unwrap_or(10.0);

            let v1 = Vertex::by_point(store, p1);
            let v2 = Vertex::by_point(store, p2);
            edges.push(Edge::by_start_vertex_end_vertex(store, v1, v2));
        }

        // Generate V-lines at specified distances
        for &v_dist in v_range {
            let p1 = v_start + u_dir * u_range.first().copied().unwrap_or(-10.0) + v_dir * v_dist;
            let p2 = v_start + u_dir * u_range.last().copied().unwrap_or(10.0) + v_dir * v_dist;

            let v1 = Vertex::by_point(store, p1);
            let v2 = Vertex::by_point(store, p2);
            edges.push(Edge::by_start_vertex_end_vertex(store, v1, v2));
        }

        Cluster::by_edges(store, edges)
    }

    /// Generate grid vertices on a face by distance ranges
    pub fn vertices_by_distances(
        store: &TopologyStore,
        face: FaceHandle,
        origin: Option<VertexHandle>,
        u_range: &[f64],
        v_range: &[f64],
        clip: bool,
        tolerance: f64,
    ) -> ClusterHandle {
        let mut vertices = Vec::new();

        // Get face basis
        let (face_origin, u_dir, v_dir) = Self::compute_face_basis(store, face);

        // Use provided origin or default
        let start = origin
            .map(|v| Vertex::point(store, v))
            .unwrap_or(face_origin);

        // Generate grid vertices
        for &u_dist in u_range {
            for &v_dist in v_range {
                let p = start + u_dir * u_dist + v_dir * v_dist;
                vertices.push(Vertex::by_point(store, p));
            }
        }

        Cluster::by_vertices(store, vertices)
    }

    /// Generate grid vertices on a face by parameter ranges
    pub fn vertices_by_parameters(
        store: &TopologyStore,
        face: FaceHandle,
        u_range: &[f64],
        v_range: &[f64],
        clip: bool,
    ) -> ClusterHandle {
        let mut vertices = Vec::new();

        // Get face vertices for computing bounds
        let face_vertices = Face::vertices(store, face);
        if face_vertices.len() < 3 {
            return Cluster::by_vertices(store, vertices);
        }

        let points: Vec<Point3> = face_vertices.iter().map(|v| Vertex::point(store, *v)).collect();
        let (origin, u_dir, v_dir) = Self::compute_face_basis(store, face);
        let (min_u, max_u, min_v, max_v) = Self::compute_uv_extent(&points, origin, u_dir, v_dir);

        // Generate grid vertices at parameter locations
        for &u in u_range {
            for &v in v_range {
                let u_pos = min_u + u * (max_u - min_u);
                let v_pos = min_v + v * (max_v - min_v);
                let p = origin + u_dir * u_pos + v_dir * v_pos;
                vertices.push(Vertex::by_point(store, p));
            }
        }

        Cluster::by_vertices(store, vertices)
    }

    /// Compute a local coordinate system (origin, u_dir, v_dir) for a face
    fn compute_face_basis(store: &TopologyStore, face: FaceHandle) -> (Point3, Vector3, Vector3) {
        let vertices = Face::vertices(store, face);
        if vertices.len() < 3 {
            return (Point3::ZERO, Vector3::X, Vector3::Y);
        }

        let points: Vec<Point3> = vertices.iter().map(|v| Vertex::point(store, *v)).collect();
        let origin = points[0];

        // U direction: along first edge
        let u_dir = (points[1] - points[0]).normalize_or_zero();

        // Normal: perpendicular to face
        let edge1 = points[1] - points[0];
        let edge2 = points[2] - points[0];
        let normal = edge1.cross(edge2).normalize_or_zero();

        // V direction: perpendicular to both U and normal
        let v_dir = normal.cross(u_dir).normalize_or_zero();

        (origin, u_dir, v_dir)
    }

    /// Compute the UV extent of points projected onto a face's local coordinate system
    fn compute_uv_extent(
        points: &[Point3],
        origin: Point3,
        u_dir: Vector3,
        v_dir: Vector3,
    ) -> (f64, f64, f64, f64) {
        if points.is_empty() {
            return (0.0, 1.0, 0.0, 1.0);
        }

        let mut min_u = f64::MAX;
        let mut max_u = f64::MIN;
        let mut min_v = f64::MAX;
        let mut max_v = f64::MIN;

        for &p in points {
            let relative = p - origin;
            let u = relative.dot(u_dir);
            let v = relative.dot(v_dir);

            min_u = min_u.min(u);
            max_u = max_u.max(u);
            min_v = min_v.min(v);
            max_v = max_v.max(v);
        }

        (min_u, max_u, min_v, max_v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::Wire;

    #[test]
    fn test_vertices_by_parameters() {
        let store = TopologyStore::new();
        let face = Face::rectangle(&store, Point3::ZERO, 10.0, 10.0);

        let u_range = vec![0.0, 0.5, 1.0];
        let v_range = vec![0.0, 0.5, 1.0];

        let cluster = GridUtil::vertices_by_parameters(&store, face, &u_range, &v_range, true);
        let vertices = Cluster::vertices(&store, cluster);

        // Should have 3x3 = 9 vertices
        assert_eq!(vertices.len(), 9);
    }

    #[test]
    fn test_edges_by_parameters() {
        let store = TopologyStore::new();
        let face = Face::rectangle(&store, Point3::ZERO, 10.0, 10.0);

        let u_range = vec![0.0, 0.5, 1.0];
        let v_range = vec![0.0, 0.5, 1.0];

        let cluster = GridUtil::edges_by_parameters(&store, face, &u_range, &v_range, true);
        let edges = Cluster::edges(&store, cluster);

        // Should have 3 U-lines + 3 V-lines = 6 edges
        assert_eq!(edges.len(), 6);
    }
}
