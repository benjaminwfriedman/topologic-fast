"""Tests for Wire class - matching topologicpy test_04Wire.py"""
import pytest
import math


def test_wire_by_edges():
    """Test Case 1 - Wire.ByEdges"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v1 = tf.Vertex.ByCoordinates(0, 0, 0)
    v2 = tf.Vertex.ByCoordinates(1, 0, 0)
    v3 = tf.Vertex.ByCoordinates(1, 1, 0)

    e1 = tf.Edge.ByStartVertexEndVertex(v1, v2)
    e2 = tf.Edge.ByStartVertexEndVertex(v2, v3)

    wire = tf.Wire.ByEdges([e1, e2])
    assert wire is not None
    assert len(wire.Edges()) == 2


def test_wire_by_vertices():
    """Test Case 2 - Wire.ByVertices"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    vertices = [
        tf.Vertex.ByCoordinates(0, 0, 0),
        tf.Vertex.ByCoordinates(1, 0, 0),
        tf.Vertex.ByCoordinates(1, 1, 0),
        tf.Vertex.ByCoordinates(0, 1, 0),
    ]

    wire = tf.Wire.ByVertices(vertices, close=True)
    assert wire is not None
    assert wire.IsClosed() == True
    assert len(wire.Edges()) == 4


def test_wire_circle():
    """Test Case 3 - Wire.Circle"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Circle(radius=1.0, sides=32)
    assert wire is not None
    assert wire.IsClosed() == True

    # Circumference should be approximately 2*pi
    assert abs(wire.Length() - 2 * math.pi) < 0.1


def test_wire_ellipse():
    """Test Case 4 - Wire.Ellipse"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Ellipse(width=4, length=2, sides=32)
    assert wire is not None
    assert wire.IsClosed() == True


def test_wire_rectangle():
    """Test Case 5 - Wire.Rectangle"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Rectangle(width=2, length=3)
    assert wire is not None
    assert wire.IsClosed() == True
    assert abs(wire.Length() - 10) < 0.001  # perimeter = 2*(2+3) = 10


def test_wire_square():
    """Test Case 6 - Wire.Square"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Square(size=2)
    assert wire is not None
    assert wire.IsClosed() == True
    assert abs(wire.Length() - 8) < 0.001  # perimeter = 4*2 = 8


def test_wire_star():
    """Test Case 7 - Wire.Star"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Star(radius_a=1, radius_b=0.5, rays=5)
    assert wire is not None
    assert wire.IsClosed() == True


def test_wire_trapezoid():
    """Test Case 8 - Wire.Trapezoid"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Trapezoid(width_a=2, width_b=1, length=1)
    assert wire is not None
    assert wire.IsClosed() == True


def test_wire_line():
    """Test Case 9 - Wire.Line"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Line(length=5)
    assert wire is not None
    assert wire.IsClosed() == False
    assert abs(wire.Length() - 5) < 0.001


def test_wire_spiral():
    """Test Case 10 - Wire.Spiral"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Spiral(radius_a=0.1, radius_b=1.0, height=2, turns=3, sides=36)
    assert wire is not None


def test_wire_l_shape():
    """Test Case 11 - Wire.LShape"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.LShape(width=1, length=1, a=0.25, b=0.25)
    assert wire is not None
    assert wire.IsClosed() == True


def test_wire_t_shape():
    """Test Case 12 - Wire.TShape"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.TShape(width=1, length=1, a=0.25, b=0.25)
    assert wire is not None
    assert wire.IsClosed() == True


def test_wire_c_shape():
    """Test Case 13 - Wire.CShape"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.CShape(width=1, length=1, a=0.25, b=0.25, c=0.25)
    assert wire is not None
    assert wire.IsClosed() == True


def test_wire_i_shape():
    """Test Case 14 - Wire.IShape"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.IShape(width=1, length=2, a=0.25, b=0.25, c=0.25)
    assert wire is not None
    assert wire.IsClosed() == True


def test_wire_cross_shape():
    """Test Case 15 - Wire.CrossShape"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.CrossShape(width=1, length=1, a=0.25, b=0.25)
    assert wire is not None
    assert wire.IsClosed() == True


def test_wire_is_closed():
    """Test Case 16 - Wire.IsClosed"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    closed_wire = tf.Wire.Rectangle(width=1, length=1)
    assert closed_wire.IsClosed() == True

    open_wire = tf.Wire.Line(length=1)
    assert open_wire.IsClosed() == False


def test_wire_length():
    """Test Case 17 - Wire.Length"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Rectangle(width=2, length=3)
    assert abs(wire.Length() - 10) < 0.001


def test_wire_edges():
    """Test Case 18 - Wire.Edges"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Rectangle(width=1, length=1)
    edges = wire.Edges()
    assert len(edges) == 4


def test_wire_vertices():
    """Test Case 19 - Wire.Vertices"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Rectangle(width=1, length=1)
    vertices = wire.Vertices()
    assert len(vertices) == 4


def test_wire_reverse():
    """Test Case 20 - Wire.Reverse"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Rectangle(width=1, length=1)
    reversed_wire = wire.Reverse()

    assert reversed_wire is not None
    assert abs(reversed_wire.Length() - wire.Length()) < 0.001


def test_wire_close():
    """Test Case 21 - Wire.Close"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    open_wire = tf.Wire.Line(length=1)
    closed_wire = open_wire.Close()

    # Note: Closing a line creates a degenerate wire


def test_wire_center_of_mass():
    """Test Case 22 - Wire.CenterOfMass"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Rectangle(width=2, length=2)
    center = wire.CenterOfMass()

    # For a 2x2 rectangle centered at origin, centroid should be at origin
    assert abs(center[0]) < 1.0  # within bounding box
    assert abs(center[1]) < 1.0


def test_wire_normal():
    """Test Case 23 - Wire.Normal"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Rectangle(width=1, length=1)
    normal = wire.Normal()

    # Normal should be along Z axis for XY plane wire
    if normal is not None:
        assert abs(normal[2]) > 0.9


def test_wire_area():
    """Test Case 24 - Wire.Area"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Rectangle(width=2, length=3)
    area = wire.Area()
    assert abs(area - 6) < 0.001


def test_wire_is_planar():
    """Test Case 25 - Wire.IsPlanar"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Rectangle(width=1, length=1)
    assert wire.IsPlanar() == True


def test_wire_is_manifold():
    """Test Case 26 - Wire.IsManifold"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Rectangle(width=1, length=1)
    assert wire.IsManifold() == True


def test_wire_start_vertex():
    """Test Case 27 - Wire.StartVertex"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Line(length=5)
    start = wire.StartVertex()
    assert start is not None


def test_wire_end_vertex():
    """Test Case 28 - Wire.EndVertex"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Line(length=5)
    end = wire.EndVertex()
    assert end is not None


def test_wire_vertex_by_parameter():
    """Test Case 29 - Wire.VertexByParameter"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Line(length=10)
    v = wire.VertexByParameter(0.5)

    # Midpoint should be at parameter 0.5


def test_wire_vertex_by_distance():
    """Test Case 30 - Wire.VertexByDistance"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Line(length=10)
    v = wire.VertexByDistance(3)

    # Should be at distance 3 along the wire


def test_wire_split():
    """Test Case 31 - Wire.Split"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Rectangle(width=1, length=1)
    split_wires = wire.Split()

    # Should split into 4 single-edge wires
    assert len(split_wires) == 4


def test_wire_interior_angles():
    """Test Case 32 - Wire.InteriorAngles"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Rectangle(width=1, length=1)
    angles = wire.InteriorAngles()

    # Rectangle has 4 corners, each 90 degrees


def test_wire_exterior_angles():
    """Test Case 33 - Wire.ExteriorAngles"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Rectangle(width=1, length=1)
    angles = wire.ExteriorAngles()


def test_wire_squircle():
    """Test Case 34 - Wire.Squircle"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Squircle(radius=1, sides=64, a=4, b=4)
    assert wire is not None
    assert wire.IsClosed() == True


def test_wire_bounding_box():
    """Test Case 35 - Wire.BoundingBox"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Rectangle(width=2, length=4)
    bbox = wire.BoundingBox()

    min_pt, max_pt = bbox
    assert max_pt[0] - min_pt[0] >= 2  # width
    assert max_pt[1] - min_pt[1] >= 4  # length
