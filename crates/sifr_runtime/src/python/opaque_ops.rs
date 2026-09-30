use super::{ForeignObject, PythonError};
use pyo3::prelude::*;

/// Invoke the declared semantic close operation and consume the sealed identity.
/// A Python exception poisons the identity before ownership is released.
pub fn semantic_close(object: ForeignObject, method: impl AsRef<str>) -> Result<(), PythonError> {
    let method = method.as_ref().to_string();
    // Claim semantic cleanup before calling Python. Existing in-flight leases
    // pin the identity, while every public alias rejects new use or cleanup.
    let lease = object
        .begin_semantic_close()
        .map_err(PythonError::runtime)?;
    let outcome = super::attach(|py| {
        let receiver = lease.clone_ref(py).map_err(PythonError::runtime)?;
        let _call_depth = super::enter_python_call();
        receiver
            .bind(py)
            .call_method0(method.as_str())
            .map(|_| ())
            .map_err(|error| PythonError::from_pyerr(py, error, "cleanup", &method))
    })
    .map_err(PythonError::runtime)
    .and_then(|outcome| outcome);
    match outcome {
        Ok(()) => {
            object.finish_semantic_close(true);
            Ok(())
        }
        Err(error) => {
            object.finish_semantic_close(false);
            Err(error)
        }
    }
}
