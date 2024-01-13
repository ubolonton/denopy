import pytest
import denopy


@pytest.fixture
def runtime():
    return denopy.Runtime()


def test_strings(runtime):
    assert runtime.eval("'abc'") == 'abc'


def test_functions(runtime):
    log = runtime.eval("console.log")
    x = runtime.eval("x = {a: 1, b: '2'}; x")
    runtime.call(log, x)
