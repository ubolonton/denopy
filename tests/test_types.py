import math

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


def test_functions(runtime):
    log = runtime.eval("console.log")
    x = runtime.eval("x = {a: 1, b: '2'}; x")
    runtime.call(log, x)

    js_sin = runtime.eval("Math.sin")
    assert runtime.call(js_sin, 1) == math.sin(1)
    assert js_sin(2) == math.sin(2)


def test_methods(runtime):
    runtime.eval("""
        function Foo(bar) {
          this.bar = bar;
        }
        Foo.prototype.get = function() {
          return this.bar;
        };
    """)
    obj = runtime.eval("obj = new Foo({x: 5})")
    assert isinstance(runtime.eval("obj.bar"), denopy.JsObject)
    assert runtime.eval("obj.bar", unwrap=True) == {'x': 5}
    get = runtime.eval("obj.get")
    assert isinstance(get(this=obj), denopy.JsObject)
    assert get(this=obj, unwrap=True) == {'x': 5}


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
