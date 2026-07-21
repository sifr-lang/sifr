use super::{
    close_callback, release_arrow, release_buffer, release_dlpack, ArrowHandle, BufferHandle,
    CallbackHandle, DlpackHandle, PythonError,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResourceKind {
    Callback,
    Buffer,
    Arrow,
    Dlpack,
}

/// Compiler-owned affine identity for Python protocol resources.
///
/// The numeric runtime key is never represented in Sifr. Generated glue stores
/// this value inside `sifr_runtime::interop::Handle`, so ordinary scope exit and
/// explicit consuming release share one exact-once cleanup path.
#[derive(Debug)]
pub struct PythonResourceIdentity {
    kind: ResourceKind,
    key: Option<(i64, i64)>,
}

impl PythonResourceIdentity {
    #[must_use]
    pub const fn callback(handle: CallbackHandle) -> Self {
        Self::new(ResourceKind::Callback, handle)
    }

    #[must_use]
    pub const fn buffer(handle: BufferHandle) -> Self {
        Self::new(ResourceKind::Buffer, handle)
    }

    #[must_use]
    pub const fn arrow(handle: ArrowHandle) -> Self {
        Self::new(ResourceKind::Arrow, handle)
    }

    #[must_use]
    pub const fn dlpack(handle: DlpackHandle) -> Self {
        Self::new(ResourceKind::Dlpack, handle)
    }

    pub fn callback_key(&self) -> Result<CallbackHandle, PythonError> {
        self.key_for(ResourceKind::Callback, "callback")
    }

    pub fn buffer_key(&self) -> Result<BufferHandle, PythonError> {
        self.key_for(ResourceKind::Buffer, "buffer")
    }

    pub fn arrow_key(&self) -> Result<ArrowHandle, PythonError> {
        self.key_for(ResourceKind::Arrow, "Arrow")
    }

    pub fn into_arrow_key(mut self) -> Result<ArrowHandle, PythonError> {
        if self.kind != ResourceKind::Arrow {
            return Err(identity_error(
                "sealed Python resource identity is not an Arrow resource".to_string(),
            ));
        }
        self.key
            .take()
            .ok_or_else(|| identity_error("sealed Python Arrow resource is closed".to_string()))
    }

    pub fn dlpack_key(&self) -> Result<DlpackHandle, PythonError> {
        self.key_for(ResourceKind::Dlpack, "DLPack")
    }

    pub fn into_dlpack_key(mut self) -> Result<DlpackHandle, PythonError> {
        if self.kind != ResourceKind::Dlpack {
            return Err(identity_error(
                "sealed Python resource identity is not a DLPack resource".to_string(),
            ));
        }
        self.key
            .take()
            .ok_or_else(|| identity_error("sealed Python DLPack resource is closed".to_string()))
    }

    pub fn close(mut self) -> Result<(), PythonError> {
        self.release()
    }

    const fn new(kind: ResourceKind, key: (i64, i64)) -> Self {
        Self {
            kind,
            key: Some(key),
        }
    }

    fn key_for(&self, expected: ResourceKind, label: &str) -> Result<(i64, i64), PythonError> {
        if self.kind != expected {
            return Err(identity_error(format!(
                "sealed Python resource identity is not a {label} resource"
            )));
        }
        self.key
            .ok_or_else(|| identity_error(format!("sealed Python {label} resource is closed")))
    }

    fn release(&mut self) -> Result<(), PythonError> {
        let Some(key) = self.key.take() else {
            return Ok(());
        };
        match self.kind {
            ResourceKind::Callback => close_callback(key),
            ResourceKind::Buffer => release_buffer(key),
            ResourceKind::Arrow => release_arrow(key),
            ResourceKind::Dlpack => release_dlpack(key),
        }
    }
}

impl Drop for PythonResourceIdentity {
    fn drop(&mut self) {
        let _ignored = self.release();
    }
}

fn identity_error(message: String) -> PythonError {
    PythonError {
        kind: "resource".to_string(),
        exception_type: "SifrPythonResourceIdentityError".to_string(),
        message,
        traceback: String::new(),
        context: "sealed Python resource identity".to_string(),
        replay: None,
    }
}
