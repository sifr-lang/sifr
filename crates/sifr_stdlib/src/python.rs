#[must_use]
pub const fn feature_name() -> &'static str {
    "python"
}

use sifr_runtime::{interop::SifrIntBridge, python};

type ObjectRaw = (i64, i64);

const fn object_raw(raw: (i64, i64)) -> ObjectRaw {
    raw
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
