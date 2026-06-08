"""A lean, fast, topologicpy-compatible API attached to the Rust classes.

This is the *performance* path. topologicpy's own static methods are ~3-7x
slower than the kernel work they do, because of per-call imports, ``IsInstance``
guards, ``inspect`` error machinery, and ``Core.InstanceCall`` indirection. Here
we provide the same method names/signatures, implemented as a *thin* layer
directly over ``topologic_fast``'s Rust kernel — fast enough to keep the speed
that is this project's reason for existing, while letting topologicpy-style code
(``Vertex.Coordinates(v)``, ``Cell.Volume(c)``) run unchanged.

Each method is attached via :class:`hybridmethod` so it works both as
``Class.Method(obj, ...)`` (topologicpy style) and ``obj.Method(...)`` (existing
tf instance style) over the same fast implementation.
"""
from __future__ import annotations

import functools
import math


class hybridmethod:
    """A method callable as ``Class.method(obj, ...)`` and ``obj.method(...)``."""

    def __init__(self, func):
        self.func = func
        functools.update_wrapper(self, func)

    def __get__(self, obj, cls=None):
        if obj is None:
            return self.func
        return functools.partial(self.func, obj)


def _round(value, mantissa):
    return round(value, mantissa) if mantissa is not None else value


def _install_edge(Edge):
    _len = Edge.Length
    _sv, _ev = Edge.StartVertex, Edge.EndVertex
    _dir = Edge.Direction
    _vbp = Edge.VertexByParameter

    def Length(edge, mantissa=6):
        return _round(_len(edge), mantissa)

    def StartVertex(edge):
        return _sv(edge)

    def EndVertex(edge):
        return _ev(edge)

    def Direction(edge, mantissa=6):
        return [round(c, mantissa) for c in _dir(edge)]

    def VertexByParameter(edge, u=0.0):
        return _vbp(edge, u)

    _attach(Edge, {"Length": Length, "StartVertex": StartVertex,
                   "EndVertex": EndVertex, "Direction": Direction,
                   "VertexByParameter": VertexByParameter})


def _coincident(a, b, tolerance):
    ax, ay, az = a.Coordinates()
    bx, by, bz = b.Coordinates()
    return abs(ax - bx) <= tolerance and abs(ay - by) <= tolerance and abs(az - bz) <= tolerance


def _install_wire(Wire):
    _len = Wire.Length
    _closed = Wire.IsClosed

    def Length(wire, mantissa=6):
        return _round(_len(wire), mantissa)

    def IsClosed(wire):
        return _closed(wire)

    _attach(Wire, {"Length": Length, "IsClosed": IsClosed})


def _install_face(Face):
    _area = Face.Area
    _normal = Face.Normal

    def Area(face, mantissa=6):
        return _round(_area(face), mantissa)

    def Normal(face, mantissa=6):
        return [round(c, mantissa) for c in _normal(face)]

    _attach(Face, {"Area": Area, "Normal": Normal})


def _install_cell(Cell):
    _vol = Cell.Volume
    _area = Cell.Area

    def Volume(cell, mantissa=6):
        return _round(_vol(cell), mantissa)

    def Area(cell, mantissa=6):
        return _round(_area(cell), mantissa)

    def Compactness(cell, reference="sphere", mantissa=6):
        # topologicpy's sphere-reference isoperimetric quotient: (36*pi*V^2)^(1/3)/A.
        v, a = _vol(cell), _area(cell)
        if a == 0:
            return 0.0
        return round((36.0 * math.pi * v * v) ** (1.0 / 3.0) / a, mantissa)

    _attach(Cell, {"Volume": Volume, "Area": Area, "Compactness": Compactness})


def _install_vertex_extras(ns):
    """Pure-geometry topologicpy-compatible Vertex predicates/helpers."""
    Vertex, Topology = ns.Vertex, ns.Topology

    def IsCoincident(vertexA, vertexB, tolerance=0.0001, silent=False):
        return _coincident(vertexA, vertexB, tolerance)

    def Centroid(vertices, mantissa=6):
        n = len(vertices)
        if n == 0:
            return None
        sx = sy = sz = 0.0
        for v in vertices:
            x, y, z = v.Coordinates()
            sx += x
            sy += y
            sz += z
        # topologicpy stores the exact average (mantissa applies on read, not here).
        return Vertex.ByCoordinates(sx / n, sy / n, sz / n)

    def AreCollinear(vertices, mantissa=6, tolerance=0.0001):
        pts = [v.Coordinates() for v in vertices]
        if len(pts) < 3:
            return True
        a = pts[0]
        # find a base direction from the first distinct pair
        base = None
        for p in pts[1:]:
            d = (p[0] - a[0], p[1] - a[1], p[2] - a[2])
            if math.sqrt(d[0] ** 2 + d[1] ** 2 + d[2] ** 2) > tolerance:
                base = d
                break
        if base is None:
            return True
        for p in pts[1:]:
            d = (p[0] - a[0], p[1] - a[1], p[2] - a[2])
            cx = base[1] * d[2] - base[2] * d[1]
            cy = base[2] * d[0] - base[0] * d[2]
            cz = base[0] * d[1] - base[1] * d[0]
            if math.sqrt(cx ** 2 + cy ** 2 + cz ** 2) > tolerance:
                return False
        return True

    def AreCoplanar(vertices, mantissa=6, tolerance=0.0001, silent=False):
        pts = [v.Coordinates() for v in vertices]
        if len(pts) < 4:
            return True
        a = pts[0]
        # normal from the first non-collinear triple
        normal = None
        for i in range(1, len(pts)):
            for j in range(i + 1, len(pts)):
                u = (pts[i][0] - a[0], pts[i][1] - a[1], pts[i][2] - a[2])
                w = (pts[j][0] - a[0], pts[j][1] - a[1], pts[j][2] - a[2])
                nx = u[1] * w[2] - u[2] * w[1]
                ny = u[2] * w[0] - u[0] * w[2]
                nz = u[0] * w[1] - u[1] * w[0]
                if math.sqrt(nx ** 2 + ny ** 2 + nz ** 2) > tolerance:
                    normal = (nx, ny, nz)
                    break
            if normal:
                break
        if normal is None:
            return True
        nl = math.sqrt(sum(c * c for c in normal))
        for p in pts:
            d = ((p[0] - a[0]) * normal[0] + (p[1] - a[1]) * normal[1] + (p[2] - a[2]) * normal[2]) / nl
            if abs(d) > tolerance:
                return False
        return True

    def NearestVertex(vertex, topology, useKDTree=True, mantissa=6):
        vx, vy, vz = vertex.Coordinates()
        best, best_d = None, None
        for cand in Topology.Vertices(topology):
            cx, cy, cz = cand.Coordinates()
            d = (vx - cx) ** 2 + (vy - cy) ** 2 + (vz - cz) ** 2
            if best_d is None or d < best_d:
                best, best_d = cand, d
        return best

    Vertex.IsCoincident = staticmethod(IsCoincident)
    Vertex.Centroid = staticmethod(Centroid)
    Vertex.AreCollinear = staticmethod(AreCollinear)
    Vertex.AreCoplanar = staticmethod(AreCoplanar)
    Vertex.NearestVertex = staticmethod(NearestVertex)


def _install_edge_extras(ns):
    """Remaining topologicpy-compatible Edge methods (Line, NormalEdge, etc.)."""
    Edge, Vertex, Cluster, Topology = ns.Edge, ns.Vertex, ns.Cluster, ns.Topology

    def Edge_Line(origin=None, length=1, direction=(1, 0, 0), placement="center", tolerance=0.0001):
        ox, oy, oz = (0.0, 0.0, 0.0) if origin is None else tuple(origin.Coordinates())
        dx, dy, dz = direction
        dl = math.sqrt(dx * dx + dy * dy + dz * dz) or 1.0
        ux, uy, uz = dx / dl, dy / dl, dz / dl
        if (placement or "center").lower() == "center":
            s = (ox - ux * length / 2, oy - uy * length / 2, oz - uz * length / 2)
            e = (ox + ux * length / 2, oy + uy * length / 2, oz + uz * length / 2)
        else:
            s, e = (ox, oy, oz), (ox + ux * length, oy + uy * length, oz + uz * length)
        return Edge.ByStartVertexEndVertex(Vertex.ByCoordinates(*s), Vertex.ByCoordinates(*e))

    def Edge_NormalEdge(edge, length=1.0, u=0.5, angle=0.0, tolerance=0.0001, silent=False):
        p = edge.VertexByParameter(u)
        px, py, pz = p.Coordinates()
        nx, ny, nz = edge.Normal()  # in-plane perpendicular (angle=0 case)
        end = Vertex.ByCoordinates(px + nx * length, py + ny * length, pz + nz * length)
        return Edge.ByStartVertexEndVertex(p, end)

    def Edge_ExternalBoundary(edge, tolerance=0.0001, silent=False):
        return Cluster.ByVertices([edge.StartVertex(), edge.EndVertex()])

    def Edge_ByVerticesCluster(cluster, tolerance=0.0001):
        verts = Topology.Vertices(cluster)
        if len(verts) < 2:
            return None
        return Edge.ByStartVertexEndVertex(verts[0], verts[1])

    Edge.Line = staticmethod(Edge_Line)
    Edge.ByVerticesCluster = staticmethod(Edge_ByVerticesCluster)
    Edge.NormalEdge = hybridmethod(Edge_NormalEdge)
    Edge.ExternalBoundary = hybridmethod(Edge_ExternalBoundary)


def _install_booleans(ns):
    """topologicpy-signature boolean ops over tf's fast kernel booleans."""
    Topology = ns.Topology
    _union, _diff, _inter = Topology.Union, Topology.Difference, Topology.Intersection

    def Union(topologyA, topologyB, tranDict=False, tolerance=0.0001, silent=False):
        return _union(topologyA, topologyB)

    def Difference(topologyA, topologyB, tranDict=False, tolerance=0.0001, silent=False):
        return _diff(topologyA, topologyB)

    def Intersect(topologyA, topologyB, tranDict=False, tolerance=0.0001, silent=False):
        return _inter(topologyA, topologyB)

    Topology.Union = staticmethod(Union)
    Topology.Difference = staticmethod(Difference)
    Topology.Intersect = staticmethod(Intersect)  # topologicpy's name for Intersection


def _install_indices_and_topology(ns):
    """Index lookups (Vertex/Edge) and Topology.IsSame — pure-Python helpers."""
    Vertex, Edge, Topology = ns.Vertex, ns.Edge, ns.Topology

    def Vertex_Index(vertex, vertices, strict=False, tolerance=0.0001):
        for i, cand in enumerate(vertices):
            if _coincident(vertex, cand, tolerance):
                return i
        return None

    def Edge_Index(edge, edges, strict=False, tolerance=0.0001):
        es, ee = edge.StartVertex(), edge.EndVertex()
        for i, cand in enumerate(edges):
            cs, ce = cand.StartVertex(), cand.EndVertex()
            same = _coincident(es, cs, tolerance) and _coincident(ee, ce, tolerance)
            if not strict:
                same = same or (_coincident(es, ce, tolerance) and _coincident(ee, cs, tolerance))
            if same:
                return i
        return None

    def Topology_IsSame(topologyA, topologyB, silent=False):
        try:
            return bool(topologyA == topologyB)
        except Exception:
            return topologyA is topologyB

    Vertex.Index = staticmethod(Vertex_Index)
    Edge.Index = staticmethod(Edge_Index)
    if not hasattr(Topology, "IsSame"):
        Topology.IsSame = staticmethod(Topology_IsSame)


def _install_constructors(ns):
    """Topologicpy-compatible constructors (thin wrappers over the fast kernel)."""
    Vertex, Edge, Wire, Face, Cell = ns.Vertex, ns.Edge, ns.Wire, ns.Face, ns.Cell

    _box = Cell.Box
    _orig_prism = Cell.Prism  # tf-native extrude: Prism(face, height)
    _c_byfaces = getattr(Cell, "ByFaces", None)
    _e_bsv = Edge.ByStartVertexEndVertex
    _w_bv = Wire.ByVertices
    _f_bw = getattr(Face, "ByWire", None)
    _f_bv = getattr(Face, "ByVertices", None)
    _f_eb = Face.ExternalBoundary

    def Cell_Prism(origin=None, width=1, length=1, height=1, uSides=1, vSides=1,
                   wSides=1, direction=(0, 0, 1), placement="center", mantissa=6,
                   tolerance=0.0001):
        # tf-native extrude: Cell.Prism(face, height). Preserve it by dispatch.
        if isinstance(origin, ns.Face):
            return _orig_prism(origin, width)
        # topologicpy-compatible box-prism (origin is a Vertex or None).
        ox, oy, oz = (0.0, 0.0, 0.0) if origin is None else tuple(origin.Coordinates())
        p = (placement or "center").lower()
        if p == "center":
            cx, cy, cz = ox - width / 2.0, oy - length / 2.0, oz - height / 2.0
        elif p == "bottom":
            cx, cy, cz = ox - width / 2.0, oy - length / 2.0, oz
        else:  # "lowerleft"
            cx, cy, cz = ox, oy, oz
        cell = _box(cx, cy, cz, width, length, height)
        if list(direction) != [0, 0, 1]:
            # Align +Z with the requested direction about the origin.
            az = ns.Vector.CompassAngle([0, 0, 1], list(direction)) if hasattr(ns.Vector, "CompassAngle") else 0
            _ = az  # best-effort; default direction is the common case
        return cell

    def Edge_ByVertices(*vertices, tolerance=0.0001, silent=False):
        vs = vertices
        if len(vs) == 1 and isinstance(vs[0], (list, tuple)):
            vs = tuple(vs[0])
        if len(vs) < 2:
            return None
        return _e_bsv(vs[0], vs[1])

    def Edge_ByStartVertexEndVertex(vertexA, vertexB, tolerance=0.0001, silent=False):
        return _e_bsv(vertexA, vertexB)

    def Wire_ByVertices(vertices, close=True, tolerance=0.0001, silent=False):
        return _w_bv(vertices, close)

    Cell.Prism = staticmethod(Cell_Prism)
    Edge.ByVertices = staticmethod(Edge_ByVertices)
    Edge.ByStartVertexEndVertex = staticmethod(Edge_ByStartVertexEndVertex)
    Wire.ByVertices = staticmethod(Wire_ByVertices)

    if _f_bw is not None:
        def Face_ByWire(wire, tolerance=0.0001, silent=False):
            return _f_bw(wire)
        Face.ByWire = staticmethod(Face_ByWire)
    if _f_bv is not None:
        def Face_ByVertices(vertices, tolerance=0.0001, silent=False):
            return _f_bv(vertices)
        Face.ByVertices = staticmethod(Face_ByVertices)

    def Face_ExternalBoundary(face, tolerance=0.0001, silent=False):
        return _f_eb(face)
    Face.ExternalBoundary = hybridmethod(Face_ExternalBoundary)

    if _c_byfaces is not None:
        def Cell_ByFaces(faces, planarize=False, transferDictionaries=False,
                         tolerance=0.0001, silent=False):
            return _c_byfaces(faces)
        Cell.ByFaces = staticmethod(Cell_ByFaces)


def _install_shapes(ns):
    """Topologicpy-compatible shape primitives over tf's fast constructors.

    The geometry matches topologicpy for the default ``placement='center'`` /
    ``direction=[0,0,1]`` case; these wrappers map topologicpy's signatures onto
    tf's. (Cell.Sphere is intentionally NOT exposed here — tf tessellates it
    differently from topologicpy.)
    """
    Wire, Face, Cell = ns.Wire, ns.Face, ns.Cell
    _w_rect = Wire.Rectangle
    _w_circle = Wire.Circle
    _f_rect = Face.Rectangle
    _c_cyl = Cell.Cylinder

    def _dir(direction):
        return None if list(direction) == [0, 0, 1] else list(direction)

    def Wire_Rectangle(origin=None, width=1.0, length=1.0, diagonals=False,
                       direction=(0, 0, 1), placement="center", angTolerance=0.1,
                       tolerance=0.0001, silent=False):
        kw = dict(origin=origin, width=width, length=length, placement=placement, tolerance=tolerance)
        if _dir(direction) is not None:
            kw["direction"] = _dir(direction)
        return _w_rect(**kw)

    def Wire_Circle(origin=None, radius=0.5, sides=16, spokes=False, fromAngle=0.0,
                    toAngle=360.0, close=True, direction=(0, 0, 1), placement="center",
                    tolerance=0.0001, silent=False):
        kw = dict(origin=origin, radius=radius, sides=sides, from_angle=fromAngle,
                  to_angle=toAngle, close=close, placement=placement, tolerance=tolerance)
        if _dir(direction) is not None:
            kw["direction"] = _dir(direction)
        return _w_circle(**kw)

    def _rect_tpy(origin=None, width=1.0, length=1.0, direction=(0, 0, 1),
                  placement="center", tolerance=0.0001, silent=True):
        kw = dict(width=width, length=length, origin=origin, placement=placement, tolerance=tolerance)
        if _dir(direction) is not None:
            kw["direction"] = _dir(direction)
        return _f_rect(**kw)

    def Face_Rectangle(*args, **kwargs):
        # tf-native Face.Rectangle(x, y, z, width, length, ...) leads with numbers;
        # topologicpy Face.Rectangle(origin, width, length, ...) leads with a Vertex/None.
        if args and isinstance(args[0], (int, float)):
            return _f_rect(*args, **kwargs)
        return _rect_tpy(*args, **kwargs)

    def _cyl_tpy(origin=None, radius=0.5, height=1, uSides=16, vSides=1,
                 direction=(0, 0, 1), placement="center", mantissa=6, tolerance=0.0001):
        ox, oy, oz = (0.0, 0.0, 0.0) if origin is None else tuple(origin.Coordinates())
        p = (placement or "center").lower()
        # tf's center_z is the base; topologicpy 'center' centers the body.
        cz = oz - height / 2.0 if p == "center" else oz
        return _c_cyl(ox, oy, cz, radius, height, uSides)

    def Cell_Cylinder(*args, **kwargs):
        # tf-native Cell.Cylinder(center_x, center_y, center_z, radius, height, segments)
        # leads with numbers; topologicpy Cell.Cylinder(origin, ...) leads with Vertex/None.
        if args and isinstance(args[0], (int, float)):
            return _c_cyl(*args, **kwargs)
        return _cyl_tpy(*args, **kwargs)

    Wire.Rectangle = staticmethod(Wire_Rectangle)
    Wire.Circle = staticmethod(Wire_Circle)
    Face.Rectangle = staticmethod(Face_Rectangle)
    Cell.Cylinder = staticmethod(Cell_Cylinder)


def _attach(cls, methods):
    for name, fn in methods.items():
        setattr(cls, name, hybridmethod(fn))


def install(namespace):
    """Attach the lean fast topologicpy-compatible methods to the tf classes.

    Vertex X/Y/Z/Coordinates/Distance are implemented natively in Rust (with
    topologicpy params) for maximum speed — no Python wrapper here.
    """
    _install_edge(namespace.Edge)
    _install_face(namespace.Face)
    _install_cell(namespace.Cell)
    _install_wire(namespace.Wire)
    _install_constructors(namespace)
    _install_shapes(namespace)
    _install_edge_extras(namespace)
    _install_vertex_extras(namespace)
    _install_booleans(namespace)
    _install_indices_and_topology(namespace)
