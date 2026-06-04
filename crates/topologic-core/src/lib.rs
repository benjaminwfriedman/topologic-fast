//! # Topologic Fast
//!
//! A high-performance non-manifold topology library for architectural and
//! engineering applications. This is a complete rewrite optimized for speed.
//!
//! ## Key Features
//! - Zero-copy topology navigation
//! - SIMD-accelerated geometry operations
//! - Parallel boolean operations
//! - Cache-friendly arena-based storage
//! - BVH spatial indexing

pub mod topology;
pub mod geometry;
pub mod boolean;
pub mod spatial;
pub mod mesh;
pub mod utilities;

pub use topology::*;
pub use geometry::*;
pub use utilities::*;

/// Re-export common types
pub mod prelude {
    pub use crate::topology::{
        TopologyOps, TopologyTransform, TopologyType, TopologyId,
        Vertex, Edge, Wire, Face, Shell, Cell, CellComplex, Cluster,
        TopologyStore, Dictionary, DictionaryValue,
    };
    pub use crate::geometry::{Point3, Vector3, Matrix4, Transform};
    pub use crate::boolean::BooleanOp;
    pub use crate::spatial::BvhTree;
    pub use crate::utilities::{VectorUtil, MatrixUtil, Matrix4x4, GridUtil, GraphUtil, GraphHandle};
}
