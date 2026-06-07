//! CellComplex topology (collection of connected cells)

use super::*;
use crate::geometry::{Point3, TOLERANCE};
use rayon::prelude::*;

/// Handle to a cell complex in the topology store
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellComplexHandle {
    pub(crate) index: ArenaIndex,
    pub(crate) id: TopologyId,
}

impl CellComplexHandle {
    pub fn id(&self) -> TopologyId {
        self.id
    }
}

/// Internal cell complex data
pub(crate) struct CellComplexData {
    pub id: TopologyId,
    pub cells: Vec<CellHandle>,
    pub dictionary: Dictionary,
}

/// CellComplex operations
pub struct CellComplex;

impl CellComplex {
    /// Create a cell complex from cells.
    ///
    /// Coincident faces shared between two cells (e.g. a partition wall) are
    /// merged into a single non-manifold face adjacent to both cells, matching
    /// Topologic semantics, so that `Face::cells` reports both owners and the
    /// shared face is counted once.
    pub fn by_cells(store: &TopologyStore, cells: Vec<CellHandle>) -> CellComplexHandle {
        Self::merge_coincident_vertices(store, &cells);
        Self::merge_coincident_faces(store, &cells);
        store.add_cell_complex(cells)
    }

    /// Unify geometrically-coincident vertices across the input cells by
    /// redirecting edge endpoints to a single canonical vertex per location.
    ///
    /// Independently-constructed cells create distinct vertices at shared
    /// locations (e.g. along a partition seam). Without this, a face shared
    /// between two cells and the walls meeting it reference different vertex
    /// handles at the same point, leaving the topology un-sewn — which breaks
    /// reconstruction-based operations such as transforms (the rebuilt shell is
    /// not closed). Sewing the vertices makes the complex consistent.
    fn merge_coincident_vertices(store: &TopologyStore, cells: &[CellHandle]) {
        use hashbrown::{HashMap, HashSet};
        const SCALE: f64 = 1e6;
        let quant = |p: Point3| {
            (
                (p.x * SCALE).round() as i64,
                (p.y * SCALE).round() as i64,
                (p.z * SCALE).round() as i64,
            )
        };

        let mut canonical: HashMap<(i64, i64, i64), VertexHandle> = HashMap::new();
        let mut redirect: HashMap<VertexHandle, VertexHandle> = HashMap::new();
        for &cell in cells {
            for v in Cell::vertices(store, cell) {
                let key = quant(Vertex::point(store, v));
                match canonical.get(&key).copied() {
                    None => {
                        canonical.insert(key, v);
                    }
                    Some(cv) if cv != v => {
                        redirect.insert(v, cv);
                    }
                    Some(_) => {}
                }
            }
        }
        if redirect.is_empty() {
            return;
        }

        // Collect the unique edges of all cells, then redirect their endpoints.
        let mut seen = HashSet::new();
        let mut edge_handles = Vec::new();
        for &cell in cells {
            for e in Cell::edges(store, cell) {
                if seen.insert(e) {
                    edge_handles.push(e);
                }
            }
        }
        let mut edges = store.edges.write();
        for e in edge_handles {
            if let Some(ed) = edges.get_mut(e.index.index()) {
                if let Some(&nv) = redirect.get(&ed.start) {
                    ed.start = nv;
                }
                if let Some(&nv) = redirect.get(&ed.end) {
                    ed.end = nv;
                }
            }
        }
    }

    /// Merge geometrically-coincident faces across the input cells so a shared
    /// partition is represented by a single face referenced by both cells,
    /// rather than duplicate coincident faces each owned by one cell.
    ///
    /// Two faces are considered coincident when their boundary vertex sets are
    /// equal (order- and winding-independent) after quantizing coordinates. The
    /// duplicate's owning shell is redirected to the canonical face, and the
    /// canonical face records the second cell as an additional owner.
    fn merge_coincident_faces(store: &TopologyStore, cells: &[CellHandle]) {
        use hashbrown::HashMap;
        const SCALE: f64 = 1e6;
        let quant = |p: Point3| {
            (
                (p.x * SCALE).round() as i64,
                (p.y * SCALE).round() as i64,
                (p.z * SCALE).round() as i64,
            )
        };
        let face_key = |face: FaceHandle| {
            let mut k: Vec<(i64, i64, i64)> = Face::vertices(store, face)
                .iter()
                .map(|v| quant(Vertex::point(store, *v)))
                .collect();
            k.sort_unstable();
            k
        };

        let mut canon: HashMap<Vec<(i64, i64, i64)>, FaceHandle> = HashMap::new();
        for &cell in cells {
            for shell in Cell::shells(store, cell) {
                for face in Shell::faces(store, shell) {
                    let key = face_key(face);
                    if key.len() < 3 {
                        continue;
                    }
                    match canon.get(&key).copied() {
                        None => {
                            canon.insert(key, face);
                        }
                        Some(canonical) if canonical != face => {
                            // Redirect this shell's reference from the duplicate
                            // face to the canonical (shared) face.
                            {
                                let mut shells = store.shells.write();
                                if let Some(s) = shells.get_mut(shell.index.index()) {
                                    for f in s.faces.iter_mut() {
                                        if *f == face {
                                            *f = canonical;
                                        }
                                    }
                                }
                            }
                            // Record this cell (and shell) as an owner of the
                            // canonical face so adjacency queries see both cells.
                            {
                                let mut faces = store.faces.write();
                                if let Some(cf) = faces.get_mut(canonical.index.index()) {
                                    if !cf.cells.contains(&cell) {
                                        cf.cells.push(cell);
                                    }
                                    if !cf.shells.contains(&shell) {
                                        cf.shells.push(shell);
                                    }
                                }
                            }
                        }
                        Some(_) => {}
                    }
                }
            }
        }
    }

    /// Create a cell complex from faces (finds connected volumes)
    pub fn by_faces(store: &TopologyStore, faces: Vec<FaceHandle>) -> CellComplexHandle {
        // This is a complex operation - for now, just create shells and cells
        let shell = Shell::by_faces(store, faces);
        if Shell::is_closed(store, shell) {
            let cell = Cell::by_shell(store, shell);
            Self::by_cells(store, vec![cell])
        } else {
            // Create a single cell anyway
            let cell = Cell::by_shell(store, shell);
            Self::by_cells(store, vec![cell])
        }
    }

    /// Create a box cell complex (single cell)
    pub fn box_complex(
        store: &TopologyStore,
        origin: Point3,
        width: f64,
        length: f64,
        height: f64,
    ) -> CellComplexHandle {
        let cell = Cell::box_cell(store, origin, width, length, height);
        Self::by_cells(store, vec![cell])
    }

    /// Get all cells
    pub fn cells(store: &TopologyStore, handle: CellComplexHandle) -> Vec<CellHandle> {
        store.cell_complexes.read()[handle.index.index()].cells.clone()
    }

    /// Get all shells
    pub fn shells(store: &TopologyStore, handle: CellComplexHandle) -> Vec<ShellHandle> {
        let mut seen = hashbrown::HashSet::new();
        Self::cells(store, handle)
            .iter()
            .flat_map(|c| Cell::shells(store, *c))
            .filter(|s| seen.insert(*s))
            .collect()
    }

    /// Get all faces
    pub fn faces(store: &TopologyStore, handle: CellComplexHandle) -> Vec<FaceHandle> {
        let mut seen = hashbrown::HashSet::new();
        Self::cells(store, handle)
            .iter()
            .flat_map(|c| Cell::faces(store, *c))
            .filter(|f| seen.insert(*f))
            .collect()
    }

    /// Get all wires
    pub fn wires(store: &TopologyStore, handle: CellComplexHandle) -> Vec<WireHandle> {
        let mut seen = hashbrown::HashSet::new();
        Self::cells(store, handle)
            .iter()
            .flat_map(|c| Cell::wires(store, *c))
            .filter(|w| seen.insert(*w))
            .collect()
    }

    /// Get all edges
    pub fn edges(store: &TopologyStore, handle: CellComplexHandle) -> Vec<EdgeHandle> {
        let mut seen = hashbrown::HashSet::new();
        Self::cells(store, handle)
            .iter()
            .flat_map(|c| Cell::edges(store, *c))
            .filter(|e| seen.insert(*e))
            .collect()
    }

    /// Get all vertices
    pub fn vertices(store: &TopologyStore, handle: CellComplexHandle) -> Vec<VertexHandle> {
        let mut seen = hashbrown::HashSet::new();
        Self::cells(store, handle)
            .iter()
            .flat_map(|c| Cell::vertices(store, *c))
            .filter(|v| seen.insert(*v))
            .collect()
    }

    /// Get the external boundary (outer shell of the complex)
    pub fn external_boundary(store: &TopologyStore, handle: CellComplexHandle) -> ShellHandle {
        let faces = Self::non_manifold_faces(store, handle);
        Shell::by_faces(store, faces)
    }

    /// Get internal boundaries (shared faces)
    pub fn internal_boundaries(store: &TopologyStore, handle: CellComplexHandle) -> Vec<FaceHandle> {
        let cells: hashbrown::HashSet<_> = Self::cells(store, handle).into_iter().collect();
        let faces = Self::faces(store, handle);

        faces
            .into_iter()
            .filter(|f| {
                let face_cells: Vec<_> = Face::cells(store, *f)
                    .into_iter()
                    .filter(|c| cells.contains(c))
                    .collect();
                face_cells.len() == 2
            })
            .collect()
    }

    /// Get non-manifold faces (used by != 2 cells in the complex)
    pub fn non_manifold_faces(store: &TopologyStore, handle: CellComplexHandle) -> Vec<FaceHandle> {
        let cells: hashbrown::HashSet<_> = Self::cells(store, handle).into_iter().collect();
        let faces = Self::faces(store, handle);

        faces
            .into_iter()
            .filter(|f| {
                let count = Face::cells(store, *f)
                    .into_iter()
                    .filter(|c| cells.contains(c))
                    .count();
                count != 2
            })
            .collect()
    }

    /// Get boundary faces (used by exactly 1 cell)
    pub fn boundary_faces(store: &TopologyStore, handle: CellComplexHandle) -> Vec<FaceHandle> {
        let cells: hashbrown::HashSet<_> = Self::cells(store, handle).into_iter().collect();
        let faces = Self::faces(store, handle);

        faces
            .into_iter()
            .filter(|f| {
                let count = Face::cells(store, *f)
                    .into_iter()
                    .filter(|c| cells.contains(c))
                    .count();
                count == 1
            })
            .collect()
    }

    /// Get total volume
    pub fn volume(store: &TopologyStore, handle: CellComplexHandle) -> f64 {
        Self::cells(store, handle)
            .iter()
            .map(|c| Cell::volume(store, *c))
            .sum()
    }

    /// Get total surface area
    pub fn area(store: &TopologyStore, handle: CellComplexHandle) -> f64 {
        // Only count boundary faces
        Self::boundary_faces(store, handle)
            .iter()
            .map(|f| Face::area(store, *f))
            .sum()
    }

    /// Get the center of mass
    pub fn center_of_mass(store: &TopologyStore, handle: CellComplexHandle) -> Point3 {
        let cells = Self::cells(store, handle);
        if cells.is_empty() {
            return Point3::ZERO;
        }

        // Weight by volume
        let mut total_volume = 0.0;
        let mut weighted_center = Point3::ZERO;

        for cell in cells {
            let vol = Cell::volume(store, cell);
            let center = Cell::center_of_mass(store, cell);
            weighted_center += center * vol;
            total_volume += vol;
        }

        if total_volume > TOLERANCE {
            weighted_center / total_volume
        } else {
            Point3::ZERO
        }
    }

    /// Get the bounding box
    pub fn bounding_box(store: &TopologyStore, handle: CellComplexHandle) -> (Point3, Point3) {
        let vertices = Self::vertices(store, handle);
        if vertices.is_empty() {
            return (Point3::ZERO, Point3::ZERO);
        }

        let points: Vec<_> = vertices.iter().map(|v| Vertex::point(store, *v)).collect();
        crate::geometry::bounding_box(&points).unwrap_or((Point3::ZERO, Point3::ZERO))
    }

    /// Get the number of cells
    pub fn num_cells(store: &TopologyStore, handle: CellComplexHandle) -> usize {
        Self::cells(store, handle).len()
    }

    /// Check if the complex is manifold
    pub fn is_manifold(store: &TopologyStore, handle: CellComplexHandle) -> bool {
        Self::non_manifold_faces(store, handle)
            .iter()
            .all(|f| {
                let cells: hashbrown::HashSet<_> = Self::cells(store, handle).into_iter().collect();
                let count = Face::cells(store, *f)
                    .into_iter()
                    .filter(|c| cells.contains(c))
                    .count();
                count <= 2
            })
    }

    /// Decompose into connected components
    pub fn decompose(store: &TopologyStore, handle: CellComplexHandle) -> Vec<CellComplexHandle> {
        let cells = Self::cells(store, handle);
        if cells.is_empty() {
            return vec![];
        }

        // Union-find for connected components
        let mut parent: hashbrown::HashMap<CellHandle, CellHandle> = cells.iter().map(|c| (*c, *c)).collect();

        fn find(parent: &mut hashbrown::HashMap<CellHandle, CellHandle>, c: CellHandle) -> CellHandle {
            if parent[&c] != c {
                let root = find(parent, parent[&c]);
                parent.insert(c, root);
            }
            parent[&c]
        }

        fn union(parent: &mut hashbrown::HashMap<CellHandle, CellHandle>, a: CellHandle, b: CellHandle) {
            let ra = find(parent, a);
            let rb = find(parent, b);
            if ra != rb {
                parent.insert(ra, rb);
            }
        }

        // Connect cells that share faces
        for cell in &cells {
            for adjacent in Cell::adjacent_cells(store, *cell) {
                if parent.contains_key(&adjacent) {
                    union(&mut parent, *cell, adjacent);
                }
            }
        }

        // Group by component
        let mut components: hashbrown::HashMap<CellHandle, Vec<CellHandle>> = hashbrown::HashMap::new();
        for cell in cells {
            let root = find(&mut parent, cell);
            components.entry(root).or_default().push(cell);
        }

        // Create cell complexes for each component
        components
            .into_values()
            .map(|cells| Self::by_cells(store, cells))
            .collect()
    }

    /// Check if a point is inside the cell complex
    pub fn contains_point(store: &TopologyStore, handle: CellComplexHandle, point: Point3) -> bool {
        Self::cells(store, handle)
            .iter()
            .any(|c| Cell::contains_point(store, *c, point))
    }

    /// Get the enclosing cell for a vertex
    pub fn enclosing_cell(
        store: &TopologyStore,
        handle: CellComplexHandle,
        vertex: VertexHandle,
    ) -> Option<CellHandle> {
        let point = Vertex::point(store, vertex);
        Self::cells(store, handle)
            .into_iter()
            .find(|c| Cell::contains_point(store, *c, point))
    }

    /// Get the dictionary attached to this cell complex
    pub fn get_dictionary(store: &TopologyStore, handle: CellComplexHandle) -> Dictionary {
        store.cell_complexes.read()[handle.index.index()].dictionary.clone()
    }

    /// Set the dictionary for this cell complex
    pub fn set_dictionary(store: &TopologyStore, handle: CellComplexHandle, dict: Dictionary) {
        store.cell_complexes.write()[handle.index.index()].dictionary = dict;
    }

    /// Set a single key-value pair in the cell complex's dictionary
    pub fn set_dictionary_value(store: &TopologyStore, handle: CellComplexHandle, key: &str, value: DictionaryValue) {
        store.cell_complexes.write()[handle.index.index()].dictionary.set(key, value);
    }

    /// Get a value from the cell complex's dictionary
    pub fn get_dictionary_value(store: &TopologyStore, handle: CellComplexHandle, key: &str) -> Option<DictionaryValue> {
        store.cell_complexes.read()[handle.index.index()].dictionary.get(key).cloned()
    }
}

impl<'a> CellComplexRef<'a> {
    pub fn cells(&self) -> Vec<CellHandle> {
        CellComplex::cells(self.store, self.handle)
    }

    pub fn faces(&self) -> Vec<FaceHandle> {
        CellComplex::faces(self.store, self.handle)
    }

    pub fn edges(&self) -> Vec<EdgeHandle> {
        CellComplex::edges(self.store, self.handle)
    }

    pub fn vertices(&self) -> Vec<VertexHandle> {
        CellComplex::vertices(self.store, self.handle)
    }

    pub fn volume(&self) -> f64 {
        CellComplex::volume(self.store, self.handle)
    }

    pub fn area(&self) -> f64 {
        CellComplex::area(self.store, self.handle)
    }

    pub fn center_of_mass(&self) -> Point3 {
        CellComplex::center_of_mass(self.store, self.handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_complex() {
        let store = TopologyStore::new();
        let complex = CellComplex::box_complex(&store, Point3::ZERO, 2.0, 2.0, 2.0);

        assert_eq!(CellComplex::num_cells(&store, complex), 1);
        assert_eq!(CellComplex::faces(&store, complex).len(), 6);
    }

    #[test]
    fn test_complex_volume() {
        let store = TopologyStore::new();
        let complex = CellComplex::box_complex(&store, Point3::ZERO, 2.0, 2.0, 2.0);

        assert!((CellComplex::volume(&store, complex) - 8.0).abs() < 0.5);
    }

    #[test]
    fn test_shared_face_merged() {
        // Two unit cubes stacked along Z share the face at z = 1.
        let store = TopologyStore::new();
        let c1 = Cell::box_cell(&store, Point3::ZERO, 1.0, 1.0, 1.0);
        let c2 = Cell::box_cell(&store, Point3::new(0.0, 0.0, 1.0), 1.0, 1.0, 1.0);
        let complex = CellComplex::by_cells(&store, vec![c1, c2]);

        // 6 + 6 - 1 shared = 11 distinct faces.
        let faces = CellComplex::faces(&store, complex);
        assert_eq!(faces.len(), 11, "shared face should be merged");

        // Exactly one face is shared by both cells; the rest are boundary.
        let counts: Vec<usize> = faces
            .iter()
            .map(|f| Face::cells(&store, *f).len())
            .collect();
        assert_eq!(counts.iter().filter(|&&n| n == 2).count(), 1);
        assert_eq!(counts.iter().filter(|&&n| n == 1).count(), 10);

        // Internal/boundary classification reflects the sharing.
        assert_eq!(CellComplex::internal_boundaries(&store, complex).len(), 1);
        assert_eq!(CellComplex::boundary_faces(&store, complex).len(), 10);

        // Volume is preserved.
        assert!((CellComplex::volume(&store, complex) - 2.0).abs() < 1e-9);
    }
}
