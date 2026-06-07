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
