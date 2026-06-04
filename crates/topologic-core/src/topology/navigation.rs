//! Navigation utilities for traversing topology hierarchies

use super::*;

/// Navigate from one topology type to another
pub struct Navigate;

impl Navigate {
    /// Get vertices from any topology
    pub fn vertices(store: &TopologyStore, handle: TopologyHandle) -> Vec<VertexHandle> {
        match handle {
            TopologyHandle::Vertex(v) => vec![v],
            TopologyHandle::Edge(e) => {
                let (s, e) = Edge::vertices(store, e);
                vec![s, e]
            }
            TopologyHandle::Wire(w) => Wire::vertices(store, w),
            TopologyHandle::Face(f) => Face::vertices(store, f),
            TopologyHandle::Shell(s) => Shell::vertices(store, s),
            TopologyHandle::Cell(c) => Cell::vertices(store, c),
            TopologyHandle::CellComplex(cc) => CellComplex::vertices(store, cc),
            TopologyHandle::Cluster(cl) => Cluster::vertices(store, cl),
        }
    }

    /// Get edges from any topology
    pub fn edges(store: &TopologyStore, handle: TopologyHandle) -> Vec<EdgeHandle> {
        match handle {
            TopologyHandle::Vertex(v) => Vertex::edges(store, v),
            TopologyHandle::Edge(e) => vec![e],
            TopologyHandle::Wire(w) => Wire::edges(store, w),
            TopologyHandle::Face(f) => Face::edges(store, f),
            TopologyHandle::Shell(s) => Shell::edges(store, s),
            TopologyHandle::Cell(c) => Cell::edges(store, c),
            TopologyHandle::CellComplex(cc) => CellComplex::edges(store, cc),
            TopologyHandle::Cluster(cl) => Cluster::edges(store, cl),
        }
    }

    /// Get wires from any topology
    pub fn wires(store: &TopologyStore, handle: TopologyHandle) -> Vec<WireHandle> {
        match handle {
            TopologyHandle::Vertex(_) => vec![],
            TopologyHandle::Edge(e) => Edge::wires(store, e),
            TopologyHandle::Wire(w) => vec![w],
            TopologyHandle::Face(f) => Face::wires(store, f),
            TopologyHandle::Shell(s) => Shell::wires(store, s),
            TopologyHandle::Cell(c) => Cell::wires(store, c),
            TopologyHandle::CellComplex(cc) => CellComplex::wires(store, cc),
            TopologyHandle::Cluster(_) => vec![],
        }
    }

    /// Get faces from any topology
    pub fn faces(store: &TopologyStore, handle: TopologyHandle) -> Vec<FaceHandle> {
        match handle {
            TopologyHandle::Vertex(_) => vec![],
            TopologyHandle::Edge(e) => Edge::faces(store, e),
            TopologyHandle::Wire(w) => Wire::faces(store, w),
            TopologyHandle::Face(f) => vec![f],
            TopologyHandle::Shell(s) => Shell::faces(store, s),
            TopologyHandle::Cell(c) => Cell::faces(store, c),
            TopologyHandle::CellComplex(cc) => CellComplex::faces(store, cc),
            TopologyHandle::Cluster(cl) => Cluster::faces(store, cl),
        }
    }

    /// Get shells from any topology
    pub fn shells(store: &TopologyStore, handle: TopologyHandle) -> Vec<ShellHandle> {
        match handle {
            TopologyHandle::Vertex(_) => vec![],
            TopologyHandle::Edge(_) => vec![],
            TopologyHandle::Wire(_) => vec![],
            TopologyHandle::Face(f) => Face::shells(store, f),
            TopologyHandle::Shell(s) => vec![s],
            TopologyHandle::Cell(c) => Cell::shells(store, c),
            TopologyHandle::CellComplex(cc) => CellComplex::shells(store, cc),
            TopologyHandle::Cluster(_) => vec![],
        }
    }

    /// Get cells from any topology
    pub fn cells(store: &TopologyStore, handle: TopologyHandle) -> Vec<CellHandle> {
        match handle {
            TopologyHandle::Vertex(_) => vec![],
            TopologyHandle::Edge(_) => vec![],
            TopologyHandle::Wire(_) => vec![],
            TopologyHandle::Face(f) => Face::cells(store, f),
            TopologyHandle::Shell(s) => Shell::cells(store, s),
            TopologyHandle::Cell(c) => vec![c],
            TopologyHandle::CellComplex(cc) => CellComplex::cells(store, cc),
            TopologyHandle::Cluster(cl) => Cluster::cells(store, cl),
        }
    }

    /// Get cell complexes from any topology
    pub fn cell_complexes(store: &TopologyStore, handle: TopologyHandle) -> Vec<CellComplexHandle> {
        match handle {
            TopologyHandle::Cell(c) => Cell::cell_complexes(store, c),
            TopologyHandle::CellComplex(cc) => vec![cc],
            _ => vec![],
        }
    }

    /// Count subentities of a specific type
    pub fn count(store: &TopologyStore, handle: TopologyHandle, target_type: TopologyType) -> usize {
        match target_type {
            TopologyType::Vertex => Self::vertices(store, handle).len(),
            TopologyType::Edge => Self::edges(store, handle).len(),
            TopologyType::Wire => Self::wires(store, handle).len(),
            TopologyType::Face => Self::faces(store, handle).len(),
            TopologyType::Shell => Self::shells(store, handle).len(),
            TopologyType::Cell => Self::cells(store, handle).len(),
            TopologyType::CellComplex => Self::cell_complexes(store, handle).len(),
            TopologyType::Cluster => 0,
        }
    }

    /// Navigate up the hierarchy (get containing topologies)
    pub fn upward(store: &TopologyStore, handle: TopologyHandle) -> Vec<TopologyHandle> {
        match handle {
            TopologyHandle::Vertex(v) => {
                Vertex::edges(store, v).into_iter().map(TopologyHandle::Edge).collect()
            }
            TopologyHandle::Edge(e) => {
                let mut result: Vec<TopologyHandle> = Edge::wires(store, e)
                    .into_iter()
                    .map(TopologyHandle::Wire)
                    .collect();
                result.extend(Edge::faces(store, e).into_iter().map(TopologyHandle::Face));
                result
            }
            TopologyHandle::Wire(w) => {
                Wire::faces(store, w).into_iter().map(TopologyHandle::Face).collect()
            }
            TopologyHandle::Face(f) => {
                let mut result: Vec<TopologyHandle> = Face::shells(store, f)
                    .into_iter()
                    .map(TopologyHandle::Shell)
                    .collect();
                result.extend(Face::cells(store, f).into_iter().map(TopologyHandle::Cell));
                result
            }
            TopologyHandle::Shell(s) => {
                Shell::cells(store, s).into_iter().map(TopologyHandle::Cell).collect()
            }
            TopologyHandle::Cell(c) => {
                Cell::cell_complexes(store, c)
                    .into_iter()
                    .map(TopologyHandle::CellComplex)
                    .collect()
            }
            TopologyHandle::CellComplex(_) | TopologyHandle::Cluster(_) => vec![],
        }
    }

    /// Navigate down the hierarchy (get contained topologies)
    pub fn downward(store: &TopologyStore, handle: TopologyHandle) -> Vec<TopologyHandle> {
        match handle {
            TopologyHandle::Vertex(_) => vec![],
            TopologyHandle::Edge(e) => {
                let (s, end) = Edge::vertices(store, e);
                vec![TopologyHandle::Vertex(s), TopologyHandle::Vertex(end)]
            }
            TopologyHandle::Wire(w) => {
                Wire::edges(store, w).into_iter().map(TopologyHandle::Edge).collect()
            }
            TopologyHandle::Face(f) => {
                Face::wires(store, f).into_iter().map(TopologyHandle::Wire).collect()
            }
            TopologyHandle::Shell(s) => {
                Shell::faces(store, s).into_iter().map(TopologyHandle::Face).collect()
            }
            TopologyHandle::Cell(c) => {
                Cell::shells(store, c).into_iter().map(TopologyHandle::Shell).collect()
            }
            TopologyHandle::CellComplex(cc) => {
                CellComplex::cells(store, cc).into_iter().map(TopologyHandle::Cell).collect()
            }
            TopologyHandle::Cluster(cl) => {
                Cluster::members(store, cl)
            }
        }
    }

    /// Find shared topologies between two handles
    pub fn shared(
        store: &TopologyStore,
        h1: TopologyHandle,
        h2: TopologyHandle,
        target_type: TopologyType,
    ) -> Vec<TopologyHandle> {
        let set1: hashbrown::HashSet<_> = match target_type {
            TopologyType::Vertex => Self::vertices(store, h1).into_iter().map(TopologyHandle::Vertex).collect(),
            TopologyType::Edge => Self::edges(store, h1).into_iter().map(TopologyHandle::Edge).collect(),
            TopologyType::Face => Self::faces(store, h1).into_iter().map(TopologyHandle::Face).collect(),
            TopologyType::Cell => Self::cells(store, h1).into_iter().map(TopologyHandle::Cell).collect(),
            _ => return vec![],
        };

        let set2: hashbrown::HashSet<_> = match target_type {
            TopologyType::Vertex => Self::vertices(store, h2).into_iter().map(TopologyHandle::Vertex).collect(),
            TopologyType::Edge => Self::edges(store, h2).into_iter().map(TopologyHandle::Edge).collect(),
            TopologyType::Face => Self::faces(store, h2).into_iter().map(TopologyHandle::Face).collect(),
            TopologyType::Cell => Self::cells(store, h2).into_iter().map(TopologyHandle::Cell).collect(),
            _ => return vec![],
        };

        set1.intersection(&set2).copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point3;

    #[test]
    fn test_navigate_vertices() {
        let store = TopologyStore::new();
        let face = Face::rectangle(&store, Point3::ZERO, 2.0, 2.0);

        let vertices = Navigate::vertices(&store, TopologyHandle::Face(face));
        assert_eq!(vertices.len(), 4);
    }

    #[test]
    fn test_navigate_edges() {
        let store = TopologyStore::new();
        let face = Face::rectangle(&store, Point3::ZERO, 2.0, 2.0);

        let edges = Navigate::edges(&store, TopologyHandle::Face(face));
        assert_eq!(edges.len(), 4);
    }

    #[test]
    fn test_navigate_upward() {
        let store = TopologyStore::new();
        let face = Face::rectangle(&store, Point3::ZERO, 2.0, 2.0);
        let wire = Face::external_boundary(&store, face);
        let edges = Wire::edges(&store, wire);

        let up = Navigate::upward(&store, TopologyHandle::Edge(edges[0]));
        assert!(!up.is_empty());
    }
}
