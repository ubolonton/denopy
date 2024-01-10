use std::rc::Rc;
use std::sync::OnceLock;

use deno_core::{FsModuleLoader, JsRuntime, ModuleCode, ModuleId, ModuleSpecifier, RuntimeOptions};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use types::JSFunction;

mod types;

/// A wrapper around Deno's JSRuntime.
#[pyclass(module = "denopy")]
struct Runtime {
    // TODO: Keep the runtime.
    js_runtime: JsRuntime
}

// Safety: Runtime implementation ensures that JSRuntime is only accessed from the thread it was created on.
// TODO: Make the implementation actually satisfy this.
unsafe impl Send for Runtime {}

#[pymethods]
impl Runtime {
    #[new]
    fn new() -> PyResult<Self> {
        let js_runtime = JsRuntime::new(RuntimeOptions {
            module_loader: Some(Rc::new(FsModuleLoader)),
            ..Default::default()
        });
        Ok(Self { js_runtime })
    }

    fn eval(&mut self, py: Python<'_>, source_code: &str) -> PyResult<PyObject> {
        let result = self.js_runtime.execute_script("<eval>", ModuleCode::from(source_code.to_owned()))?;
        types::v8_to_py(py, result, &mut self.js_runtime.handle_scope())
    }

    fn load_main_module(&mut self, py: Python<'_>, path: &str) -> PyResult<PyObject> {
        TOKIO_RUNTIME.get().unwrap().block_on(async {
            let specifier = ModuleSpecifier::from_file_path(path).unwrap();
            let module_id = self.js_runtime.load_main_module(&specifier, None).await?;
            Ok(module_id.into_py(py))
        })
    }

    fn load_side_module(&mut self, py: Python<'_>, path: &str) -> PyResult<PyObject> {
        TOKIO_RUNTIME.get().unwrap().block_on(async {
            let specifier = ModuleSpecifier::from_file_path(path).unwrap();
            let module_id = self.js_runtime.load_side_module(&specifier, None).await?;
            Ok(module_id.into_py(py))
        })
    }

    fn mod_evaluate(&mut self, py: Python<'_>, module_id: ModuleId) -> PyResult<PyObject> {
        TOKIO_RUNTIME.get().unwrap().block_on(async {
            self.js_runtime.mod_evaluate(module_id).await?;
            Ok(module_id.into_py(py))
        })
    }

    #[pyo3(signature = (function, *args))]
    fn call(&mut self, py: Python<'_>, function: &JSFunction, args: &PyTuple) -> PyResult<PyObject> {
        let args = {
            let scope = &mut self.js_runtime.handle_scope();
            args.iter()
                .map(|o| types::py_to_v8(py, o, scope))
                .collect::<PyResult<Vec<_>>>()?
        };
        TOKIO_RUNTIME.get().unwrap().block_on(async {
            let result = self.js_runtime.call_with_args(&function.inner, &args).await?;
            types::v8_to_py(py, result, &mut self.js_runtime.handle_scope())
        })
    }
}

static TOKIO_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn dbg_thread(msg: &str) {
    let thread = std::thread::current();
    let name = thread.name().unwrap_or("unknown");
    let id = thread.id();
    println!("{msg} thread: {name} {id:?}");
}

/// A wrapper around `deno_core`.
#[pymodule]
fn denopy(_py: Python, m: &PyModule) -> PyResult<()> {
    let tok = tokio::runtime::Builder::new_multi_thread().worker_threads(1)
        .enable_all().build().unwrap();
    dbg_thread("module init");
    let init = tok.spawn(async {
        dbg_thread("tokio runtime");
        Runtime::new()
    });
    tok.block_on(init).unwrap();
    TOKIO_RUNTIME.set(tok).unwrap();
    m.add_class::<Runtime>()?;
    Ok(())
}
