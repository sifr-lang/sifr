use super::PythonBufferElement;
use super::layout::{ElementAddresses, is_contiguous, snapshot_addresses};
use pyo3::exceptions::PyBufferError;
use pyo3::ffi;
use pyo3::types::PyAny;
use pyo3::{Bound, PyErr, Python};
use std::ffi::CStr;
use std::marker::PhantomPinned;
use std::mem::size_of;
use std::ops::Range;
use std::pin::Pin;
use std::ptr;
use std::slice;

#[derive(Clone, Debug)]
pub(super) struct ValidatedBuffer {
    pub len_bytes: usize,
    pub item_size: usize,
    pub dimensions: usize,
    pub shape: Vec<usize>,
    pub strides: Vec<isize>,
    pub suboffsets: Vec<isize>,
    pub readonly: bool,
    pub c_contiguous: bool,
    pub f_contiguous: bool,
    pub format: String,
}

#[repr(transparent)]
struct RawBuffer(ffi::Py_buffer, PhantomPinned);

/// An owned `Py_buffer` whose address stays stable for exporters that keep
/// self-references inside the view. `Sifr` rejects free-threaded `CPython`, and all
/// access and release operations attach to the interpreter before touching it.
pub(super) struct OwnedPyBuffer {
    raw: Pin<Box<RawBuffer>>,
    addresses: Option<ElementAddresses>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum BufferFootprint {
    Empty,
    Direct { ranges: Vec<Range<usize>> },
}

// SAFETY: This owner transfers an export, never a Python attachment token.
// Supported CPython has a process-lifetime GIL; all acquisition, metadata reads,
// address traversal, element access and release hold it. TrackedBuffer's mutex
// serializes same-handle access/release and admission excludes conflicting Sifr
// storage views. The strong raw.obj export reference pins metadata, pointer slots
// and addressed allocations through release. Foreign exporters must uphold the
// CPython protocol allocation/lifetime contract; arbitrary dangling native
// pointers cannot be proven valid by inspecting a Py_buffer. No Sync is exposed.
unsafe impl Send for OwnedPyBuffer {}

impl OwnedPyBuffer {
    pub(super) fn acquire(
        py: Python<'_>,
        object: &Bound<'_, PyAny>,
        writable: bool,
    ) -> Result<Self, PyErr> {
        let mut raw = Box::pin(RawBuffer(ffi::Py_buffer::new(), PhantomPinned));
        let flags = if writable {
            ffi::PyBUF_FULL
        } else {
            ffi::PyBUF_FULL_RO
        };
        // SAFETY: `raw` is initialized, pinned, and remains alive for the
        // lifetime of the acquired view. The call runs while attached.
        let result = unsafe {
            ffi::PyObject_GetBuffer(
                object.as_ptr(),
                ptr::from_mut(&mut raw_mut(raw.as_mut()).0),
                flags,
            )
        };
        if result != 0 {
            return Err(PyErr::fetch(py));
        }
        if raw.as_ref().get_ref().0.obj.is_null() {
            return Err(PyBufferError::new_err(
                "exporter returned a successful buffer without an owner",
            ));
        }
        Ok(Self {
            raw,
            addresses: None,
        })
    }

    pub(super) fn validate(&self, element: PythonBufferElement) -> Result<ValidatedBuffer, String> {
        let raw = self.raw();
        if raw.obj.is_null() {
            return Err("exporter returned a successful buffer without an owner".to_string());
        }
        let len_bytes = usize::try_from(raw.len)
            .map_err(|_| "exporter returned a negative buffer length".to_string())?;
        let item_size = usize::try_from(raw.itemsize)
            .map_err(|_| "exporter returned a negative item size".to_string())?;
        if item_size == 0 {
            return Err("exporter returned a zero item size".to_string());
        }
        let dimensions = usize::try_from(raw.ndim)
            .map_err(|_| "exporter returned a negative dimension count".to_string())?;
        if dimensions > buffer_dimension_limit(ffi::PyBUF_MAX_NDIM)? {
            return Err("exporter exceeds PyBUF_MAX_NDIM".to_string());
        }
        if len_bytes > 0 && raw.buf.is_null() {
            return Err("exporter returned a null data pointer for a non-empty buffer".to_string());
        }

        let (shape, strides, suboffsets) = metadata_vectors(raw, dimensions)?;
        let item_count = shape_item_count(&shape, dimensions)?;
        validate_byte_length(item_count, item_size, len_bytes)?;
        validate_format(raw, element, item_size)?;

        let c_contiguous = is_contiguous(&shape, &strides, &suboffsets, item_size, false)?;
        let f_contiguous = is_contiguous(&shape, &strides, &suboffsets, item_size, true)?;
        Ok(ValidatedBuffer {
            len_bytes,
            item_size,
            dimensions,
            shape,
            strides,
            suboffsets,
            readonly: raw.readonly != 0,
            c_contiguous,
            f_contiguous,
            format: format_string(raw)?,
        })
    }

    pub(super) fn item_count(&self) -> usize {
        self.addresses.as_ref().map_or(0, ElementAddresses::len)
    }

    pub(super) fn footprint(
        &mut self,
        validated: &ValidatedBuffer,
    ) -> Result<BufferFootprint, String> {
        let addresses = snapshot_addresses(self.raw().buf, validated)?;
        let footprint = addresses.footprint(validated.item_size)?;
        // Pin exactly the resolved logical addresses used by admission. Access
        // never re-follows exporter metadata/indirect slots after admitting a
        // different footprint. The export owns every addressed allocation.
        self.addresses = Some(addresses);
        Ok(footprint)
    }

    pub(super) fn item_ptr(&self, flat_index: usize) -> Option<*mut core::ffi::c_void> {
        self.addresses.as_ref()?.get(flat_index)
    }

    pub(super) fn release(mut self, _py: Python<'_>) {
        self.release_raw();
    }

    fn raw(&self) -> &ffi::Py_buffer {
        &self.raw.as_ref().get_ref().0
    }

    fn release_raw(&mut self) {
        // SAFETY: release(py) or Drop's try_attach holds the supported GIL.
        // This is the sole owner of the pinned view and transferred raw.obj
        // reference; snapshots serialize release through TrackedBuffer's mutex.
        // PyBuffer_Release invokes exporter cleanup then decrefs that reference.
        // Clearing obj makes the subsequent Drop inert, including explicit release.
        let raw = unsafe { &mut raw_mut(self.raw.as_mut()).0 };
        if !raw.obj.is_null() {
            unsafe { ffi::PyBuffer_Release(raw) };
            raw.obj = ptr::null_mut();
        }
    }
}

impl Drop for OwnedPyBuffer {
    fn drop(&mut self) {
        if self.raw().obj.is_null() {
            return;
        }
        let _ignored = Python::try_attach(|_py| self.release_raw());
    }
}

type MetadataVectors = (Vec<usize>, Vec<isize>, Vec<isize>);

fn buffer_dimension_limit<T>(limit: T) -> Result<usize, String>
where
    T: TryInto<usize>,
{
    limit
        .try_into()
        .map_err(|_| "PyBUF_MAX_NDIM does not fit usize".to_string())
}

fn metadata_vectors(raw: &ffi::Py_buffer, dimensions: usize) -> Result<MetadataVectors, String> {
    if dimensions == 0 {
        if !raw.shape.is_null() || !raw.strides.is_null() || !raw.suboffsets.is_null() {
            return Err("zero-dimensional buffer metadata must use null vectors".to_string());
        }
        return Ok((Vec::new(), Vec::new(), Vec::new()));
    }
    if raw.shape.is_null() || raw.strides.is_null() {
        return Err("exporter returned null shape or strides metadata".to_string());
    }
    for pointer in [raw.shape, raw.strides, raw.suboffsets] {
        if !pointer.is_null() && !(pointer as usize).is_multiple_of(std::mem::align_of::<isize>()) {
            return Err("exporter returned misaligned metadata vectors".to_string());
        }
        if !pointer.is_null() {
            (pointer as usize)
                .checked_add(dimensions * size_of::<isize>())
                .ok_or_else(|| "metadata vector address overflows".to_string())?;
        }
    }
    // SAFETY: FULL/FULL_RO supplies `ndim` initialized entries, kept alive by
    // raw.obj. The dimension limit, alignment and address extent were checked;
    // the attached caller holds the GIL. Native exporter allocation validity is
    // the CPython ABI obligation, not derivable from raw pointer values.
    let shape_raw = unsafe { slice::from_raw_parts(raw.shape, dimensions) };
    let strides = unsafe { slice::from_raw_parts(raw.strides, dimensions) }.to_vec();
    let shape = shape_raw
        .iter()
        .copied()
        .map(|value| {
            usize::try_from(value)
                .map_err(|_| "exporter returned a negative shape dimension".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let suboffsets = if raw.suboffsets.is_null() {
        Vec::new()
    } else {
        // SAFETY: a non-null suboffset vector contains `ndim` entries.
        let values = unsafe { slice::from_raw_parts(raw.suboffsets, dimensions) }.to_vec();
        if values.iter().all(|value| *value < 0) {
            return Err(
                "exporter returned a non-null suboffset vector without indirection".to_string(),
            );
        }
        values
    };
    Ok((shape, strides, suboffsets))
}

fn shape_item_count(shape: &[usize], dimensions: usize) -> Result<usize, String> {
    if dimensions == 0 {
        return Ok(1);
    }
    if shape.contains(&0) {
        return Ok(0);
    }
    shape.iter().try_fold(1_usize, |count, dimension| {
        count
            .checked_mul(*dimension)
            .ok_or_else(|| "buffer shape exceeds the supported address space".to_string())
    })
}

fn validate_byte_length(
    item_count: usize,
    item_size: usize,
    len_bytes: usize,
) -> Result<(), String> {
    let expected = item_count
        .checked_mul(item_size)
        .ok_or_else(|| "buffer byte length exceeds the supported address space".to_string())?;
    if expected != len_bytes {
        return Err(
            "exporter byte length does not equal shape product times item size".to_string(),
        );
    }
    Ok(())
}

fn validate_format(
    raw: &ffi::Py_buffer,
    element: PythonBufferElement,
    item_size: usize,
) -> Result<(), String> {
    let format = format_bytes(raw)?;
    validate_format_bytes(format, element, item_size)
}

fn validate_format_bytes(
    format: &[u8],
    element: PythonBufferElement,
    item_size: usize,
) -> Result<(), String> {
    let (prefix, code) = match format {
        [code] => (None, *code),
        [prefix @ (b'@' | b'=' | b'<' | b'>' | b'!'), code] => (Some(*prefix), *code),
        _ => return Err("buffer format is not a supported scalar PEP 3118 type".to_string()),
    };
    let (category, declared_size) = format_category_and_size(prefix, code)
        .ok_or_else(|| "buffer format is not a supported scalar PEP 3118 type".to_string())?;
    if category != element.category() || declared_size != item_size || item_size != element.width()
    {
        return Err(format!(
            "buffer contents are not compatible with {}",
            element.source_name()
        ));
    }
    if item_size > 1 && !native_endian(prefix) {
        return Err("buffer byte order does not match the native target".to_string());
    }
    Ok(())
}

fn format_category_and_size(prefix: Option<u8>, code: u8) -> Option<(FormatCategory, usize)> {
    let native_sizes = prefix.is_none() || prefix == Some(b'@');
    let value = match code {
        b'b' => (FormatCategory::Signed, 1),
        b'B' => (FormatCategory::Unsigned, 1),
        b'h' => (
            FormatCategory::Signed,
            if native_sizes {
                size_of::<libc::c_short>()
            } else {
                2
            },
        ),
        b'H' => (
            FormatCategory::Unsigned,
            if native_sizes {
                size_of::<libc::c_ushort>()
            } else {
                2
            },
        ),
        b'i' => (
            FormatCategory::Signed,
            if native_sizes {
                size_of::<libc::c_int>()
            } else {
                4
            },
        ),
        b'I' => (
            FormatCategory::Unsigned,
            if native_sizes {
                size_of::<libc::c_uint>()
            } else {
                4
            },
        ),
        b'l' => (
            FormatCategory::Signed,
            if native_sizes {
                size_of::<libc::c_long>()
            } else {
                4
            },
        ),
        b'L' => (
            FormatCategory::Unsigned,
            if native_sizes {
                size_of::<libc::c_ulong>()
            } else {
                4
            },
        ),
        b'q' => (FormatCategory::Signed, 8),
        b'Q' => (FormatCategory::Unsigned, 8),
        b'n' if native_sizes => (FormatCategory::Signed, size_of::<isize>()),
        b'N' if native_sizes => (FormatCategory::Unsigned, size_of::<usize>()),
        b'f' => (FormatCategory::Float, 4),
        b'd' => (FormatCategory::Float, 8),
        _ => return None,
    };
    Some(value)
}

const fn native_endian(prefix: Option<u8>) -> bool {
    match prefix {
        None | Some(b'@' | b'=') => true,
        Some(b'<') => cfg!(target_endian = "little"),
        Some(b'>' | b'!') => cfg!(target_endian = "big"),
        _ => false,
    }
}

fn format_bytes(raw: &ffi::Py_buffer) -> Result<&[u8], String> {
    if raw.format.is_null() {
        return Err("exporter omitted the requested PEP 3118 format".to_string());
    }
    // SAFETY: a successful formatted buffer request supplies a NUL-terminated
    // format string that remains valid until `PyBuffer_Release`.
    Ok(unsafe { CStr::from_ptr(raw.format) }.to_bytes())
}

fn format_string(raw: &ffi::Py_buffer) -> Result<String, String> {
    Ok(String::from_utf8_lossy(format_bytes(raw)?).into_owned())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FormatCategory {
    Signed,
    Unsigned,
    Float,
}

impl PythonBufferElement {
    const fn category(self) -> FormatCategory {
        match self {
            Self::I8 | Self::I16 | Self::I32 | Self::I64 | Self::ISize => FormatCategory::Signed,
            Self::U8 | Self::U16 | Self::U32 | Self::U64 | Self::USize => FormatCategory::Unsigned,
            Self::F64 => FormatCategory::Float,
        }
    }

    const fn width(self) -> usize {
        match self {
            Self::I8 | Self::U8 => 1,
            Self::I16 | Self::U16 => 2,
            Self::I32 | Self::U32 => 4,
            Self::I64 | Self::U64 | Self::F64 => 8,
            Self::ISize | Self::USize => size_of::<usize>(),
        }
    }
}

unsafe fn raw_mut(raw: Pin<&mut RawBuffer>) -> &mut RawBuffer {
    // SAFETY: callers do not move the pinned value; they only mutate fields in
    // the stable allocation passed to CPython.
    unsafe { Pin::get_unchecked_mut(raw) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endian_contract_accepts_only_native_multibyte_formats() {
        assert!(native_endian(None));
        assert!(native_endian(Some(b'@')));
        assert!(native_endian(Some(b'=')));
        assert_eq!(native_endian(Some(b'<')), cfg!(target_endian = "little"));
        assert_eq!(native_endian(Some(b'>')), cfg!(target_endian = "big"));
        assert_eq!(native_endian(Some(b'!')), cfg!(target_endian = "big"));
    }

    #[test]
    fn dimension_limit_accepts_both_supported_ffi_constant_types() {
        assert_eq!(buffer_dimension_limit(64_i32), Ok(64));
        assert_eq!(buffer_dimension_limit(64_usize), Ok(64));
    }

    #[test]
    fn byte_length_validation_rejects_floor_division_mismatch() {
        assert!(validate_byte_length(1, 4, 4).is_ok());
        assert!(validate_byte_length(1, 4, 5).is_err());
    }

    #[test]
    fn malformed_successful_view_without_owner_is_rejected() {
        let mut raw = ffi::Py_buffer::new();
        raw.buf = ptr::dangling_mut::<u8>().cast();
        raw.len = 1;
        raw.itemsize = 1;
        raw.readonly = 1;
        raw.ndim = 1;
        raw.format = c"B".as_ptr().cast_mut();
        raw.shape = &mut raw.len;
        raw.strides = &mut raw.itemsize;

        let buffer = OwnedPyBuffer {
            raw: Box::pin(RawBuffer(raw, PhantomPinned)),
            addresses: None,
        };
        let error = buffer
            .validate(PythonBufferElement::U8)
            .expect_err("null ownership must not enter the safe buffer API");

        assert!(error.contains("without an owner"));
    }

    #[test]
    fn empty_shape_short_circuits_product_overflow() {
        let shape = [usize::MAX, 2, 0];
        assert_eq!(shape_item_count(&shape, shape.len()), Ok(0));
    }

    #[test]
    fn all_negative_non_null_suboffsets_are_rejected() {
        let mut raw = ffi::Py_buffer::new();
        let mut shape = [2_isize, 2];
        let mut strides = [2_isize, 1];
        let mut suboffsets = [-1_isize, -1];
        raw.ndim = 2;
        raw.shape = shape.as_mut_ptr();
        raw.strides = strides.as_mut_ptr();
        raw.suboffsets = suboffsets.as_mut_ptr();

        let error = metadata_vectors(&raw, 2).expect_err("all-negative vector is malformed");
        assert!(error.contains("without indirection"));
    }

    #[test]
    fn primitive_format_matrix_enforces_native_endian_for_every_multibyte_family() {
        let native = if cfg!(target_endian = "little") {
            b'<'
        } else {
            b'>'
        };
        let foreign = if cfg!(target_endian = "little") {
            b'>'
        } else {
            b'<'
        };
        let families = [
            (PythonBufferElement::I16, b'h', 2, false),
            (PythonBufferElement::U16, b'H', 2, false),
            (PythonBufferElement::I32, b'i', 4, false),
            (PythonBufferElement::U32, b'I', 4, false),
            (PythonBufferElement::I64, b'q', 8, false),
            (PythonBufferElement::U64, b'Q', 8, false),
            (PythonBufferElement::ISize, b'n', size_of::<isize>(), true),
            (PythonBufferElement::USize, b'N', size_of::<usize>(), true),
            (PythonBufferElement::F64, b'd', 8, false),
        ];
        for (element, code, width, native_size_only) in families {
            assert!(validate_format_bytes(&[code], element, width).is_ok());
            assert!(validate_format_bytes(&[b'@', code], element, width).is_ok());
            assert_eq!(
                validate_format_bytes(&[b'=', code], element, width).is_ok(),
                !native_size_only
            );
            assert_eq!(
                validate_format_bytes(&[native, code], element, width).is_ok(),
                !native_size_only
            );
            assert!(validate_format_bytes(&[foreign, code], element, width).is_err());
            assert_eq!(
                validate_format_bytes(&[b'!', code], element, width).is_ok(),
                cfg!(target_endian = "big") && !native_size_only
            );
        }

        assert!(validate_format_bytes(b">b", PythonBufferElement::I8, 1).is_ok());
        assert!(validate_format_bytes(b"!B", PythonBufferElement::U8, 1).is_ok());
    }
}
