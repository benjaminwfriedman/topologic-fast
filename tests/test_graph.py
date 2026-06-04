"""Tests for Graph utility class - matching topologicpy test_13Graph.py"""
import pytest


def test_graph_by_vertices_edges():
    """Test Case 1 - Graph.ByVerticesEdges"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(0, 10, 0)
    v2 = tf.Vertex.ByCoordinates(10, 10, 0)
    v3 = tf.Vertex.ByCoordinates(10, 0, 0)
    v4 = tf.Vertex.ByCoordinates(0, 0, 10)

    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)
    e1 = tf.Edge.ByStartVertexEndVertex(v0, v2)
    e2 = tf.Edge.ByStartVertexEndVertex(v0, v3)
    e3 = tf.Edge.ByStartVertexEndVertex(v0, v4)

    graph = tf.Graph.ByVerticesEdges([v0, v1, v2, v3, v4], [e0, e1, e2, e3])
    assert graph is not None, "Should be Graph"


def test_graph_add_edge():
    """Test Case 2 - Graph.AddEdge"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(0, 10, 0)
    v2 = tf.Vertex.ByCoordinates(10, 10, 0)

    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)
    e1 = tf.Edge.ByStartVertexEndVertex(v1, v2)

    graph = tf.Graph.ByVerticesEdges([v0, v1, v2], [e0])
    assert graph.Size() == 1

    graph_ae = graph.AddEdge(e1)
    assert graph_ae.Size() == 2, "Should be 2"


def test_graph_add_vertex():
    """Test Case 3 - Graph.AddVertex"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(0, 10, 0)
    v5 = tf.Vertex.ByCoordinates(0, 10, 10)

    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)

    graph = tf.Graph.ByVerticesEdges([v0, v1], [e0])
    assert graph.Order() == 2

    graph_av = graph.AddVertex(v5)
    assert graph_av.Order() == 3, "Should be 3"


def test_graph_adjacency_list():
    """Test Case 5 - Graph.AdjacencyList"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(1, 0, 0)
    v2 = tf.Vertex.ByCoordinates(2, 0, 0)

    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)
    e1 = tf.Edge.ByStartVertexEndVertex(v1, v2)

    graph = tf.Graph.ByVerticesEdges([v0, v1, v2], [e0, e1])
    adj_list = graph.AdjacencyList()

    assert isinstance(adj_list, list), "Should be list"
    assert len(adj_list) == 3


def test_graph_adjacency_matrix():
    """Test Case 6 - Graph.AdjacencyMatrix"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(1, 0, 0)
    v2 = tf.Vertex.ByCoordinates(2, 0, 0)

    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)
    e1 = tf.Edge.ByStartVertexEndVertex(v1, v2)

    graph = tf.Graph.ByVerticesEdges([v0, v1, v2], [e0, e1])
    adj_matrix = graph.AdjacencyMatrix()

    assert isinstance(adj_matrix, list), "Should be list"
    assert len(adj_matrix) == 3
    assert len(adj_matrix[0]) == 3


def test_graph_adjacent_vertices():
    """Test Case 7 - Graph.AdjacentVertices"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(1, 0, 0)
    v2 = tf.Vertex.ByCoordinates(2, 0, 0)
    v3 = tf.Vertex.ByCoordinates(0, 1, 0)

    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)
    e1 = tf.Edge.ByStartVertexEndVertex(v0, v2)
    e2 = tf.Edge.ByStartVertexEndVertex(v0, v3)

    graph = tf.Graph.ByVerticesEdges([v0, v1, v2, v3], [e0, e1, e2])
    adj_v = graph.AdjacentVertices(v0)

    assert isinstance(adj_v, list), "Should be list"
    assert len(adj_v) == 3  # v0 connected to v1, v2, v3


def test_graph_contains_edge():
    """Test Case 11 - Graph.ContainsEdge"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(1, 0, 0)
    v2 = tf.Vertex.ByCoordinates(2, 0, 0)

    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)
    e1 = tf.Edge.ByStartVertexEndVertex(v1, v2)

    graph = tf.Graph.ByVerticesEdges([v0, v1, v2], [e0])

    assert graph.ContainsEdge(e0) == True, "Should contain e0"
    assert graph.ContainsEdge(e1) == False, "Should not contain e1"


def test_graph_contains_vertex():
    """Test Case 12 - Graph.ContainsVertex"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(1, 0, 0)
    v2 = tf.Vertex.ByCoordinates(2, 0, 0)

    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)

    graph = tf.Graph.ByVerticesEdges([v0, v1], [e0])

    assert graph.ContainsVertex(v0) == True, "Should contain v0"
    assert graph.ContainsVertex(v1) == True, "Should contain v1"
    assert graph.ContainsVertex(v2) == False, "Should not contain v2"


def test_graph_degree_sequence():
    """Test Case 13 - Graph.DegreeSequence"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(1, 0, 0)
    v2 = tf.Vertex.ByCoordinates(2, 0, 0)

    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)
    e1 = tf.Edge.ByStartVertexEndVertex(v1, v2)

    graph = tf.Graph.ByVerticesEdges([v0, v1, v2], [e0, e1])
    ds = graph.DegreeSequence()

    assert isinstance(ds, list), "Should be list"
    # v1 has degree 2, v0 and v2 have degree 1
    assert ds == [2, 1, 1]


def test_graph_density():
    """Test Case 14 - Graph.Density"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(1, 0, 0)
    v2 = tf.Vertex.ByCoordinates(2, 0, 0)

    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)
    e1 = tf.Edge.ByStartVertexEndVertex(v1, v2)

    graph = tf.Graph.ByVerticesEdges([v0, v1, v2], [e0, e1])
    density = graph.Density()

    assert isinstance(density, float), "Should be float"
    # 3 vertices means max 3 edges, we have 2
    expected = 2.0 / 3.0
    assert abs(density - expected) < 0.001


def test_graph_depth_map():
    """Test Case 15 - Graph.DepthMap"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(1, 0, 0)
    v2 = tf.Vertex.ByCoordinates(2, 0, 0)

    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)
    e1 = tf.Edge.ByStartVertexEndVertex(v1, v2)

    graph = tf.Graph.ByVerticesEdges([v0, v1, v2], [e0, e1])
    dm = graph.DepthMap(v0)

    assert isinstance(dm, list), "Should be list"
    assert dm[0] == 0  # Distance from v0 to v0
    assert dm[1] == 1  # Distance from v0 to v1
    assert dm[2] == 2  # Distance from v0 to v2


def test_graph_diameter():
    """Test Case 16 - Graph.Diameter"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(1, 0, 0)
    v2 = tf.Vertex.ByCoordinates(2, 0, 0)

    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)
    e1 = tf.Edge.ByStartVertexEndVertex(v1, v2)

    graph = tf.Graph.ByVerticesEdges([v0, v1, v2], [e0, e1])
    diameter = graph.Diameter()

    assert isinstance(diameter, int), "Should be int"
    assert diameter == 2  # Longest path is v0 -> v1 -> v2


def test_graph_distance():
    """Test Case 17 - Graph.Distance"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(1, 0, 0)
    v2 = tf.Vertex.ByCoordinates(2, 0, 0)

    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)
    e1 = tf.Edge.ByStartVertexEndVertex(v1, v2)

    graph = tf.Graph.ByVerticesEdges([v0, v1, v2], [e0, e1])
    dist = graph.Distance(v0, v2)

    assert dist == 2


def test_graph_edges():
    """Test Case 19 - Graph.Edges"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(1, 0, 0)
    v2 = tf.Vertex.ByCoordinates(2, 0, 0)

    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)
    e1 = tf.Edge.ByStartVertexEndVertex(v1, v2)

    graph = tf.Graph.ByVerticesEdges([v0, v1, v2], [e0, e1])
    edges = graph.Edges()

    assert isinstance(edges, list), "Should be list"
    assert len(edges) == 2


def test_graph_is_bipartite():
    """Test Case 20 - Graph.IsBipartite"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # A path graph is bipartite
    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(1, 0, 0)
    v2 = tf.Vertex.ByCoordinates(2, 0, 0)

    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)
    e1 = tf.Edge.ByStartVertexEndVertex(v1, v2)

    graph = tf.Graph.ByVerticesEdges([v0, v1, v2], [e0, e1])
    is_bp = graph.IsBipartite()

    assert isinstance(is_bp, bool), "Should be bool"
    assert is_bp == True, "Path graph should be bipartite"


def test_graph_is_complete():
    """Test Case 21 - Graph.IsComplete"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # A complete graph K3
    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(1, 0, 0)
    v2 = tf.Vertex.ByCoordinates(0, 1, 0)

    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)
    e1 = tf.Edge.ByStartVertexEndVertex(v1, v2)
    e2 = tf.Edge.ByStartVertexEndVertex(v2, v0)

    graph_complete = tf.Graph.ByVerticesEdges([v0, v1, v2], [e0, e1, e2])
    assert graph_complete.IsComplete() == True

    # Not complete
    graph_incomplete = tf.Graph.ByVerticesEdges([v0, v1, v2], [e0, e1])
    assert graph_incomplete.IsComplete() == False


def test_graph_isolated_vertices():
    """Test Case 23 - Graph.IsolatedVertices"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(1, 0, 0)
    v2 = tf.Vertex.ByCoordinates(2, 0, 0)  # Isolated

    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)

    graph = tf.Graph.ByVerticesEdges([v0, v1, v2], [e0])
    isolated = graph.IsolatedVertices()

    assert isinstance(isolated, list), "Should be list"
    assert len(isolated) == 1  # Only v2 is isolated


def test_graph_maximum_delta():
    """Test Case 24 - Graph.MaximumDelta"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(1, 0, 0)
    v2 = tf.Vertex.ByCoordinates(0, 1, 0)
    v3 = tf.Vertex.ByCoordinates(1, 1, 0)

    # Star graph with v0 at center
    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)
    e1 = tf.Edge.ByStartVertexEndVertex(v0, v2)
    e2 = tf.Edge.ByStartVertexEndVertex(v0, v3)

    graph = tf.Graph.ByVerticesEdges([v0, v1, v2, v3], [e0, e1, e2])
    max_d = graph.MaximumDelta()

    assert isinstance(max_d, int), "Should be integer"
    assert max_d == 3  # v0 has degree 3


def test_graph_minimum_delta():
    """Test Case 25 - Graph.MinimumDelta"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(1, 0, 0)
    v2 = tf.Vertex.ByCoordinates(0, 1, 0)
    v3 = tf.Vertex.ByCoordinates(1, 1, 0)

    # Star graph with v0 at center
    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)
    e1 = tf.Edge.ByStartVertexEndVertex(v0, v2)
    e2 = tf.Edge.ByStartVertexEndVertex(v0, v3)

    graph = tf.Graph.ByVerticesEdges([v0, v1, v2, v3], [e0, e1, e2])
    min_d = graph.MinimumDelta()

    assert isinstance(min_d, int), "Should be integer"
    assert min_d == 1  # v1, v2, v3 have degree 1


def test_graph_nearest_vertex():
    """Test Case 27 - Graph.NearestVertex"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(10, 0, 0)

    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)
    graph = tf.Graph.ByVerticesEdges([v0, v1], [e0])

    target = tf.Vertex.ByCoordinates(1, 0, 0)  # Closer to v0
    nearest = graph.NearestVertex(target)

    assert nearest is not None
    # Should be v0
    x, y, z = nearest.Coordinates()
    assert abs(x) < 0.001
    assert abs(y) < 0.001
    assert abs(z) < 0.001


def test_graph_order():
    """Test Case 28 - Graph.Order"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(1, 0, 0)
    v2 = tf.Vertex.ByCoordinates(2, 0, 0)

    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)

    graph = tf.Graph.ByVerticesEdges([v0, v1, v2], [e0])
    order = graph.Order()

    assert isinstance(order, int), "Should be integer"
    assert order == 3


def test_graph_path():
    """Test Case 29 - Graph.Path"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(1, 0, 0)
    v2 = tf.Vertex.ByCoordinates(2, 0, 0)

    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)
    e1 = tf.Edge.ByStartVertexEndVertex(v1, v2)

    graph = tf.Graph.ByVerticesEdges([v0, v1, v2], [e0, e1])
    path = graph.Path(v0, v2)

    assert path is not None, "Should find a path"


def test_graph_remove_edge():
    """Test Case 30 - Graph.RemoveEdge"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(1, 0, 0)
    v2 = tf.Vertex.ByCoordinates(2, 0, 0)

    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)
    e1 = tf.Edge.ByStartVertexEndVertex(v1, v2)

    graph = tf.Graph.ByVerticesEdges([v0, v1, v2], [e0, e1])
    assert graph.Size() == 2

    graph2 = graph.RemoveEdge(e0)
    assert graph2.Size() == 1


def test_graph_remove_vertex():
    """Test Case 31 - Graph.RemoveVertex"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(1, 0, 0)
    v2 = tf.Vertex.ByCoordinates(2, 0, 0)

    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)
    e1 = tf.Edge.ByStartVertexEndVertex(v1, v2)

    graph = tf.Graph.ByVerticesEdges([v0, v1, v2], [e0, e1])
    graph2 = graph.RemoveVertex(v1)

    # v1 was connected to both edges, so both edges should be removed
    assert graph2.Order() == 2
    assert graph2.Size() == 0  # No edges remain


def test_graph_size():
    """Test Case 34 - Graph.Size"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(1, 0, 0)
    v2 = tf.Vertex.ByCoordinates(2, 0, 0)

    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)
    e1 = tf.Edge.ByStartVertexEndVertex(v1, v2)

    graph = tf.Graph.ByVerticesEdges([v0, v1, v2], [e0, e1])
    size = graph.Size()

    assert isinstance(size, int), "Should be integer"
    assert size == 2


def test_graph_topology():
    """Test Case 36 - Graph.Topology"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(1, 0, 0)

    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)

    graph = tf.Graph.ByVerticesEdges([v0, v1], [e0])
    topo = graph.Topology()

    assert topo is not None, "Should return Cluster"


def test_graph_vertex_degree():
    """Test Case 38 - Graph.VertexDegree"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(1, 0, 0)
    v2 = tf.Vertex.ByCoordinates(0, 1, 0)
    v3 = tf.Vertex.ByCoordinates(1, 1, 0)

    # Star graph with v0 at center
    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)
    e1 = tf.Edge.ByStartVertexEndVertex(v0, v2)
    e2 = tf.Edge.ByStartVertexEndVertex(v0, v3)

    graph = tf.Graph.ByVerticesEdges([v0, v1, v2, v3], [e0, e1, e2])

    assert graph.VertexDegree(v0) == 3
    assert graph.VertexDegree(v1) == 1


def test_graph_vertices():
    """Test Case 39 - Graph.Vertices"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    v0 = tf.Vertex.ByCoordinates(0, 0, 0)
    v1 = tf.Vertex.ByCoordinates(1, 0, 0)
    v2 = tf.Vertex.ByCoordinates(2, 0, 0)

    e0 = tf.Edge.ByStartVertexEndVertex(v0, v1)

    graph = tf.Graph.ByVerticesEdges([v0, v1, v2], [e0])
    vertices = graph.Vertices()

    assert isinstance(vertices, list), "Should be list"
    assert len(vertices) == 3


def test_graph_by_topology_cell_complex():
    """Test Graph.ByTopology with CellComplex - creates dual graph"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create two adjacent cells sharing a face
    c1 = tf.Cell.Box(0, 0, 0, 1, 1, 1)
    c2 = tf.Cell.Box(1, 0, 0, 1, 1, 1)
    cc = tf.CellComplex.ByCells([c1, c2])

    graph = tf.Graph.ByTopology(cc)

    # Should have 2 vertices (one per cell) and 1 edge (shared face)
    assert graph.Order() == 2, f"Expected 2 vertices, got {graph.Order()}"
    assert graph.Size() == 1, f"Expected 1 edge, got {graph.Size()}"


def test_graph_by_topology_shell():
    """Test Graph.ByTopology with Shell - creates dual graph of faces"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create a shell from a box's faces
    cell = tf.Cell.Box(0, 0, 0, 1, 1, 1)
    faces = cell.Faces()
    shell = tf.Shell.ByFaces(faces)

    graph = tf.Graph.ByTopology(shell)

    # A cube has 6 faces, each sharing edges with 4 others
    # Total edges = (6 * 4) / 2 = 12
    assert graph.Order() == 6, f"Expected 6 vertices, got {graph.Order()}"
    assert graph.Size() == 12, f"Expected 12 edges, got {graph.Size()}"


def test_graph_by_topology_wire():
    """Test Graph.ByTopology with Wire - creates dual graph of edges"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    wire = tf.Wire.Rectangle(width=2.0, length=2.0)

    graph = tf.Graph.ByTopology(wire)

    # Rectangle has 4 edges, each connected to 2 neighbors
    assert graph.Order() == 4, f"Expected 4 vertices, got {graph.Order()}"
    assert graph.Size() == 4, f"Expected 4 edges, got {graph.Size()}"


def test_graph_by_topology_single_cell():
    """Test Graph.ByTopology with single Cell - creates single vertex"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    cell = tf.Cell.Box(0, 0, 0, 1, 1, 1)

    graph = tf.Graph.ByTopology(cell)

    # Single cell = single vertex, no edges
    assert graph.Order() == 1, f"Expected 1 vertex, got {graph.Order()}"
    assert graph.Size() == 0, f"Expected 0 edges, got {graph.Size()}"


def test_graph_by_topology_single_face():
    """Test Graph.ByTopology with single Face - creates single vertex"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    face = tf.Face.Rectangle(width=2.0, length=3.0)

    graph = tf.Graph.ByTopology(face)

    # Single face = single vertex, no edges
    assert graph.Order() == 1, f"Expected 1 vertex, got {graph.Order()}"
    assert graph.Size() == 0, f"Expected 0 edges, got {graph.Size()}"


def test_graph_by_topology_direct_false():
    """Test Graph.ByTopology with direct=False - no edges created"""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    # Create two adjacent cells
    c1 = tf.Cell.Box(0, 0, 0, 1, 1, 1)
    c2 = tf.Cell.Box(1, 0, 0, 1, 1, 1)
    cc = tf.CellComplex.ByCells([c1, c2])

    graph = tf.Graph.ByTopology(cc, direct=False)

    # With direct=False, no edges should be created
    assert graph.Order() == 2, f"Expected 2 vertices, got {graph.Order()}"
    assert graph.Size() == 0, f"Expected 0 edges, got {graph.Size()}"
