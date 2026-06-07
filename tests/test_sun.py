"""Parity tests for topologic_fast.Sun against topologicpy.Sun.

Astronomical quantities (solstices, equinoxes, azimuth, altitude, sunrise,
sunset) are produced by the same `ephem` calls in both libraries and must match
*exactly*. Derived geometry (vectors, positions, vertices, edges, paths) must
match within a tight floating-point tolerance (topologicpy rounds direction
vectors to 6 decimals).

These tests are skipped if `topologicpy` or `ephem` is not installed.
"""
import math
from datetime import datetime, timedelta

import pytest

tf = pytest.importorskip("topologic_fast")
pytest.importorskip("ephem")
topologicpy = pytest.importorskip("topologicpy")

from topologic_fast import Sun as FSun  # noqa: E402
from topologicpy.Sun import Sun as TSun  # noqa: E402
from topologicpy.Vertex import Vertex as TVertex  # noqa: E402
from topologicpy.Topology import Topology as TTopology  # noqa: E402

# Astronomy is bit-exact (same ephem calls). Derived geometry agrees to ~1e-6
# in the 6th decimal of a unit vector; that rounding noise scales with the orbit
# radius, so position/path comparisons (radius up to 10) use a looser bound.
TOL = 1e-6
VEC_TOL = 1e-5
POS_TOL = 1e-4

# A spread of locations: N hemisphere, S hemisphere, equator, high latitude.
LOCATIONS = [
    (51.5074, -0.1278),    # London
    (-33.8688, 151.2093),  # Sydney
    (0.0, 0.0),            # Gulf of Guinea (equator/prime meridian)
    (40.7128, -74.0060),   # New York
    (64.1466, -21.9426),   # Reykjavik (high latitude)
]

DATES = [
    datetime(2024, 6, 21, 13, 30, 0),
    datetime(2024, 12, 21, 9, 0, 0),
    datetime(2023, 3, 20, 6, 15, 0),
    datetime(2025, 9, 22, 17, 45, 0),
]

YEARS = [2023, 2024, 2025]


def _coords(v):
    """Return rounded [x, y, z] for a topologic_fast or topologicpy vertex."""
    if hasattr(v, "Coordinates"):
        c = v.Coordinates()
        return [round(c[0], 6), round(c[1], 6), round(c[2], 6)]
    return [round(TVertex.X(v), 6), round(TVertex.Y(v), 6), round(TVertex.Z(v), 6)]


def _approx_xyz(a, b, tol=POS_TOL):
    assert len(a) == len(b)
    for x, y in zip(a, b):
        assert abs(float(x) - float(y)) <= tol, f"{a} != {b}"


# --------------------------------------------------------------------------- #
# Astronomy: must match exactly                                               #
# --------------------------------------------------------------------------- #
@pytest.mark.parametrize("lat,lon", LOCATIONS)
@pytest.mark.parametrize("date", DATES)
def test_azimuth_altitude_exact(lat, lon, date):
    assert FSun.Azimuth(lat, lon, date) == TSun.Azimuth(lat, lon, date)
    assert FSun.Altitude(lat, lon, date) == TSun.Altitude(lat, lon, date)


@pytest.mark.parametrize("lat,lon", LOCATIONS)
@pytest.mark.parametrize("date", DATES)
def test_sunrise_sunset_exact(lat, lon, date):
    assert FSun.Sunrise(lat, lon, date) == TSun.Sunrise(lat, lon, date)
    assert FSun.Sunset(lat, lon, date) == TSun.Sunset(lat, lon, date)


@pytest.mark.parametrize("lat,lon", LOCATIONS)
@pytest.mark.parametrize("year", YEARS)
def test_solstices_equinoxes_exact(lat, lon, year):
    assert FSun.WinterSolstice(lat, year) == TSun.WinterSolstice(lat, year)
    assert FSun.SummerSolstice(lat, year) == TSun.SummerSolstice(lat, year)
    assert FSun.SpringEquinox(lat, year) == TSun.SpringEquinox(lat, year)
    assert FSun.AutumnEquinox(lat, year) == TSun.AutumnEquinox(lat, year)


# --------------------------------------------------------------------------- #
# Geometry: must match within tolerance                                       #
# --------------------------------------------------------------------------- #
@pytest.mark.parametrize("lat,lon", LOCATIONS)
@pytest.mark.parametrize("date", DATES)
@pytest.mark.parametrize("north", [0, 30, 90, 180])
def test_vector(lat, lon, date, north):
    f = FSun.Vector(lat, lon, date, north=north)
    t = [float(x) for x in TSun.Vector(lat, lon, date, north=north)]
    _approx_xyz(f, t, tol=VEC_TOL)
    # Sun vectors are unit length.
    assert abs(math.sqrt(sum(c * c for c in f)) - 1.0) <= 1e-4


@pytest.mark.parametrize("lat,lon", LOCATIONS)
@pytest.mark.parametrize("date", DATES)
@pytest.mark.parametrize("radius", [0.5, 10.0])
@pytest.mark.parametrize("north", [0, 45])
def test_position_and_vertex(lat, lon, date, radius, north):
    f_pos = FSun.Position(lat, lon, date, radius=radius, north=north)
    t_vertex = TSun.Vertex(lat, lon, date, radius=radius, north=north)
    _approx_xyz(f_pos, _coords(t_vertex))

    # Vertex() must agree with Position().
    _approx_xyz(_coords(FSun.Vertex(lat, lon, date, radius=radius, north=north)), f_pos)


@pytest.mark.parametrize("lat,lon", LOCATIONS)
@pytest.mark.parametrize("date", DATES)
def test_edge_endpoints(lat, lon, date):
    f_edge = FSun.Edge(lat, lon, date, radius=10.0)
    t_edge = TSun.Edge(lat, lon, date, radius=10.0)
    # Start = sun position, End = origin (both libraries).
    _approx_xyz(_coords(f_edge.StartVertex()), _coords(TTopology.Vertices(t_edge)[0]))
    f_pos = FSun.Position(lat, lon, date, radius=10.0)
    _approx_xyz(_coords(f_edge.StartVertex()), f_pos)


@pytest.mark.parametrize("lat,lon", [LOCATIONS[0], LOCATIONS[1]])
def test_vertices_by_date(lat, lon):
    date = datetime(2024, 6, 21, 12, 0, 0)
    f_verts = FSun.VerticesByDate(lat, lon, date, interval=30, radius=10.0)
    t_verts = TSun.VerticesByDate(lat, lon, date, interval=30, radius=10.0)
    assert len(f_verts) == len(t_verts) and len(f_verts) > 2
    for fv, tv in zip(f_verts, t_verts):
        _approx_xyz(_coords(fv), _coords(tv))


@pytest.mark.parametrize("lat,lon", [LOCATIONS[0], LOCATIONS[3]])
def test_path_by_date_vertices(lat, lon):
    date = datetime(2024, 6, 21, 12, 0, 0)
    f_wire = FSun.PathByDate(lat, lon, date, interval=30, radius=10.0)
    t_wire = TSun.PathByDate(lat, lon, date, interval=30, radius=10.0)
    f_pts = sorted(_coords(v) for v in f_wire.Vertices())
    t_pts = sorted(_coords(v) for v in TTopology.Vertices(t_wire))
    assert len(f_pts) == len(t_pts) and len(f_pts) > 2
    for fp, tp in zip(f_pts, t_pts):
        _approx_xyz(fp, tp)


def test_vertices_by_hour():
    lat, lon = LOCATIONS[0]
    # Pin the same year for both so the comparison is deterministic.
    year = datetime.now().year
    f_verts = FSun.VerticesByHour(lat, lon, hour=12, startDay=1, endDay=365,
                                  interval=10, radius=10.0, year=year)
    t_verts = TSun.VerticesByHour(lat, lon, hour=12, startDay=1, endDay=365,
                                  interval=10, radius=10.0)
    assert len(f_verts) == len(t_verts) and len(f_verts) > 2
    for fv, tv in zip(f_verts, t_verts):
        _approx_xyz(_coords(fv), _coords(tv))


def test_path_by_hour_vertices():
    lat, lon = LOCATIONS[0]
    year = datetime.now().year
    f_wire = FSun.PathByHour(lat, lon, hour=12, startDay=1, endDay=365,
                             interval=10, radius=10.0, year=year)
    t_wire = TSun.PathByHour(lat, lon, hour=12, startDay=1, endDay=365,
                             interval=10, radius=10.0)
    f_pts = sorted(_coords(v) for v in f_wire.Vertices())
    t_pts = sorted(_coords(v) for v in TTopology.Vertices(t_wire))
    assert len(f_pts) == len(t_pts) and len(f_pts) > 2
    for fp, tp in zip(f_pts, t_pts):
        _approx_xyz(fp, tp)


def test_diagram_structure():
    lat, lon = LOCATIONS[0]
    year = 2024
    diagram = FSun.Diagram(lat, lon, minuteInterval=60, dayInterval=30,
                           uSides=60, vSides=60, year=year)
    assert set(["date_paths", "hourly_paths", "metadata"]).issubset(diagram.keys())
    # winter solstice, equinox, summer solstice
    assert len(diagram["date_paths"]) == 3
    assert all(w is not None for w in diagram["date_paths"])
    # up to 24 hourly analemmas
    assert len(diagram["hourly_paths"]) >= 1
    assert diagram["metadata"]["latitude"] == lat
