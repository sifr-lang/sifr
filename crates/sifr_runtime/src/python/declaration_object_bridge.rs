//! Compiler-private conversion bridge for the sealed declaration `Object`.

use super::{
    ForeignObject, PythonAsyncValue, PythonError, async_from_object, async_to_object,
    temporary_argument_handle,
};
use crate::interop::{Handle, HandleStateError};

pub fn __sifr_declaration_object_argument(
    object: &Handle<ForeignObject>,
) -> Result<ForeignObject, PythonError> {
    temporary_argument_handle(object.inner_ref().map_err(handle_error)?)
}

pub fn __sifr_declaration_object_result(object: ForeignObject) -> Handle<ForeignObject> {
    Handle::new(object)
}

pub fn __sifr_declaration_async_from_object(
    object: &Handle<ForeignObject>,
) -> Result<PythonAsyncValue, PythonError> {
    async_from_object(object.inner_ref().map_err(handle_error)?)
}

pub fn __sifr_declaration_async_to_object(
    value: PythonAsyncValue,
) -> Result<Handle<ForeignObject>, PythonError> {
    async_to_object(value).map(Handle::new)
}

fn handle_error(error: HandleStateError) -> PythonError {
    PythonError::without_replay(
        "resource",
        "SifrPythonClosedObject",
        error.to_string(),
        "",
        "sealed Python object identity",
    )
}
