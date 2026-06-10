"""Differential parity tests: topologic_fast vs the reference topologic_core.

Each case in ``cases.CASES`` is run through both kernels and the resulting
invariant dicts are compared within tolerance. Cases tagged with an xfail reason
probe a known, not-yet-implemented divergence (e.g. the non-manifold merge
kernel); they are expected to differ today and will flip to passing once the gap
is closed.

Skipped entirely if ``topologicpy`` (the reference kernel) is not installed.
"""
import pytest

pytest.importorskip("topologic_fast")
pytest.importorskip("topologicpy")

from .kernels import CoreKernel, FastKernel  # noqa: E402
from .compare import compare  # noqa: E402
from .cases import CASES  # noqa: E402


def _params():
    params = []
    for name, fn, xfail_reason in CASES:
        marks = []
        if xfail_reason:
            marks.append(pytest.mark.xfail(reason=xfail_reason, strict=False))
        params.append(pytest.param(fn, id=name, marks=marks))
    return params


@pytest.mark.parametrize("case_fn", _params())
def test_parity(case_fn):
    core = case_fn(CoreKernel())
    fast = case_fn(FastKernel())
    ok, diffs = compare(core, fast)
    assert ok, "kernel divergence:\n  " + "\n  ".join(diffs)
