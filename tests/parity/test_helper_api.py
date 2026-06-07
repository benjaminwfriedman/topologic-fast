"""Parity for the pure-Python helper methods attached to the Rust classes.

These (Dictionary set-ops/accessors, Matrix.Invert) are ported from topologicpy
and attached via topologic_fast._pyhelpers. Compared directly to topologicpy.
"""
import pytest

tf = pytest.importorskip("topologic_fast")
pytest.importorskip("topologicpy")

from topologicpy.Dictionary import Dictionary as TD  # noqa: E402
from topologicpy.Matrix import Matrix as TM  # noqa: E402

FD = tf.Dictionary


def _pair():
    fa = FD.ByKeysValues(["x", "y", "shared"], [1, 2, "a"])
    fb = FD.ByKeysValues(["shared", "z"], ["b", 9])
    ta = TD.ByKeysValues(["x", "y", "shared"], [1, 2, "a"])
    tb = TD.ByKeysValues(["shared", "z"], ["b", 9])
    return fa, fb, ta, tb


def _as_py(d, keys_fn, val_fn):
    return {k: val_fn(d, k) for k in keys_fn(d)}


def _f_py(d):
    return {k: d.ValueAtKey(k) for k in d.Keys()}


def _t_py(d):
    return {k: TD.ValueAtKey(d, k) for k in TD.Keys(d)}


@pytest.mark.parametrize("op", ["Union", "Difference", "Intersection", "SymmetricDifference"])
def test_dictionary_setops(op):
    fa, fb, ta, tb = _pair()
    f_res = _f_py(getattr(FD, op)(fa, fb))
    t_res = _t_py(getattr(TD, op)(ta, tb))
    assert f_res == t_res


def test_values_at_keys():
    fa, _, ta, _ = _pair()
    assert FD.ValuesAtKeys(fa, ["x", "shared", "missing"]) == TD.ValuesAtKeys(ta, ["x", "shared", "missing"])


def test_keys_at_value():
    fa, _, ta, _ = _pair()
    assert sorted(FD.KeysAtValue(fa, 2)) == sorted(TD.KeysAtValue(ta, 2))


def test_set_values_at_keys():
    fa, _, ta, _ = _pair()
    f_res = _f_py(FD.SetValuesAtKeys(fa, ["x", "new"], [100, 200]))
    t_res = _t_py(TD.SetValuesAtKeys(ta, ["x", "new"], [100, 200]))
    assert f_res == t_res


def test_matrix_invert():
    mats = [
        [[1, 0, 0, 0], [0, 1, 0, 0], [0, 0, 1, 0], [0, 0, 0, 1]],
        [[2, 0, 0, 0], [0, 2, 0, 0], [0, 0, 2, 0], [0, 0, 0, 1]],
        [[1, 2, 0, 0], [0, 1, 0, 0], [0, 0, 1, 0], [0, 0, 0, 1]],
    ]
    for m in mats:
        f = tf.Matrix.Invert(m)
        t = TM.Invert(m)
        for fr, tr in zip(f, t):
            for a, b in zip(fr, tr):
                assert abs(float(a) - float(b)) < 1e-9
