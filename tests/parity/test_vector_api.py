"""Parity for the pure-math Vector methods (Phase 1 method tail).

These are pure-Python static methods in topologicpy; topologic_fast now provides
them on its Rust ``Vector`` class. Compared directly against topologicpy.
"""
import pytest

tf = pytest.importorskip("topologic_fast")
pytest.importorskip("topologicpy")

from topologicpy.Vector import Vector as V  # noqa: E402

FV = tf.Vector
TOL = 1e-6


def _approx(a, b, tol=TOL):
    a = list(a)
    b = [float(x) for x in b]
    assert len(a) == len(b), f"{a} vs {b}"
    for x, y in zip(a, b):
        assert abs(float(x) - float(y)) <= tol, f"{a} != {b}"


def test_add_subtract_sum_average():
    a, b = [1, 2, 3], [4, 5, 6]
    _approx(FV.Add(a, b), V.Add(a, b))
    _approx(FV.Subtract(a, b), V.Subtract(a, b))
    _approx(FV.Sum([a, b]), V.Sum([a, b]))
    _approx(FV.Average([a, b]), V.Average([a, b]))


def test_length_quadrance():
    a = [1, 2, 3]
    assert FV.Length(a) == V.Length(a)
    assert FV.Quadrance(a) == V.Quadrance(a)


def test_axes_and_compass_constants():
    _approx(FV.XAxis(), V.XAxis())
    _approx(FV.YAxis(), V.YAxis())
    _approx(FV.ZAxis(), V.ZAxis())
    _approx(FV.NorthEast(), V.NorthEast())
    _approx(FV.NorthWest(), V.NorthWest())
    _approx(FV.SouthEast(), V.SouthEast())
    _approx(FV.SouthWest(), V.SouthWest())
    assert FV.CompassDirections() == V.CompassDirections()


@pytest.mark.parametrize("a,b", [
    ([1, 0, 0], [2, 0, 0]), ([1, 0, 0], [-1, 0, 0]), ([1, 0, 0], [0, 1, 0]),
    ([1, 1, 0], [2, 2, 0]), ([0, 0, 1], [0, 0, -3]),
])
def test_parallel_antiparallel_same(a, b):
    assert FV.IsParallel(a, b) == V.IsParallel(a, b)
    assert FV.IsAntiParallel(a, b) == V.IsAntiParallel(a, b)
    assert FV.IsSame(a, b) == V.IsSame(a, b)


def test_is_same_tolerance():
    assert FV.IsSame([1, 0, 0], [1, 0, 0.00001]) == V.IsSame([1, 0, 0], [1, 0, 0.00001]) is True
    assert FV.IsSame([1, 0, 0], [1, 0, 0.1]) == V.IsSame([1, 0, 0], [1, 0, 0.1]) is False


@pytest.mark.parametrize("a,b", [
    ([1, 0, 0], [0, 1, 0]), ([1, 0, 0], [1, 1, 0]), ([0, 0, 1], [1, 0, 1]),
])
def test_bisect(a, b):
    _approx(FV.Bisect(a, b), V.Bisect(a, b))


@pytest.mark.parametrize("a,b", [
    ([1, 0, 0], [0, 1, 0]), ([1, 0, 0], [1, 0, 0]), ([1, 0, 0], [-1, 0, 0]),
    ([1, 2, 3], [3, 2, 1]), ([0, 0, 1], [1, 1, 1]),
])
def test_spread(a, b):
    assert FV.Spread(a, b) == V.Spread(a, b)


@pytest.mark.parametrize("vec,expected", [
    ([0, 1, 0], "North"), ([1, 1, 0], "Northeast"), ([1, 0, 0], "East"),
    ([1, -1, 0], "Southeast"), ([0, -1, 0], "South"), ([-1, -1, 0], "Southwest"),
    ([-1, 0, 0], "West"), ([-1, 1, 0], "Northwest"), ([0, 0, 1], "Up"),
    ([0, 0, -1], "Down"), ([1, 1, 1], "Up_Northeast"), ([0, 0, 0], "Origin"),
])
def test_compass_direction(vec, expected):
    assert FV.CompassDirection(vec) == V.CompassDirection(vec) == expected


@pytest.mark.parametrize("a,b", [
    ([0, 0, 1], [1, 0, 0]), ([1, 0, 0], [0, 1, 0]), ([1, 0, 0], [1, 0, 0]),
    ([1, 0, 0], [-1, 0, 0]), ([1, 2, 3], [3, 2, 1]),
])
def test_transformation_matrix(a, b):
    fm = FV.TransformationMatrix(a, b)
    tm = V.TransformationMatrix(a, b)
    assert len(fm) == 4
    for fr, tr in zip(fm, tm):
        _approx(fr, tr)
