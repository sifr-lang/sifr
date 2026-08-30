use super::{Handle, PythonError, PythonObject, object_bridge, take_resource};
use sifr_runtime::{SifrInt, python};
use std::marker::PhantomData;
use std::rc::Rc;

type ResourceIdentity = Handle<python::PythonResourceIdentity>;

mod sealed {
    pub trait Sealed {}
}

#[doc(hidden)]
pub trait PythonDlpackElement: sealed::Sealed {
    const DTYPE: &'static str;
}

macro_rules! dlpack_element {
    ($rust:ty, $dtype:literal) => {
        impl sealed::Sealed for $rust {}
        impl PythonDlpackElement for $rust {
            const DTYPE: &'static str = $dtype;
        }
    };
}

dlpack_element!(i8, "int8");
dlpack_element!(i16, "int16");
dlpack_element!(i32, "int32");
dlpack_element!(i64, "int64");
dlpack_element!(u8, "uint8");
dlpack_element!(u16, "uint16");
dlpack_element!(u32, "uint32");
dlpack_element!(u64, "uint64");
dlpack_element!(f64, "float64");
dlpack_element!(bool, "bool");

#[derive(Debug)]
pub struct PythonDlpackStream {
    metadata: python::PythonDlpackStreamMetadata,
    _non_send: PhantomData<Rc<()>>,
}

impl PythonDlpackStream {
    pub fn acquire(object: &PythonObject, device: &str) -> Result<Self, PythonError> {
        Self::acquire_foreign(object_bridge::object_value(object)?, device)
    }

    #[doc(hidden)]
    pub fn acquire_foreign(
        object: &python::ForeignObject,
        device: &str,
    ) -> Result<Self, PythonError> {
        python::dlpack_stream(object, device).map(|metadata| Self {
            metadata,
            _non_send: PhantomData,
        })
    }

    #[must_use]
    pub fn device_type(&self) -> SifrInt {
        SifrInt::from(self.metadata.device_type)
    }

    #[must_use]
    pub fn device_id(&self) -> SifrInt {
        SifrInt::from(self.metadata.device_id)
    }

    #[must_use]
    pub fn stream_token(&self) -> SifrInt {
        SifrInt::from(self.metadata.stream_token)
    }
}

#[derive(Debug)]
pub struct PythonDlpackTensor<T: PythonDlpackElement> {
    identity: ResourceIdentity,
    metadata: python::PythonDlpackTensorMetadata,
    _element: PhantomData<T>,
}

impl<T: PythonDlpackElement> PythonDlpackTensor<T> {
    pub fn acquire(
        object: &PythonObject,
        device: &str,
        stream: Option<&PythonDlpackStream>,
    ) -> Result<Self, PythonError> {
        Self::acquire_foreign(object_bridge::object_value(object)?, device, stream)
    }

    #[doc(hidden)]
    pub fn acquire_foreign(
        object: &python::ForeignObject,
        device: &str,
        stream: Option<&PythonDlpackStream>,
    ) -> Result<Self, PythonError> {
        python::acquire_dlpack_tensor(
            object,
            device,
            stream.map(|value| &value.metadata),
            Some(T::DTYPE),
        )
        .map(Self::from_metadata)
    }

    fn from_metadata(metadata: python::PythonDlpackTensorMetadata) -> Self {
        Self {
            identity: Handle::new(python::PythonResourceIdentity::dlpack((
                metadata.handle,
                metadata.token,
            ))),
            metadata,
            _element: PhantomData,
        }
    }

    pub fn release(self) -> Result<(), PythonError> {
        take_resource(self.identity)?.close()
    }

    #[doc(hidden)]
    pub fn prepare_argument(self) -> Result<PythonDlpackArgument, PythonError> {
        let identity = take_resource(self.identity)?;
        python::prepare_dlpack_argument(identity.into_dlpack_key()?).map(PythonDlpackArgument)
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
    pub fn device_type(&self) -> SifrInt {
        SifrInt::from(self.metadata.device_type)
    }

    #[must_use]
    pub fn device_id(&self) -> SifrInt {
        SifrInt::from(self.metadata.device_id)
    }

    #[must_use]
    pub fn byte_offset(&self) -> SifrInt {
        SifrInt::from(self.metadata.byte_offset)
    }
}

pub struct PythonDlpackArgument(python::PythonDlpackArgument);

impl PythonDlpackArgument {
    #[doc(hidden)]
    pub fn object(&self) -> Result<python::ForeignObject, PythonError> {
        self.0.object()
    }

    #[doc(hidden)]
    pub fn finish(self) -> Result<(), PythonError> {
        self.0.finish()
    }
}

pub fn reconcile_dlpack_argument<T>(
    outcome: Result<T, PythonError>,
    cleanup: Result<(), PythonError>,
) -> Result<T, PythonError> {
    match (outcome, cleanup) {
        (Ok(value), Ok(())) => Ok(value),
        (Ok(_), Err(error)) | (Err(error), _) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::{PythonDlpackStream, PythonDlpackTensor};

    static_assertions::assert_not_impl_any!(PythonDlpackTensor<f64>: Clone, Send, Sync);
    static_assertions::assert_not_impl_any!(PythonDlpackStream: Clone, Send, Sync);
}
