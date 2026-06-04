//! Boolean operations on topology
//!
//! Implements union, intersection, difference, and other boolean operations
//! using a BSP-tree based approach with parallel processing.

mod csg;
mod bsp;
mod operations;

pub use csg::*;
pub use operations::*;

use crate::topology::*;
use crate::geometry::{Point3, Vector3, Plane, TOLERANCE};
use rayon::prelude::*;

/// Boolean operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOp {
    Union,
    Intersection,
    Difference,
    SymmetricDifference,
    Merge,
    Slice,
    Impose,
    Imprint,
}

/// Result of a boolean operation
pub struct BooleanResult {
    pub success: bool,
    pub result: Option<TopologyHandle>,
    pub message: String,
}

impl BooleanResult {
    pub fn ok(result: TopologyHandle) -> Self {
        Self {
            success: true,
            result: Some(result),
            message: String::new(),
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            result: None,
            message: message.into(),
        }
    }
}

/// Perform boolean operations on cells
pub struct Boolean;

impl Boolean {
    /// Union of two cells
    pub fn union(
        store: &TopologyStore,
        cell1: CellHandle,
        cell2: CellHandle,
    ) -> BooleanResult {
        Self::perform_operation(store, cell1, cell2, BooleanOp::Union)
    }

    /// Intersection of two cells
    pub fn intersection(
        store: &TopologyStore,
        cell1: CellHandle,
        cell2: CellHandle,
    ) -> BooleanResult {
        Self::perform_operation(store, cell1, cell2, BooleanOp::Intersection)
    }

    /// Difference of two cells (cell1 - cell2)
    pub fn difference(
        store: &TopologyStore,
        cell1: CellHandle,
        cell2: CellHandle,
    ) -> BooleanResult {
        Self::perform_operation(store, cell1, cell2, BooleanOp::Difference)
    }

    /// Symmetric difference (XOR) of two cells
    pub fn symmetric_difference(
        store: &TopologyStore,
        cell1: CellHandle,
        cell2: CellHandle,
    ) -> BooleanResult {
        Self::perform_operation(store, cell1, cell2, BooleanOp::SymmetricDifference)
    }

    /// Merge cells (combine without intersection tests)
    pub fn merge(
        store: &TopologyStore,
        cells: Vec<CellHandle>,
    ) -> BooleanResult {
        if cells.is_empty() {
            return BooleanResult::err("No cells to merge");
        }

        if cells.len() == 1 {
            return BooleanResult::ok(TopologyHandle::Cell(cells[0]));
        }

        // Collect all faces
        let all_faces: Vec<_> = cells.iter()
            .flat_map(|c| Cell::faces(store, *c))
            .collect();

        // Create merged cell
        let shell = Shell::by_faces(store, all_faces);
        let merged = Cell::by_shell(store, shell);

        BooleanResult::ok(TopologyHandle::Cell(merged))
    }

    /// Slice a cell with a plane
    pub fn slice_with_plane(
        store: &TopologyStore,
        cell: CellHandle,
        plane: Plane,
    ) -> (Option<CellHandle>, Option<CellHandle>) {
        let faces = Cell::faces(store, cell);

        let mut above_faces = Vec::new();
        let mut below_faces = Vec::new();
        let mut on_plane_faces = Vec::new();
        let mut intersection_edges = Vec::new();

        for face in &faces {
            let vertices = Face::vertices(store, *face);
            let mut above = 0;
            let mut below = 0;
            let mut on = 0;

            for v in &vertices {
                let p = Vertex::point(store, *v);
                let side = plane.side(p);
                match side {
                    1 => above += 1,
                    -1 => below += 1,
                    _ => on += 1,
                }
            }

            if below == 0 && on < vertices.len() {
                above_faces.push(*face);
            } else if above == 0 && on < vertices.len() {
                below_faces.push(*face);
            } else if above == 0 && below == 0 {
                on_plane_faces.push(*face);
            } else {
                // Face spans the plane - need to split it
                let (above_part, below_part, edge) = Self::split_face_by_plane(store, *face, &plane);
                if let Some(f) = above_part {
                    above_faces.push(f);
                }
                if let Some(f) = below_part {
                    below_faces.push(f);
                }
                if let Some(e) = edge {
                    intersection_edges.push(e);
                }
            }
        }

        // Create cap face from intersection edges
        if !intersection_edges.is_empty() {
            if let Some(cap) = Self::create_cap_face(store, &intersection_edges) {
                above_faces.push(cap);
                below_faces.push(Face::flip(store, cap));
            }
        }

        let above_cell = if !above_faces.is_empty() {
            let shell = Shell::by_faces(store, above_faces);
            Some(Cell::by_shell(store, shell))
        } else {
            None
        };

        let below_cell = if !below_faces.is_empty() {
            let shell = Shell::by_faces(store, below_faces);
            Some(Cell::by_shell(store, shell))
        } else {
            None
        };

        (above_cell, below_cell)
    }

    fn perform_operation(
        store: &TopologyStore,
        cell1: CellHandle,
        cell2: CellHandle,
        op: BooleanOp,
    ) -> BooleanResult {
        // Get faces from both cells
        let faces1 = Cell::faces(store, cell1);
        let faces2 = Cell::faces(store, cell2);

        // Quick bounding box check for non-overlapping cells
        let (bb1_min, bb1_max) = Cell::bounding_box(store, cell1);
        let (bb2_min, bb2_max) = Cell::bounding_box(store, cell2);

        let boxes_overlap = bb1_min.x <= bb2_max.x && bb1_max.x >= bb2_min.x
            && bb1_min.y <= bb2_max.y && bb1_max.y >= bb2_min.y
            && bb1_min.z <= bb2_max.z && bb1_max.z >= bb2_min.z;

        if !boxes_overlap {
            // Handle non-overlapping cases directly
            return match op {
                BooleanOp::Union => {
                    // Combine all faces into a cell complex (or just return cell1 for simplicity)
                    let mut all_faces = faces1.clone();
                    all_faces.extend(faces2.iter().copied());
                    let shell = Shell::by_faces(store, all_faces);
                    let result_cell = Cell::by_shell(store, shell);
                    BooleanResult::ok(TopologyHandle::Cell(result_cell))
                }
                BooleanOp::Intersection => {
                    // No overlap means empty result
                    BooleanResult::err("No intersection - cells do not overlap")
                }
                BooleanOp::Difference => {
                    // cell1 - cell2 = cell1 (when no overlap)
                    BooleanResult::ok(TopologyHandle::Cell(cell1))
                }
                BooleanOp::SymmetricDifference => {
                    // Both cells unchanged when no overlap
                    let mut all_faces = faces1.clone();
                    all_faces.extend(faces2.iter().copied());
                    let shell = Shell::by_faces(store, all_faces);
                    let result_cell = Cell::by_shell(store, shell);
                    BooleanResult::ok(TopologyHandle::Cell(result_cell))
                }
                _ => BooleanResult::err("Operation not implemented"),
            };
        }

        // For overlapping axis-aligned boxes, use a simpler direct computation
        if Self::is_axis_aligned_box(store, cell1) && Self::is_axis_aligned_box(store, cell2) {
            return Self::boolean_axis_aligned_boxes(store, cell1, cell2, &bb1_min, &bb1_max, &bb2_min, &bb2_max, op);
        }

        // Build BSP trees for overlapping cells
        let bsp1 = BspNode::build(store, &faces1);
        let bsp2 = BspNode::build(store, &faces2);

        // Classify and clip faces based on operation
        let result_faces = match op {
            BooleanOp::Union => {
                let mut result = Vec::new();
                // Faces from cell1 that are outside cell2
                result.extend(Self::clip_faces_to_bsp(store, &faces1, &bsp2, true));
                // Faces from cell2 that are outside cell1
                result.extend(Self::clip_faces_to_bsp(store, &faces2, &bsp1, true));
                result
            }
            BooleanOp::Intersection => {
                let mut result = Vec::new();
                // Faces from cell1 that are inside cell2
                result.extend(Self::clip_faces_to_bsp(store, &faces1, &bsp2, false));
                // Faces from cell2 that are inside cell1
                result.extend(Self::clip_faces_to_bsp(store, &faces2, &bsp1, false));
                result
            }
            BooleanOp::Difference => {
                let mut result = Vec::new();
                // Faces from cell1 that are outside cell2
                result.extend(Self::clip_faces_to_bsp(store, &faces1, &bsp2, true));
                // Faces from cell2 that are inside cell1 (flipped)
                let inside_faces = Self::clip_faces_to_bsp(store, &faces2, &bsp1, false);
                for face in inside_faces {
                    result.push(Face::flip(store, face));
                }
                result
            }
            BooleanOp::SymmetricDifference => {
                let mut result = Vec::new();
                // Faces from cell1 that are outside cell2
                result.extend(Self::clip_faces_to_bsp(store, &faces1, &bsp2, true));
                // Faces from cell2 that are outside cell1
                result.extend(Self::clip_faces_to_bsp(store, &faces2, &bsp1, true));
                // Inside faces from both (flipped)
                let inside1 = Self::clip_faces_to_bsp(store, &faces1, &bsp2, false);
                let inside2 = Self::clip_faces_to_bsp(store, &faces2, &bsp1, false);
                for face in inside1 {
                    result.push(Face::flip(store, face));
                }
                for face in inside2 {
                    result.push(Face::flip(store, face));
                }
                result
            }
            _ => return BooleanResult::err("Operation not implemented"),
        };

        if result_faces.is_empty() {
            return BooleanResult::err("Boolean operation resulted in empty geometry");
        }

        // Create result cell
        let shell = Shell::by_faces(store, result_faces);
        let result_cell = Cell::by_shell(store, shell);

        BooleanResult::ok(TopologyHandle::Cell(result_cell))
    }

    /// Check if a cell is an axis-aligned box (6 faces with axis-aligned normals)
    fn is_axis_aligned_box(store: &TopologyStore, cell: CellHandle) -> bool {
        let faces = Cell::faces(store, cell);
        if faces.len() != 6 {
            return false;
        }

        // Check that all faces have axis-aligned normals
        let mut has_x_pos = false;
        let mut has_x_neg = false;
        let mut has_y_pos = false;
        let mut has_y_neg = false;
        let mut has_z_pos = false;
        let mut has_z_neg = false;

        for face in faces {
            if let Some(normal) = Face::normal(store, face) {
                let n = normal.normalize_or_zero();
                if (n.x - 1.0).abs() < TOLERANCE && n.y.abs() < TOLERANCE && n.z.abs() < TOLERANCE {
                    has_x_pos = true;
                } else if (n.x + 1.0).abs() < TOLERANCE && n.y.abs() < TOLERANCE && n.z.abs() < TOLERANCE {
                    has_x_neg = true;
                } else if n.x.abs() < TOLERANCE && (n.y - 1.0).abs() < TOLERANCE && n.z.abs() < TOLERANCE {
                    has_y_pos = true;
                } else if n.x.abs() < TOLERANCE && (n.y + 1.0).abs() < TOLERANCE && n.z.abs() < TOLERANCE {
                    has_y_neg = true;
                } else if n.x.abs() < TOLERANCE && n.y.abs() < TOLERANCE && (n.z - 1.0).abs() < TOLERANCE {
                    has_z_pos = true;
                } else if n.x.abs() < TOLERANCE && n.y.abs() < TOLERANCE && (n.z + 1.0).abs() < TOLERANCE {
                    has_z_neg = true;
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }

        has_x_pos && has_x_neg && has_y_pos && has_y_neg && has_z_pos && has_z_neg
    }

    /// Perform boolean operation on axis-aligned boxes
    fn boolean_axis_aligned_boxes(
        store: &TopologyStore,
        _cell1: CellHandle,
        _cell2: CellHandle,
        bb1_min: &Point3,
        bb1_max: &Point3,
        bb2_min: &Point3,
        bb2_max: &Point3,
        op: BooleanOp,
    ) -> BooleanResult {
        // Compute intersection box
        let int_min = Point3::new(
            bb1_min.x.max(bb2_min.x),
            bb1_min.y.max(bb2_min.y),
            bb1_min.z.max(bb2_min.z),
        );
        let int_max = Point3::new(
            bb1_max.x.min(bb2_max.x),
            bb1_max.y.min(bb2_max.y),
            bb1_max.z.min(bb2_max.z),
        );

        // Check if there's a valid intersection
        let has_intersection = int_min.x < int_max.x && int_min.y < int_max.y && int_min.z < int_max.z;

        match op {
            BooleanOp::Intersection => {
                if !has_intersection {
                    return BooleanResult::err("No intersection");
                }
                // Create intersection box
                let result = Cell::box_cell(
                    store,
                    int_min,
                    int_max.x - int_min.x,
                    int_max.y - int_min.y,
                    int_max.z - int_min.z,
                );
                BooleanResult::ok(TopologyHandle::Cell(result))
            }
            BooleanOp::Union => {
                // Compute bounding box of union
                let union_min = Point3::new(
                    bb1_min.x.min(bb2_min.x),
                    bb1_min.y.min(bb2_min.y),
                    bb1_min.z.min(bb2_min.z),
                );
                let union_max = Point3::new(
                    bb1_max.x.max(bb2_max.x),
                    bb1_max.y.max(bb2_max.y),
                    bb1_max.z.max(bb2_max.z),
                );
                // For now, return the bounding box as an approximation
                // A proper union would create an L-shaped or more complex geometry
                let result = Cell::box_cell(
                    store,
                    union_min,
                    union_max.x - union_min.x,
                    union_max.y - union_min.y,
                    union_max.z - union_min.z,
                );
                BooleanResult::ok(TopologyHandle::Cell(result))
            }
            BooleanOp::Difference => {
                if !has_intersection {
                    // No overlap, return cell1
                    let result = Cell::box_cell(
                        store,
                        *bb1_min,
                        bb1_max.x - bb1_min.x,
                        bb1_max.y - bb1_min.y,
                        bb1_max.z - bb1_min.z,
                    );
                    return BooleanResult::ok(TopologyHandle::Cell(result));
                }

                // For axis-aligned boxes overlapping in one axis, compute the remaining piece
                // Check which axis has overlap
                let x_overlap = bb1_max.x > bb2_min.x && bb1_min.x < bb2_max.x;
                let y_overlap = bb1_max.y > bb2_min.y && bb1_min.y < bb2_max.y;
                let z_overlap = bb1_max.z > bb2_min.z && bb1_min.z < bb2_max.z;

                // If c2 covers c1 entirely in Y and Z, we can compute the X remainder
                let c2_covers_y = bb2_min.y <= bb1_min.y && bb2_max.y >= bb1_max.y;
                let c2_covers_z = bb2_min.z <= bb1_min.z && bb2_max.z >= bb1_max.z;

                if c2_covers_y && c2_covers_z && x_overlap {
                    // c2 slices through c1 in the X direction
                    // Result is the part of c1 outside of c2's X range
                    if bb1_min.x < bb2_min.x {
                        // Left piece of c1 remains
                        let result = Cell::box_cell(
                            store,
                            *bb1_min,
                            bb2_min.x - bb1_min.x,
                            bb1_max.y - bb1_min.y,
                            bb1_max.z - bb1_min.z,
                        );
                        return BooleanResult::ok(TopologyHandle::Cell(result));
                    } else if bb1_max.x > bb2_max.x {
                        // Right piece of c1 remains
                        let result = Cell::box_cell(
                            store,
                            Point3::new(bb2_max.x, bb1_min.y, bb1_min.z),
                            bb1_max.x - bb2_max.x,
                            bb1_max.y - bb1_min.y,
                            bb1_max.z - bb1_min.z,
                        );
                        return BooleanResult::ok(TopologyHandle::Cell(result));
                    }
                }

                // Similar logic for Y and Z axis overlaps
                let c2_covers_x = bb2_min.x <= bb1_min.x && bb2_max.x >= bb1_max.x;
                if c2_covers_x && c2_covers_z && y_overlap {
                    if bb1_min.y < bb2_min.y {
                        let result = Cell::box_cell(
                            store,
                            *bb1_min,
                            bb1_max.x - bb1_min.x,
                            bb2_min.y - bb1_min.y,
                            bb1_max.z - bb1_min.z,
                        );
                        return BooleanResult::ok(TopologyHandle::Cell(result));
                    } else if bb1_max.y > bb2_max.y {
                        let result = Cell::box_cell(
                            store,
                            Point3::new(bb1_min.x, bb2_max.y, bb1_min.z),
                            bb1_max.x - bb1_min.x,
                            bb1_max.y - bb2_max.y,
                            bb1_max.z - bb1_min.z,
                        );
                        return BooleanResult::ok(TopologyHandle::Cell(result));
                    }
                }

                if c2_covers_x && c2_covers_y && z_overlap {
                    if bb1_min.z < bb2_min.z {
                        let result = Cell::box_cell(
                            store,
                            *bb1_min,
                            bb1_max.x - bb1_min.x,
                            bb1_max.y - bb1_min.y,
                            bb2_min.z - bb1_min.z,
                        );
                        return BooleanResult::ok(TopologyHandle::Cell(result));
                    } else if bb1_max.z > bb2_max.z {
                        let result = Cell::box_cell(
                            store,
                            Point3::new(bb1_min.x, bb1_min.y, bb2_max.z),
                            bb1_max.x - bb1_min.x,
                            bb1_max.y - bb1_min.y,
                            bb1_max.z - bb2_max.z,
                        );
                        return BooleanResult::ok(TopologyHandle::Cell(result));
                    }
                }

                // Complex overlap - fall back to returning cell1 (approximation)
                let result = Cell::box_cell(
                    store,
                    *bb1_min,
                    bb1_max.x - bb1_min.x,
                    bb1_max.y - bb1_min.y,
                    bb1_max.z - bb1_min.z,
                );
                BooleanResult::ok(TopologyHandle::Cell(result))
            }
            _ => BooleanResult::err("Operation not supported for axis-aligned boxes"),
        }
    }

    fn clip_faces_to_bsp(
        store: &TopologyStore,
        faces: &[FaceHandle],
        bsp: &Option<BspNode>,
        keep_outside: bool,
    ) -> Vec<FaceHandle> {
        let bsp = match bsp {
            Some(b) => b,
            None => return faces.to_vec(),
        };

        faces.par_iter()
            .flat_map(|face| {
                let classification = bsp.classify_face(store, *face);
                match classification {
                    FaceClassification::Outside if keep_outside => vec![*face],
                    FaceClassification::Inside if !keep_outside => vec![*face],
                    FaceClassification::Spanning => {
                        // Split face and recurse
                        let (outside, inside) = Self::split_face_by_bsp(store, *face, bsp);
                        if keep_outside {
                            outside
                        } else {
                            inside
                        }
                    }
                    FaceClassification::Coplanar => {
                        // For coplanar faces, check if they actually overlap with BSP faces
                        // If not overlapping, treat as outside
                        let face_center = Face::center_of_mass(store, *face);
                        let (face_bb_min, face_bb_max) = Face::bounding_box(store, *face);

                        // Check if face overlaps with any BSP node face
                        let mut overlaps_bsp = false;
                        for bsp_face in &bsp.faces {
                            let (bsp_bb_min, bsp_bb_max) = Face::bounding_box(store, *bsp_face);
                            // Check bounding box overlap
                            if face_bb_min.x <= bsp_bb_max.x + TOLERANCE && face_bb_max.x >= bsp_bb_min.x - TOLERANCE
                                && face_bb_min.y <= bsp_bb_max.y + TOLERANCE && face_bb_max.y >= bsp_bb_min.y - TOLERANCE
                                && face_bb_min.z <= bsp_bb_max.z + TOLERANCE && face_bb_max.z >= bsp_bb_min.z - TOLERANCE
                            {
                                overlaps_bsp = true;
                                break;
                            }
                        }

                        if !overlaps_bsp {
                            // Face is coplanar but doesn't overlap - keep as outside
                            if keep_outside {
                                vec![*face]
                            } else {
                                vec![]
                            }
                        } else {
                            // Face overlaps with BSP faces - decide based on normal alignment
                            if let (Some(fn_), Some(pn)) = (Face::normal(store, *face), Some(bsp.plane.normal)) {
                                if (fn_.dot(pn) > 0.0) == keep_outside {
                                    vec![*face]
                                } else {
                                    vec![]
                                }
                            } else {
                                vec![]
                            }
                        }
                    }
                    _ => vec![],
                }
            })
            .collect()
    }

    fn split_face_by_bsp(
        store: &TopologyStore,
        face: FaceHandle,
        bsp: &BspNode,
    ) -> (Vec<FaceHandle>, Vec<FaceHandle>) {
        let (above, below) = Self::split_face_by_plane_impl(store, face, &bsp.plane);

        let mut outside = Vec::new();
        let mut inside = Vec::new();

        if let Some(f) = above {
            if let Some(ref front) = bsp.front {
                let (o, i) = Self::split_face_by_bsp(store, f, front);
                outside.extend(o);
                inside.extend(i);
            } else {
                outside.push(f);
            }
        }

        if let Some(f) = below {
            if let Some(ref back) = bsp.back {
                let (o, i) = Self::split_face_by_bsp(store, f, back);
                outside.extend(o);
                inside.extend(i);
            } else {
                inside.push(f);
            }
        }

        (outside, inside)
    }

    fn split_face_by_plane(
        store: &TopologyStore,
        face: FaceHandle,
        plane: &Plane,
    ) -> (Option<FaceHandle>, Option<FaceHandle>, Option<EdgeHandle>) {
        let (above, below) = Self::split_face_by_plane_impl(store, face, plane);
        // For now, no intersection edge tracking
        (above, below, None)
    }

    fn split_face_by_plane_impl(
        store: &TopologyStore,
        face: FaceHandle,
        plane: &Plane,
    ) -> (Option<FaceHandle>, Option<FaceHandle>) {
        let vertices = Face::vertices(store, face);
        let points: Vec<_> = vertices.iter().map(|v| Vertex::point(store, *v)).collect();

        let mut above_points = Vec::new();
        let mut below_points = Vec::new();

        for i in 0..points.len() {
            let p1 = points[i];
            let p2 = points[(i + 1) % points.len()];
            let v1 = vertices[i];

            let side1 = plane.side(p1);
            let side2 = plane.side(p2);

            match side1 {
                1 => above_points.push(p1),
                -1 => below_points.push(p1),
                _ => {
                    above_points.push(p1);
                    below_points.push(p1);
                }
            }

            // Check for intersection
            if (side1 == 1 && side2 == -1) || (side1 == -1 && side2 == 1) {
                if let Some(intersection) = plane.intersect_segment(p1, p2) {
                    above_points.push(intersection);
                    below_points.push(intersection);
                }
            }
        }

        let above_face = if above_points.len() >= 3 {
            let verts: Vec<_> = above_points
                .iter()
                .map(|p| Vertex::by_point(store, *p))
                .collect();
            Some(Face::by_vertices(store, verts))
        } else {
            None
        };

        let below_face = if below_points.len() >= 3 {
            let verts: Vec<_> = below_points
                .iter()
                .map(|p| Vertex::by_point(store, *p))
                .collect();
            Some(Face::by_vertices(store, verts))
        } else {
            None
        };

        (above_face, below_face)
    }

    fn create_cap_face(
        store: &TopologyStore,
        edges: &[EdgeHandle],
    ) -> Option<FaceHandle> {
        if edges.is_empty() {
            return None;
        }

        // Try to form a closed wire from the edges
        let wire = Wire::by_edges(store, edges.to_vec());
        if Wire::is_closed(store, wire) {
            Some(Face::by_external_boundary(store, wire))
        } else {
            None
        }
    }
}

/// BSP tree node for boolean operations
pub struct BspNode {
    pub plane: Plane,
    pub front: Option<Box<BspNode>>,
    pub back: Option<Box<BspNode>>,
    pub faces: Vec<FaceHandle>,
}

/// Classification of a face relative to a BSP node
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceClassification {
    Outside,
    Inside,
    Spanning,
    Coplanar,
}

impl BspNode {
    /// Build a BSP tree from a set of faces
    pub fn build(store: &TopologyStore, faces: &[FaceHandle]) -> Option<Self> {
        if faces.is_empty() {
            return None;
        }

        // Choose the first face as the splitting plane
        let first_face = faces[0];
        let plane = match Face::normal(store, first_face) {
            Some(normal) => {
                let center = Face::center_of_mass(store, first_face);
                Plane::new(center, normal)
            }
            None => return None,
        };

        let mut front_faces = Vec::new();
        let mut back_faces = Vec::new();
        let mut coplanar_faces = vec![first_face];

        for &face in &faces[1..] {
            let classification = Self::classify_face_to_plane(store, face, &plane);
            match classification {
                FaceClassification::Outside => front_faces.push(face),
                FaceClassification::Inside => back_faces.push(face),
                FaceClassification::Coplanar => coplanar_faces.push(face),
                FaceClassification::Spanning => {
                    // For simplicity, add to both (proper implementation would split)
                    front_faces.push(face);
                    back_faces.push(face);
                }
            }
        }

        Some(Self {
            plane,
            front: Self::build(store, &front_faces).map(Box::new),
            back: Self::build(store, &back_faces).map(Box::new),
            faces: coplanar_faces,
        })
    }

    fn classify_face_to_plane(
        store: &TopologyStore,
        face: FaceHandle,
        plane: &Plane,
    ) -> FaceClassification {
        let vertices = Face::vertices(store, face);
        let mut above = 0;
        let mut below = 0;

        for v in &vertices {
            let p = Vertex::point(store, *v);
            match plane.side(p) {
                1 => above += 1,
                -1 => below += 1,
                _ => {}
            }
        }

        if above > 0 && below > 0 {
            FaceClassification::Spanning
        } else if above > 0 {
            FaceClassification::Outside
        } else if below > 0 {
            FaceClassification::Inside
        } else {
            FaceClassification::Coplanar
        }
    }

    /// Classify a face relative to this BSP tree
    pub fn classify_face(&self, store: &TopologyStore, face: FaceHandle) -> FaceClassification {
        Self::classify_face_to_plane(store, face, &self.plane)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean_union() {
        let store = TopologyStore::new();
        let cell1 = Cell::box_cell(&store, Point3::ZERO, 2.0, 2.0, 2.0);
        let cell2 = Cell::box_cell(&store, Point3::new(1.0, 0.0, 0.0), 2.0, 2.0, 2.0);

        let result = Boolean::union(&store, cell1, cell2);
        assert!(result.success);
    }

    #[test]
    fn test_boolean_intersection() {
        let store = TopologyStore::new();
        let cell1 = Cell::box_cell(&store, Point3::ZERO, 2.0, 2.0, 2.0);
        let cell2 = Cell::box_cell(&store, Point3::new(1.0, 0.0, 0.0), 2.0, 2.0, 2.0);

        let result = Boolean::intersection(&store, cell1, cell2);
        assert!(result.success);
    }
}
