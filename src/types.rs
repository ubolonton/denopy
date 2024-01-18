use deno_core::_ops::RustToV8;
use deno_core::v8;
use deno_core::v8::{Function, Global, HandleScope, Local, Value};
use pyo3::{IntoPy, PyAny, pyclass, pymethods, PyObject, PyResult, Python};
use pyo3::types::PyList;

#[pyclass(unsendable, module = "denopy")]
#[derive(Clone)]
pub struct JsFunction {
    pub inner: Global<Function>,
}

#[pyclass(unsendable, module = "denopy")]
#[derive(Clone)]
pub struct JsValue {
    inner: Global<Value>,
    type_repr: String
}

#[pymethods]
impl JsValue {
    fn __repr__(&self) -> String {
        format!("<JSValue [{}]>", self.type_repr)
    }
}

pub fn v8_to_py(py: Python<'_>, value: Local<Value>, scope: &mut HandleScope) -> PyResult<PyObject> {
    // We need to use predicates to check the type first, instead of casting, since JavaScript's
    // type casting rules are rather weird.
    // TODO: undefined should not be None.
    if value.is_null_or_undefined() {
        Ok(py.None())
    } else if value.is_string() {
        Ok(value.to_rust_string_lossy(scope).into_py(py))
    } else if value.is_boolean() {
        Ok(value.boolean_value(scope).into_py(py))
    } else if value.is_int32() {
        Ok(value.int32_value(scope).unwrap().into_py(py))
    } else if value.is_uint32() {
        Ok(value.uint32_value(scope).unwrap().into_py(py))
    } else if value.is_number() {
        Ok(value.number_value(scope).unwrap().into_py(py))
    } else if let Result::<Local<Function>, _>::Ok(function) = value.try_into() {
        Ok(JsFunction { inner: Global::new(scope, function) }.into_py(py))
    } else if value.is_array() {
        let object = value.to_object(scope).unwrap();
        let length_str = v8::String::new(scope, "length").unwrap().into();
        let length = object.get(scope, length_str).unwrap().uint32_value(scope).unwrap();
        let list = PyList::empty(py);
        for i in 0..length {
            let v = object.get_index(scope, i).unwrap();
            list.append(v8_to_py(py, v, scope)?)?;
        }
        list.extract()
    } else {
        Ok(JsValue {
            inner: Global::new(scope, value),
            type_repr: value.type_of(scope).to_rust_string_lossy(scope),
        }.into_py(py))
    }
}

pub fn py_to_v8(py: Python<'_>, object: &PyAny, scope: &mut HandleScope) -> PyResult<Global<Value>> {
    if object.is_none() {
        Ok(v8::null(scope))
            .map(Into::<Local<_>>::into)
            .map(|value| Global::new(scope, value))
    } else if let Ok(s) = object.extract::<&str>() {
        Ok(v8::String::new(scope, s).unwrap())
            .map(Into::<Local<_>>::into)
            .map(|value| Global::new(scope, value))
    } else if let Ok(b) = object.extract::<bool>() {
        Ok(v8::Boolean::new(scope, b))
            .map(Into::<Local<_>>::into)
            .map(|value| Global::new(scope, value))
    } else if let Ok(i) = object.extract::<i32>() {
        Ok(v8::Integer::new(scope, i).to_int32(scope).unwrap())
            .map(Into::<Local<_>>::into)
            .map(|value| Global::new(scope, value))
    } else if let Ok(f) = object.extract::<f64>() {
        Ok(v8::Number::new(scope, f))
            .map(Into::<Local<_>>::into)
            .map(|value| Global::new(scope, value))
    } else if let Ok(f) = object.extract::<JsFunction>() {
        Ok(f.inner.to_v8(scope))
            .map(|value| Global::new(scope, value))
    } else if let Ok(v) = object.extract::<JsValue>() {
        Ok(v.inner)
    } else {
        unimplemented!()
    }
}
