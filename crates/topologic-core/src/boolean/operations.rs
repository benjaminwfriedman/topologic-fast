//! Additional boolean-like operations

use crate::topology::*;
use crate::geometry::{Point3, Vector3, Plane, TOLERANCE};

/// Self-merge operation to clean up topology
pub fn self_merge(store: &TopologyStore, cell: CellHandle) -> CellHandle {
    let faces = Cell::faces(store, cell);

    // Remove duplicate faces
    let mut unique_faces = Vec::new();
    let mut seen = hashbrown::HashSet::new();

    for face in faces {
        let center = Face::center_of_mass(store, face);
        let key = (
            (center.x * 1000.0) as i64,
            (center.y * 1000.0) as i64,
            (center.z * 1000.0) as i64,
        );

        if seen.insert(key) {
            unique_faces.push(face);
        }
    }

    Cell::by_faces(store, unique_faces)
}

/// Impose operation: project one cell onto another
pub fn impose(
    store: &TopologyStore,
    tool: CellHandle,
    target: CellHandle,
) -> (CellHandle, Vec<CellHandle>) {
    let target_faces = Cell::faces(store, target);
    let tool_faces = Cell::faces(store, tool);

    let mut modified_faces = Vec::new();
    let mut intersection_cells = Vec::new();

    // For each face of the target, check intersections with tool
    for target_face in &target_faces {
        let mut face_modified = false;

        for tool_face in &tool_faces {
            // Check if faces intersect
            if faces_intersect(store, *target_face, *tool_face) {
                face_modified = true;
                // Split the target face by the tool face's plane
                if let Some(normal) = Face::normal(store, *tool_face) {
                    let center = Face::center_of_mass(store, *tool_face);
                    let plane = Plane::new(center, normal);

                    // Add split faces
                    let vertices = Face::vertices(store, *target_face);
                    let points: Vec<_> = vertices.iter()
                        .map(|v| Vertex::point(store, *v))
                        .collect();

                    let intersection_pts: Vec<_> = points.windows(2)
                        .filter_map(|pair| plane.intersect_segment(pair[0], pair[1]))
                        .collect();

                    if intersection_pts.len() >= 2 {
                        // Create intersection face
                        let int_verts: Vec<_> = intersection_pts.iter()
                            .map(|p| Vertex::by_point(store, *p))
                            .collect();

                        if int_verts.len() >= 3 {
                            let int_face = Face::by_vertices(store, int_verts);
                            modified_faces.push(int_face);
                        }
                    }
                }
            }
        }

        if !face_modified {
            modified_faces.push(*target_face);
        }
    }

    let result_cell = Cell::by_faces(store, modified_faces);
    (result_cell, intersection_cells)
}

/// Imprint operation: project edges onto faces
pub fn imprint(
    store: &TopologyStore,
    tool: CellHandle,
    target: CellHandle,
) -> CellHandle {
    let target_faces = Cell::faces(store, target);
    let tool_edges = Cell::edges(store, tool);

    let mut result_faces = Vec::new();

    for target_face in &target_faces {
        let face_plane = match Face::normal(store, *target_face) {
            Some(normal) => {
                let center = Face::center_of_mass(store, *target_face);
                Plane::new(center, normal)
            }
            None => {
                result_faces.push(*target_face);
                continue;
            }
        };

        let mut imprint_points = Vec::new();

        for tool_edge in &tool_edges {
            let start = Edge::start_point(store, *tool_edge);
            let end = Edge::end_point(store, *tool_edge);

            // Find intersection with face plane
            if let Some(intersection) = face_plane.intersect_segment(start, end) {
                // Check if intersection is within the face
                if Face::contains_point(store, *target_face, intersection) {
                    imprint_points.push(intersection);
                }
            }
        }

        if imprint_points.is_empty() {
            result_faces.push(*target_face);
        } else {
            // Create imprinted face with additional vertices
            let mut vertices = Face::vertices(store, *target_face);

            // Add imprint points as new vertices
            for point in imprint_points {
                let new_vertex = Vertex::by_point(store, point);
                // Insert in appropriate position (simplified)
                vertices.push(new_vertex);
            }

            result_faces.push(Face::by_vertices(store, vertices));
        }
    }

    Cell::by_faces(store, result_faces)
}

/// Check if two faces intersect
fn faces_intersect(store: &TopologyStore, f1: FaceHandle, f2: FaceHandle) -> bool {
    let (bb1_min, bb1_max) = Face::bounding_box(store, f1);
    let (bb2_min, bb2_max) = Face::bounding_box(store, f2);

    // Quick bounding box check
    if bb1_max.x < bb2_min.x || bb1_min.x > bb2_max.x {
        return false;
    }
    if bb1_max.y < bb2_min.y || bb1_min.y > bb2_max.y {
        return false;
    }
    if bb1_max.z < bb2_min.z || bb1_min.z > bb2_max.z {
        return false;
    }

    // More detailed intersection test would go here
    true
}

/// Divide a cell into multiple cells
pub fn divide(
    store: &TopologyStore,
    cell: CellHandle,
    dividers: &[Plane],
) -> Vec<CellHandle> {
    let mut current_cells = vec![cell];

    for plane in dividers {
        let mut next_cells = Vec::new();

        for current in current_cells {
            let (above, below) = super::Boolean::slice_with_plane(store, current, *plane);
            if let Some(a) = above {
                next_cells.push(a);
            }
            if let Some(b) = below {
                next_cells.push(b);
            }
        }

        current_cells = next_cells;
    }

    current_cells
}

/// Scale a cell around a center point
pub fn scale_cell(
    store: &TopologyStore,
    cell: CellHandle,
    center: Point3,
    factor: f64,
) -> CellHandle {
    let vertices = Cell::vertices(store, cell);
    let new_vertices: Vec<_> = vertices.iter()
        .map(|v| {
            let p = Vertex::point(store, *v);
            let scaled = center + (p - center) * factor;
            Vertex::by_point(store, scaled)
        })
        .collect();

    // Rebuild the cell with scaled vertices
    let faces = Cell::faces(store, cell);
    let new_faces: Vec<_> = faces.iter()
        .map(|f| {
            let face_vertices = Face::vertices(store, *f);
            let new_face_vertices: Vec<_> = face_vertices.iter()
                .map(|v| {
                    let p = Vertex::point(store, *v);
                    let idx = vertices.iter().position(|x| *x == *v).unwrap();
                    new_vertices[idx]
                })
                .collect();
            Face::by_vertices(store, new_face_vertices)
        })
        .collect();

    Cell::by_faces(store, new_faces)
}

/// Offset a cell (move all faces outward by a distance)
pub fn offset_cell(
    store: &TopologyStore,
    cell: CellHandle,
    distance: f64,
) -> CellHandle {
    let vertices = Cell::vertices(store, cell);

    // Calculate vertex normals (average of adjacent face normals)
    let mut vertex_normals: hashbrown::HashMap<VertexHandle, Vector3> = hashbrown::HashMap::new();

    for vertex in &vertices {
        let edges = Vertex::edges(store, *vertex);
        let mut normal_sum = Vector3::ZERO;
        let mut count = 0;

        for edge in edges {
            for face in Edge::faces(store, edge) {
                if Cell::faces(store, cell).contains(&face) {
                    if let Some(n) = Face::normal(store, face) {
                        normal_sum += n;
                        count += 1;
                    }
                }
            }
        }

        if count > 0 {
            vertex_normals.insert(*vertex, (normal_sum / count as f64).normalize_or_zero());
        } else {
            vertex_normals.insert(*vertex, Vector3::Z);
        }
    }

    // Create offset vertices
    let new_vertices: Vec<_> = vertices.iter()
        .map(|v| {
            let p = Vertex::point(store, *v);
            let normal = vertex_normals.get(v).copied().unwrap_or(Vector3::Z);
            Vertex::by_point(store, p + normal * distance)
        })
        .collect();

    // Rebuild faces
    let faces = Cell::faces(store, cell);
    let new_faces: Vec<_> = faces.iter()
        .map(|f| {
            let face_vertices = Face::vertices(store, *f);
            let new_face_vertices: Vec<_> = face_vertices.iter()
                .map(|v| {
                    let idx = vertices.iter().position(|x| *x == *v).unwrap();
                    new_vertices[idx]
                })
                .collect();
            Face::by_vertices(store, new_face_vertices)
        })
        .collect();

    Cell::by_faces(store, new_faces)
}
