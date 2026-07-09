#[must_use]
pub const fn feature_name() -> &'static str {
    "python"
}

use sifr_runtime::{interop::SifrIntBridge, python};

type ObjectRaw = (i64, i64);

const fn object_raw(raw: (i64, i64)) -> ObjectRaw {
    raw
}

fn object_handle(handle: SifrIntBridge, token: SifrIntBridge) -> ObjectRaw {
    (handle.to_i64_saturating(), token.to_i64_saturating())
}

pub fn py_from_none() -> Result<ObjectRaw, python::PythonError> {
    python::from_none().map(object_raw)
}

pub fn py_from_bool(value: bool) -> Result<ObjectRaw, python::PythonError> {
    python::from_bool(value).map(object_raw)
}

pub fn py_from_int(value: SifrIntBridge) -> Result<ObjectRaw, python::PythonError> {
    python::from_int(value.to_i64_saturating()).map(object_raw)
}

pub fn py_from_float(value: f64) -> Result<ObjectRaw, python::PythonError> {
    python::from_float(value).map(object_raw)
}

pub fn py_from_str(value: &str) -> Result<ObjectRaw, python::PythonError> {
    python::from_str(value).map(object_raw)
}

pub fn py_from_bytes(value: &[u8]) -> Result<ObjectRaw, python::PythonError> {
    python::from_bytes(value).map(object_raw)
}

pub fn py_to_none(handle: SifrIntBridge, token: SifrIntBridge) -> Result<(), python::PythonError> {
    python::to_none(object_handle(handle, token))
}

pub fn py_to_bool(
    handle: SifrIntBridge,
    token: SifrIntBridge,
) -> Result<bool, python::PythonError> {
    python::to_bool(object_handle(handle, token))
}

pub fn py_to_int(
    handle: SifrIntBridge,
    token: SifrIntBridge,
) -> Result<SifrIntBridge, python::PythonError> {
    python::to_int(object_handle(handle, token)).map(SifrIntBridge::from)
}

pub fn py_to_i8(handle: SifrIntBridge, token: SifrIntBridge) -> Result<i8, python::PythonError> {
    python::to_i8(object_handle(handle, token))
}

pub fn py_to_i16(handle: SifrIntBridge, token: SifrIntBridge) -> Result<i16, python::PythonError> {
    python::to_i16(object_handle(handle, token))
}

pub fn py_to_i32(handle: SifrIntBridge, token: SifrIntBridge) -> Result<i32, python::PythonError> {
    python::to_i32(object_handle(handle, token))
}

pub fn py_to_i64(handle: SifrIntBridge, token: SifrIntBridge) -> Result<i64, python::PythonError> {
    python::to_i64(object_handle(handle, token))
}

pub fn py_to_u8(handle: SifrIntBridge, token: SifrIntBridge) -> Result<u8, python::PythonError> {
    python::to_u8(object_handle(handle, token))
}

pub fn py_to_u16(handle: SifrIntBridge, token: SifrIntBridge) -> Result<u16, python::PythonError> {
    python::to_u16(object_handle(handle, token))
}

pub fn py_to_u32(handle: SifrIntBridge, token: SifrIntBridge) -> Result<u32, python::PythonError> {
    python::to_u32(object_handle(handle, token))
}

pub fn py_to_u64(handle: SifrIntBridge, token: SifrIntBridge) -> Result<u64, python::PythonError> {
    python::to_u64(object_handle(handle, token))
}

pub fn py_to_isize(
    handle: SifrIntBridge,
    token: SifrIntBridge,
) -> Result<isize, python::PythonError> {
    python::to_isize(object_handle(handle, token))
}

pub fn py_to_usize(
    handle: SifrIntBridge,
    token: SifrIntBridge,
) -> Result<usize, python::PythonError> {
    python::to_usize(object_handle(handle, token))
}

pub fn py_to_float(
    handle: SifrIntBridge,
    token: SifrIntBridge,
) -> Result<f64, python::PythonError> {
    python::to_float(object_handle(handle, token))
}

pub fn py_to_str(
    handle: SifrIntBridge,
    token: SifrIntBridge,
) -> Result<String, python::PythonError> {
    python::to_str(object_handle(handle, token))
}

pub fn py_to_bytes(
    handle: SifrIntBridge,
    token: SifrIntBridge,
) -> Result<Vec<u8>, python::PythonError> {
    python::to_bytes(object_handle(handle, token))
}

pub fn py_import_module(name: &str) -> Result<ObjectRaw, python::PythonError> {
    python::import_module(name).map(object_raw)
}

pub fn py_get_attr(
    handle: SifrIntBridge,
    token: SifrIntBridge,
    name: &str,
) -> Result<ObjectRaw, python::PythonError> {
    python::get_attr(object_handle(handle, token), name).map(object_raw)
}

pub fn py_get_item_str(
    handle: SifrIntBridge,
    token: SifrIntBridge,
    key: &str,
) -> Result<ObjectRaw, python::PythonError> {
    python::get_item_str(object_handle(handle, token), key).map(object_raw)
}

pub fn py_close(handle: SifrIntBridge, token: SifrIntBridge) -> Result<(), python::PythonError> {
    python::close_object(object_handle(handle, token))
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
