use super::object_ops::{clone_handle, store_object, store_objects};
use super::{ObjectHandle, PythonError};
use pyo3::exceptions::PyAttributeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};

pub fn from_list_results(
    values: Vec<Result<ObjectHandle, PythonError>>,
) -> Result<ObjectHandle, PythonError> {
    let values = values
        .into_iter()
        .enumerate()
        .map(|(index, value)| at_path(value, format!("[{index}]")))
        .collect::<Result<Vec<_>, _>>()?;
    super::object_ops::from_list(&values)
}

pub fn from_tuple_results(
    values: Vec<Result<ObjectHandle, PythonError>>,
) -> Result<ObjectHandle, PythonError> {
    let values = values
        .into_iter()
        .enumerate()
        .map(|(index, value)| at_path(value, format!("[{index}]")))
        .collect::<Result<Vec<_>, _>>()?;
    super::object_ops::from_tuple(&values)
}

pub fn from_dict_results(
    values: Vec<(String, Result<ObjectHandle, PythonError>)>,
) -> Result<ObjectHandle, PythonError> {
    let values = values
        .into_iter()
        .map(|(key, value)| {
            let value = at_path(value, format!("[{key:?}]"))?;
            Ok((key, value))
        })
        .collect::<Result<Vec<_>, PythonError>>()?;
    let borrowed = values
        .iter()
        .map(|(key, value)| (key.as_str(), value.clone()))
        .collect::<Vec<_>>();
    super::object_ops::from_dict_str(&borrowed)
}

pub fn from_record_results(
    values: Vec<(String, Result<ObjectHandle, PythonError>)>,
) -> Result<ObjectHandle, PythonError> {
    let values = values
        .into_iter()
        .map(|(field, value)| {
            let value = at_path(value, format!(".{field}"))?;
            Ok((field, value))
        })
        .collect::<Result<Vec<_>, PythonError>>()?;
    let borrowed = values
        .iter()
        .map(|(field, value)| (field.as_str(), value.clone()))
        .collect::<Vec<_>>();
    super::object_ops::from_record(&borrowed)
}

pub fn list_items(object: &ObjectHandle) -> Result<Vec<ObjectHandle>, PythonError> {
    let parent_path = object.boundary_path().to_string();
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        let list = object
            .bind(py)
            .cast::<PyList>()
            .map_err(|_| conversion_error("expected Python list", "list"))?;
        store_objects(list.iter().map(Bound::unbind).collect()).map(|items| {
            items
                .into_iter()
                .enumerate()
                .map(|(index, item)| item.with_child_path(format!("{parent_path}[{index}]")))
                .collect()
        })
    })
    .map_err(PythonError::runtime)?
}

pub fn tuple_items(object: &ObjectHandle) -> Result<Vec<ObjectHandle>, PythonError> {
    let parent_path = object.boundary_path().to_string();
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        let tuple = object
            .bind(py)
            .cast::<PyTuple>()
            .map_err(|_| conversion_error("expected Python tuple", "tuple"))?;
        store_objects(tuple.iter().map(Bound::unbind).collect()).map(|items| {
            items
                .into_iter()
                .enumerate()
                .map(|(index, item)| item.with_child_path(format!("{parent_path}[{index}]")))
                .collect()
        })
    })
    .map_err(PythonError::runtime)?
}

pub fn dict_str_items(object: &ObjectHandle) -> Result<Vec<(String, ObjectHandle)>, PythonError> {
    let parent_path = object.boundary_path().to_string();
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        let dict = object
            .bind(py)
            .cast::<PyDict>()
            .map_err(|_| conversion_error("expected Python dict", "dict"))?;
        let mut names = Vec::with_capacity(dict.len());
        let mut objects = Vec::with_capacity(dict.len());
        for (key, value) in dict.iter() {
            names.push(
                key.extract::<String>()
                    .map_err(|_| conversion_error("expected string dictionary key", "dict key"))?,
            );
            objects.push(value.unbind());
        }
        let handles = store_objects(objects)?;
        Ok(names
            .into_iter()
            .zip(handles)
            .map(|(name, value)| {
                let value = value.with_child_path(format!("{parent_path}[{name:?}]"));
                (name, value)
            })
            .collect())
    })
    .map_err(PythonError::runtime)?
}

pub fn record_field(
    object: &ObjectHandle,
    field: impl AsRef<str>,
) -> Result<ObjectHandle, PythonError> {
    let field = field.as_ref().to_string();
    let path = format!("{}.{field}", object.boundary_path());
    super::attach(|py| {
        let object = clone_handle(py, object)?;
        let object = object.bind(py);
        let value = match object.getattr(field.as_str()) {
            Ok(value) => value,
            Err(error) if error.is_instance_of::<PyAttributeError>(py) => {
                object.get_item(field.as_str()).map_err(|_| {
                    conversion_error(format!("missing required record field '{field}'"), &path)
                })?
            }
            Err(error) => {
                return Err(PythonError::from_pyerr(py, error, "attribute", &path));
            }
        };
        store_object(value.unbind()).map(|value| value.with_child_path(path))
    })
    .map_err(PythonError::runtime)?
}

pub fn object_is_none(object: &ObjectHandle) -> Result<bool, PythonError> {
    super::attach(|py| Ok(clone_handle(py, object)?.bind(py).is_none()))
        .map_err(PythonError::runtime)?
}

pub fn at_path<T>(result: Result<T, PythonError>, path: String) -> Result<T, PythonError> {
    result.map_err(|mut error| {
        error.context = if error.context.is_empty() {
            path
        } else if error.context.starts_with('[') || error.context.starts_with('.') {
            format!("{path}{}", error.context)
        } else {
            format!("{path}.{}", error.context)
        };
        error
    })
}

fn conversion_error(message: impl Into<String>, context: impl Into<String>) -> PythonError {
    PythonError {
        kind: "conversion".to_string(),
        exception_type: "TypeError".to_string(),
        message: message.into(),
        traceback: String::new(),
        context: context.into(),
        replay: None,
    }
}
