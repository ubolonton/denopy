use deno_core::v8::{Function, Global, HandleScope, Local, Value};
use pyo3::{IntoPy, pyclass, PyObject, PyResult, Python};

#[pyclass(unsendable, module="denopy")]
pub struct JSFunction {
    pub inner: Global<Function>
}

pub fn v8_to_py(py: Python<'_>, global_value: Global<Value>, scope: &mut HandleScope) -> PyResult<PyObject> {
    let value = Local::new(scope, global_value);
    if value.is_null_or_undefined() {
        Ok(py.None())
    } else if value.is_string() {
        Ok(value.to_rust_string_lossy(scope).into_py(py))
    } else if value.is_function() {
        let lf: Local<Function> = value.try_into().unwrap();
        let inner = Global::new(scope, lf);
        Ok(JSFunction { inner }.into_py(py))
    } else {
        let typ = value.type_of(scope).to_rust_string_lossy(scope);
        let repr = value.to_rust_string_lossy(scope);
        Ok(format!("{repr}: {typ}").into_py(py))
    }
}
