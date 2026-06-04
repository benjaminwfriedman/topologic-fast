//! Core topology data structures
//!
//! This module implements non-manifold topology with arena-based storage
//! for cache-friendly access and efficient memory management.

mod store;
mod vertex;
mod edge;
mod wire;
mod face;
mod shell;
mod cell;
mod cell_complex;
mod cluster;
mod traits;
mod navigation;
mod dictionary;
mod transform;

pub use store::*;
pub use vertex::*;
pub use edge::*;
pub use wire::*;
pub use face::*;
pub use shell::*;
pub use cell::*;
pub use cell_complex::*;
pub use cluster::*;
pub use traits::*;
pub use navigation::*;
pub use dictionary::*;
pub use transform::*;

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

/// Global topology ID counter
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Unique identifier for a topology entity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TopologyId(u64);

impl TopologyId {
    /// Generate a new unique ID
    #[inline]
    pub fn new() -> Self {
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// Get the raw ID value
    #[inline]
    pub fn raw(&self) -> u64 {
        self.0
    }
}

impl Default for TopologyId {
    fn default() -> Self {
        Self::new()
    }
}

/// Type of topological entity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum TopologyType {
    Vertex = 0,
    Edge = 1,
    Wire = 2,
    Face = 3,
    Shell = 4,
    Cell = 5,
    CellComplex = 6,
    Cluster = 7,
}

impl TopologyType {
    /// Get the dimensionality of this topology type
    #[inline]
    pub const fn dimensionality(&self) -> u8 {
        match self {
            TopologyType::Vertex => 0,
            TopologyType::Edge => 1,
            TopologyType::Wire => 1,
            TopologyType::Face => 2,
            TopologyType::Shell => 2,
            TopologyType::Cell => 3,
            TopologyType::CellComplex => 3,
            TopologyType::Cluster => 3,
        }
    }

    /// Check if this type is a container type
    #[inline]
    pub const fn is_container(&self) -> bool {
        matches!(
            self,
            TopologyType::Wire
                | TopologyType::Shell
                | TopologyType::CellComplex
                | TopologyType::Cluster
        )
    }
}

/// Common topology handle that can reference any topology type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TopologyHandle {
    Vertex(VertexHandle),
    Edge(EdgeHandle),
    Wire(WireHandle),
    Face(FaceHandle),
    Shell(ShellHandle),
    Cell(CellHandle),
    CellComplex(CellComplexHandle),
    Cluster(ClusterHandle),
}

impl TopologyHandle {
    pub fn topology_type(&self) -> TopologyType {
        match self {
            TopologyHandle::Vertex(_) => TopologyType::Vertex,
            TopologyHandle::Edge(_) => TopologyType::Edge,
            TopologyHandle::Wire(_) => TopologyType::Wire,
            TopologyHandle::Face(_) => TopologyType::Face,
            TopologyHandle::Shell(_) => TopologyType::Shell,
            TopologyHandle::Cell(_) => TopologyType::Cell,
            TopologyHandle::CellComplex(_) => TopologyType::CellComplex,
            TopologyHandle::Cluster(_) => TopologyType::Cluster,
        }
    }
}

/// Index into a topology arena
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArenaIndex(u32);

impl ArenaIndex {
    #[inline]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    #[inline]
    pub const fn index(&self) -> usize {
        self.0 as usize
    }
}
