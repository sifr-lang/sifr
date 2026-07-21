use super::{arrow_error, PythonError};
use std::ffi::{c_char, c_void};
use std::ptr::NonNull;

type SchemaRelease = unsafe extern "C" fn(*mut ArrowSchema);
type ArrayRelease = unsafe extern "C" fn(*mut ArrowArray);
type StreamGetSchema = unsafe extern "C" fn(*mut ArrowArrayStream, *mut ArrowSchema) -> i32;
type StreamGetNext = unsafe extern "C" fn(*mut ArrowArrayStream, *mut ArrowArray) -> i32;
type StreamGetLastError = unsafe extern "C" fn(*mut ArrowArrayStream) -> *const c_char;
type StreamRelease = unsafe extern "C" fn(*mut ArrowArrayStream);
type DeviceStreamGetSchema =
    unsafe extern "C" fn(*mut ArrowDeviceArrayStream, *mut ArrowSchema) -> i32;
type DeviceStreamGetNext =
    unsafe extern "C" fn(*mut ArrowDeviceArrayStream, *mut ArrowDeviceArray) -> i32;
type DeviceStreamGetLastError = unsafe extern "C" fn(*mut ArrowDeviceArrayStream) -> *const c_char;
type DeviceStreamRelease = unsafe extern "C" fn(*mut ArrowDeviceArrayStream);

#[repr(C)]
pub(super) struct ArrowSchema {
    pub(super) format: *const c_char,
    pub(super) name: *const c_char,
    pub(super) metadata: *const c_char,
    pub(super) flags: i64,
    pub(super) n_children: i64,
    pub(super) children: *mut *mut Self,
    pub(super) dictionary: *mut Self,
    pub(super) release: Option<SchemaRelease>,
    pub(super) private_data: *mut c_void,
}

unsafe impl Send for ArrowSchema {}

#[repr(C)]
pub(super) struct ArrowArray {
    pub(super) length: i64,
    pub(super) null_count: i64,
    pub(super) offset: i64,
    pub(super) n_buffers: i64,
    pub(super) n_children: i64,
    pub(super) buffers: *mut *const c_void,
    pub(super) children: *mut *mut Self,
    pub(super) dictionary: *mut Self,
    pub(super) release: Option<ArrayRelease>,
    pub(super) private_data: *mut c_void,
}

unsafe impl Send for ArrowArray {}

#[repr(C)]
pub(super) struct ArrowArrayStream {
    pub(super) get_schema: Option<StreamGetSchema>,
    pub(super) get_next: Option<StreamGetNext>,
    pub(super) get_last_error: Option<StreamGetLastError>,
    pub(super) release: Option<StreamRelease>,
    pub(super) private_data: *mut c_void,
}

unsafe impl Send for ArrowArrayStream {}

#[repr(C)]
pub(super) struct ArrowDeviceArray {
    pub(super) array: ArrowArray,
    pub(super) device_id: i64,
    pub(super) device_type: i32,
    pub(super) sync_event: *mut c_void,
    pub(super) reserved: [i64; 3],
}

unsafe impl Send for ArrowDeviceArray {}

#[repr(C)]
pub(super) struct ArrowDeviceArrayStream {
    pub(super) device_type: i32,
    pub(super) get_schema: Option<DeviceStreamGetSchema>,
    pub(super) get_next: Option<DeviceStreamGetNext>,
    pub(super) get_last_error: Option<DeviceStreamGetLastError>,
    pub(super) release: Option<DeviceStreamRelease>,
    pub(super) private_data: *mut c_void,
}

unsafe impl Send for ArrowDeviceArrayStream {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ConsumptionState {
    Unconsumed,
    Consumed,
    Partial,
}

pub(super) fn validate_schema(pointer: NonNull<c_void>, context: &str) -> Result<(), PythonError> {
    let schema = unsafe { pointer.cast::<ArrowSchema>().as_ref() };
    if schema.release.is_none() {
        return Err(arrow_error(format!(
            "{context} ArrowSchema has no release callback"
        )));
    }
    if schema.format.is_null() {
        return Err(arrow_error(format!(
            "{context} ArrowSchema has a null format"
        )));
    }
    validate_count_and_pointer(
        schema.n_children,
        schema.children.cast::<c_void>(),
        context,
        "ArrowSchema children",
    )
}

pub(super) fn validate_array(pointer: NonNull<c_void>, context: &str) -> Result<(), PythonError> {
    let array = unsafe { pointer.cast::<ArrowArray>().as_ref() };
    validate_array_value(array, context)
}

pub(super) fn validate_array_pair(
    schema_pointer: NonNull<c_void>,
    array_pointer: NonNull<c_void>,
    context: &str,
) -> Result<(), PythonError> {
    validate_schema(schema_pointer, context)?;
    validate_array(array_pointer, context)?;
    let schema = unsafe { schema_pointer.cast::<ArrowSchema>().as_ref() };
    let array = unsafe { array_pointer.cast::<ArrowArray>().as_ref() };
    if schema.n_children != array.n_children {
        return Err(arrow_error(format!(
            "{context} schema/array child counts differ ({} != {})",
            schema.n_children, array.n_children
        )));
    }
    if schema.dictionary.is_null() != array.dictionary.is_null() {
        return Err(arrow_error(format!(
            "{context} schema/array dictionary presence differs"
        )));
    }
    Ok(())
}

pub(super) fn validate_stream(pointer: NonNull<c_void>, context: &str) -> Result<(), PythonError> {
    let stream = unsafe { pointer.cast::<ArrowArrayStream>().as_ref() };
    if stream.get_schema.is_none()
        || stream.get_next.is_none()
        || stream.get_last_error.is_none()
        || stream.release.is_none()
    {
        return Err(arrow_error(format!(
            "{context} ArrowArrayStream is missing a required callback"
        )));
    }
    Ok(())
}

pub(super) fn validate_device_array_pair(
    schema_pointer: NonNull<c_void>,
    device_pointer: NonNull<c_void>,
    context: &str,
) -> Result<(), PythonError> {
    validate_schema(schema_pointer, context)?;
    let device = unsafe { device_pointer.cast::<ArrowDeviceArray>().as_ref() };
    validate_array_value(&device.array, context)?;
    validate_device_type(device.device_type, context)?;
    if device.device_id < 0 {
        return Err(arrow_error(format!(
            "{context} ArrowDeviceArray has a negative device_id"
        )));
    }
    if device.reserved != [0; 3] {
        return Err(arrow_error(format!(
            "{context} ArrowDeviceArray reserved fields must be zero"
        )));
    }
    let schema = unsafe { schema_pointer.cast::<ArrowSchema>().as_ref() };
    if schema.n_children != device.array.n_children {
        return Err(arrow_error(format!(
            "{context} schema/device-array child counts differ ({} != {})",
            schema.n_children, device.array.n_children
        )));
    }
    if schema.dictionary.is_null() != device.array.dictionary.is_null() {
        return Err(arrow_error(format!(
            "{context} schema/device-array dictionary presence differs"
        )));
    }
    Ok(())
}

pub(super) fn validate_device_stream(
    pointer: NonNull<c_void>,
    context: &str,
) -> Result<(), PythonError> {
    let stream = unsafe { pointer.cast::<ArrowDeviceArrayStream>().as_ref() };
    validate_device_type(stream.device_type, context)?;
    if stream.get_schema.is_none()
        || stream.get_next.is_none()
        || stream.get_last_error.is_none()
        || stream.release.is_none()
    {
        return Err(arrow_error(format!(
            "{context} ArrowDeviceArrayStream is missing a required callback"
        )));
    }
    Ok(())
}

pub(super) fn schema_consumption(pointer: NonNull<c_void>) -> ConsumptionState {
    bool_state(
        unsafe { pointer.cast::<ArrowSchema>().as_ref() }
            .release
            .is_none(),
    )
}

pub(super) fn array_pair_consumption(
    schema_pointer: NonNull<c_void>,
    array_pointer: NonNull<c_void>,
) -> ConsumptionState {
    combined_state([
        unsafe { schema_pointer.cast::<ArrowSchema>().as_ref() }
            .release
            .is_none(),
        unsafe { array_pointer.cast::<ArrowArray>().as_ref() }
            .release
            .is_none(),
    ])
}

pub(super) fn stream_consumption(pointer: NonNull<c_void>) -> ConsumptionState {
    bool_state(
        unsafe { pointer.cast::<ArrowArrayStream>().as_ref() }
            .release
            .is_none(),
    )
}

pub(super) fn device_array_pair_consumption(
    schema_pointer: NonNull<c_void>,
    array_pointer: NonNull<c_void>,
) -> ConsumptionState {
    combined_state([
        unsafe { schema_pointer.cast::<ArrowSchema>().as_ref() }
            .release
            .is_none(),
        unsafe { array_pointer.cast::<ArrowDeviceArray>().as_ref() }
            .array
            .release
            .is_none(),
    ])
}

pub(super) fn device_stream_consumption(pointer: NonNull<c_void>) -> ConsumptionState {
    bool_state(
        unsafe { pointer.cast::<ArrowDeviceArrayStream>().as_ref() }
            .release
            .is_none(),
    )
}

const fn bool_state(consumed: bool) -> ConsumptionState {
    if consumed {
        ConsumptionState::Consumed
    } else {
        ConsumptionState::Unconsumed
    }
}

fn combined_state(states: [bool; 2]) -> ConsumptionState {
    match states {
        [false, false] => ConsumptionState::Unconsumed,
        [true, true] => ConsumptionState::Consumed,
        _ => ConsumptionState::Partial,
    }
}

fn validate_array_value(array: &ArrowArray, context: &str) -> Result<(), PythonError> {
    if array.release.is_none() {
        return Err(arrow_error(format!(
            "{context} ArrowArray has no release callback"
        )));
    }
    if array.length < 0 || array.offset < 0 || array.n_buffers < 0 {
        return Err(arrow_error(format!(
            "{context} ArrowArray has invalid negative shape metadata"
        )));
    }
    if array.null_count < -1 {
        return Err(arrow_error(format!(
            "{context} ArrowArray has invalid null_count {}",
            array.null_count
        )));
    }
    validate_count_and_pointer(
        array.n_buffers,
        array.buffers.cast::<c_void>(),
        context,
        "ArrowArray buffers",
    )?;
    validate_count_and_pointer(
        array.n_children,
        array.children.cast::<c_void>(),
        context,
        "ArrowArray children",
    )
}

fn validate_count_and_pointer(
    count: i64,
    pointer: *mut c_void,
    context: &str,
    field: &str,
) -> Result<(), PythonError> {
    if count < 0 {
        return Err(arrow_error(format!("{context} {field} count is negative")));
    }
    if count > 0 && pointer.is_null() {
        return Err(arrow_error(format!(
            "{context} {field} pointer is null for a non-empty collection"
        )));
    }
    Ok(())
}

fn validate_device_type(device_type: i32, context: &str) -> Result<(), PythonError> {
    // ArrowDeviceType follows the stable DLPack device-type identifiers. Keep
    // this closed so unknown future values require an explicit runtime update.
    const KNOWN_DEVICE_TYPES: &[i32] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    if KNOWN_DEVICE_TYPES.contains(&device_type) {
        Ok(())
    } else {
        Err(arrow_error(format!(
            "{context} has unknown Arrow device type {device_type}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe extern "C" fn release_schema(_value: *mut ArrowSchema) {}
    unsafe extern "C" fn release_array(_value: *mut ArrowArray) {}

    #[test]
    fn rejects_missing_release_and_inconsistent_pairs() {
        let mut schema = schema();
        let mut array = array();
        schema.release = None;
        assert!(validate_schema(pointer(&mut schema), "test")
            .expect_err("release is required")
            .message
            .contains("release callback"));

        schema.release = Some(release_schema);
        schema.n_children = 1;
        schema.children = std::ptr::dangling_mut();
        assert!(
            validate_array_pair(pointer(&mut schema), pointer(&mut array), "test")
                .expect_err("child counts must match")
                .message
                .contains("child counts differ")
        );
    }

    #[test]
    fn rejects_invalid_device_metadata() {
        let mut schema = schema();
        let mut device = ArrowDeviceArray {
            array: array(),
            device_id: 0,
            device_type: 1,
            sync_event: std::ptr::null_mut(),
            reserved: [0, 1, 0],
        };
        assert!(
            validate_device_array_pair(pointer(&mut schema), pointer(&mut device), "test")
                .expect_err("reserved fields must be zero")
                .message
                .contains("reserved fields")
        );

        device.reserved = [0; 3];
        device.device_type = 999;
        assert!(
            validate_device_array_pair(pointer(&mut schema), pointer(&mut device), "test")
                .expect_err("device type must be known")
                .message
                .contains("unknown Arrow device type")
        );
    }

    fn schema() -> ArrowSchema {
        ArrowSchema {
            format: c"i".as_ptr(),
            name: std::ptr::null(),
            metadata: std::ptr::null(),
            flags: 0,
            n_children: 0,
            children: std::ptr::null_mut(),
            dictionary: std::ptr::null_mut(),
            release: Some(release_schema),
            private_data: std::ptr::null_mut(),
        }
    }

    fn array() -> ArrowArray {
        ArrowArray {
            length: 1,
            null_count: 0,
            offset: 0,
            n_buffers: 0,
            n_children: 0,
            buffers: std::ptr::null_mut(),
            children: std::ptr::null_mut(),
            dictionary: std::ptr::null_mut(),
            release: Some(release_array),
            private_data: std::ptr::null_mut(),
        }
    }

    fn pointer<T>(value: &mut T) -> NonNull<c_void> {
        NonNull::from(value).cast()
    }
}
