"""Tests for Cell class."""
import pytest
import math


def test_cell_box():
    """Test creating a box cell."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    cell = tf.Cell.Box(0, 0, 0, 2, 2, 2)

    # Box should have 6 faces, 12 edges, 8 vertices
    assert len(cell.Faces()) == 6
    assert len(cell.Edges()) == 12
    assert len(cell.Vertices()) == 8


def test_cell_box_volume():
    """Test box volume calculation."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    cell = tf.Cell.Box(0, 0, 0, 2, 3, 4)
    volume = cell.Volume()

    # Volume should be 2 * 3 * 4 = 24
    assert abs(volume - 24.0) < 1.0  # Allow some tolerance for approximation


def test_cell_box_area():
    """Test box surface area calculation."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    cell = tf.Cell.Box(0, 0, 0, 2, 2, 2)
    area = cell.Area()

    # Surface area of 2x2x2 cube = 6 * 4 = 24
    assert abs(area - 24.0) < 0.1


def test_cell_cylinder():
    """Test creating a cylinder cell."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    cell = tf.Cell.Cylinder(0, 0, 0, 1.0, 2.0, 32)

    # Volume should be approximately pi * r^2 * h = pi * 1 * 2
    expected_volume = math.pi * 2.0
    assert abs(cell.Volume() - expected_volume) < 0.5


def test_cell_sphere():
    """Test creating a sphere cell."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    cell = tf.Cell.Sphere(0, 0, 0, 1.0, 16, 8)

    # Volume should be approximately 4/3 * pi * r^3 = 4/3 * pi
    expected_volume = 4.0 / 3.0 * math.pi
    assert abs(cell.Volume() - expected_volume) < 0.5


def test_cell_center_of_mass():
    """Test center of mass calculation."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    cell = tf.Cell.Box(0, 0, 0, 2, 2, 2)
    cx, cy, cz = cell.CenterOfMass()

    # Center should be at (1, 1, 1)
    assert abs(cx - 1.0) < 0.1
    assert abs(cy - 1.0) < 0.1
    assert abs(cz - 1.0) < 0.1


def test_cell_contains_point():
    """Test point containment."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    cell = tf.Cell.Box(0, 0, 0, 2, 2, 2)

    # Point inside
    assert cell.ContainsPoint(1.0, 1.0, 1.0) == True

    # Point outside
    assert cell.ContainsPoint(5.0, 1.0, 1.0) == False


def test_cell_prism():
    """Test creating a prism cell."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    face = tf.Face.Rectangle(0, 0, 0, 2, 2)
    cell = tf.Cell.Prism(face, 3.0)

    # Volume should be base_area * height = 4 * 3 = 12
    assert abs(cell.Volume() - 12.0) < 1.0


def test_cell_compactness():
    """Test compactness calculation."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # A sphere has compactness = 1
    sphere = tf.Cell.Sphere(0, 0, 0, 1.0, 32, 16)
    compactness = sphere.Compactness()

    # Should be close to 1 for a sphere
    assert compactness > 0.5
    assert compactness <= 1.0 + 0.1
