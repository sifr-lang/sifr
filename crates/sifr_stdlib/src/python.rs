#[must_use]
pub const fn feature_name() -> &'static str {
    "python"
}

use sifr_runtime::{
    interop::{IndexMap, SifrIntBridge},
    python,
};
use std::collections::HashMap;

type ObjectRaw = (i64, i64);

const fn object_raw(raw: (i64, i64)) -> ObjectRaw {
    raw
}

fn object_handle(handle: SifrIntBridge, token: SifrIntBridge) -> ObjectRaw {
    (handle.to_i64_saturating(), token.to_i64_saturating())
}

macro_rules! copy_sequence_helpers {
    ($list_name:ident, $tuple_name:ident, $runtime_list:ident, $runtime_tuple:ident, $output:ty, $map:expr) => {
        pub fn $list_name(
            handle: SifrIntBridge,
            token: SifrIntBridge,
        ) -> Result<Vec<$output>, python::PythonError> {
            python::$runtime_list(object_handle(handle, token)).map($map)
        }

        pub fn $tuple_name(
            handle: SifrIntBridge,
            token: SifrIntBridge,
        ) -> Result<Vec<$output>, python::PythonError> {
            python::$runtime_tuple(object_handle(handle, token)).map($map)
        }
    };
}

macro_rules! copy_dict_helper {
    ($name:ident, $runtime_name:ident, $output:ty, $map:expr) => {
        pub fn $name(
            handle: SifrIntBridge,
            token: SifrIntBridge,
        ) -> Result<IndexMap<String, $output>, python::PythonError> {
            python::$runtime_name(object_handle(handle, token)).map($map)
        }
    };
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

pub fn py_call(
    handle: SifrIntBridge,
    token: SifrIntBridge,
    args: &[ObjectRaw],
    kwargs_keys: &[String],
    kwargs_values: &[ObjectRaw],
) -> Result<ObjectRaw, python::PythonError> {
    let kwargs = keyed_object_handles(
        kwargs_keys,
        kwargs_values,
        "Python call received mismatched keyword key/value counts",
    )?;
    python::call_object(object_handle(handle, token), args, &kwargs).map(object_raw)
}

pub fn py_call_attr(
    handle: SifrIntBridge,
    token: SifrIntBridge,
    name: &str,
    args: &[ObjectRaw],
    kwargs_keys: &[String],
    kwargs_values: &[ObjectRaw],
) -> Result<ObjectRaw, python::PythonError> {
    let kwargs = keyed_object_handles(
        kwargs_keys,
        kwargs_values,
        "Python attribute call received mismatched keyword key/value counts",
    )?;
    python::call_attr(object_handle(handle, token), name, args, &kwargs).map(object_raw)
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

pub fn py_from_list(values: &[ObjectRaw]) -> Result<ObjectRaw, python::PythonError> {
    python::from_list(values).map(object_raw)
}

pub fn py_from_tuple(values: &[ObjectRaw]) -> Result<ObjectRaw, python::PythonError> {
    python::from_tuple(values).map(object_raw)
}

pub fn py_from_dict_str(
    keys: &[String],
    values: &[ObjectRaw],
) -> Result<ObjectRaw, python::PythonError> {
    let keyed = keyed_object_handles(
        keys,
        values,
        "Python keyed object constructor received mismatched key/value counts",
    )?;
    python::from_dict_str(&keyed).map(object_raw)
}

pub fn py_from_record(
    keys: &[String],
    values: &[ObjectRaw],
) -> Result<ObjectRaw, python::PythonError> {
    let keyed = keyed_object_handles(
        keys,
        values,
        "Python keyed object constructor received mismatched key/value counts",
    )?;
    python::from_record(&keyed).map(object_raw)
}

copy_sequence_helpers!(
    py_copy_list_bool,
    py_copy_tuple_bool,
    copy_list_bool,
    copy_tuple_bool,
    bool,
    |values| values
);
copy_sequence_helpers!(
    py_copy_list_int,
    py_copy_tuple_int,
    copy_list_int,
    copy_tuple_int,
    SifrIntBridge,
    int_vec_to_bridge
);
copy_sequence_helpers!(
    py_copy_list_i32,
    py_copy_tuple_i32,
    copy_list_i32,
    copy_tuple_i32,
    i32,
    |values| values
);
copy_sequence_helpers!(
    py_copy_list_u8,
    py_copy_tuple_u8,
    copy_list_u8,
    copy_tuple_u8,
    u8,
    |values| values
);
copy_sequence_helpers!(
    py_copy_list_float,
    py_copy_tuple_float,
    copy_list_float,
    copy_tuple_float,
    f64,
    |values| values
);
copy_sequence_helpers!(
    py_copy_list_str,
    py_copy_tuple_str,
    copy_list_str,
    copy_tuple_str,
    String,
    |values| values
);
copy_sequence_helpers!(
    py_copy_list_bytes,
    py_copy_tuple_bytes,
    copy_list_bytes,
    copy_tuple_bytes,
    Vec<u8>,
    |values| values
);

copy_dict_helper!(
    py_copy_dict_str_bool,
    copy_dict_str_bool,
    bool,
    index_map_from_hash
);
copy_dict_helper!(
    py_copy_dict_str_int,
    copy_dict_str_int,
    SifrIntBridge,
    int_dict_to_bridge
);
copy_dict_helper!(
    py_copy_dict_str_i32,
    copy_dict_str_i32,
    i32,
    index_map_from_hash
);
copy_dict_helper!(
    py_copy_dict_str_u8,
    copy_dict_str_u8,
    u8,
    index_map_from_hash
);
copy_dict_helper!(
    py_copy_dict_str_float,
    copy_dict_str_float,
    f64,
    index_map_from_hash
);
copy_dict_helper!(
    py_copy_dict_str_str,
    copy_dict_str_str,
    String,
    index_map_from_hash
);
copy_dict_helper!(
    py_copy_dict_str_bytes,
    copy_dict_str_bytes,
    Vec<u8>,
    index_map_from_hash
);

pub fn py_copy_record_fields(
    handle: SifrIntBridge,
    token: SifrIntBridge,
    fields: &[String],
) -> Result<Vec<ObjectRaw>, python::PythonError> {
    let field_refs = fields.iter().map(String::as_str).collect::<Vec<_>>();
    python::copy_record_fields(object_handle(handle, token), &field_refs).map(|values| {
        values
            .into_iter()
            .map(|(_field, object)| object_raw(object))
            .collect()
    })
}

fn int_vec_to_bridge(values: Vec<i64>) -> Vec<SifrIntBridge> {
    values.into_iter().map(SifrIntBridge::from).collect()
}

fn int_dict_to_bridge(values: HashMap<String, i64>) -> IndexMap<String, SifrIntBridge> {
    values
        .into_iter()
        .map(|(key, value)| (key, SifrIntBridge::from(value)))
        .collect()
}

fn index_map_from_hash<T>(values: HashMap<String, T>) -> IndexMap<String, T> {
    values.into_iter().collect()
}

fn keyed_object_handles<'a>(
    keys: &'a [String],
    values: &[ObjectRaw],
    mismatch_message: &str,
) -> Result<Vec<(&'a str, ObjectRaw)>, python::PythonError> {
    if keys.len() != values.len() {
        return Err(python::PythonError {
            message: mismatch_message.to_string(),
            kind: "invalid_argument".to_string(),
            exception_type: String::new(),
            traceback: String::new(),
            context: String::new(),
        });
    }
    Ok(keys
        .iter()
        .zip(values.iter().copied())
        .map(|(key, value)| (key.as_str(), value))
        .collect())
}
