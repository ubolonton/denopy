use std::cell::RefCell;
use std::rc::Rc;

use deno_core::{FsModuleLoader, JsRuntime, ModuleCode, ModuleId, ModuleSpecifier, RuntimeOptions, v8};
use deno_core::v8::{Local, TryCatch};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use types::{JsArray, JsError, JsFunction, JsObject, JsValue};

mod types;

/// A wrapper around deno_core's JsRuntime.
///
/// Instances of this class can only be used from the thread they were created on.
/// If they are sent to another thread, they will panic when used.
///
/// Each thread is associated with at most one instance. After the constructor is called once,
/// subsequent calls on the same thread return the same instance.
#[pyclass(unsendable, module = "denopy")]
struct Runtime {
    js_runtime: JsRuntime,
    tokio_runtime: tokio::runtime::Runtime,
}

thread_local! {
    static RUNTIME: RefCell<Option<Py<Runtime>>> = RefCell::new(None);
}

macro_rules! v8_to_py {
    ($value:expr, $scope:ident, $py:ident, $unwrap:expr) => {
        RUNTIME.with(|cell| types::v8_to_py(
            $value, $scope, cell.borrow().as_ref().unwrap(), $py, $unwrap,
        ))
    }
}

#[pymethods]
impl Runtime {
    #[new]
    fn new(py: Python<'_>) -> PyResult<Py<Self>> {
        if let Some(runtime) = RUNTIME.with(|cell| {
            cell.borrow().as_ref().map(|rt| rt.clone_ref(py))
        }) {
            return Ok(runtime);
        }

        // TODO: Figure out what happens if this is called from a thread that is not a child of the thread where the
        //  module was loaded.
        let js_runtime = JsRuntime::new(RuntimeOptions {
            module_loader: Some(Rc::new(FsModuleLoader)),
            ..Default::default()
        });
        let tokio_runtime = tokio::runtime::Builder::new_current_thread()
            .max_blocking_threads(1).enable_all().build()?;
        let runtime = Py::new(py, Self { js_runtime, tokio_runtime })?;
        RUNTIME.with(|cell| cell.borrow_mut().replace(runtime.clone_ref(py)));
        Ok(runtime)
    }

    /// Convert a wrapped JavaScript value into its Python equivalent.
    fn unwrap(&mut self, py: Python<'_>, value: &PyAny) -> PyResult<PyObject> {
        let scope = &mut self.js_runtime.handle_scope();
        // TODO: Don't create JS values unnecessarily.
        let js_value = types::py_to_v8(value, scope)?;
        v8_to_py!(js_value, scope, py, true)
    }

    /// Return the value of a JavaScript object's property.
    ///
    /// The result may be a wrapped JavaScript value, unless 'unwrap' is True.
    #[pyo3(signature = (object, property, *, unwrap = false))]
    fn get(&mut self, py: Python<'_>, object: &PyAny, property: &PyAny, unwrap: bool) -> PyResult<PyObject> {
        let scope = &mut self.js_runtime.handle_scope();
        let js_value = types::py_to_v8(object, scope)?;
        if let Some(js_object) = js_value.to_object(scope) {
            let prop = types::py_to_v8(property, scope)?;
            if let Some(prop_value) = js_object.get(scope, prop) {
                return v8_to_py!(Local::new(scope, prop_value), scope, py, unwrap);
            }
        }
        Ok(py.None())
    }

    /// Evaluate a piece of JavaScript.
    ///
    /// The evaluation result may contain wrapped JavaScript values,
    /// unless 'unwrap' is True.
    ///
    /// The 'name' parameter is used in stack traces and error messages.
    /// It should be a literal string, otherwise its memory will be leaked.
    /// If it is None, the name "<eval>" is used.
    #[pyo3(signature = (source_code, *, unwrap = false, name = None))]
    fn eval(&mut self, py: Python<'_>, source_code: &str, unwrap: bool, name: Option<String>) -> PyResult<PyObject> {
        let name: &'static str = match name {
            Some(s) => s.leak(),
            None => "<eval>",
        };
        let result = self.js_runtime.execute_script(name, ModuleCode::from(source_code.to_owned()))
            .map_err(|err| JsError::new_err(err.to_string()))?;
        let scope = &mut self.js_runtime.handle_scope();
        v8_to_py!(Local::new(scope, result), scope, py, unwrap)
    }

    fn load_main_module(&mut self, py: Python<'_>, path: &str) -> PyResult<PyObject> {
        self.tokio_runtime.block_on(async {
            let specifier = ModuleSpecifier::from_file_path(path).unwrap();
            let module_id = self.js_runtime.load_main_module(&specifier, None).await?;
            Ok(module_id.into_py(py))
        })
    }

    fn load_side_module(&mut self, py: Python<'_>, path: &str) -> PyResult<PyObject> {
        self.tokio_runtime.block_on(async {
            let specifier = ModuleSpecifier::from_file_path(path).unwrap();
            let module_id = self.js_runtime.load_side_module(&specifier, None).await?;
            Ok(module_id.into_py(py))
        })
    }

    fn mod_evaluate(&mut self, py: Python<'_>, module_id: ModuleId) -> PyResult<PyObject> {
        self.tokio_runtime.block_on(async {
            self.js_runtime.mod_evaluate(module_id).await
                .map_err(|err| JsError::new_err(err.to_string()))?;
            Ok(module_id.into_py(py))
        })
    }

    /// Call a JavaScript function.
    ///
    /// The result may contain wrapped JavaScript values, unless 'unwrap' is True.
    ///
    /// If 'this' is not None, the function will be called as a method, with 'this' as the receiver.
    #[pyo3(signature = (function, * args, unwrap = false, this = None))]
    fn call(&mut self, py: Python<'_>, function: &JsFunction, args: &PyTuple, unwrap: bool, this: Option<&PyAny>) -> PyResult<PyObject> {
        let scope = &mut self.js_runtime.handle_scope();
        let this = match this {
            Some(object) => types::py_to_v8(object, scope)?,
            None => v8::undefined(scope).into(),
        };
        let args = args.iter()
            .map(|object| types::py_to_v8(object, scope))
            .collect::<PyResult<Vec<_>>>()?;
        let scope = &mut TryCatch::new(scope);
        let return_result = function.inner.open(scope).call(scope, this, &args);
        if let Some(exception) = scope.exception() {
            let js_error = deno_core::error::JsError::from_v8_exception(scope, exception);
            let exception = v8_to_py!(exception, scope, py, unwrap)?;
            let py_err = JsError::new_err(js_error.to_string());
            // XXX: We want readable traceback, so JsError.__str__ should only contain the formatted
            //  JS stacktrace. Since we don't know how to customize that, we attach the thrown value
            //  to the JsError exception after constructing it.
            py_err.to_object(py).setattr(py, "value", exception)?;
            Err(py_err)
        } else if let Some(result) = return_result {
            v8_to_py!(Local::new(scope, result), scope, py, unwrap)
        } else {
            Ok(py.None())
        }
    }
}

/// A wrapper around `deno_core`.
#[pymodule]
fn denopy(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Runtime>()?;
    m.add_class::<JsArray>()?;
    m.add_class::<JsFunction>()?;
    m.add_class::<JsObject>()?;
    m.add_class::<JsValue>()?;
    m.add("JsError", py.get_type::<JsError>())?;
    Ok(())
}
