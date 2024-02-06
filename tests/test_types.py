import math

import pytest
import denopy


@pytest.fixture
def runtime():
    return denopy.Runtime()


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


def test_roundtrips(runtime):
    identity = runtime.eval("x => x")
    for v in ["abc", 1, 5.3, True, False, None,
              [], [2, 3.4, "x"],
              {}, {'a': 5, 'b': ['x', dict(c=None)]}]:
        assert runtime.call(identity, v) == v
