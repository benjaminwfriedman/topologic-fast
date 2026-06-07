"""Smoke tests for the TopologicFastBackend Core seam.

Verifies the backend conforms to the shape topologicpy's ``Core`` facade expects:
it can be installed via ``Core.SetBackend`` and resolves the provided namespaces.
Pending namespaces raise a clear ``NotImplementedError`` until later phases.
"""
import pytest

pytest.importorskip("topologic_fast")

from topologic_fast.backend import TopologicFastBackend, _UnimplementedNamespace  # noqa: E402


def test_backend_provides_primitive_namespaces():
    backend = TopologicFastBackend()
    provided = backend.ProvidedNamespaces()
    for ns in ["Vertex", "Edge", "Face", "Cell", "CellComplex", "Dictionary", "Graph"]:
        assert ns in provided, f"{ns} should be backed by topologic_fast"
    # A provided namespace exposes real factory methods.
    assert hasattr(backend.Vertex, "ByCoordinates")
    assert hasattr(backend.Cell, "Box")
    # The Topology adapter recognizes any tf topology via isinstance.
    import topologic_fast as tf
    assert isinstance(tf.Cell.Box(0, 0, 0, 1, 1, 1), backend.Topology)
    assert isinstance(tf.Vertex.ByCoordinates(0, 0, 0), backend.Topology)


def test_pending_namespaces_raise_clear_error():
    backend = TopologicFastBackend()
    assert isinstance(backend.Aperture, _UnimplementedNamespace)
    assert isinstance(backend.Context, _UnimplementedNamespace)
    with pytest.raises(NotImplementedError):
        backend.Aperture.ByTopologyContext()


def test_installs_into_topologicpy_core():
    pytest.importorskip("topologicpy")
    from topologicpy.Core import Core
    try:
        Core.SetBackend(TopologicFastBackend())
        assert Core.HasNamespace("Vertex")
        assert Core.HasNamespace("Cell")
        # A factory call routes through the fast kernel.
        v = Core.Call("Vertex", "ByCoordinates", 1.0, 2.0, 3.0)
        assert tuple(v.Coordinates()) == (1.0, 2.0, 3.0)
    finally:
        Core.ResetBackend()  # restore the default topologic_core backend
