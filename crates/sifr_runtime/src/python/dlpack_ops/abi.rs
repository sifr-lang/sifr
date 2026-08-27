#![allow(unsafe_code)]

use super::{DEVICE_CPU, PythonDlpackTensorMetadata, dlpack_error, unsupported_dtype_error};
use std::ffi::{CStr, c_void};

pub(super) const DLTENSOR_NAME: &CStr = c"dltensor";
pub(super) const USED_DLTENSOR_NAME: &CStr = c"used_dltensor";
pub(super) const DLTENSOR_VERSIONED_NAME: &CStr = c"dltensor_versioned";
pub(super) const USED_DLTENSOR_VERSIONED_NAME: &CStr = c"used_dltensor_versioned";
const DLPACK_FLAG_BITMASK_IS_COPIED: u64 = 1 << 1;

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct DLDevice {
    pub(super) device_type: i32,
    pub(super) device_id: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct DLDataType {
    pub(super) code: u8,
    pub(super) bits: u8,
    pub(super) lanes: u16,
}

#[repr(C)]
pub(super) struct DLTensor {
    pub(super) data: *mut c_void,
    pub(super) device: DLDevice,
    pub(super) ndim: i32,
    pub(super) dtype: DLDataType,
    pub(super) shape: *mut i64,
    pub(super) strides: *mut i64,
    pub(super) byte_offset: u64,
}

#[repr(C)]
pub(super) struct DLManagedTensor {
    pub(super) dl_tensor: DLTensor,
    pub(super) manager_ctx: *mut c_void,
    pub(super) deleter: Option<unsafe extern "C" fn(*mut DLManagedTensor)>,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct DLPackVersion {
    pub(super) major: u32,
    pub(super) minor: u32,
}

#[repr(C)]
pub(super) struct DLManagedTensorVersioned {
    pub(super) version: DLPackVersion,
    pub(super) manager_ctx: *mut c_void,
    pub(super) deleter: Option<unsafe extern "C" fn(*mut DLManagedTensorVersioned)>,
    pub(super) flags: u64,
    pub(super) dl_tensor: DLTensor,
}

#[derive(Clone, Copy)]
pub(super) enum ManagedTensor {
    Legacy(*mut DLManagedTensor),
    Versioned(*mut DLManagedTensorVersioned),
}

// SAFETY: DLPack managed tensors are transferred only while attached to the one
// application-owned Python runtime. The raw pointer never leaves that owner.
unsafe impl Send for ManagedTensor {}

impl ManagedTensor {
    pub(super) fn from_capsule_name(
        pointer: *mut c_void,
        name: &CStr,
    ) -> Result<Self, super::PythonError> {
        if pointer.is_null() {
            return Err(dlpack_error("DLPack capsule pointer is null"));
        }
        if name == DLTENSOR_NAME {
            Ok(Self::Legacy(pointer.cast()))
        } else if name == DLTENSOR_VERSIONED_NAME {
            Ok(Self::Versioned(pointer.cast()))
        } else {
            Err(dlpack_error(format!(
                "unsupported DLPack capsule name '{}'",
                name.to_string_lossy()
            )))
        }
    }

    pub(super) const fn tensor(self) -> *const DLTensor {
        match self {
            // SAFETY: constructors reject null pointers and the capsule keeps
            // the managed tensor alive until its single release path runs.
            Self::Legacy(pointer) => unsafe { &raw const (*pointer).dl_tensor },
            // SAFETY: constructors reject null pointers and the capsule keeps
            // the managed tensor alive until its single release path runs.
            Self::Versioned(pointer) => unsafe { &raw const (*pointer).dl_tensor },
        }
    }

    pub(super) const fn has_deleter(self) -> bool {
        match self {
            // SAFETY: the validated managed-tensor pointer remains capsule-owned.
            Self::Legacy(pointer) => unsafe { (*pointer).deleter.is_some() },
            // SAFETY: the validated managed-tensor pointer remains capsule-owned.
            Self::Versioned(pointer) => unsafe { (*pointer).deleter.is_some() },
        }
    }

    pub(super) const fn capsule_name(self) -> &'static CStr {
        match self {
            Self::Legacy(_) => DLTENSOR_NAME,
            Self::Versioned(_) => DLTENSOR_VERSIONED_NAME,
        }
    }

    pub(super) const fn used_capsule_name(self) -> &'static CStr {
        match self {
            Self::Legacy(_) => USED_DLTENSOR_NAME,
            Self::Versioned(_) => USED_DLTENSOR_VERSIONED_NAME,
        }
    }

    pub(super) const fn pointer(self) -> *mut c_void {
        match self {
            Self::Legacy(pointer) => pointer.cast(),
            Self::Versioned(pointer) => pointer.cast(),
        }
    }

    pub(super) fn validate_version_and_copy(self) -> Result<(), super::PythonError> {
        let Self::Versioned(pointer) = self else {
            return Ok(());
        };
        // SAFETY: the non-null pointer was obtained from a versioned DLPack
        // capsule and remains owned by that capsule during validation.
        let version = unsafe { (*pointer).version };
        if version.major != 1 {
            return Err(dlpack_error(format!(
                "unsupported DLPack managed tensor major version {}.{}; expected major version 1",
                version.major, version.minor
            )));
        }
        // SAFETY: the same validated versioned payload remains live.
        let flags = unsafe { (*pointer).flags };
        if flags & DLPACK_FLAG_BITMASK_IS_COPIED != 0 {
            return Err(dlpack_error(
                "DLPack producer reported a copied tensor despite `copy=False`",
            ));
        }
        Ok(())
    }

    /// Release the producer-owned managed tensor exactly once.
    ///
    /// # Safety
    ///
    /// The caller must still own the producer-named capsule responsibility and
    /// must mark every competing Rust owner released before this call.
    pub(super) unsafe fn release(self) {
        match self {
            Self::Legacy(pointer) => {
                // SAFETY: the caller guarantees exclusive release ownership and
                // the live legacy payload stores its matching deleter.
                if let Some(deleter) = unsafe { (*pointer).deleter } {
                    // SAFETY: the deleter comes from this exact live payload and
                    // is invoked once under the caller's ownership guarantee.
                    unsafe { deleter(pointer) };
                }
            }
            Self::Versioned(pointer) => {
                // SAFETY: the caller guarantees exclusive release ownership and
                // the live versioned payload stores its matching deleter.
                if let Some(deleter) = unsafe { (*pointer).deleter } {
                    // SAFETY: the deleter comes from this exact live payload and
                    // is invoked once under the caller's ownership guarantee.
                    unsafe { deleter(pointer) };
                }
            }
        }
    }
}

pub(super) fn metadata_for_managed_tensor(
    tensor: ManagedTensor,
) -> Result<PythonDlpackTensorMetadata, super::PythonError> {
    tensor.validate_version_and_copy()?;
    // SAFETY: validation established a live capsule-owned managed tensor; the
    // borrowed metadata is consumed before ownership can be released.
    let dl_tensor = unsafe { &*tensor.tensor() };
    if dl_tensor.ndim < 0 {
        return Err(dlpack_error("DLPack dimensions must be non-negative"));
    }
    let dimensions = i64::from(dl_tensor.ndim);
    let len = usize::try_from(dl_tensor.ndim)
        .map_err(|_| dlpack_error("DLPack dimensions exceed Sifr list range"))?;
    if dl_tensor.data.is_null() && tensor_element_count(dl_tensor, len)? != 0 {
        return Err(dlpack_error("DLPack tensor data pointer is null"));
    }
    if dl_tensor.shape.is_null() && len > 0 {
        return Err(dlpack_error("DLPack tensor shape pointer is null"));
    }
    let shape = if dl_tensor.shape.is_null() {
        Vec::new()
    } else {
        // SAFETY: shape is non-null and DLPack requires ndim readable entries.
        unsafe { slice_to_vec(dl_tensor.shape, len) }
    };
    if shape.iter().any(|dimension| *dimension < 0) {
        return Err(dlpack_error(
            "DLPack tensor shape dimensions must be non-negative",
        ));
    }
    let strides = if dl_tensor.strides.is_null() {
        Vec::new()
    } else {
        // SAFETY: strides is non-null and DLPack requires ndim readable entries.
        unsafe { slice_to_vec(dl_tensor.strides, len) }
    };
    if strides.iter().any(|stride| *stride < 0) {
        return Err(dlpack_error("negative DLPack strides are not supported"));
    }
    let dtype = dtype_name(dl_tensor.dtype)?;
    let byte_offset = i64::try_from(dl_tensor.byte_offset)
        .map_err(|_| dlpack_error("DLPack byte offset exceeds Sifr int range"))?;
    Ok(PythonDlpackTensorMetadata {
        handle: -1,
        token: 0,
        dtype_code: i64::from(dl_tensor.dtype.code),
        dtype_bits: i64::from(dl_tensor.dtype.bits),
        dtype_lanes: i64::from(dl_tensor.dtype.lanes),
        dtype,
        device_type: i64::from(dl_tensor.device.device_type),
        device_id: i64::from(dl_tensor.device.device_id),
        dimensions,
        shape,
        strides,
        byte_offset,
        has_deleter: tensor.has_deleter(),
        stream_sync_required: dl_tensor.device.device_type != DEVICE_CPU,
    })
}

fn tensor_element_count(tensor: &DLTensor, len: usize) -> Result<u64, super::PythonError> {
    if len == 0 {
        return Ok(1);
    }
    if tensor.shape.is_null() {
        return Ok(0);
    }
    // SAFETY: shape is non-null and the validated ndim supplies the readable
    // element count for the DLPack shape array.
    unsafe { std::slice::from_raw_parts(tensor.shape.cast_const(), len) }
        .iter()
        .try_fold(1_u64, |count, dimension| {
            let dimension = u64::try_from(*dimension)
                .map_err(|_| dlpack_error("DLPack tensor shape dimensions must be non-negative"))?;
            count
                .checked_mul(dimension)
                .ok_or_else(|| dlpack_error("DLPack tensor element count overflow"))
        })
}

/// Copy a DLPack metadata vector from a validated managed tensor.
///
/// # Safety
///
/// `pointer` must be non-null, aligned, and readable for `len` i64 elements.
unsafe fn slice_to_vec(pointer: *mut i64, len: usize) -> Vec<i64> {
    // SAFETY: the caller provides the exact readable DLPack metadata extent.
    unsafe { std::slice::from_raw_parts(pointer.cast_const(), len) }.to_vec()
}

fn dtype_name(dtype: DLDataType) -> Result<String, super::PythonError> {
    if dtype.lanes != 1 {
        return Err(unsupported_dtype_error(dtype));
    }
    let name = match (dtype.code, dtype.bits) {
        (0, 8) => "int8",
        (0, 16) => "int16",
        (0, 32) => "int32",
        (0, 64) => "int64",
        (1, 8) => "uint8",
        (1, 16) => "uint16",
        (1, 32) => "uint32",
        (1, 64) => "uint64",
        (2, 16) => "float16",
        (2, 32) => "float32",
        (2, 64) => "float64",
        (4, 16) => "bfloat16",
        (6, 1 | 8) => "bool",
        _ => return Err(unsupported_dtype_error(dtype)),
    };
    Ok(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_accepts_null_deleter_and_preserves_offset() {
        let mut shape = [2_i64, 3];
        let mut strides = [3_i64, 1];
        let mut data = [0_u8; 48];
        let mut managed = valid_tensor(&mut shape, &mut strides, &mut data);
        managed.deleter = None;
        managed.dl_tensor.byte_offset = 8;

        let metadata = metadata_for_managed_tensor(ManagedTensor::Legacy(&raw mut managed))
            .expect("valid metadata should be accepted");
        assert!(!metadata.has_deleter);
        assert_eq!(metadata.byte_offset, 8);
        assert_eq!(metadata.shape, [2, 3]);
        assert_eq!(metadata.strides, [3, 1]);
    }

    #[test]
    fn metadata_allows_null_data_for_an_empty_tensor() {
        let mut shape = [0_i64];
        let mut strides = [1_i64];
        let mut data = [];
        let mut managed = valid_tensor(&mut shape, &mut strides, &mut data);
        managed.dl_tensor.data = std::ptr::null_mut();

        let metadata = metadata_for_managed_tensor(ManagedTensor::Legacy(&raw mut managed))
            .expect("empty tensor may have null data");
        assert_eq!(metadata.shape, [0]);
    }

    #[test]
    fn metadata_rejects_invalid_dimensions_and_pointers() {
        let mut shape = [2_i64, 3];
        let mut strides = [3_i64, 1];
        let mut data = [0_u8; 48];

        let mut negative_dimensions = valid_tensor(&mut shape, &mut strides, &mut data);
        negative_dimensions.dl_tensor.ndim = -1;
        assert_error_contains(&mut negative_dimensions, "dimensions must be non-negative");

        let mut null_shape = valid_tensor(&mut shape, &mut strides, &mut data);
        null_shape.dl_tensor.shape = std::ptr::null_mut();
        assert_error_contains(&mut null_shape, "shape pointer is null");

        let mut null_data = valid_tensor(&mut shape, &mut strides, &mut data);
        null_data.dl_tensor.data = std::ptr::null_mut();
        assert_error_contains(&mut null_data, "data pointer is null");
    }

    #[test]
    fn metadata_rejects_shape_stride_dtype_and_offset_drift() {
        let mut shape = [2_i64, 3];
        let mut strides = [3_i64, 1];
        let mut data = [0_u8; 48];

        let mut negative_shape = valid_tensor(&mut shape, &mut strides, &mut data);
        unsafe { *negative_shape.dl_tensor.shape = -1 };
        assert_error_contains(&mut negative_shape, "shape dimensions must be non-negative");
        shape[0] = 2;

        let mut negative_stride = valid_tensor(&mut shape, &mut strides, &mut data);
        unsafe { *negative_stride.dl_tensor.strides = -1 };
        assert_error_contains(&mut negative_stride, "negative DLPack strides");
        strides[0] = 3;

        let mut vector_dtype = valid_tensor(&mut shape, &mut strides, &mut data);
        vector_dtype.dl_tensor.dtype.lanes = 2;
        assert_error_contains(&mut vector_dtype, "lanes=2");

        let mut oversized_offset = valid_tensor(&mut shape, &mut strides, &mut data);
        oversized_offset.dl_tensor.byte_offset = u64::MAX;
        assert_error_contains(&mut oversized_offset, "byte offset exceeds Sifr int range");
    }

    fn valid_tensor(shape: &mut [i64], strides: &mut [i64], data: &mut [u8]) -> DLManagedTensor {
        DLManagedTensor {
            dl_tensor: DLTensor {
                data: data.as_mut_ptr().cast(),
                device: DLDevice {
                    device_type: DEVICE_CPU,
                    device_id: 0,
                },
                ndim: i32::try_from(shape.len()).expect("test rank should fit"),
                dtype: DLDataType {
                    code: 2,
                    bits: 64,
                    lanes: 1,
                },
                shape: shape.as_mut_ptr(),
                strides: strides.as_mut_ptr(),
                byte_offset: 0,
            },
            manager_ctx: std::ptr::null_mut(),
            deleter: None,
        }
    }

    fn assert_error_contains(managed: &mut DLManagedTensor, expected: &str) {
        let error = metadata_for_managed_tensor(ManagedTensor::Legacy(managed))
            .expect_err("invalid metadata must fail");
        assert!(error.message.contains(expected), "{error:?}");
    }
}
