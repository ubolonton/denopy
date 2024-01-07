use std::rc::Rc;

use deno_core::{FsModuleLoader, JsRuntime, ModuleCode, RuntimeOptions};
use pyo3::prelude::*;

/// A wrapper around Deno's JSRuntime.
#[pyclass(unsendable, module = "denopy")]
struct Runtime {
    js_runtime: JsRuntime
}

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

    fn eval(&mut self, source_code: &str) -> PyResult<String> {
        let result = self.js_runtime.execute_script("<eval>", ModuleCode::from(source_code.to_owned()))?;
        let mut scope = self.js_runtime.handle_scope();
        let value = result.open(&mut scope);
        Ok(value.to_rust_string_lossy(&mut scope))
    }
}

/// A wrapper around `deno_core`.
#[pymodule]
fn denopy(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Runtime>()?;
    Ok(())
}
