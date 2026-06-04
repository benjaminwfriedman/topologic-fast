//! Common traits for topology operations

use super::*;
use crate::geometry::Point3;

/// Common operations for all topology types
pub trait Topology: Sized {
    /// Get the topology type
    fn topology_type() -> TopologyType;

    /// Get the dimensionality (0=vertex, 1=edge, 2=face, 3=cell)
    fn dimensionality() -> u8 {
        Self::topology_type().dimensionality()
    }

    /// Check if this is a container type
    fn is_container() -> bool {
        Self::topology_type().is_container()
    }
}

impl Topology for VertexHandle {
    fn topology_type() -> TopologyType {
        TopologyType::Vertex
    }
}

impl Topology for EdgeHandle {
    fn topology_type() -> TopologyType {
        TopologyType::Edge
    }
}

impl Topology for WireHandle {
    fn topology_type() -> TopologyType {
        TopologyType::Wire
    }
}

impl Topology for FaceHandle {
    fn topology_type() -> TopologyType {
        TopologyType::Face
    }
}

impl Topology for ShellHandle {
    fn topology_type() -> TopologyType {
        TopologyType::Shell
    }
}

impl Topology for CellHandle {
    fn topology_type() -> TopologyType {
        TopologyType::Cell
    }
}

impl Topology for CellComplexHandle {
    fn topology_type() -> TopologyType {
        TopologyType::CellComplex
    }
}

impl Topology for ClusterHandle {
    fn topology_type() -> TopologyType {
        TopologyType::Cluster
    }
}

/// Trait for getting geometric center
pub trait HasCenterOfMass {
    fn center_of_mass(&self, store: &TopologyStore) -> Point3;
}

/// Trait for getting bounding box
pub trait HasBoundingBox {
    fn bounding_box(&self, store: &TopologyStore) -> (Point3, Point3);
}

/// Trait for topology with area
pub trait HasArea {
    fn area(&self, store: &TopologyStore) -> f64;
}

/// Trait for topology with volume
pub trait HasVolume {
    fn volume(&self, store: &TopologyStore) -> f64;
}

/// Trait for topology that can be manifold
pub trait Manifold {
    fn is_manifold(&self, store: &TopologyStore) -> bool;
}

// Implement traits for handle types

impl HasCenterOfMass for VertexHandle {
    fn center_of_mass(&self, store: &TopologyStore) -> Point3 {
        Vertex::center_of_mass(store, *self)
    }
}

impl HasCenterOfMass for EdgeHandle {
    fn center_of_mass(&self, store: &TopologyStore) -> Point3 {
        Edge::center_of_mass(store, *self)
    }
}

impl HasCenterOfMass for WireHandle {
    fn center_of_mass(&self, store: &TopologyStore) -> Point3 {
        Wire::center_of_mass(store, *self)
    }
}

impl HasCenterOfMass for FaceHandle {
    fn center_of_mass(&self, store: &TopologyStore) -> Point3 {
        Face::center_of_mass(store, *self)
    }
}

impl HasCenterOfMass for ShellHandle {
    fn center_of_mass(&self, store: &TopologyStore) -> Point3 {
        Shell::center_of_mass(store, *self)
    }
}

impl HasCenterOfMass for CellHandle {
    fn center_of_mass(&self, store: &TopologyStore) -> Point3 {
        Cell::center_of_mass(store, *self)
    }
}

impl HasCenterOfMass for CellComplexHandle {
    fn center_of_mass(&self, store: &TopologyStore) -> Point3 {
        CellComplex::center_of_mass(store, *self)
    }
}

impl HasCenterOfMass for ClusterHandle {
    fn center_of_mass(&self, store: &TopologyStore) -> Point3 {
        Cluster::center_of_mass(store, *self)
    }
}

impl HasArea for FaceHandle {
    fn area(&self, store: &TopologyStore) -> f64 {
        Face::area(store, *self)
    }
}

impl HasArea for ShellHandle {
    fn area(&self, store: &TopologyStore) -> f64 {
        Shell::area(store, *self)
    }
}

impl HasArea for CellHandle {
    fn area(&self, store: &TopologyStore) -> f64 {
        Cell::area(store, *self)
    }
}

impl HasArea for CellComplexHandle {
    fn area(&self, store: &TopologyStore) -> f64 {
        CellComplex::area(store, *self)
    }
}

impl HasVolume for ShellHandle {
    fn volume(&self, store: &TopologyStore) -> f64 {
        Shell::volume(store, *self)
    }
}

impl HasVolume for CellHandle {
    fn volume(&self, store: &TopologyStore) -> f64 {
        Cell::volume(store, *self)
    }
}

impl HasVolume for CellComplexHandle {
    fn volume(&self, store: &TopologyStore) -> f64 {
        CellComplex::volume(store, *self)
    }
}

impl Manifold for EdgeHandle {
    fn is_manifold(&self, store: &TopologyStore) -> bool {
        Edge::is_manifold(store, *self)
    }
}

impl Manifold for FaceHandle {
    fn is_manifold(&self, store: &TopologyStore) -> bool {
        Face::is_manifold(store, *self)
    }
}

impl Manifold for ShellHandle {
    fn is_manifold(&self, store: &TopologyStore) -> bool {
        Shell::is_manifold(store, *self)
    }
}

impl Manifold for CellHandle {
    fn is_manifold(&self, store: &TopologyStore) -> bool {
        Cell::is_manifold(store, *self)
    }
}

impl Manifold for CellComplexHandle {
    fn is_manifold(&self, store: &TopologyStore) -> bool {
        CellComplex::is_manifold(store, *self)
    }
}
