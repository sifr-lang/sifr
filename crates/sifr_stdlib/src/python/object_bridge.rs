use super::PythonError;
use sifr_runtime::{
    interop::{Handle, HandleStateError, IndexMap, SifrIntBridge},
    python,
};
use std::collections::HashMap;

pub type PythonObject = Handle<python::ForeignObject>;

pub(super) fn object_value(object: &PythonObject) -> Result<&python::ForeignObject, PythonError> {
    object.inner_ref().map_err(handle_error)
}

pub(super) fn take_object(object: PythonObject) -> Result<python::ForeignObject, PythonError> {
    object.into_inner().map_err(handle_error)
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

fn wrap(result: Result<python::ForeignObject, PythonError>) -> Result<PythonObject, PythonError> {
    result.map(Handle::new)
}

fn object_values(values: &[PythonObject]) -> Result<Vec<python::ForeignObject>, PythonError> {
    values
        .iter()
        .map(|object| object_value(object).cloned())
        .collect()
}

pub fn py_from_none() -> Result<PythonObject, PythonError> {
    wrap(python::from_none())
}

pub fn py_from_bool(value: bool) -> Result<PythonObject, PythonError> {
    wrap(python::from_bool(value))
}

pub fn py_from_int(value: SifrIntBridge) -> Result<PythonObject, PythonError> {
    wrap(python::from_int(value.into_sifr_int()))
}

pub fn py_from_float(value: f64) -> Result<PythonObject, PythonError> {
    wrap(python::from_float(value))
}

pub fn py_from_str(value: &str) -> Result<PythonObject, PythonError> {
    wrap(python::from_str(value))
}

pub fn py_from_bytes(value: &[u8]) -> Result<PythonObject, PythonError> {
    wrap(python::from_bytes(value))
}

macro_rules! scalar_copy {
    ($name:ident, $runtime:ident, $output:ty, $map:expr) => {
        pub fn $name(object: &PythonObject) -> Result<$output, PythonError> {
            python::$runtime(object_value(object)?).map($map)
        }
    };
}

scalar_copy!(py_to_none, to_none, (), |value| value);
scalar_copy!(py_to_bool, to_bool, bool, |value| value);
scalar_copy!(py_to_int, to_int, SifrIntBridge, SifrIntBridge::from);
scalar_copy!(py_to_i8, to_i8, i8, |value| value);
scalar_copy!(py_to_i16, to_i16, i16, |value| value);
scalar_copy!(py_to_i32, to_i32, i32, |value| value);
scalar_copy!(py_to_i64, to_i64, i64, |value| value);
scalar_copy!(py_to_u8, to_u8, u8, |value| value);
scalar_copy!(py_to_u16, to_u16, u16, |value| value);
scalar_copy!(py_to_u32, to_u32, u32, |value| value);
scalar_copy!(py_to_u64, to_u64, u64, |value| value);
scalar_copy!(py_to_isize, to_isize, isize, |value| value);
scalar_copy!(py_to_usize, to_usize, usize, |value| value);
scalar_copy!(py_to_float, to_float, f64, |value| value);
scalar_copy!(py_to_str, to_str, String, |value| value);
scalar_copy!(py_to_bytes, to_bytes, Vec<u8>, |value| value);

pub fn py_import_module(name: &str) -> Result<PythonObject, PythonError> {
    wrap(python::import_module(name))
}

pub fn py_get_attr(object: &PythonObject, name: &str) -> Result<PythonObject, PythonError> {
    wrap(python::get_attr(object_value(object)?, name))
}

pub fn py_get_item_str(object: &PythonObject, key: &str) -> Result<PythonObject, PythonError> {
    wrap(python::get_item_str(object_value(object)?, key))
}

pub fn py_call(
    object: &PythonObject,
    args: &[PythonObject],
    kwargs_keys: &[String],
    kwargs_values: &[PythonObject],
) -> Result<PythonObject, PythonError> {
    let values = object_values(kwargs_values)?;
    let kwargs = keyed_objects(kwargs_keys, &values)?;
    wrap(python::call_object(
        object_value(object)?,
        &object_values(args)?,
        &kwargs,
    ))
}

pub fn py_call_attr(
    object: &PythonObject,
    name: &str,
    args: &[PythonObject],
    kwargs_keys: &[String],
    kwargs_values: &[PythonObject],
) -> Result<PythonObject, PythonError> {
    let values = object_values(kwargs_values)?;
    let kwargs = keyed_objects(kwargs_keys, &values)?;
    wrap(python::call_attr(
        object_value(object)?,
        name,
        &object_values(args)?,
        &kwargs,
    ))
}

pub fn py_call_keyed(
    object: &PythonObject,
    args: &[PythonObject],
    kwargs: &[(String, PythonObject)],
) -> Result<PythonObject, PythonError> {
    let keys = kwargs
        .iter()
        .map(|(key, _value)| key.clone())
        .collect::<Vec<_>>();
    let values = kwargs
        .iter()
        .map(|(_key, value)| object_value(value).cloned())
        .collect::<Result<Vec<_>, _>>()?;
    wrap(python::call_object(
        object_value(object)?,
        &object_values(args)?,
        &keyed_objects(&keys, &values)?,
    ))
}

pub fn py_call_attr_keyed(
    object: &PythonObject,
    name: &str,
    args: &[PythonObject],
    kwargs: &[(String, PythonObject)],
) -> Result<PythonObject, PythonError> {
    let keys = kwargs
        .iter()
        .map(|(key, _value)| key.clone())
        .collect::<Vec<_>>();
    let values = kwargs
        .iter()
        .map(|(_key, value)| object_value(value).cloned())
        .collect::<Result<Vec<_>, _>>()?;
    wrap(python::call_attr(
        object_value(object)?,
        name,
        &object_values(args)?,
        &keyed_objects(&keys, &values)?,
    ))
}

pub fn py_close(object: PythonObject) -> Result<(), PythonError> {
    drop(object);
    Ok(())
}

pub fn py_from_list(values: &[PythonObject]) -> Result<PythonObject, PythonError> {
    wrap(python::from_list(&object_values(values)?))
}

pub fn py_from_tuple(values: &[PythonObject]) -> Result<PythonObject, PythonError> {
    wrap(python::from_tuple(&object_values(values)?))
}

pub fn py_from_dict_str(
    keys: &[String],
    values: &[PythonObject],
) -> Result<PythonObject, PythonError> {
    let values = object_values(values)?;
    wrap(python::from_dict_str(&keyed_objects(keys, &values)?))
}

pub fn py_from_record(
    keys: &[String],
    values: &[PythonObject],
) -> Result<PythonObject, PythonError> {
    let values = object_values(values)?;
    wrap(python::from_record(&keyed_objects(keys, &values)?))
}

macro_rules! sequence_copy {
    ($list:ident, $tuple:ident, $runtime_list:ident, $runtime_tuple:ident, $output:ty, $map:expr) => {
        pub fn $list(object: &PythonObject) -> Result<Vec<$output>, PythonError> {
            python::$runtime_list(object_value(object)?).map($map)
        }
        pub fn $tuple(object: &PythonObject) -> Result<Vec<$output>, PythonError> {
            python::$runtime_tuple(object_value(object)?).map($map)
        }
    };
}

sequence_copy!(
    py_copy_list_bool,
    py_copy_tuple_bool,
    copy_list_bool,
    copy_tuple_bool,
    bool,
    |v| v
);
sequence_copy!(
    py_copy_list_int,
    py_copy_tuple_int,
    copy_list_int,
    copy_tuple_int,
    SifrIntBridge,
    |v| v.into_iter().map(SifrIntBridge::from).collect()
);
sequence_copy!(
    py_copy_list_i32,
    py_copy_tuple_i32,
    copy_list_i32,
    copy_tuple_i32,
    i32,
    |v| v
);
sequence_copy!(
    py_copy_list_u8,
    py_copy_tuple_u8,
    copy_list_u8,
    copy_tuple_u8,
    u8,
    |v| v
);
sequence_copy!(
    py_copy_list_float,
    py_copy_tuple_float,
    copy_list_float,
    copy_tuple_float,
    f64,
    |v| v
);
sequence_copy!(
    py_copy_list_str,
    py_copy_tuple_str,
    copy_list_str,
    copy_tuple_str,
    String,
    |v| v
);
sequence_copy!(
    py_copy_list_bytes,
    py_copy_tuple_bytes,
    copy_list_bytes,
    copy_tuple_bytes,
    Vec<u8>,
    |v| v
);

macro_rules! dict_copy {
    ($name:ident, $runtime:ident, $output:ty, $map:expr) => {
        pub fn $name(object: &PythonObject) -> Result<IndexMap<String, $output>, PythonError> {
            python::$runtime(object_value(object)?).map($map)
        }
    };
}

dict_copy!(
    py_copy_dict_str_bool,
    copy_dict_str_bool,
    bool,
    index_map_from_hash
);
dict_copy!(
    py_copy_dict_str_int,
    copy_dict_str_int,
    SifrIntBridge,
    |values| values
        .into_iter()
        .map(|(key, value)| (key, SifrIntBridge::from(value)))
        .collect()
);
dict_copy!(
    py_copy_dict_str_i32,
    copy_dict_str_i32,
    i32,
    index_map_from_hash
);
dict_copy!(
    py_copy_dict_str_u8,
    copy_dict_str_u8,
    u8,
    index_map_from_hash
);
dict_copy!(
    py_copy_dict_str_float,
    copy_dict_str_float,
    f64,
    index_map_from_hash
);
dict_copy!(
    py_copy_dict_str_str,
    copy_dict_str_str,
    String,
    index_map_from_hash
);
dict_copy!(
    py_copy_dict_str_bytes,
    copy_dict_str_bytes,
    Vec<u8>,
    index_map_from_hash
);

pub fn py_copy_record_fields(
    object: &PythonObject,
    fields: &[String],
) -> Result<Vec<PythonObject>, PythonError> {
    let fields = fields.iter().map(String::as_str).collect::<Vec<_>>();
    python::copy_record_fields(object_value(object)?, &fields).map(|values| {
        values
            .into_iter()
            .map(|(_name, value)| Handle::new(value))
            .collect()
    })
}

fn keyed_objects<'a>(
    keys: &'a [String],
    values: &'a [python::ForeignObject],
) -> Result<Vec<(&'a str, python::ForeignObject)>, PythonError> {
    if keys.len() != values.len() {
        return Err(PythonError::without_replay(
            "conversion",
            "SifrPythonArgumentError",
            "Python keyword key/value counts differ",
            "",
            "sealed Python call arguments",
        ));
    }
    Ok(keys
        .iter()
        .zip(values)
        .map(|(key, value)| (key.as_str(), value.clone()))
        .collect())
}

fn index_map_from_hash<T>(values: HashMap<String, T>) -> IndexMap<String, T> {
    values.into_iter().collect()
}
