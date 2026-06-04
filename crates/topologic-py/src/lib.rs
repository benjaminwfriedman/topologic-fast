//! Python bindings for topologic-fast
//!
//! Provides a Python API compatible with topologicpy while leveraging
//! the high-performance Rust implementation.

use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use pyo3::types::{IntoPyDict, PyModule};
use topologic_core::prelude::*;
use topologic_core::topology::{
    TopologyStore, TopologyHandle, TopologyTransform,
    VertexHandle, EdgeHandle, WireHandle, FaceHandle, ShellHandle, CellHandle, CellComplexHandle,
    Dictionary, DictionaryValue, Cluster, ClusterHandle, Wire, Vertex, Edge, Face, Shell, Cell, CellComplex,
};
use topologic_core::geometry::{Point3, Vector3};
use topologic_core::boolean::{Boolean, BooleanOp, Csg};
use topologic_core::spatial::{Aabb, BvhTree, SpatialQuery};
use topologic_core::mesh::{TriangleMesh, MeshGenerator, MeshExporter};
use topologic_core::utilities::{VectorUtil, MatrixUtil, Matrix4x4, GridUtil, GraphUtil, GraphHandle};
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;

/// Global topology store (thread-safe)
static GLOBAL_STORE: once_cell::sync::Lazy<Arc<RwLock<TopologyStore>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(TopologyStore::new())));

fn get_store() -> &'static Arc<RwLock<TopologyStore>> {
    &GLOBAL_STORE
}

/// Python wrapper for Vertex
#[pyclass(name = "Vertex")]
#[derive(Clone)]
pub struct PyVertex {
    handle: VertexHandle,
}

#[pymethods]
impl PyVertex {
    /// Create a vertex at the origin
    #[staticmethod]
    fn Origin() -> PyResult<Self> {
        let store = get_store().read();
        Ok(Self {
            handle: Vertex::origin(&store),
        })
    }

    /// Create a vertex from coordinates
    #[staticmethod]
    #[pyo3(signature = (x=0.0, y=0.0, z=0.0))]
    fn ByCoordinates(x: f64, y: f64, z: f64) -> PyResult<Self> {
        let store = get_store().read();
        Ok(Self {
            handle: Vertex::by_coordinates(&store, x, y, z),
        })
    }

    /// Get the X coordinate
    fn X(&self) -> f64 {
        let store = get_store().read();
        Vertex::x(&store, self.handle)
    }

    /// Get the Y coordinate
    fn Y(&self) -> f64 {
        let store = get_store().read();
        Vertex::y(&store, self.handle)
    }

    /// Get the Z coordinate
    fn Z(&self) -> f64 {
        let store = get_store().read();
        Vertex::z(&store, self.handle)
    }

    /// Get all coordinates
    fn Coordinates(&self) -> (f64, f64, f64) {
        let store = get_store().read();
        Vertex::coordinates(&store, self.handle)
    }

    /// Get adjacent vertices
    fn AdjacentVertices(&self) -> Vec<PyVertex> {
        let store = get_store().read();
        Vertex::adjacent_vertices(&store, self.handle)
            .into_iter()
            .map(|h| PyVertex { handle: h })
            .collect()
    }

    /// Calculate distance to another vertex
    fn Distance(&self, other: &PyVertex) -> f64 {
        let store = get_store().read();
        Vertex::distance(&store, self.handle, other.handle)
    }

    /// Get the dictionary attached to this vertex
    fn GetDictionary(&self) -> PyDictionary {
        let store = get_store().read();
        PyDictionary {
            dict: Vertex::get_dictionary(&store, self.handle),
        }
    }

    /// Set the dictionary for this vertex
    fn SetDictionary(&self, dictionary: &PyDictionary) {
        let store = get_store().read();
        Vertex::set_dictionary(&store, self.handle, dictionary.dict.clone());
    }

    /// Set a single key-value pair in the vertex's dictionary
    fn SetDictionaryValue(&self, key: &str, value: PyObject, py: Python<'_>) -> PyResult<()> {
        let store = get_store().read();
        let dv = python_to_dict_value(&value, py)?;
        Vertex::set_dictionary_value(&store, self.handle, key, dv);
        Ok(())
    }

    /// Get a value from the vertex's dictionary
    fn GetDictionaryValue(&self, key: &str, py: Python<'_>) -> Option<PyObject> {
        let store = get_store().read();
        Vertex::get_dictionary_value(&store, self.handle, key)
            .map(|v| dict_value_to_python(&v, py))
    }

    fn __repr__(&self) -> String {
        let (x, y, z) = self.Coordinates();
        format!("Vertex({:.4}, {:.4}, {:.4})", x, y, z)
    }
}

/// Python wrapper for Edge
#[pyclass(name = "Edge")]
#[derive(Clone)]
pub struct PyEdge {
    handle: EdgeHandle,
}

#[pymethods]
impl PyEdge {
    /// Create an edge from start and end vertices
    #[staticmethod]
    fn ByStartVertexEndVertex(start: &PyVertex, end: &PyVertex) -> PyResult<Self> {
        let store = get_store().read();
        Ok(Self {
            handle: Edge::by_start_vertex_end_vertex(&store, start.handle, end.handle),
        })
    }

    /// Create an edge from coordinates
    #[staticmethod]
    fn ByCoordinates(x1: f64, y1: f64, z1: f64, x2: f64, y2: f64, z2: f64) -> PyResult<Self> {
        let store = get_store().read();
        Ok(Self {
            handle: Edge::by_coordinates(&store, x1, y1, z1, x2, y2, z2),
        })
    }

    /// Create an edge by origin, direction, and length
    #[staticmethod]
    fn ByOriginDirectionLength(origin: &PyVertex, direction: Vec<f64>, length: f64) -> PyResult<Self> {
        let store = get_store().read();
        let origin_point = Vertex::point(&store, origin.handle);
        let dir = Vector3::new(
            direction.get(0).copied().unwrap_or(0.0),
            direction.get(1).copied().unwrap_or(0.0),
            direction.get(2).copied().unwrap_or(1.0),
        );
        Ok(Self {
            handle: Edge::by_origin_direction_length(&store, origin_point, dir, length),
        })
    }

    /// Get the start vertex
    fn StartVertex(&self) -> PyVertex {
        let store = get_store().read();
        PyVertex {
            handle: Edge::start_vertex(&store, self.handle),
        }
    }

    /// Get the end vertex
    fn EndVertex(&self) -> PyVertex {
        let store = get_store().read();
        PyVertex {
            handle: Edge::end_vertex(&store, self.handle),
        }
    }

    /// Get both vertices
    fn Vertices(&self) -> (PyVertex, PyVertex) {
        let store = get_store().read();
        let (start, end) = Edge::vertices(&store, self.handle);
        (PyVertex { handle: start }, PyVertex { handle: end })
    }

    /// Get the length
    fn Length(&self) -> f64 {
        let store = get_store().read();
        Edge::length(&store, self.handle)
    }

    /// Get the direction (normalized)
    fn Direction(&self) -> (f64, f64, f64) {
        let store = get_store().read();
        let d = Edge::direction(&store, self.handle);
        (d.x, d.y, d.z)
    }

    /// Get the midpoint
    fn Midpoint(&self) -> (f64, f64, f64) {
        let store = get_store().read();
        let m = Edge::midpoint(&store, self.handle);
        (m.x, m.y, m.z)
    }

    /// Get the center of mass
    fn CenterOfMass(&self) -> (f64, f64, f64) {
        let store = get_store().read();
        let c = Edge::center_of_mass(&store, self.handle);
        (c.x, c.y, c.z)
    }

    /// Get all wires containing this edge
    fn Wires(&self) -> Vec<PyWire> {
        let store = get_store().read();
        Edge::wires(&store, self.handle)
            .into_iter()
            .map(|h| PyWire { handle: h })
            .collect()
    }

    /// Get all faces containing this edge
    fn Faces(&self) -> Vec<PyFace> {
        let store = get_store().read();
        Edge::faces(&store, self.handle)
            .into_iter()
            .map(|h| PyFace { handle: h })
            .collect()
    }

    /// Get adjacent edges (sharing a vertex)
    fn AdjacentEdges(&self) -> Vec<PyEdge> {
        let store = get_store().read();
        Edge::adjacent_edges(&store, self.handle)
            .into_iter()
            .map(|h| PyEdge { handle: h })
            .collect()
    }

    /// Get shared vertices between two edges
    #[staticmethod]
    fn SharedVertices(edge1: &PyEdge, edge2: &PyEdge) -> Vec<PyVertex> {
        let store = get_store().read();
        Edge::shared_vertices(&store, edge1.handle, edge2.handle)
            .into_iter()
            .map(|h| PyVertex { handle: h })
            .collect()
    }

    /// Check if the edge is manifold
    fn IsManifold(&self) -> bool {
        let store = get_store().read();
        Edge::is_manifold(&store, self.handle)
    }

    /// Evaluate a point along the edge at parameter t ∈ [0, 1]
    fn Evaluate(&self, t: f64) -> (f64, f64, f64) {
        let store = get_store().read();
        let p = Edge::evaluate(&store, self.handle, t);
        (p.x, p.y, p.z)
    }

    /// Get the parameter of the closest point on the edge to a given point
    fn ClosestParameter(&self, point: Vec<f64>) -> f64 {
        let store = get_store().read();
        let p = Point3::new(
            point.get(0).copied().unwrap_or(0.0),
            point.get(1).copied().unwrap_or(0.0),
            point.get(2).copied().unwrap_or(0.0),
        );
        Edge::closest_parameter(&store, self.handle, p)
    }

    /// Get the closest point on the edge to a given point
    fn ClosestPoint(&self, point: Vec<f64>) -> (f64, f64, f64) {
        let store = get_store().read();
        let p = Point3::new(
            point.get(0).copied().unwrap_or(0.0),
            point.get(1).copied().unwrap_or(0.0),
            point.get(2).copied().unwrap_or(0.0),
        );
        let closest = Edge::closest_point(&store, self.handle, p);
        (closest.x, closest.y, closest.z)
    }

    /// Get the distance from a point to the edge
    fn DistanceToPoint(&self, point: Vec<f64>) -> f64 {
        let store = get_store().read();
        let p = Point3::new(
            point.get(0).copied().unwrap_or(0.0),
            point.get(1).copied().unwrap_or(0.0),
            point.get(2).copied().unwrap_or(0.0),
        );
        Edge::distance_to_point(&store, self.handle, p)
    }

    /// Reverse the edge direction
    fn Reverse(&self) -> PyEdge {
        let store = get_store().read();
        PyEdge {
            handle: Edge::reverse(&store, self.handle),
        }
    }

    /// Split the edge at parameter t
    fn SplitAt(&self, t: f64) -> (PyEdge, PyEdge) {
        let store = get_store().read();
        let (e1, e2) = Edge::split_at(&store, self.handle, t);
        (PyEdge { handle: e1 }, PyEdge { handle: e2 })
    }

    /// Get the angle between two edges in degrees
    #[staticmethod]
    fn Angle(edge1: &PyEdge, edge2: &PyEdge) -> f64 {
        let store = get_store().read();
        Edge::angle(&store, edge1.handle, edge2.handle)
    }

    /// Bisect the edge at a given parameter
    fn Bisect(&self, t: f64) -> PyVertex {
        let store = get_store().read();
        PyVertex {
            handle: Edge::bisect(&store, self.handle, t),
        }
    }

    /// Check if two edges are collinear
    #[staticmethod]
    #[pyo3(signature = (edge1, edge2, tolerance=0.0001))]
    fn IsCollinear(edge1: &PyEdge, edge2: &PyEdge, tolerance: f64) -> bool {
        let store = get_store().read();
        Edge::is_collinear(&store, edge1.handle, edge2.handle, tolerance)
    }

    /// Check if two edges are parallel
    #[staticmethod]
    #[pyo3(signature = (edge1, edge2, tolerance=0.0001))]
    fn IsParallel(edge1: &PyEdge, edge2: &PyEdge, tolerance: f64) -> bool {
        let store = get_store().read();
        Edge::is_parallel(&store, edge1.handle, edge2.handle, tolerance)
    }

    /// Check if two edges are coplanar
    #[staticmethod]
    #[pyo3(signature = (edge1, edge2, tolerance=0.0001))]
    fn IsCoplanar(edge1: &PyEdge, edge2: &PyEdge, tolerance: f64) -> bool {
        let store = get_store().read();
        Edge::is_coplanar(&store, edge1.handle, edge2.handle, tolerance)
    }

    /// Extend the edge by a distance
    #[pyo3(signature = (distance, both_sides=false))]
    fn Extend(&self, distance: f64, both_sides: bool) -> PyEdge {
        let store = get_store().read();
        PyEdge {
            handle: Edge::extend(&store, self.handle, distance, both_sides),
        }
    }

    /// Normalize the edge to unit length
    fn Normalize(&self) -> PyEdge {
        let store = get_store().read();
        PyEdge {
            handle: Edge::normalize(&store, self.handle),
        }
    }

    /// Set the length of the edge
    fn SetLength(&self, new_length: f64) -> PyEdge {
        let store = get_store().read();
        PyEdge {
            handle: Edge::set_length(&store, self.handle, new_length),
        }
    }

    /// Get parameter value at a vertex on the edge
    fn ParameterAtVertex(&self, vertex: &PyVertex) -> f64 {
        let store = get_store().read();
        Edge::parameter_at_vertex(&store, self.handle, vertex.handle)
    }

    /// Get vertex by distance along the edge
    fn VertexByDistance(&self, distance: f64) -> PyVertex {
        let store = get_store().read();
        PyVertex {
            handle: Edge::vertex_by_distance(&store, self.handle, distance),
        }
    }

    /// Get vertex by parameter t in [0, 1]
    fn VertexByParameter(&self, t: f64) -> PyVertex {
        let store = get_store().read();
        PyVertex {
            handle: Edge::vertex_by_parameter(&store, self.handle, t),
        }
    }

    /// Trim the edge to a new parameter range
    fn Trim(&self, t1: f64, t2: f64) -> PyEdge {
        let store = get_store().read();
        PyEdge {
            handle: Edge::trim(&store, self.handle, t1, t2),
        }
    }

    /// Create an offset edge in 2D (XY plane)
    fn ByOffset2D(&self, offset: f64) -> PyEdge {
        let store = get_store().read();
        PyEdge {
            handle: Edge::by_offset_2d(&store, self.handle, offset),
        }
    }

    /// Get the normal to the edge (perpendicular in XY plane)
    fn Normal(&self) -> (f64, f64, f64) {
        let store = get_store().read();
        let n = Edge::normal(&store, self.handle);
        (n.x, n.y, n.z)
    }

    /// Get the quadrance (squared length) of the edge
    fn Quadrance(&self) -> f64 {
        let store = get_store().read();
        Edge::quadrance(&store, self.handle)
    }

    /// Get the spread (squared sine of angle) between two edges
    #[staticmethod]
    fn Spread(edge1: &PyEdge, edge2: &PyEdge) -> f64 {
        let store = get_store().read();
        Edge::spread(&store, edge1.handle, edge2.handle)
    }

    /// Find intersection point of two edges in 2D
    #[staticmethod]
    #[pyo3(signature = (edge1, edge2, tolerance=0.0001))]
    fn Intersect2D(edge1: &PyEdge, edge2: &PyEdge, tolerance: f64) -> Option<(f64, f64, f64)> {
        let store = get_store().read();
        Edge::intersect_2d(&store, edge1.handle, edge2.handle, tolerance)
            .map(|p| (p.x, p.y, p.z))
    }

    /// Get the 2D line equation coefficients (ax + by + c = 0)
    fn Equation2D(&self) -> (f64, f64, f64) {
        let store = get_store().read();
        Edge::equation_2d(&store, self.handle)
    }

    /// Extend edge to intersect another edge
    #[pyo3(signature = (other, tolerance=0.0001))]
    fn ExtendToEdge(&self, other: &PyEdge, tolerance: f64) -> Option<PyEdge> {
        let store = get_store().read();
        Edge::extend_to_edge(&store, self.handle, other.handle, tolerance)
            .map(|h| PyEdge { handle: h })
    }

    /// Trim edge by another edge (at intersection point)
    #[pyo3(signature = (cutter, tolerance=0.0001))]
    fn TrimByEdge(&self, cutter: &PyEdge, tolerance: f64) -> Option<PyEdge> {
        let store = get_store().read();
        Edge::trim_by_edge(&store, self.handle, cutter.handle, tolerance)
            .map(|h| PyEdge { handle: h })
    }

    /// Create an edge along the normal of a face
    #[staticmethod]
    fn ByFaceNormal(face: &PyFace, length: f64) -> Option<PyEdge> {
        let store = get_store().read();
        Edge::by_face_normal(&store, face.handle, length)
            .map(|h| PyEdge { handle: h })
    }

    /// Align the edge to 2D (project to XY plane)
    fn Align2D(&self) -> PyEdge {
        let store = get_store().read();
        PyEdge {
            handle: Edge::align_2d(&store, self.handle),
        }
    }

    /// Get the dictionary attached to this edge
    fn GetDictionary(&self) -> PyDictionary {
        let store = get_store().read();
        PyDictionary {
            dict: Edge::get_dictionary(&store, self.handle),
        }
    }

    /// Set the dictionary for this edge
    fn SetDictionary(&self, dictionary: &PyDictionary) {
        let store = get_store().read();
        Edge::set_dictionary(&store, self.handle, dictionary.dict.clone());
    }

    /// Set a single key-value pair in the edge's dictionary
    fn SetDictionaryValue(&self, key: &str, value: PyObject, py: Python<'_>) -> PyResult<()> {
        let store = get_store().read();
        let dv = python_to_dict_value(&value, py)?;
        Edge::set_dictionary_value(&store, self.handle, key, dv);
        Ok(())
    }

    /// Get a value from the edge's dictionary
    fn GetDictionaryValue(&self, key: &str, py: Python<'_>) -> Option<PyObject> {
        let store = get_store().read();
        Edge::get_dictionary_value(&store, self.handle, key)
            .map(|v| dict_value_to_python(&v, py))
    }

    fn __repr__(&self) -> String {
        format!("Edge(length={:.4})", self.Length())
    }
}

/// Python wrapper for Wire
#[pyclass(name = "Wire")]
#[derive(Clone)]
pub struct PyWire {
    handle: WireHandle,
}

#[pymethods]
impl PyWire {
    /// Create a wire from edges
    #[staticmethod]
    fn ByEdges(edges: Vec<PyEdge>) -> PyResult<Self> {
        let store = get_store().read();
        let handles: Vec<_> = edges.iter().map(|e| e.handle).collect();
        Ok(Self {
            handle: Wire::by_edges(&store, handles),
        })
    }

    /// Create a wire from vertices
    #[staticmethod]
    #[pyo3(signature = (vertices, close=true))]
    fn ByVertices(vertices: Vec<PyVertex>, close: bool) -> PyResult<Self> {
        let store = get_store().read();
        let handles: Vec<_> = vertices.iter().map(|v| v.handle).collect();
        Ok(Self {
            handle: Wire::by_vertices(&store, handles, close),
        })
    }

    /// Create a rectangular wire
    #[staticmethod]
    #[pyo3(signature = (origin=None, width=1.0, length=1.0, direction=None, placement="center", tolerance=0.0001))]
    fn Rectangle(
        origin: Option<&PyVertex>,
        width: f64,
        length: f64,
        direction: Option<Vec<f64>>,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        // Adjust for placement
        let adjusted = match placement {
            "center" => origin_point - Vector3::new(width / 2.0, length / 2.0, 0.0),
            "lowerleft" => origin_point,
            _ => origin_point - Vector3::new(width / 2.0, length / 2.0, 0.0),
        };
        Ok(Self {
            handle: Wire::rectangle(&store, adjusted, width, length),
        })
    }

    /// Create a circular wire
    #[staticmethod]
    #[pyo3(signature = (origin=None, radius=0.5, sides=16, from_angle=0.0, to_angle=360.0, close=true, direction=None, placement="center", tolerance=0.0001))]
    fn Circle(
        origin: Option<&PyVertex>,
        radius: f64,
        sides: usize,
        from_angle: f64,
        to_angle: f64,
        close: bool,
        direction: Option<Vec<f64>>,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let center = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        Ok(Self {
            handle: Wire::circle(&store, center, radius, sides),
        })
    }

    /// Create an ellipse wire
    #[staticmethod]
    #[pyo3(signature = (origin=None, width=2.0, length=1.0, sides=32, from_angle=0.0, to_angle=360.0, close=true, direction=None, placement="center", tolerance=0.0001))]
    fn Ellipse(
        origin: Option<&PyVertex>,
        width: f64,
        length: f64,
        sides: usize,
        from_angle: f64,
        to_angle: f64,
        close: bool,
        direction: Option<Vec<f64>>,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let center = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        Ok(Self {
            handle: Wire::ellipse(&store, center, width, length, sides, from_angle, to_angle, close),
        })
    }

    /// Create a star wire
    #[staticmethod]
    #[pyo3(signature = (origin=None, radius_a=0.5, radius_b=0.2, rays=8, direction=None, placement="center", tolerance=0.0001))]
    fn Star(
        origin: Option<&PyVertex>,
        radius_a: f64,
        radius_b: f64,
        rays: usize,
        direction: Option<Vec<f64>>,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let center = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        Ok(Self {
            handle: Wire::star(&store, center, radius_a, radius_b, rays),
        })
    }

    /// Create a trapezoid wire
    #[staticmethod]
    #[pyo3(signature = (origin=None, width_a=1.0, width_b=0.75, offset_a=0.0, offset_b=0.0, length=1.0, direction=None, placement="center", tolerance=0.0001))]
    fn Trapezoid(
        origin: Option<&PyVertex>,
        width_a: f64,
        width_b: f64,
        offset_a: f64,
        offset_b: f64,
        length: f64,
        direction: Option<Vec<f64>>,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        Ok(Self {
            handle: Wire::trapezoid(&store, origin_point, width_a, width_b, offset_a, offset_b, length),
        })
    }

    /// Create a square wire
    #[staticmethod]
    #[pyo3(signature = (origin=None, size=1.0, direction=None, placement="center", tolerance=0.0001))]
    fn Square(
        origin: Option<&PyVertex>,
        size: f64,
        direction: Option<Vec<f64>>,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        let adjusted = match placement {
            "center" => origin_point - Vector3::new(size / 2.0, size / 2.0, 0.0),
            _ => origin_point,
        };
        Ok(Self {
            handle: Wire::square(&store, adjusted, size, Vector3::Z),
        })
    }

    /// Create a line wire
    #[staticmethod]
    #[pyo3(signature = (origin=None, length=1.0, direction=None, sides=2, placement="center", tolerance=0.0001))]
    fn Line(
        origin: Option<&PyVertex>,
        length: f64,
        direction: Option<Vec<f64>>,
        sides: usize,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        let dir = match direction {
            Some(d) => Vector3::new(
                d.get(0).copied().unwrap_or(1.0),
                d.get(1).copied().unwrap_or(0.0),
                d.get(2).copied().unwrap_or(0.0),
            ),
            None => Vector3::X,
        };
        Ok(Self {
            handle: Wire::line(&store, origin_point, length, dir),
        })
    }

    /// Create a spiral wire
    #[staticmethod]
    #[pyo3(signature = (origin=None, radius_a=0.05, radius_b=0.5, height=1.0, turns=10, sides=36, clockwise=false, direction=None, placement="center", tolerance=0.0001))]
    fn Spiral(
        origin: Option<&PyVertex>,
        radius_a: f64,
        radius_b: f64,
        height: f64,
        turns: usize,
        sides: usize,
        clockwise: bool,
        direction: Option<Vec<f64>>,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let center = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        Ok(Self {
            handle: Wire::spiral(&store, center, radius_a, radius_b, height, turns, sides, clockwise),
        })
    }

    /// Create an L-shaped wire
    #[staticmethod]
    #[pyo3(signature = (origin=None, width=1.0, length=1.0, a=0.25, b=0.25, flip_horizontal=false, flip_vertical=false, direction=None, placement="center", tolerance=0.0001))]
    fn LShape(
        origin: Option<&PyVertex>,
        width: f64,
        length: f64,
        a: f64,
        b: f64,
        flip_horizontal: bool,
        flip_vertical: bool,
        direction: Option<Vec<f64>>,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        Ok(Self {
            handle: Wire::l_shape(&store, origin_point, width, length, a, b),
        })
    }

    /// Create a T-shaped wire
    #[staticmethod]
    #[pyo3(signature = (origin=None, width=1.0, length=1.0, a=0.25, b=0.25, flip_horizontal=false, flip_vertical=false, direction=None, placement="center", tolerance=0.0001))]
    fn TShape(
        origin: Option<&PyVertex>,
        width: f64,
        length: f64,
        a: f64,
        b: f64,
        flip_horizontal: bool,
        flip_vertical: bool,
        direction: Option<Vec<f64>>,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        Ok(Self {
            handle: Wire::t_shape(&store, origin_point, width, length, a, b),
        })
    }

    /// Create a C-shaped wire
    #[staticmethod]
    #[pyo3(signature = (origin=None, width=1.0, length=1.0, a=0.25, b=0.25, c=0.25, flip_horizontal=false, flip_vertical=false, direction=None, placement="center", tolerance=0.0001))]
    fn CShape(
        origin: Option<&PyVertex>,
        width: f64,
        length: f64,
        a: f64,
        b: f64,
        c: f64,
        flip_horizontal: bool,
        flip_vertical: bool,
        direction: Option<Vec<f64>>,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        Ok(Self {
            handle: Wire::c_shape(&store, origin_point, width, length, a, b, c),
        })
    }

    /// Create an I-shaped wire
    #[staticmethod]
    #[pyo3(signature = (origin=None, width=1.0, length=1.0, a=0.25, b=0.25, c=0.25, flip_horizontal=false, flip_vertical=false, direction=None, placement="center", tolerance=0.0001))]
    fn IShape(
        origin: Option<&PyVertex>,
        width: f64,
        length: f64,
        a: f64,
        b: f64,
        c: f64,
        flip_horizontal: bool,
        flip_vertical: bool,
        direction: Option<Vec<f64>>,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        Ok(Self {
            handle: Wire::i_shape(&store, origin_point, width, length, a, b, c),
        })
    }

    /// Create a cross-shaped wire
    #[staticmethod]
    #[pyo3(signature = (origin=None, width=1.0, length=1.0, a=0.25, b=0.25, flip_horizontal=false, flip_vertical=false, direction=None, placement="center", tolerance=0.0001))]
    fn CrossShape(
        origin: Option<&PyVertex>,
        width: f64,
        length: f64,
        a: f64,
        b: f64,
        flip_horizontal: bool,
        flip_vertical: bool,
        direction: Option<Vec<f64>>,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        Ok(Self {
            handle: Wire::cross_shape(&store, origin_point, width, length, a, b),
        })
    }

    /// Create a squircle (super-ellipse) wire
    #[staticmethod]
    #[pyo3(signature = (origin=None, radius=0.5, sides=121, a=2.0, b=2.0, direction=None, placement="center", tolerance=0.0001))]
    fn Squircle(
        origin: Option<&PyVertex>,
        radius: f64,
        sides: usize,
        a: f64,
        b: f64,
        direction: Option<Vec<f64>>,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let center = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        // Use average of a and b as the exponent
        let exponent = (a + b) / 2.0;
        Ok(Self {
            handle: Wire::squircle(&store, center, radius, sides, exponent),
        })
    }

    /// Check if the wire is closed
    fn IsClosed(&self) -> bool {
        let store = get_store().read();
        Wire::is_closed(&store, self.handle)
    }

    /// Check if the wire is manifold
    fn IsManifold(&self) -> bool {
        let store = get_store().read();
        Wire::is_manifold(&store, self.handle)
    }

    /// Check if the wire is planar
    fn IsPlanar(&self) -> bool {
        let store = get_store().read();
        Wire::is_planar(&store, self.handle)
    }

    /// Check if two wires are similar
    #[staticmethod]
    #[pyo3(signature = (wire_a, wire_b, ang_tolerance=0.1, tolerance=0.0001))]
    fn IsSimilar(wire_a: &PyWire, wire_b: &PyWire, ang_tolerance: f64, tolerance: f64) -> bool {
        let store = get_store().read();
        Wire::is_similar(&store, wire_a.handle, wire_b.handle, ang_tolerance)
    }

    /// Get the length
    fn Length(&self) -> f64 {
        let store = get_store().read();
        Wire::length(&store, self.handle)
    }

    /// Get the area (if closed)
    fn Area(&self) -> f64 {
        let store = get_store().read();
        Wire::area(&store, self.handle)
    }

    /// Get the center of mass
    fn CenterOfMass(&self) -> (f64, f64, f64) {
        let store = get_store().read();
        let c = Wire::center_of_mass(&store, self.handle);
        (c.x, c.y, c.z)
    }

    /// Get the normal (if planar)
    fn Normal(&self) -> Option<(f64, f64, f64)> {
        let store = get_store().read();
        Wire::normal(&store, self.handle).map(|n| (n.x, n.y, n.z))
    }

    /// Get the bounding box
    fn BoundingBox(&self) -> ((f64, f64, f64), (f64, f64, f64)) {
        let store = get_store().read();
        let (min, max) = Wire::bounding_box(&store, self.handle);
        ((min.x, min.y, min.z), (max.x, max.y, max.z))
    }

    /// Get all edges
    fn Edges(&self) -> Vec<PyEdge> {
        let store = get_store().read();
        Wire::edges(&store, self.handle)
            .into_iter()
            .map(|h| PyEdge { handle: h })
            .collect()
    }

    /// Get all vertices
    fn Vertices(&self) -> Vec<PyVertex> {
        let store = get_store().read();
        Wire::vertices(&store, self.handle)
            .into_iter()
            .map(|h| PyVertex { handle: h })
            .collect()
    }

    /// Debug: get edge orientations
    fn DebugOrientations(&self) -> Vec<bool> {
        let store = get_store().read();
        Wire::edge_orientations(&store, self.handle)
    }

    /// Get faces using this wire
    fn Faces(&self) -> Vec<PyFace> {
        let store = get_store().read();
        Wire::faces(&store, self.handle)
            .into_iter()
            .map(|h| PyFace { handle: h })
            .collect()
    }

    /// Get the start vertex
    fn StartVertex(&self) -> Option<PyVertex> {
        let store = get_store().read();
        Wire::start_vertex(&store, self.handle).map(|h| PyVertex { handle: h })
    }

    /// Get the end vertex
    fn EndVertex(&self) -> Option<PyVertex> {
        let store = get_store().read();
        Wire::end_vertex(&store, self.handle).map(|h| PyVertex { handle: h })
    }

    /// Reverse the wire direction
    fn Reverse(&self) -> PyWire {
        let store = get_store().read();
        PyWire {
            handle: Wire::reverse(&store, self.handle),
        }
    }

    /// Invert the wire (same as reverse)
    fn Invert(&self) -> PyWire {
        let store = get_store().read();
        PyWire {
            handle: Wire::invert(&store, self.handle),
        }
    }

    /// Close the wire if not already closed
    fn Close(&self) -> PyWire {
        let store = get_store().read();
        PyWire {
            handle: Wire::close(&store, self.handle),
        }
    }

    /// Get vertex by parameter t in [0, 1]
    fn VertexByParameter(&self, u: f64) -> PyVertex {
        let store = get_store().read();
        PyVertex {
            handle: Wire::vertex_by_parameter(&store, self.handle, u),
        }
    }

    /// Get vertex by distance along the wire
    #[pyo3(signature = (distance=0.0, origin=None, tolerance=0.0001))]
    fn VertexByDistance(&self, distance: f64, origin: Option<&PyVertex>, tolerance: f64) -> PyVertex {
        let store = get_store().read();
        PyVertex {
            handle: Wire::vertex_by_distance(&store, self.handle, distance),
        }
    }

    /// Get parameter at a vertex
    #[pyo3(signature = (vertex, tolerance=0.0001))]
    fn ParameterAtVertex(&self, vertex: &PyVertex, tolerance: f64) -> f64 {
        let store = get_store().read();
        Wire::parameter_at_vertex(&store, self.handle, vertex.handle, tolerance)
    }

    /// Get interior angles at each vertex
    #[pyo3(signature = (tolerance=0.0001, mantissa=6))]
    fn InteriorAngles(&self, tolerance: f64, mantissa: u32) -> Vec<f64> {
        let store = get_store().read();
        Wire::interior_angles(&store, self.handle)
    }

    /// Get exterior angles at each vertex
    #[pyo3(signature = (tolerance=0.0001, mantissa=6))]
    fn ExteriorAngles(&self, tolerance: f64, mantissa: u32) -> Vec<f64> {
        let store = get_store().read();
        Wire::exterior_angles(&store, self.handle)
    }

    /// Split wire into individual edges
    fn Split(&self) -> Vec<PyWire> {
        let store = get_store().read();
        Wire::split(&store, self.handle)
            .into_iter()
            .map(|h| PyWire { handle: h })
            .collect()
    }

    /// Project wire onto a face
    #[pyo3(signature = (face, direction=None, tolerance=0.0001))]
    fn Project(&self, face: &PyFace, direction: Option<Vec<f64>>, tolerance: f64) -> PyWire {
        let store = get_store().read();
        let dir = match direction {
            Some(d) => Vector3::new(
                d.get(0).copied().unwrap_or(0.0),
                d.get(1).copied().unwrap_or(0.0),
                d.get(2).copied().unwrap_or(1.0),
            ),
            None => Vector3::ZERO,
        };
        PyWire {
            handle: Wire::project(&store, self.handle, face.handle, dir),
        }
    }

    /// Create an offset wire (2D offset)
    #[staticmethod]
    #[pyo3(signature = (wire, offset, miter_threshold=2.0))]
    fn ByOffset(wire: &PyWire, offset: f64, miter_threshold: f64) -> PyWire {
        let store = get_store().read();
        PyWire {
            handle: Wire::by_offset(&store, wire.handle, offset, miter_threshold),
        }
    }

    /// Create a fillet (rounded corners) on the wire
    #[pyo3(signature = (radius, segments=8))]
    fn Fillet(&self, radius: f64, segments: usize) -> PyWire {
        let store = get_store().read();
        PyWire {
            handle: Wire::fillet(&store, self.handle, radius, segments),
        }
    }

    /// Offset this wire (instance method)
    #[pyo3(signature = (offset, miter_threshold=2.0))]
    fn Offset(&self, offset: f64, miter_threshold: f64) -> PyWire {
        let store = get_store().read();
        PyWire {
            handle: Wire::by_offset(&store, self.handle, offset, miter_threshold),
        }
    }

    /// Get the dictionary attached to this wire
    fn GetDictionary(&self) -> PyDictionary {
        let store = get_store().read();
        PyDictionary {
            dict: Wire::get_dictionary(&store, self.handle),
        }
    }

    /// Set the dictionary for this wire
    fn SetDictionary(&self, dictionary: &PyDictionary) {
        let store = get_store().read();
        Wire::set_dictionary(&store, self.handle, dictionary.dict.clone());
    }

    /// Set a single key-value pair in the wire's dictionary
    fn SetDictionaryValue(&self, key: &str, value: PyObject, py: Python<'_>) -> PyResult<()> {
        let store = get_store().read();
        let dv = python_to_dict_value(&value, py)?;
        Wire::set_dictionary_value(&store, self.handle, key, dv);
        Ok(())
    }

    /// Get a value from the wire's dictionary
    fn GetDictionaryValue(&self, key: &str, py: Python<'_>) -> Option<PyObject> {
        let store = get_store().read();
        Wire::get_dictionary_value(&store, self.handle, key)
            .map(|v| dict_value_to_python(&v, py))
    }

    fn __repr__(&self) -> String {
        format!("Wire(closed={}, length={:.4})", self.IsClosed(), self.Length())
    }
}

/// Python wrapper for Face
#[pyclass(name = "Face")]
#[derive(Clone)]
pub struct PyFace {
    handle: FaceHandle,
}

#[pymethods]
impl PyFace {
    /// Create a face from a wire
    #[staticmethod]
    fn ByExternalBoundary(wire: &PyWire) -> PyResult<Self> {
        let store = get_store().read();
        Ok(Self {
            handle: Face::by_external_boundary(&store, wire.handle),
        })
    }

    /// Create a face from a wire (alias)
    #[staticmethod]
    fn ByWire(wire: &PyWire) -> PyResult<Self> {
        let store = get_store().read();
        Ok(Self {
            handle: Face::by_wire(&store, wire.handle),
        })
    }

    /// Create a face from multiple wires
    #[staticmethod]
    fn ByWires(wires: Vec<PyWire>) -> PyResult<Self> {
        let store = get_store().read();
        let handles: Vec<_> = wires.iter().map(|w| w.handle).collect();
        match Face::by_wires(&store, handles) {
            Some(h) => Ok(Self { handle: h }),
            None => Err(PyValueError::new_err("At least one wire required")),
        }
    }

    /// Create a face from vertices
    #[staticmethod]
    fn ByVertices(vertices: Vec<PyVertex>) -> PyResult<Self> {
        let store = get_store().read();
        let handles: Vec<_> = vertices.iter().map(|v| v.handle).collect();
        Ok(Self {
            handle: Face::by_vertices(&store, handles),
        })
    }

    /// Create a face with holes
    #[staticmethod]
    fn ByExternalInternalBoundaries(outer: &PyWire, inner: Vec<PyWire>) -> PyResult<Self> {
        let store = get_store().read();
        let inner_handles: Vec<_> = inner.iter().map(|w| w.handle).collect();
        Ok(Self {
            handle: Face::by_external_internal_boundaries(&store, outer.handle, inner_handles),
        })
    }

    /// Create a rectangular face
    /// Can be called as Face.Rectangle(x, y, z, width, length) or Face.Rectangle(origin=vertex, width=w, length=l)
    #[staticmethod]
    #[pyo3(signature = (x=0.0, y=0.0, z=0.0, width=1.0, length=1.0, origin=None, direction=None, placement="center", tolerance=0.0001))]
    fn Rectangle(
        x: f64,
        y: f64,
        z: f64,
        width: f64,
        length: f64,
        origin: Option<&PyVertex>,
        direction: Option<Vec<f64>>,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::new(x, y, z),
        };
        let adjusted = match placement {
            "center" => origin_point - Vector3::new(width / 2.0, length / 2.0, 0.0),
            "lowerleft" => origin_point,
            _ => origin_point,
        };
        Ok(Self {
            handle: Face::rectangle(&store, adjusted, width, length),
        })
    }

    /// Create a square face
    #[staticmethod]
    #[pyo3(signature = (origin=None, size=1.0, direction=None, placement="center", tolerance=0.0001))]
    fn Square(
        origin: Option<&PyVertex>,
        size: f64,
        direction: Option<Vec<f64>>,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        let adjusted = match placement {
            "center" => origin_point - Vector3::new(size / 2.0, size / 2.0, 0.0),
            _ => origin_point,
        };
        Ok(Self {
            handle: Face::square(&store, adjusted, size),
        })
    }

    /// Create a circular face
    #[staticmethod]
    #[pyo3(signature = (origin=None, radius=0.5, sides=16, direction=None, placement="center", tolerance=0.0001))]
    fn Circle(
        origin: Option<&PyVertex>,
        radius: f64,
        sides: usize,
        direction: Option<Vec<f64>>,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let center = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        Ok(Self {
            handle: Face::circle(&store, center, radius, sides),
        })
    }

    /// Create an ellipse face
    #[staticmethod]
    #[pyo3(signature = (origin=None, width=2.0, length=1.0, sides=32, direction=None, placement="center", tolerance=0.0001))]
    fn Ellipse(
        origin: Option<&PyVertex>,
        width: f64,
        length: f64,
        sides: usize,
        direction: Option<Vec<f64>>,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let center = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        Ok(Self {
            handle: Face::ellipse(&store, center, width, length, sides),
        })
    }

    /// Create a star face
    #[staticmethod]
    #[pyo3(signature = (origin=None, radius_a=0.5, radius_b=0.2, rays=8, direction=None, placement="center", tolerance=0.0001))]
    fn Star(
        origin: Option<&PyVertex>,
        radius_a: f64,
        radius_b: f64,
        rays: usize,
        direction: Option<Vec<f64>>,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let center = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        Ok(Self {
            handle: Face::star(&store, center, radius_a, radius_b, rays),
        })
    }

    /// Create a trapezoid face
    #[staticmethod]
    #[pyo3(signature = (origin=None, width_a=1.0, width_b=0.75, offset_a=0.0, offset_b=0.0, length=1.0, direction=None, placement="center", tolerance=0.0001))]
    fn Trapezoid(
        origin: Option<&PyVertex>,
        width_a: f64,
        width_b: f64,
        offset_a: f64,
        offset_b: f64,
        length: f64,
        direction: Option<Vec<f64>>,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        Ok(Self {
            handle: Face::trapezoid(&store, origin_point, width_a, width_b, offset_a, offset_b, length),
        })
    }

    /// Create an L-shaped face
    #[staticmethod]
    #[pyo3(signature = (origin=None, width=1.0, length=1.0, a=0.25, b=0.25, direction=None, placement="center", tolerance=0.0001))]
    fn LShape(
        origin: Option<&PyVertex>,
        width: f64,
        length: f64,
        a: f64,
        b: f64,
        direction: Option<Vec<f64>>,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        Ok(Self {
            handle: Face::l_shape(&store, origin_point, width, length, a, b),
        })
    }

    /// Create a T-shaped face
    #[staticmethod]
    #[pyo3(signature = (origin=None, width=1.0, length=1.0, a=0.25, b=0.25, direction=None, placement="center", tolerance=0.0001))]
    fn TShape(
        origin: Option<&PyVertex>,
        width: f64,
        length: f64,
        a: f64,
        b: f64,
        direction: Option<Vec<f64>>,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        Ok(Self {
            handle: Face::t_shape(&store, origin_point, width, length, a, b),
        })
    }

    /// Create a C-shaped face
    #[staticmethod]
    #[pyo3(signature = (origin=None, width=1.0, length=1.0, a=0.25, b=0.25, c=0.25, direction=None, placement="center", tolerance=0.0001))]
    fn CShape(
        origin: Option<&PyVertex>,
        width: f64,
        length: f64,
        a: f64,
        b: f64,
        c: f64,
        direction: Option<Vec<f64>>,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        Ok(Self {
            handle: Face::c_shape(&store, origin_point, width, length, a, b, c),
        })
    }

    /// Create an I-shaped face
    #[staticmethod]
    #[pyo3(signature = (origin=None, width=1.0, length=1.0, a=0.25, b=0.25, c=0.25, direction=None, placement="center", tolerance=0.0001))]
    fn IShape(
        origin: Option<&PyVertex>,
        width: f64,
        length: f64,
        a: f64,
        b: f64,
        c: f64,
        direction: Option<Vec<f64>>,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        Ok(Self {
            handle: Face::i_shape(&store, origin_point, width, length, a, b, c),
        })
    }

    /// Create a cross-shaped face
    #[staticmethod]
    #[pyo3(signature = (origin=None, width=1.0, length=1.0, a=0.25, b=0.25, direction=None, placement="center", tolerance=0.0001))]
    fn CrossShape(
        origin: Option<&PyVertex>,
        width: f64,
        length: f64,
        a: f64,
        b: f64,
        direction: Option<Vec<f64>>,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        Ok(Self {
            handle: Face::cross_shape(&store, origin_point, width, length, a, b),
        })
    }

    /// Create a squircle face
    #[staticmethod]
    #[pyo3(signature = (origin=None, radius=0.5, sides=121, a=2.0, b=2.0, direction=None, placement="center", tolerance=0.0001))]
    fn Squircle(
        origin: Option<&PyVertex>,
        radius: f64,
        sides: usize,
        a: f64,
        b: f64,
        direction: Option<Vec<f64>>,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let center = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        let exponent = (a + b) / 2.0;
        Ok(Self {
            handle: Face::squircle(&store, center, radius, sides, exponent),
        })
    }

    /// Create a ring face (circle with a hole)
    #[staticmethod]
    #[pyo3(signature = (origin=None, inner_radius=0.25, outer_radius=0.5, sides=32, direction=None, placement="center", tolerance=0.0001))]
    fn Ring(
        origin: Option<&PyVertex>,
        inner_radius: f64,
        outer_radius: f64,
        sides: usize,
        direction: Option<Vec<f64>>,
        placement: &str,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let center = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        Ok(Self {
            handle: Face::ring(&store, center, inner_radius, outer_radius, sides),
        })
    }

    /// Get the area
    fn Area(&self) -> f64 {
        let store = get_store().read();
        Face::area(&store, self.handle)
    }

    /// Get the perimeter
    fn Perimeter(&self) -> f64 {
        let store = get_store().read();
        Face::perimeter(&store, self.handle)
    }

    /// Get the compactness
    fn Compactness(&self) -> f64 {
        let store = get_store().read();
        Face::compactness(&store, self.handle)
    }

    /// Get the normal
    fn Normal(&self) -> Option<(f64, f64, f64)> {
        let store = get_store().read();
        Face::normal(&store, self.handle).map(|n| (n.x, n.y, n.z))
    }

    /// Get the center of mass
    fn CenterOfMass(&self) -> (f64, f64, f64) {
        let store = get_store().read();
        let c = Face::center_of_mass(&store, self.handle);
        (c.x, c.y, c.z)
    }

    /// Get the bounding box
    fn BoundingBox(&self) -> ((f64, f64, f64), (f64, f64, f64)) {
        let store = get_store().read();
        let (min, max) = Face::bounding_box(&store, self.handle);
        ((min.x, min.y, min.z), (max.x, max.y, max.z))
    }

    /// Get vertices
    fn Vertices(&self) -> Vec<PyVertex> {
        let store = get_store().read();
        Face::vertices(&store, self.handle)
            .into_iter()
            .map(|h| PyVertex { handle: h })
            .collect()
    }

    /// Get edges
    fn Edges(&self) -> Vec<PyEdge> {
        let store = get_store().read();
        Face::edges(&store, self.handle)
            .into_iter()
            .map(|h| PyEdge { handle: h })
            .collect()
    }

    /// Get all wires
    fn Wires(&self) -> Vec<PyWire> {
        let store = get_store().read();
        Face::wires(&store, self.handle)
            .into_iter()
            .map(|h| PyWire { handle: h })
            .collect()
    }

    /// Get the external boundary wire
    fn ExternalBoundary(&self) -> PyWire {
        let store = get_store().read();
        PyWire {
            handle: Face::external_boundary(&store, self.handle),
        }
    }

    /// Get internal boundary wires (holes)
    fn InternalBoundaries(&self) -> Vec<PyWire> {
        let store = get_store().read();
        Face::internal_boundaries(&store, self.handle)
            .into_iter()
            .map(|h| PyWire { handle: h })
            .collect()
    }

    /// Get adjacent faces
    fn AdjacentFaces(&self) -> Vec<PyFace> {
        let store = get_store().read();
        Face::adjacent_faces(&store, self.handle)
            .into_iter()
            .map(|h| PyFace { handle: h })
            .collect()
    }

    /// Get shared edges with another face
    #[staticmethod]
    fn SharedEdges(face1: &PyFace, face2: &PyFace) -> Vec<PyEdge> {
        let store = get_store().read();
        Face::shared_edges(&store, face1.handle, face2.handle)
            .into_iter()
            .map(|h| PyEdge { handle: h })
            .collect()
    }

    /// Get shells containing this face
    fn Shells(&self) -> Vec<PyShell> {
        let store = get_store().read();
        Face::shells(&store, self.handle)
            .into_iter()
            .map(|h| PyShell { handle: h })
            .collect()
    }

    /// Get cells containing this face
    fn Cells(&self) -> Vec<PyCell> {
        let store = get_store().read();
        Face::cells(&store, self.handle)
            .into_iter()
            .map(|h| PyCell { handle: h })
            .collect()
    }

    /// Check if the face is planar
    fn IsPlanar(&self) -> bool {
        let store = get_store().read();
        Face::is_planar(&store, self.handle)
    }

    /// Check if the face is convex
    fn IsConvex(&self) -> bool {
        let store = get_store().read();
        Face::is_convex(&store, self.handle)
    }

    /// Check if the face is manifold
    fn IsManifold(&self) -> bool {
        let store = get_store().read();
        Face::is_manifold(&store, self.handle)
    }

    /// Check if two faces are coplanar
    #[staticmethod]
    #[pyo3(signature = (face1, face2, tolerance=0.0001))]
    fn IsCoplanar(face1: &PyFace, face2: &PyFace, tolerance: f64) -> bool {
        let store = get_store().read();
        Face::is_coplanar(&store, face1.handle, face2.handle, tolerance)
    }

    /// Check if a point is contained in the face
    fn ContainsPoint(&self, x: f64, y: f64, z: f64) -> bool {
        let store = get_store().read();
        Face::contains_point(&store, self.handle, Point3::new(x, y, z))
    }

    /// Triangulate the face
    fn Triangulate(&self) -> Vec<(PyVertex, PyVertex, PyVertex)> {
        let store = get_store().read();
        Face::triangulate(&store, self.handle)
            .into_iter()
            .map(|(a, b, c)| (
                PyVertex { handle: a },
                PyVertex { handle: b },
                PyVertex { handle: c },
            ))
            .collect()
    }

    /// Flip/Invert the face normal
    fn Invert(&self) -> PyFace {
        let store = get_store().read();
        PyFace {
            handle: Face::invert(&store, self.handle),
        }
    }

    /// Get the angle between two faces
    #[staticmethod]
    fn Angle(face1: &PyFace, face2: &PyFace) -> Option<f64> {
        let store = get_store().read();
        Face::angle(&store, face1.handle, face2.handle)
    }

    /// Get the compass angle of the face normal
    fn CompassAngle(&self) -> Option<f64> {
        let store = get_store().read();
        Face::compass_angle(&store, self.handle)
    }

    /// Check if the face is facing toward a point
    fn FacingToward(&self, x: f64, y: f64, z: f64) -> bool {
        let store = get_store().read();
        Face::facing_toward(&store, self.handle, Point3::new(x, y, z))
    }

    /// Get the plane equation coefficients
    fn PlaneEquation(&self) -> Option<(f64, f64, f64, f64)> {
        let store = get_store().read();
        Face::plane_equation(&store, self.handle)
    }

    /// Get vertex by UV parameters
    fn VertexByParameters(&self, u: f64, v: f64) -> PyVertex {
        let store = get_store().read();
        PyVertex {
            handle: Face::vertex_by_parameters(&store, self.handle, u, v),
        }
    }

    /// Get an internal vertex (center)
    fn InternalVertex(&self) -> PyVertex {
        let store = get_store().read();
        PyVertex {
            handle: Face::internal_vertex(&store, self.handle),
        }
    }

    /// Create an edge along the face normal
    fn NormalEdge(&self, length: f64) -> Option<PyEdge> {
        let store = get_store().read();
        Face::normal_edge(&store, self.handle, length).map(|h| PyEdge { handle: h })
    }

    /// Add internal boundaries (holes)
    fn AddInternalBoundaries(&self, internal: Vec<PyWire>) -> PyFace {
        let store = get_store().read();
        let handles: Vec<_> = internal.iter().map(|w| w.handle).collect();
        PyFace {
            handle: Face::add_internal_boundaries(&store, self.handle, handles),
        }
    }

    /// Get interior angles
    #[pyo3(signature = (tolerance=0.0001, mantissa=6))]
    fn InteriorAngles(&self, tolerance: f64, mantissa: u32) -> Vec<f64> {
        let store = get_store().read();
        Face::interior_angles(&store, self.handle)
    }

    /// Get exterior angles
    #[pyo3(signature = (tolerance=0.0001, mantissa=6))]
    fn ExteriorAngles(&self, tolerance: f64, mantissa: u32) -> Vec<f64> {
        let store = get_store().read();
        Face::exterior_angles(&store, self.handle)
    }

    /// Project face onto another face
    #[pyo3(signature = (target, direction=None, tolerance=0.0001))]
    fn Project(&self, target: &PyFace, direction: Option<Vec<f64>>, tolerance: f64) -> PyFace {
        let store = get_store().read();
        let dir = match direction {
            Some(d) => Vector3::new(
                d.get(0).copied().unwrap_or(0.0),
                d.get(1).copied().unwrap_or(0.0),
                d.get(2).copied().unwrap_or(1.0),
            ),
            None => Vector3::ZERO,
        };
        PyFace {
            handle: Face::project(&store, self.handle, target.handle, dir),
        }
    }

    /// Create a face with target area by offsetting
    #[staticmethod]
    #[pyo3(signature = (face, target_area, tolerance=0.01))]
    fn ByOffsetArea(face: &PyFace, target_area: f64, tolerance: f64) -> Option<PyFace> {
        let store = get_store().read();
        Face::by_offset_area(&store, face.handle, target_area, tolerance)
            .map(|h| PyFace { handle: h })
    }

    /// Scale face to achieve target area
    #[pyo3(signature = (target_area))]
    fn ScaleToArea(&self, target_area: f64) -> PyFace {
        let store = get_store().read();
        PyFace {
            handle: Face::scale_to_area(&store, self.handle, target_area),
        }
    }

    /// Get the dictionary attached to this face
    fn GetDictionary(&self) -> PyDictionary {
        let store = get_store().read();
        PyDictionary {
            dict: Face::get_dictionary(&store, self.handle),
        }
    }

    /// Set the dictionary for this face
    fn SetDictionary(&self, dictionary: &PyDictionary) {
        let store = get_store().read();
        Face::set_dictionary(&store, self.handle, dictionary.dict.clone());
    }

    /// Set a single key-value pair in the face's dictionary
    fn SetDictionaryValue(&self, key: &str, value: PyObject, py: Python<'_>) -> PyResult<()> {
        let store = get_store().read();
        let dv = python_to_dict_value(&value, py)?;
        Face::set_dictionary_value(&store, self.handle, key, dv);
        Ok(())
    }

    /// Get a value from the face's dictionary
    fn GetDictionaryValue(&self, key: &str, py: Python<'_>) -> Option<PyObject> {
        let store = get_store().read();
        Face::get_dictionary_value(&store, self.handle, key)
            .map(|v| dict_value_to_python(&v, py))
    }

    fn __repr__(&self) -> String {
        format!("Face(area={:.4})", self.Area())
    }
}

/// Python wrapper for Shell
#[pyclass(name = "Shell")]
#[derive(Clone)]
pub struct PyShell {
    handle: ShellHandle,
}

#[pymethods]
impl PyShell {
    /// Create a shell from faces
    #[staticmethod]
    fn ByFaces(faces: Vec<PyFace>) -> PyResult<Self> {
        let store = get_store().read();
        let handles: Vec<_> = faces.iter().map(|f| f.handle).collect();
        Ok(Self {
            handle: Shell::by_faces(&store, handles),
        })
    }

    /// Check if closed
    fn IsClosed(&self) -> bool {
        let store = get_store().read();
        Shell::is_closed(&store, self.handle)
    }

    /// Get the area
    fn Area(&self) -> f64 {
        let store = get_store().read();
        Shell::area(&store, self.handle)
    }

    /// Get the volume (if closed)
    fn Volume(&self) -> f64 {
        let store = get_store().read();
        Shell::volume(&store, self.handle)
    }

    /// Get faces
    fn Faces(&self) -> Vec<PyFace> {
        let store = get_store().read();
        Shell::faces(&store, self.handle)
            .into_iter()
            .map(|h| PyFace { handle: h })
            .collect()
    }

    /// Get the dictionary attached to this shell
    fn GetDictionary(&self) -> PyDictionary {
        let store = get_store().read();
        PyDictionary {
            dict: Shell::get_dictionary(&store, self.handle),
        }
    }

    /// Set the dictionary for this shell
    fn SetDictionary(&self, dictionary: &PyDictionary) {
        let store = get_store().read();
        Shell::set_dictionary(&store, self.handle, dictionary.dict.clone());
    }

    /// Set a single key-value pair in the shell's dictionary
    fn SetDictionaryValue(&self, key: &str, value: PyObject, py: Python<'_>) -> PyResult<()> {
        let store = get_store().read();
        let dv = python_to_dict_value(&value, py)?;
        Shell::set_dictionary_value(&store, self.handle, key, dv);
        Ok(())
    }

    /// Get a value from the shell's dictionary
    fn GetDictionaryValue(&self, key: &str, py: Python<'_>) -> Option<PyObject> {
        let store = get_store().read();
        Shell::get_dictionary_value(&store, self.handle, key)
            .map(|v| dict_value_to_python(&v, py))
    }

    fn __repr__(&self) -> String {
        format!("Shell(closed={}, area={:.4})", self.IsClosed(), self.Area())
    }
}

/// Python wrapper for Cell
#[pyclass(name = "Cell")]
#[derive(Clone)]
pub struct PyCell {
    handle: CellHandle,
}

#[pymethods]
impl PyCell {
    /// Create a cell from a shell
    #[staticmethod]
    fn ByShell(shell: &PyShell) -> PyResult<Self> {
        let store = get_store().read();
        Ok(Self {
            handle: Cell::by_shell(&store, shell.handle),
        })
    }

    /// Create a cell from faces
    #[staticmethod]
    fn ByFaces(faces: Vec<PyFace>) -> PyResult<Self> {
        let store = get_store().read();
        let handles: Vec<_> = faces.iter().map(|f| f.handle).collect();
        Ok(Self {
            handle: Cell::by_faces(&store, handles),
        })
    }

    /// Create a box cell
    #[staticmethod]
    #[pyo3(signature = (origin_x=0.0, origin_y=0.0, origin_z=0.0, width=1.0, length=1.0, height=1.0))]
    fn Box(origin_x: f64, origin_y: f64, origin_z: f64, width: f64, length: f64, height: f64) -> PyResult<Self> {
        let store = get_store().read();
        let origin = Point3::new(origin_x, origin_y, origin_z);
        Ok(Self {
            handle: Cell::box_cell(&store, origin, width, length, height),
        })
    }

    /// Create a prism cell
    #[staticmethod]
    fn Prism(face: &PyFace, height: f64) -> PyResult<Self> {
        let store = get_store().read();
        Ok(Self {
            handle: Cell::prism(&store, face.handle, height),
        })
    }

    /// Create a cylinder cell
    #[staticmethod]
    #[pyo3(signature = (center_x=0.0, center_y=0.0, center_z=0.0, radius=1.0, height=1.0, segments=32))]
    fn Cylinder(center_x: f64, center_y: f64, center_z: f64, radius: f64, height: f64, segments: usize) -> PyResult<Self> {
        let store = get_store().read();
        let center = Point3::new(center_x, center_y, center_z);
        Ok(Self {
            handle: Cell::cylinder(&store, center, radius, height, segments),
        })
    }

    /// Create a sphere cell
    #[staticmethod]
    #[pyo3(signature = (center_x=0.0, center_y=0.0, center_z=0.0, radius=1.0, u_segments=16, v_segments=8))]
    fn Sphere(center_x: f64, center_y: f64, center_z: f64, radius: f64, u_segments: usize, v_segments: usize) -> PyResult<Self> {
        let store = get_store().read();
        let center = Point3::new(center_x, center_y, center_z);
        Ok(Self {
            handle: Cell::sphere(&store, center, radius, u_segments, v_segments),
        })
    }

    /// Get the volume
    fn Volume(&self) -> f64 {
        let store = get_store().read();
        Cell::volume(&store, self.handle)
    }

    /// Get the surface area
    fn Area(&self) -> f64 {
        let store = get_store().read();
        Cell::area(&store, self.handle)
    }

    /// Get the center of mass
    fn CenterOfMass(&self) -> (f64, f64, f64) {
        let store = get_store().read();
        let c = Cell::center_of_mass(&store, self.handle);
        (c.x, c.y, c.z)
    }

    /// Get the compactness
    fn Compactness(&self) -> f64 {
        let store = get_store().read();
        Cell::compactness(&store, self.handle)
    }

    /// Get faces
    fn Faces(&self) -> Vec<PyFace> {
        let store = get_store().read();
        Cell::faces(&store, self.handle)
            .into_iter()
            .map(|h| PyFace { handle: h })
            .collect()
    }

    /// Get edges
    fn Edges(&self) -> Vec<PyEdge> {
        let store = get_store().read();
        Cell::edges(&store, self.handle)
            .into_iter()
            .map(|h| PyEdge { handle: h })
            .collect()
    }

    /// Get vertices
    fn Vertices(&self) -> Vec<PyVertex> {
        let store = get_store().read();
        Cell::vertices(&store, self.handle)
            .into_iter()
            .map(|h| PyVertex { handle: h })
            .collect()
    }

    /// Check if a point is inside
    fn ContainsPoint(&self, x: f64, y: f64, z: f64) -> bool {
        let store = get_store().read();
        Cell::contains_point(&store, self.handle, Point3::new(x, y, z))
    }

    /// Create a torus cell
    #[staticmethod]
    #[pyo3(signature = (center_x=0.0, center_y=0.0, center_z=0.0, major_radius=1.0, minor_radius=0.25, u_segments=24, v_segments=12, direction=None))]
    fn Torus(
        center_x: f64,
        center_y: f64,
        center_z: f64,
        major_radius: f64,
        minor_radius: f64,
        u_segments: usize,
        v_segments: usize,
        direction: Option<Vec<f64>>,
    ) -> PyResult<Self> {
        let store = get_store().read();
        let center = Point3::new(center_x, center_y, center_z);
        let dir = direction.map(|d| Vector3::new(
            d.get(0).copied().unwrap_or(0.0),
            d.get(1).copied().unwrap_or(0.0),
            d.get(2).copied().unwrap_or(1.0),
        )).unwrap_or(Vector3::Z);
        Ok(Self {
            handle: Cell::torus(&store, center, major_radius, minor_radius, u_segments, v_segments, dir),
        })
    }

    /// Create a roof cell from a base face
    #[staticmethod]
    #[pyo3(signature = (base_face, height=3.0, roof_type="gable"))]
    fn Roof(base_face: &PyFace, height: f64, roof_type: &str) -> PyResult<Self> {
        let store = get_store().read();
        Ok(Self {
            handle: Cell::roof(&store, base_face.handle, height, roof_type),
        })
    }

    /// Create a cone cell
    #[staticmethod]
    #[pyo3(signature = (center_x=0.0, center_y=0.0, center_z=0.0, radius=1.0, height=2.0, segments=16))]
    fn Cone(center_x: f64, center_y: f64, center_z: f64, radius: f64, height: f64, segments: usize) -> PyResult<Self> {
        let store = get_store().read();
        let center = Point3::new(center_x, center_y, center_z);
        Ok(Self {
            handle: Cell::cone(&store, center, radius, height, segments),
        })
    }

    /// Get the dictionary attached to this cell
    fn GetDictionary(&self) -> PyDictionary {
        let store = get_store().read();
        PyDictionary {
            dict: Cell::get_dictionary(&store, self.handle),
        }
    }

    /// Set the dictionary for this cell
    fn SetDictionary(&self, dictionary: &PyDictionary) {
        let store = get_store().read();
        Cell::set_dictionary(&store, self.handle, dictionary.dict.clone());
    }

    /// Set a single key-value pair in the cell's dictionary
    fn SetDictionaryValue(&self, key: &str, value: PyObject, py: Python<'_>) -> PyResult<()> {
        let store = get_store().read();
        let dv = python_to_dict_value(&value, py)?;
        Cell::set_dictionary_value(&store, self.handle, key, dv);
        Ok(())
    }

    /// Get a value from the cell's dictionary
    fn GetDictionaryValue(&self, key: &str, py: Python<'_>) -> Option<PyObject> {
        let store = get_store().read();
        Cell::get_dictionary_value(&store, self.handle, key)
            .map(|v| dict_value_to_python(&v, py))
    }

    fn __repr__(&self) -> String {
        format!("Cell(volume={:.4}, area={:.4})", self.Volume(), self.Area())
    }
}

/// Python wrapper for CellComplex
#[pyclass(name = "CellComplex")]
#[derive(Clone)]
pub struct PyCellComplex {
    handle: CellComplexHandle,
}

#[pymethods]
impl PyCellComplex {
    /// Create a cell complex from cells
    #[staticmethod]
    fn ByCells(cells: Vec<PyCell>) -> PyResult<Self> {
        let store = get_store().read();
        let handles: Vec<_> = cells.iter().map(|c| c.handle).collect();
        Ok(Self {
            handle: CellComplex::by_cells(&store, handles),
        })
    }

    /// Create a box cell complex
    #[staticmethod]
    #[pyo3(signature = (origin_x=0.0, origin_y=0.0, origin_z=0.0, width=1.0, length=1.0, height=1.0))]
    fn Box(origin_x: f64, origin_y: f64, origin_z: f64, width: f64, length: f64, height: f64) -> PyResult<Self> {
        let store = get_store().read();
        let origin = Point3::new(origin_x, origin_y, origin_z);
        Ok(Self {
            handle: CellComplex::box_complex(&store, origin, width, length, height),
        })
    }

    /// Get the number of cells
    fn NumCells(&self) -> usize {
        let store = get_store().read();
        CellComplex::num_cells(&store, self.handle)
    }

    /// Get the volume
    fn Volume(&self) -> f64 {
        let store = get_store().read();
        CellComplex::volume(&store, self.handle)
    }

    /// Get the area
    fn Area(&self) -> f64 {
        let store = get_store().read();
        CellComplex::area(&store, self.handle)
    }

    /// Get cells
    fn Cells(&self) -> Vec<PyCell> {
        let store = get_store().read();
        CellComplex::cells(&store, self.handle)
            .into_iter()
            .map(|h| PyCell { handle: h })
            .collect()
    }

    /// Get faces
    fn Faces(&self) -> Vec<PyFace> {
        let store = get_store().read();
        CellComplex::faces(&store, self.handle)
            .into_iter()
            .map(|h| PyFace { handle: h })
            .collect()
    }

    /// Get the dictionary attached to this cell complex
    fn GetDictionary(&self) -> PyDictionary {
        let store = get_store().read();
        PyDictionary {
            dict: CellComplex::get_dictionary(&store, self.handle),
        }
    }

    /// Set the dictionary for this cell complex
    fn SetDictionary(&self, dictionary: &PyDictionary) {
        let store = get_store().read();
        CellComplex::set_dictionary(&store, self.handle, dictionary.dict.clone());
    }

    /// Set a single key-value pair in the cell complex's dictionary
    fn SetDictionaryValue(&self, key: &str, value: PyObject, py: Python<'_>) -> PyResult<()> {
        let store = get_store().read();
        let dv = python_to_dict_value(&value, py)?;
        CellComplex::set_dictionary_value(&store, self.handle, key, dv);
        Ok(())
    }

    /// Get a value from the cell complex's dictionary
    fn GetDictionaryValue(&self, key: &str, py: Python<'_>) -> Option<PyObject> {
        let store = get_store().read();
        CellComplex::get_dictionary_value(&store, self.handle, key)
            .map(|v| dict_value_to_python(&v, py))
    }

    fn __repr__(&self) -> String {
        format!("CellComplex(cells={}, volume={:.4})", self.NumCells(), self.Volume())
    }
}

/// Topology operations module
#[pyclass(name = "Topology")]
pub struct PyTopology;

#[pymethods]
impl PyTopology {
    /// Union of two cells
    #[staticmethod]
    fn Union(cell1: &PyCell, cell2: &PyCell) -> PyResult<PyCell> {
        let store = get_store().read();
        let result = Boolean::union(&store, cell1.handle, cell2.handle);
        if result.success {
            if let Some(TopologyHandle::Cell(h)) = result.result {
                return Ok(PyCell { handle: h });
            }
        }
        Err(PyValueError::new_err(result.message))
    }

    /// Intersection of two cells
    #[staticmethod]
    fn Intersection(cell1: &PyCell, cell2: &PyCell) -> PyResult<PyCell> {
        let store = get_store().read();
        let result = Boolean::intersection(&store, cell1.handle, cell2.handle);
        if result.success {
            if let Some(TopologyHandle::Cell(h)) = result.result {
                return Ok(PyCell { handle: h });
            }
        }
        Err(PyValueError::new_err(result.message))
    }

    /// Difference of two cells
    #[staticmethod]
    fn Difference(cell1: &PyCell, cell2: &PyCell) -> PyResult<PyCell> {
        let store = get_store().read();
        let result = Boolean::difference(&store, cell1.handle, cell2.handle);
        if result.success {
            if let Some(TopologyHandle::Cell(h)) = result.result {
                return Ok(PyCell { handle: h });
            }
        }
        Err(PyValueError::new_err(result.message))
    }

    /// Translate a vertex
    #[staticmethod]
    #[pyo3(signature = (vertex, x=0.0, y=0.0, z=0.0))]
    fn TranslateVertex(vertex: &PyVertex, x: f64, y: f64, z: f64) -> PyVertex {
        let store = get_store().read();
        PyVertex {
            handle: topologic_core::topology::TopologyTransform::translate_vertex(&store, vertex.handle, x, y, z),
        }
    }

    /// Translate an edge
    #[staticmethod]
    #[pyo3(signature = (edge, x=0.0, y=0.0, z=0.0))]
    fn TranslateEdge(edge: &PyEdge, x: f64, y: f64, z: f64) -> PyEdge {
        let store = get_store().read();
        PyEdge {
            handle: topologic_core::topology::TopologyTransform::translate_edge(&store, edge.handle, x, y, z),
        }
    }

    /// Translate a wire
    #[staticmethod]
    #[pyo3(signature = (wire, x=0.0, y=0.0, z=0.0))]
    fn TranslateWire(wire: &PyWire, x: f64, y: f64, z: f64) -> PyWire {
        let store = get_store().read();
        PyWire {
            handle: topologic_core::topology::TopologyTransform::translate_wire(&store, wire.handle, x, y, z),
        }
    }

    /// Translate a face
    #[staticmethod]
    #[pyo3(signature = (face, x=0.0, y=0.0, z=0.0))]
    fn TranslateFace(face: &PyFace, x: f64, y: f64, z: f64) -> PyFace {
        let store = get_store().read();
        PyFace {
            handle: topologic_core::topology::TopologyTransform::translate_face(&store, face.handle, x, y, z),
        }
    }

    /// Translate a shell
    #[staticmethod]
    #[pyo3(signature = (shell, x=0.0, y=0.0, z=0.0))]
    fn TranslateShell(shell: &PyShell, x: f64, y: f64, z: f64) -> PyShell {
        let store = get_store().read();
        PyShell {
            handle: topologic_core::topology::TopologyTransform::translate_shell(&store, shell.handle, x, y, z),
        }
    }

    /// Translate a cell
    #[staticmethod]
    #[pyo3(signature = (cell, x=0.0, y=0.0, z=0.0))]
    fn TranslateCell(cell: &PyCell, x: f64, y: f64, z: f64) -> PyCell {
        let store = get_store().read();
        PyCell {
            handle: topologic_core::topology::TopologyTransform::translate_cell(&store, cell.handle, x, y, z),
        }
    }

    /// Rotate a vertex around an axis
    #[staticmethod]
    #[pyo3(signature = (vertex, origin=None, axis=None, angle=0.0))]
    fn RotateVertex(vertex: &PyVertex, origin: Option<&PyVertex>, axis: Option<Vec<f64>>, angle: f64) -> PyVertex {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        let axis_vec = match axis {
            Some(a) => Vector3::new(
                a.get(0).copied().unwrap_or(0.0),
                a.get(1).copied().unwrap_or(0.0),
                a.get(2).copied().unwrap_or(1.0),
            ),
            None => Vector3::Z,
        };
        PyVertex {
            handle: topologic_core::topology::TopologyTransform::rotate_vertex(&store, vertex.handle, origin_point, axis_vec, angle),
        }
    }

    /// Rotate an edge around an axis
    #[staticmethod]
    #[pyo3(signature = (edge, origin=None, axis=None, angle=0.0))]
    fn RotateEdge(edge: &PyEdge, origin: Option<&PyVertex>, axis: Option<Vec<f64>>, angle: f64) -> PyEdge {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        let axis_vec = match axis {
            Some(a) => Vector3::new(
                a.get(0).copied().unwrap_or(0.0),
                a.get(1).copied().unwrap_or(0.0),
                a.get(2).copied().unwrap_or(1.0),
            ),
            None => Vector3::Z,
        };
        PyEdge {
            handle: topologic_core::topology::TopologyTransform::rotate_edge(&store, edge.handle, origin_point, axis_vec, angle),
        }
    }

    /// Rotate a wire around an axis
    #[staticmethod]
    #[pyo3(signature = (wire, origin=None, axis=None, angle=0.0))]
    fn RotateWire(wire: &PyWire, origin: Option<&PyVertex>, axis: Option<Vec<f64>>, angle: f64) -> PyWire {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        let axis_vec = match axis {
            Some(a) => Vector3::new(
                a.get(0).copied().unwrap_or(0.0),
                a.get(1).copied().unwrap_or(0.0),
                a.get(2).copied().unwrap_or(1.0),
            ),
            None => Vector3::Z,
        };
        PyWire {
            handle: topologic_core::topology::TopologyTransform::rotate_wire(&store, wire.handle, origin_point, axis_vec, angle),
        }
    }

    /// Rotate a face around an axis
    #[staticmethod]
    #[pyo3(signature = (face, origin=None, axis=None, angle=0.0))]
    fn RotateFace(face: &PyFace, origin: Option<&PyVertex>, axis: Option<Vec<f64>>, angle: f64) -> PyFace {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        let axis_vec = match axis {
            Some(a) => Vector3::new(
                a.get(0).copied().unwrap_or(0.0),
                a.get(1).copied().unwrap_or(0.0),
                a.get(2).copied().unwrap_or(1.0),
            ),
            None => Vector3::Z,
        };
        PyFace {
            handle: topologic_core::topology::TopologyTransform::rotate_face(&store, face.handle, origin_point, axis_vec, angle),
        }
    }

    /// Rotate a shell around an axis
    #[staticmethod]
    #[pyo3(signature = (shell, origin=None, axis=None, angle=0.0))]
    fn RotateShell(shell: &PyShell, origin: Option<&PyVertex>, axis: Option<Vec<f64>>, angle: f64) -> PyShell {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        let axis_vec = match axis {
            Some(a) => Vector3::new(
                a.get(0).copied().unwrap_or(0.0),
                a.get(1).copied().unwrap_or(0.0),
                a.get(2).copied().unwrap_or(1.0),
            ),
            None => Vector3::Z,
        };
        PyShell {
            handle: topologic_core::topology::TopologyTransform::rotate_shell(&store, shell.handle, origin_point, axis_vec, angle),
        }
    }

    /// Rotate a cell around an axis
    #[staticmethod]
    #[pyo3(signature = (cell, origin=None, axis=None, angle=0.0))]
    fn RotateCell(cell: &PyCell, origin: Option<&PyVertex>, axis: Option<Vec<f64>>, angle: f64) -> PyCell {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        let axis_vec = match axis {
            Some(a) => Vector3::new(
                a.get(0).copied().unwrap_or(0.0),
                a.get(1).copied().unwrap_or(0.0),
                a.get(2).copied().unwrap_or(1.0),
            ),
            None => Vector3::Z,
        };
        PyCell {
            handle: topologic_core::topology::TopologyTransform::rotate_cell(&store, cell.handle, origin_point, axis_vec, angle),
        }
    }

    /// Scale a vertex from an origin
    #[staticmethod]
    #[pyo3(signature = (vertex, origin=None, x=1.0, y=1.0, z=1.0))]
    fn ScaleVertex(vertex: &PyVertex, origin: Option<&PyVertex>, x: f64, y: f64, z: f64) -> PyVertex {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        PyVertex {
            handle: topologic_core::topology::TopologyTransform::scale_vertex(&store, vertex.handle, origin_point, x, y, z),
        }
    }

    /// Scale an edge from an origin
    #[staticmethod]
    #[pyo3(signature = (edge, origin=None, x=1.0, y=1.0, z=1.0))]
    fn ScaleEdge(edge: &PyEdge, origin: Option<&PyVertex>, x: f64, y: f64, z: f64) -> PyEdge {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        PyEdge {
            handle: topologic_core::topology::TopologyTransform::scale_edge(&store, edge.handle, origin_point, x, y, z),
        }
    }

    /// Scale a wire from an origin
    #[staticmethod]
    #[pyo3(signature = (wire, origin=None, x=1.0, y=1.0, z=1.0))]
    fn ScaleWire(wire: &PyWire, origin: Option<&PyVertex>, x: f64, y: f64, z: f64) -> PyWire {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        PyWire {
            handle: topologic_core::topology::TopologyTransform::scale_wire(&store, wire.handle, origin_point, x, y, z),
        }
    }

    /// Scale a face from an origin
    #[staticmethod]
    #[pyo3(signature = (face, origin=None, x=1.0, y=1.0, z=1.0))]
    fn ScaleFace(face: &PyFace, origin: Option<&PyVertex>, x: f64, y: f64, z: f64) -> PyFace {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        PyFace {
            handle: topologic_core::topology::TopologyTransform::scale_face(&store, face.handle, origin_point, x, y, z),
        }
    }

    /// Scale a shell from an origin
    #[staticmethod]
    #[pyo3(signature = (shell, origin=None, x=1.0, y=1.0, z=1.0))]
    fn ScaleShell(shell: &PyShell, origin: Option<&PyVertex>, x: f64, y: f64, z: f64) -> PyShell {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        PyShell {
            handle: topologic_core::topology::TopologyTransform::scale_shell(&store, shell.handle, origin_point, x, y, z),
        }
    }

    /// Scale a cell from an origin
    #[staticmethod]
    #[pyo3(signature = (cell, origin=None, x=1.0, y=1.0, z=1.0))]
    fn ScaleCell(cell: &PyCell, origin: Option<&PyVertex>, x: f64, y: f64, z: f64) -> PyCell {
        let store = get_store().read();
        let origin_point = match origin {
            Some(v) => Vertex::point(&store, v.handle),
            None => Point3::ZERO,
        };
        PyCell {
            handle: topologic_core::topology::TopologyTransform::scale_cell(&store, cell.handle, origin_point, x, y, z),
        }
    }

    /// Transform a face by a 4x4 matrix
    #[staticmethod]
    fn TransformFace(face: &PyFace, matrix: Vec<Vec<f64>>) -> PyResult<PyFace> {
        let store = get_store().read();
        let mat = PyMatrix::vec_to_matrix(&matrix)?;
        Ok(PyFace {
            handle: topologic_core::topology::TopologyTransform::transform_face(&store, face.handle, mat),
        })
    }

    /// Transform a cell by a 4x4 matrix
    #[staticmethod]
    fn TransformCell(cell: &PyCell, matrix: Vec<Vec<f64>>) -> PyResult<PyCell> {
        let store = get_store().read();
        let mat = PyMatrix::vec_to_matrix(&matrix)?;
        Ok(PyCell {
            handle: topologic_core::topology::TopologyTransform::transform_cell(&store, cell.handle, mat),
        })
    }

    /// Show a topology using Plotly visualization
    /// Requires plotly to be installed: pip install plotly
    #[staticmethod]
    #[pyo3(signature = (topology, renderer="browser", face_color="lightblue", edge_color="black", vertex_color="red", face_opacity=0.5, show_edges=true, show_vertices=true, width=800, height=600))]
    fn Show(
        py: Python<'_>,
        topology: &Bound<'_, PyAny>,
        renderer: &str,
        face_color: &str,
        edge_color: &str,
        vertex_color: &str,
        face_opacity: f64,
        show_edges: bool,
        show_vertices: bool,
        width: i32,
        height: i32,
    ) -> PyResult<PyObject> {
        let store = get_store().read();

        // Collect vertices and edges based on topology type
        let mut vertices: Vec<(f64, f64, f64)> = Vec::new();
        let mut edges: Vec<((f64, f64, f64), (f64, f64, f64))> = Vec::new();
        let mut triangles: Vec<((f64, f64, f64), (f64, f64, f64), (f64, f64, f64))> = Vec::new();

        // Helper function to extract geometry from a face handle
        fn extract_face_geometry(
            store: &topologic_core::topology::TopologyStore,
            face_handle: FaceHandle,
            verts: &mut Vec<(f64, f64, f64)>,
            edgs: &mut Vec<((f64, f64, f64), (f64, f64, f64))>,
            tris: &mut Vec<((f64, f64, f64), (f64, f64, f64), (f64, f64, f64))>,
        ) {
            // Get edges from face
            let face_edges = Face::edges(store, face_handle);
            for edge_handle in &face_edges {
                let (start, end) = Edge::vertices(store, *edge_handle);
                let p1 = Vertex::point(store, start);
                let p2 = Vertex::point(store, end);
                edgs.push(((p1.x, p1.y, p1.z), (p2.x, p2.y, p2.z)));
            }

            // Get vertices from face
            let face_verts = Face::vertices(store, face_handle);
            for v in &face_verts {
                let p = Vertex::point(store, *v);
                verts.push((p.x, p.y, p.z));
            }

            // Triangulate the face for mesh display
            let mesh = MeshGenerator::from_face(store, face_handle);
            for tri in &mesh.indices {
                let v0 = mesh.vertices[tri[0] as usize];
                let v1 = mesh.vertices[tri[1] as usize];
                let v2 = mesh.vertices[tri[2] as usize];
                tris.push((
                    (v0.x, v0.y, v0.z),
                    (v1.x, v1.y, v1.z),
                    (v2.x, v2.y, v2.z),
                ));
            }
        }

        // Determine topology type and extract geometry
        if let Ok(vertex) = topology.extract::<PyVertex>() {
            let p = Vertex::point(&store, vertex.handle);
            vertices.push((p.x, p.y, p.z));
        } else if let Ok(edge) = topology.extract::<PyEdge>() {
            let (start, end) = Edge::vertices(&store, edge.handle);
            let p1 = Vertex::point(&store, start);
            let p2 = Vertex::point(&store, end);
            vertices.push((p1.x, p1.y, p1.z));
            vertices.push((p2.x, p2.y, p2.z));
            edges.push(((p1.x, p1.y, p1.z), (p2.x, p2.y, p2.z)));
        } else if let Ok(wire) = topology.extract::<PyWire>() {
            let wire_edges = Wire::edges(&store, wire.handle);
            for edge_handle in &wire_edges {
                let (start, end) = Edge::vertices(&store, *edge_handle);
                let p1 = Vertex::point(&store, start);
                let p2 = Vertex::point(&store, end);
                vertices.push((p1.x, p1.y, p1.z));
                vertices.push((p2.x, p2.y, p2.z));
                edges.push(((p1.x, p1.y, p1.z), (p2.x, p2.y, p2.z)));
            }
        } else if let Ok(face) = topology.extract::<PyFace>() {
            extract_face_geometry(&store, face.handle, &mut vertices, &mut edges, &mut triangles);
        } else if let Ok(shell) = topology.extract::<PyShell>() {
            let faces = Shell::faces(&store, shell.handle);
            for face_handle in faces {
                extract_face_geometry(&store, face_handle, &mut vertices, &mut edges, &mut triangles);
            }
        } else if let Ok(cell) = topology.extract::<PyCell>() {
            let faces = Cell::faces(&store, cell.handle);
            for face_handle in faces {
                extract_face_geometry(&store, face_handle, &mut vertices, &mut edges, &mut triangles);
            }
        } else {
            return Err(PyValueError::new_err("Unsupported topology type for Show"));
        }

        // Import plotly
        let plotly = py.import("plotly.graph_objects")?;

        // Create figure
        let figure_class = plotly.getattr("Figure")?;
        let fig = figure_class.call0()?;

        // Add mesh if we have triangles
        if !triangles.is_empty() {
            let mut x_mesh: Vec<f64> = Vec::new();
            let mut y_mesh: Vec<f64> = Vec::new();
            let mut z_mesh: Vec<f64> = Vec::new();
            let mut i_idx: Vec<i32> = Vec::new();
            let mut j_idx: Vec<i32> = Vec::new();
            let mut k_idx: Vec<i32> = Vec::new();

            for (idx, (v0, v1, v2)) in triangles.iter().enumerate() {
                let base = (idx * 3) as i32;
                x_mesh.push(v0.0); y_mesh.push(v0.1); z_mesh.push(v0.2);
                x_mesh.push(v1.0); y_mesh.push(v1.1); z_mesh.push(v1.2);
                x_mesh.push(v2.0); y_mesh.push(v2.1); z_mesh.push(v2.2);
                i_idx.push(base);
                j_idx.push(base + 1);
                k_idx.push(base + 2);
            }

            let mesh3d_class = plotly.getattr("Mesh3d")?;
            let kwargs = pyo3::types::PyDict::new(py);
            kwargs.set_item("x", x_mesh)?;
            kwargs.set_item("y", y_mesh)?;
            kwargs.set_item("z", z_mesh)?;
            kwargs.set_item("i", i_idx)?;
            kwargs.set_item("j", j_idx)?;
            kwargs.set_item("k", k_idx)?;
            kwargs.set_item("color", face_color)?;
            kwargs.set_item("opacity", face_opacity)?;
            kwargs.set_item("flatshading", true)?;

            let mesh_trace = mesh3d_class.call((), Some(&kwargs))?;
            fig.call_method1("add_trace", (mesh_trace,))?;
        }

        // Add edges
        if show_edges && !edges.is_empty() {
            let scatter3d_class = plotly.getattr("Scatter3d")?;

            for (p1, p2) in &edges {
                let x_line = vec![p1.0, p2.0];
                let y_line = vec![p1.1, p2.1];
                let z_line = vec![p1.2, p2.2];

                let kwargs = pyo3::types::PyDict::new(py);
                kwargs.set_item("x", x_line)?;
                kwargs.set_item("y", y_line)?;
                kwargs.set_item("z", z_line)?;
                kwargs.set_item("mode", "lines")?;
                kwargs.set_item("showlegend", false)?;

                let line_dict = pyo3::types::PyDict::new(py);
                line_dict.set_item("color", edge_color)?;
                line_dict.set_item("width", 2)?;
                kwargs.set_item("line", line_dict)?;

                let edge_trace = scatter3d_class.call((), Some(&kwargs))?;
                fig.call_method1("add_trace", (edge_trace,))?;
            }
        }

        // Add vertices
        if show_vertices && !vertices.is_empty() {
            // Deduplicate vertices
            let mut unique_verts: Vec<(f64, f64, f64)> = Vec::new();
            for v in &vertices {
                let is_dup = unique_verts.iter().any(|u| {
                    (u.0 - v.0).abs() < 1e-9 && (u.1 - v.1).abs() < 1e-9 && (u.2 - v.2).abs() < 1e-9
                });
                if !is_dup {
                    unique_verts.push(*v);
                }
            }

            let x_verts: Vec<f64> = unique_verts.iter().map(|v| v.0).collect();
            let y_verts: Vec<f64> = unique_verts.iter().map(|v| v.1).collect();
            let z_verts: Vec<f64> = unique_verts.iter().map(|v| v.2).collect();

            let scatter3d_class = plotly.getattr("Scatter3d")?;
            let kwargs = pyo3::types::PyDict::new(py);
            kwargs.set_item("x", x_verts)?;
            kwargs.set_item("y", y_verts)?;
            kwargs.set_item("z", z_verts)?;
            kwargs.set_item("mode", "markers")?;
            kwargs.set_item("showlegend", false)?;

            let marker_dict = pyo3::types::PyDict::new(py);
            marker_dict.set_item("color", vertex_color)?;
            marker_dict.set_item("size", 4)?;
            kwargs.set_item("marker", marker_dict)?;

            let vertex_trace = scatter3d_class.call((), Some(&kwargs))?;
            fig.call_method1("add_trace", (vertex_trace,))?;
        }

        // Update layout
        let layout_kwargs = pyo3::types::PyDict::new(py);
        layout_kwargs.set_item("width", width)?;
        layout_kwargs.set_item("height", height)?;

        let scene_dict = pyo3::types::PyDict::new(py);
        let aspect_dict = pyo3::types::PyDict::new(py);
        aspect_dict.set_item("x", 1)?;
        aspect_dict.set_item("y", 1)?;
        aspect_dict.set_item("z", 1)?;
        scene_dict.set_item("aspectmode", "data")?;
        layout_kwargs.set_item("scene", scene_dict)?;

        fig.call_method("update_layout", (), Some(&layout_kwargs))?;

        // Show the figure
        let show_kwargs = pyo3::types::PyDict::new(py);
        show_kwargs.set_item("renderer", renderer)?;
        fig.call_method("show", (), Some(&show_kwargs))?;

        Ok(fig.unbind())
    }

    /// Explode a cell into its constituent faces
    #[staticmethod]
    fn ExplodeCellToFaces(cell: &PyCell) -> Vec<PyFace> {
        let store = get_store().read();
        Cell::faces(&store, cell.handle)
            .into_iter()
            .map(|h| PyFace { handle: h })
            .collect()
    }

    /// Explode a cell into its constituent edges
    #[staticmethod]
    fn ExplodeCellToEdges(cell: &PyCell) -> Vec<PyEdge> {
        let store = get_store().read();
        Cell::edges(&store, cell.handle)
            .into_iter()
            .map(|h| PyEdge { handle: h })
            .collect()
    }

    /// Explode a cell into its constituent vertices
    #[staticmethod]
    fn ExplodeCellToVertices(cell: &PyCell) -> Vec<PyVertex> {
        let store = get_store().read();
        Cell::vertices(&store, cell.handle)
            .into_iter()
            .map(|h| PyVertex { handle: h })
            .collect()
    }

    /// Explode a face into its constituent edges
    #[staticmethod]
    fn ExplodeFaceToEdges(face: &PyFace) -> Vec<PyEdge> {
        let store = get_store().read();
        Face::edges(&store, face.handle)
            .into_iter()
            .map(|h| PyEdge { handle: h })
            .collect()
    }

    /// Explode a face into its constituent vertices
    #[staticmethod]
    fn ExplodeFaceToVertices(face: &PyFace) -> Vec<PyVertex> {
        let store = get_store().read();
        Face::vertices(&store, face.handle)
            .into_iter()
            .map(|h| PyVertex { handle: h })
            .collect()
    }

    /// Explode a wire into its constituent edges
    #[staticmethod]
    fn ExplodeWireToEdges(wire: &PyWire) -> Vec<PyEdge> {
        let store = get_store().read();
        Wire::edges(&store, wire.handle)
            .into_iter()
            .map(|h| PyEdge { handle: h })
            .collect()
    }

    /// Explode a wire into its constituent vertices
    #[staticmethod]
    fn ExplodeWireToVertices(wire: &PyWire) -> Vec<PyVertex> {
        let store = get_store().read();
        Wire::vertices(&store, wire.handle)
            .into_iter()
            .map(|h| PyVertex { handle: h })
            .collect()
    }

    /// Explode a CellComplex into its constituent cells
    #[staticmethod]
    fn ExplodeCellComplexToCells(cc: &PyCellComplex) -> Vec<PyCell> {
        let store = get_store().read();
        CellComplex::cells(&store, cc.handle)
            .into_iter()
            .map(|h| PyCell { handle: h })
            .collect()
    }

    /// Get faces adjacent to an edge (shared by the edge)
    #[staticmethod]
    fn FacesAdjacentToEdge(edge: &PyEdge) -> Vec<PyFace> {
        let store = get_store().read();
        Edge::faces(&store, edge.handle)
            .into_iter()
            .map(|h| PyFace { handle: h })
            .collect()
    }

    /// Get adjacent vertices of a vertex
    #[staticmethod]
    fn AdjacentVertices(vertex: &PyVertex) -> Vec<PyVertex> {
        let store = get_store().read();
        Vertex::adjacent_vertices(&store, vertex.handle)
            .into_iter()
            .map(|h| PyVertex { handle: h })
            .collect()
    }

    /// Get wires containing an edge
    #[staticmethod]
    fn WiresContainingEdge(edge: &PyEdge) -> Vec<PyWire> {
        let store = get_store().read();
        Edge::wires(&store, edge.handle)
            .into_iter()
            .map(|h| PyWire { handle: h })
            .collect()
    }

    /// Get adjacent edges of an edge (sharing a vertex)
    #[staticmethod]
    fn AdjacentEdges(edge: &PyEdge) -> Vec<PyEdge> {
        let store = get_store().read();
        Edge::adjacent_edges(&store, edge.handle)
            .into_iter()
            .map(|h| PyEdge { handle: h })
            .collect()
    }
}

/// Mesh generation
#[pyclass(name = "Mesh")]
pub struct PyMesh {
    mesh: TriangleMesh,
}

#[pymethods]
impl PyMesh {
    /// Create mesh from a face
    #[staticmethod]
    fn ByFace(face: &PyFace) -> Self {
        let store = get_store().read();
        Self {
            mesh: MeshGenerator::from_face(&store, face.handle),
        }
    }

    /// Create mesh from a cell
    #[staticmethod]
    fn ByCell(cell: &PyCell) -> Self {
        let store = get_store().read();
        Self {
            mesh: MeshGenerator::from_cell(&store, cell.handle),
        }
    }

    /// Get the number of vertices
    fn NumVertices(&self) -> usize {
        self.mesh.num_vertices()
    }

    /// Get the number of triangles
    fn NumTriangles(&self) -> usize {
        self.mesh.num_triangles()
    }

    /// Get the area
    fn Area(&self) -> f64 {
        self.mesh.area()
    }

    /// Export to OBJ format
    fn ToOBJ(&self) -> String {
        MeshExporter::to_obj(&self.mesh)
    }

    /// Export to STL format
    fn ToSTL(&self) -> String {
        MeshExporter::to_stl(&self.mesh)
    }

    fn __repr__(&self) -> String {
        format!("Mesh(vertices={}, triangles={})", self.NumVertices(), self.NumTriangles())
    }
}

/// Python wrapper for Cluster
#[pyclass(name = "Cluster")]
#[derive(Clone)]
pub struct PyCluster {
    handle: ClusterHandle,
}

#[pymethods]
impl PyCluster {
    /// Create a cluster from vertices
    #[staticmethod]
    fn ByVertices(vertices: Vec<PyVertex>) -> PyResult<Self> {
        let store = get_store().read();
        let handles: Vec<_> = vertices.iter().map(|v| v.handle).collect();
        Ok(Self {
            handle: Cluster::by_vertices(&store, handles),
        })
    }

    /// Create a cluster from edges
    #[staticmethod]
    fn ByEdges(edges: Vec<PyEdge>) -> PyResult<Self> {
        let store = get_store().read();
        let handles: Vec<_> = edges.iter().map(|e| e.handle).collect();
        Ok(Self {
            handle: Cluster::by_edges(&store, handles),
        })
    }

    /// Create a cluster from faces
    #[staticmethod]
    fn ByFaces(faces: Vec<PyFace>) -> PyResult<Self> {
        let store = get_store().read();
        let handles: Vec<_> = faces.iter().map(|f| f.handle).collect();
        Ok(Self {
            handle: Cluster::by_faces(&store, handles),
        })
    }

    /// Create a cluster from cells
    #[staticmethod]
    fn ByCells(cells: Vec<PyCell>) -> PyResult<Self> {
        let store = get_store().read();
        let handles: Vec<_> = cells.iter().map(|c| c.handle).collect();
        Ok(Self {
            handle: Cluster::by_cells(&store, handles),
        })
    }

    /// Get all vertices in the cluster
    fn Vertices(&self) -> Vec<PyVertex> {
        let store = get_store().read();
        Cluster::vertices(&store, self.handle)
            .into_iter()
            .map(|h| PyVertex { handle: h })
            .collect()
    }

    /// Get all edges in the cluster
    fn Edges(&self) -> Vec<PyEdge> {
        let store = get_store().read();
        Cluster::edges(&store, self.handle)
            .into_iter()
            .map(|h| PyEdge { handle: h })
            .collect()
    }

    /// Get all faces in the cluster
    fn Faces(&self) -> Vec<PyFace> {
        let store = get_store().read();
        Cluster::faces(&store, self.handle)
            .into_iter()
            .map(|h| PyFace { handle: h })
            .collect()
    }

    /// Get all cells in the cluster
    fn Cells(&self) -> Vec<PyCell> {
        let store = get_store().read();
        Cluster::cells(&store, self.handle)
            .into_iter()
            .map(|h| PyCell { handle: h })
            .collect()
    }

    /// Get the number of members
    fn Size(&self) -> usize {
        let store = get_store().read();
        Cluster::size(&store, self.handle)
    }

    /// Check if empty
    fn IsEmpty(&self) -> bool {
        let store = get_store().read();
        Cluster::is_empty(&store, self.handle)
    }

    /// Get the center of mass
    fn CenterOfMass(&self) -> (f64, f64, f64) {
        let store = get_store().read();
        let c = Cluster::center_of_mass(&store, self.handle);
        (c.x, c.y, c.z)
    }

    /// Get the bounding box
    fn BoundingBox(&self) -> ((f64, f64, f64), (f64, f64, f64)) {
        let store = get_store().read();
        let (min, max) = Cluster::bounding_box(&store, self.handle);
        ((min.x, min.y, min.z), (max.x, max.y, max.z))
    }

    /// Get the highest dimension
    fn HighestDimension(&self) -> Option<u8> {
        let store = get_store().read();
        Cluster::highest_dimension(&store, self.handle)
    }

    /// Get the dictionary attached to this cluster
    fn GetDictionary(&self) -> PyDictionary {
        let store = get_store().read();
        PyDictionary {
            dict: Cluster::get_dictionary(&store, self.handle),
        }
    }

    /// Set the dictionary for this cluster
    fn SetDictionary(&self, dictionary: &PyDictionary) {
        let store = get_store().read();
        Cluster::set_dictionary(&store, self.handle, dictionary.dict.clone());
    }

    /// Set a single key-value pair in the cluster's dictionary
    fn SetDictionaryValue(&self, key: &str, value: PyObject, py: Python<'_>) -> PyResult<()> {
        let store = get_store().read();
        let dv = python_to_dict_value(&value, py)?;
        Cluster::set_dictionary_value(&store, self.handle, key, dv);
        Ok(())
    }

    /// Get a value from the cluster's dictionary
    fn GetDictionaryValue(&self, key: &str, py: Python<'_>) -> Option<PyObject> {
        let store = get_store().read();
        Cluster::get_dictionary_value(&store, self.handle, key)
            .map(|v| dict_value_to_python(&v, py))
    }

    /// K-Means clustering on vertex positions
    /// Returns cluster labels for each vertex (0 to k-1)
    #[staticmethod]
    #[pyo3(signature = (vertices, k, max_iterations=100, tolerance=0.0001))]
    fn KMeans(vertices: Vec<PyVertex>, k: usize, max_iterations: usize, tolerance: f64) -> Vec<usize> {
        let store = get_store().read();
        let handles: Vec<_> = vertices.iter().map(|v| v.handle).collect();
        Cluster::kmeans(&store, &handles, k, max_iterations, tolerance)
    }

    /// DBSCAN clustering on vertex positions
    /// Returns cluster labels for each vertex (-1 for noise, 0+ for clusters)
    #[staticmethod]
    #[pyo3(signature = (vertices, eps, min_samples=5))]
    fn DBSCAN(vertices: Vec<PyVertex>, eps: f64, min_samples: usize) -> Vec<i32> {
        let store = get_store().read();
        let handles: Vec<_> = vertices.iter().map(|v| v.handle).collect();
        Cluster::dbscan(&store, &handles, eps, min_samples)
    }

    /// HDBSCAN clustering on vertex positions
    /// Returns cluster labels for each vertex (-1 for noise, 0+ for clusters)
    #[staticmethod]
    #[pyo3(signature = (vertices, min_cluster_size=5, min_samples=5))]
    fn HDBSCAN(vertices: Vec<PyVertex>, min_cluster_size: usize, min_samples: usize) -> Vec<i32> {
        let store = get_store().read();
        let handles: Vec<_> = vertices.iter().map(|v| v.handle).collect();
        Cluster::hdbscan(&store, &handles, min_cluster_size, min_samples)
    }

    fn __repr__(&self) -> String {
        format!("Cluster(size={})", self.Size())
    }
}

/// Python wrapper for Dictionary
#[pyclass(name = "Dictionary")]
#[derive(Clone)]
pub struct PyDictionary {
    dict: Dictionary,
}

#[pymethods]
impl PyDictionary {
    /// Create a new empty dictionary
    #[new]
    fn new() -> Self {
        Self {
            dict: Dictionary::new(),
        }
    }

    /// Create a dictionary from a single key-value pair
    #[staticmethod]
    fn ByKeyValue(key: &str, value: PyObject, py: Python<'_>) -> PyResult<Self> {
        let mut dict = Dictionary::new();
        let dv = python_to_dict_value(&value, py)?;
        dict.set(key, dv);
        Ok(Self { dict })
    }

    /// Create a dictionary from keys and values
    #[staticmethod]
    fn ByKeysValues(keys: Vec<String>, values: Vec<PyObject>, py: Python<'_>) -> PyResult<Self> {
        let mut dict = Dictionary::new();
        for (key, value) in keys.iter().zip(values.iter()) {
            let dv = python_to_dict_value(value, py)?;
            dict.set(key, dv);
        }
        Ok(Self { dict })
    }

    /// Create a dictionary from a Python dict
    #[staticmethod]
    fn ByPythonDictionary(py_dict: Bound<'_, pyo3::types::PyDict>) -> PyResult<Self> {
        let mut dict = Dictionary::new();
        for (key, value) in py_dict.iter() {
            let key_str: String = key.extract()?;
            let dv = python_to_dict_value(&value.unbind(), key.py())?;
            dict.set(&key_str, dv);
        }
        Ok(Self { dict })
    }

    /// Get a value by key
    fn ValueAtKey(&self, key: &str, py: Python<'_>) -> Option<PyObject> {
        self.dict.get(key).map(|v| dict_value_to_python(v, py))
    }

    /// Set a value at a key
    fn SetValueAtKey(&mut self, key: &str, value: PyObject, py: Python<'_>) -> PyResult<()> {
        let dv = python_to_dict_value(&value, py)?;
        self.dict.set(key, dv);
        Ok(())
    }

    /// Get all keys
    fn Keys(&self) -> Vec<String> {
        self.dict.keys().cloned().collect()
    }

    /// Get all values
    fn Values(&self, py: Python<'_>) -> Vec<PyObject> {
        self.dict.values().map(|v| dict_value_to_python(v, py)).collect()
    }

    /// Get number of entries
    fn Len(&self) -> usize {
        self.dict.len()
    }

    /// Check if empty
    fn IsEmpty(&self) -> bool {
        self.dict.is_empty()
    }

    /// Check if key exists
    fn Contains(&self, key: &str) -> bool {
        self.dict.contains(key)
    }

    /// Remove a key
    fn RemoveKey(&mut self, key: &str) {
        self.dict.remove(key);
    }

    /// Merge with another dictionary (other values take precedence)
    fn Merge(&mut self, other: &PyDictionary) {
        self.dict.merge(&other.dict);
    }

    /// Create a copy of this dictionary
    fn Copy(&self) -> Self {
        Self {
            dict: self.dict.clone(),
        }
    }

    /// Convert to Python dict
    fn PythonDictionary(&self, py: Python<'_>) -> PyResult<PyObject> {
        let py_dict = pyo3::types::PyDict::new(py);
        for key in self.dict.keys() {
            if let Some(value) = self.dict.get(key) {
                py_dict.set_item(key, dict_value_to_python(value, py))?;
            }
        }
        Ok(py_dict.into())
    }

    /// Convert to JSON
    fn ToJSON(&self) -> PyResult<String> {
        self.dict.to_json().map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// Create from JSON
    #[staticmethod]
    fn FromJSON(json: &str) -> PyResult<Self> {
        let dict = Dictionary::from_json(json).map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(Self { dict })
    }

    fn __repr__(&self) -> String {
        format!("Dictionary(len={})", self.dict.len())
    }
}

/// Convert Python object to DictionaryValue
fn python_to_dict_value(obj: &PyObject, py: Python<'_>) -> PyResult<DictionaryValue> {
    if obj.is_none(py) {
        return Ok(DictionaryValue::Null);
    }

    if let Ok(v) = obj.extract::<bool>(py) {
        return Ok(DictionaryValue::Bool(v));
    }
    if let Ok(v) = obj.extract::<i64>(py) {
        return Ok(DictionaryValue::Int(v));
    }
    if let Ok(v) = obj.extract::<f64>(py) {
        return Ok(DictionaryValue::Float(v));
    }
    if let Ok(v) = obj.extract::<String>(py) {
        return Ok(DictionaryValue::String(v));
    }
    if let Ok(v) = obj.extract::<Vec<PyObject>>(py) {
        let list: Result<Vec<_>, _> = v.iter().map(|item| python_to_dict_value(item, py)).collect();
        return Ok(DictionaryValue::List(list?));
    }

    // Default to string representation
    Ok(DictionaryValue::String(obj.to_string()))
}

/// Convert DictionaryValue to Python object
fn dict_value_to_python(value: &DictionaryValue, py: Python<'_>) -> PyObject {
    match value {
        DictionaryValue::Null => py.None(),
        DictionaryValue::Bool(v) => v.into_py(py),
        DictionaryValue::Int(v) => v.into_py(py),
        DictionaryValue::Float(v) => v.into_py(py),
        DictionaryValue::String(v) => v.into_py(py),
        DictionaryValue::List(v) => {
            let items: Vec<_> = v.iter().map(|item| dict_value_to_python(item, py)).collect();
            items.into_py(py)
        }
        DictionaryValue::Dict(v) => {
            let py_dict = pyo3::types::PyDict::new(py);
            for (k, val) in v {
                let _ = py_dict.set_item(k, dict_value_to_python(val, py));
            }
            py_dict.into()
        }
    }
}

/// Python wrapper for Vector utility class
#[pyclass(name = "Vector")]
pub struct PyVector;

#[pymethods]
impl PyVector {
    /// Calculate angle between two vectors in degrees
    #[staticmethod]
    fn Angle(vec_a: Vec<f64>, vec_b: Vec<f64>) -> f64 {
        let a = [vec_a.get(0).copied().unwrap_or(0.0),
                 vec_a.get(1).copied().unwrap_or(0.0),
                 vec_a.get(2).copied().unwrap_or(0.0)];
        let b = [vec_b.get(0).copied().unwrap_or(0.0),
                 vec_b.get(1).copied().unwrap_or(0.0),
                 vec_b.get(2).copied().unwrap_or(0.0)];
        VectorUtil::angle(a, b)
    }

    /// Get azimuth and altitude of a vector
    #[staticmethod]
    #[pyo3(signature = (vec, mantissa=6))]
    fn AzimuthAltitude(vec: Vec<f64>, mantissa: u32, py: Python<'_>) -> PyResult<PyObject> {
        let v = [vec.get(0).copied().unwrap_or(0.0),
                 vec.get(1).copied().unwrap_or(0.0),
                 vec.get(2).copied().unwrap_or(0.0)];
        let (az, alt) = VectorUtil::azimuth_altitude(v, mantissa);
        let dict = pyo3::types::PyDict::new(py);
        dict.set_item("azimuth", az)?;
        dict.set_item("altitude", alt)?;
        Ok(dict.into())
    }

    /// Create a vector from azimuth and altitude
    #[staticmethod]
    fn ByAzimuthAltitude(azimuth: f64, altitude: f64) -> Vec<f64> {
        let v = VectorUtil::by_azimuth_altitude(azimuth, altitude);
        vec![v[0], v[1], v[2]]
    }

    /// Create a vector from coordinates
    #[staticmethod]
    #[pyo3(signature = (x=0.0, y=0.0, z=0.0))]
    fn ByCoordinates(x: f64, y: f64, z: f64) -> Vec<f64> {
        vec![x, y, z]
    }

    /// Create a vector from two vertices (end - start)
    #[staticmethod]
    fn ByVertices(start: &PyVertex, end: &PyVertex) -> Vec<f64> {
        let store = get_store().read();
        let s = Vertex::coordinates(&store, start.handle);
        let e = Vertex::coordinates(&store, end.handle);
        vec![e.0 - s.0, e.1 - s.1, e.2 - s.2]
    }

    /// Get the compass angle (from North, clockwise)
    #[staticmethod]
    #[pyo3(signature = (vec, mantissa=6))]
    fn CompassAngle(vec: Vec<f64>, mantissa: u32) -> f64 {
        let v = [vec.get(0).copied().unwrap_or(0.0),
                 vec.get(1).copied().unwrap_or(0.0),
                 vec.get(2).copied().unwrap_or(0.0)];
        VectorUtil::compass_angle(v, mantissa)
    }

    /// Get coordinates in various formats
    #[staticmethod]
    #[pyo3(signature = (vec, output_type="xyz"))]
    fn Coordinates(vec: Vec<f64>, output_type: &str) -> Vec<f64> {
        let v = [vec.get(0).copied().unwrap_or(0.0),
                 vec.get(1).copied().unwrap_or(0.0),
                 vec.get(2).copied().unwrap_or(0.0)];
        VectorUtil::coordinates(v, output_type)
    }

    /// Cross product of two vectors
    #[staticmethod]
    fn Cross(vec_a: Vec<f64>, vec_b: Vec<f64>) -> Vec<f64> {
        let a = [vec_a.get(0).copied().unwrap_or(0.0),
                 vec_a.get(1).copied().unwrap_or(0.0),
                 vec_a.get(2).copied().unwrap_or(0.0)];
        let b = [vec_b.get(0).copied().unwrap_or(0.0),
                 vec_b.get(1).copied().unwrap_or(0.0),
                 vec_b.get(2).copied().unwrap_or(0.0)];
        let r = VectorUtil::cross(a, b);
        vec![r[0], r[1], r[2]]
    }

    /// Dot product of two vectors
    #[staticmethod]
    fn Dot(vec_a: Vec<f64>, vec_b: Vec<f64>) -> f64 {
        let a = [vec_a.get(0).copied().unwrap_or(0.0),
                 vec_a.get(1).copied().unwrap_or(0.0),
                 vec_a.get(2).copied().unwrap_or(0.0)];
        let b = [vec_b.get(0).copied().unwrap_or(0.0),
                 vec_b.get(1).copied().unwrap_or(0.0),
                 vec_b.get(2).copied().unwrap_or(0.0)];
        VectorUtil::dot(a, b)
    }

    /// Get the Up vector
    #[staticmethod]
    fn Up() -> Vec<f64> {
        let v = VectorUtil::up();
        vec![v[0], v[1], v[2]]
    }

    /// Get the Down vector
    #[staticmethod]
    fn Down() -> Vec<f64> {
        let v = VectorUtil::down();
        vec![v[0], v[1], v[2]]
    }

    /// Get the North vector
    #[staticmethod]
    fn North() -> Vec<f64> {
        let v = VectorUtil::north();
        vec![v[0], v[1], v[2]]
    }

    /// Get the South vector
    #[staticmethod]
    fn South() -> Vec<f64> {
        let v = VectorUtil::south();
        vec![v[0], v[1], v[2]]
    }

    /// Get the East vector
    #[staticmethod]
    fn East() -> Vec<f64> {
        let v = VectorUtil::east();
        vec![v[0], v[1], v[2]]
    }

    /// Get the West vector
    #[staticmethod]
    fn West() -> Vec<f64> {
        let v = VectorUtil::west();
        vec![v[0], v[1], v[2]]
    }

    /// Check if two vectors are collinear
    #[staticmethod]
    #[pyo3(signature = (vec_a, vec_b, tolerance=0.0001))]
    fn IsCollinear(vec_a: Vec<f64>, vec_b: Vec<f64>, tolerance: f64) -> bool {
        let a = [vec_a.get(0).copied().unwrap_or(0.0),
                 vec_a.get(1).copied().unwrap_or(0.0),
                 vec_a.get(2).copied().unwrap_or(0.0)];
        let b = [vec_b.get(0).copied().unwrap_or(0.0),
                 vec_b.get(1).copied().unwrap_or(0.0),
                 vec_b.get(2).copied().unwrap_or(0.0)];
        VectorUtil::is_collinear(a, b, tolerance)
    }

    /// Get the magnitude of a vector
    #[staticmethod]
    fn Magnitude(vec: Vec<f64>) -> f64 {
        let v = [vec.get(0).copied().unwrap_or(0.0),
                 vec.get(1).copied().unwrap_or(0.0),
                 vec.get(2).copied().unwrap_or(0.0)];
        VectorUtil::magnitude(v)
    }

    /// Multiply a vector by a scalar
    #[staticmethod]
    fn Multiply(vec: Vec<f64>, scalar: f64) -> Vec<f64> {
        let v = [vec.get(0).copied().unwrap_or(0.0),
                 vec.get(1).copied().unwrap_or(0.0),
                 vec.get(2).copied().unwrap_or(0.0)];
        let r = VectorUtil::multiply(v, scalar);
        vec![r[0], r[1], r[2]]
    }

    /// Normalize a vector
    #[staticmethod]
    fn Normalize(vec: Vec<f64>) -> Vec<f64> {
        let v = [vec.get(0).copied().unwrap_or(0.0),
                 vec.get(1).copied().unwrap_or(0.0),
                 vec.get(2).copied().unwrap_or(0.0)];
        let r = VectorUtil::normalize(v);
        vec![r[0], r[1], r[2]]
    }

    /// Reverse a vector
    #[staticmethod]
    fn Reverse(vec: Vec<f64>) -> Vec<f64> {
        let v = [vec.get(0).copied().unwrap_or(0.0),
                 vec.get(1).copied().unwrap_or(0.0),
                 vec.get(2).copied().unwrap_or(0.0)];
        let r = VectorUtil::reverse(v);
        vec![r[0], r[1], r[2]]
    }

    /// Set the magnitude of a vector
    #[staticmethod]
    fn SetMagnitude(vec: Vec<f64>, magnitude: f64) -> Vec<f64> {
        let v = [vec.get(0).copied().unwrap_or(0.0),
                 vec.get(1).copied().unwrap_or(0.0),
                 vec.get(2).copied().unwrap_or(0.0)];
        let r = VectorUtil::set_magnitude(v, magnitude);
        vec![r[0], r[1], r[2]]
    }
}

/// Python wrapper for Matrix utility class
#[pyclass(name = "Matrix")]
pub struct PyMatrix;

#[pymethods]
impl PyMatrix {
    /// Create an identity matrix
    #[staticmethod]
    fn Identity() -> Vec<Vec<f64>> {
        let m = MatrixUtil::identity();
        m.iter().map(|row| row.to_vec()).collect()
    }

    /// Create a rotation matrix from Euler angles (degrees)
    #[staticmethod]
    #[pyo3(signature = (rx=0.0, ry=0.0, rz=0.0))]
    fn ByRotation(rx: f64, ry: f64, rz: f64) -> Vec<Vec<f64>> {
        let m = MatrixUtil::by_rotation(rx, ry, rz);
        m.iter().map(|row| row.to_vec()).collect()
    }

    /// Create a scaling matrix
    #[staticmethod]
    #[pyo3(signature = (sx=1.0, sy=1.0, sz=1.0))]
    fn ByScaling(sx: f64, sy: f64, sz: f64) -> Vec<Vec<f64>> {
        let m = MatrixUtil::by_scaling(sx, sy, sz);
        m.iter().map(|row| row.to_vec()).collect()
    }

    /// Create a translation matrix
    #[staticmethod]
    #[pyo3(signature = (tx=0.0, ty=0.0, tz=0.0))]
    fn ByTranslation(tx: f64, ty: f64, tz: f64) -> Vec<Vec<f64>> {
        let m = MatrixUtil::by_translation(tx, ty, tz);
        m.iter().map(|row| row.to_vec()).collect()
    }

    /// Add two matrices
    #[staticmethod]
    fn Add(a: Vec<Vec<f64>>, b: Vec<Vec<f64>>) -> PyResult<Vec<Vec<f64>>> {
        let ma = Self::vec_to_matrix(&a)?;
        let mb = Self::vec_to_matrix(&b)?;
        let result = MatrixUtil::add(ma, mb);
        Ok(result.iter().map(|row| row.to_vec()).collect())
    }

    /// Subtract two matrices
    #[staticmethod]
    fn Subtract(a: Vec<Vec<f64>>, b: Vec<Vec<f64>>) -> PyResult<Vec<Vec<f64>>> {
        let ma = Self::vec_to_matrix(&a)?;
        let mb = Self::vec_to_matrix(&b)?;
        let result = MatrixUtil::subtract(ma, mb);
        Ok(result.iter().map(|row| row.to_vec()).collect())
    }

    /// Multiply two matrices
    #[staticmethod]
    fn Multiply(a: Vec<Vec<f64>>, b: Vec<Vec<f64>>) -> PyResult<Vec<Vec<f64>>> {
        let ma = Self::vec_to_matrix(&a)?;
        let mb = Self::vec_to_matrix(&b)?;
        let result = MatrixUtil::multiply(ma, mb);
        Ok(result.iter().map(|row| row.to_vec()).collect())
    }

    /// Transpose a matrix
    #[staticmethod]
    fn Transpose(m: Vec<Vec<f64>>) -> PyResult<Vec<Vec<f64>>> {
        let matrix = Self::vec_to_matrix(&m)?;
        let result = MatrixUtil::transpose(matrix);
        Ok(result.iter().map(|row| row.to_vec()).collect())
    }
}

impl PyMatrix {
    fn vec_to_matrix(v: &Vec<Vec<f64>>) -> PyResult<Matrix4x4> {
        if v.len() != 4 {
            return Err(PyValueError::new_err("Matrix must have 4 rows"));
        }
        let mut result = [[0.0; 4]; 4];
        for (i, row) in v.iter().enumerate() {
            if row.len() != 4 {
                return Err(PyValueError::new_err("Each row must have 4 columns"));
            }
            for (j, &val) in row.iter().enumerate() {
                result[i][j] = val;
            }
        }
        Ok(result)
    }
}

/// Python wrapper for Grid utility class
#[pyclass(name = "Grid")]
pub struct PyGrid;

#[pymethods]
impl PyGrid {
    /// Generate grid edges by parameters
    #[staticmethod]
    #[pyo3(signature = (face=None, u_range=None, v_range=None, clip=true))]
    fn EdgesByParameters(
        face: Option<&PyFace>,
        u_range: Option<Vec<f64>>,
        v_range: Option<Vec<f64>>,
        clip: bool,
    ) -> PyResult<PyCluster> {
        let store = get_store().read();

        let default_face = Face::rectangle(&store, Point3::ZERO, 1.0, 1.0);
        let face_handle = face.map(|f| f.handle).unwrap_or(default_face);

        let u = u_range.unwrap_or_else(|| vec![0.0, 0.25, 0.5, 0.75, 1.0]);
        let v = v_range.unwrap_or_else(|| vec![0.0, 0.25, 0.5, 0.75, 1.0]);

        let cluster = GridUtil::edges_by_parameters(&store, face_handle, &u, &v, clip);
        Ok(PyCluster { handle: cluster })
    }

    /// Generate grid edges by distances
    #[staticmethod]
    #[pyo3(signature = (face=None, u_origin=None, v_origin=None, u_range=None, v_range=None, clip=true, tolerance=0.001))]
    fn EdgesByDistances(
        face: Option<&PyFace>,
        u_origin: Option<&PyVertex>,
        v_origin: Option<&PyVertex>,
        u_range: Option<Vec<f64>>,
        v_range: Option<Vec<f64>>,
        clip: bool,
        tolerance: f64,
    ) -> PyResult<PyCluster> {
        let store = get_store().read();

        let default_face = Face::rectangle(&store, Point3::ZERO, 1.0, 1.0);
        let face_handle = face.map(|f| f.handle).unwrap_or(default_face);

        let u = u_range.unwrap_or_else(|| vec![-0.5, -0.25, 0.0, 0.25, 0.5]);
        let v = v_range.unwrap_or_else(|| vec![-0.5, -0.25, 0.0, 0.25, 0.5]);

        let cluster = GridUtil::edges_by_distances(
            &store,
            face_handle,
            u_origin.map(|v| v.handle),
            v_origin.map(|v| v.handle),
            &u,
            &v,
            clip,
            tolerance,
        );
        Ok(PyCluster { handle: cluster })
    }

    /// Generate grid vertices by distances
    #[staticmethod]
    #[pyo3(signature = (face=None, origin=None, u_range=None, v_range=None, clip=true, tolerance=0.001))]
    fn VerticesByDistances(
        face: Option<&PyFace>,
        origin: Option<&PyVertex>,
        u_range: Option<Vec<f64>>,
        v_range: Option<Vec<f64>>,
        clip: bool,
        tolerance: f64,
    ) -> PyResult<PyCluster> {
        let store = get_store().read();

        let default_face = Face::rectangle(&store, Point3::ZERO, 1.0, 1.0);
        let face_handle = face.map(|f| f.handle).unwrap_or(default_face);

        let u = u_range.unwrap_or_else(|| vec![-0.5, -0.25, 0.0, 0.25, 0.5]);
        let v = v_range.unwrap_or_else(|| vec![-0.5, -0.25, 0.0, 0.25, 0.5]);

        let cluster = GridUtil::vertices_by_distances(
            &store,
            face_handle,
            origin.map(|v| v.handle),
            &u,
            &v,
            clip,
            tolerance,
        );
        Ok(PyCluster { handle: cluster })
    }
}

/// Python wrapper for Graph utility class
#[pyclass(name = "Graph")]
#[derive(Clone)]
pub struct PyGraph {
    handle: GraphHandle,
}

#[pymethods]
impl PyGraph {
    /// Create a graph from vertices and edges
    #[staticmethod]
    fn ByVerticesEdges(vertices: Vec<PyVertex>, edges: Vec<PyEdge>) -> PyResult<Self> {
        let store = get_store().read();
        let v_handles: Vec<_> = vertices.iter().map(|v| v.handle).collect();
        let e_handles: Vec<_> = edges.iter().map(|e| e.handle).collect();
        Ok(Self {
            handle: GraphUtil::by_vertices_edges(&store, v_handles, e_handles),
        })
    }

    /// Create a graph from a topology (dual graph)
    ///
    /// For CellComplex: Creates a vertex for each Cell, edges between Cells that share a Face
    /// For Cell: Creates a single vertex at the Cell's centroid
    /// For Shell: Creates a vertex for each Face, edges between Faces that share an Edge
    /// For Face: Creates a single vertex at the Face's centroid
    /// For Wire: Creates a vertex for each Edge, edges between Edges that share a Vertex
    /// For Edge: Creates a single vertex at the Edge's midpoint
    /// For Vertex: Uses the vertex directly
    #[staticmethod]
    #[pyo3(signature = (topology, direct=true, via_shared_topologies=false, to_exterior_topologies=false, tolerance=0.0001))]
    fn ByTopology(
        topology: &Bound<'_, PyAny>,
        direct: bool,
        via_shared_topologies: bool,
        to_exterior_topologies: bool,
        tolerance: f64,
    ) -> PyResult<Self> {
        let store = get_store().read();

        // Determine the topology type and create the appropriate handle
        let topo_handle = if let Ok(cc) = topology.extract::<PyCellComplex>() {
            TopologyHandle::CellComplex(cc.handle)
        } else if let Ok(cell) = topology.extract::<PyCell>() {
            TopologyHandle::Cell(cell.handle)
        } else if let Ok(shell) = topology.extract::<PyShell>() {
            TopologyHandle::Shell(shell.handle)
        } else if let Ok(face) = topology.extract::<PyFace>() {
            TopologyHandle::Face(face.handle)
        } else if let Ok(wire) = topology.extract::<PyWire>() {
            TopologyHandle::Wire(wire.handle)
        } else if let Ok(edge) = topology.extract::<PyEdge>() {
            TopologyHandle::Edge(edge.handle)
        } else if let Ok(vertex) = topology.extract::<PyVertex>() {
            TopologyHandle::Vertex(vertex.handle)
        } else if let Ok(cluster) = topology.extract::<PyCluster>() {
            TopologyHandle::Cluster(cluster.handle)
        } else {
            return Err(PyValueError::new_err("Unsupported topology type for Graph.ByTopology"));
        };

        Ok(Self {
            handle: GraphUtil::by_topology(&store, topo_handle, direct, via_shared_topologies, to_exterior_topologies, tolerance),
        })
    }

    /// Add an edge to the graph
    fn AddEdge(&self, edge: &PyEdge) -> PyResult<Self> {
        let store = get_store().read();
        Ok(Self {
            handle: GraphUtil::add_edge(&store, self.handle, edge.handle),
        })
    }

    /// Add a vertex to the graph
    fn AddVertex(&self, vertex: &PyVertex) -> PyResult<Self> {
        let store = get_store().read();
        Ok(Self {
            handle: GraphUtil::add_vertex(&store, self.handle, vertex.handle),
        })
    }

    /// Get adjacency list
    #[pyo3(signature = (tolerance=0.001))]
    fn AdjacencyList(&self, tolerance: f64) -> Vec<Vec<usize>> {
        let store = get_store().read();
        GraphUtil::adjacency_list(&store, self.handle, tolerance)
    }

    /// Get adjacency matrix
    #[pyo3(signature = (tolerance=0.001))]
    fn AdjacencyMatrix(&self, tolerance: f64) -> Vec<Vec<i32>> {
        let store = get_store().read();
        GraphUtil::adjacency_matrix(&store, self.handle, tolerance)
    }

    /// Get adjacent vertices
    #[pyo3(signature = (vertex, tolerance=0.001))]
    fn AdjacentVertices(&self, vertex: &PyVertex, tolerance: f64) -> Vec<PyVertex> {
        let store = get_store().read();
        GraphUtil::adjacent_vertices(&store, self.handle, vertex.handle, tolerance)
            .into_iter()
            .map(|h| PyVertex { handle: h })
            .collect()
    }

    /// Check if graph contains an edge
    #[pyo3(signature = (edge, tolerance=0.001))]
    fn ContainsEdge(&self, edge: &PyEdge, tolerance: f64) -> bool {
        let store = get_store().read();
        GraphUtil::contains_edge(&store, self.handle, edge.handle, tolerance)
    }

    /// Check if graph contains a vertex
    #[pyo3(signature = (vertex, tolerance=0.001))]
    fn ContainsVertex(&self, vertex: &PyVertex, tolerance: f64) -> bool {
        let store = get_store().read();
        GraphUtil::contains_vertex(&store, self.handle, vertex.handle, tolerance)
    }

    /// Get degree sequence
    #[pyo3(signature = (tolerance=0.001))]
    fn DegreeSequence(&self, tolerance: f64) -> Vec<usize> {
        let store = get_store().read();
        GraphUtil::degree_sequence(&store, self.handle, tolerance)
    }

    /// Get graph density
    fn Density(&self) -> f64 {
        let store = get_store().read();
        GraphUtil::density(&store, self.handle)
    }

    /// Get depth map from source
    #[pyo3(signature = (source, tolerance=0.001))]
    fn DepthMap(&self, source: &PyVertex, tolerance: f64) -> Vec<i32> {
        let store = get_store().read();
        GraphUtil::depth_map(&store, self.handle, source.handle, tolerance)
    }

    /// Get graph diameter
    #[pyo3(signature = (tolerance=0.001))]
    fn Diameter(&self, tolerance: f64) -> i32 {
        let store = get_store().read();
        GraphUtil::diameter(&store, self.handle, tolerance)
    }

    /// Get distance between two vertices
    #[pyo3(signature = (v1, v2, tolerance=0.001))]
    fn Distance(&self, v1: &PyVertex, v2: &PyVertex, tolerance: f64) -> Option<i32> {
        let store = get_store().read();
        GraphUtil::distance(&store, self.handle, v1.handle, v2.handle, tolerance)
    }

    /// Get edges
    fn Edges(&self) -> Vec<PyEdge> {
        let store = get_store().read();
        GraphUtil::edges(&store, self.handle)
            .into_iter()
            .map(|h| PyEdge { handle: h })
            .collect()
    }

    /// Get vertices
    fn Vertices(&self) -> Vec<PyVertex> {
        let store = get_store().read();
        GraphUtil::vertices(&store, self.handle)
            .into_iter()
            .map(|h| PyVertex { handle: h })
            .collect()
    }

    /// Check if bipartite
    #[pyo3(signature = (tolerance=0.001))]
    fn IsBipartite(&self, tolerance: f64) -> bool {
        let store = get_store().read();
        GraphUtil::is_bipartite(&store, self.handle, tolerance)
    }

    /// Check if complete
    fn IsComplete(&self) -> bool {
        let store = get_store().read();
        GraphUtil::is_complete(&store, self.handle)
    }

    /// Get isolated vertices
    #[pyo3(signature = (tolerance=0.001))]
    fn IsolatedVertices(&self, tolerance: f64) -> Vec<PyVertex> {
        let store = get_store().read();
        GraphUtil::isolated_vertices(&store, self.handle, tolerance)
            .into_iter()
            .map(|h| PyVertex { handle: h })
            .collect()
    }

    /// Get maximum degree
    #[pyo3(signature = (tolerance=0.001))]
    fn MaximumDelta(&self, tolerance: f64) -> usize {
        let store = get_store().read();
        GraphUtil::maximum_delta(&store, self.handle, tolerance)
    }

    /// Get minimum degree
    #[pyo3(signature = (tolerance=0.001))]
    fn MinimumDelta(&self, tolerance: f64) -> usize {
        let store = get_store().read();
        GraphUtil::minimum_delta(&store, self.handle, tolerance)
    }

    /// Find nearest vertex
    fn NearestVertex(&self, target: &PyVertex) -> Option<PyVertex> {
        let store = get_store().read();
        GraphUtil::nearest_vertex(&store, self.handle, target.handle)
            .map(|h| PyVertex { handle: h })
    }

    /// Get order (number of vertices)
    fn Order(&self) -> usize {
        let store = get_store().read();
        GraphUtil::order(&store, self.handle)
    }

    /// Get size (number of edges)
    fn Size(&self) -> usize {
        let store = get_store().read();
        GraphUtil::size(&store, self.handle)
    }

    /// Find shortest path as wire
    #[pyo3(signature = (start, end, tolerance=0.001))]
    fn Path(&self, start: &PyVertex, end: &PyVertex, tolerance: f64) -> Option<PyWire> {
        let store = get_store().read();
        GraphUtil::shortest_path(&store, self.handle, start.handle, end.handle, tolerance)
            .map(|h| PyWire { handle: h })
    }

    /// Remove an edge
    #[pyo3(signature = (edge, tolerance=0.001))]
    fn RemoveEdge(&self, edge: &PyEdge, tolerance: f64) -> Self {
        let store = get_store().read();
        Self {
            handle: GraphUtil::remove_edge(&store, self.handle, edge.handle, tolerance),
        }
    }

    /// Remove a vertex
    #[pyo3(signature = (vertex, tolerance=0.001))]
    fn RemoveVertex(&self, vertex: &PyVertex, tolerance: f64) -> Self {
        let store = get_store().read();
        Self {
            handle: GraphUtil::remove_vertex(&store, self.handle, vertex.handle, tolerance),
        }
    }

    /// Get vertex degree
    #[pyo3(signature = (vertex, tolerance=0.001))]
    fn VertexDegree(&self, vertex: &PyVertex, tolerance: f64) -> usize {
        let store = get_store().read();
        GraphUtil::vertex_degree(&store, self.handle, vertex.handle, tolerance)
    }

    /// Convert to topology (cluster)
    fn Topology(&self) -> PyCluster {
        let store = get_store().read();
        PyCluster {
            handle: GraphUtil::topology(&store, self.handle),
        }
    }

    /// Create a graph from an adjacency matrix
    #[staticmethod]
    #[pyo3(signature = (matrix, vertices=None))]
    fn ByAdjacencyMatrix(matrix: Vec<Vec<i32>>, vertices: Option<Vec<PyVertex>>) -> PyResult<Self> {
        let store = get_store().read();
        let v_handles = vertices.map(|v| v.iter().map(|pv| pv.handle).collect());
        Ok(Self {
            handle: GraphUtil::by_adjacency_matrix(&store, &matrix, v_handles),
        })
    }

    /// Calculate betweenness centrality for all vertices
    #[pyo3(signature = (tolerance=0.001, normalized=true))]
    fn BetweennessCentrality(&self, tolerance: f64, normalized: bool) -> Vec<f64> {
        let store = get_store().read();
        GraphUtil::betweenness_centrality(&store, self.handle, tolerance, normalized)
    }

    /// Calculate PageRank for all vertices
    #[pyo3(signature = (tolerance=0.001, damping=0.85, max_iterations=100, convergence=0.0001))]
    fn PageRank(&self, tolerance: f64, damping: f64, max_iterations: usize, convergence: f64) -> Vec<f64> {
        let store = get_store().read();
        GraphUtil::page_rank(&store, self.handle, tolerance, damping, max_iterations, convergence)
    }

    /// Find bridge edges (edges whose removal disconnects the graph)
    #[pyo3(signature = (tolerance=0.001))]
    fn Bridges(&self, tolerance: f64) -> Vec<PyEdge> {
        let store = get_store().read();
        GraphUtil::bridges(&store, self.handle, tolerance)
            .into_iter()
            .map(|h| PyEdge { handle: h })
            .collect()
    }

    /// Find cut vertices (articulation points)
    #[pyo3(signature = (tolerance=0.001))]
    fn CutVertices(&self, tolerance: f64) -> Vec<PyVertex> {
        let store = get_store().read();
        GraphUtil::cut_vertices(&store, self.handle, tolerance)
            .into_iter()
            .map(|h| PyVertex { handle: h })
            .collect()
    }

    /// Reshape graph using force-directed or other layout algorithms
    #[pyo3(signature = (shape="spring 2d", k=None, iterations=None, root_vertex=None, tolerance=0.001))]
    fn Reshape(
        &self,
        shape: &str,
        k: Option<f64>,
        iterations: Option<usize>,
        root_vertex: Option<&PyVertex>,
        tolerance: f64,
    ) -> Self {
        let store = get_store().read();
        Self {
            handle: GraphUtil::reshape(
                &store,
                self.handle,
                tolerance,
                shape,
                k,
                iterations,
                root_vertex.map(|v| v.handle),
            ),
        }
    }

    /// Union of two graphs
    #[staticmethod]
    #[pyo3(signature = (graph1, graph2, tolerance=0.001))]
    fn Union(graph1: &PyGraph, graph2: &PyGraph, tolerance: f64) -> Self {
        let store = get_store().read();
        Self {
            handle: GraphUtil::union(&store, graph1.handle, graph2.handle, tolerance),
        }
    }

    /// Intersection of two graphs
    #[staticmethod]
    #[pyo3(signature = (graph1, graph2, tolerance=0.001))]
    fn Intersect(graph1: &PyGraph, graph2: &PyGraph, tolerance: f64) -> Self {
        let store = get_store().read();
        Self {
            handle: GraphUtil::intersect(&store, graph1.handle, graph2.handle, tolerance),
        }
    }

    /// Difference of two graphs (graph1 - graph2)
    #[staticmethod]
    #[pyo3(signature = (graph1, graph2, tolerance=0.001))]
    fn Difference(graph1: &PyGraph, graph2: &PyGraph, tolerance: f64) -> Self {
        let store = get_store().read();
        Self {
            handle: GraphUtil::difference(&store, graph1.handle, graph2.handle, tolerance),
        }
    }

    /// Symmetric difference of two graphs (XOR)
    #[staticmethod]
    #[pyo3(signature = (graph1, graph2, tolerance=0.001))]
    fn SymmetricDifference(graph1: &PyGraph, graph2: &PyGraph, tolerance: f64) -> Self {
        let store = get_store().read();
        Self {
            handle: GraphUtil::symmetric_difference(&store, graph1.handle, graph2.handle, tolerance),
        }
    }

    /// Create a line graph (edge-to-vertex dual)
    #[pyo3(signature = (tolerance=0.001))]
    fn LineGraph(&self, tolerance: f64) -> Self {
        let store = get_store().read();
        Self {
            handle: GraphUtil::line_graph(&store, self.handle, tolerance),
        }
    }

    /// Create a quotient graph by contracting vertices in the same partition
    #[pyo3(signature = (partition, tolerance=0.001))]
    fn Quotient(&self, partition: Vec<usize>, tolerance: f64) -> Self {
        let store = get_store().read();
        Self {
            handle: GraphUtil::quotient(&store, self.handle, &partition, tolerance),
        }
    }

    /// Check if the graph is connected
    #[pyo3(signature = (tolerance=0.001))]
    fn IsConnected(&self, tolerance: f64) -> bool {
        let store = get_store().read();
        GraphUtil::is_connected(&store, self.handle, tolerance)
    }

    /// Get connected components
    #[pyo3(signature = (tolerance=0.001))]
    fn ConnectedComponents(&self, tolerance: f64) -> Vec<Vec<PyVertex>> {
        let store = get_store().read();
        GraphUtil::connected_components(&store, self.handle, tolerance)
            .into_iter()
            .map(|component| {
                component.into_iter().map(|h| PyVertex { handle: h }).collect()
            })
            .collect()
    }

    /// Partition the graph into communities
    #[pyo3(signature = (method="community", num_partitions=None, tolerance=0.001))]
    fn Partition(&self, method: &str, num_partitions: Option<usize>, tolerance: f64) -> Vec<usize> {
        let store = get_store().read();
        GraphUtil::partition(&store, self.handle, tolerance, method, num_partitions)
    }

    /// Get the eccentricity of each vertex
    #[pyo3(signature = (tolerance=0.001))]
    fn Eccentricity(&self, tolerance: f64) -> Vec<i32> {
        let store = get_store().read();
        GraphUtil::eccentricity(&store, self.handle, tolerance)
    }

    /// Get the radius of the graph
    #[pyo3(signature = (tolerance=0.001))]
    fn Radius(&self, tolerance: f64) -> i32 {
        let store = get_store().read();
        GraphUtil::radius(&store, self.handle, tolerance)
    }

    /// Get the center vertices
    #[pyo3(signature = (tolerance=0.001))]
    fn Center(&self, tolerance: f64) -> Vec<PyVertex> {
        let store = get_store().read();
        GraphUtil::center(&store, self.handle, tolerance)
            .into_iter()
            .map(|h| PyVertex { handle: h })
            .collect()
    }

    /// Get the periphery vertices
    #[pyo3(signature = (tolerance=0.001))]
    fn Periphery(&self, tolerance: f64) -> Vec<PyVertex> {
        let store = get_store().read();
        GraphUtil::periphery(&store, self.handle, tolerance)
            .into_iter()
            .map(|h| PyVertex { handle: h })
            .collect()
    }

    /// Get the closeness centrality of all vertices
    #[pyo3(signature = (tolerance=0.001))]
    fn ClosenessCentrality(&self, tolerance: f64) -> Vec<f64> {
        let store = get_store().read();
        GraphUtil::closeness_centrality(&store, self.handle, tolerance)
    }

    /// Get the degree centrality of all vertices
    #[pyo3(signature = (tolerance=0.001))]
    fn DegreeCentrality(&self, tolerance: f64) -> Vec<f64> {
        let store = get_store().read();
        GraphUtil::degree_centrality(&store, self.handle, tolerance)
    }

    /// Partition the graph into communities and return as groups of vertices
    /// This matches topologicpy's Graph.CommunityPartition() API
    #[pyo3(signature = (method=None, tolerance=0.001))]
    fn CommunityPartition(&self, method: Option<&str>, tolerance: f64) -> Vec<Vec<PyVertex>> {
        let store = get_store().read();
        GraphUtil::community_partition(&store, self.handle, tolerance, method)
            .into_iter()
            .map(|community| {
                community.into_iter().map(|h| PyVertex { handle: h }).collect()
            })
            .collect()
    }

    /// Louvain community detection algorithm
    #[pyo3(signature = (tolerance=0.001, resolution=1.0, max_iterations=100))]
    fn Louvain(&self, tolerance: f64, resolution: f64, max_iterations: usize) -> Vec<usize> {
        let store = get_store().read();
        GraphUtil::louvain(&store, self.handle, tolerance, resolution, max_iterations)
    }

    /// Get modularity score for a partition
    #[pyo3(signature = (partition, tolerance=0.001))]
    fn Modularity(&self, partition: Vec<usize>, tolerance: f64) -> f64 {
        let store = get_store().read();
        GraphUtil::modularity(&store, self.handle, &partition, tolerance)
    }

    fn __repr__(&self) -> String {
        let store = get_store().read();
        format!(
            "Graph(vertices={}, edges={})",
            GraphUtil::order(&store, self.handle),
            GraphUtil::size(&store, self.handle)
        )
    }
}

/// Clear all topology data
#[pyfunction]
fn clear_store() {
    get_store().write().clear();
}

/// Get store statistics
#[pyfunction]
fn store_stats() -> (usize, usize, usize, usize, usize, usize, usize, usize) {
    let stats = get_store().read().stats();
    (
        stats.vertices,
        stats.edges,
        stats.wires,
        stats.faces,
        stats.shells,
        stats.cells,
        stats.cell_complexes,
        stats.clusters,
    )
}

/// Python module
#[pymodule]
fn topologic_fast(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyVertex>()?;
    m.add_class::<PyEdge>()?;
    m.add_class::<PyWire>()?;
    m.add_class::<PyFace>()?;
    m.add_class::<PyShell>()?;
    m.add_class::<PyCell>()?;
    m.add_class::<PyCellComplex>()?;
    m.add_class::<PyCluster>()?;
    m.add_class::<PyTopology>()?;
    m.add_class::<PyDictionary>()?;
    m.add_class::<PyVector>()?;
    m.add_class::<PyMatrix>()?;
    m.add_class::<PyGrid>()?;
    m.add_class::<PyGraph>()?;
    m.add_class::<PyMesh>()?;
    m.add_function(wrap_pyfunction!(clear_store, m)?)?;
    m.add_function(wrap_pyfunction!(store_stats, m)?)?;
    Ok(())
}
