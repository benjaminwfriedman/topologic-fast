//! Shell topology (collection of connected faces)

use super::*;
use crate::geometry::{Point3, Vector3, TOLERANCE};
use smallvec::SmallVec;

/// Handle to a shell in the topology store
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShellHandle {
    pub(crate) index: ArenaIndex,
    pub(crate) id: TopologyId,
}

impl ShellHandle {
    pub fn id(&self) -> TopologyId {
        self.id
    }
}

/// Internal shell data
pub(crate) struct ShellData {
    pub id: TopologyId,
    pub faces: Vec<FaceHandle>,
    pub is_closed: bool,
    pub cells: SmallVec<[CellHandle; 2]>,
    pub dictionary: Dictionary,
}

/// Shell operations
pub struct Shell;

impl Shell {
    /// Create a shell from a collection of faces
    pub fn by_faces(store: &TopologyStore, faces: Vec<FaceHandle>) -> ShellHandle {
        let is_closed = Self::check_closed(store, &faces);
        store.add_shell(faces, is_closed)
    }

    fn check_closed(store: &TopologyStore, faces: &[FaceHandle]) -> bool {
        // A shell is closed if every edge is shared by exactly 2 faces
        let mut edge_count: hashbrown::HashMap<EdgeHandle, usize> = hashbrown::HashMap::new();

        for face in faces {
            for edge in Face::edges(store, *face) {
                *edge_count.entry(edge).or_insert(0) += 1;
            }
        }

        edge_count.values().all(|&count| count == 2)
    }

    /// Create a box shell
    pub fn box_shell(
        store: &TopologyStore,
        origin: Point3,
        width: f64,
        length: f64,
        height: f64,
    ) -> ShellHandle {
        // Create 8 vertices
        let v = [
            Vertex::by_point(store, origin),
            Vertex::by_point(store, origin + Vector3::new(width, 0.0, 0.0)),
            Vertex::by_point(store, origin + Vector3::new(width, length, 0.0)),
            Vertex::by_point(store, origin + Vector3::new(0.0, length, 0.0)),
            Vertex::by_point(store, origin + Vector3::new(0.0, 0.0, height)),
            Vertex::by_point(store, origin + Vector3::new(width, 0.0, height)),
            Vertex::by_point(store, origin + Vector3::new(width, length, height)),
            Vertex::by_point(store, origin + Vector3::new(0.0, length, height)),
        ];

        // Create 6 faces
        let faces = vec![
            Face::by_vertices(store, vec![v[0], v[3], v[2], v[1]]), // Bottom
            Face::by_vertices(store, vec![v[4], v[5], v[6], v[7]]), // Top
            Face::by_vertices(store, vec![v[0], v[1], v[5], v[4]]), // Front
            Face::by_vertices(store, vec![v[2], v[3], v[7], v[6]]), // Back
            Face::by_vertices(store, vec![v[0], v[4], v[7], v[3]]), // Left
            Face::by_vertices(store, vec![v[1], v[2], v[6], v[5]]), // Right
        ];

        Self::by_faces(store, faces)
    }

    /// Get all faces in the shell
    pub fn faces(store: &TopologyStore, handle: ShellHandle) -> Vec<FaceHandle> {
        store.shells.read()[handle.index.index()].faces.clone()
    }

    /// Get all edges in the shell
    pub fn edges(store: &TopologyStore, handle: ShellHandle) -> Vec<EdgeHandle> {
        let faces = Self::faces(store, handle);
        let mut seen = hashbrown::HashSet::new();
        faces
            .iter()
            .flat_map(|f| Face::edges(store, *f))
            .filter(|e| seen.insert(*e))
            .collect()
    }

    /// Get all vertices in the shell
    pub fn vertices(store: &TopologyStore, handle: ShellHandle) -> Vec<VertexHandle> {
        let faces = Self::faces(store, handle);
        let mut seen = hashbrown::HashSet::new();
        faces
            .iter()
            .flat_map(|f| Face::vertices(store, *f))
            .filter(|v| seen.insert(*v))
            .collect()
    }

    /// Get all wires in the shell
    pub fn wires(store: &TopologyStore, handle: ShellHandle) -> Vec<WireHandle> {
        let faces = Self::faces(store, handle);
        let mut seen = hashbrown::HashSet::new();
        faces
            .iter()
            .flat_map(|f| Face::wires(store, *f))
            .filter(|w| seen.insert(*w))
            .collect()
    }

    /// Get cells using this shell
    pub fn cells(store: &TopologyStore, handle: ShellHandle) -> Vec<CellHandle> {
        store.shells.read()[handle.index.index()].cells.to_vec()
    }

    /// Check if the shell is closed
    pub fn is_closed(store: &TopologyStore, handle: ShellHandle) -> bool {
        store.shells.read()[handle.index.index()].is_closed
    }

    /// Get the total surface area
    pub fn area(store: &TopologyStore, handle: ShellHandle) -> f64 {
        Self::faces(store, handle)
            .iter()
            .map(|f| Face::area(store, *f))
            .sum()
    }

    /// Get the center of mass
    pub fn center_of_mass(store: &TopologyStore, handle: ShellHandle) -> Point3 {
        let faces = Self::faces(store, handle);
        if faces.is_empty() {
            return Point3::ZERO;
        }

        // Weight by face area
        let mut total_area = 0.0;
        let mut weighted_center = Point3::ZERO;

        for face in faces {
            let area = Face::area(store, face);
            let center = Face::center_of_mass(store, face);
            weighted_center += center * area;
            total_area += area;
        }

        if total_area > TOLERANCE {
            weighted_center / total_area
        } else {
            Point3::ZERO
        }
    }

    /// Get the bounding box
    pub fn bounding_box(store: &TopologyStore, handle: ShellHandle) -> (Point3, Point3) {
        let vertices = Self::vertices(store, handle);
        if vertices.is_empty() {
            return (Point3::ZERO, Point3::ZERO);
        }

        let points: Vec<_> = vertices.iter().map(|v| Vertex::point(store, *v)).collect();
        crate::geometry::bounding_box(&points).unwrap_or((Point3::ZERO, Point3::ZERO))
    }

    /// Get the Euler characteristic (V - E + F)
    pub fn euler_characteristic(store: &TopologyStore, handle: ShellHandle) -> i32 {
        let v = Self::vertices(store, handle).len() as i32;
        let e = Self::edges(store, handle).len() as i32;
        let f = Self::faces(store, handle).len() as i32;
        v - e + f
    }

    /// Check if the shell is manifold
    pub fn is_manifold(store: &TopologyStore, handle: ShellHandle) -> bool {
        let edges = Self::edges(store, handle);

        // Every edge should be shared by exactly 2 faces in the shell
        let shell_faces: hashbrown::HashSet<_> = Self::faces(store, handle).into_iter().collect();

        for edge in edges {
            let edge_faces: Vec<_> = Edge::faces(store, edge)
                .into_iter()
                .filter(|f| shell_faces.contains(f))
                .collect();

            if edge_faces.len() != 2 {
                return false;
            }
        }

        true
    }

    /// Get non-manifold edges
    pub fn non_manifold_edges(store: &TopologyStore, handle: ShellHandle) -> Vec<EdgeHandle> {
        let edges = Self::edges(store, handle);
        let shell_faces: hashbrown::HashSet<_> = Self::faces(store, handle).into_iter().collect();

        edges
            .into_iter()
            .filter(|edge| {
                let count = Edge::faces(store, *edge)
                    .into_iter()
                    .filter(|f| shell_faces.contains(f))
                    .count();
                count != 2
            })
            .collect()
    }

    /// Get boundary edges (used by only one face)
    pub fn boundary_edges(store: &TopologyStore, handle: ShellHandle) -> Vec<EdgeHandle> {
        let edges = Self::edges(store, handle);
        let shell_faces: hashbrown::HashSet<_> = Self::faces(store, handle).into_iter().collect();

        edges
            .into_iter()
            .filter(|edge| {
                let count = Edge::faces(store, *edge)
                    .into_iter()
                    .filter(|f| shell_faces.contains(f))
                    .count();
                count == 1
            })
            .collect()
    }

    /// Calculate the volume (for closed shells only)
    pub fn volume(store: &TopologyStore, handle: ShellHandle) -> f64 {
        if !Self::is_closed(store, handle) {
            return 0.0;
        }

        // Divergence theorem: V = (1/3) * sum(face_centroid · face_normal * face_area).
        // Face normals come from wire winding, which is not guaranteed to be
        // consistently outward — e.g. a face shared with another cell (merged in
        // CellComplex::by_cells) keeps the winding of its canonical owner. We
        // therefore orient each face's normal outward relative to an interior
        // reference point (the mean of the face centroids), which is correct for
        // the convex / star-shaped cells produced by the constructors and a
        // no-op when the winding is already consistent.
        let faces = Self::faces(store, handle);
        if faces.is_empty() {
            return 0.0;
        }

        let centers: Vec<Point3> = faces.iter().map(|f| Face::center_of_mass(store, *f)).collect();
        let mut reference = Point3::ZERO;
        for c in &centers {
            reference += *c;
        }
        reference = reference / (centers.len() as f64);

        let mut volume = 0.0;
        for (face, center) in faces.iter().zip(centers.iter()) {
            if let Some(normal) = Face::normal(store, *face) {
                let area = Face::area(store, *face);
                // Flip the contribution if this face's normal points inward.
                let outward = center.dot(normal) - reference.dot(normal);
                let sign = if outward < 0.0 { -1.0 } else { 1.0 };
                volume += sign * center.dot(normal) * area;
            }
        }

        (volume / 3.0).abs()
    }

    /// Get the dictionary attached to this shell
    pub fn get_dictionary(store: &TopologyStore, handle: ShellHandle) -> Dictionary {
        store.shells.read()[handle.index.index()].dictionary.clone()
    }

    /// Set the dictionary for this shell
    pub fn set_dictionary(store: &TopologyStore, handle: ShellHandle, dict: Dictionary) {
        store.shells.write()[handle.index.index()].dictionary = dict;
    }

    /// Set a single key-value pair in the shell's dictionary
    pub fn set_dictionary_value(store: &TopologyStore, handle: ShellHandle, key: &str, value: DictionaryValue) {
        store.shells.write()[handle.index.index()].dictionary.set(key, value);
    }

    /// Get a value from the shell's dictionary
    pub fn get_dictionary_value(store: &TopologyStore, handle: ShellHandle, key: &str) -> Option<DictionaryValue> {
        store.shells.read()[handle.index.index()].dictionary.get(key).cloned()
    }
}

impl<'a> ShellRef<'a> {
    pub fn faces(&self) -> Vec<FaceHandle> {
        Shell::faces(self.store, self.handle)
    }

    pub fn edges(&self) -> Vec<EdgeHandle> {
        Shell::edges(self.store, self.handle)
    }

    pub fn vertices(&self) -> Vec<VertexHandle> {
        Shell::vertices(self.store, self.handle)
    }

    pub fn is_closed(&self) -> bool {
        Shell::is_closed(self.store, self.handle)
    }

    pub fn area(&self) -> f64 {
        Shell::area(self.store, self.handle)
    }

    pub fn volume(&self) -> f64 {
        Shell::volume(self.store, self.handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_shell() {
        let store = TopologyStore::new();
        let shell = Shell::box_shell(&store, Point3::ZERO, 2.0, 2.0, 2.0);

        assert!(Shell::is_closed(&store, shell));
        assert_eq!(Shell::faces(&store, shell).len(), 6);
        assert_eq!(Shell::vertices(&store, shell).len(), 8);
        assert_eq!(Shell::edges(&store, shell).len(), 12);

        // Euler characteristic for a box should be 2
        assert_eq!(Shell::euler_characteristic(&store, shell), 2);
    }

    #[test]
    fn test_box_area() {
        let store = TopologyStore::new();
        let shell = Shell::box_shell(&store, Point3::ZERO, 2.0, 2.0, 2.0);

        // Surface area of a 2x2x2 box = 6 * 4 = 24
        assert!((Shell::area(&store, shell) - 24.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_box_volume() {
        let store = TopologyStore::new();
        let shell = Shell::box_shell(&store, Point3::ZERO, 2.0, 2.0, 2.0);

        // Volume of a 2x2x2 box = 8
        assert!((Shell::volume(&store, shell) - 8.0).abs() < 0.1);
    }
}
