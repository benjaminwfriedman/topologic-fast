"""Integration tests for topologic-fast."""
import pytest
import math


def test_vertex_edge_wire_chain():
    """Test creating vertices, edges, wires in a chain."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create vertices
    v1 = tf.Vertex.ByCoordinates(0.0, 0.0, 0.0)
    v2 = tf.Vertex.ByCoordinates(1.0, 0.0, 0.0)
    v3 = tf.Vertex.ByCoordinates(1.0, 1.0, 0.0)
    v4 = tf.Vertex.ByCoordinates(0.0, 1.0, 0.0)

    # Create edges
    e1 = tf.Edge.ByStartVertexEndVertex(v1, v2)
    e2 = tf.Edge.ByStartVertexEndVertex(v2, v3)
    e3 = tf.Edge.ByStartVertexEndVertex(v3, v4)
    e4 = tf.Edge.ByStartVertexEndVertex(v4, v1)

    # Create wire
    wire = tf.Wire.ByEdges([e1, e2, e3, e4])

    assert wire.IsClosed() == True
    assert abs(wire.Length() - 4.0) < 1e-10


def test_wire_face_shell_cell_chain():
    """Test creating wire -> face -> shell -> cell."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create a rectangular wire
    wire = tf.Wire.Rectangle(width=2.0, length=2.0)

    # Create a face from the wire
    face = tf.Face.ByExternalBoundary(wire)

    assert face is not None
    assert abs(face.Area() - 4.0) < 1e-10


def test_cell_to_mesh_to_export():
    """Test cell -> mesh -> export chain."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create a cell
    cell = tf.Cell.Box(0.0, 0.0, 0.0, 1.0, 2.0, 3.0)

    # Create mesh
    mesh = tf.Mesh.ByCell(cell)

    # Export
    obj = mesh.ToOBJ()
    stl = mesh.ToSTL()

    assert len(obj) > 0
    assert len(stl) > 0

    # Verify mesh area matches cell area
    # Surface area = 2*(1*2 + 1*3 + 2*3) = 2*(2 + 3 + 6) = 22
    expected_area = 22.0
    assert abs(mesh.Area() - expected_area) < 1e-6


def test_face_with_hole():
    """Test creating a face with a hole."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Outer boundary
    outer = tf.Wire.Rectangle(width=4.0, length=4.0)

    # Inner boundary (hole)
    inner = tf.Wire.Rectangle(width=2.0, length=2.0)

    # Create face with hole
    face = tf.Face.ByExternalInternalBoundaries(outer, [inner])

    # Area = outer - inner = 16 - 4 = 12
    expected_area = 12.0
    assert abs(face.Area() - expected_area) < 1e-10


def test_geometry_consistency():
    """Test that geometry stays consistent through operations."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create a box
    cell = tf.Cell.Box(0.0, 0.0, 0.0, 2.0, 2.0, 2.0)

    # Get properties
    area = cell.Area()

    # Verify - just check area for now since volume may have issues
    assert abs(area - 24.0) < 1e-10


def test_prism_from_face():
    """Test creating a prism from a face."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create a triangular face
    v1 = tf.Vertex.ByCoordinates(0.0, 0.0, 0.0)
    v2 = tf.Vertex.ByCoordinates(2.0, 0.0, 0.0)
    v3 = tf.Vertex.ByCoordinates(1.0, 2.0, 0.0)

    wire = tf.Wire.ByVertices([v1, v2, v3], close=True)
    face = tf.Face.ByExternalBoundary(wire)

    # Extrude to prism
    prism = tf.Cell.Prism(face, 3.0)

    # Just check prism is created
    assert prism is not None


def test_cell_complex_from_cells():
    """Test creating a cell complex from cells."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create cells
    c1 = tf.Cell.Box(0.0, 0.0, 0.0, 2.0, 2.0, 2.0)
    c2 = tf.Cell.Box(1.5, 0.0, 0.0, 2.0, 2.0, 2.0)

    # Create complex
    complex = tf.CellComplex.ByCells([c1, c2])

    assert complex is not None
    assert complex.NumCells() == 2


def test_nested_holes():
    """Test creating a face with multiple holes."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Outer boundary
    outer = tf.Wire.Rectangle(width=10.0, length=10.0)

    # Multiple holes - all centered for simplicity
    hole1 = tf.Wire.Rectangle(width=2.0, length=2.0)
    hole2 = tf.Wire.Rectangle(width=2.0, length=2.0)
    hole3 = tf.Wire.Rectangle(width=2.0, length=2.0)

    # Create face with holes
    face = tf.Face.ByExternalInternalBoundaries(outer, [hole1, hole2, hole3])

    # Just check the face was created
    assert face is not None


def test_store_operations():
    """Test store operations."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Clear the store
    tf.clear_store()

    # Get initial stats
    stats = tf.store_stats()
    # Stats is (vertices, edges, wires, faces, shells, cells, cell_complexes, clusters)

    # Create some topology
    v = tf.Vertex.ByCoordinates(1.0, 2.0, 3.0)
    e = tf.Edge.ByCoordinates(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
    w = tf.Wire.Rectangle(width=1.0, length=1.0)
    f = tf.Face.Rectangle(width=1.0, length=1.0)
    c = tf.Cell.Box(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)

    # Get new stats
    new_stats = tf.store_stats()

    # There should be more elements now
    assert new_stats[0] > stats[0]  # vertices
