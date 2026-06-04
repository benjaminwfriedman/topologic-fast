"""Tests for Grid utility class - matching topologicpy test_11Grid.py"""
import pytest


def test_grid_edges_by_distances():
    """Test Case 1 - Grid.EdgesByDistances"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Test with default parameters
    clus_ed = tf.Grid.EdgesByDistances()
    assert clus_ed is not None, "Grid.EdgesByDistances. Should return Cluster"

    # Test with custom face and parameters
    face = tf.Face.Rectangle(width=10, length=10)
    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(0, 10, 0)

    clus_ed1 = tf.Grid.EdgesByDistances(
        face=face,
        u_origin=v0,
        v_origin=v1,
        u_range=[-0.5, -0.25, 0, 0.25, 0.5],
        v_range=[-0.5, -0.25, 0, 0.25, 0.5],
        clip=True,
        tolerance=0.001
    )
    assert clus_ed1 is not None, "Grid.EdgesByDistances. Should return Cluster"

    clus_ed2 = tf.Grid.EdgesByDistances(
        face=face,
        u_origin=v1,
        v_origin=None,
        u_range=[-0.5, -0.25, 0, 0.25, 0.5],
        v_range=[-0.5, -0.25, 0, 0.25, 0.5],
        clip=False,
        tolerance=0.001
    )
    assert clus_ed2 is not None, "Grid.EdgesByDistances. Should return Cluster"


def test_grid_edges_by_parameters():
    """Test Case 2 - Grid.EdgesByParameters"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    face = tf.Face.Rectangle(width=10, length=10)

    clus_ep = tf.Grid.EdgesByParameters(face)
    assert clus_ep is not None, "Grid.EdgesByParameters. Should return Cluster"

    clus_ep1 = tf.Grid.EdgesByParameters(
        face,
        u_range=[0, 0.25, 0.5, 0.75, 1.0],
        v_range=[0, 0.25, 0.5, 0.75, 1.0],
        clip=False
    )
    assert clus_ep1 is not None, "Grid.EdgesByParameters. Should return Cluster"

    # Check that edges were created
    edges = clus_ep1.Edges()
    assert len(edges) > 0, "Should have created edges"


def test_grid_vertices_by_distances():
    """Test Case 3 - Grid.VerticesByDistances"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Test with default parameters
    clus_vd = tf.Grid.VerticesByDistances()
    assert clus_vd is not None, "Grid.VerticesByDistances. Should return Cluster"

    # Test with custom face and parameters
    face = tf.Face.Rectangle(width=10, length=10)
    v2 = tf.Vertex.ByCoordinates(10, 10, 0)

    clus_vd1 = tf.Grid.VerticesByDistances(
        face=face,
        origin=v2,
        u_range=[-0.5, -0.25, 0, 0.25, 0.5],
        v_range=[-0.5, -0.25, 0, 0.25, 0.5],
        clip=False,
        tolerance=0.001
    )
    assert clus_vd1 is not None, "Grid.VerticesByDistances. Should return Cluster"

    clus_vd2 = tf.Grid.VerticesByDistances(
        face=face,
        origin=None,
        u_range=[-0.5, -0.25, 0, 0.25, 0.5],
        v_range=[-0.5, -0.25, 0, 0.25, 0.5],
        clip=True,
        tolerance=0.001
    )
    assert clus_vd2 is not None, "Grid.VerticesByDistances. Should return Cluster"

    # Check that vertices were created
    vertices = clus_vd2.Vertices()
    assert len(vertices) > 0, "Should have created vertices"


def test_grid_default_face():
    """Test Grid with default face"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Should work with None face (uses default)
    clus = tf.Grid.EdgesByParameters()
    assert clus is not None


def test_grid_vertices_count():
    """Test that correct number of vertices are created"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    face = tf.Face.Rectangle(width=10, length=10)

    # 3x3 grid should have 9 vertices
    u_range = [0.0, 0.5, 1.0]
    v_range = [0.0, 0.5, 1.0]

    clus = tf.Grid.VerticesByDistances(
        face=face,
        u_range=u_range,
        v_range=v_range
    )

    vertices = clus.Vertices()
    # May have more vertices from face itself
    assert len(vertices) >= 9


def test_grid_edges_count():
    """Test that correct number of edges are created"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    face = tf.Face.Rectangle(width=10, length=10)

    # With 3 U-values and 3 V-values, should have 3+3=6 grid lines
    u_range = [0.0, 0.5, 1.0]
    v_range = [0.0, 0.5, 1.0]

    clus = tf.Grid.EdgesByParameters(
        face,
        u_range=u_range,
        v_range=v_range
    )

    edges = clus.Edges()
    assert len(edges) == 6, f"Expected 6 edges, got {len(edges)}"
