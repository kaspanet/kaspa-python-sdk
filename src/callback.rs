use pyo3::{
    exceptions::PyException,
    prelude::*,
    types::{PyDict, PyModule, PyTuple},
};
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct PyCallback {
    callback: Arc<Py<PyAny>>,
    args: Option<Arc<Py<PyTuple>>>,
    kwargs: Option<Arc<Py<PyDict>>>,
}

impl PyCallback {
    pub(crate) fn new(callback: Py<PyAny>, args: Py<PyTuple>, kwargs: Py<PyDict>) -> Self {
        Self {
            callback: Arc::new(callback),
            args: Some(Arc::new(args)),
            kwargs: Some(Arc::new(kwargs)),
        }
    }

    pub(crate) fn callback_ptr_eq(&self, callback: &Py<PyAny>) -> bool {
        self.callback.as_ref().as_ptr() == callback.as_ptr()
    }

    fn add_event_to_args(&self, py: Python, event: Bound<PyDict>) -> PyResult<Py<PyTuple>> {
        match &self.args {
            Some(existing_args) => {
                let tuple_ref = existing_args.bind(py);
                let mut new_args: Vec<Py<PyAny>> =
                    tuple_ref.iter().map(|arg| arg.unbind()).collect();
                new_args.push(event.into());
                Ok(Py::from(PyTuple::new(py, new_args)?))
            }
            None => Ok(Py::from(PyTuple::new(py, [event])?)),
        }
    }

    pub(crate) fn execute(&self, py: Python, event: Bound<PyDict>) -> PyResult<Py<PyAny>> {
        let args = self.add_event_to_args(py, event)?;
        let kwargs = self.kwargs.as_ref().map(|kw| kw.bind(py));

        self.callback
            .call(py, args.bind(py), kwargs)
            .map_err(|err| {
                let traceback = PyModule::import(py, "traceback")
                    .and_then(|traceback| {
                        traceback.call_method(
                            "format_exception",
                            (err.get_type(py), err.value(py), err.traceback(py)),
                            None,
                        )
                    })
                    .map(|formatted| {
                        let trace_lines: Vec<String> = formatted
                            .extract()
                            .unwrap_or_else(|_| vec!["<Failed to retrieve traceback>".to_string()]);
                        trace_lines.join("")
                    })
                    .unwrap_or_else(|_| "<Failed to retrieve traceback>".to_string());

                PyException::new_err(traceback.to_string())
            })
    }
}
