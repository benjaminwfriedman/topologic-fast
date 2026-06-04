"""Tests for Topology transformation methods - matching topologicpy test_15Topology.py"""
import pytest
import math


def test_topology_translate_vertex():
    """Test Topology.TranslateVertex"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v = tf.Vertex.ByCoordinates(0, 0, 0)
    translated = tf.Topology.TranslateVertex(v, x=1, y=2, z=3)

    assert abs(translated.X() - 1.0) < 0.001
    assert abs(translated.Y() - 2.0) < 0.001
    assert abs(translated.Z() - 3.0) < 0.001


def test_topology_translate_edge():
    """Test Topology.TranslateEdge"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    edge = tf.Edge.ByCoordinates(0, 0, 0, 1, 0, 0)
    translated = tf.Topology.TranslateEdge(edge, x=5, y=0, z=0)

    start = translated.StartVertex()
    assert abs(start.X() - 5.0) < 0.001


def test_topology_translate_face():
    """Test Topology.TranslateFace"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    face = tf.Face.Rectangle(width=2, length=2)
    translated = tf.Topology.TranslateFace(face, x=0, y=0, z=5)

    center = translated.CenterOfMass()
    assert abs(center[2] - 5.0) < 0.001


def test_topology_translate_cell():
    """Test Topology.TranslateCell"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    cell = tf.Cell.Box()
    translated = tf.Topology.TranslateCell(cell, x=10, y=0, z=0)

    center = translated.CenterOfMass()
    assert abs(center[0] - 10.0) < 1.0


def test_topology_rotate_vertex():
    """Test Topology.RotateVertex"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v = tf.Vertex.ByCoordinates(1, 0, 0)
    # Rotate 90 degrees around Z axis
    rotated = tf.Topology.RotateVertex(v, angle=90)

    assert abs(rotated.X()) < 0.001
    assert abs(rotated.Y() - 1.0) < 0.001


def test_topology_rotate_face():
    """Test Topology.RotateFace"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    face = tf.Face.Rectangle(width=2, length=2)
    # Rotate 180 degrees around Z axis
    rotated = tf.Topology.RotateFace(face, angle=180)

    # Face should still have same area
    assert abs(rotated.Area() - 4.0) < 0.001


def test_topology_scale_vertex():
    """Test Topology.ScaleVertex"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v = tf.Vertex.ByCoordinates(1, 1, 1)
    scaled = tf.Topology.ScaleVertex(v, x=2, y=2, z=2)

    assert abs(scaled.X() - 2.0) < 0.001
    assert abs(scaled.Y() - 2.0) < 0.001
    assert abs(scaled.Z() - 2.0) < 0.001


def test_topology_scale_face():
    """Test Topology.ScaleFace"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    face = tf.Face.Rectangle(width=1, length=1)
    # Scale by 2x in X and Y
    scaled = tf.Topology.ScaleFace(face, x=2, y=2, z=1)

    # Area should be 4x larger
    assert abs(scaled.Area() - 4.0) < 0.1


def test_topology_scale_cell():
    """Test Topology.ScaleCell"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    cell = tf.Cell.Box(width=1, length=1, height=1)
    # Scale by 2x in all directions
    scaled = tf.Topology.ScaleCell(cell, x=2, y=2, z=2)

    # Volume should be 8x larger
    assert abs(scaled.Volume() - 8.0) < 0.5


def test_topology_transform_chain():
    """Test multiple transforms in sequence"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v = tf.Vertex.ByCoordinates(0, 0, 0)

    # Translate, then rotate
    v1 = tf.Topology.TranslateVertex(v, x=1, y=0, z=0)
    v2 = tf.Topology.RotateVertex(v1, angle=90)

    # Should be at (0, 1, 0) after rotation
    assert abs(v2.X()) < 0.001
    assert abs(v2.Y() - 1.0) < 0.001


def test_topology_rotate_around_point():
    """Test rotation around a specific point"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v = tf.Vertex.ByCoordinates(2, 0, 0)
    origin = tf.Vertex.ByCoordinates(1, 0, 0)

    # Rotate 90 degrees around Z axis through origin (1, 0, 0)
    rotated = tf.Topology.RotateVertex(v, origin=origin, angle=90)

    # Should end up at (1, 1, 0)
    assert abs(rotated.X() - 1.0) < 0.001
    assert abs(rotated.Y() - 1.0) < 0.001


def test_topology_scale_from_origin():
    """Test scaling from a specific origin"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v = tf.Vertex.ByCoordinates(2, 2, 0)
    origin = tf.Vertex.ByCoordinates(1, 1, 0)

    # Scale by 2 from origin (1, 1, 0)
    scaled = tf.Topology.ScaleVertex(v, origin=origin, x=2, y=2, z=1)

    # Should end up at (3, 3, 0)
    assert abs(scaled.X() - 3.0) < 0.001
    assert abs(scaled.Y() - 3.0) < 0.001


def test_topology_rotate_different_axes():
    """Test rotation around different axes"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v = tf.Vertex.ByCoordinates(1, 0, 0)

    # Rotate around Y axis
    rotated = tf.Topology.RotateVertex(v, axis=[0, 1, 0], angle=90)

    # Should be at (0, 0, -1)
    assert abs(rotated.X()) < 0.001
    assert abs(rotated.Z() + 1.0) < 0.001


def test_wire_transformations():
    """Test Wire transformation methods"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Rectangle(width=2, length=2)
    original_length = wire.Length()

    # Translate
    translated = tf.Topology.TranslateWire(wire, x=5, y=5, z=5)
    assert abs(translated.Length() - original_length) < 0.001

    # Rotate
    rotated = tf.Topology.RotateWire(wire, angle=45)
    assert abs(rotated.Length() - original_length) < 0.001

    # Scale
    scaled = tf.Topology.ScaleWire(wire, x=2, y=2, z=1)
    assert abs(scaled.Length() - original_length * 2) < 0.01


def test_shell_transformations():
    """Test Shell transformation methods"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    cell = tf.Cell.Box(width=1, length=1, height=1)
    faces = cell.Faces()
    shell = tf.Shell.ByFaces(faces)
    original_area = shell.Area()

    # Translate
    translated = tf.Topology.TranslateShell(shell, x=5, y=0, z=0)
    assert abs(translated.Area() - original_area) < 0.001

    # Scale
    scaled = tf.Topology.ScaleShell(shell, x=2, y=2, z=2)
    assert abs(scaled.Area() - original_area * 4) < 0.1  # Area scales as square
