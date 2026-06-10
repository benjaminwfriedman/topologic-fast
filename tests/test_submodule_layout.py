"""topologicpy-compatible submodule layout.

topologicpy code imports `from topologicpy.Vertex import Vertex`. topologic_fast
must support the same shape (`from topologic_fast.Vertex import Vertex`) so
migration is a search-and-replace of the import root, while keeping flat
`tf.Vertex` access working.
"""
import pytest

tf = pytest.importorskip("topologic_fast")

CLASSES = ["Vertex", "Edge", "Wire", "Face", "Shell", "Cell", "CellComplex",
           "Cluster", "Topology", "Dictionary", "Graph", "Vector", "Matrix", "Grid"]


@pytest.mark.parametrize("name", CLASSES)
def test_submodule_import_matches_flat(name):
    import importlib
    mod = importlib.import_module("topologic_fast.{}".format(name))
    cls = getattr(mod, name)               # from topologic_fast.X import X
    assert cls is getattr(tf, name)        # same object as flat tf.X


def test_flat_access_survives_submodule_import():
    # Importing a submodule must not turn flat tf.Vertex into a module.
    from topologic_fast.Vertex import Vertex  # noqa: F401
    assert tf.Cell.Box(0, 0, 0, 1, 1, 1).Volume() == 1.0
    assert len(tf.Topology.Faces(tf.Cell.Box(0, 0, 0, 1, 1, 1))) == 6


def test_migration_style_code_runs():
    from topologic_fast.Vertex import Vertex
    from topologic_fast.Edge import Edge
    from topologic_fast.Cell import Cell
    from topologic_fast.Topology import Topology
    e = Edge.ByStartVertexEndVertex(Vertex.ByCoordinates(0, 0, 0), Vertex.ByCoordinates(3, 4, 0))
    assert Edge.Length(e) == 5.0
    box = Cell.Box(0, 0, 0, 2, 3, 4)
    assert Cell.Volume(box) == 24.0 and len(Topology.Faces(box)) == 6


def test_energymodel_alias():
    from topologic_fast.EnergyModel import EnergyModel
    assert EnergyModel is tf.Energy
