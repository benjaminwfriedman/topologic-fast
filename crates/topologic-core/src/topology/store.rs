//! Arena-based topology storage for cache-friendly access

use super::*;
use crate::geometry::Point3;
use hashbrown::HashMap;
use parking_lot::RwLock;
use smallvec::SmallVec;
use std::sync::Arc;

/// Thread-safe topology store using arena allocation
#[derive(Default)]
pub struct TopologyStore {
    pub(crate) vertices: RwLock<Vec<VertexData>>,
    pub(crate) edges: RwLock<Vec<EdgeData>>,
    pub(crate) wires: RwLock<Vec<WireData>>,
    pub(crate) faces: RwLock<Vec<FaceData>>,
    pub(crate) shells: RwLock<Vec<ShellData>>,
    pub(crate) cells: RwLock<Vec<CellData>>,
    pub(crate) cell_complexes: RwLock<Vec<CellComplexData>>,
    pub(crate) clusters: RwLock<Vec<ClusterData>>,

    /// Spatial index for vertices (optional, for fast lookup)
    pub(crate) vertex_index: RwLock<Option<VertexSpatialIndex>>,

    /// Edge index for edge sharing (maps ordered vertex pair to edge handle)
    pub(crate) edge_index: RwLock<HashMap<(u32, u32), EdgeHandle>>,
}

impl TopologyStore {
    /// Create a new empty topology store
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a store with pre-allocated capacity
    pub fn with_capacity(vertices: usize, edges: usize, faces: usize, cells: usize) -> Self {
        Self {
            vertices: RwLock::new(Vec::with_capacity(vertices)),
            edges: RwLock::new(Vec::with_capacity(edges)),
            wires: RwLock::new(Vec::with_capacity(edges)), // Approximate
            faces: RwLock::new(Vec::with_capacity(faces)),
            shells: RwLock::new(Vec::with_capacity(faces / 4)),
            cells: RwLock::new(Vec::with_capacity(cells)),
            cell_complexes: RwLock::new(Vec::with_capacity(cells / 4)),
            clusters: RwLock::new(Vec::new()),
            vertex_index: RwLock::new(None),
            edge_index: RwLock::new(HashMap::with_capacity(edges)),
        }
    }

    /// Get statistics about the store
    pub fn stats(&self) -> StoreStats {
        StoreStats {
            vertices: self.vertices.read().len(),
            edges: self.edges.read().len(),
            wires: self.wires.read().len(),
            faces: self.faces.read().len(),
            shells: self.shells.read().len(),
            cells: self.cells.read().len(),
            cell_complexes: self.cell_complexes.read().len(),
            clusters: self.clusters.read().len(),
        }
    }

    /// Clear all topology data
    pub fn clear(&self) {
        self.vertices.write().clear();
        self.edges.write().clear();
        self.wires.write().clear();
        self.faces.write().clear();
        self.shells.write().clear();
        self.cells.write().clear();
        self.cell_complexes.write().clear();
        self.clusters.write().clear();
        *self.vertex_index.write() = None;
        self.edge_index.write().clear();
    }

    /// Add a vertex and return its handle
    pub fn add_vertex(&self, point: Point3) -> VertexHandle {
        let mut vertices = self.vertices.write();
        let index = ArenaIndex::new(vertices.len() as u32);
        let id = TopologyId::new();
        vertices.push(VertexData {
            id,
            point,
            edges: SmallVec::new(),
            dictionary: Dictionary::new(),
        });
        VertexHandle { index, id }
    }

    /// Add an edge between two vertices (reuses existing edge if one exists)
    pub fn add_edge(&self, start: VertexHandle, end: VertexHandle) -> EdgeHandle {
        // Create ordered key for edge lookup (smaller index first)
        let key = if start.index.index() <= end.index.index() {
            (start.index.0, end.index.0)
        } else {
            (end.index.0, start.index.0)
        };

        // Check if edge already exists
        {
            let edge_index = self.edge_index.read();
            if let Some(&existing) = edge_index.get(&key) {
                return existing;
            }
        }

        // Create new edge (without sharing, then add to index)
        let handle = self.add_edge_no_share(start, end);

        // Add to edge index so future calls can share this edge
        self.edge_index.write().insert(key, handle);

        handle
    }

    /// Add an edge between two vertices without edge sharing (for reverse, etc.)
    pub fn add_edge_no_share(&self, start: VertexHandle, end: VertexHandle) -> EdgeHandle {
        let mut edges = self.edges.write();
        let index = ArenaIndex::new(edges.len() as u32);
        let id = TopologyId::new();

        let handle = EdgeHandle { index, id };

        edges.push(EdgeData {
            id,
            start,
            end,
            wires: SmallVec::new(),
            faces: SmallVec::new(),
            dictionary: Dictionary::new(),
        });

        // Update vertex connectivity
        {
            let mut vertices = self.vertices.write();
            if let Some(v) = vertices.get_mut(start.index.index()) {
                if !v.edges.contains(&handle) {
                    v.edges.push(handle);
                }
            }
            if let Some(v) = vertices.get_mut(end.index.index()) {
                if !v.edges.contains(&handle) {
                    v.edges.push(handle);
                }
            }
        }

        // Don't add to edge index - this edge is not shared

        handle
    }

    /// Add a wire from a sequence of edges with their orientations
    pub fn add_wire(&self, edges: Vec<EdgeHandle>, edge_orientations: Vec<bool>, is_closed: bool) -> WireHandle {
        let mut wires = self.wires.write();
        let index = ArenaIndex::new(wires.len() as u32);
        let id = TopologyId::new();

        let handle = WireHandle { index, id };

        wires.push(WireData {
            id,
            edges: edges.clone(),
            edge_orientations,
            is_closed,
            faces: SmallVec::new(),
            dictionary: Dictionary::new(),
        });

        // Update edge connectivity
        {
            let mut edge_data = self.edges.write();
            for edge in edges {
                if let Some(e) = edge_data.get_mut(edge.index.index()) {
                    e.wires.push(handle);
                }
            }
        }

        handle
    }

    /// Add a face from an outer wire and optional inner wires (holes)
    pub fn add_face(&self, outer_wire: WireHandle, inner_wires: Vec<WireHandle>) -> FaceHandle {
        let mut faces = self.faces.write();
        let index = ArenaIndex::new(faces.len() as u32);
        let id = TopologyId::new();

        let handle = FaceHandle { index, id };

        let all_wires: Vec<_> = std::iter::once(outer_wire).chain(inner_wires.iter().copied()).collect();

        faces.push(FaceData {
            id,
            outer_wire,
            inner_wires,
            shells: SmallVec::new(),
            cells: SmallVec::new(),
            dictionary: Dictionary::new(),
            normal_cache: None,
        });

        // Update wire and edge connectivity
        {
            let mut wire_data = self.wires.write();
            for wire in &all_wires {
                if let Some(w) = wire_data.get_mut(wire.index.index()) {
                    w.faces.push(handle);
                }
            }
        }

        // Update edge-face connectivity
        {
            let wires = self.wires.read();
            let mut edge_data = self.edges.write();
            for wire in &all_wires {
                if let Some(w) = wires.get(wire.index.index()) {
                    for edge in &w.edges {
                        if let Some(e) = edge_data.get_mut(edge.index.index()) {
                            if !e.faces.contains(&handle) {
                                e.faces.push(handle);
                            }
                        }
                    }
                }
            }
        }

        handle
    }

    /// Add a shell from a set of faces
    pub fn add_shell(&self, faces: Vec<FaceHandle>, is_closed: bool) -> ShellHandle {
        let mut shells = self.shells.write();
        let index = ArenaIndex::new(shells.len() as u32);
        let id = TopologyId::new();

        let handle = ShellHandle { index, id };

        shells.push(ShellData {
            id,
            faces: faces.clone(),
            is_closed,
            cells: SmallVec::new(),
            dictionary: Dictionary::new(),
        });

        // Update face connectivity
        {
            let mut face_data = self.faces.write();
            for face in faces {
                if let Some(f) = face_data.get_mut(face.index.index()) {
                    f.shells.push(handle);
                }
            }
        }

        handle
    }

    /// Add a cell from a shell
    pub fn add_cell(&self, outer_shell: ShellHandle, inner_shells: Vec<ShellHandle>) -> CellHandle {
        let mut cells = self.cells.write();
        let index = ArenaIndex::new(cells.len() as u32);
        let id = TopologyId::new();

        let handle = CellHandle { index, id };

        let all_shells: Vec<_> = std::iter::once(outer_shell)
            .chain(inner_shells.iter().copied())
            .collect();

        cells.push(CellData {
            id,
            outer_shell,
            inner_shells,
            cell_complexes: SmallVec::new(),
            dictionary: Dictionary::new(),
            volume_cache: None,
        });

        // Update shell and face connectivity
        {
            let mut shell_data = self.shells.write();
            for shell in &all_shells {
                if let Some(s) = shell_data.get_mut(shell.index.index()) {
                    s.cells.push(handle);
                }
            }
        }

        {
            let shells = self.shells.read();
            let mut face_data = self.faces.write();
            for shell in &all_shells {
                if let Some(s) = shells.get(shell.index.index()) {
                    for face in &s.faces {
                        if let Some(f) = face_data.get_mut(face.index.index()) {
                            if !f.cells.contains(&handle) {
                                f.cells.push(handle);
                            }
                        }
                    }
                }
            }
        }

        handle
    }

    /// Add a cell complex from a set of cells
    pub fn add_cell_complex(&self, cells: Vec<CellHandle>) -> CellComplexHandle {
        let mut cell_complexes = self.cell_complexes.write();
        let index = ArenaIndex::new(cell_complexes.len() as u32);
        let id = TopologyId::new();

        let handle = CellComplexHandle { index, id };

        cell_complexes.push(CellComplexData {
            id,
            cells: cells.clone(),
            dictionary: Dictionary::new(),
        });

        // Update cell connectivity
        {
            let mut cell_data = self.cells.write();
            for cell in cells {
                if let Some(c) = cell_data.get_mut(cell.index.index()) {
                    c.cell_complexes.push(handle);
                }
            }
        }

        handle
    }

    /// Add a cluster from mixed topology handles
    pub fn add_cluster(&self, members: Vec<TopologyHandle>) -> ClusterHandle {
        let mut clusters = self.clusters.write();
        let index = ArenaIndex::new(clusters.len() as u32);
        let id = TopologyId::new();

        clusters.push(ClusterData {
            id,
            members,
            dictionary: Dictionary::new(),
        });

        ClusterHandle { index, id }
    }

    // Accessor methods

    pub fn get_vertex(&self, handle: VertexHandle) -> Option<VertexRef<'_>> {
        let guard = self.vertices.read();
        if handle.index.index() < guard.len() {
            Some(VertexRef {
                store: self,
                handle,
            })
        } else {
            None
        }
    }

    pub fn get_edge(&self, handle: EdgeHandle) -> Option<EdgeRef<'_>> {
        let guard = self.edges.read();
        if handle.index.index() < guard.len() {
            Some(EdgeRef {
                store: self,
                handle,
            })
        } else {
            None
        }
    }

    pub fn get_wire(&self, handle: WireHandle) -> Option<WireRef<'_>> {
        let guard = self.wires.read();
        if handle.index.index() < guard.len() {
            Some(WireRef {
                store: self,
                handle,
            })
        } else {
            None
        }
    }

    pub fn get_face(&self, handle: FaceHandle) -> Option<FaceRef<'_>> {
        let guard = self.faces.read();
        if handle.index.index() < guard.len() {
            Some(FaceRef {
                store: self,
                handle,
            })
        } else {
            None
        }
    }

    pub fn get_shell(&self, handle: ShellHandle) -> Option<ShellRef<'_>> {
        let guard = self.shells.read();
        if handle.index.index() < guard.len() {
            Some(ShellRef {
                store: self,
                handle,
            })
        } else {
            None
        }
    }

    pub fn get_cell(&self, handle: CellHandle) -> Option<CellRef<'_>> {
        let guard = self.cells.read();
        if handle.index.index() < guard.len() {
            Some(CellRef {
                store: self,
                handle,
            })
        } else {
            None
        }
    }

    pub fn get_cell_complex(&self, handle: CellComplexHandle) -> Option<CellComplexRef<'_>> {
        let guard = self.cell_complexes.read();
        if handle.index.index() < guard.len() {
            Some(CellComplexRef {
                store: self,
                handle,
            })
        } else {
            None
        }
    }

    pub fn get_cluster(&self, handle: ClusterHandle) -> Option<ClusterRef<'_>> {
        let guard = self.clusters.read();
        if handle.index.index() < guard.len() {
            Some(ClusterRef {
                store: self,
                handle,
            })
        } else {
            None
        }
    }
}

/// Statistics about the topology store
#[derive(Debug, Clone, Copy, Default)]
pub struct StoreStats {
    pub vertices: usize,
    pub edges: usize,
    pub wires: usize,
    pub faces: usize,
    pub shells: usize,
    pub cells: usize,
    pub cell_complexes: usize,
    pub clusters: usize,
}

/// Spatial index for fast vertex lookup
pub struct VertexSpatialIndex {
    // Using a simple grid-based approach
    // For production, consider using parry3d's BVH
    grid: HashMap<(i32, i32, i32), Vec<VertexHandle>>,
    cell_size: f64,
}

impl VertexSpatialIndex {
    pub fn new(cell_size: f64) -> Self {
        Self {
            grid: HashMap::new(),
            cell_size,
        }
    }

    fn grid_key(&self, point: Point3) -> (i32, i32, i32) {
        (
            (point.x / self.cell_size).floor() as i32,
            (point.y / self.cell_size).floor() as i32,
            (point.z / self.cell_size).floor() as i32,
        )
    }

    pub fn insert(&mut self, point: Point3, handle: VertexHandle) {
        let key = self.grid_key(point);
        self.grid.entry(key).or_default().push(handle);
    }

    pub fn find_nearby(&self, point: Point3, radius: f64) -> Vec<VertexHandle> {
        let mut result = Vec::new();
        let cells = (radius / self.cell_size).ceil() as i32 + 1;
        let center = self.grid_key(point);

        for dx in -cells..=cells {
            for dy in -cells..=cells {
                for dz in -cells..=cells {
                    let key = (center.0 + dx, center.1 + dy, center.2 + dz);
                    if let Some(handles) = self.grid.get(&key) {
                        result.extend(handles.iter().copied());
                    }
                }
            }
        }

        result
    }
}

/// Reference types for accessing topology data
pub struct VertexRef<'a> {
    pub(crate) store: &'a TopologyStore,
    pub(crate) handle: VertexHandle,
}

pub struct EdgeRef<'a> {
    pub(crate) store: &'a TopologyStore,
    pub(crate) handle: EdgeHandle,
}

pub struct WireRef<'a> {
    pub(crate) store: &'a TopologyStore,
    pub(crate) handle: WireHandle,
}

pub struct FaceRef<'a> {
    pub(crate) store: &'a TopologyStore,
    pub(crate) handle: FaceHandle,
}

pub struct ShellRef<'a> {
    pub(crate) store: &'a TopologyStore,
    pub(crate) handle: ShellHandle,
}

pub struct CellRef<'a> {
    pub(crate) store: &'a TopologyStore,
    pub(crate) handle: CellHandle,
}

pub struct CellComplexRef<'a> {
    pub(crate) store: &'a TopologyStore,
    pub(crate) handle: CellComplexHandle,
}

pub struct ClusterRef<'a> {
    pub(crate) store: &'a TopologyStore,
    pub(crate) handle: ClusterHandle,
}
