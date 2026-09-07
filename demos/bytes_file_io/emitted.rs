// src/main.rs
mod sifr_generated_generated_support {
    use crate::{IOError, SifrGeneratedIoNativeFileHandle};
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) fn exists(path: &str) -> bool {
        ::sifr_stdlib::fs::exists(path)
    }
    pub(crate) fn sifr_generated_file_close(handle: &str) {
        ::sifr_stdlib::fs::file_close(handle);
    }
    pub(crate) fn sifr_generated_file_read_bytes(
        handle: &str,
        size: Option<SifrInt>,
    ) -> Result<Vec<u8>, IOError> {
        ::sifr_stdlib::fs::file_read_bytes(
            handle,
            size.map(::sifr_runtime::interop::SifrIntBridge::from),
        )
        .map_err(sifr_generated_io_err)
    }
    pub(crate) fn sifr_generated_file_write_bytes(
        handle: &str,
        data: &[u8],
    ) -> Result<(), IOError> {
        ::sifr_stdlib::fs::file_write_bytes(handle, data).map_err(sifr_generated_io_err)
    }
    pub(crate) fn file_close(handle: &SifrGeneratedIoNativeFileHandle) {
        sifr_generated_file_close(&handle.id.clone());
    }
    pub(crate) fn file_read_bytes(
        handle: &SifrGeneratedIoNativeFileHandle,
        size: Option<SifrInt>,
    ) -> Result<Vec<u8>, IOError> {
        sifr_generated_file_read_bytes(&handle.id.clone(), size.clone())
    }
    pub(crate) fn file_write_bytes(
        handle: &SifrGeneratedIoNativeFileHandle,
        data: &[u8],
    ) -> Result<(), IOError> {
        sifr_generated_file_write_bytes(&handle.id.clone(), data)
    }
    pub(crate) fn remove_file(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::remove_file(path).map_err(sifr_generated_io_err)
    }
    pub(crate) fn sifr_generated_closed_stream_error() -> String {
        "I/O operation on closed stream".to_string()
    }
    pub(crate) fn sifr_generated_mode_is_readable(mode: &str) -> bool {
        mode.contains(&"r".to_string()) || mode.contains(&"+".to_string())
    }
    pub(crate) fn sifr_generated_mode_is_writable(mode: &str) -> bool {
        mode.contains(&"w".to_string())
            || mode.contains(&"a".to_string())
            || mode.contains(&"+".to_string())
    }
    pub(crate) fn sifr_generated_io_err<E: ::std::fmt::Display + 'static>(e: E) -> IOError {
        let msg = e.to_string();
        let kind = {
            let sifr_generated_io_kind = (&e as &dyn ::std::any::Any)
                .downcast_ref::<std::io::Error>()
                .map(::std::io::Error::kind);
            match sifr_generated_io_kind {
                Some(::std::io::ErrorKind::NotFound) => "FileNotFound".to_string(),
                Some(::std::io::ErrorKind::PermissionDenied) => "PermissionDenied".to_string(),
                Some(::std::io::ErrorKind::AlreadyExists) => "FileExists".to_string(),
                Some(::std::io::ErrorKind::IsADirectory) => "IsADirectory".to_string(),
                Some(::std::io::ErrorKind::NotADirectory) => "NotADirectory".to_string(),
                Some(::std::io::ErrorKind::DirectoryNotEmpty) => "DirectoryNotEmpty".to_string(),
                _ => "Other".to_string(),
            }
        };
        IOError { message: msg, kind }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SifrGeneratedIoNativeFileHandle {
    pub id: String,
}
impl SifrGeneratedIoNativeFileHandle {
    #[must_use]
    pub const fn new(id: String) -> Self {
        let sifr_generated_field_value_b90e3b1a0ca5e613_5f6964: String = id;
        Self {
            id: sifr_generated_field_value_b90e3b1a0ca5e613_5f6964,
        }
    }
}
impl ::std::fmt::Display for SifrGeneratedIoNativeFileHandle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "NativeFileHandle(_id={})", self.id)
    }
}
mod sifr_generated_project_nominals {
    use crate::SifrGeneratedIoNativeFileHandle;
    use crate::sifr_generated_generated_support::*;
    use ::sifr_runtime::SifrInt;
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedIoFileHandle {
        pub handle: SifrGeneratedIoNativeFileHandle,
        pub mode: String,
        pub closed: bool,
    }
    impl SifrGeneratedIoFileHandle {
        #[must_use]
        pub const fn new(handle: SifrGeneratedIoNativeFileHandle, mode: String) -> Self {
            let sifr_generated_field_value_b31dc5f344797918_5f68616e646c65: SifrGeneratedIoNativeFileHandle = handle;
            let sifr_generated_field_value_e0efc38c5ec2afd5_5f6d6f6465: String = mode;
            let sifr_generated_field_value_8bc7f577e5ffacda_5f636c6f736564: bool = false;
            Self {
                handle: sifr_generated_field_value_b31dc5f344797918_5f68616e646c65,
                mode: sifr_generated_field_value_e0efc38c5ec2afd5_5f6d6f6465,
                closed: sifr_generated_field_value_8bc7f577e5ffacda_5f636c6f736564,
            }
        }
    }
    impl SifrGeneratedIoFileHandle {
        pub fn close(&mut self) {
            if self.closed {
                return;
            }
            file_close(&self.handle);
            self.closed = true;
        }
    }
    impl SifrGeneratedIoFileHandle {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn read_bytes(&self, size: &Option<SifrInt>) -> Result<Vec<u8>, IOError> {
            if self.closed {
                return Err(IOError::new(sifr_generated_closed_stream_error()));
            }
            if !self.readable() {
                return Err(IOError::new("stream is not readable".to_string()));
            }
            file_read_bytes(&self.handle, size.clone())
        }
    }
    impl SifrGeneratedIoFileHandle {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn write_bytes(&self, data: &[u8]) -> Result<(), IOError> {
            if self.closed {
                return Err(IOError::new(sifr_generated_closed_stream_error()));
            }
            if !self.writable() {
                return Err(IOError::new("stream is not writable".to_string()));
            }
            file_write_bytes(&self.handle, data)
        }
    }
    impl SifrGeneratedIoFileHandle {
        #[must_use]
        pub fn readable(&self) -> bool {
            sifr_generated_mode_is_readable(&self.mode)
        }
    }
    impl SifrGeneratedIoFileHandle {
        #[must_use]
        pub fn writable(&self) -> bool {
            sifr_generated_mode_is_writable(&self.mode)
        }
    }
    impl ::std::fmt::Display for SifrGeneratedIoFileHandle {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f,
                "FileHandle(_handle={:?}, _mode={}, _closed={})",
                self.handle, self.mode, self.closed
            )
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct IOError {
        pub message: String,
        pub kind: String,
    }
    impl IOError {
        #[must_use]
        pub fn new(message: String) -> Self {
            Self {
                message,
                kind: "Other".to_string(),
            }
        }
    }
    impl ::std::fmt::Display for IOError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for IOError {}
}
use crate::sifr_generated_generated_support::*;
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::IOError;
pub use sifr_generated_project_nominals::SifrGeneratedIoFileHandle;
fn main() {
    let path: String = "/tmp/sifr_bytes_bytes_file_io.bin".to_string();
    let mut loaded_ok: bool = false;
    let mut ints_ok: bool = false;
    let mut cleanup_ok: bool = false;
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let mut writer: SifrGeneratedIoFileHandle = (|| {
            let sifr_generated_path = path.to_string();
            let sifr_generated_mode = "wb".to_string();
            let sifr_generated_handle_id = ::sifr_stdlib::fs::open_file(
                sifr_generated_path.as_str(),
                sifr_generated_mode.as_str(),
            )
            .map_err(sifr_generated_io_err)?;
            Ok::<SifrGeneratedIoFileHandle, IOError>(SifrGeneratedIoFileHandle::new(
                SifrGeneratedIoNativeFileHandle::new(sifr_generated_handle_id),
                sifr_generated_mode.to_string(),
            ))
        })()?;
        (&mut writer).write_bytes(&vec![
            98_u8, 121_u8, 116_u8, 101_u8, 115_u8, 95_u8, 102_u8, 105_u8, 108_u8, 101_u8, 95_u8,
            105_u8, 111_u8,
        ])?;
        (&mut writer).close();
        let mut reader: SifrGeneratedIoFileHandle = (|| {
            let sifr_generated_path = path.to_string();
            let sifr_generated_mode = "rb".to_string();
            let sifr_generated_handle_id = ::sifr_stdlib::fs::open_file(
                sifr_generated_path.as_str(),
                sifr_generated_mode.as_str(),
            )
            .map_err(sifr_generated_io_err)?;
            Ok::<SifrGeneratedIoFileHandle, IOError>(SifrGeneratedIoFileHandle::new(
                SifrGeneratedIoNativeFileHandle::new(sifr_generated_handle_id),
                sifr_generated_mode.to_string(),
            ))
        })()?;
        let loaded: Vec<u8> = reader.read_bytes(&None)?;
        (&mut reader).close();
        loaded_ok = loaded
            == vec![
                98_u8, 121_u8, 116_u8, 101_u8, 115_u8, 95_u8, 102_u8, 105_u8, 108_u8, 101_u8,
                95_u8, 105_u8, 111_u8,
            ];
        ints_ok = format!(
            "{:?}",
            loaded
                .iter()
                .map(|sifr_generated_byte| SifrInt::from(*sifr_generated_byte))
                .collect::<Vec<SifrInt>>()
        ) == "[98, 121, 116, 101, 115, 95, 102, 105, 108, 101, 95, 105, 111]";
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
    }
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        if exists(&path) {
            remove_file(&path)?;
        }
        cleanup_ok = !exists(&path);
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
    }
    assert!(loaded_ok);
    assert!(ints_ok);
    assert!(cleanup_ok);
}
