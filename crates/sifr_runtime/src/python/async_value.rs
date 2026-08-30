use super::async_context::PythonAsyncExitCause;
use super::foreign_object::ForeignObjectLease;
use super::int_conversion::{extract_sifr_int, int_to_python};
use super::object_ops::{clone_handle, store_object};
use super::{ObjectHandle, PythonError};
use crate::SifrInt;
use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList, PyTuple};

/// Compiler-private owned value transported to and from the asyncio loop.
///
/// Generated glue is the only intended consumer. Python objects are represented
/// by pinned sealed identities and are resolved only by the loop thread.
#[doc(hidden)]
#[derive(Debug)]
pub enum PythonAsyncValue {
    None,
    Bool(bool),
    Int(SifrInt),
    Float(f64),
    Str(String),
    Bytes(Vec<u8>),
    List(Vec<Self>),
    Tuple(Vec<Self>),
    Dict(Vec<(String, Self)>),
    Record(Vec<(String, Self)>),
    Object(PythonAsyncObject),
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub enum PythonAsyncType {
    None,
    Bool,
    Int,
    Float,
    Str,
    Bytes,
    Option(Box<Self>),
    List(Box<Self>),
    Tuple(Vec<Self>),
    Dict(Box<Self>),
    Record(Vec<(String, Self)>),
    Object,
    Opaque(Vec<String>),
}

#[doc(hidden)]
#[derive(Debug)]
pub struct PythonAsyncRequest {
    pub(super) target: PythonAsyncTarget,
    pub(super) args: Vec<PythonAsyncValue>,
    pub(super) kwargs: Vec<(String, PythonAsyncValue)>,
    pub(super) output: PythonAsyncType,
    completion: PythonAsyncCompletion,
    context_exit_cause: Option<PythonAsyncExitCause>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PythonAsyncCompletion {
    ReturnValue,
    SemanticClose,
}

#[derive(Debug)]
pub(super) enum PythonAsyncTarget {
    Function(Vec<String>),
    Method {
        receiver: PythonAsyncObject,
        member: String,
    },
}

#[doc(hidden)]
#[derive(Debug)]
pub struct PythonAsyncObject {
    lease: ForeignObjectLease,
    owner: Option<ObjectHandle>,
}

impl PythonAsyncRequest {
    #[doc(hidden)]
    #[must_use]
    pub fn function(
        target: Vec<String>,
        args: Vec<PythonAsyncValue>,
        kwargs: Vec<(String, PythonAsyncValue)>,
        output: PythonAsyncType,
    ) -> Self {
        Self {
            target: PythonAsyncTarget::Function(target),
            args,
            kwargs,
            output,
            completion: PythonAsyncCompletion::ReturnValue,
            context_exit_cause: None,
        }
    }

    #[doc(hidden)]
    pub fn borrowed_method(
        receiver: &ObjectHandle,
        member: String,
        args: Vec<PythonAsyncValue>,
        kwargs: Vec<(String, PythonAsyncValue)>,
        output: PythonAsyncType,
    ) -> Result<Self, PythonError> {
        Ok(Self {
            target: PythonAsyncTarget::Method {
                receiver: PythonAsyncObject::borrowed(receiver)?,
                member,
            },
            args,
            kwargs,
            output,
            completion: PythonAsyncCompletion::ReturnValue,
            context_exit_cause: None,
        })
    }

    #[doc(hidden)]
    pub fn owned_method(
        receiver: ObjectHandle,
        member: String,
        args: Vec<PythonAsyncValue>,
        kwargs: Vec<(String, PythonAsyncValue)>,
        output: PythonAsyncType,
    ) -> Result<Self, PythonError> {
        Ok(Self {
            target: PythonAsyncTarget::Method {
                receiver: PythonAsyncObject::owned(receiver)?,
                member,
            },
            args,
            kwargs,
            output,
            completion: PythonAsyncCompletion::ReturnValue,
            context_exit_cause: None,
        })
    }

    #[doc(hidden)]
    pub fn semantic_close_method(
        receiver: ObjectHandle,
        member: String,
    ) -> Result<Self, PythonError> {
        Ok(Self {
            target: PythonAsyncTarget::Method {
                receiver: PythonAsyncObject::semantic_close(receiver)?,
                member,
            },
            args: Vec::new(),
            kwargs: Vec::new(),
            output: PythonAsyncType::None,
            completion: PythonAsyncCompletion::SemanticClose,
            context_exit_cause: None,
        })
    }

    pub(super) fn semantic_context_exit_method(
        receiver: ObjectHandle,
        cause: PythonAsyncExitCause,
    ) -> Result<Self, PythonError> {
        Ok(Self {
            target: PythonAsyncTarget::Method {
                receiver: PythonAsyncObject::semantic_close(receiver)?,
                member: "__aexit__".to_string(),
            },
            args: Vec::new(),
            kwargs: Vec::new(),
            output: PythonAsyncType::None,
            completion: PythonAsyncCompletion::SemanticClose,
            context_exit_cause: Some(cause),
        })
    }

    pub(super) fn context_exit_cause(&self) -> Option<&PythonAsyncExitCause> {
        self.context_exit_cause.as_ref()
    }

    pub(super) fn finish_semantic_close(&self, succeeded: bool) {
        if self.completion != PythonAsyncCompletion::SemanticClose {
            return;
        }
        if let PythonAsyncTarget::Method { receiver, .. } = &self.target {
            receiver.finish_semantic_close(succeeded);
        }
    }
}

impl Drop for PythonAsyncRequest {
    fn drop(&mut self) {
        self.finish_semantic_close(false);
    }
}

impl PythonAsyncObject {
    fn borrowed(object: &ObjectHandle) -> Result<Self, PythonError> {
        Ok(Self {
            lease: object.lease().map_err(PythonError::runtime)?,
            owner: None,
        })
    }

    fn owned(object: ObjectHandle) -> Result<Self, PythonError> {
        let lease = object.lease().map_err(PythonError::runtime)?;
        Ok(Self {
            lease,
            owner: Some(object),
        })
    }

    fn semantic_close(object: ObjectHandle) -> Result<Self, PythonError> {
        let lease = object
            .begin_semantic_close()
            .map_err(PythonError::runtime)?;
        Ok(Self {
            lease,
            owner: Some(object),
        })
    }

    fn finish_semantic_close(&self, succeeded: bool) {
        if let Some(owner) = &self.owner {
            owner.finish_semantic_close(succeeded);
        }
    }

    pub(super) fn clone_ref(&self, py: Python<'_>) -> Result<Py<PyAny>, PythonError> {
        self.lease.clone_ref(py).map_err(PythonError::runtime)
    }

    fn into_owner(self) -> Result<ObjectHandle, PythonError> {
        self.owner.ok_or_else(|| {
            conversion_error(
                "borrowed Python identity cannot be used as an owned async result",
                self.lease.boundary_path(),
            )
        })
    }
}

macro_rules! primitive_constructor {
    ($name:ident, $variant:ident, $ty:ty) => {
        #[doc(hidden)]
        pub fn $name(value: $ty) -> Result<PythonAsyncValue, PythonError> {
            Ok(PythonAsyncValue::$variant(value))
        }
    };
}

#[doc(hidden)]
pub fn async_from_none() -> Result<PythonAsyncValue, PythonError> {
    Ok(PythonAsyncValue::None)
}

primitive_constructor!(async_from_bool, Bool, bool);
primitive_constructor!(async_from_float, Float, f64);

#[doc(hidden)]
pub fn async_from_int(value: impl Into<SifrInt>) -> Result<PythonAsyncValue, PythonError> {
    Ok(PythonAsyncValue::Int(value.into()))
}

#[doc(hidden)]
pub fn async_from_str(value: &str) -> Result<PythonAsyncValue, PythonError> {
    Ok(PythonAsyncValue::Str(value.to_string()))
}

#[doc(hidden)]
pub fn async_from_bytes(value: &[u8]) -> Result<PythonAsyncValue, PythonError> {
    Ok(PythonAsyncValue::Bytes(value.to_vec()))
}

#[doc(hidden)]
pub fn async_from_object(value: &ObjectHandle) -> Result<PythonAsyncValue, PythonError> {
    PythonAsyncObject::borrowed(value).map(PythonAsyncValue::Object)
}

#[doc(hidden)]
pub fn async_from_owned_object(value: ObjectHandle) -> Result<PythonAsyncValue, PythonError> {
    PythonAsyncObject::owned(value).map(PythonAsyncValue::Object)
}

#[doc(hidden)]
pub fn async_from_list_results(
    values: Vec<Result<PythonAsyncValue, PythonError>>,
) -> Result<PythonAsyncValue, PythonError> {
    collect_values(values, "list").map(PythonAsyncValue::List)
}

#[doc(hidden)]
pub fn async_from_tuple_results(
    values: Vec<Result<PythonAsyncValue, PythonError>>,
) -> Result<PythonAsyncValue, PythonError> {
    collect_values(values, "tuple").map(PythonAsyncValue::Tuple)
}

#[doc(hidden)]
pub fn async_from_dict_results(
    values: Vec<(String, Result<PythonAsyncValue, PythonError>)>,
) -> Result<PythonAsyncValue, PythonError> {
    collect_named_values(values, "dict").map(PythonAsyncValue::Dict)
}

#[doc(hidden)]
pub fn async_from_record_results(
    values: Vec<(String, Result<PythonAsyncValue, PythonError>)>,
) -> Result<PythonAsyncValue, PythonError> {
    collect_named_values(values, "record").map(PythonAsyncValue::Record)
}

#[doc(hidden)]
pub fn async_value_is_none(value: &PythonAsyncValue) -> bool {
    matches!(value, PythonAsyncValue::None)
}

macro_rules! primitive_extractor {
    ($name:ident, $variant:ident, $ty:ty, $expected:literal) => {
        #[doc(hidden)]
        pub fn $name(value: PythonAsyncValue) -> Result<$ty, PythonError> {
            match value {
                PythonAsyncValue::$variant(value) => Ok(value),
                _ => Err(conversion_error(
                    concat!("expected converted ", $expected),
                    stringify!($name),
                )),
            }
        }
    };
}

#[doc(hidden)]
pub fn async_to_none(value: PythonAsyncValue) -> Result<(), PythonError> {
    match value {
        PythonAsyncValue::None => Ok(()),
        _ => Err(conversion_error("expected converted None", "async_to_none")),
    }
}

primitive_extractor!(async_to_bool, Bool, bool, "bool");
primitive_extractor!(async_to_float, Float, f64, "float");
primitive_extractor!(async_to_str, Str, String, "str");
primitive_extractor!(async_to_bytes, Bytes, Vec<u8>, "bytes");

#[doc(hidden)]
pub fn async_to_int(value: PythonAsyncValue) -> Result<SifrInt, PythonError> {
    match value {
        PythonAsyncValue::Int(value) => Ok(value),
        _ => Err(conversion_error("expected converted int", "async_to_int")),
    }
}

#[doc(hidden)]
pub fn async_list_items(value: PythonAsyncValue) -> Result<Vec<PythonAsyncValue>, PythonError> {
    match value {
        PythonAsyncValue::List(values) => Ok(values),
        _ => Err(conversion_error(
            "expected converted list",
            "async_list_items",
        )),
    }
}

#[doc(hidden)]
pub fn async_tuple_items(value: PythonAsyncValue) -> Result<Vec<PythonAsyncValue>, PythonError> {
    match value {
        PythonAsyncValue::Tuple(values) => Ok(values),
        _ => Err(conversion_error(
            "expected converted tuple",
            "async_tuple_items",
        )),
    }
}

#[doc(hidden)]
pub fn async_dict_items(
    value: PythonAsyncValue,
) -> Result<Vec<(String, PythonAsyncValue)>, PythonError> {
    match value {
        PythonAsyncValue::Dict(values) => Ok(values),
        _ => Err(conversion_error(
            "expected converted dict",
            "async_dict_items",
        )),
    }
}

#[doc(hidden)]
pub fn async_record_field(
    value: &mut PythonAsyncValue,
    field: &str,
) -> Result<PythonAsyncValue, PythonError> {
    let PythonAsyncValue::Record(values) = value else {
        return Err(conversion_error(
            "expected converted record",
            "async_record_field",
        ));
    };
    let Some(index) = values.iter().position(|(name, _)| name == field) else {
        return Err(conversion_error(
            format!("missing converted record field '{field}'"),
            format!(".{field}"),
        ));
    };
    Ok(values.remove(index).1)
}

#[doc(hidden)]
pub fn async_to_object(value: PythonAsyncValue) -> Result<ObjectHandle, PythonError> {
    match value {
        PythonAsyncValue::Object(object) => object.into_owner(),
        _ => Err(conversion_error(
            "expected converted Python object",
            "async_to_object",
        )),
    }
}

pub(super) fn materialize(
    py: Python<'_>,
    value: &PythonAsyncValue,
    context: &str,
) -> Result<Py<PyAny>, PythonError> {
    match value {
        PythonAsyncValue::None => Ok(py.None()),
        PythonAsyncValue::Bool(value) => into_python(py, *value, context),
        PythonAsyncValue::Int(value) => int_to_python(py, value, context),
        PythonAsyncValue::Float(value) => into_python(py, *value, context),
        PythonAsyncValue::Str(value) => into_python(py, value.as_str(), context),
        PythonAsyncValue::Bytes(value) => Ok(PyBytes::new(py, value).into_any().unbind()),
        PythonAsyncValue::List(values) => {
            let values = materialize_values(py, values, context)?;
            PyList::new(py, values.iter())
                .map(|value| value.into_any().unbind())
                .map_err(|error| PythonError::from_pyerr(py, error, "conversion", context))
        }
        PythonAsyncValue::Tuple(values) => {
            let values = materialize_values(py, values, context)?;
            PyTuple::new(py, values.iter())
                .map(|value| value.into_any().unbind())
                .map_err(|error| PythonError::from_pyerr(py, error, "conversion", context))
        }
        PythonAsyncValue::Dict(values) | PythonAsyncValue::Record(values) => {
            let dict = PyDict::new(py);
            for (name, value) in values {
                let value = materialize(py, value, &format!("{context}[{name:?}]"))?;
                dict.set_item(name, value.bind(py))
                    .map_err(|error| PythonError::from_pyerr(py, error, "conversion", context))?;
            }
            Ok(dict.into_any().unbind())
        }
        PythonAsyncValue::Object(object) => object.clone_ref(py),
    }
}

pub(super) fn convert_output(
    py: Python<'_>,
    value: &Bound<'_, PyAny>,
    schema: &PythonAsyncType,
    context: &str,
) -> Result<PythonAsyncValue, PythonError> {
    match schema {
        PythonAsyncType::None => {
            if value.is_none() {
                Ok(PythonAsyncValue::None)
            } else {
                Err(conversion_error("expected Python None", context))
            }
        }
        PythonAsyncType::Bool => extract(py, value, context).map(PythonAsyncValue::Bool),
        PythonAsyncType::Int => extract_sifr_int(py, value, context).map(PythonAsyncValue::Int),
        PythonAsyncType::Float => extract(py, value, context).map(PythonAsyncValue::Float),
        PythonAsyncType::Str => extract(py, value, context).map(PythonAsyncValue::Str),
        PythonAsyncType::Bytes => value
            .cast::<PyBytes>()
            .map(|value| PythonAsyncValue::Bytes(value.as_bytes().to_vec()))
            .map_err(|_| conversion_error("expected Python bytes", context)),
        PythonAsyncType::Option(inner) => {
            if value.is_none() {
                Ok(PythonAsyncValue::None)
            } else {
                convert_output(py, value, inner, context)
            }
        }
        PythonAsyncType::List(inner) => {
            let list = value
                .cast::<PyList>()
                .map_err(|_| conversion_error("expected Python list", context))?;
            let mut converted = Vec::with_capacity(list.len());
            for (index, item) in list.iter().enumerate() {
                converted.push(convert_output(
                    py,
                    &item,
                    inner,
                    &format!("{context}[{index}]"),
                )?);
            }
            Ok(PythonAsyncValue::List(converted))
        }
        PythonAsyncType::Tuple(items) => {
            let tuple = value
                .cast::<PyTuple>()
                .map_err(|_| conversion_error("expected Python tuple", context))?;
            if tuple.len() != items.len() {
                return Err(conversion_error(
                    format!("expected Python tuple of length {}", items.len()),
                    context,
                ));
            }
            let mut converted = Vec::with_capacity(items.len());
            for (index, schema) in items.iter().enumerate() {
                let item = tuple
                    .get_item(index)
                    .map_err(|error| PythonError::from_pyerr(py, error, "conversion", context))?;
                converted.push(convert_output(
                    py,
                    &item,
                    schema,
                    &format!("{context}[{index}]"),
                )?);
            }
            Ok(PythonAsyncValue::Tuple(converted))
        }
        PythonAsyncType::Dict(inner) => {
            let dict = value
                .cast::<PyDict>()
                .map_err(|_| conversion_error("expected Python dict", context))?;
            let mut converted = Vec::with_capacity(dict.len());
            for (key, item) in dict.iter() {
                let key = extract::<String>(py, &key, &format!("{context}.<key>"))?;
                let item = convert_output(py, &item, inner, &format!("{context}[{key:?}]"))?;
                converted.push((key, item));
            }
            Ok(PythonAsyncValue::Dict(converted))
        }
        PythonAsyncType::Record(fields) => {
            let mut converted = Vec::with_capacity(fields.len());
            for (field, schema) in fields {
                let item = value
                    .getattr(field.as_str())
                    .or_else(|_| value.get_item(field.as_str()))
                    .map_err(|_| {
                        conversion_error(
                            format!("missing required record field '{field}'"),
                            format!("{context}.{field}"),
                        )
                    })?;
                converted.push((
                    field.clone(),
                    convert_output(py, &item, schema, &format!("{context}.{field}"))?,
                ));
            }
            Ok(PythonAsyncValue::Record(converted))
        }
        PythonAsyncType::Object => stored_object(value.clone().unbind()),
        PythonAsyncType::Opaque(expected) => {
            let expected_handle = super::object_ops::resolve_target(expected)?;
            let expected_type = clone_handle(py, &expected_handle)?;
            let matches = value
                .is_instance(expected_type.bind(py))
                .map_err(|error| PythonError::from_pyerr(py, error, "conversion", context))?;
            if !matches {
                return Err(conversion_error(
                    format!("expected Python instance of {}", expected.join(".")),
                    context,
                ));
            }
            stored_object(value.clone().unbind())
        }
    }
}

fn stored_object(object: Py<PyAny>) -> Result<PythonAsyncValue, PythonError> {
    let object = store_object(object)?;
    PythonAsyncObject::owned(object).map(PythonAsyncValue::Object)
}

fn collect_values(
    values: Vec<Result<PythonAsyncValue, PythonError>>,
    context: &str,
) -> Result<Vec<PythonAsyncValue>, PythonError> {
    values
        .into_iter()
        .enumerate()
        .map(|(index, value)| value.map_err(|error| at_path(error, format!("{context}[{index}]"))))
        .collect()
}

fn collect_named_values(
    values: Vec<(String, Result<PythonAsyncValue, PythonError>)>,
    context: &str,
) -> Result<Vec<(String, PythonAsyncValue)>, PythonError> {
    values
        .into_iter()
        .map(|(name, value)| {
            value
                .map(|value| (name.clone(), value))
                .map_err(|error| at_path(error, format!("{context}[{name:?}]")))
        })
        .collect()
}

fn materialize_values(
    py: Python<'_>,
    values: &[PythonAsyncValue],
    context: &str,
) -> Result<Vec<Py<PyAny>>, PythonError> {
    values
        .iter()
        .enumerate()
        .map(|(index, value)| materialize(py, value, &format!("{context}[{index}]")))
        .collect()
}

fn into_python<'py, T>(py: Python<'py>, value: T, context: &str) -> Result<Py<PyAny>, PythonError>
where
    T: IntoPyObjectExt<'py>,
{
    value
        .into_py_any(py)
        .map_err(|error| PythonError::from_pyerr(py, error, "conversion", context))
}

fn extract<'a, 'py, T>(
    py: Python<'py>,
    value: &'a Bound<'py, PyAny>,
    context: &str,
) -> Result<T, PythonError>
where
    T: FromPyObject<'a, 'py>,
{
    value
        .extract::<T>()
        .map_err(|error| PythonError::from_pyerr(py, error.into(), "conversion", context))
}

fn at_path(mut error: PythonError, path: String) -> PythonError {
    error.context = if error.context.is_empty() {
        path
    } else {
        format!("{path}.{}", error.context)
    };
    error
}

fn conversion_error(message: impl Into<String>, context: impl Into<String>) -> PythonError {
    PythonError::without_replay(
        "conversion",
        "SifrPythonTypeConversionError",
        message,
        String::new(),
        context,
    )
}
