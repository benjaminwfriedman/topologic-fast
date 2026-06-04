"""Tests for Topology class (Boolean operations)."""
import pytest
import math


def test_topology_union():
    """Test union of two cells."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create two overlapping boxes
    c1 = tf.Cell.Box(0.0, 0.0, 0.0, 2.0, 2.0, 2.0)
    c2 = tf.Cell.Box(1.0, 0.0, 0.0, 2.0, 2.0, 2.0)

    result = tf.Topology.Union(c1, c2)

    assert result is not None
    # Union volume should be larger than individual volumes but less than sum
    # c1 volume = 8, c2 volume = 8, overlap = 4
    # Union volume = 8 + 8 - 4 = 12
    assert result.Volume() > 8.0


def test_topology_union_non_overlapping():
    """Test union of two non-overlapping cells."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create two separate boxes
    c1 = tf.Cell.Box(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
    c2 = tf.Cell.Box(5.0, 0.0, 0.0, 1.0, 1.0, 1.0)

    result = tf.Topology.Union(c1, c2)

    assert result is not None


def test_topology_intersection():
    """Test intersection of two cells."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create two overlapping boxes
    # Box 1: 0 to 2 in all dimensions
    c1 = tf.Cell.Box(0.0, 0.0, 0.0, 2.0, 2.0, 2.0)
    # Box 2: 1 to 3 in x, 0 to 2 in y and z
    c2 = tf.Cell.Box(1.0, 0.0, 0.0, 2.0, 2.0, 2.0)

    result = tf.Topology.Intersection(c1, c2)

    assert result is not None
    # Intersection is a 1x2x2 box with volume 4
    expected_volume = 4.0
    assert abs(result.Volume() - expected_volume) < 1e-6


def test_topology_intersection_contained():
    """Test intersection when one cell contains another."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Large box containing a smaller box
    c1 = tf.Cell.Box(0.0, 0.0, 0.0, 3.0, 3.0, 3.0)
    c2 = tf.Cell.Box(0.5, 0.5, 0.5, 1.0, 1.0, 1.0)

    result = tf.Topology.Intersection(c1, c2)

    assert result is not None
    # Intersection should be the smaller box
    assert abs(result.Volume() - 1.0) < 1e-6


def test_topology_difference():
    """Test difference of two cells."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create two overlapping boxes
    c1 = tf.Cell.Box(0.0, 0.0, 0.0, 2.0, 2.0, 2.0)
    c2 = tf.Cell.Box(1.0, 0.0, 0.0, 2.0, 2.0, 2.0)

    result = tf.Topology.Difference(c1, c2)

    assert result is not None
    # c1 - c2: remove the overlapping portion
    # c1 volume = 8, overlap = 4
    # Difference volume = 8 - 4 = 4
    expected_volume = 4.0
    assert abs(result.Volume() - expected_volume) < 1e-6


def test_topology_difference_no_overlap():
    """Test difference when cells don't overlap."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create two non-overlapping boxes
    c1 = tf.Cell.Box(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
    c2 = tf.Cell.Box(5.0, 0.0, 0.0, 1.0, 1.0, 1.0)

    result = tf.Topology.Difference(c1, c2)

    assert result is not None
    # When no overlap, result is the original cell
    assert abs(result.Volume() - 1.0) < 1e-6


def test_topology_boolean_chain():
    """Test chaining multiple boolean operations."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create three boxes
    c1 = tf.Cell.Box(0.0, 0.0, 0.0, 2.0, 2.0, 2.0)
    c2 = tf.Cell.Box(1.0, 0.0, 0.0, 2.0, 2.0, 2.0)
    c3 = tf.Cell.Box(0.5, 0.5, 0.5, 1.0, 1.0, 1.0)

    # First union c1 and c2
    union = tf.Topology.Union(c1, c2)

    # Then subtract c3
    result = tf.Topology.Difference(union, c3)

    assert result is not None
    assert result.Volume() > 0


def test_topology_union_cylinders():
    """Test union of cylinder cells."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create two overlapping cylinders
    c1 = tf.Cell.Cylinder(0.0, 0.0, 0.0, 1.0, 2.0, 32)
    c2 = tf.Cell.Cylinder(0.5, 0.0, 0.0, 1.0, 2.0, 32)

    result = tf.Topology.Union(c1, c2)

    assert result is not None
    # Union volume should be less than sum of individual volumes
    # Each cylinder volume = pi * r^2 * h = pi * 1 * 2 = 2*pi
    single_vol = math.pi * 1.0 * 2.0
    assert result.Volume() < 2 * single_vol


def test_topology_intersection_spheres():
    """Test intersection of sphere cells."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create two overlapping spheres
    s1 = tf.Cell.Sphere(0.0, 0.0, 0.0, 1.0, 16, 8)
    s2 = tf.Cell.Sphere(1.0, 0.0, 0.0, 1.0, 16, 8)

    result = tf.Topology.Intersection(s1, s2)

    assert result is not None
    # Intersection of two unit spheres 1 unit apart
    # Should have volume less than either sphere
    sphere_vol = (4.0/3.0) * math.pi  # ~4.19
    assert result.Volume() < sphere_vol
