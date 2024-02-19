import math

import pytest
import denopy


@pytest.fixture
def runtime():
    return denopy.Runtime()


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
