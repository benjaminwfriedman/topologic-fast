//! Wire topology (connected sequence of edges)

use super::*;
use crate::geometry::{Point3, Vector3, TOLERANCE};
use smallvec::SmallVec;

/// Handle to a wire in the topology store
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WireHandle {
    pub(crate) index: ArenaIndex,
    pub(crate) id: TopologyId,
}

impl WireHandle {
    pub fn id(&self) -> TopologyId {
        self.id
    }
}

/// Internal wire data
pub(crate) struct WireData {
    pub id: TopologyId,
    pub edges: Vec<EdgeHandle>,
    /// For each edge, true if the wire uses the edge in its original direction,
    /// false if the edge should be traversed in reverse (from end to start)
    pub edge_orientations: Vec<bool>,
    pub is_closed: bool,
    pub faces: SmallVec<[FaceHandle; 2]>,
    pub dictionary: Dictionary,
}

/// Wire operations
pub struct Wire;

impl Wire {
    /// Create a wire from a sequence of edges
    /// Automatically computes edge orientations by tracing connectivity
    pub fn by_edges(store: &TopologyStore, edges: Vec<EdgeHandle>) -> WireHandle {
        if edges.is_empty() {
            return store.add_wire(vec![], vec![], false);
        }

        let mut orientations = Vec::with_capacity(edges.len());

        if edges.len() == 1 {
            // Single edge - use forward orientation
            orientations.push(true);
        } else {
            // Determine orientation of first edge by checking which end connects to second edge
            let (first_start, first_end) = Edge::vertices(store, edges[0]);
            let (second_start, second_end) = Edge::vertices(store, edges[1]);

            let first_forward = first_end == second_start || first_end == second_end;
            orientations.push(first_forward);

            let mut prev_end = if first_forward { first_end } else { first_start };

            // Compute orientations for remaining edges
            for i in 1..edges.len() {
                let (start, end) = Edge::vertices(store, edges[i]);
                let forward = start == prev_end;
                orientations.push(forward);
                prev_end = if forward { end } else { start };
            }
        }

        let is_closed = Self::check_closed_with_orientations(store, &edges, &orientations);
        store.add_wire(edges, orientations, is_closed)
    }

    /// Create a closed wire from vertices (connecting them in order)
    pub fn by_vertices(store: &TopologyStore, vertices: Vec<VertexHandle>, close: bool) -> WireHandle {
        if vertices.len() < 2 {
            panic!("Wire requires at least 2 vertices");
        }

        let mut edges = Vec::with_capacity(vertices.len());
        let mut orientations = Vec::with_capacity(vertices.len());

        for i in 0..vertices.len() - 1 {
            let start = vertices[i];
            let end = vertices[i + 1];
            let edge = Edge::by_start_vertex_end_vertex(store, start, end);
            edges.push(edge);

            // Check if the edge is stored in the direction we want
            let (edge_start, _) = Edge::vertices(store, edge);
            let forward = edge_start == start;
            orientations.push(forward);
        }

        if close && vertices.len() > 2 {
            let start = *vertices.last().unwrap();
            let end = vertices[0];
            let edge = Edge::by_start_vertex_end_vertex(store, start, end);
            edges.push(edge);

            let (edge_start, _) = Edge::vertices(store, edge);
            let forward = edge_start == start;
            orientations.push(forward);
        }

        store.add_wire(edges, orientations, close)
    }

    /// Create a rectangular wire
    pub fn rectangle(
        store: &TopologyStore,
        origin: Point3,
        width: f64,
        height: f64,
    ) -> WireHandle {
        let v0 = Vertex::by_point(store, origin);
        let v1 = Vertex::by_point(store, origin + Vector3::new(width, 0.0, 0.0));
        let v2 = Vertex::by_point(store, origin + Vector3::new(width, height, 0.0));
        let v3 = Vertex::by_point(store, origin + Vector3::new(0.0, height, 0.0));

        Wire::by_vertices(store, vec![v0, v1, v2, v3], true)
    }

    /// Create a circle wire (approximated by segments)
    pub fn circle(
        store: &TopologyStore,
        center: Point3,
        radius: f64,
        segments: usize,
    ) -> WireHandle {
        let segments = segments.max(3);
        let mut vertices = Vec::with_capacity(segments);

        for i in 0..segments {
            let angle = std::f64::consts::TAU * i as f64 / segments as f64;
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();
            vertices.push(Vertex::by_coordinates(store, x, y, center.z));
        }

        Wire::by_vertices(store, vertices, true)
    }

    fn check_closed(store: &TopologyStore, edges: &[EdgeHandle]) -> bool {
        if edges.len() < 2 {
            return false;
        }

        let first_start = Edge::start_vertex(store, edges[0]);
        let last_end = Edge::end_vertex(store, *edges.last().unwrap());

        first_start == last_end
    }

    fn check_closed_with_orientations(store: &TopologyStore, edges: &[EdgeHandle], orientations: &[bool]) -> bool {
        if edges.len() < 2 || edges.len() != orientations.len() {
            return false;
        }

        // Get the start vertex of the first edge (respecting orientation)
        let (first_start, first_end) = Edge::vertices(store, edges[0]);
        let wire_start = if orientations[0] { first_start } else { first_end };

        // Get the end vertex of the last edge (respecting orientation)
        let (last_start, last_end) = Edge::vertices(store, *edges.last().unwrap());
        let wire_end = if *orientations.last().unwrap() { last_end } else { last_start };

        wire_start == wire_end
    }

    /// Get all edges in the wire
    pub fn edges(store: &TopologyStore, handle: WireHandle) -> Vec<EdgeHandle> {
        store.wires.read()[handle.index.index()].edges.clone()
    }

    /// Get edge orientations (for debugging)
    pub fn edge_orientations(store: &TopologyStore, handle: WireHandle) -> Vec<bool> {
        store.wires.read()[handle.index.index()].edge_orientations.clone()
    }

    /// Get all vertices in the wire (in order)
    /// Uses stored edge orientations to properly follow vertex connectivity
    pub fn vertices(store: &TopologyStore, handle: WireHandle) -> Vec<VertexHandle> {
        let wires = store.wires.read();
        let wire = &wires[handle.index.index()];
        let edges = &wire.edges;
        let orientations = &wire.edge_orientations;

        if edges.is_empty() {
            return Vec::new();
        }

        let is_closed = wire.is_closed;
        let mut vertices = Vec::with_capacity(edges.len() + if is_closed { 0 } else { 1 });

        // Use orientations to get correct vertex order
        for (i, edge) in edges.iter().enumerate() {
            let (edge_start, edge_end) = Edge::vertices(store, *edge);
            let forward = orientations.get(i).copied().unwrap_or(true);
            let start_v = if forward { edge_start } else { edge_end };
            vertices.push(start_v);
        }

        // For open wires, add the end vertex of the last edge
        if !is_closed && !edges.is_empty() {
            let last_edge = *edges.last().unwrap();
            let (edge_start, edge_end) = Edge::vertices(store, last_edge);
            let forward = orientations.last().copied().unwrap_or(true);
            let end_v = if forward { edge_end } else { edge_start };
            vertices.push(end_v);
        }

        vertices
    }

    /// Check if the wire is closed
    pub fn is_closed(store: &TopologyStore, handle: WireHandle) -> bool {
        store.wires.read()[handle.index.index()].is_closed
    }

    /// Get the faces using this wire
    pub fn faces(store: &TopologyStore, handle: WireHandle) -> Vec<FaceHandle> {
        store.wires.read()[handle.index.index()].faces.to_vec()
    }

    /// Get the total length of the wire
    pub fn length(store: &TopologyStore, handle: WireHandle) -> f64 {
        Self::edges(store, handle)
            .iter()
            .map(|e| Edge::length(store, *e))
            .sum()
    }

    /// Get the center of mass of the wire
    pub fn center_of_mass(store: &TopologyStore, handle: WireHandle) -> Point3 {
        let vertices = Self::vertices(store, handle);
        if vertices.is_empty() {
            return Point3::ZERO;
        }

        let sum: Point3 = vertices
            .iter()
            .map(|v| Vertex::point(store, *v))
            .sum();

        sum / vertices.len() as f64
    }

    /// Get the bounding box of the wire
    pub fn bounding_box(store: &TopologyStore, handle: WireHandle) -> (Point3, Point3) {
        let vertices = Self::vertices(store, handle);
        if vertices.is_empty() {
            return (Point3::ZERO, Point3::ZERO);
        }

        let points: Vec<_> = vertices.iter().map(|v| Vertex::point(store, *v)).collect();
        crate::geometry::bounding_box(&points).unwrap_or((Point3::ZERO, Point3::ZERO))
    }

    /// Check if the wire is planar
    pub fn is_planar(store: &TopologyStore, handle: WireHandle) -> bool {
        let vertices = Self::vertices(store, handle);
        if vertices.len() < 4 {
            return true;
        }

        let points: Vec<_> = vertices.iter().map(|v| Vertex::point(store, *v)).collect();

        // Check if all points are coplanar
        let v1 = points[1] - points[0];
        let v2 = points[2] - points[0];
        let normal = v1.cross(v2);

        if normal.length_squared() < TOLERANCE * TOLERANCE {
            return true; // Points are collinear
        }

        let normal = normal.normalize();

        for point in &points[3..] {
            let d = (*point - points[0]).dot(normal);
            if d.abs() > TOLERANCE {
                return false;
            }
        }

        true
    }

    /// Get the normal of the wire (if planar)
    pub fn normal(store: &TopologyStore, handle: WireHandle) -> Option<Vector3> {
        let vertices = Self::vertices(store, handle);
        if vertices.len() < 3 {
            return None;
        }

        let points: Vec<_> = vertices.iter().map(|v| Vertex::point(store, *v)).collect();

        // Newell's method for computing normal
        let mut normal = Vector3::ZERO;

        for i in 0..points.len() {
            let curr = points[i];
            let next = points[(i + 1) % points.len()];

            normal.x += (curr.y - next.y) * (curr.z + next.z);
            normal.y += (curr.z - next.z) * (curr.x + next.x);
            normal.z += (curr.x - next.x) * (curr.y + next.y);
        }

        let len = normal.length();
        if len < TOLERANCE {
            return None;
        }

        Some(normal / len)
    }

    /// Get the signed area of the wire (if planar and closed)
    pub fn area(store: &TopologyStore, handle: WireHandle) -> f64 {
        if !Self::is_closed(store, handle) {
            return 0.0;
        }

        let vertices = Self::vertices(store, handle);
        if vertices.len() < 3 {
            return 0.0;
        }

        let points: Vec<_> = vertices.iter().map(|v| Vertex::point(store, *v)).collect();

        // Shoelace formula generalized to 3D
        let normal = match Self::normal(store, handle) {
            Some(n) => n,
            None => return 0.0,
        };

        let mut area = Vector3::ZERO;
        for i in 0..points.len() {
            let curr = points[i];
            let next = points[(i + 1) % points.len()];
            area += curr.cross(next);
        }

        (area.dot(normal) / 2.0).abs()
    }

    /// Reverse the direction of the wire
    /// Keeps the same edges but reverses their order and flips orientations
    pub fn reverse(store: &TopologyStore, handle: WireHandle) -> WireHandle {
        let wires = store.wires.read();
        let wire = &wires[handle.index.index()];

        // Reverse edge order and flip all orientations
        let edges: Vec<_> = wire.edges.iter().rev().copied().collect();
        let orientations: Vec<_> = wire.edge_orientations.iter().rev().map(|o| !o).collect();
        let is_closed = wire.is_closed;

        drop(wires);
        store.add_wire(edges, orientations, is_closed)
    }

    /// Create a square wire
    pub fn square(
        store: &TopologyStore,
        origin: Point3,
        size: f64,
        direction: Vector3,
    ) -> WireHandle {
        Self::rectangle(store, origin, size, size)
    }

    /// Create a line wire (2 vertices)
    pub fn line(
        store: &TopologyStore,
        origin: Point3,
        length: f64,
        direction: Vector3,
    ) -> WireHandle {
        let dir = direction.normalize_or_zero();
        let end = origin + dir * length;
        let v0 = Vertex::by_point(store, origin);
        let v1 = Vertex::by_point(store, end);
        Wire::by_vertices(store, vec![v0, v1], false)
    }

    /// Create an ellipse wire
    pub fn ellipse(
        store: &TopologyStore,
        center: Point3,
        width: f64,
        length: f64,
        segments: usize,
        from_angle: f64,
        to_angle: f64,
        close: bool,
    ) -> WireHandle {
        let segments = segments.max(3);
        let a = width / 2.0;
        let b = length / 2.0;
        let from_rad = from_angle.to_radians();
        let to_rad = to_angle.to_radians();
        let angle_range = to_rad - from_rad;

        let mut vertices = Vec::with_capacity(segments + 1);
        for i in 0..=segments {
            let t = i as f64 / segments as f64;
            let angle = from_rad + angle_range * t;
            let x = center.x + a * angle.cos();
            let y = center.y + b * angle.sin();
            vertices.push(Vertex::by_coordinates(store, x, y, center.z));
        }

        // Remove duplicate end vertex if closed
        if close && (to_angle - from_angle).abs() >= 360.0 - TOLERANCE {
            vertices.pop();
        }

        Wire::by_vertices(store, vertices, close)
    }

    /// Create a star wire
    pub fn star(
        store: &TopologyStore,
        center: Point3,
        outer_radius: f64,
        inner_radius: f64,
        rays: usize,
    ) -> WireHandle {
        let rays = rays.max(3);
        let mut vertices = Vec::with_capacity(rays * 2);
        let angle_step = std::f64::consts::TAU / (rays * 2) as f64;

        for i in 0..(rays * 2) {
            let angle = angle_step * i as f64 - std::f64::consts::FRAC_PI_2;
            let radius = if i % 2 == 0 { outer_radius } else { inner_radius };
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();
            vertices.push(Vertex::by_coordinates(store, x, y, center.z));
        }

        Wire::by_vertices(store, vertices, true)
    }

    /// Create a trapezoid wire
    pub fn trapezoid(
        store: &TopologyStore,
        origin: Point3,
        width_a: f64,
        width_b: f64,
        offset_a: f64,
        offset_b: f64,
        length: f64,
    ) -> WireHandle {
        let v0 = Vertex::by_point(store, origin + Vector3::new(offset_a, 0.0, 0.0));
        let v1 = Vertex::by_point(store, origin + Vector3::new(offset_a + width_a, 0.0, 0.0));
        let v2 = Vertex::by_point(store, origin + Vector3::new(offset_b + width_b, length, 0.0));
        let v3 = Vertex::by_point(store, origin + Vector3::new(offset_b, length, 0.0));
        Wire::by_vertices(store, vec![v0, v1, v2, v3], true)
    }

    /// Create a spiral wire
    pub fn spiral(
        store: &TopologyStore,
        center: Point3,
        radius_a: f64,
        radius_b: f64,
        height: f64,
        turns: usize,
        sides: usize,
        clockwise: bool,
    ) -> WireHandle {
        let total_segments = turns * sides;
        let mut vertices = Vec::with_capacity(total_segments + 1);

        for i in 0..=total_segments {
            let t = i as f64 / total_segments as f64;
            let angle = std::f64::consts::TAU * turns as f64 * t;
            let angle = if clockwise { -angle } else { angle };
            let radius = radius_a + (radius_b - radius_a) * t;
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();
            let z = center.z + height * t;
            vertices.push(Vertex::by_coordinates(store, x, y, z));
        }

        Wire::by_vertices(store, vertices, false)
    }

    /// Create an arc wire
    pub fn arc(
        store: &TopologyStore,
        start: Point3,
        middle: Point3,
        end: Point3,
        sides: usize,
        close: bool,
    ) -> Option<WireHandle> {
        // Calculate circle center from 3 points
        let a = start;
        let b = middle;
        let c = end;

        let d = 2.0 * (a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y));
        if d.abs() < TOLERANCE {
            return None; // Points are collinear
        }

        let ux = ((a.x * a.x + a.y * a.y) * (b.y - c.y)
                + (b.x * b.x + b.y * b.y) * (c.y - a.y)
                + (c.x * c.x + c.y * c.y) * (a.y - b.y)) / d;
        let uy = ((a.x * a.x + a.y * a.y) * (c.x - b.x)
                + (b.x * b.x + b.y * b.y) * (a.x - c.x)
                + (c.x * c.x + c.y * c.y) * (b.x - a.x)) / d;

        let center = Point3::new(ux, uy, a.z);
        let radius = ((a.x - ux) * (a.x - ux) + (a.y - uy) * (a.y - uy)).sqrt();

        let start_angle = (a.y - uy).atan2(a.x - ux);
        let end_angle = (c.y - uy).atan2(c.x - ux);

        let mut angle_range = end_angle - start_angle;
        // Check if we should go the other way around
        let mid_angle = (b.y - uy).atan2(b.x - ux);
        let normalized_mid = (mid_angle - start_angle).rem_euclid(std::f64::consts::TAU);
        let normalized_end = angle_range.rem_euclid(std::f64::consts::TAU);
        if normalized_mid > normalized_end {
            angle_range = angle_range - std::f64::consts::TAU;
        }

        let sides = sides.max(3);
        let mut vertices = Vec::with_capacity(sides + 1);

        for i in 0..=sides {
            let t = i as f64 / sides as f64;
            let angle = start_angle + angle_range * t;
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();
            vertices.push(Vertex::by_coordinates(store, x, y, a.z));
        }

        Some(Wire::by_vertices(store, vertices, close))
    }

    /// Create an L-shaped wire
    pub fn l_shape(
        store: &TopologyStore,
        origin: Point3,
        width: f64,
        length: f64,
        a: f64,
        b: f64,
    ) -> WireHandle {
        let v0 = Vertex::by_point(store, origin);
        let v1 = Vertex::by_point(store, origin + Vector3::new(a, 0.0, 0.0));
        let v2 = Vertex::by_point(store, origin + Vector3::new(a, length - b, 0.0));
        let v3 = Vertex::by_point(store, origin + Vector3::new(width, length - b, 0.0));
        let v4 = Vertex::by_point(store, origin + Vector3::new(width, length, 0.0));
        let v5 = Vertex::by_point(store, origin + Vector3::new(0.0, length, 0.0));
        Wire::by_vertices(store, vec![v0, v1, v2, v3, v4, v5], true)
    }

    /// Create a T-shaped wire
    pub fn t_shape(
        store: &TopologyStore,
        origin: Point3,
        width: f64,
        length: f64,
        a: f64,
        b: f64,
    ) -> WireHandle {
        let half_stem = a / 2.0;
        let mid = width / 2.0;

        let v0 = Vertex::by_point(store, origin + Vector3::new(mid - half_stem, 0.0, 0.0));
        let v1 = Vertex::by_point(store, origin + Vector3::new(mid + half_stem, 0.0, 0.0));
        let v2 = Vertex::by_point(store, origin + Vector3::new(mid + half_stem, length - b, 0.0));
        let v3 = Vertex::by_point(store, origin + Vector3::new(width, length - b, 0.0));
        let v4 = Vertex::by_point(store, origin + Vector3::new(width, length, 0.0));
        let v5 = Vertex::by_point(store, origin + Vector3::new(0.0, length, 0.0));
        let v6 = Vertex::by_point(store, origin + Vector3::new(0.0, length - b, 0.0));
        let v7 = Vertex::by_point(store, origin + Vector3::new(mid - half_stem, length - b, 0.0));

        Wire::by_vertices(store, vec![v0, v1, v2, v3, v4, v5, v6, v7], true)
    }

    /// Create a C-shaped wire
    pub fn c_shape(
        store: &TopologyStore,
        origin: Point3,
        width: f64,
        length: f64,
        a: f64,
        b: f64,
        c: f64,
    ) -> WireHandle {
        let v0 = Vertex::by_point(store, origin);
        let v1 = Vertex::by_point(store, origin + Vector3::new(width, 0.0, 0.0));
        let v2 = Vertex::by_point(store, origin + Vector3::new(width, c, 0.0));
        let v3 = Vertex::by_point(store, origin + Vector3::new(a, c, 0.0));
        let v4 = Vertex::by_point(store, origin + Vector3::new(a, length - c, 0.0));
        let v5 = Vertex::by_point(store, origin + Vector3::new(width, length - c, 0.0));
        let v6 = Vertex::by_point(store, origin + Vector3::new(width, length, 0.0));
        let v7 = Vertex::by_point(store, origin + Vector3::new(0.0, length, 0.0));

        Wire::by_vertices(store, vec![v0, v1, v2, v3, v4, v5, v6, v7], true)
    }

    /// Create a cross-shaped wire
    pub fn cross_shape(
        store: &TopologyStore,
        origin: Point3,
        width: f64,
        length: f64,
        a: f64,
        b: f64,
    ) -> WireHandle {
        let cx = width / 2.0;
        let cy = length / 2.0;
        let half_a = a / 2.0;
        let half_b = b / 2.0;

        let vertices = vec![
            Vertex::by_point(store, origin + Vector3::new(cx - half_a, 0.0, 0.0)),
            Vertex::by_point(store, origin + Vector3::new(cx + half_a, 0.0, 0.0)),
            Vertex::by_point(store, origin + Vector3::new(cx + half_a, cy - half_b, 0.0)),
            Vertex::by_point(store, origin + Vector3::new(width, cy - half_b, 0.0)),
            Vertex::by_point(store, origin + Vector3::new(width, cy + half_b, 0.0)),
            Vertex::by_point(store, origin + Vector3::new(cx + half_a, cy + half_b, 0.0)),
            Vertex::by_point(store, origin + Vector3::new(cx + half_a, length, 0.0)),
            Vertex::by_point(store, origin + Vector3::new(cx - half_a, length, 0.0)),
            Vertex::by_point(store, origin + Vector3::new(cx - half_a, cy + half_b, 0.0)),
            Vertex::by_point(store, origin + Vector3::new(0.0, cy + half_b, 0.0)),
            Vertex::by_point(store, origin + Vector3::new(0.0, cy - half_b, 0.0)),
            Vertex::by_point(store, origin + Vector3::new(cx - half_a, cy - half_b, 0.0)),
        ];

        Wire::by_vertices(store, vertices, true)
    }

    /// Create an I-shaped wire
    pub fn i_shape(
        store: &TopologyStore,
        origin: Point3,
        width: f64,
        length: f64,
        a: f64,
        b: f64,
        c: f64,
    ) -> WireHandle {
        let cx = width / 2.0;
        let half_a = a / 2.0;

        let vertices = vec![
            Vertex::by_point(store, origin + Vector3::new(0.0, 0.0, 0.0)),
            Vertex::by_point(store, origin + Vector3::new(width, 0.0, 0.0)),
            Vertex::by_point(store, origin + Vector3::new(width, b, 0.0)),
            Vertex::by_point(store, origin + Vector3::new(cx + half_a, b, 0.0)),
            Vertex::by_point(store, origin + Vector3::new(cx + half_a, length - c, 0.0)),
            Vertex::by_point(store, origin + Vector3::new(width, length - c, 0.0)),
            Vertex::by_point(store, origin + Vector3::new(width, length, 0.0)),
            Vertex::by_point(store, origin + Vector3::new(0.0, length, 0.0)),
            Vertex::by_point(store, origin + Vector3::new(0.0, length - c, 0.0)),
            Vertex::by_point(store, origin + Vector3::new(cx - half_a, length - c, 0.0)),
            Vertex::by_point(store, origin + Vector3::new(cx - half_a, b, 0.0)),
            Vertex::by_point(store, origin + Vector3::new(0.0, b, 0.0)),
        ];

        Wire::by_vertices(store, vertices, true)
    }

    /// Close the wire if not already closed
    pub fn close(store: &TopologyStore, handle: WireHandle) -> WireHandle {
        if Self::is_closed(store, handle) {
            return handle;
        }

        let edges = Self::edges(store, handle);
        if edges.is_empty() {
            return handle;
        }

        let first_vertex = Edge::start_vertex(store, edges[0]);
        let last_vertex = Edge::end_vertex(store, *edges.last().unwrap());

        let mut new_edges = edges;
        new_edges.push(Edge::by_start_vertex_end_vertex(store, last_vertex, first_vertex));

        Wire::by_edges(store, new_edges)
    }

    /// Get the start vertex of the wire
    pub fn start_vertex(store: &TopologyStore, handle: WireHandle) -> Option<VertexHandle> {
        let edges = Self::edges(store, handle);
        if edges.is_empty() {
            return None;
        }
        Some(Edge::start_vertex(store, edges[0]))
    }

    /// Get the end vertex of the wire
    pub fn end_vertex(store: &TopologyStore, handle: WireHandle) -> Option<VertexHandle> {
        let edges = Self::edges(store, handle);
        if edges.is_empty() {
            return None;
        }
        Some(Edge::end_vertex(store, *edges.last().unwrap()))
    }

    /// Check if the wire is manifold (each vertex connected to exactly 2 edges)
    pub fn is_manifold(store: &TopologyStore, handle: WireHandle) -> bool {
        let edges = Self::edges(store, handle);
        if edges.len() < 2 {
            return false;
        }

        // For a closed wire, every vertex should have exactly 2 edges
        // For an open wire, end vertices have 1 edge, others have 2
        let is_closed = Self::is_closed(store, handle);

        for (i, edge) in edges.iter().enumerate() {
            let end_v = Edge::end_vertex(store, *edge);

            if i < edges.len() - 1 {
                let next_start = Edge::start_vertex(store, edges[i + 1]);
                if end_v != next_start {
                    return false;
                }
            }
        }

        if is_closed {
            let last_end = Edge::end_vertex(store, *edges.last().unwrap());
            let first_start = Edge::start_vertex(store, edges[0]);
            if last_end != first_start {
                return false;
            }
        }

        true
    }

    /// Get the vertex by parameter t in [0, 1]
    pub fn vertex_by_parameter(store: &TopologyStore, handle: WireHandle, t: f64) -> VertexHandle {
        let edges = Self::edges(store, handle);
        if edges.is_empty() {
            return Vertex::origin(store);
        }

        let total_length = Self::length(store, handle);
        let target_distance = total_length * t.clamp(0.0, 1.0);

        let mut accumulated = 0.0;
        for edge in &edges {
            let edge_length = Edge::length(store, *edge);
            if accumulated + edge_length >= target_distance {
                let edge_t = if edge_length > TOLERANCE {
                    (target_distance - accumulated) / edge_length
                } else {
                    0.0
                };
                let point = Edge::evaluate(store, *edge, edge_t);
                return Vertex::by_point(store, point);
            }
            accumulated += edge_length;
        }

        // Return end vertex
        Edge::end_vertex(store, *edges.last().unwrap())
    }

    /// Get the vertex by distance along the wire
    pub fn vertex_by_distance(store: &TopologyStore, handle: WireHandle, distance: f64) -> VertexHandle {
        let total_length = Self::length(store, handle);
        if total_length < TOLERANCE {
            return Self::start_vertex(store, handle).unwrap_or(Vertex::origin(store));
        }
        Self::vertex_by_parameter(store, handle, distance / total_length)
    }

    /// Get parameter at a vertex
    pub fn parameter_at_vertex(
        store: &TopologyStore,
        handle: WireHandle,
        vertex: VertexHandle,
        tolerance: f64,
    ) -> f64 {
        let edges = Self::edges(store, handle);
        let total_length = Self::length(store, handle);
        if total_length < TOLERANCE {
            return 0.0;
        }

        let target_point = Vertex::point(store, vertex);
        let mut accumulated = 0.0;

        for edge in &edges {
            let start = Edge::start_point(store, *edge);
            if start.distance(target_point) < tolerance {
                return accumulated / total_length;
            }

            let edge_length = Edge::length(store, *edge);
            accumulated += edge_length;
        }

        // Check end vertex
        if let Some(end_v) = Self::end_vertex(store, handle) {
            let end_point = Vertex::point(store, end_v);
            if end_point.distance(target_point) < tolerance {
                return 1.0;
            }
        }

        // Find closest point on wire
        let mut best_param = 0.0;
        let mut best_dist = f64::MAX;
        accumulated = 0.0;

        for edge in &edges {
            let t = Edge::closest_parameter(store, *edge, target_point);
            let closest = Edge::evaluate(store, *edge, t);
            let dist = closest.distance(target_point);

            if dist < best_dist {
                best_dist = dist;
                let edge_length = Edge::length(store, *edge);
                best_param = (accumulated + t * edge_length) / total_length;
            }
            accumulated += Edge::length(store, *edge);
        }

        best_param
    }

    /// Invert the wire (reverse direction)
    pub fn invert(store: &TopologyStore, handle: WireHandle) -> WireHandle {
        Self::reverse(store, handle)
    }

    /// Create a squircle (super-ellipse) wire
    pub fn squircle(
        store: &TopologyStore,
        center: Point3,
        radius: f64,
        sides: usize,
        exponent: f64,
    ) -> WireHandle {
        let sides = sides.max(4);
        let mut vertices = Vec::with_capacity(sides);

        for i in 0..sides {
            let angle = std::f64::consts::TAU * i as f64 / sides as f64;
            let cos_a = angle.cos();
            let sin_a = angle.sin();

            // Super-ellipse formula: |x/a|^n + |y/b|^n = 1
            let sign_cos = cos_a.signum();
            let sign_sin = sin_a.signum();
            let x = sign_cos * cos_a.abs().powf(2.0 / exponent) * radius;
            let y = sign_sin * sin_a.abs().powf(2.0 / exponent) * radius;

            vertices.push(Vertex::by_coordinates(store, center.x + x, center.y + y, center.z));
        }

        Wire::by_vertices(store, vertices, true)
    }

    /// Project wire onto a face
    pub fn project(
        store: &TopologyStore,
        handle: WireHandle,
        face: FaceHandle,
        direction: Vector3,
    ) -> WireHandle {
        let vertices = Self::vertices(store, handle);
        let is_closed = Self::is_closed(store, handle);

        let face_normal = Face::normal(store, face).unwrap_or(Vector3::Z);
        let face_center = Face::center_of_mass(store, face);

        let dir = if direction.length_squared() < TOLERANCE {
            face_normal
        } else {
            direction.normalize()
        };

        let projected: Vec<_> = vertices
            .iter()
            .map(|v| {
                let point = Vertex::point(store, *v);
                // Project point onto face plane
                let to_plane = face_center - point;
                let d = to_plane.dot(face_normal) / dir.dot(face_normal);
                let projected_point = point + dir * d;
                Vertex::by_point(store, projected_point)
            })
            .collect();

        Wire::by_vertices(store, projected, is_closed)
    }

    /// Get interior angles at each vertex
    pub fn interior_angles(store: &TopologyStore, handle: WireHandle) -> Vec<f64> {
        let edges = Self::edges(store, handle);
        if edges.len() < 2 {
            return vec![];
        }

        let is_closed = Self::is_closed(store, handle);
        let n = edges.len();
        let mut angles = Vec::with_capacity(n);

        for i in 0..n {
            let prev_idx = if i == 0 {
                if is_closed { n - 1 } else { continue; }
            } else {
                i - 1
            };

            let d1 = Edge::direction(store, edges[prev_idx]);
            let d2 = Edge::direction(store, edges[i]);

            let dot = d1.dot(d2).clamp(-1.0, 1.0);
            let angle = std::f64::consts::PI - dot.acos();
            angles.push(angle.to_degrees());
        }

        if !is_closed && edges.len() >= 2 {
            // Add the last angle for non-closed wires
            let d1 = Edge::direction(store, edges[n - 2]);
            let d2 = Edge::direction(store, edges[n - 1]);
            let dot = d1.dot(d2).clamp(-1.0, 1.0);
            let angle = std::f64::consts::PI - dot.acos();
            angles.push(angle.to_degrees());
        }

        angles
    }

    /// Get exterior angles at each vertex
    pub fn exterior_angles(store: &TopologyStore, handle: WireHandle) -> Vec<f64> {
        Self::interior_angles(store, handle)
            .iter()
            .map(|a| 180.0 - a)
            .collect()
    }

    /// Check if two wires are similar (same shape, possibly scaled/rotated)
    pub fn is_similar(
        store: &TopologyStore,
        wire_a: WireHandle,
        wire_b: WireHandle,
        angle_tolerance: f64,
    ) -> bool {
        let angles_a = Self::interior_angles(store, wire_a);
        let angles_b = Self::interior_angles(store, wire_b);

        if angles_a.len() != angles_b.len() {
            return false;
        }

        // Check if angles match (possibly rotated)
        let n = angles_a.len();
        for start in 0..n {
            let mut matches = true;
            for i in 0..n {
                let diff = (angles_a[i] - angles_b[(start + i) % n]).abs();
                if diff > angle_tolerance {
                    matches = false;
                    break;
                }
            }
            if matches {
                return true;
            }
        }

        false
    }

    /// Split wire into individual edges
    pub fn split(store: &TopologyStore, handle: WireHandle) -> Vec<WireHandle> {
        Self::edges(store, handle)
            .iter()
            .map(|e| Wire::by_edges(store, vec![*e]))
            .collect()
    }

    /// Create an offset wire (2D offset in XY plane)
    pub fn by_offset(
        store: &TopologyStore,
        wire: WireHandle,
        offset: f64,
        miter_threshold: f64,
    ) -> WireHandle {
        let vertices = Self::vertices(store, wire);
        let is_closed = Self::is_closed(store, wire);
        let n = vertices.len();

        if n < 2 {
            return wire;
        }

        // Get edge normals (perpendicular in 2D)
        let edges = Self::edges(store, wire);
        let mut edge_normals: Vec<Vector3> = Vec::with_capacity(edges.len());

        for edge in &edges {
            let dir = Edge::direction(store, *edge);
            // Perpendicular in XY plane
            edge_normals.push(Vector3::new(-dir.y, dir.x, 0.0).normalize());
        }

        // Calculate offset vertices
        let mut offset_vertices = Vec::with_capacity(n);

        for i in 0..n {
            let p = Vertex::point(store, vertices[i]);

            if is_closed || (i > 0 && i < n - 1) {
                // Interior vertex: bisector offset
                let prev_idx = if i == 0 { edges.len() - 1 } else { i - 1 };
                let curr_idx = i.min(edges.len() - 1);

                let n1 = if prev_idx < edge_normals.len() { edge_normals[prev_idx] } else { edge_normals[0] };
                let n2 = if curr_idx < edge_normals.len() { edge_normals[curr_idx] } else { edge_normals[0] };

                // Bisector direction
                let bisector = (n1 + n2).normalize();

                // Calculate the miter length
                let cos_half = n1.dot(bisector).abs().max(0.001);
                let miter_length = offset / cos_half;

                // Apply miter threshold
                let actual_offset = if miter_length.abs() > miter_threshold.abs() * offset.abs() {
                    offset
                } else {
                    miter_length
                };

                let offset_point = p + bisector * actual_offset;
                offset_vertices.push(Vertex::by_point(store, offset_point));
            } else if i == 0 && !is_closed && !edge_normals.is_empty() {
                // Start vertex
                let offset_point = p + edge_normals[0] * offset;
                offset_vertices.push(Vertex::by_point(store, offset_point));
            } else if i == n - 1 && !is_closed && !edge_normals.is_empty() {
                // End vertex
                let last_normal = edge_normals.last().unwrap_or(&edge_normals[0]);
                let offset_point = p + *last_normal * offset;
                offset_vertices.push(Vertex::by_point(store, offset_point));
            }
        }

        Wire::by_vertices(store, offset_vertices, is_closed)
    }

    /// Create a fillet (rounded corners) on the wire
    pub fn fillet(
        store: &TopologyStore,
        wire: WireHandle,
        radius: f64,
        segments: usize,
    ) -> WireHandle {
        let vertices = Self::vertices(store, wire);
        let is_closed = Self::is_closed(store, wire);
        let n = vertices.len();
        let segments = segments.max(2);

        if n < 3 {
            return wire;
        }

        let edges = Self::edges(store, wire);
        let mut new_vertices = Vec::new();

        for i in 0..n {
            let prev_idx = if i == 0 { n - 1 } else { i - 1 };
            let has_prev = is_closed || i > 0;
            let has_next = is_closed || i < n - 1;

            if !has_prev || !has_next {
                // Keep endpoint as is
                new_vertices.push(vertices[i]);
                continue;
            }

            let p = Vertex::point(store, vertices[i]);
            let p_prev = Vertex::point(store, vertices[prev_idx]);
            let p_next = Vertex::point(store, vertices[(i + 1) % n]);

            // Directions from vertex
            let d1 = (p_prev - p).normalize();
            let d2 = (p_next - p).normalize();

            // Angle at vertex
            let cos_angle = d1.dot(d2).clamp(-1.0, 1.0);
            let angle = cos_angle.acos();

            if angle.abs() < 0.01 || (std::f64::consts::PI - angle).abs() < 0.01 {
                // Nearly straight or too sharp
                new_vertices.push(vertices[i]);
                continue;
            }

            // Calculate fillet tangent points
            let tan_dist = radius / (angle / 2.0).tan();

            let t1 = p + d1 * tan_dist;
            let t2 = p + d2 * tan_dist;

            // Calculate arc center
            let bisector = (d1 + d2).normalize();
            let center_dist = radius / (angle / 2.0).sin();
            let center = p + bisector * center_dist;

            // Generate arc points
            let start_angle = (t1 - center).y.atan2((t1 - center).x);
            let end_angle = (t2 - center).y.atan2((t2 - center).x);

            let mut angle_diff = end_angle - start_angle;
            if angle_diff.abs() > std::f64::consts::PI {
                if angle_diff > 0.0 {
                    angle_diff -= std::f64::consts::TAU;
                } else {
                    angle_diff += std::f64::consts::TAU;
                }
            }

            for j in 0..=segments {
                let t = j as f64 / segments as f64;
                let a = start_angle + angle_diff * t;
                let fillet_point = Point3::new(
                    center.x + radius * a.cos(),
                    center.y + radius * a.sin(),
                    p.z,
                );
                new_vertices.push(Vertex::by_point(store, fillet_point));
            }
        }

        Wire::by_vertices(store, new_vertices, is_closed)
    }

    /// Get the dictionary attached to this wire
    pub fn get_dictionary(store: &TopologyStore, handle: WireHandle) -> Dictionary {
        store.wires.read()[handle.index.index()].dictionary.clone()
    }

    /// Set the dictionary for this wire
    pub fn set_dictionary(store: &TopologyStore, handle: WireHandle, dict: Dictionary) {
        store.wires.write()[handle.index.index()].dictionary = dict;
    }

    /// Set a single key-value pair in the wire's dictionary
    pub fn set_dictionary_value(store: &TopologyStore, handle: WireHandle, key: &str, value: DictionaryValue) {
        store.wires.write()[handle.index.index()].dictionary.set(key, value);
    }

    /// Get a value from the wire's dictionary
    pub fn get_dictionary_value(store: &TopologyStore, handle: WireHandle, key: &str) -> Option<DictionaryValue> {
        store.wires.read()[handle.index.index()].dictionary.get(key).cloned()
    }
}

impl<'a> WireRef<'a> {
    pub fn edges(&self) -> Vec<EdgeHandle> {
        Wire::edges(self.store, self.handle)
    }

    pub fn vertices(&self) -> Vec<VertexHandle> {
        Wire::vertices(self.store, self.handle)
    }

    pub fn is_closed(&self) -> bool {
        Wire::is_closed(self.store, self.handle)
    }

    pub fn length(&self) -> f64 {
        Wire::length(self.store, self.handle)
    }

    pub fn area(&self) -> f64 {
        Wire::area(self.store, self.handle)
    }

    pub fn normal(&self) -> Option<Vector3> {
        Wire::normal(self.store, self.handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rectangle_wire() {
        let store = TopologyStore::new();
        let wire = Wire::rectangle(&store, Point3::ZERO, 2.0, 3.0);

        assert!(Wire::is_closed(&store, wire));
        assert_eq!(Wire::edges(&store, wire).len(), 4);
        assert!((Wire::length(&store, wire) - 10.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_wire_area() {
        let store = TopologyStore::new();
        let wire = Wire::rectangle(&store, Point3::ZERO, 2.0, 3.0);

        assert!((Wire::area(&store, wire) - 6.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_circle_wire() {
        let store = TopologyStore::new();
        let wire = Wire::circle(&store, Point3::ZERO, 1.0, 360);

        assert!(Wire::is_closed(&store, wire));
        // Circumference should be approximately 2*pi
        let length = Wire::length(&store, wire);
        assert!((length - std::f64::consts::TAU).abs() < 0.01);
    }

    #[test]
    fn test_wire_with_shared_edge() {
        let store = TopologyStore::new();

        // Create two vertices
        let v0 = Vertex::by_coordinates(&store, 0.0, 0.0, 0.0);
        let v1 = Vertex::by_coordinates(&store, 2.0, 0.0, 0.0);

        // Create wire v0 -> v1 (forward direction)
        let wire1 = Wire::by_vertices(&store, vec![v0, v1], false);
        let verts1 = Wire::vertices(&store, wire1);
        assert_eq!(verts1.len(), 2);
        assert_eq!(Vertex::point(&store, verts1[0]).x, 0.0);
        assert_eq!(Vertex::point(&store, verts1[1]).x, 2.0);

        // Create wire v1 -> v0 (reverse direction, should reuse the edge)
        let wire2 = Wire::by_vertices(&store, vec![v1, v0], false);
        let verts2 = Wire::vertices(&store, wire2);
        assert_eq!(verts2.len(), 2);
        assert_eq!(Vertex::point(&store, verts2[0]).x, 2.0, "Expected wire2 to start at v1 (2,0,0)");
        assert_eq!(Vertex::point(&store, verts2[1]).x, 0.0, "Expected wire2 to end at v0 (0,0,0)");
    }
}
