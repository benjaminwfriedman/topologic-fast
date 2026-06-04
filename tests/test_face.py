"""Tests for Face class."""
import pytest
import math


def test_face_rectangle():
    """Test creating a rectangular face."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    face = tf.Face.Rectangle(width=2, length=3)
    assert abs(face.Area() - 6.0) < 1e-10


def test_face_circle():
    """Test creating a circular face."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    face = tf.Face.Circle(radius=1.0, sides=32)

    # Area should be approximately pi * r^2 = pi
    expected_area = math.pi
    assert abs(face.Area() - expected_area) < 0.1


def test_face_perimeter():
    """Test perimeter calculation."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    face = tf.Face.Rectangle(width=2, length=3)

    # Perimeter = 2 * (2 + 3) = 10
    assert abs(face.Perimeter() - 10.0) < 1e-10


def test_face_normal():
    """Test normal calculation."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    face = tf.Face.Rectangle(width=1, length=1)
    normal = face.Normal()

    assert normal is not None
    nx, ny, nz = normal

    # Normal should be pointing in Z direction (either +Z or -Z)
    assert abs(abs(nz) - 1.0) < 1e-10


def test_face_center_of_mass():
    """Test center of mass calculation."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    face = tf.Face.Rectangle(width=2, length=2)
    cx, cy, cz = face.CenterOfMass()

    # Center should be near origin for centered placement
    assert abs(cx) < 1e-10
    assert abs(cy) < 1e-10
    assert abs(cz) < 1e-10


def test_face_vertices():
    """Test getting face vertices."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    face = tf.Face.Rectangle(width=1, length=1)
    vertices = face.Vertices()

    # Rectangle should have 4 vertices
    assert len(vertices) == 4


def test_face_edges():
    """Test getting face edges."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    face = tf.Face.Rectangle(width=1, length=1)
    edges = face.Edges()

    # Rectangle should have 4 edges
    assert len(edges) == 4


def test_face_by_external_boundary():
    """Test creating a face from a wire."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Rectangle(width=2, length=2)
    face = tf.Face.ByExternalBoundary(wire)

    assert abs(face.Area() - 4.0) < 1e-10


def test_face_with_hole():
    """Test creating a face with a hole."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    outer = tf.Wire.Rectangle(width=4, length=4)
    inner = tf.Wire.Rectangle(width=2, length=2)
    face = tf.Face.ByExternalInternalBoundaries(outer, [inner])

    # Area should be 16 - 4 = 12
    assert abs(face.Area() - 12.0) < 1e-10
