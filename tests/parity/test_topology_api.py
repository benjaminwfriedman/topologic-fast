"""Parity for the generic, type-dispatching ``Topology.*`` API (Phase 1).

topologicpy code calls ``Topology.Faces(t)``, ``Topology.Translate(t, ...)``,
``Topology.Type(t)`` etc. on *any* topology. These compare topologic_fast's
generic dispatcher against topologicpy's on equivalent geometry.

Skipped if topologicpy is not installed.
"""
import pytest

tf = pytest.importorskip("topologic_fast")
pytest.importorskip("topologicpy")

from topologicpy.Topology import Topology as TT  # noqa: E402
from topologicpy.Cell import Cell as TC  # noqa: E402
from topologicpy.Dictionary import Dictionary as TD  # noqa: E402


def _tf_box():
    return tf.Cell.Box(0, 0, 0, 2, 3, 4)


def _tpy_box():
    return TT.Translate(TC.Prism(width=2, length=3, height=4), 1, 1.5, 2)


def test_type_and_typeasstring():
    fb, tb = _tf_box(), _tpy_box()
    assert tf.Topology.Type(fb) == TT.Type(tb) == 32
    assert tf.Topology.TypeAsString(fb) == TT.TypeAsString(tb) == "Cell"


def test_is_instance():
    fb, tb = _tf_box(), _tpy_box()
    for name in ["Cell", "Topology", "cell"]:
        assert tf.Topology.IsInstance(fb, name) == TT.IsInstance(tb, name) is True
    for name in ["Face", "Vertex", "Edge"]:
        assert tf.Topology.IsInstance(fb, name) == TT.IsInstance(tb, name) is False


@pytest.mark.parametrize("kind", ["Vertices", "Edges", "Faces"])
def test_subtopology_counts(kind):
    fb, tb = _tf_box(), _tpy_box()
    assert len(getattr(tf.Topology, kind)(fb)) == len(getattr(TT, kind)(tb))


@pytest.mark.parametrize("sub", ["vertex", "edge", "face"])
def test_subtopologies_dispatch(sub):
    fb, tb = _tf_box(), _tpy_box()
    assert len(tf.Topology.SubTopologies(fb, sub)) == len(TT.SubTopologies(tb, sub))


def test_translate_preserves_volume_and_type():
    fb, tb = _tf_box(), _tpy_box()
    fm = tf.Topology.Translate(fb, 5, -2, 3)
    tm = TT.Translate(tb, 5, -2, 3)
    assert type(fm).__name__ == "Cell"
    assert abs(fm.Volume() - TC.Volume(tm)) < 1e-6
    assert abs(fm.Volume() - 24.0) < 1e-6


def test_rotate_preserves_volume():
    fb, tb = _tf_box(), _tpy_box()
    fm = tf.Topology.Rotate(fb, None, [0, 0, 1], 90.0)
    tm = TT.Rotate(tb, None, [0, 0, 1], 90.0)
    assert abs(fm.Volume() - TC.Volume(tm)) < 1e-6
    assert abs(fm.Volume() - 24.0) < 1e-6


def test_scale_volume_matches():
    fb, tb = _tf_box(), _tpy_box()
    fm = tf.Topology.Scale(fb, None, 2, 2, 2)
    tm = TT.Scale(tb, None, 2, 2, 2)
    # 2x in each axis -> 8x volume.
    assert abs(fm.Volume() - 8 * 24.0) < 1e-6
    assert abs(fm.Volume() - TC.Volume(tm)) < 1e-6


def _xyz(v):
    c = v.Coordinates()
    return (round(c[0], 6), round(c[1], 6), round(c[2], 6))


def _txyz(v):
    from topologicpy.Vertex import Vertex as TVx
    return (round(TVx.X(v), 6), round(TVx.Y(v), 6), round(TVx.Z(v), 6))


def test_center_of_mass_matches():
    fb, tb = _tf_box(), _tpy_box()
    assert _xyz(tf.Topology.CenterOfMass(fb)) == _txyz(TT.CenterOfMass(tb)) == (1.0, 1.5, 2.0)
    # a face of the box, too
    ff = tf.Topology.Faces(fb)[0]
    fc = _xyz(tf.Topology.CenterOfMass(ff))
    assert len(fc) == 3  # well-defined


def test_centroid_matches():
    fb, tb = _tf_box(), _tpy_box()
    assert _xyz(tf.Topology.Centroid(fb)) == _txyz(TT.Centroid(tb)) == (1.0, 1.5, 2.0)


def _tf_stack():
    return tf.CellComplex.ByCells([tf.Cell.Box(0, 0, 0, 2, 2, 2), tf.Cell.Box(0, 0, 2, 2, 2, 2)])


def _tpy_stack():
    from topologicpy.CellComplex import CellComplex as TCC
    b1 = TT.Translate(TC.Prism(width=2, length=2, height=2), 1, 1, 1)
    b2 = TT.Translate(TC.Prism(width=2, length=2, height=2), 1, 1, 3)
    return TCC.ByCells([b1, b2])


def _cc_invariants(cc, faces_fn, cells_fn, vol_fn):
    cells = cells_fn(cc)
    return (len(faces_fn(cc)), len(cells), round(sum(vol_fn(c) for c in cells), 6))


def test_cellcomplex_translate_parity():
    f = _tf_stack()
    t = _tpy_stack()
    fm = tf.Topology.Translate(f, 10, -3, 2)
    tm = TT.Translate(t, 10, -3, 2)
    f_inv = (len(tf.Topology.Faces(fm)), fm.NumCells(), round(sum(c.Volume() for c in fm.Cells()), 6))
    t_inv = (len(TT.Faces(tm)), len(TT.Cells(tm)), round(sum(TC.Volume(c) for c in TT.Cells(tm)), 6))
    assert f_inv == t_inv == (11, 2, 16.0)


def test_cellcomplex_scale_parity():
    f = _tf_stack()
    t = _tpy_stack()
    fm = tf.Topology.Scale(f, None, 2, 2, 2)
    tm = TT.Scale(t, None, 2, 2, 2)
    f_vol = round(sum(c.Volume() for c in fm.Cells()), 6)
    t_vol = round(sum(TC.Volume(c) for c in TT.Cells(tm)), 6)
    assert f_vol == t_vol == 128.0  # 2 cells, each 2x2x2 -> 4x4x4 = 64
    assert len(tf.Topology.Faces(fm)) == len(TT.Faces(tm)) == 11


def test_set_and_get_dictionary_roundtrip():
    fb = _tf_box()
    fd = tf.Dictionary.ByKeysValues(["name", "n"], ["kitchen", 3])
    tf.Topology.SetDictionary(fb, fd)
    got = tf.Topology.Dictionary(fb)
    assert got.ValueAtKey("name") == "kitchen"
    assert got.ValueAtKey("n") == 3

    # And topologicpy's equivalent on its own kernel, for shape parity.
    tb = _tpy_box()
    td = TD.ByKeysValues(["name", "n"], ["kitchen", 3])
    tb = TT.SetDictionary(tb, td)
    tgot = TT.Dictionary(tb)
    assert TD.ValueAtKey(tgot, "name") == "kitchen"
    assert TD.ValueAtKey(tgot, "n") == 3
