# Changelog

## 0.2.1

### Fixed
- **`Graph.ByTopology` is now a true topologicpy drop-in.** It previously used a
  truncated snake_case signature (`direct`, `via_shared_topologies`,
  `to_exterior_topologies`, `tolerance`), so topologicpy-style calls passing
  `directApertures=`, `viaSharedTopologies=`, `useInternalVertex=`, etc. raised
  `TypeError`. The full topologicpy keyword signature is now accepted.
- **Apertures can drive connectivity.** For a `CellComplex`, `directApertures`
  connects only cells that share a face carrying an aperture (the *access*
  graph), distinct from `direct`, which connects cells sharing *any* face (the
  *adjacency* graph). `viaSharedTopologies`/`viaSharedApertures`/
  `toExteriorTopologies`/`toExteriorApertures` are likewise honored.
- Each cell's dictionary is transferred onto its representative graph vertex
  (with a stable `index` key), matching topologicpy.

`useInternalVertex` is accepted for compatibility; tf has no internal-vertex
primitive, so the cell centroid is used as the representative point (interior
for convex cells; graph structure is unaffected).

## 0.2.0

A large compatibility + correctness release. topologic_fast now mirrors much of
topologicpy's API (names, signatures, and import layout) and runs a substantial,
verified subset many times faster.

### Added
- **`Sun`** module (solar geometry; `ephem`-based, bit-exact astronomy).
- **`Energy`** module (OpenStudio `EnergyModel` port; requires `openstudio`,
  CPython ≤ 3.12). Bundles OSM/EPW/DDY assets.
- **topologicpy-compatible submodule layout** — `from topologic_fast.Vertex import
  Vertex` now works (one submodule per class), alongside flat `tf.Vertex`. An
  `EnergyModel` alias maps to `tf.Energy`. Migration is a search-and-replace of
  the import root.
- **~100 topologicpy-compatible methods** across `Vertex`/`Edge`/`Wire`/`Face`/
  `Shell`/`Cell`/`CellComplex`/`Topology`/`Vector`/`Dictionary`/`Matrix`/`Graph`,
  each verified against topologicpy. Hot methods are pure Rust (2–280× faster);
  both `Class.Method(obj)` and `obj.Method()` call styles work.
- `Topology.AddApertures`/`Apertures` and `Topology.UUID` (stable entity id).
- `TopologicFastBackend` — run topologicpy's own layer on the fast kernel via
  `Core.SetBackend(...)`.
- Parity tooling: `PARITY.md`/`ROADMAP.md` generators and a differential test
  harness.

### Fixed
- **Non-manifold kernel:** `CellComplex.ByCells` now merges coincident faces into
  a single shared non-manifold face and sews coincident vertices.
- `Shell::volume` made robust to inconsistent face winding.
- `Face.Compactness` and `Graph.ClosenessCentrality` corrected to match
  topologicpy (previously diverged).

### Changed (potentially breaking)
- `Vertex.Coordinates()` now returns a **list** (was a tuple).
- `Cell.Volume`/`Area`, `Edge.Length`, `Face.Area`, `Wire.Length`,
  `CellComplex.Volume`/`Area` now **round to `mantissa=6` by default** (was raw
  precision); pass `mantissa=None` for raw values.
- `CellComplex.ByCells([a, b])` of two adjacent cells now yields **11 faces**
  (shared face merged), not 12.

## 0.1.0

Initial release.
