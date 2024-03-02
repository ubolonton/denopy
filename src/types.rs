use deno_core::v8;
use deno_core::v8::{Global, HandleScope, Local, Value};
use pyo3::{IntoPy, Py, PyAny, pyclass, pymethods, PyObject, PyResult, Python};
use pyo3::types::{PyDict, PyList, PyTuple};

use crate::Runtime;

/// A JavaScript function that is callable from Python code.
#[pyclass(unsendable, module = "denopy")]
#[derive(Clone)]
pub struct JsFunction {
    pub inner: Global<v8::Function>,
    runtime: Py<Runtime>,
}

#[pymethods]
impl JsFunction {
    #[pyo3(signature = (* args, unwrap = false, this = None))]
    fn __call__(&self, py: Python<'_>, args: &PyTuple, unwrap: bool, this: Option<&PyAny>) -> PyResult<PyObject> {
        self.runtime.borrow_mut(py).call(py, self, args, unwrap, this)
    }

    fn __repr__(&self, py: Python<'_>) -> String {
        let mut runtime = self.runtime.borrow_mut(py);
        let scope = &mut runtime.js_runtime.handle_scope();
        let f = self.inner.open(scope);
        let name = f.get_name(scope).to_rust_string_lossy(scope);
        let detail = f.to_detail_string(scope).unwrap().to_rust_string_lossy(scope);
        format!("<JsFunction {name}: {detail}>")
    }
}

#[pyclass(unsendable, module = "denopy")]
#[derive(Clone)]
pub struct JsObject {
    inner: Global<v8::Object>,
}

#[pyclass(unsendable, module = "denopy")]
#[derive(Clone)]
pub struct JsArray {
    inner: Global<v8::Array>,
}

/// A generic JavaScript value.
#[pyclass(unsendable, module = "denopy")]
#[derive(Clone)]
pub struct JsValue {
    inner: Global<Value>,
    type_repr: String,
}

#[pymethods]
impl JsValue {
    fn __repr__(&self) -> String {
        format!("<JSValue [{}]>", self.type_repr)
    }
}

pyo3::create_exception!(denopy, JsError, pyo3::exceptions::PyException);

/// Converts a V8 value into a Python object.
///
/// Unless `unwrap` is true, complex types like objects and arrays are wrapped in opaque Python
/// objects.
pub fn v8_to_py(value: Local<Value>, scope: &mut HandleScope, runtime: &Py<Runtime>, py: Python<'_>,
                unwrap: bool) -> PyResult<PyObject> {
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
        Ok(JsFunction {
            inner: Global::new(scope, function),
            runtime: runtime.clone_ref(py),
        }.into_py(py))
    } else if let Result::<Local<v8::Array>, _>::Ok(array) = value.try_into() {
        if unwrap {
            let list = PyList::empty(py);
            for i in 0..array.length() {
                let v = array.get_index(scope, i).unwrap();
                list.append(v8_to_py(v, scope, runtime, py, unwrap)?)?;
            }
            list.extract()
        } else {
            Ok(JsArray { inner: Global::new(scope, array) }.into_py(py))
        }
    } else if value.is_object() {
        let object = value.to_object(scope).unwrap();
        if unwrap {
            let props = object.get_own_property_names(scope, Default::default()).unwrap();
            let dict = PyDict::new(py);
            for i in 0..props.length() {
                let prop = props.get_index(scope, i).unwrap();
                let prop_value = object.get(scope, prop).unwrap();
                dict.set_item(
                    v8_to_py(prop, scope, runtime, py, unwrap)?,
                    v8_to_py(prop_value, scope, runtime, py, unwrap)?,
                )?;
            }
            dict.extract()
        } else {
            Ok(JsObject { inner: Global::new(scope, object) }.into_py(py))
        }
    } else {
        Ok(JsValue {
            inner: Global::new(scope, value),
            type_repr: value.type_of(scope).to_rust_string_lossy(scope),
        }.into_py(py))
    }
}

/// Converts a Python object into a V8 value.
///
/// Lists are converted into arrays.
/// Dicts are converted into objects.
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
    } else if let Ok(dict) = object.downcast::<PyDict>() {
        let object = v8::Object::new(scope);
        for (k, o) in dict.iter() {
            let prop = py_to_v8(k, scope)?;
            let prop_value = py_to_v8(o, scope)?;
            object.set(scope, prop, prop_value);
        }
        Ok(object.into())
    } else if let Ok(list) = object.downcast::<PyList>() {
        let array = v8::Array::new(scope, list.len().try_into()?);
        for (i, o) in list.iter().enumerate() {
            let v = py_to_v8(o, scope)?;
            array.set_index(scope, i.try_into()?, v);
        }
        Ok(array.into())
    } else if let Ok(f) = object.extract::<JsFunction>() {
        Ok(Local::new(scope, f.inner).into())
    } else if let Ok(f) = object.extract::<JsObject>() {
        Ok(Local::new(scope, f.inner).into())
    } else if let Ok(f) = object.extract::<JsArray>() {
        Ok(Local::new(scope, f.inner).into())
    } else if let Ok(v) = object.extract::<JsValue>() {
        Ok(Local::new(scope, v.inner).into())
    } else {
        unimplemented!()
    }
}
