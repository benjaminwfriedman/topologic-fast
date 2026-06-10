"""The lean fast topologicpy-compatible native API (the performance path).

Verifies the native ``tf.*`` methods (1) match topologicpy's results, (2) work
both as ``Class.Method(obj)`` and ``obj.Method()``, and (3) are substantially
faster than topologicpy (the whole point — see _native_api.py).
"""
import time

import pytest

tf = pytest.importorskip("topologic_fast")
pytest.importorskip("topologicpy")

from topologicpy.Core import Core  # noqa: E402
from topologicpy.Vertex import Vertex as TV  # noqa: E402
from topologicpy.Edge import Edge as TE  # noqa: E402
from topologicpy.Cell import Cell as TC  # noqa: E402
from topologicpy.Topology import Topology as TT  # noqa: E402

Core.ResetBackend()  # native API must not depend on the backend swap


def _fbox():
    return tf.Cell.Box(0, 0, 0, 2, 3, 4)


def _tbox():
    return TT.Translate(TC.Prism(width=2, length=3, height=4), 1, 1.5, 2)


def test_vertex_methods_match():
    fa, fb = tf.Vertex.ByCoordinates(1, 2, 3), tf.Vertex.ByCoordinates(4, 6, 3)
    ta, tb = TV.ByCoordinates(1, 2, 3), TV.ByCoordinates(4, 6, 3)
    assert tf.Vertex.Coordinates(fa) == TV.Coordinates(ta)
    assert tf.Vertex.Coordinates(fa, "zyx") == TV.Coordinates(ta, "zyx")
    assert tf.Vertex.X(fa, mantissa=2) == TV.X(ta, mantissa=2)
    assert tf.Vertex.Distance(fa, fb) == TV.Distance(ta, tb)


def test_both_call_styles_work():
    v = tf.Vertex.ByCoordinates(1, 2, 3)
    assert tf.Vertex.Coordinates(v) == v.Coordinates() == [1.0, 2.0, 3.0]
    box = _fbox()
    assert tf.Cell.Volume(box) == box.Volume() == 24.0


def test_edge_cell_methods_match():
    fa, fb = tf.Vertex.ByCoordinates(1, 2, 3), tf.Vertex.ByCoordinates(4, 6, 3)
    ta, tb = TV.ByCoordinates(1, 2, 3), TV.ByCoordinates(4, 6, 3)
    fe = tf.Edge.ByStartVertexEndVertex(fa, fb)
    te = TE.ByStartVertexEndVertex(ta, tb)
    assert tf.Edge.Length(fe) == round(TE.Length(te), 6)
    assert tf.Cell.Volume(_fbox()) == TC.Volume(_tbox())


def test_cell_prism_matches():
    from topologicpy.Cell import Cell as TC2
    for placement in ["center", "bottom", "lowerleft"]:
        fp = tf.Cell.Prism(width=2, length=3, height=4, placement=placement)
        tp = TC.Prism(width=2, length=3, height=4, placement=placement)
        assert tf.Cell.Volume(fp) == TC2.Volume(tp) == 24.0
        f_com = [round(c, 5) for c in tf.Vertex.Coordinates(tf.Topology.Centroid(fp))]
        t_com = [round(c, 5) for c in TV.Coordinates(TT.Centroid(tp))]
        assert f_com == t_com
        assert len(tf.Topology.Faces(fp)) == len(TT.Faces(tp)) == 6


def test_edge_byvertices_both_forms():
    v1, v2 = tf.Vertex.ByCoordinates(0, 0, 0), tf.Vertex.ByCoordinates(3, 4, 0)
    assert tf.Edge.Length(tf.Edge.ByVertices([v1, v2])) == 5.0
    assert tf.Edge.Length(tf.Edge.ByVertices(v1, v2)) == 5.0
    # tolerance arg (topologicpy signature) is accepted
    assert tf.Edge.Length(tf.Edge.ByStartVertexEndVertex(v1, v2, 0.0001)) == 5.0


def test_face_external_boundary_both_styles():
    face = tf.Topology.Faces(tf.Cell.Box(0, 0, 0, 1, 1, 1))[0]
    assert len(tf.Topology.Vertices(tf.Face.ExternalBoundary(face))) == 4
    assert len(tf.Topology.Vertices(face.ExternalBoundary())) == 4


def test_cell_measures_match():
    from topologicpy.Cell import Cell as TC2
    fbox, tbox = _fbox(), _tbox()
    assert tf.Cell.Area(fbox) == TC2.Area(tbox) == 52.0
    assert tf.Cell.Compactness(fbox) == round(TC2.Compactness(tbox), 6)
    # ByFaces rebuilds an equivalent cell
    rebuilt = tf.Cell.ByFaces(tf.Topology.Faces(fbox))
    assert round(rebuilt.Volume(), 3) == 24.0


def test_edge_wire_accessors():
    e = tf.Edge.ByStartVertexEndVertex(tf.Vertex.ByCoordinates(0, 0, 0),
                                       tf.Vertex.ByCoordinates(2, 0, 0))
    assert tf.Vertex.Coordinates(tf.Edge.VertexByParameter(e, 0.5)) == [1.0, 0.0, 0.0]
    assert tf.Vertex.Coordinates(e.VertexByParameter(0.5)) == [1.0, 0.0, 0.0]
    w = tf.Wire.Rectangle(width=2, length=2)
    assert tf.Wire.IsClosed(w) is True and w.IsClosed() is True


def test_index_and_issame_match():
    from topologicpy.Vertex import Vertex as TV2
    fvs = [tf.Vertex.ByCoordinates(0, 0, 0), tf.Vertex.ByCoordinates(1, 1, 1),
           tf.Vertex.ByCoordinates(2, 2, 2)]
    tvs = [TV2.ByCoordinates(0, 0, 0), TV2.ByCoordinates(1, 1, 1), TV2.ByCoordinates(2, 2, 2)]
    assert tf.Vertex.Index(tf.Vertex.ByCoordinates(1, 1, 1), fvs) == TV2.Index(TV2.ByCoordinates(1, 1, 1), tvs) == 1
    assert tf.Vertex.Index(tf.Vertex.ByCoordinates(9, 9, 9), fvs) == TV2.Index(TV2.ByCoordinates(9, 9, 9), tvs)
    e = tf.Edge.ByStartVertexEndVertex(fvs[0], fvs[1])
    assert tf.Topology.IsSame(e, e) is True
    assert tf.Topology.IsSame(e, tf.Edge.ByStartVertexEndVertex(fvs[0], fvs[2])) is False


def test_boolean_ops():
    fa, fb = tf.Cell.Box(0, 0, 0, 2, 2, 2), tf.Cell.Box(1, 0, 0, 2, 2, 2)
    assert round(tf.Topology.Union(fa, fb).Volume(), 3) == 12.0
    # topologicpy signature (extra tranDict/tolerance/silent args) accepted
    assert round(tf.Topology.Union(fa, fb, False, 0.0001).Volume(), 3) == 12.0
    assert round(tf.Topology.Difference(fa, fb).Volume(), 3) == 4.0
    assert round(tf.Topology.Intersect(fa, fb).Volume(), 3) == 4.0  # topologicpy name
    assert round(tf.Topology.Intersection(fa, fb).Volume(), 3) == 4.0  # tf-native name


def test_vertex_predicates_match():
    from topologicpy.Vertex import Vertex as TV2

    def fmk(cs):
        return [tf.Vertex.ByCoordinates(*c) for c in cs]

    def tmk(cs):
        return [TV2.ByCoordinates(*c) for c in cs]

    collinear = [[(0, 0, 0), (1, 1, 1), (2, 2, 2)], [(0, 0, 0), (1, 0, 0), (0, 1, 0)]]
    for cs in collinear:
        assert tf.Vertex.AreCollinear(fmk(cs)) == TV2.AreCollinear(tmk(cs))
    coplanar = [[(0, 0, 0), (1, 0, 0), (0, 1, 0), (1, 1, 0)],
                [(0, 0, 0), (1, 0, 0), (0, 1, 0), (0, 0, 1)]]
    for cs in coplanar:
        assert tf.Vertex.AreCoplanar(fmk(cs)) == TV2.AreCoplanar(tmk(cs))
    assert tf.Vertex.IsCoincident(tf.Vertex.ByCoordinates(1, 1, 1), tf.Vertex.ByCoordinates(1, 1, 1.00001)) \
        == TV2.IsCoincident(TV2.ByCoordinates(1, 1, 1), TV2.ByCoordinates(1, 1, 1.00001)) is True
    cs = [(0, 0, 0), (2, 0, 0), (0, 3, 0)]
    assert tf.Vertex.Coordinates(tf.Vertex.Centroid(fmk(cs))) == TV2.Coordinates(TV2.Centroid(tmk(cs)))
    nv = tf.Vertex.NearestVertex(tf.Vertex.ByCoordinates(0.1, 0.1, 0.1), tf.Cell.Box(0, 0, 0, 2, 2, 2))
    assert tf.Vertex.Coordinates(nv) == [0.0, 0.0, 0.0]


def test_edge_extras_match():
    from topologicpy.Edge import Edge as TE2
    assert _vset(tf.Edge.Line(length=2, direction=[1, 0, 0])) == \
        _tvset(TE2.Line(length=2, direction=[1, 0, 0]))
    e = tf.Edge.ByStartVertexEndVertex(tf.Vertex.ByCoordinates(0, 0, 0),
                                       tf.Vertex.ByCoordinates(2, 0, 0))
    te = TE2.ByStartVertexEndVertex(TV.ByCoordinates(0, 0, 0), TV.ByCoordinates(2, 0, 0))
    assert _vset(tf.Edge.NormalEdge(e, length=1, u=0.5)) == _tvset(TE2.NormalEdge(te, length=1, u=0.5))
    eb = tf.Edge.ExternalBoundary(e)
    assert tf.Topology.TypeAsString(eb) == "Cluster" and len(tf.Topology.Vertices(eb)) == 2
    cl = tf.Cluster.ByVertices([tf.Vertex.ByCoordinates(0, 0, 0), tf.Vertex.ByCoordinates(5, 0, 0)])
    assert tf.Edge.Length(tf.Edge.ByVerticesCluster(cl)) == 5.0


def _vset(t):
    return sorted(tuple(round(c, 4) for c in tf.Vertex.Coordinates(v))
                  for v in tf.Topology.Vertices(t))


def _tvset(t):
    return sorted(tuple(round(c, 4) for c in TV.Coordinates(v)) for v in TT.Vertices(t))


def test_wire_and_face_shapes_match():
    from topologicpy.Wire import Wire as TW
    from topologicpy.Face import Face as TF2
    assert _vset(tf.Wire.Rectangle(width=2, length=3)) == _tvset(TW.Rectangle(width=2, length=3))
    assert _vset(tf.Wire.Circle(radius=1, sides=12)) == _tvset(TW.Circle(radius=1, sides=12))
    # camelCase fromAngle/toAngle kwargs (topologicpy signature) accepted
    assert _vset(tf.Wire.Circle(radius=1, sides=8, fromAngle=0, toAngle=360)) == \
        _tvset(TW.Circle(radius=1, sides=8, fromAngle=0, toAngle=360))
    assert _vset(tf.Face.Rectangle(width=2, length=3)) == _tvset(TF2.Rectangle(width=2, length=3))


def test_cell_cylinder_matches():
    fcyl = tf.Cell.Cylinder(radius=1, height=2, uSides=16)
    tcyl = TC.Cylinder(radius=1, height=2, uSides=16)
    assert round(fcyl.Volume(), 4) == round(TC.Volume(tcyl), 4)
    f_com = [round(c, 4) for c in tf.Vertex.Coordinates(tf.Topology.Centroid(fcyl))]
    t_com = [round(c, 4) for c in TV.Coordinates(TT.Centroid(tcyl))]
    assert f_com == t_com == [0.0, 0.0, 0.0]  # 'center' placement


def _bench(op, n=4000):
    op()
    t0 = time.perf_counter()
    for _ in range(n):
        op()
    return (time.perf_counter() - t0) / n


@pytest.mark.parametrize("name,native,reference", [
    ("Cell.Volume", lambda b: tf.Cell.Volume(b[0]), lambda b: TC.Volume(b[1])),
    ("Vertex.Coordinates",
     lambda b: tf.Vertex.Coordinates(tf.Vertex.ByCoordinates(1, 2, 3)),
     lambda b: TV.Coordinates(TV.ByCoordinates(1, 2, 3))),
])
def test_native_is_faster(name, native, reference):
    b = (_fbox(), _tbox())
    t_native = _bench(lambda: native(b))
    t_ref = _bench(lambda: reference(b))
    # The native path must be clearly faster (it is typically 5-200x).
    assert t_native < t_ref, f"{name}: native {t_native*1e6:.2f}us not < topologicpy {t_ref*1e6:.2f}us"


def test_geometry_extras_match():
    from topologicpy.Cell import Cell as TC2
    fbox, tbox = _fbox(), _tbox()
    assert tf.Cell.SurfaceArea(fbox) == TC2.SurfaceArea(tbox) == 52.0
    assert round(tf.Cell.Volume(tf.Topology.BoundingBox(fbox)), 3) == 24.0
    fd, td = tf.Cell.Decompose(fbox), TC2.Decompose(tbox)
    assert set(fd.keys()) == set(td.keys())
    for k in fd:
        assert len(fd[k]) == len(td[k]), k


def test_merge_and_graph_distances():
    b1, b2 = tf.Cell.Box(0, 0, 0, 2, 2, 2), tf.Cell.Box(0, 0, 2, 2, 2, 2)
    sm = tf.Topology.SelfMerge(tf.Cluster.ByCells([b1, b2]))
    assert tf.Topology.TypeAsString(sm) == "CellComplex" and len(tf.Topology.Faces(sm)) == 11
    mg = tf.Topology.Merge(tf.Cell.Box(0, 0, 0, 2, 2, 2), tf.Cell.Box(0, 0, 2, 2, 2, 2))
    assert tf.Topology.TypeAsString(mg) == "CellComplex" and len(tf.Topology.Faces(mg)) == 11
    vs = [tf.Vertex.ByCoordinates(0, 0, 0), tf.Vertex.ByCoordinates(1, 0, 0), tf.Vertex.ByCoordinates(1, 1, 0)]
    es = [tf.Edge.ByStartVertexEndVertex(vs[0], vs[1]), tf.Edge.ByStartVertexEndVertex(vs[1], vs[2])]
    g = tf.Graph.ByVerticesEdges(vs, es)
    assert tf.Graph.TopologicalDistance(g, vs[0], vs[2]) == 2
    assert tf.Graph.MetricDistance(g, vs[0], vs[2]) == 2.0


def test_decompose_matches():
    from topologicpy.CellComplex import CellComplex as TCC
    from topologicpy.Cell import Cell as TC3
    fcc = tf.CellComplex.ByCells([tf.Cell.Box(0, 0, 0, 2, 2, 2), tf.Cell.Box(0, 0, 2, 2, 2, 2)])
    tcc = TCC.ByCells([TT.Translate(TC3.Prism(width=2, length=2, height=2), 1, 1, 1),
                       TT.Translate(TC3.Prism(width=2, length=2, height=2), 1, 1, 3)])
    fd, td = tf.CellComplex.Decompose(fcc), TCC.Decompose(tcc)
    assert set(fd.keys()) == set(td.keys())
    for k in td:
        fl = len(fd[k]) if isinstance(fd[k], list) else fd[k]
        tl = len(td[k]) if isinstance(td[k], list) else td[k]
        assert fl == tl, k
    assert len(tf.Topology.Decompose(fcc)["cells"]) == 2


def test_thickened_face_and_present_methods():
    from topologicpy.Cell import Cell as TC4
    from topologicpy.Face import Face as TF4
    ff, tfa = tf.Face.Rectangle(width=2, length=2), TF4.Rectangle(width=2, length=2)
    for both in (True, False):
        fc = tf.Cell.ByThickenedFace(ff, thickness=0.5, bothSides=both)
        tc = TC4.ByThickenedFace(tfa, thickness=0.5, bothSides=both)
        assert round(fc.Volume(), 4) == round(TC4.Volume(tc), 4) == 2.0
    # Dictionary static-style calls match topologicpy
    from topologicpy.Dictionary import Dictionary as TD4
    d, td = tf.Dictionary.ByKeysValues(["a", "b"], [1, 2]), TD4.ByKeysValues(["a", "b"], [1, 2])
    assert sorted(tf.Dictionary.Keys(d)) == sorted(TD4.Keys(td))
    assert tf.Dictionary.ValueAtKey(d, "a") == TD4.ValueAtKey(td, "a") == 1


def test_compactness_and_closeness_fixed():
    from topologicpy.Face import Face as TF5
    from topologicpy.Graph import Graph as TG5
    from topologicpy.Edge import Edge as TE5
    from topologicpy.Vertex import Vertex as TV5
    ff, tfa = tf.Face.Rectangle(width=2, length=3), TF5.Rectangle(width=2, length=3)
    assert tf.Face.Compactness(ff) == round(TF5.Compactness(tfa), 6)
    coords = [(0, 0, 0), (1, 0, 0), (2, 0, 0), (1, 1, 0)]
    edges = [(0, 1), (1, 2), (1, 3)]
    fvs = [tf.Vertex.ByCoordinates(*c) for c in coords]
    fes = [tf.Edge.ByStartVertexEndVertex(fvs[a], fvs[b]) for a, b in edges]
    tvs = [TV5.ByCoordinates(*c) for c in coords]
    tes = [TE5.ByStartVertexEndVertex(tvs[a], tvs[b]) for a, b in edges]
    fg, tg = tf.Graph.ByVerticesEdges(fvs, fes), TG5.ByVerticesEdges(tvs, tes)
    assert sorted(round(x, 4) for x in tf.Graph.ClosenessCentrality(fg)) == \
        sorted(round(x, 4) for x in TG5.ClosenessCentrality(tg))


def test_add_apertures():
    # A window coplanar with and inside a wall face attaches; a far one does not.
    wall = tf.Face.Rectangle(width=4, length=3)
    window = tf.Face.Rectangle(width=2, length=1)
    wall = tf.Topology.AddApertures(wall, [window])
    aps = tf.Topology.Apertures(wall)
    assert len(aps) == 1 and round(aps[0].Area(None), 3) == 2.0
    wall2 = tf.Face.Rectangle(width=4, length=3)
    far = tf.Topology.Translate(tf.Face.Rectangle(width=1, length=1), 10, 0, 0)
    tf.Topology.AddApertures(wall2, [far])
    assert len(tf.Topology.Apertures(wall2)) == 0
    # UUID is stable per kernel entity.
    f = tf.Topology.Faces(tf.Cell.Box(0, 0, 0, 1, 1, 1))[0]
    assert tf.Topology.UUID(tf.Topology.Faces(tf.Cell.Box(0, 0, 0, 1, 1, 1))[0]) is not None
