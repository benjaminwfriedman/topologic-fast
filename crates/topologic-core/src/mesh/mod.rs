//! Mesh generation and operations
//!
//! Provides triangulation, mesh simplification, and other mesh operations.

use crate::topology::*;
use crate::geometry::{Point3, Vector3, TOLERANCE};
use rayon::prelude::*;

/// Triangle mesh representation
#[derive(Debug, Clone)]
pub struct TriangleMesh {
    pub vertices: Vec<Point3>,
    pub indices: Vec<[u32; 3]>,
    pub normals: Option<Vec<Vector3>>,
    pub uvs: Option<Vec<(f64, f64)>>,
}

impl TriangleMesh {
    /// Create a new empty mesh
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            normals: None,
            uvs: None,
        }
    }

    /// Create a mesh with capacity
    pub fn with_capacity(vertices: usize, triangles: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(vertices),
            indices: Vec::with_capacity(triangles),
            normals: None,
            uvs: None,
        }
    }

    /// Get the number of vertices
    pub fn num_vertices(&self) -> usize {
        self.vertices.len()
    }

    /// Get the number of triangles
    pub fn num_triangles(&self) -> usize {
        self.indices.len()
    }

    /// Add a vertex and return its index
    pub fn add_vertex(&mut self, point: Point3) -> u32 {
        let idx = self.vertices.len() as u32;
        self.vertices.push(point);
        idx
    }

    /// Add a triangle
    pub fn add_triangle(&mut self, a: u32, b: u32, c: u32) {
        self.indices.push([a, b, c]);
    }

    /// Compute face normals
    pub fn compute_normals(&mut self) {
        let mut normals = vec![Vector3::ZERO; self.vertices.len()];

        for tri in &self.indices {
            let a = self.vertices[tri[0] as usize];
            let b = self.vertices[tri[1] as usize];
            let c = self.vertices[tri[2] as usize];

            let normal = (b - a).cross(c - a);

            normals[tri[0] as usize] += normal;
            normals[tri[1] as usize] += normal;
            normals[tri[2] as usize] += normal;
        }

        for normal in &mut normals {
            *normal = normal.normalize_or_zero();
        }

        self.normals = Some(normals);
    }

    /// Get the bounding box
    pub fn bounding_box(&self) -> Option<(Point3, Point3)> {
        crate::geometry::bounding_box(&self.vertices)
    }

    /// Get the center of mass
    pub fn center_of_mass(&self) -> Point3 {
        crate::geometry::centroid(&self.vertices)
    }

    /// Compute the surface area
    pub fn area(&self) -> f64 {
        self.indices
            .par_iter()
            .map(|tri| {
                let a = self.vertices[tri[0] as usize];
                let b = self.vertices[tri[1] as usize];
                let c = self.vertices[tri[2] as usize];
                crate::geometry::triangle_area(a, b, c)
            })
            .sum()
    }

    /// Flip all face normals
    pub fn flip_normals(&mut self) {
        for tri in &mut self.indices {
            tri.swap(1, 2);
        }
        if let Some(ref mut normals) = self.normals {
            for normal in normals {
                *normal = -*normal;
            }
        }
    }

    /// Merge another mesh into this one
    pub fn merge(&mut self, other: &TriangleMesh) {
        let offset = self.vertices.len() as u32;

        self.vertices.extend(other.vertices.iter().copied());

        for tri in &other.indices {
            self.indices.push([tri[0] + offset, tri[1] + offset, tri[2] + offset]);
        }

        self.normals = None; // Invalidate normals
    }

    /// Translate all vertices
    pub fn translate(&mut self, offset: Vector3) {
        for vertex in &mut self.vertices {
            *vertex += offset;
        }
    }

    /// Scale all vertices
    pub fn scale(&mut self, factor: f64) {
        let center = self.center_of_mass();
        for vertex in &mut self.vertices {
            *vertex = center + (*vertex - center) * factor;
        }
    }
}

impl Default for TriangleMesh {
    fn default() -> Self {
        Self::new()
    }
}

/// Mesh generation for topology
pub struct MeshGenerator;

impl MeshGenerator {
    /// Generate a mesh from a face
    pub fn from_face(store: &TopologyStore, face: FaceHandle) -> TriangleMesh {
        let triangles = Face::triangulate(store, face);
        let mut mesh = TriangleMesh::with_capacity(triangles.len() * 3, triangles.len());

        let mut vertex_map: hashbrown::HashMap<VertexHandle, u32> = hashbrown::HashMap::new();

        for (v0, v1, v2) in triangles {
            let idx0 = *vertex_map.entry(v0).or_insert_with(|| {
                mesh.add_vertex(Vertex::point(store, v0))
            });
            let idx1 = *vertex_map.entry(v1).or_insert_with(|| {
                mesh.add_vertex(Vertex::point(store, v1))
            });
            let idx2 = *vertex_map.entry(v2).or_insert_with(|| {
                mesh.add_vertex(Vertex::point(store, v2))
            });

            mesh.add_triangle(idx0, idx1, idx2);
        }

        mesh.compute_normals();
        mesh
    }

    /// Generate a mesh from a shell
    pub fn from_shell(store: &TopologyStore, shell: ShellHandle) -> TriangleMesh {
        let faces = Shell::faces(store, shell);
        let mut mesh = TriangleMesh::new();

        for face in faces {
            let face_mesh = Self::from_face(store, face);
            mesh.merge(&face_mesh);
        }

        mesh.compute_normals();
        mesh
    }

    /// Generate a mesh from a cell
    pub fn from_cell(store: &TopologyStore, cell: CellHandle) -> TriangleMesh {
        let shell = Cell::external_boundary(store, cell);
        Self::from_shell(store, shell)
    }

    /// Generate a mesh from a cell complex
    pub fn from_cell_complex(store: &TopologyStore, complex: CellComplexHandle) -> TriangleMesh {
        let faces = CellComplex::boundary_faces(store, complex);
        let mut mesh = TriangleMesh::new();

        for face in faces {
            let face_mesh = Self::from_face(store, face);
            mesh.merge(&face_mesh);
        }

        mesh.compute_normals();
        mesh
    }
}

/// Mesh simplification using quadric error metrics
pub struct MeshSimplifier;

impl MeshSimplifier {
    /// Simplify a mesh to a target triangle count
    pub fn simplify(mesh: &TriangleMesh, target_triangles: usize) -> TriangleMesh {
        if mesh.num_triangles() <= target_triangles {
            return mesh.clone();
        }

        // Simple edge collapse implementation
        let mut result = mesh.clone();

        while result.num_triangles() > target_triangles {
            // Find shortest edge
            let mut shortest_edge: Option<(usize, usize, f64)> = None;

            for tri in &result.indices {
                for i in 0..3 {
                    let a = tri[i] as usize;
                    let b = tri[(i + 1) % 3] as usize;

                    let len = result.vertices[a].distance(result.vertices[b]);

                    match shortest_edge {
                        Some((_, _, min_len)) if len < min_len => {
                            shortest_edge = Some((a, b, len));
                        }
                        None => {
                            shortest_edge = Some((a, b, len));
                        }
                        _ => {}
                    }
                }
            }

            if let Some((a, b, _)) = shortest_edge {
                // Collapse edge: merge vertex b into a
                let new_pos = (result.vertices[a] + result.vertices[b]) * 0.5;
                result.vertices[a] = new_pos;

                // Update all references to b to point to a
                for tri in &mut result.indices {
                    for i in 0..3 {
                        if tri[i] as usize == b {
                            tri[i] = a as u32;
                        }
                    }
                }

                // Remove degenerate triangles
                result.indices.retain(|tri| {
                    tri[0] != tri[1] && tri[1] != tri[2] && tri[2] != tri[0]
                });
            } else {
                break;
            }
        }

        result.compute_normals();
        result
    }
}

/// Export mesh to various formats
pub struct MeshExporter;

impl MeshExporter {
    /// Export to OBJ format
    pub fn to_obj(mesh: &TriangleMesh) -> String {
        let mut output = String::new();

        // Vertices
        for v in &mesh.vertices {
            output.push_str(&format!("v {} {} {}\n", v.x, v.y, v.z));
        }

        // Normals (if available)
        if let Some(ref normals) = mesh.normals {
            for n in normals {
                output.push_str(&format!("vn {} {} {}\n", n.x, n.y, n.z));
            }
        }

        // Faces (1-indexed)
        for tri in &mesh.indices {
            if mesh.normals.is_some() {
                output.push_str(&format!(
                    "f {}//{} {}//{} {}//{}\n",
                    tri[0] + 1, tri[0] + 1,
                    tri[1] + 1, tri[1] + 1,
                    tri[2] + 1, tri[2] + 1
                ));
            } else {
                output.push_str(&format!(
                    "f {} {} {}\n",
                    tri[0] + 1, tri[1] + 1, tri[2] + 1
                ));
            }
        }

        output
    }

    /// Export to STL format (ASCII)
    pub fn to_stl(mesh: &TriangleMesh) -> String {
        let mut output = String::from("solid mesh\n");

        for tri in &mesh.indices {
            let a = mesh.vertices[tri[0] as usize];
            let b = mesh.vertices[tri[1] as usize];
            let c = mesh.vertices[tri[2] as usize];

            let normal = (b - a).cross(c - a).normalize_or_zero();

            output.push_str(&format!(
                "facet normal {} {} {}\n",
                normal.x, normal.y, normal.z
            ));
            output.push_str("  outer loop\n");
            output.push_str(&format!("    vertex {} {} {}\n", a.x, a.y, a.z));
            output.push_str(&format!("    vertex {} {} {}\n", b.x, b.y, b.z));
            output.push_str(&format!("    vertex {} {} {}\n", c.x, c.y, c.z));
            output.push_str("  endloop\n");
            output.push_str("endfacet\n");
        }

        output.push_str("endsolid mesh\n");
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_from_face() {
        let store = TopologyStore::new();
        let face = Face::rectangle(&store, Point3::ZERO, 1.0, 1.0);

        let mesh = MeshGenerator::from_face(&store, face);

        assert_eq!(mesh.num_vertices(), 4);
        assert_eq!(mesh.num_triangles(), 2);
    }

    #[test]
    fn test_mesh_from_cell() {
        let store = TopologyStore::new();
        let cell = Cell::box_cell(&store, Point3::ZERO, 1.0, 1.0, 1.0);

        let mesh = MeshGenerator::from_cell(&store, cell);

        // A box has 6 faces, each triangulated into 2 triangles = 12 triangles
        assert_eq!(mesh.num_triangles(), 12);
    }

    #[test]
    fn test_mesh_area() {
        let store = TopologyStore::new();
        let face = Face::rectangle(&store, Point3::ZERO, 2.0, 3.0);

        let mesh = MeshGenerator::from_face(&store, face);
        let area = mesh.area();

        assert!((area - 6.0).abs() < TOLERANCE);
    }
}
