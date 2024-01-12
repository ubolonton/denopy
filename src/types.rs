use deno_core::_ops::RustToV8;
use deno_core::v8;
use deno_core::v8::{Function, Global, HandleScope, Local, Value};
use pyo3::{IntoPy, PyAny, pyclass, PyObject, PyResult, Python};
use pyo3::types::PyString;

#[pyclass(unsendable, module = "denopy")]
#[derive(Clone)]
pub struct JSFunction {
    pub inner: Global<Function>,
}

#[pyclass(unsendable, module = "denopy")]
#[derive(Clone)]
pub struct JSValue {
    pub inner: Global<Value>,
}

pub fn v8_to_py(py: Python<'_>, global_value: Global<Value>, scope: &mut HandleScope) -> PyResult<PyObject> {
    let value = global_value.to_v8(scope);
    // TODO: undefined should not be None.
    if value.is_null_or_undefined() {
        Ok(py.None())
    } else if value.is_string() {
        Ok(value.to_rust_string_lossy(scope).into_py(py))
    } else if value.is_function() {
        let function: Local<Function> = value.try_into().unwrap();
        Ok(JSFunction { inner: Global::new(scope, function) }.into_py(py))
    } else {
        Ok(JSValue { inner: Global::new(scope, value) }.into_py(py))
    }
}

pub fn py_to_v8(py: Python<'_>, object: &PyAny, scope: &mut HandleScope) -> PyResult<Global<Value>> {
    if object.is_none() {
        Ok(v8::null(scope))
            .map(Into::<Local<_>>::into)
            .map(|value| Global::new(scope, value))
    } else if let Ok(s) = object.downcast::<PyString>() {
        Ok(v8::String::new(scope, s.to_str()?).unwrap())
            .map(Into::<Local<_>>::into)
            .map(|value| Global::new(scope, value))
    } else if let Ok(f) = object.extract::<JSFunction>() {
        Ok(f.inner.to_v8(scope))
            .map(|value| Global::new(scope, value))
    } else if let Ok(v) = object.extract::<JSValue>() {
        Ok(v.inner)
    } else {
        unimplemented!()
    }
}
