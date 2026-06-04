#!/usr/bin/env python3
"""
Benchmark comparison between topologic-fast (Rust) and topologicpy (C++)
"""

import time
import statistics
from typing import Callable, List, Tuple

# Import both libraries
try:
    import topologic_fast as tf
    HAS_FAST = True
except ImportError:
    HAS_FAST = False
    print("Warning: topologic_fast not available")

try:
    from topologicpy.Vertex import Vertex as TPVertex
    from topologicpy.Edge import Edge as TPEdge
    from topologicpy.Wire import Wire as TPWire
    from topologicpy.Face import Face as TPFace
    from topologicpy.Shell import Shell as TPShell
    from topologicpy.Cell import Cell as TPCell
    from topologicpy.CellComplex import CellComplex as TPCellComplex
    from topologicpy.Topology import Topology as TPTopology
    HAS_TOPOLOGICPY = True
except ImportError:
    HAS_TOPOLOGICPY = False
    print("Warning: topologicpy not available")


def benchmark(func: Callable, iterations: int = 100, warmup: int = 5) -> Tuple[float, float, float]:
    """Run benchmark and return (mean, std, min) times in milliseconds."""
    # Warmup
    for _ in range(warmup):
        func()

    # Actual benchmark
    times = []
    for _ in range(iterations):
        start = time.perf_counter()
        func()
        end = time.perf_counter()
        times.append((end - start) * 1000)  # Convert to ms

    return statistics.mean(times), statistics.stdev(times) if len(times) > 1 else 0, min(times)


def print_result(name: str, fast_time: Tuple[float, float, float], py_time: Tuple[float, float, float]):
    """Print benchmark result with speedup."""
    fast_mean, fast_std, fast_min = fast_time
    py_mean, py_std, py_min = py_time

    speedup = py_mean / fast_mean if fast_mean > 0 else float('inf')

    print(f"\n{name}:")
    print(f"  topologic-fast: {fast_mean:.4f}ms ± {fast_std:.4f}ms (min: {fast_min:.4f}ms)")
    print(f"  topologicpy:    {py_mean:.4f}ms ± {py_std:.4f}ms (min: {py_min:.4f}ms)")
    print(f"  Speedup:        {speedup:.1f}x faster")

    return speedup


# ============== Benchmark Functions ==============

def bench_vertex_creation_fast():
    """Create 1000 vertices (topologic-fast)."""
    for i in range(1000):
        tf.Vertex.ByCoordinates(float(i), float(i), float(i))

def bench_vertex_creation_py():
    """Create 1000 vertices (topologicpy)."""
    for i in range(1000):
        TPVertex.ByCoordinates(float(i), float(i), float(i))


def bench_edge_creation_fast():
    """Create 500 edges (topologic-fast)."""
    for i in range(500):
        v1 = tf.Vertex.ByCoordinates(float(i), 0.0, 0.0)
        v2 = tf.Vertex.ByCoordinates(float(i+1), 0.0, 0.0)
        tf.Edge.ByStartVertexEndVertex(v1, v2)

def bench_edge_creation_py():
    """Create 500 edges (topologicpy)."""
    for i in range(500):
        v1 = TPVertex.ByCoordinates(float(i), 0.0, 0.0)
        v2 = TPVertex.ByCoordinates(float(i+1), 0.0, 0.0)
        TPEdge.ByStartVertexEndVertex(v1, v2)


def bench_rectangle_creation_fast():
    """Create 100 rectangles (topologic-fast)."""
    for i in range(100):
        tf.Face.Rectangle(float(i), 0.0, 0.0, 2.0, 3.0)

def bench_rectangle_creation_py():
    """Create 100 rectangles (topologicpy)."""
    for i in range(100):
        TPFace.Rectangle(width=2.0, length=3.0)


def bench_box_creation_fast():
    """Create 50 box cells (topologic-fast)."""
    for i in range(50):
        tf.Cell.Box(float(i), 0.0, 0.0, 2.0, 2.0, 2.0)

def bench_box_creation_py():
    """Create 50 box cells (topologicpy)."""
    for i in range(50):
        TPCell.Box(width=2.0, length=2.0, height=2.0)


def bench_volume_calculation_fast():
    """Calculate volume 100 times (topologic-fast)."""
    cell = tf.Cell.Box(0.0, 0.0, 0.0, 2.0, 2.0, 2.0)
    for _ in range(100):
        cell.Volume()

def bench_volume_calculation_py():
    """Calculate volume 100 times (topologicpy)."""
    cell = TPCell.Box(width=2.0, length=2.0, height=2.0)
    for _ in range(100):
        TPCell.Volume(cell)


def bench_area_calculation_fast():
    """Calculate area 100 times (topologic-fast)."""
    cell = tf.Cell.Box(0.0, 0.0, 0.0, 2.0, 2.0, 2.0)
    for _ in range(100):
        cell.Area()

def bench_area_calculation_py():
    """Calculate area 100 times (topologicpy)."""
    cell = TPCell.Box(width=2.0, length=2.0, height=2.0)
    for _ in range(100):
        TPCell.Area(cell)


def bench_vertices_query_fast():
    """Query vertices from box 100 times (topologic-fast)."""
    cell = tf.Cell.Box(0.0, 0.0, 0.0, 2.0, 2.0, 2.0)
    for _ in range(100):
        cell.Vertices()

def bench_vertices_query_py():
    """Query vertices from box 100 times (topologicpy)."""
    cell = TPCell.Box(width=2.0, length=2.0, height=2.0)
    for _ in range(100):
        TPTopology.Vertices(cell)


def bench_edges_query_fast():
    """Query edges from box 100 times (topologic-fast)."""
    cell = tf.Cell.Box(0.0, 0.0, 0.0, 2.0, 2.0, 2.0)
    for _ in range(100):
        cell.Edges()

def bench_edges_query_py():
    """Query edges from box 100 times (topologicpy)."""
    cell = TPCell.Box(width=2.0, length=2.0, height=2.0)
    for _ in range(100):
        TPTopology.Edges(cell)


def bench_faces_query_fast():
    """Query faces from box 100 times (topologic-fast)."""
    cell = tf.Cell.Box(0.0, 0.0, 0.0, 2.0, 2.0, 2.0)
    for _ in range(100):
        cell.Faces()

def bench_faces_query_py():
    """Query faces from box 100 times (topologicpy)."""
    cell = TPCell.Box(width=2.0, length=2.0, height=2.0)
    for _ in range(100):
        TPTopology.Faces(cell)


def bench_prism_creation_fast():
    """Create 20 prisms (topologic-fast)."""
    for i in range(20):
        face = tf.Face.Rectangle(float(i), 0.0, 0.0, 2.0, 2.0)
        tf.Cell.Prism(face, 3.0)

def bench_prism_creation_py():
    """Create 20 prisms (topologicpy)."""
    for i in range(20):
        face = TPFace.Rectangle(width=2.0, length=2.0)
        TPCell.Prism(face, 3.0)


def bench_cylinder_creation_fast():
    """Create 10 cylinders (topologic-fast)."""
    for i in range(10):
        tf.Cell.Cylinder(float(i), 0.0, 0.0, 1.0, 2.0, 32)

def bench_cylinder_creation_py():
    """Create 10 cylinders (topologicpy)."""
    for i in range(10):
        TPCell.Cylinder(radius=1.0, height=2.0, uSides=32)


def bench_boolean_union_fast():
    """Boolean union 10 times (topologic-fast)."""
    for i in range(10):
        c1 = tf.Cell.Box(0.0, 0.0, 0.0, 2.0, 2.0, 2.0)
        c2 = tf.Cell.Box(1.0, 0.0, 0.0, 2.0, 2.0, 2.0)
        tf.Topology.Union(c1, c2)

def bench_boolean_union_py():
    """Boolean union 10 times (topologicpy)."""
    for i in range(10):
        c1 = TPCell.Box(width=2.0, length=2.0, height=2.0)
        c2 = TPCell.Box(origin=TPVertex.ByCoordinates(1.0, 0.0, 0.0), width=2.0, length=2.0, height=2.0)
        TPTopology.Union(c1, c2)


def bench_boolean_intersection_fast():
    """Boolean intersection 10 times (topologic-fast)."""
    for i in range(10):
        c1 = tf.Cell.Box(0.0, 0.0, 0.0, 2.0, 2.0, 2.0)
        c2 = tf.Cell.Box(1.0, 0.0, 0.0, 2.0, 2.0, 2.0)
        tf.Topology.Intersection(c1, c2)

def bench_boolean_intersection_py():
    """Boolean intersection 10 times (topologicpy)."""
    for i in range(10):
        c1 = TPCell.Box(width=2.0, length=2.0, height=2.0)
        c2 = TPCell.Box(origin=TPVertex.ByCoordinates(1.0, 0.0, 0.0), width=2.0, length=2.0, height=2.0)
        TPTopology.Intersect(c1, c2)


def bench_wire_circle_fast():
    """Create 50 circle wires (topologic-fast)."""
    for i in range(50):
        v = tf.Vertex.ByCoordinates(0.0, 0.0, 0.0)
        tf.Wire.Circle(origin=v, radius=1.0, sides=64)

def bench_wire_circle_py():
    """Create 50 circle wires (topologicpy)."""
    for i in range(50):
        TPWire.Circle(radius=1.0, sides=64)


def bench_large_mesh_fast():
    """Create 10 spheres with many faces (topologic-fast)."""
    for i in range(10):
        tf.Cell.Sphere(float(i)*3, 0.0, 0.0, 1.0, 16, 8)

def bench_large_mesh_py():
    """Create 10 spheres with many faces (topologicpy)."""
    for i in range(10):
        TPCell.Sphere(radius=1.0, uSides=16, vSides=8)


def main():
    if not HAS_FAST:
        print("Error: topologic-fast not available")
        return

    if not HAS_TOPOLOGICPY:
        print("Error: topologicpy not available")
        return

    print("=" * 60)
    print("Benchmark: topologic-fast (Rust) vs topologicpy (C++)")
    print("=" * 60)

    speedups = []

    # Run benchmarks
    benchmarks = [
        ("Vertex Creation (1000x)", bench_vertex_creation_fast, bench_vertex_creation_py, 50),
        ("Edge Creation (500x)", bench_edge_creation_fast, bench_edge_creation_py, 50),
        ("Rectangle Creation (100x)", bench_rectangle_creation_fast, bench_rectangle_creation_py, 50),
        ("Box Creation (50x)", bench_box_creation_fast, bench_box_creation_py, 50),
        ("Volume Calculation (100x)", bench_volume_calculation_fast, bench_volume_calculation_py, 100),
        ("Area Calculation (100x)", bench_area_calculation_fast, bench_area_calculation_py, 100),
        ("Vertices Query (100x)", bench_vertices_query_fast, bench_vertices_query_py, 100),
        ("Edges Query (100x)", bench_edges_query_fast, bench_edges_query_py, 100),
        ("Faces Query (100x)", bench_faces_query_fast, bench_faces_query_py, 100),
        ("Prism Creation (20x)", bench_prism_creation_fast, bench_prism_creation_py, 30),
        ("Cylinder Creation (10x)", bench_cylinder_creation_fast, bench_cylinder_creation_py, 30),
        ("Wire Circle (50x)", bench_wire_circle_fast, bench_wire_circle_py, 50),
        ("Sphere Creation (10x)", bench_large_mesh_fast, bench_large_mesh_py, 20),
        ("Boolean Union (10x)", bench_boolean_union_fast, bench_boolean_union_py, 20),
        ("Boolean Intersection (10x)", bench_boolean_intersection_fast, bench_boolean_intersection_py, 20),
    ]

    for name, fast_func, py_func, iterations in benchmarks:
        try:
            fast_result = benchmark(fast_func, iterations)
            py_result = benchmark(py_func, iterations)
            speedup = print_result(name, fast_result, py_result)
            speedups.append((name, speedup))
        except Exception as e:
            print(f"\n{name}: Error - {e}")

    # Summary
    print("\n" + "=" * 60)
    print("SUMMARY")
    print("=" * 60)

    if speedups:
        avg_speedup = statistics.mean([s[1] for s in speedups])
        min_speedup = min(speedups, key=lambda x: x[1])
        max_speedup = max(speedups, key=lambda x: x[1])

        print(f"\nAverage Speedup: {avg_speedup:.1f}x faster")
        print(f"Minimum Speedup: {min_speedup[1]:.1f}x ({min_speedup[0]})")
        print(f"Maximum Speedup: {max_speedup[1]:.1f}x ({max_speedup[0]})")

        print("\n" + "-" * 60)
        print("All Results (sorted by speedup):")
        print("-" * 60)
        for name, speedup in sorted(speedups, key=lambda x: x[1], reverse=True):
            bar = "█" * int(speedup / 2) if speedup < 100 else "█" * 50 + "..."
            print(f"  {speedup:6.1f}x  {bar}  {name}")


if __name__ == "__main__":
    main()
