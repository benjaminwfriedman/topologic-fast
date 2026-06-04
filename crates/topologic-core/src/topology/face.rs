//! Face topology (2-dimensional surface)

use super::*;
use crate::geometry::{Point3, Vector3, TOLERANCE, triangle_area, triangle_normal};
use smallvec::SmallVec;
use rayon::prelude::*;

/// Handle to a face in the topology store
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FaceHandle {
    pub(crate) index: ArenaIndex,
    pub(crate) id: TopologyId,
}

impl FaceHandle {
    pub fn id(&self) -> TopologyId {
        self.id
    }
}

/// Internal face data
pub(crate) struct FaceData {
    pub id: TopologyId,
    pub outer_wire: WireHandle,
    pub inner_wires: Vec<WireHandle>,
    pub shells: SmallVec<[ShellHandle; 2]>,
    pub cells: SmallVec<[CellHandle; 2]>,
    pub dictionary: Dictionary,
    pub normal_cache: Option<Vector3>,
}

/// Face operations
pub struct Face;

impl Face {
    /// Create a face from an outer wire
    pub fn by_external_boundary(store: &TopologyStore, wire: WireHandle) -> FaceHandle {
        store.add_face(wire, vec![])
    }

    /// Create a face from outer wire and inner wires (holes)
    pub fn by_external_internal_boundaries(
        store: &TopologyStore,
        outer: WireHandle,
        inner: Vec<WireHandle>,
    ) -> FaceHandle {
        store.add_face(outer, inner)
    }

    /// Create a rectangular face
    pub fn rectangle(
        store: &TopologyStore,
        origin: Point3,
        width: f64,
        height: f64,
    ) -> FaceHandle {
        let wire = Wire::rectangle(store, origin, width, height);
        Self::by_external_boundary(store, wire)
    }

    /// Create a circular face
    pub fn circle(
        store: &TopologyStore,
        center: Point3,
        radius: f64,
        segments: usize,
    ) -> FaceHandle {
        let wire = Wire::circle(store, center, radius, segments);
        Self::by_external_boundary(store, wire)
    }

    /// Create a face from vertices (creates wire automatically)
    pub fn by_vertices(store: &TopologyStore, vertices: Vec<VertexHandle>) -> FaceHandle {
        let wire = Wire::by_vertices(store, vertices, true);
        Self::by_external_boundary(store, wire)
    }

    /// Get the outer wire
    pub fn external_boundary(store: &TopologyStore, handle: FaceHandle) -> WireHandle {
        store.faces.read()[handle.index.index()].outer_wire
    }

    /// Get inner wires (holes)
    pub fn internal_boundaries(store: &TopologyStore, handle: FaceHandle) -> Vec<WireHandle> {
        store.faces.read()[handle.index.index()].inner_wires.clone()
    }

    /// Get all wires (outer + inner)
    pub fn wires(store: &TopologyStore, handle: FaceHandle) -> Vec<WireHandle> {
        let faces = store.faces.read();
        let face = &faces[handle.index.index()];
        let mut wires = vec![face.outer_wire];
        wires.extend(face.inner_wires.iter().copied());
        wires
    }

    /// Get all edges in the face
    pub fn edges(store: &TopologyStore, handle: FaceHandle) -> Vec<EdgeHandle> {
        let wires = Self::wires(store, handle);
        wires.iter().flat_map(|w| Wire::edges(store, *w)).collect()
    }

    /// Get all vertices in the face
    pub fn vertices(store: &TopologyStore, handle: FaceHandle) -> Vec<VertexHandle> {
        let wires = Self::wires(store, handle);
        let mut seen = hashbrown::HashSet::new();
        wires
            .iter()
            .flat_map(|w| Wire::vertices(store, *w))
            .filter(|v| seen.insert(*v))
            .collect()
    }

    /// Get shells containing this face
    pub fn shells(store: &TopologyStore, handle: FaceHandle) -> Vec<ShellHandle> {
        store.faces.read()[handle.index.index()].shells.to_vec()
    }

    /// Get cells containing this face
    pub fn cells(store: &TopologyStore, handle: FaceHandle) -> Vec<CellHandle> {
        store.faces.read()[handle.index.index()].cells.to_vec()
    }

    /// Get the face normal (cached)
    pub fn normal(store: &TopologyStore, handle: FaceHandle) -> Option<Vector3> {
        // Check cache first
        {
            let faces = store.faces.read();
            if let Some(normal) = faces[handle.index.index()].normal_cache {
                return Some(normal);
            }
        }

        // Calculate normal from outer wire
        let outer = Self::external_boundary(store, handle);
        let normal = Wire::normal(store, outer)?;

        // Cache the result
        {
            let mut faces = store.faces.write();
            faces[handle.index.index()].normal_cache = Some(normal);
        }

        Some(normal)
    }

    /// Get the area of the face
    pub fn area(store: &TopologyStore, handle: FaceHandle) -> f64 {
        let outer = Self::external_boundary(store, handle);
        let outer_area = Wire::area(store, outer);

        let inner_area: f64 = Self::internal_boundaries(store, handle)
            .iter()
            .map(|w| Wire::area(store, *w))
            .sum();

        outer_area - inner_area
    }

    /// Get the center of mass of the face
    pub fn center_of_mass(store: &TopologyStore, handle: FaceHandle) -> Point3 {
        let vertices = Self::vertices(store, handle);
        if vertices.is_empty() {
            return Point3::ZERO;
        }

        let points: Vec<_> = vertices.iter().map(|v| Vertex::point(store, *v)).collect();
        crate::geometry::centroid(&points)
    }

    /// Get the perimeter of the face
    pub fn perimeter(store: &TopologyStore, handle: FaceHandle) -> f64 {
        Self::wires(store, handle)
            .iter()
            .map(|w| Wire::length(store, *w))
            .sum()
    }

    /// Get the bounding box of the face
    pub fn bounding_box(store: &TopologyStore, handle: FaceHandle) -> (Point3, Point3) {
        let vertices = Self::vertices(store, handle);
        if vertices.is_empty() {
            return (Point3::ZERO, Point3::ZERO);
        }

        let points: Vec<_> = vertices.iter().map(|v| Vertex::point(store, *v)).collect();
        crate::geometry::bounding_box(&points).unwrap_or((Point3::ZERO, Point3::ZERO))
    }

    /// Check if the face is planar
    pub fn is_planar(store: &TopologyStore, handle: FaceHandle) -> bool {
        let outer = Self::external_boundary(store, handle);
        Wire::is_planar(store, outer)
    }

    /// Check if a point lies on the face
    pub fn contains_point(store: &TopologyStore, handle: FaceHandle, point: Point3) -> bool {
        let normal = match Self::normal(store, handle) {
            Some(n) => n,
            None => return false,
        };

        let center = Self::center_of_mass(store, handle);

        // Check if point is on the face plane
        let d = (point - center).dot(normal);
        if d.abs() > TOLERANCE {
            return false;
        }

        // Ray casting for point-in-polygon
        let vertices = Self::vertices(store, handle);
        let points: Vec<_> = vertices.iter().map(|v| Vertex::point(store, *v)).collect();

        // Project to 2D for ray casting
        let (u_axis, v_axis) = {
            let u = if normal.x.abs() < 0.9 {
                Vector3::X.cross(normal).normalize()
            } else {
                Vector3::Y.cross(normal).normalize()
            };
            let v = normal.cross(u);
            (u, v)
        };

        let project = |p: Point3| -> (f64, f64) {
            let d = p - center;
            (d.dot(u_axis), d.dot(v_axis))
        };

        let test = project(point);
        let polygon: Vec<_> = points.iter().map(|p| project(*p)).collect();

        // Ray casting algorithm
        let mut inside = false;
        let n = polygon.len();
        for i in 0..n {
            let j = (i + 1) % n;
            let (xi, yi) = polygon[i];
            let (xj, yj) = polygon[j];

            if ((yi > test.1) != (yj > test.1))
                && (test.0 < (xj - xi) * (test.1 - yi) / (yj - yi) + xi)
            {
                inside = !inside;
            }
        }

        inside
    }

    /// Triangulate the face
    pub fn triangulate(store: &TopologyStore, handle: FaceHandle) -> Vec<(VertexHandle, VertexHandle, VertexHandle)> {
        let vertices = Self::vertices(store, handle);
        if vertices.len() < 3 {
            return vec![];
        }

        // Simple ear-clipping triangulation for convex/simple polygons
        let mut remaining: Vec<_> = vertices.clone();
        let mut triangles = Vec::new();

        while remaining.len() > 3 {
            let n = remaining.len();
            let mut found_ear = false;

            for i in 0..n {
                let prev = (i + n - 1) % n;
                let next = (i + 1) % n;

                // Check if this is an ear
                let p0 = Vertex::point(store, remaining[prev]);
                let p1 = Vertex::point(store, remaining[i]);
                let p2 = Vertex::point(store, remaining[next]);

                // Check if convex
                let v1 = p1 - p0;
                let v2 = p2 - p1;
                let cross = v1.cross(v2);

                if let Some(normal) = Self::normal(store, handle) {
                    if cross.dot(normal) > 0.0 {
                        // Check if any other vertex is inside this triangle
                        let mut is_ear = true;
                        for (j, v) in remaining.iter().enumerate() {
                            if j != prev && j != i && j != next {
                                let p = Vertex::point(store, *v);
                                if Self::point_in_triangle(p, p0, p1, p2) {
                                    is_ear = false;
                                    break;
                                }
                            }
                        }

                        if is_ear {
                            triangles.push((remaining[prev], remaining[i], remaining[next]));
                            remaining.remove(i);
                            found_ear = true;
                            break;
                        }
                    }
                }
            }

            if !found_ear {
                break; // Avoid infinite loop for non-simple polygons
            }
        }

        if remaining.len() == 3 {
            triangles.push((remaining[0], remaining[1], remaining[2]));
        }

        triangles
    }

    fn point_in_triangle(p: Point3, a: Point3, b: Point3, c: Point3) -> bool {
        let v0 = c - a;
        let v1 = b - a;
        let v2 = p - a;

        let dot00 = v0.dot(v0);
        let dot01 = v0.dot(v1);
        let dot02 = v0.dot(v2);
        let dot11 = v1.dot(v1);
        let dot12 = v1.dot(v2);

        let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
        let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
        let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

        u >= 0.0 && v >= 0.0 && (u + v) <= 1.0
    }

    /// Get adjacent faces (sharing an edge)
    pub fn adjacent_faces(store: &TopologyStore, handle: FaceHandle) -> Vec<FaceHandle> {
        let edges = Self::edges(store, handle);
        let mut adjacent = Vec::new();

        for edge in edges {
            for face in Edge::faces(store, edge) {
                if face != handle && !adjacent.contains(&face) {
                    adjacent.push(face);
                }
            }
        }

        adjacent
    }

    /// Get shared edges between two faces
    pub fn shared_edges(store: &TopologyStore, f1: FaceHandle, f2: FaceHandle) -> Vec<EdgeHandle> {
        let edges1: hashbrown::HashSet<_> = Self::edges(store, f1).into_iter().collect();
        let edges2: hashbrown::HashSet<_> = Self::edges(store, f2).into_iter().collect();
        edges1.intersection(&edges2).copied().collect()
    }

    /// Check if the face is manifold
    pub fn is_manifold(store: &TopologyStore, handle: FaceHandle) -> bool {
        let edges = Self::edges(store, handle);
        edges.iter().all(|e| Edge::is_manifold(store, *e))
    }

    /// Flip the face normal
    pub fn flip(store: &TopologyStore, handle: FaceHandle) -> FaceHandle {
        let outer = Self::external_boundary(store, handle);
        let inner = Self::internal_boundaries(store, handle);

        let reversed_outer = Wire::reverse(store, outer);
        let reversed_inner: Vec<_> = inner.iter().map(|w| Wire::reverse(store, *w)).collect();

        Self::by_external_internal_boundaries(store, reversed_outer, reversed_inner)
    }

    /// Invert the face (same as flip)
    pub fn invert(store: &TopologyStore, handle: FaceHandle) -> FaceHandle {
        Self::flip(store, handle)
    }

    /// Create a face from a wire (alias for by_external_boundary)
    pub fn by_wire(store: &TopologyStore, wire: WireHandle) -> FaceHandle {
        Self::by_external_boundary(store, wire)
    }

    /// Create a face from multiple wires (first is external, rest are internal)
    pub fn by_wires(store: &TopologyStore, wires: Vec<WireHandle>) -> Option<FaceHandle> {
        if wires.is_empty() {
            return None;
        }
        let outer = wires[0];
        let inner = wires[1..].to_vec();
        Some(Self::by_external_internal_boundaries(store, outer, inner))
    }

    /// Create a square face
    pub fn square(
        store: &TopologyStore,
        origin: Point3,
        size: f64,
    ) -> FaceHandle {
        Self::rectangle(store, origin, size, size)
    }

    /// Create an ellipse face
    pub fn ellipse(
        store: &TopologyStore,
        center: Point3,
        width: f64,
        length: f64,
        segments: usize,
    ) -> FaceHandle {
        let wire = Wire::ellipse(store, center, width, length, segments, 0.0, 360.0, true);
        Self::by_external_boundary(store, wire)
    }

    /// Create a star face
    pub fn star(
        store: &TopologyStore,
        center: Point3,
        outer_radius: f64,
        inner_radius: f64,
        rays: usize,
    ) -> FaceHandle {
        let wire = Wire::star(store, center, outer_radius, inner_radius, rays);
        Self::by_external_boundary(store, wire)
    }

    /// Create a trapezoid face
    pub fn trapezoid(
        store: &TopologyStore,
        origin: Point3,
        width_a: f64,
        width_b: f64,
        offset_a: f64,
        offset_b: f64,
        length: f64,
    ) -> FaceHandle {
        let wire = Wire::trapezoid(store, origin, width_a, width_b, offset_a, offset_b, length);
        Self::by_external_boundary(store, wire)
    }

    /// Create an L-shaped face
    pub fn l_shape(
        store: &TopologyStore,
        origin: Point3,
        width: f64,
        length: f64,
        a: f64,
        b: f64,
    ) -> FaceHandle {
        let wire = Wire::l_shape(store, origin, width, length, a, b);
        Self::by_external_boundary(store, wire)
    }

    /// Create a T-shaped face
    pub fn t_shape(
        store: &TopologyStore,
        origin: Point3,
        width: f64,
        length: f64,
        a: f64,
        b: f64,
    ) -> FaceHandle {
        let wire = Wire::t_shape(store, origin, width, length, a, b);
        Self::by_external_boundary(store, wire)
    }

    /// Create a C-shaped face
    pub fn c_shape(
        store: &TopologyStore,
        origin: Point3,
        width: f64,
        length: f64,
        a: f64,
        b: f64,
        c: f64,
    ) -> FaceHandle {
        let wire = Wire::c_shape(store, origin, width, length, a, b, c);
        Self::by_external_boundary(store, wire)
    }

    /// Create an I-shaped face
    pub fn i_shape(
        store: &TopologyStore,
        origin: Point3,
        width: f64,
        length: f64,
        a: f64,
        b: f64,
        c: f64,
    ) -> FaceHandle {
        let wire = Wire::i_shape(store, origin, width, length, a, b, c);
        Self::by_external_boundary(store, wire)
    }

    /// Create a cross-shaped face
    pub fn cross_shape(
        store: &TopologyStore,
        origin: Point3,
        width: f64,
        length: f64,
        a: f64,
        b: f64,
    ) -> FaceHandle {
        let wire = Wire::cross_shape(store, origin, width, length, a, b);
        Self::by_external_boundary(store, wire)
    }

    /// Create a squircle face
    pub fn squircle(
        store: &TopologyStore,
        center: Point3,
        radius: f64,
        sides: usize,
        exponent: f64,
    ) -> FaceHandle {
        let wire = Wire::squircle(store, center, radius, sides, exponent);
        Self::by_external_boundary(store, wire)
    }

    /// Create a ring face (circle with a hole)
    pub fn ring(
        store: &TopologyStore,
        center: Point3,
        inner_radius: f64,
        outer_radius: f64,
        segments: usize,
    ) -> FaceHandle {
        let outer_wire = Wire::circle(store, center, outer_radius, segments);
        let inner_wire = Wire::circle(store, center, inner_radius, segments);
        Self::by_external_internal_boundaries(store, outer_wire, vec![inner_wire])
    }

    /// Get the compactness of the face (4π * area / perimeter²)
    pub fn compactness(store: &TopologyStore, handle: FaceHandle) -> f64 {
        let area = Self::area(store, handle);
        let perimeter = Self::perimeter(store, handle);
        if perimeter < TOLERANCE {
            return 0.0;
        }
        4.0 * std::f64::consts::PI * area / (perimeter * perimeter)
    }

    /// Check if the face is convex
    pub fn is_convex(store: &TopologyStore, handle: FaceHandle) -> bool {
        // A face with holes is never convex
        if !Self::internal_boundaries(store, handle).is_empty() {
            return false;
        }

        let outer = Self::external_boundary(store, handle);
        let edges = Wire::edges(store, outer);
        if edges.len() < 3 {
            return true;
        }

        let normal = match Self::normal(store, handle) {
            Some(n) => n,
            None => return true,
        };

        // Check if all cross products have the same sign relative to normal
        for i in 0..edges.len() {
            let j = (i + 1) % edges.len();
            let d1 = Edge::direction(store, edges[i]);
            let d2 = Edge::direction(store, edges[j]);
            let cross = d1.cross(d2);

            if cross.dot(normal) < -TOLERANCE {
                return false;
            }
        }

        true
    }

    /// Check if two faces are coplanar
    pub fn is_coplanar(store: &TopologyStore, f1: FaceHandle, f2: FaceHandle, tolerance: f64) -> bool {
        let n1 = match Self::normal(store, f1) {
            Some(n) => n,
            None => return false,
        };
        let n2 = match Self::normal(store, f2) {
            Some(n) => n,
            None => return false,
        };

        // Check if normals are parallel
        let cross = n1.cross(n2);
        if cross.length_squared() > tolerance * tolerance {
            return false;
        }

        // Check if centers are on the same plane
        let c1 = Self::center_of_mass(store, f1);
        let c2 = Self::center_of_mass(store, f2);
        let d = (c2 - c1).dot(n1).abs();

        d < tolerance
    }

    /// Get the plane equation coefficients (ax + by + cz + d = 0)
    pub fn plane_equation(store: &TopologyStore, handle: FaceHandle) -> Option<(f64, f64, f64, f64)> {
        let normal = Self::normal(store, handle)?;
        let center = Self::center_of_mass(store, handle);

        // d = -n · p
        let d = -(normal.x * center.x + normal.y * center.y + normal.z * center.z);

        Some((normal.x, normal.y, normal.z, d))
    }

    /// Get the angle between two faces at their shared edge
    pub fn angle(store: &TopologyStore, f1: FaceHandle, f2: FaceHandle) -> Option<f64> {
        let n1 = Self::normal(store, f1)?;
        let n2 = Self::normal(store, f2)?;

        let dot = n1.dot(n2).clamp(-1.0, 1.0);
        Some(dot.acos().to_degrees())
    }

    /// Get the compass angle of the face normal
    pub fn compass_angle(store: &TopologyStore, handle: FaceHandle) -> Option<f64> {
        let normal = Self::normal(store, handle)?;
        // Project normal to XY plane and get angle from North (Y axis)
        let angle = normal.x.atan2(normal.y).to_degrees();
        let compass = if angle < 0.0 { angle + 360.0 } else { angle };
        Some(compass)
    }

    /// Check if the face is facing toward a point
    pub fn facing_toward(store: &TopologyStore, handle: FaceHandle, point: Point3) -> bool {
        let normal = match Self::normal(store, handle) {
            Some(n) => n,
            None => return false,
        };
        let center = Self::center_of_mass(store, handle);
        let to_point = (point - center).normalize_or_zero();
        normal.dot(to_point) > 0.0
    }

    /// Get vertex by UV parameters on the face
    pub fn vertex_by_parameters(
        store: &TopologyStore,
        handle: FaceHandle,
        u: f64,
        v: f64,
    ) -> VertexHandle {
        let bbox = Self::bounding_box(store, handle);
        let normal = Self::normal(store, handle).unwrap_or(Vector3::Z);

        // Project to the face plane
        let (u_axis, v_axis) = {
            let u_ax = if normal.x.abs() < 0.9 {
                Vector3::X.cross(normal).normalize_or_zero()
            } else {
                Vector3::Y.cross(normal).normalize_or_zero()
            };
            let v_ax = normal.cross(u_ax);
            (u_ax, v_ax)
        };

        let center = Self::center_of_mass(store, handle);
        let width = bbox.1.x - bbox.0.x;
        let height = bbox.1.y - bbox.0.y;

        let point = center
            + u_axis * ((u - 0.5) * width)
            + v_axis * ((v - 0.5) * height);

        Vertex::by_point(store, point)
    }

    /// Get an internal vertex (center of mass)
    pub fn internal_vertex(store: &TopologyStore, handle: FaceHandle) -> VertexHandle {
        let center = Self::center_of_mass(store, handle);
        Vertex::by_point(store, center)
    }

    /// Create an edge along the face normal from its center
    pub fn normal_edge(store: &TopologyStore, handle: FaceHandle, length: f64) -> Option<EdgeHandle> {
        Edge::by_face_normal(store, handle, length)
    }

    /// Add internal boundaries (holes) to a face
    pub fn add_internal_boundaries(
        store: &TopologyStore,
        handle: FaceHandle,
        internal_wires: Vec<WireHandle>,
    ) -> FaceHandle {
        let outer = Self::external_boundary(store, handle);
        let mut inner = Self::internal_boundaries(store, handle);
        inner.extend(internal_wires);
        Self::by_external_internal_boundaries(store, outer, inner)
    }

    /// Get interior angles at each vertex
    pub fn interior_angles(store: &TopologyStore, handle: FaceHandle) -> Vec<f64> {
        let outer = Self::external_boundary(store, handle);
        Wire::interior_angles(store, outer)
    }

    /// Get exterior angles at each vertex
    pub fn exterior_angles(store: &TopologyStore, handle: FaceHandle) -> Vec<f64> {
        let outer = Self::external_boundary(store, handle);
        Wire::exterior_angles(store, outer)
    }

    /// Project a face onto another face's plane
    pub fn project(
        store: &TopologyStore,
        handle: FaceHandle,
        target: FaceHandle,
        direction: Vector3,
    ) -> FaceHandle {
        let outer = Self::external_boundary(store, handle);
        let inner = Self::internal_boundaries(store, handle);

        let projected_outer = Wire::project(store, outer, target, direction);
        let projected_inner: Vec<_> = inner
            .iter()
            .map(|w| Wire::project(store, *w, target, direction))
            .collect();

        Self::by_external_internal_boundaries(store, projected_outer, projected_inner)
    }

    /// Create a face by offsetting a wire
    pub fn by_offset(
        store: &TopologyStore,
        wire: WireHandle,
        offset: f64,
    ) -> FaceHandle {
        // Create the outer boundary by offsetting outward
        // and the inner boundary is the original wire
        let outer_vertices = Wire::vertices(store, wire);
        let normal = Wire::normal(store, wire).unwrap_or(Vector3::Z);

        let mut offset_vertices = Vec::with_capacity(outer_vertices.len());
        let edges = Wire::edges(store, wire);

        for (i, v) in outer_vertices.iter().enumerate() {
            let point = Vertex::point(store, *v);

            // Calculate the offset direction at this vertex
            let prev_edge = if i == 0 { edges.last().unwrap() } else { &edges[i - 1] };
            let next_edge = &edges[i % edges.len()];

            let d1 = Edge::direction(store, *prev_edge);
            let d2 = Edge::direction(store, *next_edge);

            // Bisector direction
            let bisector = (d1 + d2).normalize_or_zero();
            let perp = normal.cross(bisector).normalize_or_zero();

            let offset_point = point + perp * offset;
            offset_vertices.push(Vertex::by_point(store, offset_point));
        }

        let outer_wire = Wire::by_vertices(store, offset_vertices, true);
        Self::by_external_internal_boundaries(store, outer_wire, vec![wire])
    }

    /// Create a thickened wire as a face
    pub fn by_thickened_wire(
        store: &TopologyStore,
        wire: WireHandle,
        thickness: f64,
    ) -> FaceHandle {
        Self::by_offset(store, wire, thickness)
    }

    /// Create an offset face by a target area (binary search for offset)
    pub fn by_offset_area(
        store: &TopologyStore,
        face: FaceHandle,
        target_area: f64,
        tolerance: f64,
    ) -> Option<FaceHandle> {
        let original_area = Self::area(store, face);
        let wire = Self::external_boundary(store, face);

        if target_area < tolerance || target_area >= original_area {
            return None;
        }

        // Use binary search to find the right offset
        let mut low = 0.0;
        let mut high = Self::estimate_max_offset(store, face);
        let max_iterations = 50;

        for _ in 0..max_iterations {
            let mid = (low + high) / 2.0;
            let offset_wire = Wire::by_offset(store, wire, -mid, 2.0);
            let offset_face = Self::by_external_boundary(store, offset_wire);
            let offset_area = Self::area(store, offset_face);

            if (offset_area - target_area).abs() < tolerance {
                return Some(offset_face);
            }

            if offset_area > target_area {
                low = mid;
            } else {
                high = mid;
            }

            if (high - low).abs() < tolerance / 10.0 {
                break;
            }
        }

        // Return best approximation
        let offset = (low + high) / 2.0;
        let offset_wire = Wire::by_offset(store, wire, -offset, 2.0);
        Some(Self::by_external_boundary(store, offset_wire))
    }

    /// Estimate maximum inward offset (half the minimum dimension)
    fn estimate_max_offset(store: &TopologyStore, face: FaceHandle) -> f64 {
        let vertices = Self::vertices(store, face);
        if vertices.len() < 3 {
            return 0.0;
        }

        // Find bounding box
        let points: Vec<_> = vertices.iter().map(|v| Vertex::point(store, *v)).collect();
        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;

        for p in &points {
            min_x = min_x.min(p.x);
            max_x = max_x.max(p.x);
            min_y = min_y.min(p.y);
            max_y = max_y.max(p.y);
        }

        // Maximum offset is half the minimum dimension
        ((max_x - min_x).min(max_y - min_y)) / 2.0
    }

    /// Scale face to achieve target area
    pub fn scale_to_area(
        store: &TopologyStore,
        face: FaceHandle,
        target_area: f64,
    ) -> FaceHandle {
        let current_area = Self::area(store, face);
        if current_area < TOLERANCE {
            return face;
        }

        let scale = (target_area / current_area).sqrt();
        let center = Self::center_of_mass(store, face);

        let vertices = Self::vertices(store, face);
        let scaled_vertices: Vec<_> = vertices
            .iter()
            .map(|v| {
                let p = Vertex::point(store, *v);
                let offset = (p - center) * scale;
                Vertex::by_point(store, center + offset)
            })
            .collect();

        let wire = Wire::by_vertices(store, scaled_vertices, true);
        Self::by_external_boundary(store, wire)
    }

    /// Get the dictionary attached to this face
    pub fn get_dictionary(store: &TopologyStore, handle: FaceHandle) -> Dictionary {
        store.faces.read()[handle.index.index()].dictionary.clone()
    }

    /// Set the dictionary for this face
    pub fn set_dictionary(store: &TopologyStore, handle: FaceHandle, dict: Dictionary) {
        store.faces.write()[handle.index.index()].dictionary = dict;
    }

    /// Set a single key-value pair in the face's dictionary
    pub fn set_dictionary_value(store: &TopologyStore, handle: FaceHandle, key: &str, value: DictionaryValue) {
        store.faces.write()[handle.index.index()].dictionary.set(key, value);
    }

    /// Get a value from the face's dictionary
    pub fn get_dictionary_value(store: &TopologyStore, handle: FaceHandle, key: &str) -> Option<DictionaryValue> {
        store.faces.read()[handle.index.index()].dictionary.get(key).cloned()
    }
}

impl<'a> FaceRef<'a> {
    pub fn external_boundary(&self) -> WireHandle {
        Face::external_boundary(self.store, self.handle)
    }

    pub fn internal_boundaries(&self) -> Vec<WireHandle> {
        Face::internal_boundaries(self.store, self.handle)
    }

    pub fn edges(&self) -> Vec<EdgeHandle> {
        Face::edges(self.store, self.handle)
    }

    pub fn vertices(&self) -> Vec<VertexHandle> {
        Face::vertices(self.store, self.handle)
    }

    pub fn normal(&self) -> Option<Vector3> {
        Face::normal(self.store, self.handle)
    }

    pub fn area(&self) -> f64 {
        Face::area(self.store, self.handle)
    }

    pub fn center_of_mass(&self) -> Point3 {
        Face::center_of_mass(self.store, self.handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rectangle_face() {
        let store = TopologyStore::new();
        let face = Face::rectangle(&store, Point3::ZERO, 2.0, 3.0);

        assert!((Face::area(&store, face) - 6.0).abs() < TOLERANCE);
        assert!((Face::perimeter(&store, face) - 10.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_face_normal() {
        let store = TopologyStore::new();
        let face = Face::rectangle(&store, Point3::ZERO, 1.0, 1.0);

        let normal = Face::normal(&store, face).unwrap();
        assert!((normal.z.abs() - 1.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_face_triangulation() {
        let store = TopologyStore::new();
        let face = Face::rectangle(&store, Point3::ZERO, 1.0, 1.0);

        let triangles = Face::triangulate(&store, face);
        assert_eq!(triangles.len(), 2);
    }
}
