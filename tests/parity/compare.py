"""Tolerant comparison of invariant dictionaries produced by two kernels."""
from __future__ import annotations

import math

TOL = 1e-6


def _eq(a, b, tol=TOL):
    if isinstance(a, bool) or isinstance(b, bool):
        return a == b
    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
        return math.isclose(a, b, rel_tol=0.0, abs_tol=tol) or abs(a - b) <= tol
    if isinstance(a, (list, tuple)) and isinstance(b, (list, tuple)):
        return len(a) == len(b) and all(_eq(x, y, tol) for x, y in zip(a, b))
    return a == b


def compare(core_result, fast_result, tol=TOL):
    """Compare two invariant dicts. Returns (ok, list_of_diff_strings)."""
    diffs = []
    keys = set(core_result) | set(fast_result)
    for k in sorted(keys):
        if k not in core_result:
            diffs.append(f"{k}: missing in core")
            continue
        if k not in fast_result:
            diffs.append(f"{k}: missing in fast")
            continue
        a, b = core_result[k], fast_result[k]
        if not _eq(a, b, tol):
            diffs.append(f"{k}: core={a!r} fast={b!r}")
    return (len(diffs) == 0, diffs)
