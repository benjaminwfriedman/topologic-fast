"""Tests for Mesh class."""
import pytest
import math


def test_mesh_by_face():
    """Test creating a mesh from a face."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    face = tf.Face.Rectangle(width=2.0, length=3.0)
    mesh = tf.Mesh.ByFace(face)

    assert mesh is not None
    assert mesh.NumVertices() >= 4
    assert mesh.NumTriangles() >= 2


def test_mesh_by_cell():
    """Test creating a mesh from a cell."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    cell = tf.Cell.Box(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
    mesh = tf.Mesh.ByCell(cell)

    assert mesh is not None
    assert mesh.NumVertices() >= 8  # At least 8 vertices for a box
    assert mesh.NumTriangles() >= 12  # 6 faces * 2 triangles each


def test_mesh_num_vertices():
    """Test mesh vertex count."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    face = tf.Face.Rectangle(width=1.0, length=1.0)
    mesh = tf.Mesh.ByFace(face)

    # A rectangle has at least 4 vertices
    assert mesh.NumVertices() >= 4


def test_mesh_num_triangles():
    """Test mesh triangle count."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    face = tf.Face.Rectangle(width=1.0, length=1.0)
    mesh = tf.Mesh.ByFace(face)

    # A rectangle needs at least 2 triangles
    assert mesh.NumTriangles() >= 2


def test_mesh_area():
    """Test mesh area calculation."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    face = tf.Face.Rectangle(width=2.0, length=3.0)
    mesh = tf.Mesh.ByFace(face)

    # Area should match the face area = 2 * 3 = 6
    assert abs(mesh.Area() - 6.0) < 1e-6


def test_mesh_area_circle():
    """Test mesh area for circular face."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    radius = 1.0
    face = tf.Face.Circle(radius=radius, sides=64)
    mesh = tf.Mesh.ByFace(face)

    # Area should approximate pi * r^2
    expected_area = math.pi * radius * radius
    assert abs(mesh.Area() - expected_area) < 0.1  # Allow for approximation


def test_mesh_to_obj():
    """Test OBJ export."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    face = tf.Face.Rectangle(width=1.0, length=1.0)
    mesh = tf.Mesh.ByFace(face)

    obj_content = mesh.ToOBJ()

    assert obj_content is not None
    assert len(obj_content) > 0
    # OBJ format should have vertices (v) and faces (f)
    assert "v " in obj_content or "v\t" in obj_content


def test_mesh_to_stl():
    """Test STL export."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    face = tf.Face.Rectangle(width=1.0, length=1.0)
    mesh = tf.Mesh.ByFace(face)

    stl_content = mesh.ToSTL()

    assert stl_content is not None
    assert len(stl_content) > 0
    # STL format should have "solid" and "facet"
    assert "solid" in stl_content.lower()


def test_mesh_from_cube():
    """Test mesh from a cube cell."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    cell = tf.Cell.Box(0.0, 0.0, 0.0, 2.0, 2.0, 2.0)
    mesh = tf.Mesh.ByCell(cell)

    # Surface area of a 2x2x2 cube = 6 * 4 = 24
    expected_area = 24.0
    assert abs(mesh.Area() - expected_area) < 1e-6


def test_mesh_from_cylinder():
    """Test mesh from a cylinder cell."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    radius = 1.0
    height = 2.0
    cell = tf.Cell.Cylinder(0.0, 0.0, 0.0, radius, height, 32)
    mesh = tf.Mesh.ByCell(cell)

    assert mesh is not None
    assert mesh.NumVertices() > 0
    assert mesh.NumTriangles() > 0


def test_mesh_repr():
    """Test mesh string representation."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    face = tf.Face.Rectangle(width=1.0, length=1.0)
    mesh = tf.Mesh.ByFace(face)
    repr_str = repr(mesh)

    assert "Mesh" in repr_str
    assert "vertices" in repr_str.lower()
    assert "triangles" in repr_str.lower()


def test_mesh_sphere():
    """Test mesh from a sphere cell."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    radius = 1.0
    cell = tf.Cell.Sphere(0.0, 0.0, 0.0, radius, 16, 8)
    mesh = tf.Mesh.ByCell(cell)

    # Sphere surface area = 4 * pi * r^2
    expected_area = 4.0 * math.pi * radius * radius
    # Allow significant tolerance due to polygon approximation
    assert abs(mesh.Area() - expected_area) < expected_area * 0.2


def test_mesh_obj_format_valid():
    """Test that OBJ output has valid format."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    cell = tf.Cell.Box(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
    mesh = tf.Mesh.ByCell(cell)
    obj_content = mesh.ToOBJ()

    lines = obj_content.strip().split('\n')
    vertex_count = 0
    face_count = 0

    for line in lines:
        line = line.strip()
        if line.startswith('v '):
            vertex_count += 1
            parts = line.split()
            # Each vertex line should have x, y, z coordinates
            assert len(parts) >= 4, f"Invalid vertex line: {line}"
        elif line.startswith('f '):
            face_count += 1

    assert vertex_count >= 8  # Minimum for a box
    assert face_count >= 12  # 6 faces * 2 triangles
