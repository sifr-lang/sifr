//! Checked PEP 3118 address traversal. The live export pins foreign allocations;
//! GIL-bound callers own all observations. Numeric checks reject malformed
//! layouts before element access, without claiming to validate arbitrary native
//! allocations supplied by an exporter.
use super::raw::{BufferFootprint, ValidatedBuffer};
use std::ffi::c_void;
use std::mem::{align_of, size_of};
use std::ptr;

pub(super) enum ElementAddresses {
    Empty,
    Contiguous {
        start: *mut c_void,
        count: usize,
        width: usize,
    },
    Fortran {
        start: *mut c_void,
        count: usize,
        shape: Vec<usize>,
        strides: Vec<usize>,
    },
    Strided(Vec<*mut c_void>),
}

impl ElementAddresses {
    pub(super) fn len(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Contiguous { count, .. } | Self::Fortran { count, .. } => *count,
            Self::Strided(pointers) => pointers.len(),
        }
    }

    pub(super) fn get(&self, index: usize) -> Option<*mut c_void> {
        match self {
            Self::Empty => None,
            Self::Contiguous {
                start,
                count,
                width,
            } if index < *count => {
                // Extent validation at acquisition proves this bounded offset.
                Some(start.wrapping_byte_add(index.checked_mul(*width)?))
            }
            Self::Fortran {
                start,
                count,
                shape,
                strides,
                ..
            } if index < *count => {
                let mut remaining = index;
                let mut offset = 0_usize;
                for axis in (0..shape.len()).rev() {
                    offset = offset
                        .checked_add((remaining % shape[axis]).checked_mul(strides[axis])?)?;
                    remaining /= shape[axis];
                }
                Some(start.wrapping_byte_add(offset))
            }
            Self::Contiguous { .. } | Self::Fortran { .. } => None,
            Self::Strided(pointers) => pointers.get(index).copied(),
        }
    }

    pub(super) fn footprint(&self, width: usize) -> Result<BufferFootprint, String> {
        let ranges = match self {
            Self::Empty => return Ok(BufferFootprint::Empty),
            Self::Contiguous { start, count, .. } | Self::Fortran { start, count, .. } => {
                vec![
                    (*start as usize)
                        ..extent(*start, count.checked_mul(width).ok_or_else(overflow)?)?,
                ]
            }
            Self::Strided(pointers) => {
                let mut ranges = Vec::new();
                ranges
                    .try_reserve_exact(pointers.len())
                    .map_err(|_| capacity())?;
                for pointer in pointers {
                    ranges.push((*pointer as usize)..extent(*pointer, width)?);
                }
                ranges
            }
        };
        let mut ranges = ranges;
        ranges.sort_unstable_by_key(|range| range.start);
        // Merge in place, retaining exact item intervals (including interleaved
        // disjoint views), without a second footprint allocation.
        let mut written = 0;
        for read in 0..ranges.len() {
            if written > 0 && ranges[read].start <= ranges[written - 1].end {
                ranges[written - 1].end = ranges[written - 1].end.max(ranges[read].end);
            } else {
                ranges[written] = ranges[read].clone();
                written += 1;
            }
        }
        ranges.truncate(written);
        Ok(BufferFootprint::Direct { ranges })
    }
}

pub(super) fn is_contiguous(
    shape: &[usize],
    strides: &[isize],
    suboffsets: &[isize],
    width: usize,
    fortran: bool,
) -> Result<bool, String> {
    if !suboffsets.is_empty() {
        return Ok(false);
    }
    if shape.contains(&0) {
        return Ok(true);
    }
    let mut expected = isize::try_from(width).map_err(|_| overflow())?;
    for step in 0..shape.len() {
        let axis = if fortran {
            step
        } else {
            shape.len() - step - 1
        };
        if shape[axis] > 1 && strides[axis] != expected {
            return Ok(false);
        }
        expected = expected
            .checked_mul(isize::try_from(shape[axis]).map_err(|_| overflow())?)
            .ok_or_else(overflow)?;
    }
    Ok(true)
}

pub(super) fn snapshot_addresses(
    start: *mut c_void,
    layout: &ValidatedBuffer,
) -> Result<ElementAddresses, String> {
    if layout.len_bytes == 0 {
        return Ok(ElementAddresses::Empty);
    }
    // Reject arithmetic overflow in every segment before following even the
    // first indirect pointer. Each indirection starts a new allocation segment.
    validate_segments(layout)?;
    let count = layout.len_bytes / layout.item_size;
    if layout.c_contiguous {
        extent(start, layout.len_bytes)?;
        return Ok(ElementAddresses::Contiguous {
            start,
            count,
            width: layout.item_size,
        });
    }
    if layout.f_contiguous {
        extent(start, layout.len_bytes)?;
        // Dense Fortran views retain bounded dimension metadata rather than
        // allocating one pointer per item. Public iteration remains C order.
        let strides = layout
            .strides
            .iter()
            .zip(&layout.shape)
            .map(|(stride, size)| {
                if *size <= 1 {
                    Ok(0)
                } else {
                    usize::try_from(*stride).map_err(|_| overflow())
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(ElementAddresses::Fortran {
            start,
            count,
            shape: layout.shape.clone(),
            strides,
        });
    }
    let mut pointers = Vec::new();
    pointers.try_reserve_exact(count).map_err(|_| capacity())?;
    let mut indices = vec![0; layout.dimensions];
    for index in 0..count {
        let mut remaining = index;
        for axis in (0..layout.dimensions).rev() {
            indices[axis] = remaining % layout.shape[axis];
            remaining /= layout.shape[axis];
        }
        let mut pointer = start;
        for (axis, index) in indices.iter().copied().enumerate() {
            let offset = layout.strides[axis]
                .checked_mul(isize::try_from(index).map_err(|_| overflow())?)
                .ok_or_else(overflow)?;
            pointer = offset_pointer(pointer, offset)?;
            if let Some(suboffset) = layout.suboffsets.get(axis).copied().filter(|v| *v >= 0) {
                extent(pointer, size_of::<*mut c_void>())?;
                if !(pointer as usize).is_multiple_of(align_of::<*mut c_void>()) {
                    return Err("exporter returned a misaligned indirect pointer slot".to_string());
                }
                // SAFETY: FULL/FULL_RO's nonnegative suboffset declares a live
                // initialized pointer slot at this position. The export pins
                // its table allocation; caller holds the GIL. Numeric offset,
                // nonnull slot, alignment and slot extent are checked before
                // this read. No Rust reference or allocation ownership forms.
                pointer = unsafe { ptr::read(pointer.cast::<*mut c_void>()) };
                pointer = offset_pointer(pointer, suboffset)?;
            }
        }
        extent(pointer, layout.item_size)?;
        pointers.push(pointer);
    }
    Ok(ElementAddresses::Strided(pointers))
}

fn validate_segments(layout: &ValidatedBuffer) -> Result<(), String> {
    let (mut low, mut high) = (0_isize, 0_isize);
    for axis in 0..layout.dimensions {
        let span = layout.strides[axis]
            .checked_mul(isize::try_from(layout.shape[axis] - 1).map_err(|_| overflow())?)
            .ok_or_else(overflow)?;
        low = low.checked_add(span.min(0)).ok_or_else(overflow)?;
        high = high.checked_add(span.max(0)).ok_or_else(overflow)?;
        if layout.suboffsets.get(axis).is_some_and(|value| *value >= 0) {
            high.checked_sub(low)
                .and_then(|span| span.checked_add(isize::try_from(size_of::<*mut c_void>()).ok()?))
                .ok_or_else(overflow)?;
            low = 0;
            high = layout.suboffsets[axis];
        }
    }
    high.checked_sub(low)
        .and_then(|span| span.checked_add(isize::try_from(layout.item_size).ok()?))
        .ok_or_else(overflow)?;
    Ok(())
}

fn offset_pointer(pointer: *mut c_void, offset: isize) -> Result<*mut c_void, String> {
    if pointer.is_null() {
        return Err("exporter returned a null logical or indirect pointer".to_string());
    }
    let address = (pointer as usize)
        .checked_add_signed(offset)
        .ok_or_else(overflow)?;
    if address == 0 {
        return Err("exporter returned a null logical address".to_string());
    }
    Ok(pointer.wrapping_byte_offset(offset))
}

fn extent(pointer: *mut c_void, width: usize) -> Result<usize, String> {
    if pointer.is_null() {
        return Err("exporter returned a null logical or indirect pointer".to_string());
    }
    if width > isize::MAX as usize {
        return Err(overflow());
    }
    (pointer as usize).checked_add(width).ok_or_else(overflow)
}

fn overflow() -> String {
    "buffer layout address or stride arithmetic overflows".to_string()
}

fn capacity() -> String {
    "buffer footprint is too large to track safely".to_string()
}
