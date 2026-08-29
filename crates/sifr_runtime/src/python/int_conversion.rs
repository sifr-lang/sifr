use super::PythonError;
use super::object_ops::{ObjectHandle, clone_handle, conversion_error, store_object};
use crate::{DEFAULT_MAX_INTEGER_DIGITS, SifrInt};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyInt, PyList, PyTuple};
use std::collections::HashMap;

pub fn from_int(value: impl Into<SifrInt>) -> Result<ObjectHandle, PythonError> {
    let value = value.into();
    super::attach(|py| int_to_python(py, &value, "from int").and_then(store_object))
        .map_err(PythonError::runtime)?
}

pub fn to_int(object: &ObjectHandle) -> Result<SifrInt, PythonError> {
    let boundary_path = object.boundary_path().to_string();
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        extract_sifr_int(
            py,
            object.bind(py),
            if boundary_path.is_empty() {
                "to_int"
            } else {
                &boundary_path
            },
        )
    })
    .map_err(PythonError::runtime)?
}

pub fn copy_list_int(object: &ObjectHandle) -> Result<Vec<SifrInt>, PythonError> {
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        let list = object
            .bind(py)
            .cast::<PyList>()
            .map_err(|_| conversion_error("expected Python list", "copy_list_int"))?;
        list.iter()
            .enumerate()
            .map(|(index, value)| extract_sifr_int(py, &value, &format!("copy_list_int[{index}]")))
            .collect()
    })
    .map_err(PythonError::runtime)?
}

pub fn copy_tuple_int(object: &ObjectHandle) -> Result<Vec<SifrInt>, PythonError> {
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        let tuple = object
            .bind(py)
            .cast::<PyTuple>()
            .map_err(|_| conversion_error("expected Python tuple", "copy_tuple_int"))?;
        tuple
            .iter()
            .enumerate()
            .map(|(index, value)| extract_sifr_int(py, &value, &format!("copy_tuple_int[{index}]")))
            .collect()
    })
    .map_err(PythonError::runtime)?
}

pub fn copy_dict_str_int(object: &ObjectHandle) -> Result<HashMap<String, SifrInt>, PythonError> {
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        let dict = object
            .bind(py)
            .cast::<PyDict>()
            .map_err(|_| conversion_error("expected Python dict", "copy_dict_str_int"))?;
        let mut converted = HashMap::with_capacity(dict.len());
        for (key, value) in dict.iter() {
            let key = key
                .extract::<String>()
                .map_err(|_| conversion_error("expected Python str", "copy_dict_str_int.<key>"))?;
            let value = extract_sifr_int(py, &value, &format!("copy_dict_str_int[{key:?}]"))?;
            converted.insert(key, value);
        }
        Ok(converted)
    })
    .map_err(PythonError::runtime)?
}

pub(super) fn int_to_python(
    py: Python<'_>,
    value: &SifrInt,
    context: &str,
) -> Result<Py<PyAny>, PythonError> {
    let integer = py
        .import("builtins")
        .and_then(|builtins| builtins.getattr("int"))
        .and_then(|constructor| constructor.call1((value.to_string(),)))
        .map_err(|error| PythonError::from_pyerr(py, error, "conversion", context))?;
    Ok(integer.unbind())
}

pub(super) fn extract_sifr_int(
    py: Python<'_>,
    value: &Bound<'_, PyAny>,
    context: &str,
) -> Result<SifrInt, PythonError> {
    let integer = value
        .cast::<PyInt>()
        .map_err(|_| conversion_error("expected Python int", context))?;
    let text = integer
        .str()
        .and_then(|value| value.to_str().map(str::to_string))
        .map_err(|error| PythonError::from_pyerr(py, error, "conversion", context))?;
    SifrInt::parse_decimal(&text, DEFAULT_MAX_INTEGER_DIGITS).map_err(|error| {
        conversion_error(
            format!("Python int cannot cross the exact-integer boundary: {error}"),
            context,
        )
    })
}
