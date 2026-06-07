"""End-to-end drop-in test: topologicpy's OWN Python layer on the fast kernel.

The strongest parity oracle: run the *same* topologicpy methods twice — once on
the stock ``topologic_core`` kernel, once with
``Core.SetBackend(TopologicFastBackend())`` so they execute on ``topologic_fast``
— and assert identical results.

Each case is written once against the topologicpy API and an equivalent box is
built natively in each kernel. As the backend contract grows, add cases here;
cases that exercise not-yet-supported kernel calls are marked xfail.

Skipped if topologicpy is not installed.
"""
import pytest

tf = pytest.importorskip("topologic_fast")
pytest.importorskip("topologicpy")

from topologicpy.Core import Core  # noqa: E402
from topologicpy.Topology import Topology  # noqa: E402
from topologicpy.Vertex import Vertex  # noqa: E402
from topologicpy.Cell import Cell as TCell  # noqa: E402
from topologic_fast.backend import TopologicFastBackend  # noqa: E402


def _coords_set(vertices):
    return sorted(tuple(round(c, 5) for c in Vertex.Coordinates(v)) for v in vertices)


# Each case: fn(box) -> comparable value, using the topologicpy API only.
CASES = [
    ("type_as_string", lambda b: Topology.TypeAsString(b), None),
    ("dimensionality", lambda b: Topology.Dimensionality(b), None),
    ("num_vertices", lambda b: len(Topology.Vertices(b)), None),
    ("num_edges", lambda b: len(Topology.Edges(b)), None),
    ("num_faces", lambda b: len(Topology.Faces(b)), None),
    ("vertex_coords", lambda b: _coords_set(Topology.Vertices(b)), None),
    ("subtopologies_faces", lambda b: len(Topology.SubTopologies(b, "face")), None),
    ("translate_vertices", lambda b: _coords_set(Topology.Vertices(Topology.Translate(b, 10, -3, 2))), None),
    ("face_vertex_counts",
     lambda b: sorted(len(Topology.Vertices(f)) for f in Topology.Faces(b)), None),
]


def _run_stock():
    Core.ResetBackend()
    box = Topology.Translate(TCell.Prism(width=2, length=3, height=4), 1, 1.5, 2)
    return {name: fn(box) for name, fn, _ in CASES}


def _run_fast():
    Core.SetBackend(TopologicFastBackend())
    try:
        box = tf.Cell.Box(0, 0, 0, 2, 3, 4)
        return {name: fn(box) for name, fn, _ in CASES}
    finally:
        Core.ResetBackend()


_STOCK = _run_stock()
_FAST = _run_fast()


@pytest.mark.parametrize("name,fn,xfail", CASES, ids=[c[0] for c in CASES])
def test_dropin_matches_stock(name, fn, xfail):
    if xfail:
        pytest.xfail(xfail)
    assert _FAST[name] == _STOCK[name], f"{name}: fast={_FAST[name]} stock={_STOCK[name]}"
