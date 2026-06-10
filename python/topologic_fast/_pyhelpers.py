"""Pure-Python parity methods attached to the Rust topology classes.

Many topologicpy methods are pure-Python data/maths helpers (no geometry
kernel). Rather than reimplement each in Rust, we port topologicpy's exact
behavior here and attach them as static methods onto the corresponding
``topologic_fast`` classes (whose Rust types accept ``setattr``). This keeps the
drop-in surface growing quickly and verifiably.

These additions are purely additive — they only add method names that the Rust
classes do not already provide, so existing instance methods are untouched.

``install()`` is called once from ``__init__``.
"""
from __future__ import annotations


def _install_dictionary(Dictionary):
    """Attach topologicpy-compatible static Dictionary helpers."""

    def _to_py(d):
        return dict(d.PythonDictionary()) if d is not None else {}

    def _from_py(d):
        return Dictionary.ByPythonDictionary(d)

    def ValuesAtKeys(dictionary, keys, defaultValue=None, silent=False):
        d = _to_py(dictionary)
        return [d.get(k, defaultValue) for k in keys]

    def KeysAtValue(dictionary, value, silent=False):
        return [k for k, v in _to_py(dictionary).items() if v == value]

    def SetValuesAtKeys(dictionary, keys, values, silent=False):
        d = _to_py(dictionary)
        for k, v in zip(keys, values):
            d[k] = v
        return _from_py(d)

    def Union(dictionaryA, dictionaryB, silent=False):
        a, b = _to_py(dictionaryA), _to_py(dictionaryB)
        out = {}
        for k in set(a) | set(b):
            if k in a and k in b:
                out[k] = [a[k], b[k]]
            else:
                out[k] = a[k] if k in a else b[k]
        return _from_py(out)

    def Difference(dictionaryA, dictionaryB, silent=False):
        a, b = _to_py(dictionaryA), _to_py(dictionaryB)
        return _from_py({k: v for k, v in a.items() if k not in b})

    def Intersection(dictionaryA, dictionaryB, silent=False):
        a, b = _to_py(dictionaryA), _to_py(dictionaryB)
        return _from_py({k: [a[k], b[k]] for k in a if k in b})

    def SymmetricDifference(dictionaryA, dictionaryB, silent=False):
        a, b = _to_py(dictionaryA), _to_py(dictionaryB)
        out = {k: v for k, v in a.items() if k not in b}
        out.update({k: v for k, v in b.items() if k not in a})
        return _from_py(out)

    def ByMergedDictionaries(*dictionaries, silent=False):
        # Accept either varargs or a single list of dictionaries.
        if len(dictionaries) == 1 and isinstance(dictionaries[0], (list, tuple)):
            dictionaries = tuple(dictionaries[0])
        merged = {}
        for d in dictionaries:
            for k, v in _to_py(d).items():
                if k in merged:
                    existing = merged[k]
                    if isinstance(existing, list):
                        existing.append(v)
                    else:
                        merged[k] = [existing, v]
                else:
                    merged[k] = v
        return _from_py(merged)

    methods = {
        "ValuesAtKeys": ValuesAtKeys,
        "KeysAtValue": KeysAtValue,
        "SetValuesAtKeys": SetValuesAtKeys,
        "Union": Union,
        "Difference": Difference,
        "Intersection": Intersection,
        "SymmetricDifference": SymmetricDifference,
        "SymDif": SymmetricDifference,
        "ByMergedDictionaries": ByMergedDictionaries,
    }
    for name, fn in methods.items():
        if not hasattr(Dictionary, name):
            setattr(Dictionary, name, staticmethod(fn))


def _install_matrix(Matrix):
    """Attach topologicpy-compatible static Matrix helpers."""

    def Invert(matA, silent=False):
        import numpy as np
        return [list(map(float, row)) for row in np.linalg.inv(np.array(matA, dtype=float))]

    if not hasattr(Matrix, "Invert"):
        Matrix.Invert = staticmethod(Invert)


def _install_core_compat(namespace):
    """Attach ``topologic_core``-compatible instance methods to tf topology classes.

    topologicpy calls a small set of kernel instance methods on topology objects
    (e.g. ``GetTypeAsString``, ``Type``). topologic_fast's classes don't provide
    those topologic_core-specific names; attach them so topologicpy's Python layer
    (running on this kernel via the Core backend) can read tf objects.
    """
    type_table = [
        ("Vertex", 1), ("Edge", 2), ("Wire", 4), ("Face", 8),
        ("Shell", 16), ("Cell", 32), ("CellComplex", 64), ("Cluster", 128),
    ]
    for class_name, code in type_table:
        cls = getattr(namespace, class_name, None)
        if cls is None:
            continue
        name = class_name
        if not hasattr(cls, "GetTypeAsString"):
            cls.GetTypeAsString = (lambda _name: lambda self: _name)(name)
        if not hasattr(cls, "Type"):
            cls.Type = (lambda _code: lambda self: _code)(code)


def _install_fill_list_methods(namespace):
    """Make tf sub-topology accessors support topologic_core's fill-a-list idiom.

    topologicpy reads sub-topologies via ``topology.Vertices(None, out)`` where
    ``out`` is a list it expects the kernel to populate (the old C++ binding
    convention). tf's accessors instead return a list and take no args. We wrap
    each so it accepts the optional ``(host, out)`` arguments, extends ``out``
    when given, and still returns the list for ordinary callers.
    """
    SUB = ["Vertices", "Edges", "Wires", "Faces", "Shells", "Cells",
           "CellComplexes", "Clusters", "Apertures"]
    classes = ["Vertex", "Edge", "Wire", "Face", "Shell", "Cell", "CellComplex", "Cluster"]

    def make_wrapper(orig):
        def wrapper(self, host=None, out=None):
            res = list(orig(self)) if orig is not None else []
            if isinstance(out, list):
                out.extend(res)
            return res
        wrapper._tf_fill_wrapped = True
        return wrapper

    for class_name in classes:
        cls = getattr(namespace, class_name, None)
        if cls is None:
            continue
        for mname in SUB:
            orig = getattr(cls, mname, None)
            if orig is not None and getattr(orig, "_tf_fill_wrapped", False):
                continue
            setattr(cls, mname, make_wrapper(orig))


def install(namespace):
    """Attach all pure-Python helpers onto the given module namespace's classes."""
    _install_dictionary(namespace.Dictionary)
    _install_matrix(namespace.Matrix)
    _install_core_compat(namespace)
    _install_fill_list_methods(namespace)
