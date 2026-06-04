//! Constructive Solid Geometry operations

use crate::topology::*;
use crate::geometry::{Point3, Vector3};

/// CSG operations for creating complex shapes
pub struct Csg;

impl Csg {
    /// Create a box
    pub fn cube(store: &TopologyStore, center: Point3, size: f64) -> CellHandle {
        let half = size / 2.0;
        let origin = center - Vector3::new(half, half, half);
        Cell::box_cell(store, origin, size, size, size)
    }

    /// Create a rectangular box
    pub fn box_shape(
        store: &TopologyStore,
        center: Point3,
        width: f64,
        length: f64,
        height: f64,
    ) -> CellHandle {
        let origin = center - Vector3::new(width / 2.0, length / 2.0, height / 2.0);
        Cell::box_cell(store, origin, width, length, height)
    }

    /// Create a sphere
    pub fn sphere(
        store: &TopologyStore,
        center: Point3,
        radius: f64,
        segments: usize,
    ) -> CellHandle {
        Cell::sphere(store, center, radius, segments, segments / 2)
    }

    /// Create a cylinder
    pub fn cylinder(
        store: &TopologyStore,
        center: Point3,
        radius: f64,
        height: f64,
        segments: usize,
    ) -> CellHandle {
        let bottom_center = center - Vector3::new(0.0, 0.0, height / 2.0);
        Cell::cylinder(store, bottom_center, radius, height, segments)
    }

    /// Create a cone
    pub fn cone(
        store: &TopologyStore,
        base_center: Point3,
        base_radius: f64,
        apex: Point3,
        segments: usize,
    ) -> CellHandle {
        let segments = segments.max(3);
        let mut faces = Vec::new();

        // Create base vertices
        let axis = (apex - base_center).normalize_or_zero();
        let (u, v) = {
            let u = if axis.x.abs() < 0.9 {
                Vector3::X.cross(axis).normalize()
            } else {
                Vector3::Y.cross(axis).normalize()
            };
            let v = axis.cross(u);
            (u, v)
        };

        let mut base_vertices = Vec::with_capacity(segments);
        for i in 0..segments {
            let angle = std::f64::consts::TAU * i as f64 / segments as f64;
            let p = base_center + (u * angle.cos() + v * angle.sin()) * base_radius;
            base_vertices.push(Vertex::by_point(store, p));
        }

        let apex_vertex = Vertex::by_point(store, apex);

        // Base face
        let base_wire = Wire::by_vertices(store, base_vertices.clone(), true);
        faces.push(Face::by_external_boundary(store, Wire::reverse(store, base_wire)));

        // Side faces
        for i in 0..segments {
            let j = (i + 1) % segments;
            faces.push(Face::by_vertices(
                store,
                vec![base_vertices[i], apex_vertex, base_vertices[j]],
            ));
        }

        Cell::by_faces(store, faces)
    }

    /// Create a torus
    pub fn torus(
        store: &TopologyStore,
        center: Point3,
        major_radius: f64,
        minor_radius: f64,
        major_segments: usize,
        minor_segments: usize,
    ) -> CellHandle {
        let major_segments = major_segments.max(3);
        let minor_segments = minor_segments.max(3);

        let mut rings: Vec<Vec<VertexHandle>> = Vec::with_capacity(major_segments);

        for i in 0..major_segments {
            let theta = std::f64::consts::TAU * i as f64 / major_segments as f64;
            let ring_center = center + Vector3::new(
                major_radius * theta.cos(),
                major_radius * theta.sin(),
                0.0,
            );

            let mut ring = Vec::with_capacity(minor_segments);
            for j in 0..minor_segments {
                let phi = std::f64::consts::TAU * j as f64 / minor_segments as f64;
                let p = ring_center + Vector3::new(
                    minor_radius * phi.cos() * theta.cos(),
                    minor_radius * phi.cos() * theta.sin(),
                    minor_radius * phi.sin(),
                );
                ring.push(Vertex::by_point(store, p));
            }
            rings.push(ring);
        }

        let mut faces = Vec::new();

        for i in 0..major_segments {
            let i_next = (i + 1) % major_segments;
            for j in 0..minor_segments {
                let j_next = (j + 1) % minor_segments;
                faces.push(Face::by_vertices(
                    store,
                    vec![
                        rings[i][j],
                        rings[i][j_next],
                        rings[i_next][j_next],
                        rings[i_next][j],
                    ],
                ));
            }
        }

        Cell::by_faces(store, faces)
    }

    /// Extrude a face along a vector
    pub fn extrude(
        store: &TopologyStore,
        face: FaceHandle,
        direction: Vector3,
    ) -> CellHandle {
        Cell::prism(store, face, direction.length())
    }

    /// Revolve a face around an axis
    pub fn revolve(
        store: &TopologyStore,
        face: FaceHandle,
        axis_origin: Point3,
        axis_direction: Vector3,
        angle: f64,
        segments: usize,
    ) -> CellHandle {
        let segments = segments.max(3);
        let vertices = Face::vertices(store, face);
        let angle_step = angle / segments as f64;

        let axis = axis_direction.normalize_or_zero();

        // Generate rotated vertices for each segment
        let mut rings: Vec<Vec<VertexHandle>> = Vec::new();

        for i in 0..=segments {
            let current_angle = angle_step * i as f64;
            let cos_a = current_angle.cos();
            let sin_a = current_angle.sin();

            let ring: Vec<_> = vertices
                .iter()
                .map(|v| {
                    let p = Vertex::point(store, *v);
                    let d = p - axis_origin;

                    // Rodrigues' rotation
                    let rotated = d * cos_a
                        + axis.cross(d) * sin_a
                        + axis * axis.dot(d) * (1.0 - cos_a);

                    Vertex::by_point(store, axis_origin + rotated)
                })
                .collect();

            rings.push(ring);
        }

        // Create faces
        let mut faces = Vec::new();

        // Side faces
        for i in 0..segments {
            for j in 0..vertices.len() {
                let j_next = (j + 1) % vertices.len();
                faces.push(Face::by_vertices(
                    store,
                    vec![
                        rings[i][j],
                        rings[i][j_next],
                        rings[i + 1][j_next],
                        rings[i + 1][j],
                    ],
                ));
            }
        }

        // End caps if not full revolution
        if (angle - std::f64::consts::TAU).abs() > 0.01 {
            // Start cap
            let start_wire = Wire::by_vertices(store, rings[0].clone(), true);
            faces.push(Face::by_external_boundary(store, start_wire));

            // End cap
            let end_wire = Wire::by_vertices(store, rings[segments].clone(), true);
            faces.push(Face::by_external_boundary(store, Wire::reverse(store, end_wire)));
        }

        Cell::by_faces(store, faces)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csg_cube() {
        let store = TopologyStore::new();
        let cube = Csg::cube(&store, Point3::ZERO, 2.0);

        assert_eq!(Cell::faces(&store, cube).len(), 6);
        assert!((Cell::volume(&store, cube) - 8.0).abs() < 0.5);
    }

    #[test]
    fn test_csg_sphere() {
        let store = TopologyStore::new();
        let sphere = Csg::sphere(&store, Point3::ZERO, 1.0, 16);

        // Volume should be approximately 4/3 * pi * r^3
        let expected_volume = 4.0 / 3.0 * std::f64::consts::PI;
        let actual_volume = Cell::volume(&store, sphere);

        // Allow for approximation error due to tessellation
        assert!((actual_volume - expected_volume).abs() < 0.5);
    }

    #[test]
    fn test_csg_cylinder() {
        let store = TopologyStore::new();
        let cylinder = Csg::cylinder(&store, Point3::ZERO, 1.0, 2.0, 32);

        // Volume should be approximately pi * r^2 * h
        let expected_volume = std::f64::consts::PI * 2.0;
        let actual_volume = Cell::volume(&store, cylinder);

        assert!((actual_volume - expected_volume).abs() < 0.3);
    }
}
