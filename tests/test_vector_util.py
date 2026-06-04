"""Tests for Vector utility class - matching topologicpy test_14Vector.py"""
import pytest
import math


def test_vector_angle():
    """Test Case 1 - Vector.Angle"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    angle = tf.Vector.Angle([1, 0, 0], [0, 1, 0])
    assert angle == 90, "Vector.Angle. Should be 90"


def test_vector_azimuth_altitude():
    """Test Case 2 - Vector.AzimuthAltitude"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    result = tf.Vector.AzimuthAltitude([1, 0, 0], mantissa=6)
    assert result['azimuth'] == 90
    assert result['altitude'] == 0


def test_vector_by_azimuth_altitude():
    """Test Case 3 - Vector.ByAzimuthAltitude"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # North (azimuth=0)
    vec = tf.Vector.ByAzimuthAltitude(0, 0)
    assert abs(vec[0]) < 0.001  # x ~ 0
    assert abs(vec[1] - 1.0) < 0.001  # y ~ 1
    assert abs(vec[2]) < 0.001  # z ~ 0

    # East (azimuth=90)
    vec = tf.Vector.ByAzimuthAltitude(90, 0)
    assert abs(vec[0] - 1.0) < 0.001  # x ~ 1
    assert abs(vec[1]) < 0.001  # y ~ 0


def test_vector_by_coordinates():
    """Test Case 4 - Vector.ByCoordinates"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    vec = tf.Vector.ByCoordinates(1, 2, 3)
    assert vec == [1, 2, 3]


def test_vector_by_vertices():
    """Test Case 5 - Vector.ByVertices"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(1, 2, 3)
    vec = tf.Vector.ByVertices(v0, v1)
    assert abs(vec[0] - 1) < 0.001
    assert abs(vec[1] - 2) < 0.001
    assert abs(vec[2] - 3) < 0.001


def test_vector_compass_angle():
    """Test Case 6 - Vector.CompassAngle"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # East should be 90 degrees from North
    angle = tf.Vector.CompassAngle([1, 0, 0])
    assert abs(angle - 90) < 0.001


def test_vector_coordinates():
    """Test Case 7 - Vector.Coordinates"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    vec = [1, 2, 3]

    xyz = tf.Vector.Coordinates(vec, "xyz")
    assert xyz == [1, 2, 3]

    xy = tf.Vector.Coordinates(vec, "xy")
    assert xy == [1, 2]

    xz = tf.Vector.Coordinates(vec, "xz")
    assert xz == [1, 3]


def test_vector_cross():
    """Test Case 8 - Vector.Cross"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # X cross Y = Z
    result = tf.Vector.Cross([1, 0, 0], [0, 1, 0])
    assert abs(result[0]) < 0.001
    assert abs(result[1]) < 0.001
    assert abs(result[2] - 1.0) < 0.001


def test_vector_up():
    """Test Case 9 - Vector.Up"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    vec = tf.Vector.Up()
    assert vec == [0, 0, 1]


def test_vector_down():
    """Test Case 10 - Vector.Down"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    vec = tf.Vector.Down()
    assert vec == [0, 0, -1]


def test_vector_north():
    """Test Case 11 - Vector.North"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    vec = tf.Vector.North()
    assert vec == [0, 1, 0]


def test_vector_east():
    """Test Case 12 - Vector.East"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    vec = tf.Vector.East()
    assert vec == [1, 0, 0]


def test_vector_south():
    """Test Case 13 - Vector.South"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    vec = tf.Vector.South()
    assert vec == [0, -1, 0]


def test_vector_west():
    """Test Case 14 - Vector.West"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    vec = tf.Vector.West()
    assert vec == [-1, 0, 0]


def test_vector_is_collinear():
    """Test Case 15 - Vector.IsCollinear"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Parallel vectors are collinear
    assert tf.Vector.IsCollinear([1, 0, 0], [2, 0, 0]) == True
    # Perpendicular vectors are not collinear
    assert tf.Vector.IsCollinear([1, 0, 0], [0, 1, 0]) == False


def test_vector_magnitude():
    """Test Case 16 - Vector.Magnitude"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    mag = tf.Vector.Magnitude([3, 4, 0])
    assert abs(mag - 5.0) < 0.001


def test_vector_multiply():
    """Test Case 17 - Vector.Multiply"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    result = tf.Vector.Multiply([1, 2, 3], 2)
    assert abs(result[0] - 2) < 0.001
    assert abs(result[1] - 4) < 0.001
    assert abs(result[2] - 6) < 0.001


def test_vector_normalize():
    """Test Case 18 - Vector.Normalize"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    result = tf.Vector.Normalize([3, 0, 0])
    assert abs(result[0] - 1.0) < 0.001
    assert abs(result[1]) < 0.001
    assert abs(result[2]) < 0.001


def test_vector_reverse():
    """Test Case 19 - Vector.Reverse"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    result = tf.Vector.Reverse([1, 2, 3])
    assert result == [-1, -2, -3]


def test_vector_set_magnitude():
    """Test Case 20 - Vector.SetMagnitude"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    result = tf.Vector.SetMagnitude([3, 0, 0], 5)
    assert abs(result[0] - 5.0) < 0.001
    assert abs(result[1]) < 0.001
    assert abs(result[2]) < 0.001
