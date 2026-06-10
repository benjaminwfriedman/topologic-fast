"""
topologic-fast: High-performance non-manifold topology library.

This package provides Python bindings for the topologic-fast Rust library,
which is a high-performance implementation of non-manifold topology operations
for architectural and engineering applications.

Example:
    >>> import topologic_fast as tf
    >>> box = tf.Cell.Box(0, 0, 0, 2, 2, 2)
    >>> print(f"Volume: {box.Volume()}")
    Volume: 8.0
"""

# The actual implementation comes from the Rust extension
# This file is only for type hints and documentation
from .topologic_fast import *

# Pure-Python modules built on top of the Rust extension.
from .Sun import Sun
from .Energy import Energy
from .backend import TopologicFastBackend

# Attach pure-Python parity helpers (topologicpy-compatible static methods) onto
# the Rust topology classes.
from . import _pyhelpers as _pyhelpers
from . import _native_api as _native_api
import sys as _sys
_pyhelpers.install(_sys.modules[__name__])
# Lean, fast, topologicpy-compatible static methods (the performance path).
_native_api.install(_sys.modules[__name__])

# --- topologicpy-compatible submodule layout -------------------------------
# topologicpy code imports classes as ``from topologicpy.Vertex import Vertex``
# (one submodule per class). topologic_fast exposes flat classes (``tf.Vertex``).
# To make migration a search-and-replace of ``topologicpy`` -> ``topologic_fast``,
# we register each class as its own submodule in ``sys.modules`` with a
# self-reference, so BOTH styles work:
#     from topologic_fast.Vertex import Vertex      # topologicpy style
#     import topologic_fast as tf; tf.Vertex.Box(..)  # flat style (unchanged)
from . import topologic_fast as _ext  # the Rust extension module

_CLASS_SUBMODULES = [
    "Vertex", "Edge", "Wire", "Face", "Shell", "Cell", "CellComplex", "Cluster",
    "Topology", "Dictionary", "Graph", "Vector", "Matrix", "Grid", "Mesh",
]
for _name in _CLASS_SUBMODULES:
    _cls = getattr(_ext, _name, None)
    if _cls is None:
        continue
    try:
        # Self-reference so ``from topologic_fast.Vertex import Vertex`` resolves
        # (mirrors topologicpy, whose Vertex module also exposes a Vertex class).
        setattr(_cls, _name, _cls)
    except Exception:
        pass
    _sys.modules["{}.{}".format(__name__, _name)] = _cls

# topologicpy names the energy module ``EnergyModel``; expose that alias too.
try:
    Energy.EnergyModel = Energy
    _sys.modules["{}.EnergyModel".format(__name__)] = Energy
except Exception:
    pass

__version__ = "0.1.0"
__all__ = [
    "Vertex",
    "Edge",
    "Wire",
    "Face",
    "Shell",
    "Cell",
    "CellComplex",
    "Topology",
    "Mesh",
    "Sun",
    "Energy",
    "TopologicFastBackend",
    "clear_store",
    "store_stats",
]
