use super::{object_bridge, resource_value, take_resource, Handle, PythonError, PythonObject};
use sifr_runtime::python;

type ResourceIdentity = Handle<python::PythonResourceIdentity>;

#[derive(Debug)]
struct ArrowResource {
    identity: ResourceIdentity,
    metadata: python::PythonArrowCapsuleMetadata,
}

impl ArrowResource {
    fn new(metadata: python::PythonArrowCapsuleMetadata) -> Result<Self, PythonError> {
        if let Err(error) = python::require_arrow_certification(&metadata) {
            let _ignored = python::release_arrow((metadata.handle, metadata.token));
            return Err(error);
        }
        let identity = Handle::new(python::PythonResourceIdentity::arrow((
            metadata.handle,
            metadata.token,
        )));
        Ok(Self { identity, metadata })
    }

    fn release(self) -> Result<(), PythonError> {
        take_resource(self.identity)?.close()
    }

    fn capsule_names(&self) -> Result<Vec<String>, PythonError> {
        python::arrow_capsule_names(resource_value(&self.identity)?.arrow_key()?)
    }

    fn producer_module(&self) -> String {
        self.metadata.producer_module.clone()
    }

    fn producer_type(&self) -> String {
        self.metadata.producer_type.clone()
    }

    fn handle(&self) -> Result<python::ArrowHandle, PythonError> {
        resource_value(&self.identity)?.arrow_key()
    }

    fn prepare_argument(self) -> Result<PythonArrowArgument, PythonError> {
        let identity = take_resource(self.identity)?;
        python::prepare_arrow_argument(identity.into_arrow_key()?).map(PythonArrowArgument)
    }
}

pub struct PythonArrowArgument(python::PythonArrowArgument);

impl PythonArrowArgument {
    #[doc(hidden)]
    pub fn object(&self) -> Result<python::ForeignObject, PythonError> {
        self.0.object()
    }

    #[doc(hidden)]
    pub fn finish(self) -> Result<(), PythonError> {
        self.0.finish()
    }
}

pub fn reconcile_arrow_argument<T>(
    outcome: Result<T, PythonError>,
    cleanup: Result<(), PythonError>,
) -> Result<T, PythonError> {
    match (outcome, cleanup) {
        (Ok(value), Ok(())) => Ok(value),
        (Ok(_), Err(error)) | (Err(error), _) => Err(error),
    }
}

macro_rules! arrow_resource {
    ($name:ident, $acquire:ident) => {
        #[derive(Debug)]
        pub struct $name {
            resource: ArrowResource,
        }

        impl $name {
            pub fn acquire(object: &PythonObject) -> Result<Self, PythonError> {
                Self::acquire_foreign(object_bridge::object_value(object)?)
            }

            #[doc(hidden)]
            pub fn acquire_foreign(object: &python::ForeignObject) -> Result<Self, PythonError> {
                python::$acquire(object)
                    .and_then(ArrowResource::new)
                    .map(|resource| Self { resource })
            }

            pub fn release(self) -> Result<(), PythonError> {
                self.resource.release()
            }

            #[doc(hidden)]
            pub fn prepare_argument(self) -> Result<PythonArrowArgument, PythonError> {
                self.resource.prepare_argument()
            }

            pub fn capsule_names(&self) -> Result<Vec<String>, PythonError> {
                self.resource.capsule_names()
            }

            #[must_use]
            pub fn producer_module(&self) -> String {
                self.resource.producer_module()
            }

            #[must_use]
            pub fn producer_type(&self) -> String {
                self.resource.producer_type()
            }
        }
    };
}

arrow_resource!(PythonArrowArray, arrow_array);
arrow_resource!(PythonArrowSchema, arrow_schema);
arrow_resource!(PythonArrowStream, arrow_stream);
arrow_resource!(PythonArrowDeviceArray, arrow_device_array);
arrow_resource!(PythonArrowDeviceStream, arrow_device_stream);

macro_rules! requested_schema_acquisition {
    ($name:ident, $acquire:ident) => {
        impl $name {
            pub fn acquire_with_schema(
                object: &PythonObject,
                schema: &PythonArrowSchema,
            ) -> Result<Self, PythonError> {
                Self::acquire_foreign_with_schema(object_bridge::object_value(object)?, schema)
            }

            #[doc(hidden)]
            pub fn acquire_foreign_with_schema(
                object: &python::ForeignObject,
                schema: &PythonArrowSchema,
            ) -> Result<Self, PythonError> {
                python::$acquire(object, schema.resource.handle()?)
                    .and_then(ArrowResource::new)
                    .map(|resource| Self { resource })
            }
        }
    };
}

requested_schema_acquisition!(PythonArrowArray, arrow_array_with_schema);
requested_schema_acquisition!(PythonArrowStream, arrow_stream_with_schema);
requested_schema_acquisition!(PythonArrowDeviceArray, arrow_device_array_with_schema);
requested_schema_acquisition!(PythonArrowDeviceStream, arrow_device_stream_with_schema);

#[cfg(test)]
mod tests {
    use super::{
        PythonArrowArray, PythonArrowDeviceArray, PythonArrowDeviceStream, PythonArrowSchema,
        PythonArrowStream,
    };

    static_assertions::assert_not_impl_any!(PythonArrowArray: Clone, Send, Sync);
    static_assertions::assert_not_impl_any!(PythonArrowSchema: Clone, Send, Sync);
    static_assertions::assert_not_impl_any!(PythonArrowStream: Clone, Send, Sync);
    static_assertions::assert_not_impl_any!(PythonArrowDeviceArray: Clone, Send, Sync);
    static_assertions::assert_not_impl_any!(PythonArrowDeviceStream: Clone, Send, Sync);
}
