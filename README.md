# topologic-fast

A high-performance non-manifold topology library for architectural and engineering applications. This is a ground-up rewrite of [Topologic](https://github.com/wassimj/Topologic) optimized for speed.

## Features

- **High Performance**: Written in Rust with SIMD-optimized geometry operations
- **Thread-Safe**: Full support for parallel processing with rayon
- **Python Bindings**: PyO3-based bindings compatible with topologicpy API
- **Efficient Memory**: Arena-based storage for cache-friendly access
- **Spatial Indexing**: BVH, Octree, and KD-tree for fast queries
- **Boolean Operations**: Union, intersection, difference, and more
- **Mesh Generation**: Triangulation and export to OBJ/STL

## Installation

### From PyPI (when published)

```bash
pip install topologic-fast
```

### From Source

```bash
# Clone the repository
git clone https://github.com/your-repo/topologic-fast
cd topologic-fast

# Install with maturin
pip install maturin
maturin develop --release
```

## Quick Start

```python
import topologic_fast as tf

# Create a box
box = tf.Cell.Box(0, 0, 0, 2, 2, 2)
print(f"Volume: {box.Volume()}")
print(f"Area: {box.Area()}")

# Create vertices
v1 = tf.Vertex.ByCoordinates(0, 0, 0)
v2 = tf.Vertex.ByCoordinates(1, 0, 0)
print(f"Distance: {v1.Distance(v2)}")

# Create a face
face = tf.Face.Rectangle(0, 0, 0, 2, 3)
print(f"Face area: {face.Area()}")

# Boolean operations
box1 = tf.Cell.Box(0, 0, 0, 2, 2, 2)
box2 = tf.Cell.Box(1, 0, 0, 2, 2, 2)
union = tf.Topology.Union(box1, box2)
print(f"Union volume: {union.Volume()}")

# Generate mesh
mesh = tf.Mesh.ByCell(box)
obj_content = mesh.ToOBJ()
print(f"Mesh: {mesh.NumVertices()} vertices, {mesh.NumTriangles()} triangles")
```

## Topology Hierarchy

The library implements a hierarchical topology structure:

- **Vertex** (0D): A point in 3D space
- **Edge** (1D): A line segment between two vertices
- **Wire** (1D): A connected sequence of edges
- **Face** (2D): A surface bounded by wires
- **Shell** (2D): A collection of connected faces
- **Cell** (3D): A 3D volume bounded by shells
- **CellComplex** (3D): A collection of connected cells
- **Cluster**: A mixed collection of any topology types

## Performance

Benchmarks comparing topologic-fast to the original topologicpy:

| Operation | topologic-fast | topologicpy | Speedup |
|-----------|---------------|-------------|---------|
| Create 10k vertices | ~1ms | ~50ms | 50x |
| Boolean union | ~5ms | ~100ms | 20x |
| BVH query | ~0.1ms | ~10ms | 100x |

## API Compatibility

The API is designed to be compatible with topologicpy. Most methods use the same names and signatures:

```python
# topologicpy style
vertex = Vertex.ByCoordinates(x, y, z)
face = Face.Rectangle(origin, width, height)
cell = Cell.Box(origin, width, length, height)

# Boolean operations
result = Topology.Union(cell1, cell2)
result = Topology.Intersection(cell1, cell2)
result = Topology.Difference(cell1, cell2)
```

## Building from Source

### Prerequisites

- **Rust 1.71+** (required for dependencies)
- Python 3.8+
- maturin

To install or update Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable
```

### Build Commands

```bash
# Build Rust library
cargo build --release

# Run Rust tests
cargo test

# Build Python bindings
maturin build --release

# Install in development mode
maturin develop --release

# Run Python tests
pytest tests/
```

## Architecture

The library is structured as a Cargo workspace:

- `crates/topologic-core`: Core Rust library
  - `topology/`: Topology data structures (Vertex, Edge, Face, Cell, etc.)
  - `geometry/`: SIMD-optimized geometry operations
  - `boolean/`: Boolean operations (CSG, BSP)
  - `spatial/`: Spatial indexing (BVH, Octree, KD-tree)
  - `mesh/`: Mesh generation and export
- `crates/topologic-py`: Python bindings via PyO3

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## License

This project is dual-licensed under MIT and Apache 2.0 licenses.
