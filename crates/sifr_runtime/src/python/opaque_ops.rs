use super::{ForeignObject, PythonError};
use pyo3::prelude::*;

/// Invoke the declared semantic close operation and consume the sealed identity.
/// A Python exception poisons the identity before ownership is released.
pub fn semantic_close(object: ForeignObject, method: impl AsRef<str>) -> Result<(), PythonError> {
    let method = method.as_ref().to_string();
    let outcome = super::attach(|py| {
        let receiver = object.clone_ref(py).map_err(PythonError::runtime)?;
        let _call_depth = super::enter_python_call();
        receiver
            .bind(py)
            .call_method0(method.as_str())
            .map(|_| ())
            .map_err(|error| PythonError::from_pyerr(py, error, "cleanup", &method))
    })
    .map_err(PythonError::runtime)?;
    match outcome {
        Ok(()) => {
            object.close();
            Ok(())
        }
        Err(error) => {
            object.poison();
            Err(error)
        }
    }
}
