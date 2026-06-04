"""Pytest configuration for topologic-fast tests."""
import pytest


def pytest_configure(config):
    """Configure pytest."""
    config.addinivalue_line(
        "markers", "slow: marks tests as slow (deselect with '-m \"not slow\"')"
    )


@pytest.fixture(autouse=True)
def clear_store():
    """Clear the topology store before each test."""
    try:
        import topologic_fast as tf
        tf.clear_store()
    except ImportError:
        pass
    yield
    try:
        import topologic_fast as tf
        tf.clear_store()
    except ImportError:
        pass
