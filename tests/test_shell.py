"""Tests for Shell class."""
import pytest
import math


def test_shell_by_faces():
    """Test creating a shell from faces."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create a simple open shell from two adjacent faces
    f1 = tf.Face.Rectangle(width=1.0, length=1.0)
    f2 = tf.Face.Rectangle(width=1.0, length=1.0)

    shell = tf.Shell.ByFaces([f1, f2])

    assert shell is not None


def test_shell_is_closed_open():
    """Test that an open shell is not closed."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # A single face is an open shell
    face = tf.Face.Rectangle(width=1.0, length=1.0)
    shell = tf.Shell.ByFaces([face])

    assert shell.IsClosed() == False


def test_shell_area():
    """Test shell area calculation."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create a shell from two 1x1 faces
    f1 = tf.Face.Rectangle(width=1.0, length=1.0)
    f2 = tf.Face.Rectangle(width=1.0, length=1.0)

    shell = tf.Shell.ByFaces([f1, f2])

    # Total area = 1 + 1 = 2
    assert abs(shell.Area() - 2.0) < 1e-10


def test_shell_get_faces():
    """Test getting faces from a shell."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    f1 = tf.Face.Rectangle(width=1.0, length=1.0)
    f2 = tf.Face.Rectangle(width=1.0, length=1.0)
    f3 = tf.Face.Rectangle(width=1.0, length=1.0)

    shell = tf.Shell.ByFaces([f1, f2, f3])
    faces = shell.Faces()

    assert len(faces) == 3


def test_shell_from_box_faces():
    """Test creating a closed shell from a box's faces."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create a unit cube cell and get its shell/faces
    cell = tf.Cell.Box(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
    faces = cell.Faces()

    # A cube has 6 faces
    assert len(faces) == 6

    # Create a shell from the faces
    shell = tf.Shell.ByFaces(faces)

    assert shell is not None


def test_shell_area_from_box():
    """Test shell area from box faces."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create a 1x2x3 box
    cell = tf.Cell.Box(0.0, 0.0, 0.0, 1.0, 2.0, 3.0)
    faces = cell.Faces()
    shell = tf.Shell.ByFaces(faces)

    # Surface area = 2*(1*2 + 1*3 + 2*3) = 2*(2 + 3 + 6) = 22
    expected_area = 22.0
    assert abs(shell.Area() - expected_area) < 1.0  # Allow some tolerance


def test_shell_repr():
    """Test shell string representation."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    face = tf.Face.Rectangle(width=1.0, length=1.0)
    shell = tf.Shell.ByFaces([face])
    repr_str = repr(shell)

    assert "Shell" in repr_str


def test_shell_circular_faces():
    """Test shell with circular faces."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    radius = 1.0
    f1 = tf.Face.Circle(radius=radius, sides=32)
    f2 = tf.Face.Circle(radius=radius, sides=32)

    shell = tf.Shell.ByFaces([f1, f2])

    # Each circle has area approximately pi*r^2
    expected_area = 2.0 * math.pi * radius * radius
    assert abs(shell.Area() - expected_area) < 0.1  # Allow for polygon approximation
