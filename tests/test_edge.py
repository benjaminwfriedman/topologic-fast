"""Tests for Edge class - matching topologicpy test_03Edge.py"""
import pytest
import math


def test_edge_by_start_vertex_end_vertex():
    """Test Case 1 - Edge.ByStartVertexEndVertex"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v1 = tf.Vertex.ByCoordinates(0.0, 0.0, 0.0)
    v2 = tf.Vertex.ByCoordinates(3.0, 4.0, 0.0)
    edge = tf.Edge.ByStartVertexEndVertex(v1, v2)

    assert edge is not None, "Edge.ByStartVertexEndVertex. Should be valid edge"
    assert abs(edge.Length() - 5.0) < 1e-10


def test_edge_by_coordinates():
    """Test Edge.ByCoordinates"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    edge = tf.Edge.ByCoordinates(0.0, 0.0, 0.0, 1.0, 0.0, 0.0)

    assert edge is not None
    assert abs(edge.Length() - 1.0) < 1e-10


def test_edge_angle():
    """Test Case 2 - Edge.Angle"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create two perpendicular edges
    edge1 = tf.Edge.ByCoordinates(0, 0, 0, 1, 0, 0)  # X axis
    edge2 = tf.Edge.ByCoordinates(0, 0, 0, 0, 1, 0)  # Y axis

    angle = tf.Edge.Angle(edge1, edge2)
    assert abs(angle - 90.0) < 0.001, f"Edge.Angle. Expected 90, got {angle}"

    # Parallel edges
    edge3 = tf.Edge.ByCoordinates(0, 0, 0, 2, 0, 0)
    angle2 = tf.Edge.Angle(edge1, edge3)
    assert abs(angle2) < 0.001, f"Edge.Angle for parallel. Expected 0, got {angle2}"


def test_edge_direction():
    """Test Case 3 - Edge.Direction"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    edge = tf.Edge.ByCoordinates(0, 0, 0, 1, 0, 0)
    direction = edge.Direction()

    assert abs(direction[0] - 1.0) < 0.001
    assert abs(direction[1]) < 0.001
    assert abs(direction[2]) < 0.001


def test_edge_end_vertex():
    """Test Case 4 - Edge.EndVertex"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v1 = tf.Vertex.ByCoordinates(1.0, 2.0, 3.0)
    v2 = tf.Vertex.ByCoordinates(4.0, 5.0, 6.0)
    edge = tf.Edge.ByStartVertexEndVertex(v1, v2)

    end = edge.EndVertex()
    assert end is not None
    assert abs(end.X() - 4.0) < 1e-10
    assert abs(end.Y() - 5.0) < 1e-10
    assert abs(end.Z() - 6.0) < 1e-10


def test_edge_extend():
    """Test Case 5 - Edge.Extend"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    edge = tf.Edge.ByCoordinates(0, 0, 0, 1, 0, 0)
    original_length = edge.Length()

    extended = edge.Extend(0.5)
    assert extended.Length() > original_length


def test_edge_is_collinear():
    """Test Case 6 - Edge.IsCollinear"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Collinear edges (same line)
    edge1 = tf.Edge.ByCoordinates(0, 0, 0, 1, 0, 0)
    edge2 = tf.Edge.ByCoordinates(2, 0, 0, 3, 0, 0)

    assert tf.Edge.IsCollinear(edge1, edge2) == True

    # Non-collinear edges
    edge3 = tf.Edge.ByCoordinates(0, 0, 0, 0, 1, 0)
    assert tf.Edge.IsCollinear(edge1, edge3) == False


def test_edge_is_parallel():
    """Test Case 7 - Edge.IsParallel"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Parallel edges
    edge1 = tf.Edge.ByCoordinates(0, 0, 0, 1, 0, 0)
    edge2 = tf.Edge.ByCoordinates(0, 1, 0, 1, 1, 0)

    assert tf.Edge.IsParallel(edge1, edge2) == True


def test_edge_length():
    """Test Case 8 - Edge.Length"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Unit edge
    edge1 = tf.Edge.ByCoordinates(0, 0, 0, 1, 0, 0)
    assert abs(edge1.Length() - 1.0) < 1e-10

    # 3-4-5 triangle edge
    edge2 = tf.Edge.ByCoordinates(0, 0, 0, 3, 4, 0)
    assert abs(edge2.Length() - 5.0) < 1e-10


def test_edge_midpoint():
    """Test Case 9 - Edge.Midpoint"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    edge = tf.Edge.ByCoordinates(0, 0, 0, 2, 0, 0)
    midpoint = edge.Midpoint()

    assert abs(midpoint[0] - 1.0) < 0.001
    assert abs(midpoint[1]) < 0.001
    assert abs(midpoint[2]) < 0.001


def test_edge_normalize():
    """Test Case 10 - Edge.Normalize"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    edge = tf.Edge.ByCoordinates(0, 0, 0, 5, 0, 0)
    normalized = edge.Normalize()

    assert abs(normalized.Length() - 1.0) < 0.001


def test_edge_reverse():
    """Test Case 11 - Edge.Reverse"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    edge = tf.Edge.ByCoordinates(0, 0, 0, 1, 0, 0)
    reversed_edge = edge.Reverse()

    start = reversed_edge.StartVertex()
    end = reversed_edge.EndVertex()

    assert abs(start.X() - 1.0) < 0.001
    assert abs(end.X()) < 0.001


def test_edge_set_length():
    """Test Case 12 - Edge.SetLength"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    edge = tf.Edge.ByCoordinates(0, 0, 0, 1, 0, 0)
    new_edge = edge.SetLength(5.0)

    assert abs(new_edge.Length() - 5.0) < 0.001


def test_edge_start_vertex():
    """Test Case 13 - Edge.StartVertex"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v1 = tf.Vertex.ByCoordinates(1.0, 2.0, 3.0)
    v2 = tf.Vertex.ByCoordinates(4.0, 5.0, 6.0)
    edge = tf.Edge.ByStartVertexEndVertex(v1, v2)

    start = edge.StartVertex()
    assert start is not None
    assert abs(start.X() - 1.0) < 1e-10
    assert abs(start.Y() - 2.0) < 1e-10
    assert abs(start.Z() - 3.0) < 1e-10


def test_edge_trim():
    """Test Case 14 - Edge.Trim"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    edge = tf.Edge.ByCoordinates(0, 0, 0, 10, 0, 0)
    trimmed = edge.Trim(0.2, 0.8)

    assert abs(trimmed.Length() - 6.0) < 0.001


def test_edge_vertex_by_parameter():
    """Test Case 15 - Edge.VertexByParameter"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    edge = tf.Edge.ByCoordinates(0, 0, 0, 10, 0, 0)

    # Get vertex at parameter 0.5
    v = edge.VertexByParameter(0.5)
    assert abs(v.X() - 5.0) < 0.001


def test_edge_vertex_by_distance():
    """Test Case 16 - Edge.VertexByDistance"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    edge = tf.Edge.ByCoordinates(0, 0, 0, 10, 0, 0)

    # Get vertex at distance 3
    v = edge.VertexByDistance(3.0)
    assert abs(v.X() - 3.0) < 0.001


def test_edge_vertices():
    """Test Case 17 - Edge.Vertices"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v1 = tf.Vertex.ByCoordinates(0, 0, 0)
    v2 = tf.Vertex.ByCoordinates(1, 0, 0)
    edge = tf.Edge.ByStartVertexEndVertex(v1, v2)

    vertices = edge.Vertices()
    assert len(vertices) == 2


def test_edge_normal():
    """Test Edge.Normal"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    edge = tf.Edge.ByCoordinates(0, 0, 0, 1, 0, 0)
    normal = edge.Normal()

    # Normal should be perpendicular to edge direction
    assert abs(normal[1]) > 0.9 or abs(normal[2]) > 0.9


def test_edge_evaluate():
    """Test Edge.Evaluate"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    edge = tf.Edge.ByCoordinates(0, 0, 0, 10, 10, 10)

    # Evaluate at t=0.5
    point = edge.Evaluate(0.5)
    assert abs(point[0] - 5.0) < 0.001
    assert abs(point[1] - 5.0) < 0.001
    assert abs(point[2] - 5.0) < 0.001


def test_edge_split_at():
    """Test Edge.SplitAt"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    edge = tf.Edge.ByCoordinates(0, 0, 0, 10, 0, 0)
    e1, e2 = edge.SplitAt(0.4)

    assert abs(e1.Length() - 4.0) < 0.001
    assert abs(e2.Length() - 6.0) < 0.001


def test_edge_center_of_mass():
    """Test Edge.CenterOfMass"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    edge = tf.Edge.ByCoordinates(0, 0, 0, 10, 0, 0)
    center = edge.CenterOfMass()

    assert abs(center[0] - 5.0) < 0.001


def test_edge_quadrance():
    """Test Edge.Quadrance (squared length)"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    edge = tf.Edge.ByCoordinates(0, 0, 0, 3, 4, 0)
    q = edge.Quadrance()

    assert abs(q - 25.0) < 0.001  # 3^2 + 4^2 = 25


def test_edge_bisect():
    """Test Edge.Bisect"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    edge = tf.Edge.ByCoordinates(0, 0, 0, 10, 0, 0)
    v = edge.Bisect(0.5)

    assert abs(v.X() - 5.0) < 0.001


def test_edge_offset_2d():
    """Test Edge.ByOffset2D"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    edge = tf.Edge.ByCoordinates(0, 0, 0, 10, 0, 0)
    offset_edge = edge.ByOffset2D(1.0)

    # Offset edge should be parallel and shifted in Y
    start = offset_edge.StartVertex()
    assert abs(start.Y() - 1.0) < 0.001


def test_edge_closest_point():
    """Test Edge.ClosestPoint"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    edge = tf.Edge.ByCoordinates(0, 0, 0, 10, 0, 0)

    # Point above edge
    closest = edge.ClosestPoint([5, 5, 0])
    assert abs(closest[0] - 5.0) < 0.001
    assert abs(closest[1]) < 0.001


def test_edge_distance_to_point():
    """Test Edge.DistanceToPoint"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    edge = tf.Edge.ByCoordinates(0, 0, 0, 10, 0, 0)

    # Point 5 units above the edge midpoint
    dist = edge.DistanceToPoint([5, 5, 0])
    assert abs(dist - 5.0) < 0.001
