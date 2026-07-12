mod object_bridge;
pub use object_bridge::*;

use sifr_runtime::{
    interop::{Handle, SifrIntBridge},
    python,
};

type ResourceIdentity = Handle<python::PythonResourceIdentity>;
type CallbackRaw = (ResourceIdentity, PythonObject, String);
type BufferRaw = (ResourceIdentity, i64, i64, bool, i64, bool, bool, String);
type ArrowRaw = (ResourceIdentity, String, String, String, bool);
type DlpackRaw = (
    ResourceIdentity,
    i64,
    i64,
    i64,
    String,
    i64,
    i64,
    i64,
    i64,
    bool,
    bool,
);

pub type PythonError = python::PythonError;

fn resource_value(
    identity: &ResourceIdentity,
) -> Result<&python::PythonResourceIdentity, python::PythonError> {
    identity.inner_ref().map_err(|error| {
        python::PythonError::without_replay(
            "resource",
            "SifrPythonResourceIdentityError",
            error.to_string(),
            "",
            "sealed Python resource identity",
        )
    })
}

fn take_resource(
    identity: ResourceIdentity,
) -> Result<python::PythonResourceIdentity, python::PythonError> {
    identity.into_inner().map_err(|error| {
        python::PythonError::without_replay(
            "resource",
            "SifrPythonResourceIdentityError",
            error.to_string(),
            "",
            "sealed Python resource identity",
        )
    })
}

pub fn py_resource_diagnostics() -> Result<(bool, i64, i64), python::PythonError> {
    python::resource_diagnostics().map(|diagnostics| {
        (
            diagnostics.initialized,
            diagnostics.live_objects,
            diagnostics.leaked_objects,
        )
    })
}

pub fn py_buffer_u8(
    object: &PythonObject,
    require_writable: bool,
) -> Result<BufferRaw, python::PythonError> {
    let metadata = python::buffer_u8(object_bridge::object_value(object)?, require_writable)?;
    Ok(buffer_raw(metadata))
}

pub fn py_buffer_shape(
    identity: &ResourceIdentity,
) -> Result<Vec<SifrIntBridge>, python::PythonError> {
    python::buffer_shape(resource_value(identity)?.buffer_key()?).map(int_vec_to_bridge)
}

pub fn py_buffer_strides(
    identity: &ResourceIdentity,
) -> Result<Vec<SifrIntBridge>, python::PythonError> {
    python::buffer_strides(resource_value(identity)?.buffer_key()?).map(int_vec_to_bridge)
}

pub fn py_buffer_suboffsets(
    identity: &ResourceIdentity,
) -> Result<Vec<SifrIntBridge>, python::PythonError> {
    python::buffer_suboffsets(resource_value(identity)?.buffer_key()?).map(int_vec_to_bridge)
}

pub fn py_copy_buffer_u8(identity: &ResourceIdentity) -> Result<Vec<u8>, python::PythonError> {
    python::copy_buffer_u8(resource_value(identity)?.buffer_key()?)
}

pub fn py_release_buffer(identity: ResourceIdentity) -> Result<(), python::PythonError> {
    take_resource(identity)?.close()
}

pub fn py_arrow_array(object: &PythonObject) -> Result<ArrowRaw, python::PythonError> {
    let metadata = python::arrow_array(object_bridge::object_value(object)?)?;
    Ok(arrow_raw(metadata))
}

pub fn py_arrow_stream(object: &PythonObject) -> Result<ArrowRaw, python::PythonError> {
    let metadata = python::arrow_stream(object_bridge::object_value(object)?)?;
    Ok(arrow_raw(metadata))
}

pub fn py_arrow_schema(object: &PythonObject) -> Result<ArrowRaw, python::PythonError> {
    let metadata = python::arrow_schema(object_bridge::object_value(object)?)?;
    Ok(arrow_raw(metadata))
}

pub fn py_arrow_capsule_names(
    identity: &ResourceIdentity,
) -> Result<Vec<String>, python::PythonError> {
    python::arrow_capsule_names(resource_value(identity)?.arrow_key()?)
}

pub fn py_release_arrow(identity: ResourceIdentity) -> Result<(), python::PythonError> {
    take_resource(identity)?.close()
}

pub fn py_dlpack_tensor(object: &PythonObject) -> Result<DlpackRaw, python::PythonError> {
    let metadata = python::dlpack_tensor(object_bridge::object_value(object)?)?;
    Ok(dlpack_raw(metadata))
}

pub fn py_dlpack_shape(
    identity: &ResourceIdentity,
) -> Result<Vec<SifrIntBridge>, python::PythonError> {
    python::dlpack_shape(resource_value(identity)?.dlpack_key()?).map(int_vec_to_bridge)
}

pub fn py_dlpack_strides(
    identity: &ResourceIdentity,
) -> Result<Vec<SifrIntBridge>, python::PythonError> {
    python::dlpack_strides(resource_value(identity)?.dlpack_key()?).map(int_vec_to_bridge)
}

pub fn py_release_dlpack(identity: ResourceIdentity) -> Result<(), python::PythonError> {
    take_resource(identity)?.close()
}

pub fn py_enter_context(object: &PythonObject) -> Result<PythonObject, python::PythonError> {
    python::enter_context(object_bridge::object_value(object)?)
        .map(sifr_runtime::interop::Handle::new)
}

pub fn py_exit_context(object: &PythonObject) -> Result<(), python::PythonError> {
    python::exit_context(object_bridge::object_value(object)?)
}

pub fn py_exit_context_with_error(
    object: &PythonObject,
    kind: &str,
    exception_type: &str,
    message: &str,
    traceback: &str,
    context: &str,
) -> Result<(), python::PythonError> {
    python::exit_context_with_error(
        object_bridge::object_value(object)?,
        kind,
        exception_type,
        message,
        traceback,
        context,
    )
}

pub fn py_run_coroutine_blocking(
    object: &PythonObject,
) -> Result<PythonObject, python::PythonError> {
    python::run_coroutine_blocking(object_bridge::object_value(object)?)
        .map(sifr_runtime::interop::Handle::new)
}

pub fn py_local_callback<F>(callback: F) -> Result<CallbackRaw, python::PythonError>
where
    F: Fn(PythonObject) -> Result<PythonObject, python::PythonError> + Send + Sync + 'static,
{
    python::local_callback(move |object| {
        callback(sifr_runtime::interop::Handle::new(object)).and_then(object_bridge::take_object)
    })
    .map(callback_raw)
}

pub fn py_threadsafe_callback<F>(callback: F) -> Result<CallbackRaw, python::PythonError>
where
    F: Fn(PythonObject) -> Result<PythonObject, python::PythonError> + Send + Sync + 'static,
{
    python::threadsafe_callback(move |object| {
        callback(sifr_runtime::interop::Handle::new(object)).and_then(object_bridge::take_object)
    })
    .map(callback_raw)
}

pub fn py_local_callback_echo() -> Result<CallbackRaw, python::PythonError> {
    python::local_callback_echo().map(callback_raw)
}

pub fn py_threadsafe_callback_echo() -> Result<CallbackRaw, python::PythonError> {
    python::threadsafe_callback_echo().map(callback_raw)
}

pub fn py_close_callback(identity: ResourceIdentity) -> Result<(), python::PythonError> {
    take_resource(identity)?.close()
}

fn callback_raw(metadata: python::PythonCallbackMetadata) -> CallbackRaw {
    (
        Handle::new(python::PythonResourceIdentity::callback((
            metadata.handle,
            metadata.token,
        ))),
        sifr_runtime::interop::Handle::new(metadata.object),
        metadata.kind,
    )
}

fn buffer_raw(metadata: python::PythonBufferMetadata) -> BufferRaw {
    (
        Handle::new(python::PythonResourceIdentity::buffer((
            metadata.handle,
            metadata.token,
        ))),
        metadata.len_bytes,
        metadata.item_size,
        metadata.readonly,
        metadata.dimensions,
        metadata.c_contiguous,
        metadata.f_contiguous,
        metadata.format,
    )
}

fn arrow_raw(metadata: python::PythonArrowCapsuleMetadata) -> ArrowRaw {
    (
        Handle::new(python::PythonResourceIdentity::arrow((
            metadata.handle,
            metadata.token,
        ))),
        metadata.kind,
        metadata.producer_module,
        metadata.producer_type,
        metadata.copy_possible,
    )
}

fn dlpack_raw(metadata: python::PythonDlpackTensorMetadata) -> DlpackRaw {
    (
        Handle::new(python::PythonResourceIdentity::dlpack((
            metadata.handle,
            metadata.token,
        ))),
        metadata.dtype_code,
        metadata.dtype_bits,
        metadata.dtype_lanes,
        metadata.dtype,
        metadata.device_type,
        metadata.device_id,
        metadata.dimensions,
        metadata.byte_offset,
        metadata.has_deleter,
        metadata.stream_sync_required,
    )
}

fn int_vec_to_bridge(values: Vec<i64>) -> Vec<SifrIntBridge> {
    values.into_iter().map(SifrIntBridge::from).collect()
}
