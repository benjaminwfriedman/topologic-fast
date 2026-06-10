"""Kernel adapters for differential parity testing.

A *case* is written once against the abstract :class:`Kernel` vocabulary and run
through two adapters:

* :class:`CoreKernel` — topologicpy on the reference ``topologic_core`` kernel.
* :class:`FastKernel` — the ``topologic_fast`` Rust kernel.

The adapters translate the shared vocabulary into each library's own idioms, so
the cases stay library-agnostic and the translation differences live here. This
is also the seed of the eventual ``TopologicFastBackend`` contract.
"""
from __future__ import annotations


class Kernel:
    """Abstract vocabulary that parity cases are written against."""

    name = "abstract"

    # --- construction ---------------------------------------------------
    def vertex(self, x, y, z): raise NotImplementedError
    def edge(self, a, b): raise NotImplementedError
    def box(self, x, y, z, w, l, h): raise NotImplementedError
    def cellcomplex(self, cells): raise NotImplementedError

    # --- downward traversal --------------------------------------------
    def vertices(self, t): raise NotImplementedError
    def edges(self, t): raise NotImplementedError
    def faces(self, t): raise NotImplementedError
    def cells(self, t): raise NotImplementedError

    # --- measures -------------------------------------------------------
    def coords(self, v): raise NotImplementedError
    def length(self, e): raise NotImplementedError
    def area(self, f): raise NotImplementedError
    def volume(self, c): raise NotImplementedError
    def center_of_mass(self, t): raise NotImplementedError

    # --- adjacency (exposes non-manifold sharing) -----------------------
    def face_cell_counts(self, complex_):
        """Sorted list of '#cells adjacent to each face' for a cell complex."""
        raise NotImplementedError

    # --- derived helpers (kernel-agnostic) ------------------------------
    def vertex_set(self, t, nd=6):
        """Canonical, order-independent set of rounded vertex coordinates."""
        return sorted(tuple(round(c, nd) for c in self.coords(v)) for v in self.vertices(t))

    def total_cell_volume(self, complex_):
        return sum(self.volume(c) for c in self.cells(complex_))


class CoreKernel(Kernel):
    name = "topologic_core"

    def __init__(self):
        from topologicpy.Vertex import Vertex
        from topologicpy.Edge import Edge
        from topologicpy.Face import Face
        from topologicpy.Cell import Cell
        from topologicpy.CellComplex import CellComplex
        from topologicpy.Topology import Topology
        self._V, self._E, self._F = Vertex, Edge, Face
        self._C, self._CC, self._T = Cell, CellComplex, Topology

    def vertex(self, x, y, z):
        return self._V.ByCoordinates(x, y, z)

    def edge(self, a, b):
        return self._E.ByStartVertexEndVertex(a, b)

    def box(self, x, y, z, w, l, h):
        c = self._C.Prism(width=w, length=l, height=h)
        return self._T.Translate(c, x + w / 2.0, y + l / 2.0, z + h / 2.0)

    def cellcomplex(self, cells):
        return self._CC.ByCells(cells)

    def translate(self, t, x, y, z):
        return self._T.Translate(t, x, y, z)

    def vertices(self, t):
        return self._T.SubTopologies(t, "Vertex")

    def edges(self, t):
        return self._T.SubTopologies(t, "Edge")

    def faces(self, t):
        return self._T.SubTopologies(t, "Face")

    def cells(self, t):
        return self._T.SubTopologies(t, "Cell")

    def coords(self, v):
        return tuple(self._V.Coordinates(v))

    def length(self, e):
        return self._E.Length(e)

    def area(self, f):
        return self._F.Area(f)

    def volume(self, c):
        return self._C.Volume(c)

    def center_of_mass(self, t):
        return tuple(self._V.Coordinates(self._T.CenterOfMass(t)))

    def face_cell_counts(self, complex_):
        return sorted(
            len(self._T.AdjacentTopologies(f, complex_, "cell"))
            for f in self.faces(complex_)
        )


class FastKernel(Kernel):
    name = "topologic_fast"

    def __init__(self):
        import topologic_fast as tf
        self._tf = tf

    def vertex(self, x, y, z):
        return self._tf.Vertex.ByCoordinates(x, y, z)

    def edge(self, a, b):
        return self._tf.Edge.ByStartVertexEndVertex(a, b)

    def box(self, x, y, z, w, l, h):
        return self._tf.Cell.Box(x, y, z, w, l, h)

    def cellcomplex(self, cells):
        return self._tf.CellComplex.ByCells(cells)

    def translate(self, t, x, y, z):
        return self._tf.Topology.Translate(t, x, y, z)

    def vertices(self, t):
        return t.Vertices()

    def edges(self, t):
        return t.Edges()

    def faces(self, t):
        return t.Faces()

    def cells(self, t):
        return t.Cells()

    def coords(self, v):
        return tuple(v.Coordinates())

    def length(self, e):
        return e.Length()

    def area(self, f):
        return f.Area()

    def volume(self, c):
        return c.Volume()

    def center_of_mass(self, t):
        return tuple(t.CenterOfMass())

    def face_cell_counts(self, complex_):
        return sorted(len(f.Cells()) for f in complex_.Faces())
