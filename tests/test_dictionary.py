"""Tests for Dictionary class."""
import pytest


def test_dictionary_new():
    """Test creating a new empty dictionary."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    d = tf.Dictionary()

    assert d is not None
    assert d.IsEmpty() == True
    assert d.Len() == 0


def test_dictionary_by_keys_values():
    """Test creating a dictionary from keys and values."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    keys = ["name", "value", "active"]
    values = ["test", 42, True]

    d = tf.Dictionary.ByKeysValues(keys, values)

    assert d.Len() == 3
    assert d.Contains("name") == True
    assert d.Contains("value") == True
    assert d.Contains("active") == True


def test_dictionary_by_python_dictionary():
    """Test creating from a Python dict."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    py_dict = {
        "name": "test",
        "count": 100,
        "ratio": 3.14,
        "enabled": True
    }

    d = tf.Dictionary.ByPythonDictionary(py_dict)

    assert d.Len() == 4


def test_dictionary_value_at_key():
    """Test getting a value by key."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    d = tf.Dictionary.ByKeysValues(["name", "value"], ["test", 42])

    assert d.ValueAtKey("name") == "test"
    assert d.ValueAtKey("value") == 42
    assert d.ValueAtKey("nonexistent") is None


def test_dictionary_set_value_at_key():
    """Test setting a value at a key."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    d = tf.Dictionary()

    d.SetValueAtKey("key1", "value1")
    d.SetValueAtKey("key2", 123)
    d.SetValueAtKey("key3", 3.14)
    d.SetValueAtKey("key4", True)

    assert d.Len() == 4
    assert d.ValueAtKey("key1") == "value1"
    assert d.ValueAtKey("key2") == 123
    assert abs(d.ValueAtKey("key3") - 3.14) < 1e-10
    assert d.ValueAtKey("key4") == True


def test_dictionary_keys():
    """Test getting all keys."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    d = tf.Dictionary.ByKeysValues(["a", "b", "c"], [1, 2, 3])
    keys = d.Keys()

    assert len(keys) == 3
    assert "a" in keys
    assert "b" in keys
    assert "c" in keys


def test_dictionary_values():
    """Test getting all values."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    d = tf.Dictionary.ByKeysValues(["a", "b"], [10, 20])
    values = d.Values()

    assert len(values) == 2
    assert 10 in values
    assert 20 in values


def test_dictionary_contains():
    """Test checking if a key exists."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    d = tf.Dictionary.ByKeysValues(["exists"], ["value"])

    assert d.Contains("exists") == True
    assert d.Contains("missing") == False


def test_dictionary_python_dictionary():
    """Test converting back to Python dict."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    original = {
        "name": "test",
        "value": 42
    }

    d = tf.Dictionary.ByPythonDictionary(original)
    result = d.PythonDictionary()

    assert result["name"] == "test"
    assert result["value"] == 42


def test_dictionary_to_json():
    """Test converting to JSON."""
    try:
        import topologic_fast as tf
        import json
    except ImportError:
        pytest.skip("topologic_fast not built")

    d = tf.Dictionary.ByKeysValues(["key"], ["value"])
    json_str = d.ToJSON()

    # Should be valid JSON
    parsed = json.loads(json_str)
    assert "key" in parsed


def test_dictionary_from_json():
    """Test creating from JSON."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    json_str = '{"name": "test", "value": 123}'
    d = tf.Dictionary.FromJSON(json_str)

    assert d.Len() == 2
    assert d.ValueAtKey("name") == "test"
    assert d.ValueAtKey("value") == 123


def test_dictionary_roundtrip_json():
    """Test JSON roundtrip."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    original = tf.Dictionary.ByKeysValues(
        ["str", "int", "float", "bool"],
        ["hello", 42, 3.14, True]
    )

    json_str = original.ToJSON()
    restored = tf.Dictionary.FromJSON(json_str)

    assert original.Len() == restored.Len()
    assert restored.ValueAtKey("str") == "hello"
    assert restored.ValueAtKey("int") == 42


def test_dictionary_update_value():
    """Test updating an existing value."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    d = tf.Dictionary.ByKeysValues(["key"], ["old_value"])
    assert d.ValueAtKey("key") == "old_value"

    d.SetValueAtKey("key", "new_value")
    assert d.ValueAtKey("key") == "new_value"


def test_dictionary_list_values():
    """Test dictionary with list values."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    d = tf.Dictionary()
    d.SetValueAtKey("list", [1, 2, 3, 4, 5])

    result = d.ValueAtKey("list")
    assert len(result) == 5
    assert result[0] == 1
    assert result[4] == 5


def test_dictionary_repr():
    """Test dictionary string representation."""
    try:
        import topologic_fast as tf
    except ImportError:
        pytest.skip("topologic_fast not built")

    d = tf.Dictionary.ByKeysValues(["a", "b"], [1, 2])
    repr_str = repr(d)

    assert "Dictionary" in repr_str
    assert "2" in repr_str  # len=2
