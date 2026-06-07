"""The parity case corpus.

Each case is ``fn(kernel) -> dict`` of invariants, written once against the
abstract kernel vocabulary and run through both kernels (see kernels.py). Cases
that probe a known, not-yet-implemented divergence are marked ``xfail`` with a
reason so the suite stays green while documenting the gap; when the gap is
closed the xfail flips to a pass (``strict=False`` keeps that non-fatal).

Add cases here as coverage grows — this corpus is the differential half of the
parity scoreboard.
"""
from __future__ import annotations


# --------------------------------------------------------------------------- #
# Primitive construction & measures (expected to MATCH today)                 #
# --------------------------------------------------------------------------- #
def vertex_roundtrip(k):
    v = k.vertex(1.5, -2.0, 3.25)
    return {"coords": k.coords(v)}


def edge_length(k):
    e = k.edge(k.vertex(0, 0, 0), k.vertex(3, 4, 0))
    return {"length": k.length(e)}


def box_invariants(k):
    b = k.box(0, 0, 0, 3, 4, 5)
    return {
        "volume": k.volume(b),
        "num_faces": len(k.faces(b)),
        "num_edges": len(k.edges(b)),
        "num_vertices": len(k.vertices(b)),
        "vertices": k.vertex_set(b),
        "com": k.center_of_mass(b),
    }


def box_face_areas(k):
    b = k.box(0, 0, 0, 2, 3, 4)
    return {"face_areas": sorted(round(k.area(f), 6) for f in k.faces(b))}


def translated_box_vertices(k):
    b = k.box(1, 2, 3, 1, 1, 1)
    return {"vertices": k.vertex_set(b), "volume": k.volume(b)}


# --------------------------------------------------------------------------- #
# Non-manifold construction (Phase 2 merge kernel — now expected to MATCH)    #
# --------------------------------------------------------------------------- #
def cellcomplex_shared_face(k):
    cc = k.cellcomplex([k.box(0, 0, 0, 2, 2, 2), k.box(0, 0, 2, 2, 2, 2)])
    return {
        "num_cells": len(k.cells(cc)),
        "num_faces": len(k.faces(cc)),
        "total_cell_volume": k.total_cell_volume(cc),
        "face_cell_counts": k.face_cell_counts(cc),
    }


def cellcomplex_translate(k):
    cc = k.cellcomplex([k.box(0, 0, 0, 2, 2, 2), k.box(0, 0, 2, 2, 2, 2)])
    moved = k.translate(cc, 10, -3, 2)
    return {
        "num_cells": len(k.cells(moved)),
        "num_faces": len(k.faces(moved)),
        "total_cell_volume": k.total_cell_volume(moved),
        "face_cell_counts": k.face_cell_counts(moved),
    }


def cellcomplex_3_stack(k):
    cc = k.cellcomplex([k.box(0, 0, i * 2, 2, 2, 2) for i in range(3)])
    return {
        "num_cells": len(k.cells(cc)),
        "num_faces": len(k.faces(cc)),
        "face_cell_counts": k.face_cell_counts(cc),
    }


# name, fn, xfail_reason (None = expected to match today)
CASES = [
    ("vertex_roundtrip", vertex_roundtrip, None),
    ("edge_length", edge_length, None),
    ("box_invariants", box_invariants, None),
    ("box_face_areas", box_face_areas, None),
    ("translated_box_vertices", translated_box_vertices, None),
    ("cellcomplex_shared_face", cellcomplex_shared_face, None),
    ("cellcomplex_3_stack", cellcomplex_3_stack, None),
    ("cellcomplex_translate", cellcomplex_translate, None),
]
