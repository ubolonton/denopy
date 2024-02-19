import pytest
import denopy


@pytest.fixture
def runtime():
    return denopy.Runtime()


@pytest.fixture
def identity(runtime):
    return runtime.eval("x => x")


def test_numbers(runtime):
    for n in [1, 1.0, -5, -7.5]:
        assert runtime.eval(f"{n}") == n


def test_strings(runtime):
    assert runtime.eval("'abc'") == 'abc'


def test_roundtrips(runtime, identity):
    for v in ["abc", 1, 5.3, True, False, None,
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
