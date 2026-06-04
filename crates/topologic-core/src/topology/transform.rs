//! Topology transformations (translate, rotate, scale, etc.)

use super::*;
use crate::geometry::{Point3, Vector3, TOLERANCE};
use crate::utilities::{MatrixUtil, Matrix4x4};

/// Topology transformation operations
pub struct TopologyTransform;

impl TopologyTransform {
    /// Transform a vertex by a 4x4 matrix
    pub fn transform_vertex(
        store: &TopologyStore,
        handle: VertexHandle,
        matrix: Matrix4x4,
    ) -> VertexHandle {
        let point = Vertex::point(store, handle);
        let transformed = MatrixUtil::transform_point(matrix, [point.x, point.y, point.z]);
        Vertex::by_coordinates(store, transformed[0], transformed[1], transformed[2])
    }

    /// Transform an edge by a 4x4 matrix
    pub fn transform_edge(
        store: &TopologyStore,
        handle: EdgeHandle,
        matrix: Matrix4x4,
    ) -> EdgeHandle {
        let start = Edge::start_vertex(store, handle);
        let end = Edge::end_vertex(store, handle);

        let new_start = Self::transform_vertex(store, start, matrix);
        let new_end = Self::transform_vertex(store, end, matrix);

        Edge::by_start_vertex_end_vertex(store, new_start, new_end)
    }

    /// Transform a wire by a 4x4 matrix
    pub fn transform_wire(
        store: &TopologyStore,
        handle: WireHandle,
        matrix: Matrix4x4,
    ) -> WireHandle {
        let edges = Wire::edges(store, handle);
        let new_edges: Vec<_> = edges
            .iter()
            .map(|e| Self::transform_edge(store, *e, matrix))
            .collect();
        Wire::by_edges(store, new_edges)
    }

    /// Transform a face by a 4x4 matrix
    /// Transforms vertices first, then rebuilds wires to maintain proper connectivity
    pub fn transform_face(
        store: &TopologyStore,
        handle: FaceHandle,
        matrix: Matrix4x4,
    ) -> FaceHandle {
        use hashbrown::HashMap;

        // Collect all unique vertices from the face and transform them
        let mut vertex_map: HashMap<VertexHandle, VertexHandle> = HashMap::new();
        for vertex in Face::vertices(store, handle) {
            vertex_map.entry(vertex).or_insert_with(|| {
                Self::transform_vertex(store, vertex, matrix)
            });
        }

        // Transform the outer wire
        let outer = Face::external_boundary(store, handle);
        let outer_verts = Wire::vertices(store, outer);
        let new_outer_verts: Vec<_> = outer_verts
            .iter()
            .map(|v| *vertex_map.get(v).unwrap_or(v))
            .collect();
        let is_closed = Wire::is_closed(store, outer);
        let new_outer = Wire::by_vertices(store, new_outer_verts, is_closed);

        // Transform inner wires (holes)
        let inner = Face::internal_boundaries(store, handle);
        let new_inner: Vec<_> = inner
            .iter()
            .map(|w| {
                let inner_verts = Wire::vertices(store, *w);
                let new_inner_verts: Vec<_> = inner_verts
                    .iter()
                    .map(|v| *vertex_map.get(v).unwrap_or(v))
                    .collect();
                let is_closed = Wire::is_closed(store, *w);
                Wire::by_vertices(store, new_inner_verts, is_closed)
            })
            .collect();

        Face::by_external_internal_boundaries(store, new_outer, new_inner)
    }

    /// Transform a shell by a 4x4 matrix
    /// This ensures vertices are shared between faces for proper connectivity
    pub fn transform_shell(
        store: &TopologyStore,
        handle: ShellHandle,
        matrix: Matrix4x4,
    ) -> ShellHandle {
        use hashbrown::HashMap;

        // Get all faces from the shell
        let faces = Shell::faces(store, handle);

        // Collect all unique vertices from the shell and transform them
        let mut vertex_map: HashMap<VertexHandle, VertexHandle> = HashMap::new();
        for face in &faces {
            for vertex in Face::vertices(store, *face) {
                vertex_map.entry(vertex).or_insert_with(|| {
                    Self::transform_vertex(store, vertex, matrix)
                });
            }
        }

        // Create new faces using the mapped vertices
        let new_faces: Vec<_> = faces
            .iter()
            .map(|face| {
                // Transform the outer wire
                let outer = Face::external_boundary(store, *face);
                let outer_verts = Wire::vertices(store, outer);
                let new_outer_verts: Vec<_> = outer_verts
                    .iter()
                    .map(|v| *vertex_map.get(v).unwrap_or(v))
                    .collect();
                let is_closed = Wire::is_closed(store, outer);
                let new_outer = Wire::by_vertices(store, new_outer_verts, is_closed);

                // Transform inner wires (holes)
                let inner = Face::internal_boundaries(store, *face);
                let new_inner: Vec<_> = inner
                    .iter()
                    .map(|w| {
                        let inner_verts = Wire::vertices(store, *w);
                        let new_inner_verts: Vec<_> = inner_verts
                            .iter()
                            .map(|v| *vertex_map.get(v).unwrap_or(v))
                            .collect();
                        let is_closed = Wire::is_closed(store, *w);
                        Wire::by_vertices(store, new_inner_verts, is_closed)
                    })
                    .collect();

                Face::by_external_internal_boundaries(store, new_outer, new_inner)
            })
            .collect();

        Shell::by_faces(store, new_faces)
    }

    /// Transform a cell by a 4x4 matrix
    pub fn transform_cell(
        store: &TopologyStore,
        handle: CellHandle,
        matrix: Matrix4x4,
    ) -> CellHandle {
        let shell = Cell::external_boundary(store, handle);
        let new_shell = Self::transform_shell(store, shell, matrix);
        Cell::by_shell(store, new_shell)
    }

    /// Transform a cell complex by a 4x4 matrix
    pub fn transform_cell_complex(
        store: &TopologyStore,
        handle: CellComplexHandle,
        matrix: Matrix4x4,
    ) -> CellComplexHandle {
        let cells = CellComplex::cells(store, handle);
        let new_cells: Vec<_> = cells
            .iter()
            .map(|c| Self::transform_cell(store, *c, matrix))
            .collect();
        CellComplex::by_cells(store, new_cells)
    }

    /// Transform a cluster by a 4x4 matrix
    pub fn transform_cluster(
        store: &TopologyStore,
        handle: ClusterHandle,
        matrix: Matrix4x4,
    ) -> ClusterHandle {
        let vertices = Cluster::vertices(store, handle);
        let edges = Cluster::edges(store, handle);
        let faces = Cluster::faces(store, handle);
        let cells = Cluster::cells(store, handle);

        let new_vertices: Vec<_> = vertices.iter().map(|v| Self::transform_vertex(store, *v, matrix)).collect();
        let new_edges: Vec<_> = edges.iter().map(|e| Self::transform_edge(store, *e, matrix)).collect();
        let new_faces: Vec<_> = faces.iter().map(|f| Self::transform_face(store, *f, matrix)).collect();
        let new_cells: Vec<_> = cells.iter().map(|c| Self::transform_cell(store, *c, matrix)).collect();

        // Build cluster from all transformed elements
        if !new_cells.is_empty() {
            Cluster::by_cells(store, new_cells)
        } else if !new_faces.is_empty() {
            Cluster::by_faces(store, new_faces)
        } else if !new_edges.is_empty() {
            Cluster::by_edges(store, new_edges)
        } else {
            Cluster::by_vertices(store, new_vertices)
        }
    }

    /// Translate a vertex
    pub fn translate_vertex(
        store: &TopologyStore,
        handle: VertexHandle,
        x: f64,
        y: f64,
        z: f64,
    ) -> VertexHandle {
        let matrix = MatrixUtil::by_translation(x, y, z);
        Self::transform_vertex(store, handle, matrix)
    }

    /// Translate an edge
    pub fn translate_edge(
        store: &TopologyStore,
        handle: EdgeHandle,
        x: f64,
        y: f64,
        z: f64,
    ) -> EdgeHandle {
        let matrix = MatrixUtil::by_translation(x, y, z);
        Self::transform_edge(store, handle, matrix)
    }

    /// Translate a wire
    pub fn translate_wire(
        store: &TopologyStore,
        handle: WireHandle,
        x: f64,
        y: f64,
        z: f64,
    ) -> WireHandle {
        let matrix = MatrixUtil::by_translation(x, y, z);
        Self::transform_wire(store, handle, matrix)
    }

    /// Translate a face
    pub fn translate_face(
        store: &TopologyStore,
        handle: FaceHandle,
        x: f64,
        y: f64,
        z: f64,
    ) -> FaceHandle {
        let matrix = MatrixUtil::by_translation(x, y, z);
        Self::transform_face(store, handle, matrix)
    }

    /// Translate a shell
    pub fn translate_shell(
        store: &TopologyStore,
        handle: ShellHandle,
        x: f64,
        y: f64,
        z: f64,
    ) -> ShellHandle {
        let matrix = MatrixUtil::by_translation(x, y, z);
        Self::transform_shell(store, handle, matrix)
    }

    /// Translate a cell
    pub fn translate_cell(
        store: &TopologyStore,
        handle: CellHandle,
        x: f64,
        y: f64,
        z: f64,
    ) -> CellHandle {
        let matrix = MatrixUtil::by_translation(x, y, z);
        Self::transform_cell(store, handle, matrix)
    }

    /// Rotate a vertex around an axis by an angle (degrees)
    pub fn rotate_vertex(
        store: &TopologyStore,
        handle: VertexHandle,
        origin: Point3,
        axis: Vector3,
        angle: f64,
    ) -> VertexHandle {
        let matrix = Self::rotation_matrix(origin, axis, angle);
        Self::transform_vertex(store, handle, matrix)
    }

    /// Rotate an edge around an axis by an angle (degrees)
    pub fn rotate_edge(
        store: &TopologyStore,
        handle: EdgeHandle,
        origin: Point3,
        axis: Vector3,
        angle: f64,
    ) -> EdgeHandle {
        let matrix = Self::rotation_matrix(origin, axis, angle);
        Self::transform_edge(store, handle, matrix)
    }

    /// Rotate a wire around an axis by an angle (degrees)
    pub fn rotate_wire(
        store: &TopologyStore,
        handle: WireHandle,
        origin: Point3,
        axis: Vector3,
        angle: f64,
    ) -> WireHandle {
        let matrix = Self::rotation_matrix(origin, axis, angle);
        Self::transform_wire(store, handle, matrix)
    }

    /// Rotate a face around an axis by an angle (degrees)
    pub fn rotate_face(
        store: &TopologyStore,
        handle: FaceHandle,
        origin: Point3,
        axis: Vector3,
        angle: f64,
    ) -> FaceHandle {
        let matrix = Self::rotation_matrix(origin, axis, angle);
        Self::transform_face(store, handle, matrix)
    }

    /// Rotate a shell around an axis by an angle (degrees)
    pub fn rotate_shell(
        store: &TopologyStore,
        handle: ShellHandle,
        origin: Point3,
        axis: Vector3,
        angle: f64,
    ) -> ShellHandle {
        let matrix = Self::rotation_matrix(origin, axis, angle);
        Self::transform_shell(store, handle, matrix)
    }

    /// Rotate a cell around an axis by an angle (degrees)
    pub fn rotate_cell(
        store: &TopologyStore,
        handle: CellHandle,
        origin: Point3,
        axis: Vector3,
        angle: f64,
    ) -> CellHandle {
        let matrix = Self::rotation_matrix(origin, axis, angle);
        Self::transform_cell(store, handle, matrix)
    }

    /// Scale a vertex from an origin
    pub fn scale_vertex(
        store: &TopologyStore,
        handle: VertexHandle,
        origin: Point3,
        sx: f64,
        sy: f64,
        sz: f64,
    ) -> VertexHandle {
        let matrix = Self::scale_matrix(origin, sx, sy, sz);
        Self::transform_vertex(store, handle, matrix)
    }

    /// Scale an edge from an origin
    pub fn scale_edge(
        store: &TopologyStore,
        handle: EdgeHandle,
        origin: Point3,
        sx: f64,
        sy: f64,
        sz: f64,
    ) -> EdgeHandle {
        let matrix = Self::scale_matrix(origin, sx, sy, sz);
        Self::transform_edge(store, handle, matrix)
    }

    /// Scale a wire from an origin
    pub fn scale_wire(
        store: &TopologyStore,
        handle: WireHandle,
        origin: Point3,
        sx: f64,
        sy: f64,
        sz: f64,
    ) -> WireHandle {
        let matrix = Self::scale_matrix(origin, sx, sy, sz);
        Self::transform_wire(store, handle, matrix)
    }

    /// Scale a face from an origin
    pub fn scale_face(
        store: &TopologyStore,
        handle: FaceHandle,
        origin: Point3,
        sx: f64,
        sy: f64,
        sz: f64,
    ) -> FaceHandle {
        let matrix = Self::scale_matrix(origin, sx, sy, sz);
        Self::transform_face(store, handle, matrix)
    }

    /// Scale a shell from an origin
    pub fn scale_shell(
        store: &TopologyStore,
        handle: ShellHandle,
        origin: Point3,
        sx: f64,
        sy: f64,
        sz: f64,
    ) -> ShellHandle {
        let matrix = Self::scale_matrix(origin, sx, sy, sz);
        Self::transform_shell(store, handle, matrix)
    }

    /// Scale a cell from an origin
    pub fn scale_cell(
        store: &TopologyStore,
        handle: CellHandle,
        origin: Point3,
        sx: f64,
        sy: f64,
        sz: f64,
    ) -> CellHandle {
        let matrix = Self::scale_matrix(origin, sx, sy, sz);
        Self::transform_cell(store, handle, matrix)
    }

    /// Helper: Create rotation matrix around an arbitrary axis
    fn rotation_matrix(origin: Point3, axis: Vector3, angle_degrees: f64) -> Matrix4x4 {
        let angle = angle_degrees.to_radians();
        let axis = axis.normalize_or_zero();

        // Rodrigues' rotation formula
        let c = angle.cos();
        let s = angle.sin();
        let t = 1.0 - c;
        let x = axis.x;
        let y = axis.y;
        let z = axis.z;

        // Rotation matrix
        let r00 = t * x * x + c;
        let r01 = t * x * y - s * z;
        let r02 = t * x * z + s * y;
        let r10 = t * x * y + s * z;
        let r11 = t * y * y + c;
        let r12 = t * y * z - s * x;
        let r20 = t * x * z - s * y;
        let r21 = t * y * z + s * x;
        let r22 = t * z * z + c;

        // Translate to origin, rotate, translate back
        let tx = origin.x - (r00 * origin.x + r01 * origin.y + r02 * origin.z);
        let ty = origin.y - (r10 * origin.x + r11 * origin.y + r12 * origin.z);
        let tz = origin.z - (r20 * origin.x + r21 * origin.y + r22 * origin.z);

        [
            [r00, r01, r02, tx],
            [r10, r11, r12, ty],
            [r20, r21, r22, tz],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }

    /// Helper: Create scale matrix from an origin
    fn scale_matrix(origin: Point3, sx: f64, sy: f64, sz: f64) -> Matrix4x4 {
        // Translate to origin, scale, translate back
        let tx = origin.x * (1.0 - sx);
        let ty = origin.y * (1.0 - sy);
        let tz = origin.z * (1.0 - sz);

        [
            [sx, 0.0, 0.0, tx],
            [0.0, sy, 0.0, ty],
            [0.0, 0.0, sz, tz],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }
}

/// Topology utility operations
pub struct TopologyOps;

impl TopologyOps {
    /// Transform topology by a 4x4 matrix
    pub fn transform(
        store: &TopologyStore,
        topology: TopologyHandle,
        matrix: Matrix4x4,
    ) -> TopologyHandle {
        match topology {
            TopologyHandle::Vertex(h) => TopologyHandle::Vertex(TopologyTransform::transform_vertex(store, h, matrix)),
            TopologyHandle::Edge(h) => TopologyHandle::Edge(TopologyTransform::transform_edge(store, h, matrix)),
            TopologyHandle::Wire(h) => TopologyHandle::Wire(TopologyTransform::transform_wire(store, h, matrix)),
            TopologyHandle::Face(h) => TopologyHandle::Face(TopologyTransform::transform_face(store, h, matrix)),
            TopologyHandle::Shell(h) => TopologyHandle::Shell(TopologyTransform::transform_shell(store, h, matrix)),
            TopologyHandle::Cell(h) => TopologyHandle::Cell(TopologyTransform::transform_cell(store, h, matrix)),
            TopologyHandle::CellComplex(h) => TopologyHandle::CellComplex(TopologyTransform::transform_cell_complex(store, h, matrix)),
            TopologyHandle::Cluster(h) => TopologyHandle::Cluster(TopologyTransform::transform_cluster(store, h, matrix)),
        }
    }

    /// Translate topology
    pub fn translate(
        store: &TopologyStore,
        topology: TopologyHandle,
        x: f64,
        y: f64,
        z: f64,
    ) -> TopologyHandle {
        let matrix = MatrixUtil::by_translation(x, y, z);
        Self::transform(store, topology, matrix)
    }

    /// Rotate topology around an axis
    pub fn rotate(
        store: &TopologyStore,
        topology: TopologyHandle,
        origin: Point3,
        axis: Vector3,
        angle: f64,
    ) -> TopologyHandle {
        let matrix = TopologyTransform::rotation_matrix(origin, axis, angle);
        Self::transform(store, topology, matrix)
    }

    /// Scale topology from an origin
    pub fn scale(
        store: &TopologyStore,
        topology: TopologyHandle,
        origin: Point3,
        sx: f64,
        sy: f64,
        sz: f64,
    ) -> TopologyHandle {
        let matrix = TopologyTransform::scale_matrix(origin, sx, sy, sz);
        Self::transform(store, topology, matrix)
    }

    /// Get the centroid of any topology
    pub fn centroid(store: &TopologyStore, topology: TopologyHandle) -> Point3 {
        match topology {
            TopologyHandle::Vertex(h) => Vertex::point(store, h),
            TopologyHandle::Edge(h) => Edge::center_of_mass(store, h),
            TopologyHandle::Wire(h) => Wire::center_of_mass(store, h),
            TopologyHandle::Face(h) => Face::center_of_mass(store, h),
            TopologyHandle::Shell(h) => Shell::center_of_mass(store, h),
            TopologyHandle::Cell(h) => Cell::center_of_mass(store, h),
            TopologyHandle::CellComplex(h) => CellComplex::center_of_mass(store, h),
            TopologyHandle::Cluster(h) => Cluster::center_of_mass(store, h),
        }
    }

    /// Get the bounding box of any topology
    pub fn bounding_box(store: &TopologyStore, topology: TopologyHandle) -> (Point3, Point3) {
        match topology {
            TopologyHandle::Vertex(h) => {
                let p = Vertex::point(store, h);
                (p, p)
            }
            TopologyHandle::Edge(h) => {
                let s = Edge::start_point(store, h);
                let e = Edge::end_point(store, h);
                (
                    Point3::new(s.x.min(e.x), s.y.min(e.y), s.z.min(e.z)),
                    Point3::new(s.x.max(e.x), s.y.max(e.y), s.z.max(e.z)),
                )
            }
            TopologyHandle::Wire(h) => Wire::bounding_box(store, h),
            TopologyHandle::Face(h) => Face::bounding_box(store, h),
            TopologyHandle::Shell(h) => Shell::bounding_box(store, h),
            TopologyHandle::Cell(h) => Cell::bounding_box(store, h),
            TopologyHandle::CellComplex(h) => CellComplex::bounding_box(store, h),
            TopologyHandle::Cluster(h) => Cluster::bounding_box(store, h),
        }
    }

    /// Get all vertices from any topology
    pub fn vertices(store: &TopologyStore, topology: TopologyHandle) -> Vec<VertexHandle> {
        match topology {
            TopologyHandle::Vertex(h) => vec![h],
            TopologyHandle::Edge(h) => {
                let (s, e) = Edge::vertices(store, h);
                vec![s, e]
            }
            TopologyHandle::Wire(h) => Wire::vertices(store, h),
            TopologyHandle::Face(h) => Face::vertices(store, h),
            TopologyHandle::Shell(h) => Shell::vertices(store, h),
            TopologyHandle::Cell(h) => Cell::vertices(store, h),
            TopologyHandle::CellComplex(h) => CellComplex::vertices(store, h),
            TopologyHandle::Cluster(h) => Cluster::vertices(store, h),
        }
    }

    /// Get all edges from any topology
    pub fn edges(store: &TopologyStore, topology: TopologyHandle) -> Vec<EdgeHandle> {
        match topology {
            TopologyHandle::Vertex(_) => vec![],
            TopologyHandle::Edge(h) => vec![h],
            TopologyHandle::Wire(h) => Wire::edges(store, h),
            TopologyHandle::Face(h) => Face::edges(store, h),
            TopologyHandle::Shell(h) => Shell::edges(store, h),
            TopologyHandle::Cell(h) => Cell::edges(store, h),
            TopologyHandle::CellComplex(h) => CellComplex::edges(store, h),
            TopologyHandle::Cluster(h) => Cluster::edges(store, h),
        }
    }

    /// Get all faces from any topology
    pub fn faces(store: &TopologyStore, topology: TopologyHandle) -> Vec<FaceHandle> {
        match topology {
            TopologyHandle::Vertex(_) | TopologyHandle::Edge(_) | TopologyHandle::Wire(_) => vec![],
            TopologyHandle::Face(h) => vec![h],
            TopologyHandle::Shell(h) => Shell::faces(store, h),
            TopologyHandle::Cell(h) => Cell::faces(store, h),
            TopologyHandle::CellComplex(h) => CellComplex::faces(store, h),
            TopologyHandle::Cluster(h) => Cluster::faces(store, h),
        }
    }

    /// Get all cells from any topology
    pub fn cells(store: &TopologyStore, topology: TopologyHandle) -> Vec<CellHandle> {
        match topology {
            TopologyHandle::Vertex(_) | TopologyHandle::Edge(_) | TopologyHandle::Wire(_) | TopologyHandle::Face(_) | TopologyHandle::Shell(_) => vec![],
            TopologyHandle::Cell(h) => vec![h],
            TopologyHandle::CellComplex(h) => CellComplex::cells(store, h),
            TopologyHandle::Cluster(h) => Cluster::cells(store, h),
        }
    }

    /// Get the dimensionality of a topology
    pub fn dimensionality(topology: TopologyHandle) -> u8 {
        topology.topology_type().dimensionality()
    }

    /// Copy a topology (deep copy)
    pub fn copy(store: &TopologyStore, topology: TopologyHandle) -> TopologyHandle {
        // Identity transform = copy
        let identity = MatrixUtil::identity();
        Self::transform(store, topology, identity)
    }
}
