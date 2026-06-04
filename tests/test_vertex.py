"""Tests for Vertex class."""
import pytest
import math


def test_vertex_origin():
    """Test creating a vertex at the origin."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v = tf.Vertex.Origin()
    assert v.X() == 0.0
    assert v.Y() == 0.0
    assert v.Z() == 0.0


def test_vertex_by_coordinates():
    """Test creating a vertex from coordinates."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v = tf.Vertex.ByCoordinates(1.0, 2.0, 3.0)
    assert abs(v.X() - 1.0) < 1e-10
    assert abs(v.Y() - 2.0) < 1e-10
    assert abs(v.Z() - 3.0) < 1e-10


def test_vertex_coordinates():
    """Test getting all coordinates at once."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v = tf.Vertex.ByCoordinates(1.5, 2.5, 3.5)
    x, y, z = v.Coordinates()
    assert abs(x - 1.5) < 1e-10
    assert abs(y - 2.5) < 1e-10
    assert abs(z - 3.5) < 1e-10


def test_vertex_distance():
    """Test calculating distance between vertices."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v1 = tf.Vertex.ByCoordinates(0.0, 0.0, 0.0)
    v2 = tf.Vertex.ByCoordinates(3.0, 4.0, 0.0)

    dist = v1.Distance(v2)
    assert abs(dist - 5.0) < 1e-10


def test_vertex_3d_distance():
    """Test 3D distance calculation."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v1 = tf.Vertex.ByCoordinates(0.0, 0.0, 0.0)
    v2 = tf.Vertex.ByCoordinates(1.0, 1.0, 1.0)

    dist = v1.Distance(v2)
    expected = math.sqrt(3.0)
    assert abs(dist - expected) < 1e-10


def test_vertex_repr():
    """Test vertex string representation."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v = tf.Vertex.ByCoordinates(1.0, 2.0, 3.0)
    repr_str = repr(v)
    assert "Vertex" in repr_str
    assert "1.0" in repr_str or "1.00" in repr_str
