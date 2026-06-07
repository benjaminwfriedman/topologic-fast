"""Parity tests for topologic_fast.Energy against topologicpy.EnergyModel.

For the same building geometry, ``topologic_fast.Energy.ByTopology`` must build an
OpenStudio model equivalent to the one ``topologicpy.EnergyModel.ByTopology``
builds. We compare order-invariant model aggregates (space / thermal-zone /
surface / sub-surface counts, surface types, gross areas, boundary conditions
and total zone volume) so the comparison does not depend on the order in which
each library happens to enumerate faces.

Both libraries are driven with the *same* bundled OSM template / EPW / DDY
assets so any difference can only come from the geometry pipeline.

These tests require the ``openstudio`` Python bindings to actually work. Their
wheels import but crash on object construction under CPython >= 3.13, so the
whole module is skipped unless a subprocess can construct an ``openstudio``
object (i.e. it is running on a supported interpreter such as 3.12) and
``topologicpy`` is installed.
"""
import os
import subprocess
import sys
from collections import Counter

import pytest

tf = pytest.importorskip("topologic_fast")
pytest.importorskip("openstudio")
pytest.importorskip("topologicpy")


def _openstudio_usable():
    """True if openstudio can construct an object without crashing the process."""
    try:
        r = subprocess.run(
            [sys.executable, "-c", "import openstudio; openstudio.Point3d(0, 0, 0)"],
            capture_output=True, timeout=120,
        )
        return r.returncode == 0
    except Exception:
        return False


pytestmark = pytest.mark.skipif(
    not _openstudio_usable(),
    reason="openstudio bindings cannot construct objects on this interpreter (need CPython <= 3.12)",
)

from topologic_fast import Energy as FEnergy  # noqa: E402
from topologicpy.Cell import Cell as TCell  # noqa: E402
from topologicpy.CellComplex import CellComplex as TCellComplex  # noqa: E402
from topologicpy.Topology import Topology as TTopology  # noqa: E402
from topologicpy.EnergyModel import EnergyModel  # noqa: E402

_ASSETS = os.path.join(os.path.dirname(tf.__file__), "assets", "EnergyModel")
_KW = dict(
    osModelPath=os.path.join(_ASSETS, "OSMTemplate-OfficeBuilding-3.10.0.osm"),
    weatherFilePath=os.path.join(_ASSETS, "GBR_London.Gatwick.037760_IWEC.epw"),
    designDayFilePath=os.path.join(_ASSETS, "GBR_London.Gatwick.037760_IWEC.ddy"),
)


# --------------------------------------------------------------------------- #
# Geometry factories: build the SAME box / complex in each library            #
# --------------------------------------------------------------------------- #
def _tf_box(w, l, h, z0=0.0):
    return tf.Cell.Box(0, 0, z0, w, l, h)


def _tpy_box(w, l, h, z0=0.0):
    # topologicpy's Prism is centred on the origin; translate so it spans
    # [0, w] x [0, l] x [z0, z0 + h] to match tf.Cell.Box.
    prism = TCell.Prism(width=w, length=l, height=h)
    return TTopology.Translate(prism, w / 2.0, l / 2.0, z0 + h / 2.0)


def _tf_stacked(w, l, h, n):
    return tf.CellComplex.ByCells([_tf_box(w, l, h, z0=i * h) for i in range(n)])


def _tpy_stacked(w, l, h, n):
    return TCellComplex.ByCells([_tpy_box(w, l, h, z0=i * h) for i in range(n)])


# --------------------------------------------------------------------------- #
# Aggregate extraction (order-invariant)                                      #
# --------------------------------------------------------------------------- #
def _aggregates(model):
    surfaces = model.getSurfaces()
    areas = {}
    for s in surfaces:
        areas[s.surfaceType()] = round(areas.get(s.surfaceType(), 0.0) + s.grossArea(), 3)
    sub = model.getSubSurfaces()
    sub_area = round(sum(ss.grossArea() for ss in sub), 3)
    zone_volume = round(
        sum(z.volume().get() for z in model.getThermalZones() if z.volume().is_initialized()), 3
    )
    return {
        "spaces": len(model.getSpaces()),
        "zones": len(model.getThermalZones()),
        "surfaces": len(surfaces),
        "surface_types": dict(Counter(s.surfaceType() for s in surfaces)),
        "areas": areas,
        "boundary_conditions": dict(Counter(s.outsideBoundaryCondition() for s in surfaces)),
        "subsurfaces": len(sub),
        "subsurface_types": dict(Counter(ss.subSurfaceType() for ss in sub)),
        "subsurface_area": sub_area,
        "zone_volume": zone_volume,
    }


# Each case: (label, tf_building_factory, tpy_building_factory, kwargs)
CASES = [
    ("box_10x10x3", lambda: _tf_box(10, 10, 3), lambda: _tpy_box(10, 10, 3), {}),
    ("box_6x8x4", lambda: _tf_box(6, 8, 4), lambda: _tpy_box(6, 8, 4), {}),
    ("box_glazing_0.4", lambda: _tf_box(10, 10, 3), lambda: _tpy_box(10, 10, 3),
     {"glazingRatio": 0.4}),
    ("box_glazing_0.25", lambda: _tf_box(12, 9, 3.5), lambda: _tpy_box(12, 9, 3.5),
     {"glazingRatio": 0.25}),
    ("stacked_2", lambda: _tf_stacked(10, 10, 3, 2), lambda: _tpy_stacked(10, 10, 3, 2), {}),
    ("stacked_3", lambda: _tf_stacked(8, 8, 3, 3), lambda: _tpy_stacked(8, 8, 3, 3), {}),
]


@pytest.mark.parametrize("label,f_factory,t_factory,kwargs",
                         CASES, ids=[c[0] for c in CASES])
def test_energy_model_parity(label, f_factory, t_factory, kwargs):
    f_model = FEnergy.ByTopology(f_factory(), **_KW, **kwargs)
    t_model = EnergyModel.ByTopology(t_factory(), **_KW, **kwargs)
    assert f_model is not None and t_model is not None
    assert _aggregates(f_model) == _aggregates(t_model)


def test_single_box_expected_classification():
    """Sanity-check absolute (not just relative) results for a known box."""
    model = FEnergy.ByTopology(_tf_box(10, 10, 3), **_KW)
    agg = _aggregates(model)
    assert agg["spaces"] == 1 and agg["zones"] == 1
    assert agg["surface_types"] == {"Wall": 4, "RoofCeiling": 1, "Floor": 1}
    assert agg["areas"] == {"Wall": 120.0, "RoofCeiling": 100.0, "Floor": 100.0}
    assert agg["boundary_conditions"] == {"Outdoors": 5, "Ground": 1}
    assert agg["zone_volume"] == 300.0


def test_interior_surfaces_matched():
    """A 2-storey stack must produce two matched interior surfaces."""
    model = FEnergy.ByTopology(_tf_stacked(10, 10, 3, 2), **_KW)
    agg = _aggregates(model)
    assert agg["spaces"] == 2 and agg["zones"] == 2
    # 2 shared faces become interior surfaces with "Surface" boundary condition.
    assert agg["boundary_conditions"].get("Surface") == 2


def test_glazing_creates_windows():
    model = FEnergy.ByTopology(_tf_box(10, 10, 3), glazingRatio=0.4, **_KW)
    agg = _aggregates(model)
    assert agg["subsurfaces"] == 4
    assert agg["subsurface_types"] == {"FixedWindow": 4}
    # 40% of total wall area (120) glazed.
    assert agg["subsurface_area"] == pytest.approx(0.4 * 120.0, rel=1e-3)


def test_space_type_names_match_template():
    """Both libraries read the same space-type list from the shared template."""
    import openstudio
    translator = openstudio.osversion.VersionTranslator()
    model = translator.loadModel(openstudio.path(_KW["osModelPath"])).get()
    f_names = sorted(FEnergy.SpaceTypeNames(model))
    t_names = sorted(EnergyModel.SpaceTypeNames(model))
    assert f_names == t_names and len(f_names) > 0


def test_export_to_osm_roundtrip(tmp_path):
    import openstudio
    model = FEnergy.ByTopology(_tf_box(10, 10, 3), **_KW)
    out = os.path.join(str(tmp_path), "model.osm")
    assert FEnergy.ExportToOSM(model, out, overwrite=True)
    assert os.path.exists(out)
    reloaded = openstudio.osversion.VersionTranslator().loadModel(openstudio.path(out))
    assert reloaded.is_initialized()
    assert len(reloaded.get().getSpaces()) == 1


def test_query_without_sql_returns_none():
    """Query is graceful when no simulation has been run (no SQL attached)."""
    model = FEnergy.ByTopology(_tf_box(10, 10, 3), **_KW)
    with pytest.warns(UserWarning):
        assert FEnergy.Query(model) is None
