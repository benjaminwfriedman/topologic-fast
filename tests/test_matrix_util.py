"""Tests for Matrix utility class - matching topologicpy test_12Matrix.py"""
import pytest
import math


def test_matrix_by_rotation():
    """Test Case 1 - Matrix.ByRotation"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Test rotation around X axis
    mat_rx = tf.Matrix.ByRotation(rx=45, ry=0, rz=0)
    assert isinstance(mat_rx, list), "Matrix.ByRotation. Should be list"
    assert len(mat_rx) == 4, "Matrix should have 4 rows"
    assert len(mat_rx[0]) == 4, "Matrix should have 4 columns"

    # Test rotation around Y axis
    mat_ry = tf.Matrix.ByRotation(rx=0, ry=45, rz=0)
    assert isinstance(mat_ry, list), "Matrix.ByRotation. Should be list"
    assert len(mat_ry) == 4, "Matrix should have 4 rows"

    # Test rotation around Z axis
    mat_rz = tf.Matrix.ByRotation(rx=0, ry=0, rz=45)
    assert isinstance(mat_rz, list), "Matrix.ByRotation. Should be list"
    assert len(mat_rz) == 4, "Matrix should have 4 rows"


def test_matrix_by_scaling():
    """Test Case 2 - Matrix.ByScaling"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    mat_s1 = tf.Matrix.ByScaling(sx=2, sy=3, sz=4)
    assert isinstance(mat_s1, list), "Matrix.ByScaling. Should be list"
    assert len(mat_s1) == 4, "Matrix should have 4 rows"
    assert len(mat_s1[0]) == 4, "Matrix should have 4 columns"

    # Check diagonal elements are the scale factors
    assert abs(mat_s1[0][0] - 2) < 0.001
    assert abs(mat_s1[1][1] - 3) < 0.001
    assert abs(mat_s1[2][2] - 4) < 0.001

    mat_s2 = tf.Matrix.ByScaling(sx=0.5, sy=0.5, sz=0.5)
    assert isinstance(mat_s2, list), "Matrix.ByScaling. Should be list"


def test_matrix_add():
    """Test Case 3 - Matrix.Add"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    mat_r = tf.Matrix.ByRotation(rx=30)
    mat_s = tf.Matrix.ByScaling(sx=2, sy=2, sz=2)

    mat_add = tf.Matrix.Add(mat_r, mat_s)
    assert isinstance(mat_add, list), "Matrix.Add. Should be list"
    assert len(mat_add) == 4, "Matrix should have 4 rows"
    assert len(mat_add[0]) == 4, "Matrix should have 4 columns"


def test_matrix_by_translation():
    """Test Case 4 - Matrix.ByTranslation"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    mat_t1 = tf.Matrix.ByTranslation(tx=10, ty=20, tz=30)
    assert isinstance(mat_t1, list), "Matrix.ByTranslation. Should be list"
    assert len(mat_t1) == 4, "Matrix should have 4 rows"
    assert len(mat_t1[0]) == 4, "Matrix should have 4 columns"

    # Translation should be in the last column
    assert abs(mat_t1[0][3] - 10) < 0.001
    assert abs(mat_t1[1][3] - 20) < 0.001
    assert abs(mat_t1[2][3] - 30) < 0.001

    mat_t2 = tf.Matrix.ByTranslation(tx=-5, ty=-10, tz=-15)
    assert isinstance(mat_t2, list), "Matrix.ByTranslation. Should be list"


def test_matrix_multiply():
    """Test Case 5 - Matrix.Multiply"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    mat_r = tf.Matrix.ByRotation(rx=45)
    mat_s = tf.Matrix.ByScaling(sx=2, sy=2, sz=2)
    mat_t = tf.Matrix.ByTranslation(tx=5, ty=5, tz=5)

    # Multiply rotation and scaling
    mat_rs = tf.Matrix.Multiply(mat_r, mat_s)
    assert isinstance(mat_rs, list), "Matrix.Multiply. Should be list"
    assert len(mat_rs) == 4, "Matrix should have 4 rows"

    # Multiply with translation
    mat_rst = tf.Matrix.Multiply(mat_rs, mat_t)
    assert isinstance(mat_rst, list), "Matrix.Multiply. Should be list"


def test_matrix_subtract():
    """Test Case 6 - Matrix.Subtract"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    mat_s1 = tf.Matrix.ByScaling(sx=2, sy=2, sz=2)
    mat_s2 = tf.Matrix.ByScaling(sx=1, sy=1, sz=1)

    mat_sub = tf.Matrix.Subtract(mat_s1, mat_s2)
    assert isinstance(mat_sub, list), "Matrix.Subtract. Should be list"
    assert len(mat_sub) == 4, "Matrix should have 4 rows"
    assert len(mat_sub[0]) == 4, "Matrix should have 4 columns"

    # Diagonal should be 2-1=1
    assert abs(mat_sub[0][0] - 1) < 0.001
    assert abs(mat_sub[1][1] - 1) < 0.001
    assert abs(mat_sub[2][2] - 1) < 0.001


def test_matrix_transpose():
    """Test Case 7 - Matrix.Transpose"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    mat_t = tf.Matrix.ByTranslation(tx=1, ty=2, tz=3)
    mat_trans = tf.Matrix.Transpose(mat_t)

    assert isinstance(mat_trans, list), "Matrix.Transpose. Should be list"
    assert len(mat_trans) == 4, "Matrix should have 4 rows"
    assert len(mat_trans[0]) == 4, "Matrix should have 4 columns"

    # After transpose, translation values should be in last row instead of last column
    assert abs(mat_trans[3][0] - 1) < 0.001
    assert abs(mat_trans[3][1] - 2) < 0.001
    assert abs(mat_trans[3][2] - 3) < 0.001


def test_matrix_identity():
    """Test Matrix.Identity"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    mat_i = tf.Matrix.Identity()
    assert isinstance(mat_i, list)
    assert len(mat_i) == 4

    # Identity matrix has 1s on diagonal
    for i in range(4):
        for j in range(4):
            expected = 1.0 if i == j else 0.0
            assert abs(mat_i[i][j] - expected) < 0.001


def test_matrix_combined_transform():
    """Test combining multiple transforms"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Scale -> Rotate -> Translate
    mat_s = tf.Matrix.ByScaling(2, 2, 2)
    mat_r = tf.Matrix.ByRotation(0, 0, 90)
    mat_t = tf.Matrix.ByTranslation(10, 0, 0)

    # Combine: T * R * S (applied right to left)
    mat_rs = tf.Matrix.Multiply(mat_r, mat_s)
    mat_trs = tf.Matrix.Multiply(mat_t, mat_rs)

    assert isinstance(mat_trs, list)
    assert len(mat_trs) == 4
