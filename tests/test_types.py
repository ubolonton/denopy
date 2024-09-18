import pytest
import denopy


@pytest.fixture
def runtime():
    return denopy.Runtime()


@pytest.fixture
def identity(runtime):
    return runtime.eval("x => x")


def assert_eq_type(expected_value, actual):
    assert actual == expected_value
    assert isinstance(actual, type(expected_value))


def test_floats(runtime, identity):
    for f in [1.5, 0.1, -10.2]:
        assert_eq_type(f, runtime.eval(f"{f}"))
        assert_eq_type(f, identity(f))


def test_small_integers(runtime, identity):
    # Fit into i32. Always int.
    for n in [2 ** 8, 2 ** 16, 2 ** 31 - 1, -(2 ** 31)]:
        assert_eq_type(n, runtime.eval(f"{n}"))
        assert_eq_type(n, identity(n))
        assert_eq_type(n, runtime.eval(f"{n}", convert_safe_integers=True))
        assert_eq_type(n, identity(n, convert_safe_integers=True))
        # Even if it's a whole float.
        assert_eq_type(n, runtime.eval(f"{float(n)}"))
        assert_eq_type(n, identity(float(n)))


def test_safe_integers(runtime, identity):
    # Fit into JavaScript safe integer range (f64 has 53-bit mantissa).
    for n in [2 ** 31, 2 ** 32, 2 ** 53 - 1, -(2 ** 53 - 1)]:
        # Default behavior.
        assert_eq_type(float(n), runtime.eval(f"{n}"))
        assert_eq_type(float(n), identity(n))
        # Nice behavior.
        assert_eq_type(n, runtime.eval(f"{n}", convert_safe_integers=True))
        assert_eq_type(n, identity(n, convert_safe_integers=True))


def test_unsafe_integers(runtime, identity):
    # Don't fit into the above. Always float.
    for n in [2 ** 53, - (2 ** 53), 2 ** 64, -(2 ** 64)]:
        assert_eq_type(float(n), runtime.eval(f"{n}"))
        assert_eq_type(float(n), identity(n))
        # Even with aggressive conversion.
        assert_eq_type(float(n), runtime.eval(f"{n}", convert_safe_integers=True))
        assert_eq_type(float(n), identity(n, convert_safe_integers=True))


def test_strings(runtime):
    assert runtime.eval("'abc'") == 'abc'


def test_roundtrips(runtime, identity):
    for v in ["abc", 1, 1.0, 5.3, True, False, None,
              [], [2, 3.4, "x"],
              {}, {'a': 5, 'b': ['x', dict(c=None)]}]:
        assert runtime.call(identity, v, unwrap=True) == v


def test_objects(runtime, identity):
    for py_object in [{}, dict(x=5)]:
        js_value = identity(py_object)
        assert isinstance(js_value, denopy.JsObject)
        assert runtime.unwrap(js_value) == py_object


def test_arrays(runtime, identity):
    for py_object in [[], list('abc')]:
        js_value = identity(py_object)
        assert isinstance(js_value, denopy.JsArray)
        assert runtime.unwrap(js_value) == py_object
