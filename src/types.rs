use deno_core::v8;
use deno_core::v8::{Global, HandleScope, Local, Value};
use pyo3::{IntoPy, PyAny, pyclass, pymethods, PyObject, PyResult, Python};
use pyo3::types::{PyDict, PyList};

#[pyclass(unsendable, module = "denopy")]
#[derive(Clone)]
pub struct JsFunction {
    pub inner: Global<v8::Function>,
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
    } else if let Result::<Local<v8::Function>, _>::Ok(function) = value.try_into() {
        Ok(JsFunction { inner: Global::new(scope, function) }.into_py(py))
    } else if let Result::<Local<v8::Array>, _>::Ok(array) = value.try_into() {
        let list = PyList::empty(py);
        for i in 0..array.length() {
            let v = array.get_index(scope, i).unwrap();
            list.append(v8_to_py(py, v, scope)?)?;
        }
        list.extract()
    } else if value.is_object() {
        let object = value.to_object(scope).unwrap();
        let props = object.get_own_property_names(scope, Default::default()).unwrap();
        let dict = PyDict::new(py);
        for i in 0..props.length() {
            let prop = props.get_index(scope, i).unwrap();
            let prop_value = object.get(scope, prop).unwrap();
            dict.set_item(
                v8_to_py(py, prop, scope)?,
                v8_to_py(py, prop_value, scope)?,
            )?;
        }
        dict.extract()
    } else {
        Ok(JsValue {
            inner: Global::new(scope, value),
            type_repr: value.type_of(scope).to_rust_string_lossy(scope),
        }.into_py(py))
    }
}

pub fn py_to_v8<'s>(object: &PyAny, scope: &mut HandleScope<'s>) -> PyResult<Local<'s, Value>> {
    if object.is_none() {
        Ok(v8::null(scope).into())
    } else if let Ok(s) = object.extract::<&str>() {
        Ok(v8::String::new(scope, s).unwrap().into())
    } else if let Ok(b) = object.extract::<bool>() {
        Ok(v8::Boolean::new(scope, b).into())
    } else if let Ok(i) = object.extract::<i32>() {
        Ok(v8::Integer::new(scope, i).to_int32(scope).unwrap().into())
    } else if let Ok(f) = object.extract::<f64>() {
        Ok(v8::Number::new(scope, f).into())
    } else if let Ok(list) = object.downcast::<PyList>() {
        let array = v8::Array::new(scope, list.len().try_into()?);
        for (i, o) in list.iter().enumerate() {
            let v = py_to_v8(o, scope)?;
            array.set_index(scope, i.try_into()?, v);
        }
        Ok(array.into())
    } else if let Ok(f) = object.extract::<JsFunction>() {
        Ok(Local::new(scope, f.inner).into())
    } else if let Ok(v) = object.extract::<JsValue>() {
        Ok(Local::new(scope, v.inner).into())
    } else {
        unimplemented!()
    }
}
