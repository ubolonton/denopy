use deno_core::v8::{Global, HandleScope, Value};
use pyo3::{IntoPy, pyclass, PyObject, PyResult, Python};

#[pyclass(unsendable, module="denopy")]
struct Function {
    function: Global<Value>
}

pub fn v8_to_py(py: Python<'_>, global_value: Global<Value>, scope: &mut HandleScope) -> PyResult<PyObject> {
    let value = global_value.open(scope);
    if value.is_null_or_undefined() {
        Ok(py.None())
    } else if value.is_string() {
        Ok(value.to_rust_string_lossy(scope).into_py(py))
    } else if value.is_function() {
        Ok(Function { function: global_value }.into_py(py))
    } else {
        let typ = value.type_of(scope).to_rust_string_lossy(scope);
        let repr = value.to_rust_string_lossy(scope);
        Ok(format!("{repr}: {typ}").into_py(py))
    }
}
