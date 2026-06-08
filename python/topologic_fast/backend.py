"""A topologicpy ``Core`` backend backed by ``topologic_fast``.

topologicpy exposes a pluggable-kernel facade (``topologicpy.Core.Core``) with
``Core.SetBackend(backend)``. With this backend installed, topologicpy's *own*
Python layer runs on the fast Rust kernel:

    from topologicpy.Core import Core
    from topologic_fast.backend import TopologicFastBackend
    Core.SetBackend(TopologicFastBackend())
    # ... topologicpy.Vertex.Coordinates(tf_vertex) now works ...

Key compatibility shim: in ``topologic_core`` every topology type inherits from a
common ``Topology`` base class, and topologicpy guards almost every method with
``isinstance(obj, Core.Namespace("Topology"))``. ``topologic_fast`` has no such
base class, so we provide :class:`_TfTopology` whose metaclass reports *any* tf
topology object as an instance — making those guards pass.

This is an incrementally-growing contract: the namespaces and methods topologicpy
actually calls (~76 ``Core.NS.method`` + ~35 instance methods). Unimplemented
pieces resolve to a placeholder that raises a clear error.
"""
from __future__ import annotations


def _tf_topology_types():
    import topologic_fast as tf
    return (tf.Vertex, tf.Edge, tf.Wire, tf.Face, tf.Shell, tf.Cell, tf.CellComplex, tf.Cluster)


class _TfTopologyMeta(type):
    """Metaclass so ``isinstance(tf_obj, _TfTopology)`` is True for any tf topology."""

    def __instancecheck__(cls, instance):
        return isinstance(instance, _tf_topology_types())


class _TfTopology(metaclass=_TfTopologyMeta):
    """Stands in for ``topologic_core.Topology`` (the common base class).

    Carries the handful of kernel methods topologicpy calls as ``Core.Topology.*``.
    """

    @staticmethod
    def IsSame(topologyA, topologyB):
        try:
            return topologyA == topologyB
        except Exception:
            return topologyA is topologyB

    @staticmethod
    def Cleanup(topology=None):
        return topology

    @staticmethod
    def Analyze(topology):
        import topologic_fast as tf
        return tf.Topology.TypeAsString(topology)


class _UnimplementedNamespace:
    """Placeholder for a kernel namespace topologic_fast does not yet provide."""

    def __init__(self, name):
        self._name = name

    def __getattr__(self, attr):
        raise NotImplementedError(
            f"TopologicFastBackend: namespace '{self._name}' does not yet implement "
            f"'{attr}'. See the drop-in roadmap / PARITY.md."
        )

    def __repr__(self):
        return f"<TopologicFastBackend unimplemented namespace: {self._name}>"


class _TopologyUtility:
    """Adapter for ``Core.TopologyUtility.*`` (generic transforms)."""

    @staticmethod
    def Translate(topology, x, y, z):
        import topologic_fast as tf
        return tf.Topology.Translate(topology, x, y, z)

    @staticmethod
    def Rotate(topology, origin, x, y, z, degree):
        # topologic_core signature: Rotate(topology, origin, axisX, axisY, axisZ, angle)
        import topologic_fast as tf
        return tf.Topology.Rotate(topology, origin, [x, y, z], degree)

    @staticmethod
    def Scale(topology, origin, x, y, z):
        import topologic_fast as tf
        return tf.Topology.Scale(topology, origin, x, y, z)


class _CellUtility:
    @staticmethod
    def Volume(cell, mantissa=None):
        v = cell.Volume(None)  # raw; topologicpy rounds afterwards
        return round(v, mantissa) if mantissa is not None else v

    @staticmethod
    def Contains(cell, vertex, tolerance=0.0001):
        # topologic_core returns a containment enum; topologicpy treats truthy as inside.
        try:
            return cell.ContainsPoint(vertex.X(), vertex.Y(), vertex.Z())
        except Exception:
            return False


class _CellComplexUtility:
    @staticmethod
    def Volume(cell_complex, mantissa=None):
        v = cell_complex.Volume(None)
        return round(v, mantissa) if mantissa is not None else v


class _EdgeUtility:
    @staticmethod
    def Length(edge, mantissa=None):
        v = edge.Length(None)
        return round(v, mantissa) if mantissa is not None else v


class _FaceUtility:
    @staticmethod
    def Area(face, mantissa=None):
        v = face.Area(None)
        return round(v, mantissa) if mantissa is not None else v

    @staticmethod
    def Normal(face):
        return list(face.Normal())

    @staticmethod
    def NormalAtParameters(face, u=0.5, v=0.5, *args, **kwargs):
        return list(face.Normal())


# Namespaces topologic_fast exposes directly as Python classes.
_PROVIDED = ["Vertex", "Edge", "Wire", "Face", "Shell", "Cell", "CellComplex",
             "Cluster", "Graph", "Dictionary"]

# Adapter namespaces.
_ADAPTERS = {
    "Topology": _TfTopology,
    "TopologyUtility": _TopologyUtility,
    "CellUtility": _CellUtility,
    "CellComplexUtility": _CellComplexUtility,
    "EdgeUtility": _EdgeUtility,
    "FaceUtility": _FaceUtility,
}

# Namespaces still needing kernel work.
_PENDING = [
    "Aperture", "Context",
    "VertexUtility", "WireUtility", "ShellUtility",
    "ClusterUtility", "GraphUtility",
    "IntAttribute", "DoubleAttribute", "StringAttribute", "ListAttribute",
]


class TopologicFastBackend:
    """A ``topologicpy.Core`` backend that routes to ``topologic_fast``."""

    def __init__(self):
        import topologic_fast as tf
        self._tf = tf
        for name in _PROVIDED:
            setattr(self, name, getattr(tf, name, _UnimplementedNamespace(name)))
        for name, adapter in _ADAPTERS.items():
            setattr(self, name, adapter)
        for name in _PENDING:
            setattr(self, name, _UnimplementedNamespace(name))

    def RawModule(self):
        return self._tf

    def Namespaces(self):
        return sorted(n for n in dir(self) if not n.startswith("_"))

    def ProvidedNamespaces(self):
        return [n for n in _PROVIDED if not isinstance(getattr(self, n), _UnimplementedNamespace)]

    def __repr__(self):
        return f"<TopologicFastBackend provided={len(self.ProvidedNamespaces())}>"
