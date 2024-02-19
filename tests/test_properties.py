import math

import pytest
import denopy


@pytest.fixture
def runtime():
    return denopy.Runtime()


def get(runtime, object, *properties):
    value = object
    for property in properties:
        value = runtime.get(value, property)
    return value


def test_get(runtime):
    x = runtime.eval('x = {foo: {bar: [1]}}')
    assert isinstance(runtime.get(x, 'foo'), denopy.JsObject)
    assert runtime.get(x, 'foo', unwrap=True) == {'bar': [1]}
    assert get(runtime, x, 'foo', 'bar', 0) == 1, "Repeated get should work"

    js_sin = runtime.eval("Math.sin")
    js_math = runtime.eval("Math")
    assert js_sin(3) == runtime.get(js_math, 'sin')(3)
