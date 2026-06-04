"""Tests for CellComplex class."""
import pytest
import math


def test_cell_complex_by_cells():
    """Test creating a cell complex from cells."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    c1 = tf.Cell.Box(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
    c2 = tf.Cell.Box(1.0, 0.0, 0.0, 1.0, 1.0, 1.0)

    complex = tf.CellComplex.ByCells([c1, c2])

    assert complex is not None


def test_cell_complex_box():
    """Test creating a box cell complex."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    complex = tf.CellComplex.Box(0.0, 0.0, 0.0, 2.0, 2.0, 2.0)

    assert complex is not None


def test_cell_complex_box_default():
    """Test creating a box cell complex with default parameters."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    complex = tf.CellComplex.Box()

    assert complex is not None
    # Default 1x1x1 box, volume = 1
    assert abs(complex.Volume() - 1.0) < 1e-10


def test_cell_complex_num_cells():
    """Test getting number of cells in a complex."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    c1 = tf.Cell.Box(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
    c2 = tf.Cell.Box(1.0, 0.0, 0.0, 1.0, 1.0, 1.0)
    c3 = tf.Cell.Box(2.0, 0.0, 0.0, 1.0, 1.0, 1.0)

    complex = tf.CellComplex.ByCells([c1, c2, c3])

    assert complex.NumCells() == 3


def test_cell_complex_volume():
    """Test cell complex volume calculation."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create a 2x2x2 box cell complex
    complex = tf.CellComplex.Box(0.0, 0.0, 0.0, 2.0, 2.0, 2.0)

    # Volume = 2 * 2 * 2 = 8
    assert abs(complex.Volume() - 8.0) < 1e-10


def test_cell_complex_volume_multiple_cells():
    """Test volume of cell complex with multiple cells."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create three unit cubes
    c1 = tf.Cell.Box(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
    c2 = tf.Cell.Box(1.0, 0.0, 0.0, 1.0, 1.0, 1.0)
    c3 = tf.Cell.Box(2.0, 0.0, 0.0, 1.0, 1.0, 1.0)

    complex = tf.CellComplex.ByCells([c1, c2, c3])

    # Total volume = 3 * 1 = 3
    assert abs(complex.Volume() - 3.0) < 1e-10


def test_cell_complex_area():
    """Test cell complex surface area."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create a 1x1x1 box cell complex
    complex = tf.CellComplex.Box(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)

    # Surface area of a unit cube = 6
    assert abs(complex.Area() - 6.0) < 1e-10


def test_cell_complex_get_cells():
    """Test getting cells from a complex."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    c1 = tf.Cell.Box(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
    c2 = tf.Cell.Box(1.0, 0.0, 0.0, 1.0, 1.0, 1.0)

    complex = tf.CellComplex.ByCells([c1, c2])
    cells = complex.Cells()

    assert len(cells) == 2


def test_cell_complex_get_faces():
    """Test getting faces from a complex."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # A single box cell complex
    complex = tf.CellComplex.Box(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
    faces = complex.Faces()

    # A cube has 6 faces
    assert len(faces) == 6


def test_cell_complex_repr():
    """Test cell complex string representation."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    complex = tf.CellComplex.Box(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
    repr_str = repr(complex)

    assert "CellComplex" in repr_str


def test_cell_complex_various_dimensions():
    """Test cell complex with various box dimensions."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Test various box dimensions
    test_cases = [
        (1.0, 1.0, 1.0, 1.0),   # Unit cube
        (2.0, 2.0, 2.0, 8.0),   # 2x2x2 cube
        (1.0, 2.0, 3.0, 6.0),   # 1x2x3 box
        (3.0, 4.0, 5.0, 60.0),  # 3x4x5 box
    ]

    for w, l, h, expected_vol in test_cases:
        complex = tf.CellComplex.Box(0.0, 0.0, 0.0, w, l, h)
        assert abs(complex.Volume() - expected_vol) < 1e-10, f"Failed for {w}x{l}x{h}"
