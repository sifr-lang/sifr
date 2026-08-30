use super::{PythonError, PythonObject, ResourceIdentity, object_bridge, take_resource};
use sifr_runtime::{SifrInt, interop::Handle, python};

mod sealed {
    pub trait Sealed {}
}

/// Rust-side element bridge used only by generated `python.Buffer[T]` glue.
#[doc(hidden)]
pub trait PythonBufferElement: sealed::Sealed + Copy {
    const ELEMENT: python::PythonBufferElement;

    fn read(handle: python::BufferHandle, index: i64) -> Result<Self, PythonError>;
    fn write(handle: python::BufferHandle, index: i64, value: Self) -> Result<(), PythonError>;
    fn copy_slice(
        handle: python::BufferHandle,
        start: i64,
        length: i64,
    ) -> Result<Vec<Self>, PythonError>;
}

macro_rules! buffer_element {
    ($ty:ty, $element:ident, $read:ident, $write:ident, $copy:ident) => {
        impl sealed::Sealed for $ty {}

        impl PythonBufferElement for $ty {
            const ELEMENT: python::PythonBufferElement = python::PythonBufferElement::$element;

            fn read(handle: python::BufferHandle, index: i64) -> Result<Self, PythonError> {
                python::$read(handle, index)
            }

            fn write(
                handle: python::BufferHandle,
                index: i64,
                value: Self,
            ) -> Result<(), PythonError> {
                python::$write(handle, index, value)
            }

            fn copy_slice(
                handle: python::BufferHandle,
                start: i64,
                length: i64,
            ) -> Result<Vec<Self>, PythonError> {
                python::$copy(handle, start, length)
            }
        }
    };
}

buffer_element!(
    i8,
    I8,
    buffer_read_i8,
    buffer_write_i8,
    copy_buffer_slice_i8
);
buffer_element!(
    i16,
    I16,
    buffer_read_i16,
    buffer_write_i16,
    copy_buffer_slice_i16
);
buffer_element!(
    i32,
    I32,
    buffer_read_i32,
    buffer_write_i32,
    copy_buffer_slice_i32
);
buffer_element!(
    i64,
    I64,
    buffer_read_i64,
    buffer_write_i64,
    copy_buffer_slice_i64
);
buffer_element!(
    isize,
    ISize,
    buffer_read_isize,
    buffer_write_isize,
    copy_buffer_slice_isize
);
buffer_element!(
    u8,
    U8,
    buffer_read_u8,
    buffer_write_u8,
    copy_buffer_slice_u8
);
buffer_element!(
    u16,
    U16,
    buffer_read_u16,
    buffer_write_u16,
    copy_buffer_slice_u16
);
buffer_element!(
    u32,
    U32,
    buffer_read_u32,
    buffer_write_u32,
    copy_buffer_slice_u32
);
buffer_element!(
    u64,
    U64,
    buffer_read_u64,
    buffer_write_u64,
    copy_buffer_slice_u64
);
buffer_element!(
    usize,
    USize,
    buffer_read_usize,
    buffer_write_usize,
    copy_buffer_slice_usize
);
buffer_element!(
    f64,
    F64,
    buffer_read_f64,
    buffer_write_f64,
    copy_buffer_slice_f64
);

/// Sealed affine owner for one validated Python buffer export.
#[derive(Debug)]
pub struct PythonBuffer<T: PythonBufferElement> {
    identity: ResourceIdentity,
    metadata: python::PythonBufferMetadata,
    marker: std::marker::PhantomData<T>,
}

impl<T: PythonBufferElement> PythonBuffer<T> {
    pub fn acquire(
        object: &PythonObject,
        access: python::PythonBufferAccess,
        layout: python::PythonBufferLayout,
    ) -> Result<Self, PythonError> {
        Self::acquire_foreign(object_bridge::object_value(object)?, access, layout)
    }

    #[doc(hidden)]
    pub fn acquire_foreign(
        object: &python::ForeignObject,
        access: python::PythonBufferAccess,
        layout: python::PythonBufferLayout,
    ) -> Result<Self, PythonError> {
        let metadata = python::acquire_buffer(
            object,
            python::PythonBufferRequest {
                element: T::ELEMENT,
                access,
                layout,
            },
        )?;
        let identity = Handle::new(python::PythonResourceIdentity::buffer((
            metadata.handle,
            metadata.token,
        )));
        Ok(Self {
            identity,
            metadata,
            marker: std::marker::PhantomData,
        })
    }

    pub fn read(&self, index: SifrInt) -> Result<T, PythonError> {
        T::read(self.handle()?, exact_buffer_offset(&index, "buffer index")?)
    }

    pub fn write(&mut self, index: SifrInt, value: T) -> Result<(), PythonError> {
        T::write(
            self.handle()?,
            exact_buffer_offset(&index, "buffer index")?,
            value,
        )
    }

    pub fn copy_slice(&self, start: SifrInt, length: SifrInt) -> Result<Vec<T>, PythonError> {
        T::copy_slice(
            self.handle()?,
            exact_buffer_offset(&start, "buffer slice start")?,
            exact_buffer_offset(&length, "buffer slice length")?,
        )
    }

    pub fn release(self) -> Result<(), PythonError> {
        take_resource(self.identity)?.close()
    }

    #[must_use]
    pub fn length(&self) -> SifrInt {
        SifrInt::from(self.metadata.len_bytes / self.metadata.item_size)
    }

    #[must_use]
    pub fn item_size(&self) -> SifrInt {
        SifrInt::from(self.metadata.item_size)
    }

    #[must_use]
    pub fn dimensions(&self) -> SifrInt {
        SifrInt::from(self.metadata.dimensions)
    }

    #[must_use]
    pub fn shape(&self) -> Vec<SifrInt> {
        self.metadata
            .shape
            .iter()
            .copied()
            .map(SifrInt::from)
            .collect()
    }

    #[must_use]
    pub fn strides(&self) -> Vec<SifrInt> {
        self.metadata
            .strides
            .iter()
            .copied()
            .map(SifrInt::from)
            .collect()
    }

    #[must_use]
    pub fn suboffsets(&self) -> Vec<SifrInt> {
        self.metadata
            .suboffsets
            .iter()
            .copied()
            .map(SifrInt::from)
            .collect()
    }

    #[must_use]
    pub fn format(&self) -> String {
        self.metadata.format.clone()
    }

    #[must_use]
    pub const fn readonly(&self) -> bool {
        self.metadata.readonly
    }

    #[must_use]
    pub const fn c_contiguous(&self) -> bool {
        self.metadata.c_contiguous
    }

    #[must_use]
    pub const fn f_contiguous(&self) -> bool {
        self.metadata.f_contiguous
    }

    fn handle(&self) -> Result<python::BufferHandle, PythonError> {
        super::resource_value(&self.identity)?.buffer_key()
    }
}

fn exact_buffer_offset(value: &SifrInt, name: &str) -> Result<i64, PythonError> {
    value.try_to_i64().map_err(|error| {
        PythonError::without_replay(
            "index",
            "IndexError",
            format!("{name} is outside the addressable buffer range: {error}"),
            "",
            "typed Python buffer access",
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{PythonBuffer, exact_buffer_offset};
    use sifr_runtime::SifrInt;

    static_assertions::assert_not_impl_any!(PythonBuffer<u8>: Clone, Send, Sync);

    #[test]
    fn exact_buffer_offsets_reject_values_outside_the_runtime_address_range() {
        assert_eq!(
            exact_buffer_offset(&SifrInt::from(42), "buffer index")
                .expect("small exact index should fit"),
            42
        );

        let too_large = SifrInt::from_i128(i128::from(i64::MAX) + 1);
        let error = exact_buffer_offset(&too_large, "buffer index")
            .expect_err("oversized exact index should remain a typed failure");
        assert_eq!(error.kind, "index");
        assert_eq!(error.exception_type, "IndexError");
        assert!(
            error
                .message
                .contains("outside the addressable buffer range")
        );
    }
}
