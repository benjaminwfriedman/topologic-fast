//! Cell topology (3-dimensional volume)

use super::*;
use crate::geometry::{Point3, Vector3, TOLERANCE};
use smallvec::SmallVec;

/// Handle to a cell in the topology store
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellHandle {
    pub(crate) index: ArenaIndex,
    pub(crate) id: TopologyId,
}

impl CellHandle {
    pub fn id(&self) -> TopologyId {
        self.id
    }
}

/// Internal cell data
pub(crate) struct CellData {
    pub id: TopologyId,
    pub outer_shell: ShellHandle,
    pub inner_shells: Vec<ShellHandle>,
    pub cell_complexes: SmallVec<[CellComplexHandle; 2]>,
    pub dictionary: Dictionary,
    pub volume_cache: Option<f64>,
}

/// Cell operations
pub struct Cell;

impl Cell {
    /// Create a cell from an outer shell
    pub fn by_shell(store: &TopologyStore, shell: ShellHandle) -> CellHandle {
        store.add_cell(shell, vec![])
    }

    /// Create a cell from faces (creates shell automatically)
    pub fn by_faces(store: &TopologyStore, faces: Vec<FaceHandle>) -> CellHandle {
        let shell = Shell::by_faces(store, faces);
        Self::by_shell(store, shell)
    }

    /// Create a cell with internal voids
    pub fn by_shells(
        store: &TopologyStore,
        outer: ShellHandle,
        inner: Vec<ShellHandle>,
    ) -> CellHandle {
        store.add_cell(outer, inner)
    }

    /// Create a box cell
    pub fn box_cell(
        store: &TopologyStore,
        origin: Point3,
        width: f64,
        length: f64,
        height: f64,
    ) -> CellHandle {
        let shell = Shell::box_shell(store, origin, width, length, height);
        Self::by_shell(store, shell)
    }

    /// Create a prism cell from a base face and height
    pub fn prism(store: &TopologyStore, face: FaceHandle, height: f64) -> CellHandle {
        let normal = Face::normal(store, face).unwrap_or(Vector3::Z);
        let direction = normal * height;

        let bottom_vertices = Face::vertices(store, face);
        let top_vertices: Vec<_> = bottom_vertices
            .iter()
            .map(|v| {
                let p = Vertex::point(store, *v) + direction;
                Vertex::by_point(store, p)
            })
            .collect();

        let mut faces = Vec::new();

        // Bottom face (reversed)
        faces.push(Face::flip(store, face));

        // Top face
        let top_wire = Wire::by_vertices(store, top_vertices.clone(), true);
        faces.push(Face::by_external_boundary(store, top_wire));

        // Side faces
        let n = bottom_vertices.len();
        for i in 0..n {
            let j = (i + 1) % n;
            let side_face = Face::by_vertices(
                store,
                vec![
                    bottom_vertices[i],
                    bottom_vertices[j],
                    top_vertices[j],
                    top_vertices[i],
                ],
            );
            faces.push(side_face);
        }

        Cell::by_faces(store, faces)
    }

    /// Create a cylinder cell
    pub fn cylinder(
        store: &TopologyStore,
        center: Point3,
        radius: f64,
        height: f64,
        segments: usize,
    ) -> CellHandle {
        let bottom = Face::circle(store, center, radius, segments);
        Cell::prism(store, bottom, height)
    }

    /// Create a sphere cell (approximated)
    pub fn sphere(
        store: &TopologyStore,
        center: Point3,
        radius: f64,
        u_segments: usize,
        v_segments: usize,
    ) -> CellHandle {
        let u_segments = u_segments.max(4);
        let v_segments = v_segments.max(2);

        let mut faces = Vec::new();

        // Generate vertices for each ring
        let mut rings: Vec<Vec<VertexHandle>> = Vec::new();

        // Bottom pole
        let bottom_pole = Vertex::by_point(store, center - Vector3::new(0.0, 0.0, radius));

        // Middle rings
        for v in 1..v_segments {
            let phi = std::f64::consts::PI * v as f64 / v_segments as f64;
            let z = center.z - radius * phi.cos();
            let r = radius * phi.sin();

            let mut ring = Vec::with_capacity(u_segments);
            for u in 0..u_segments {
                let theta = std::f64::consts::TAU * u as f64 / u_segments as f64;
                let x = center.x + r * theta.cos();
                let y = center.y + r * theta.sin();
                ring.push(Vertex::by_coordinates(store, x, y, z));
            }
            rings.push(ring);
        }

        // Top pole
        let top_pole = Vertex::by_point(store, center + Vector3::new(0.0, 0.0, radius));

        // Bottom cap faces
        for u in 0..u_segments {
            let u_next = (u + 1) % u_segments;
            faces.push(Face::by_vertices(
                store,
                vec![bottom_pole, rings[0][u], rings[0][u_next]],
            ));
        }

        // Middle band faces
        for v in 0..rings.len() - 1 {
            for u in 0..u_segments {
                let u_next = (u + 1) % u_segments;
                faces.push(Face::by_vertices(
                    store,
                    vec![rings[v][u], rings[v + 1][u], rings[v + 1][u_next], rings[v][u_next]],
                ));
            }
        }

        // Top cap faces
        let last_ring = rings.last().unwrap();
        for u in 0..u_segments {
            let u_next = (u + 1) % u_segments;
            faces.push(Face::by_vertices(
                store,
                vec![last_ring[u], top_pole, last_ring[u_next]],
            ));
        }

        Cell::by_faces(store, faces)
    }

    /// Get the outer shell
    pub fn external_boundary(store: &TopologyStore, handle: CellHandle) -> ShellHandle {
        store.cells.read()[handle.index.index()].outer_shell
    }

    /// Get inner shells (voids)
    pub fn internal_boundaries(store: &TopologyStore, handle: CellHandle) -> Vec<ShellHandle> {
        store.cells.read()[handle.index.index()].inner_shells.clone()
    }

    /// Get all shells
    pub fn shells(store: &TopologyStore, handle: CellHandle) -> Vec<ShellHandle> {
        let cells = store.cells.read();
        let cell = &cells[handle.index.index()];
        let mut shells = vec![cell.outer_shell];
        shells.extend(cell.inner_shells.iter().copied());
        shells
    }

    /// Get all faces
    pub fn faces(store: &TopologyStore, handle: CellHandle) -> Vec<FaceHandle> {
        Self::shells(store, handle)
            .iter()
            .flat_map(|s| Shell::faces(store, *s))
            .collect()
    }

    /// Get all edges
    pub fn edges(store: &TopologyStore, handle: CellHandle) -> Vec<EdgeHandle> {
        let mut seen = hashbrown::HashSet::new();
        Self::shells(store, handle)
            .iter()
            .flat_map(|s| Shell::edges(store, *s))
            .filter(|e| seen.insert(*e))
            .collect()
    }

    /// Get all vertices
    pub fn vertices(store: &TopologyStore, handle: CellHandle) -> Vec<VertexHandle> {
        let mut seen = hashbrown::HashSet::new();
        Self::shells(store, handle)
            .iter()
            .flat_map(|s| Shell::vertices(store, *s))
            .filter(|v| seen.insert(*v))
            .collect()
    }

    /// Get all wires
    pub fn wires(store: &TopologyStore, handle: CellHandle) -> Vec<WireHandle> {
        let mut seen = hashbrown::HashSet::new();
        Self::shells(store, handle)
            .iter()
            .flat_map(|s| Shell::wires(store, *s))
            .filter(|w| seen.insert(*w))
            .collect()
    }

    /// Get cell complexes containing this cell
    pub fn cell_complexes(store: &TopologyStore, handle: CellHandle) -> Vec<CellComplexHandle> {
        store.cells.read()[handle.index.index()].cell_complexes.to_vec()
    }

    /// Get adjacent cells (sharing a face)
    pub fn adjacent_cells(store: &TopologyStore, handle: CellHandle) -> Vec<CellHandle> {
        let faces = Self::faces(store, handle);
        let mut adjacent = Vec::new();

        for face in faces {
            for cell in Face::cells(store, face) {
                if cell != handle && !adjacent.contains(&cell) {
                    adjacent.push(cell);
                }
            }
        }

        adjacent
    }

    /// Get shared faces between two cells
    pub fn shared_faces(store: &TopologyStore, c1: CellHandle, c2: CellHandle) -> Vec<FaceHandle> {
        let faces1: hashbrown::HashSet<_> = Self::faces(store, c1).into_iter().collect();
        let faces2: hashbrown::HashSet<_> = Self::faces(store, c2).into_iter().collect();
        faces1.intersection(&faces2).copied().collect()
    }

    /// Get shared edges between two cells
    pub fn shared_edges(store: &TopologyStore, c1: CellHandle, c2: CellHandle) -> Vec<EdgeHandle> {
        let edges1: hashbrown::HashSet<_> = Self::edges(store, c1).into_iter().collect();
        let edges2: hashbrown::HashSet<_> = Self::edges(store, c2).into_iter().collect();
        edges1.intersection(&edges2).copied().collect()
    }

    /// Get shared vertices between two cells
    pub fn shared_vertices(
        store: &TopologyStore,
        c1: CellHandle,
        c2: CellHandle,
    ) -> Vec<VertexHandle> {
        let verts1: hashbrown::HashSet<_> = Self::vertices(store, c1).into_iter().collect();
        let verts2: hashbrown::HashSet<_> = Self::vertices(store, c2).into_iter().collect();
        verts1.intersection(&verts2).copied().collect()
    }

    /// Calculate the volume (cached)
    pub fn volume(store: &TopologyStore, handle: CellHandle) -> f64 {
        // Check cache
        {
            let cells = store.cells.read();
            if let Some(vol) = cells[handle.index.index()].volume_cache {
                return vol;
            }
        }

        // Calculate
        let outer = Self::external_boundary(store, handle);
        let inner = Self::internal_boundaries(store, handle);

        let outer_vol = Shell::volume(store, outer);
        let inner_vol: f64 = inner.iter().map(|s| Shell::volume(store, *s)).sum();
        let vol = outer_vol - inner_vol;

        // Cache
        {
            let mut cells = store.cells.write();
            cells[handle.index.index()].volume_cache = Some(vol);
        }

        vol
    }

    /// Get the surface area
    pub fn area(store: &TopologyStore, handle: CellHandle) -> f64 {
        Self::shells(store, handle)
            .iter()
            .map(|s| Shell::area(store, *s))
            .sum()
    }

    /// Get the center of mass
    pub fn center_of_mass(store: &TopologyStore, handle: CellHandle) -> Point3 {
        let outer = Self::external_boundary(store, handle);
        Shell::center_of_mass(store, outer)
    }

    /// Get the bounding box
    pub fn bounding_box(store: &TopologyStore, handle: CellHandle) -> (Point3, Point3) {
        let vertices = Self::vertices(store, handle);
        if vertices.is_empty() {
            return (Point3::ZERO, Point3::ZERO);
        }

        let points: Vec<_> = vertices.iter().map(|v| Vertex::point(store, *v)).collect();
        crate::geometry::bounding_box(&points).unwrap_or((Point3::ZERO, Point3::ZERO))
    }

    /// Get the compactness (isoperimetric quotient)
    pub fn compactness(store: &TopologyStore, handle: CellHandle) -> f64 {
        let volume = Self::volume(store, handle);
        let area = Self::area(store, handle);

        if area < TOLERANCE {
            return 0.0;
        }

        // Compactness = 36 * pi * V^2 / A^3
        // Normalized so a sphere has compactness = 1
        let pi = std::f64::consts::PI;
        36.0 * pi * volume * volume / (area * area * area)
    }

    /// Check if a point is inside the cell
    pub fn contains_point(store: &TopologyStore, handle: CellHandle, point: Point3) -> bool {
        // Ray casting from the point
        let faces = Self::faces(store, handle);
        let ray_dir = Vector3::new(1.0, 0.0, 0.0);

        let mut intersections = 0;

        for face_handle in faces {
            if let Some(normal) = Face::normal(store, face_handle) {
                let center = Face::center_of_mass(store, face_handle);

                let denom = ray_dir.dot(normal);
                if denom.abs() > TOLERANCE {
                    let t = (center - point).dot(normal) / denom;
                    if t > 0.0 {
                        let intersection = point + ray_dir * t;
                        if Face::contains_point(store, face_handle, intersection) {
                            intersections += 1;
                        }
                    }
                }
            }
        }

        intersections % 2 == 1
    }

    /// Check if the cell is manifold
    pub fn is_manifold(store: &TopologyStore, handle: CellHandle) -> bool {
        let outer = Self::external_boundary(store, handle);
        Shell::is_manifold(store, outer)
    }

    /// Create a torus cell
    pub fn torus(
        store: &TopologyStore,
        center: Point3,
        major_radius: f64,
        minor_radius: f64,
        u_segments: usize,
        v_segments: usize,
        direction: Vector3,
    ) -> CellHandle {
        let u_segments = u_segments.max(4);
        let v_segments = v_segments.max(4);

        // Compute basis vectors
        let dir = direction.normalize();
        let (axis_u, axis_v) = Self::perpendicular_vectors(dir);

        let mut faces = Vec::new();

        // Generate vertices for the torus surface
        let mut rings: Vec<Vec<VertexHandle>> = Vec::new();

        for u in 0..u_segments {
            let theta = std::f64::consts::TAU * u as f64 / u_segments as f64;
            let cos_theta = theta.cos();
            let sin_theta = theta.sin();

            // Center of the tube ring
            let ring_center = center + axis_u * (major_radius * cos_theta) + axis_v * (major_radius * sin_theta);

            // Direction from center to ring_center
            let radial = (axis_u * cos_theta + axis_v * sin_theta).normalize();

            let mut ring = Vec::with_capacity(v_segments);
            for v in 0..v_segments {
                let phi = std::f64::consts::TAU * v as f64 / v_segments as f64;
                let cos_phi = phi.cos();
                let sin_phi = phi.sin();

                // Point on the tube surface
                let point = ring_center + radial * (minor_radius * cos_phi) + dir * (minor_radius * sin_phi);
                ring.push(Vertex::by_point(store, point));
            }
            rings.push(ring);
        }

        // Create quad faces connecting the rings
        for u in 0..u_segments {
            let u_next = (u + 1) % u_segments;
            for v in 0..v_segments {
                let v_next = (v + 1) % v_segments;
                faces.push(Face::by_vertices(
                    store,
                    vec![
                        rings[u][v],
                        rings[u_next][v],
                        rings[u_next][v_next],
                        rings[u][v_next],
                    ],
                ));
            }
        }

        Cell::by_faces(store, faces)
    }

    /// Create perpendicular basis vectors
    fn perpendicular_vectors(dir: Vector3) -> (Vector3, Vector3) {
        let up = if dir.z.abs() < 0.9 {
            Vector3::Z
        } else {
            Vector3::X
        };
        let axis_u = dir.cross(up).normalize();
        let axis_v = dir.cross(axis_u).normalize();
        (axis_u, axis_v)
    }

    /// Create a roof cell from a base face and ridge parameters
    pub fn roof(
        store: &TopologyStore,
        base_face: FaceHandle,
        height: f64,
        roof_type: &str,
    ) -> CellHandle {
        match roof_type.to_lowercase().as_str() {
            "gable" | "gabled" => Self::gable_roof(store, base_face, height),
            "hip" | "hipped" => Self::hip_roof(store, base_face, height),
            "flat" => Self::flat_roof(store, base_face, height),
            "shed" => Self::shed_roof(store, base_face, height),
            _ => Self::gable_roof(store, base_face, height),
        }
    }

    /// Create a gable roof
    fn gable_roof(store: &TopologyStore, base_face: FaceHandle, height: f64) -> CellHandle {
        let vertices = Face::vertices(store, base_face);
        let n = vertices.len();

        if n < 4 {
            // For triangular bases, create a pyramid
            return Self::pyramid_roof(store, base_face, height);
        }

        // Find the bounding box to determine orientation
        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;
        let mut avg_z = 0.0;

        for v in &vertices {
            let p = Vertex::point(store, *v);
            min_x = min_x.min(p.x);
            max_x = max_x.max(p.x);
            min_y = min_y.min(p.y);
            max_y = max_y.max(p.y);
            avg_z += p.z;
        }
        avg_z /= n as f64;

        let width_x = max_x - min_x;
        let width_y = max_y - min_y;

        // Create ridge line along the longer dimension
        let (ridge_start, ridge_end) = if width_x >= width_y {
            (
                Point3::new(min_x, (min_y + max_y) / 2.0, avg_z + height),
                Point3::new(max_x, (min_y + max_y) / 2.0, avg_z + height),
            )
        } else {
            (
                Point3::new((min_x + max_x) / 2.0, min_y, avg_z + height),
                Point3::new((min_x + max_x) / 2.0, max_y, avg_z + height),
            )
        };

        let ridge_v1 = Vertex::by_point(store, ridge_start);
        let ridge_v2 = Vertex::by_point(store, ridge_end);

        let mut faces = Vec::new();

        // Bottom face (reversed)
        faces.push(Face::flip(store, base_face));

        // Side faces connecting base to ridge
        for i in 0..n {
            let j = (i + 1) % n;
            let p1 = Vertex::point(store, vertices[i]);
            let p2 = Vertex::point(store, vertices[j]);

            // Determine which ridge vertex is closer
            let mid = Point3::new((p1.x + p2.x) / 2.0, (p1.y + p2.y) / 2.0, p1.z);
            let closer_ridge = if mid.distance(ridge_start) < mid.distance(ridge_end) {
                ridge_v1
            } else {
                ridge_v2
            };

            faces.push(Face::by_vertices(
                store,
                vec![vertices[i], vertices[j], closer_ridge],
            ));
        }

        // Ridge connecting faces (gable ends)
        // Add triangular gable ends if needed
        faces.push(Face::by_vertices(
            store,
            vec![ridge_v1, ridge_v2, ridge_v1], // This creates a degenerate face, actual implementation would be more complex
        ));

        Cell::by_faces(store, faces)
    }

    /// Create a hip roof
    fn hip_roof(store: &TopologyStore, base_face: FaceHandle, height: f64) -> CellHandle {
        // For a hip roof, all edges slope up to a central ridge
        Self::pyramid_roof(store, base_face, height)
    }

    /// Create a pyramid roof (single apex)
    fn pyramid_roof(store: &TopologyStore, base_face: FaceHandle, height: f64) -> CellHandle {
        let vertices = Face::vertices(store, base_face);
        let center = Face::center_of_mass(store, base_face);
        let apex = Vertex::by_point(store, center + Vector3::new(0.0, 0.0, height));

        let mut faces = Vec::new();

        // Bottom face (reversed)
        faces.push(Face::flip(store, base_face));

        // Triangular side faces
        let n = vertices.len();
        for i in 0..n {
            let j = (i + 1) % n;
            faces.push(Face::by_vertices(
                store,
                vec![vertices[i], vertices[j], apex],
            ));
        }

        Cell::by_faces(store, faces)
    }

    /// Create a flat roof (essentially a prism)
    fn flat_roof(store: &TopologyStore, base_face: FaceHandle, height: f64) -> CellHandle {
        Cell::prism(store, base_face, height)
    }

    /// Create a shed roof (single slope)
    fn shed_roof(store: &TopologyStore, base_face: FaceHandle, height: f64) -> CellHandle {
        let vertices = Face::vertices(store, base_face);
        let n = vertices.len();

        if n < 3 {
            return Cell::prism(store, base_face, height);
        }

        // Create top face with one edge raised
        let top_vertices: Vec<VertexHandle> = vertices
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let p = Vertex::point(store, *v);
                // Raise vertices on one side (first half)
                let h = if i < n / 2 { height } else { 0.0 };
                Vertex::by_point(store, p + Vector3::new(0.0, 0.0, h))
            })
            .collect();

        let mut faces = Vec::new();

        // Bottom face (reversed)
        faces.push(Face::flip(store, base_face));

        // Top face
        let top_wire = Wire::by_vertices(store, top_vertices.clone(), true);
        faces.push(Face::by_external_boundary(store, top_wire));

        // Side faces
        for i in 0..n {
            let j = (i + 1) % n;
            faces.push(Face::by_vertices(
                store,
                vec![vertices[i], vertices[j], top_vertices[j], top_vertices[i]],
            ));
        }

        Cell::by_faces(store, faces)
    }

    /// Create a cone cell
    pub fn cone(
        store: &TopologyStore,
        center: Point3,
        radius: f64,
        height: f64,
        segments: usize,
    ) -> CellHandle {
        let base = Face::circle(store, center, radius, segments.max(3));
        Self::pyramid_roof(store, base, height)
    }

    /// Get the dictionary attached to this cell
    pub fn get_dictionary(store: &TopologyStore, handle: CellHandle) -> Dictionary {
        store.cells.read()[handle.index.index()].dictionary.clone()
    }

    /// Set the dictionary for this cell
    pub fn set_dictionary(store: &TopologyStore, handle: CellHandle, dict: Dictionary) {
        store.cells.write()[handle.index.index()].dictionary = dict;
    }

    /// Set a single key-value pair in the cell's dictionary
    pub fn set_dictionary_value(store: &TopologyStore, handle: CellHandle, key: &str, value: DictionaryValue) {
        store.cells.write()[handle.index.index()].dictionary.set(key, value);
    }

    /// Get a value from the cell's dictionary
    pub fn get_dictionary_value(store: &TopologyStore, handle: CellHandle, key: &str) -> Option<DictionaryValue> {
        store.cells.read()[handle.index.index()].dictionary.get(key).cloned()
    }
}

impl<'a> CellRef<'a> {
    pub fn external_boundary(&self) -> ShellHandle {
        Cell::external_boundary(self.store, self.handle)
    }

    pub fn internal_boundaries(&self) -> Vec<ShellHandle> {
        Cell::internal_boundaries(self.store, self.handle)
    }

    pub fn faces(&self) -> Vec<FaceHandle> {
        Cell::faces(self.store, self.handle)
    }

    pub fn edges(&self) -> Vec<EdgeHandle> {
        Cell::edges(self.store, self.handle)
    }

    pub fn vertices(&self) -> Vec<VertexHandle> {
        Cell::vertices(self.store, self.handle)
    }

    pub fn volume(&self) -> f64 {
        Cell::volume(self.store, self.handle)
    }

    pub fn area(&self) -> f64 {
        Cell::area(self.store, self.handle)
    }

    pub fn center_of_mass(&self) -> Point3 {
        Cell::center_of_mass(self.store, self.handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_cell() {
        let store = TopologyStore::new();
        let cell = Cell::box_cell(&store, Point3::ZERO, 2.0, 2.0, 2.0);

        assert_eq!(Cell::faces(&store, cell).len(), 6);
        assert_eq!(Cell::edges(&store, cell).len(), 12);
        assert_eq!(Cell::vertices(&store, cell).len(), 8);
    }

    #[test]
    fn test_box_volume() {
        let store = TopologyStore::new();
        let cell = Cell::box_cell(&store, Point3::ZERO, 2.0, 3.0, 4.0);

        assert!((Cell::volume(&store, cell) - 24.0).abs() < 0.5);
    }

    #[test]
    fn test_box_area() {
        let store = TopologyStore::new();
        let cell = Cell::box_cell(&store, Point3::ZERO, 2.0, 2.0, 2.0);

        // Surface area = 6 * 4 = 24
        assert!((Cell::area(&store, cell) - 24.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_contains_point() {
        let store = TopologyStore::new();
        let cell = Cell::box_cell(&store, Point3::ZERO, 2.0, 2.0, 2.0);

        assert!(Cell::contains_point(&store, cell, Point3::new(1.0, 1.0, 1.0)));
        assert!(!Cell::contains_point(&store, cell, Point3::new(3.0, 1.0, 1.0)));
    }
}
