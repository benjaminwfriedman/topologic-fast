"""Tests for Cluster class."""
import pytest
import math


def test_cluster_by_vertices():
    """Test creating a cluster from vertices."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v1 = tf.Vertex.ByCoordinates(0.0, 0.0, 0.0)
    v2 = tf.Vertex.ByCoordinates(1.0, 0.0, 0.0)
    v3 = tf.Vertex.ByCoordinates(2.0, 0.0, 0.0)

    cluster = tf.Cluster.ByVertices([v1, v2, v3])

    assert cluster is not None
    assert cluster.Size() == 3
    assert cluster.IsEmpty() == False


def test_cluster_by_edges():
    """Test creating a cluster from edges."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    e1 = tf.Edge.ByCoordinates(0.0, 0.0, 0.0, 1.0, 0.0, 0.0)
    e2 = tf.Edge.ByCoordinates(1.0, 0.0, 0.0, 2.0, 0.0, 0.0)

    cluster = tf.Cluster.ByEdges([e1, e2])

    assert cluster is not None
    assert cluster.Size() == 2


def test_cluster_by_faces():
    """Test creating a cluster from faces."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    f1 = tf.Face.Rectangle(width=1.0, length=1.0)
    f2 = tf.Face.Rectangle(width=1.0, length=1.0)
    f3 = tf.Face.Rectangle(width=1.0, length=1.0)

    cluster = tf.Cluster.ByFaces([f1, f2, f3])

    assert cluster is not None
    assert cluster.Size() == 3


def test_cluster_by_cells():
    """Test creating a cluster from cells."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    c1 = tf.Cell.Box(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
    c2 = tf.Cell.Box(2.0, 0.0, 0.0, 1.0, 1.0, 1.0)

    cluster = tf.Cluster.ByCells([c1, c2])

    assert cluster is not None
    assert cluster.Size() == 2


def test_cluster_get_vertices():
    """Test getting vertices from a cluster."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v1 = tf.Vertex.ByCoordinates(0.0, 0.0, 0.0)
    v2 = tf.Vertex.ByCoordinates(1.0, 0.0, 0.0)

    cluster = tf.Cluster.ByVertices([v1, v2])
    vertices = cluster.Vertices()

    assert len(vertices) == 2


def test_cluster_get_edges():
    """Test getting edges from a cluster."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    e1 = tf.Edge.ByCoordinates(0.0, 0.0, 0.0, 1.0, 0.0, 0.0)
    e2 = tf.Edge.ByCoordinates(1.0, 0.0, 0.0, 2.0, 0.0, 0.0)

    cluster = tf.Cluster.ByEdges([e1, e2])
    edges = cluster.Edges()

    assert len(edges) == 2


def test_cluster_get_faces():
    """Test getting faces from a cluster."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    f1 = tf.Face.Rectangle(width=1.0, length=1.0)
    f2 = tf.Face.Rectangle(width=1.0, length=1.0)

    cluster = tf.Cluster.ByFaces([f1, f2])
    faces = cluster.Faces()

    assert len(faces) == 2


def test_cluster_get_cells():
    """Test getting cells from a cluster."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    c1 = tf.Cell.Box(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
    c2 = tf.Cell.Box(2.0, 0.0, 0.0, 1.0, 1.0, 1.0)

    cluster = tf.Cluster.ByCells([c1, c2])
    cells = cluster.Cells()

    assert len(cells) == 2


def test_cluster_center_of_mass():
    """Test cluster center of mass calculation."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create vertices at corners of a unit square
    v1 = tf.Vertex.ByCoordinates(0.0, 0.0, 0.0)
    v2 = tf.Vertex.ByCoordinates(2.0, 0.0, 0.0)
    v3 = tf.Vertex.ByCoordinates(2.0, 2.0, 0.0)
    v4 = tf.Vertex.ByCoordinates(0.0, 2.0, 0.0)

    cluster = tf.Cluster.ByVertices([v1, v2, v3, v4])
    com = cluster.CenterOfMass()

    # Center should be at (1, 1, 0)
    assert abs(com[0] - 1.0) < 1e-10
    assert abs(com[1] - 1.0) < 1e-10
    assert abs(com[2] - 0.0) < 1e-10


def test_cluster_bounding_box():
    """Test cluster bounding box calculation."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v1 = tf.Vertex.ByCoordinates(0.0, 0.0, 0.0)
    v2 = tf.Vertex.ByCoordinates(3.0, 4.0, 5.0)

    cluster = tf.Cluster.ByVertices([v1, v2])
    min_pt, max_pt = cluster.BoundingBox()

    assert abs(min_pt[0] - 0.0) < 1e-10
    assert abs(min_pt[1] - 0.0) < 1e-10
    assert abs(min_pt[2] - 0.0) < 1e-10
    assert abs(max_pt[0] - 3.0) < 1e-10
    assert abs(max_pt[1] - 4.0) < 1e-10
    assert abs(max_pt[2] - 5.0) < 1e-10


def test_cluster_highest_dimension():
    """Test getting highest dimension in cluster."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v = tf.Vertex.ByCoordinates(0.0, 0.0, 0.0)
    cluster_v = tf.Cluster.ByVertices([v])

    # Vertex is 0-dimensional
    assert cluster_v.HighestDimension() == 0

    c = tf.Cell.Box(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
    cluster_c = tf.Cluster.ByCells([c])

    # Cell is 3-dimensional
    assert cluster_c.HighestDimension() == 3


def test_cluster_empty():
    """Test empty cluster behavior."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create an empty cluster
    cluster = tf.Cluster.ByVertices([])

    assert cluster.Size() == 0
    assert cluster.IsEmpty() == True


def test_cluster_repr():
    """Test cluster string representation."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v1 = tf.Vertex.ByCoordinates(0.0, 0.0, 0.0)
    v2 = tf.Vertex.ByCoordinates(1.0, 0.0, 0.0)
    cluster = tf.Cluster.ByVertices([v1, v2])

    repr_str = repr(cluster)
    assert "Cluster" in repr_str
    assert "2" in repr_str  # size=2


def test_cluster_nested_vertices():
    """Test getting vertices from cluster of faces."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # A face has 4 vertices
    f = tf.Face.Rectangle(width=2.0, length=3.0)
    cluster = tf.Cluster.ByFaces([f])

    vertices = cluster.Vertices()
    # A rectangle has 4 vertices
    assert len(vertices) >= 4
