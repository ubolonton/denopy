use std::rc::Rc;

use deno_core::{FsModuleLoader, JsRuntime, ModuleCode, ModuleId, ModuleSpecifier, RuntimeOptions};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use types::JsFunction;

mod types;

/// A wrapper around Deno's JsRuntime.
/// Objects of this class can only be used from the thread they were created on.
/// If they are sent to another thread, they will panic when used.
#[pyclass(unsendable, module = "denopy")]
struct Runtime {
    js_runtime: JsRuntime,
    tokio_runtime: tokio::runtime::Runtime,
}

#[pymethods]
impl Runtime {
    #[new]
    fn new() -> PyResult<Self> {
        let js_runtime = JsRuntime::new(RuntimeOptions {
            module_loader: Some(Rc::new(FsModuleLoader)),
            ..Default::default()
        });
        let tokio_runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
        Ok(Self { js_runtime, tokio_runtime })
    }

    fn eval(&mut self, py: Python<'_>, source_code: &str) -> PyResult<PyObject> {
        let result = self.js_runtime.execute_script("<eval>", ModuleCode::from(source_code.to_owned()))?;
        types::v8_to_py(py, result, &mut self.js_runtime.handle_scope())
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
            self.js_runtime.mod_evaluate(module_id).await?;
            Ok(module_id.into_py(py))
        })
    }

    #[pyo3(signature = (function, *args))]
    fn call(&mut self, py: Python<'_>, function: &JsFunction, args: &PyTuple) -> PyResult<PyObject> {
        let args = {
            let scope = &mut self.js_runtime.handle_scope();
            args.iter()
                .map(|object| types::py_to_v8(py, object, scope))
                .collect::<PyResult<Vec<_>>>()?
        };
        self.tokio_runtime.block_on(async {
            let result = self.js_runtime.call_with_args(&function.inner, &args).await?;
            types::v8_to_py(py, result, &mut self.js_runtime.handle_scope())
        })
    }
}

fn dbg_thread(msg: &str) {
    let thread = std::thread::current();
    let name = thread.name().unwrap_or("unknown");
    let id = thread.id();
    println!("{msg} thread: {name} {id:?}");
}

/// A wrapper around `deno_core`.
#[pymodule]
fn denopy(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Runtime>()?;
    Ok(())
}
