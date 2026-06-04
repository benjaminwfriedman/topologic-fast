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
    "clear_store",
    "store_stats",
]
