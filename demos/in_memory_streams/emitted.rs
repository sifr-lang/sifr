// src/main.rs
mod sifr_generated_generated_support {
    use crate::{IOError, SifrGeneratedIoBinaryFileHandle, SifrGeneratedIoNativeFileHandle};
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) fn exists(path: &str) -> bool {
        ::sifr_stdlib::fs::exists(path)
    }
    pub(crate) fn sifr_generated_open_file(path: &str, mode: &str) -> Result<String, IOError> {
        ::sifr_stdlib::fs::open_file(path, mode).map_err(sifr_generated_io_err)
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
    pub(crate) fn open_file(
        path: &str,
        mode: &str,
    ) -> Result<SifrGeneratedIoNativeFileHandle, IOError> {
        let sifr_generated_try_res: Result<
            Result<SifrGeneratedIoNativeFileHandle, IOError>,
            IOError,
        > = (|| {
            let handle_id: String = sifr_generated_open_file(path, mode)?;
            Ok(Ok(SifrGeneratedIoNativeFileHandle::new(handle_id)))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err.clone();
            Err(e)
        })
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
    pub(crate) fn sifr_generated_invalid_whence_error(whence: SifrInt) -> String {
        {
            let mut sifr_generated_concat: String = String::with_capacity(16usize);
            sifr_generated_concat.push_str("invalid whence: ");
            sifr_generated_concat.push_str(whence.to_string().as_str());
            sifr_generated_concat
        }
    }
    pub(crate) fn sifr_generated_negative_seek_error(offset: SifrInt) -> String {
        {
            let mut sifr_generated_concat: String = String::with_capacity(24usize);
            sifr_generated_concat.push_str("negative seek position: ");
            sifr_generated_concat.push_str(offset.to_string().as_str());
            sifr_generated_concat
        }
    }
    pub(crate) fn sifr_generated_mode_is_readable(mode: &str) -> bool {
        mode.contains(&"r".to_string()) || mode.contains(&"+".to_string())
    }
    pub(crate) fn sifr_generated_mode_is_writable(mode: &str) -> bool {
        mode.contains(&"w".to_string())
            || mode.contains(&"a".to_string())
            || mode.contains(&"+".to_string())
    }
    pub(crate) fn open_binary(
        path: &str,
        mode: &str,
    ) -> Result<SifrGeneratedIoBinaryFileHandle, IOError> {
        if !mode.contains(&"b".to_string()) {
            return Err(IOError::new("open_binary requires binary mode".to_string()));
        }
        let sifr_generated_try_res: Result<
            Result<SifrGeneratedIoBinaryFileHandle, IOError>,
            IOError,
        > = (|| {
            let handle: SifrGeneratedIoNativeFileHandle = open_file(path, mode)?;
            Ok(Ok(SifrGeneratedIoBinaryFileHandle::new(
                handle,
                mode.to_owned(),
            )))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err.clone();
            Err(e)
        })
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
    pub struct SifrGeneratedIoBinaryFileHandle {
        pub handle: SifrGeneratedIoNativeFileHandle,
        pub mode: String,
        pub closed: bool,
    }
    impl SifrGeneratedIoBinaryFileHandle {
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
    impl SifrGeneratedIoBinaryFileHandle {
        pub fn close(&mut self) {
            if self.closed {
                return;
            }
            file_close(&self.handle);
            self.closed = true;
        }
    }
    impl SifrGeneratedIoBinaryFileHandle {
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
    impl SifrGeneratedIoBinaryFileHandle {
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
    impl SifrGeneratedIoBinaryFileHandle {
        #[must_use]
        pub fn readable(&self) -> bool {
            sifr_generated_mode_is_readable(&self.mode)
        }
    }
    impl SifrGeneratedIoBinaryFileHandle {
        #[must_use]
        pub fn writable(&self) -> bool {
            sifr_generated_mode_is_writable(&self.mode)
        }
    }
    impl ::std::fmt::Display for SifrGeneratedIoBinaryFileHandle {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f,
                "BinaryFileHandle(_handle={:?}, _mode={}, _closed={})",
                self.handle, self.mode, self.closed
            )
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2eioX2eStringIO {
        pub buffer: String,
        pub cursor: SifrInt,
        pub closed: bool,
    }
    impl SifrGeneratedStdlibSifrX2eioX2eStringIO {
        #[must_use]
        pub fn new(initial: String) -> Self {
            let sifr_generated_field_value_b60ec91c25cb3d78_5f627566666572: String = {
                let mut sifr_generated_concat: String = String::with_capacity(initial.len());
                sifr_generated_concat.push_str(initial.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            let sifr_generated_field_value_d0bd94583b33fdec_5f637572736f72: SifrInt =
                SifrInt::from_i64(0);
            let sifr_generated_field_value_8bc7f577e5ffacda_5f636c6f736564: bool = false;
            Self {
                buffer: sifr_generated_field_value_b60ec91c25cb3d78_5f627566666572,
                cursor: sifr_generated_field_value_d0bd94583b33fdec_5f637572736f72,
                closed: sifr_generated_field_value_8bc7f577e5ffacda_5f636c6f736564,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2eioX2eStringIO {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn read(&mut self, size: &Option<SifrInt>) -> Result<String, IOError> {
            if self.closed {
                return Err(IOError::new(sifr_generated_closed_stream_error()));
            }
            let start: SifrInt = self.cursor.clone();
            let mut end: SifrInt = SifrInt::from(self.buffer.chars().count());
            if let Some(size) = size.as_ref() {
                let maybe_size: SifrInt = size.clone();
                if &maybe_size >= &SifrInt::from_i64(0) {
                    let requested: SifrInt = &start + &maybe_size;
                    if &requested < &end {
                        end = requested.clone();
                    }
                }
            }
            let piece: String = {
                let sifr_generated_slice_src = self.buffer.clone().chars().collect::<Vec<char>>();
                let sifr_generated_slice_len = sifr_generated_slice_src.len();
                let sifr_generated_slice_start = start.clamp_slice_bound(sifr_generated_slice_len);
                let sifr_generated_slice_stop = end.clamp_slice_bound(sifr_generated_slice_len);
                String::from_iter(
                    sifr_generated_slice_src
                        .iter()
                        .skip(sifr_generated_slice_start)
                        .take(sifr_generated_slice_stop.saturating_sub(sifr_generated_slice_start))
                        .copied(),
                )
            };
            self.cursor = end.clone();
            Ok(piece)
        }
    }
    impl SifrGeneratedStdlibSifrX2eioX2eStringIO {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn write(&mut self, data: &str) -> Result<(), IOError> {
            if self.closed {
                return Err(IOError::new(sifr_generated_closed_stream_error()));
            }
            let left: String = {
                let sifr_generated_slice_src = self.buffer.clone().chars().collect::<Vec<char>>();
                let sifr_generated_slice_len = sifr_generated_slice_src.len();
                let sifr_generated_slice_start = 0;
                let sifr_generated_slice_stop = self
                    .cursor
                    .clone()
                    .clamp_slice_bound(sifr_generated_slice_len);
                String::from_iter(
                    sifr_generated_slice_src
                        .iter()
                        .skip(sifr_generated_slice_start)
                        .take(sifr_generated_slice_stop.saturating_sub(sifr_generated_slice_start))
                        .copied(),
                )
            };
            let tail_start: SifrInt = &self.cursor.clone() + &SifrInt::from(data.chars().count());
            let right: String = if &tail_start < &SifrInt::from(self.buffer.chars().count()) {
                {
                    let sifr_generated_slice_src =
                        self.buffer.clone().chars().collect::<Vec<char>>();
                    let sifr_generated_slice_len = sifr_generated_slice_src.len();
                    let sifr_generated_slice_start =
                        tail_start.clamp_slice_bound(sifr_generated_slice_len);
                    let sifr_generated_slice_stop = sifr_generated_slice_len;
                    String::from_iter(
                        sifr_generated_slice_src
                            .iter()
                            .skip(sifr_generated_slice_start)
                            .take(
                                sifr_generated_slice_stop
                                    .saturating_sub(sifr_generated_slice_start),
                            )
                            .copied(),
                    )
                }
            } else {
                String::new()
            };
            self.buffer = {
                let mut sifr_generated_concat: String =
                    String::with_capacity(left.len() + data.len() + right.len());
                sifr_generated_concat.push_str(left.as_str());
                sifr_generated_concat.push_str(data);
                sifr_generated_concat.push_str(right.as_str());
                sifr_generated_concat
            };
            self.cursor = &self.cursor.clone() + &SifrInt::from(data.chars().count());
            Ok(())
        }
    }
    impl SifrGeneratedStdlibSifrX2eioX2eStringIO {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn seek(&mut self, offset: &SifrInt, whence: &SifrInt) -> Result<SifrInt, IOError> {
            if self.closed {
                return Err(IOError::new(sifr_generated_closed_stream_error()));
            }
            let origin: SifrInt = if whence == &SifrInt::from_i64(0) {
                SifrInt::from_i64(0)
            } else if whence == &SifrInt::from_i64(1) {
                self.cursor.clone()
            } else if whence == &SifrInt::from_i64(2) {
                SifrInt::from(self.buffer.chars().count())
            } else {
                return Err(IOError::new(sifr_generated_invalid_whence_error(
                    whence.clone(),
                )));
            };
            let mut next_pos: SifrInt = &origin + offset;
            if &next_pos < &SifrInt::from_i64(0) {
                return Err(IOError::new(sifr_generated_negative_seek_error(
                    next_pos.clone(),
                )));
            }
            let end: SifrInt = SifrInt::from(self.buffer.chars().count());
            if &next_pos > &end {
                next_pos = end.clone();
            }
            self.cursor = next_pos.clone();
            Ok(self.cursor.clone())
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2eioX2eStringIO {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f,
                "StringIO(_buffer={}, _cursor={}, _closed={})",
                self.buffer, self.cursor, self.closed
            )
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2eioX2eBytesIO {
        pub buffer: Vec<u8>,
        pub cursor: SifrInt,
        pub closed: bool,
    }
    impl SifrGeneratedStdlibSifrX2eioX2eBytesIO {
        #[must_use]
        pub const fn new(initial: Vec<u8>) -> Self {
            let sifr_generated_field_value_b60ec91c25cb3d78_5f627566666572: Vec<u8> = initial;
            let sifr_generated_field_value_d0bd94583b33fdec_5f637572736f72: SifrInt =
                SifrInt::from_i64(0);
            let sifr_generated_field_value_8bc7f577e5ffacda_5f636c6f736564: bool = false;
            Self {
                buffer: sifr_generated_field_value_b60ec91c25cb3d78_5f627566666572,
                cursor: sifr_generated_field_value_d0bd94583b33fdec_5f637572736f72,
                closed: sifr_generated_field_value_8bc7f577e5ffacda_5f636c6f736564,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2eioX2eBytesIO {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn read_bytes(&mut self, size: &Option<SifrInt>) -> Result<Vec<u8>, IOError> {
            if self.closed {
                return Err(IOError::new(sifr_generated_closed_stream_error()));
            }
            let start: SifrInt = self.cursor.clone();
            let mut end: SifrInt = SifrInt::from(self.buffer.len());
            if let Some(size) = size.as_ref() {
                let maybe_size: SifrInt = size.clone();
                if &maybe_size >= &SifrInt::from_i64(0) {
                    let requested: SifrInt = &start + &maybe_size;
                    if &requested < &end {
                        end = requested.clone();
                    }
                }
            }
            let chunk: Vec<u8> = {
                let sifr_generated_slice_src = &self.buffer.clone();
                let sifr_generated_slice_len = sifr_generated_slice_src.len();
                let sifr_generated_slice_start = start.clamp_slice_bound(sifr_generated_slice_len);
                let sifr_generated_slice_stop = end.clamp_slice_bound(sifr_generated_slice_len);
                Vec::from_iter(
                    sifr_generated_slice_src
                        .iter()
                        .skip(sifr_generated_slice_start)
                        .take(sifr_generated_slice_stop.saturating_sub(sifr_generated_slice_start))
                        .cloned(),
                )
            };
            self.cursor = end.clone();
            Ok(chunk)
        }
    }
    impl SifrGeneratedStdlibSifrX2eioX2eBytesIO {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn write_bytes(&mut self, data: &[u8]) -> Result<(), IOError> {
            if self.closed {
                return Err(IOError::new(sifr_generated_closed_stream_error()));
            }
            if &self.cursor.clone() == &SifrInt::from(self.buffer.len()) {
                self.buffer = {
                    let mut sifr_generated_v = self.buffer.clone().to_vec();
                    sifr_generated_v.extend(data.iter().cloned());
                    sifr_generated_v
                };
                self.cursor = &self.cursor.clone() + &SifrInt::from(data.len());
                return Ok(());
            }
            let left: Vec<u8> = {
                let sifr_generated_slice_src = &self.buffer.clone();
                let sifr_generated_slice_len = sifr_generated_slice_src.len();
                let sifr_generated_slice_start = 0;
                let sifr_generated_slice_stop = self
                    .cursor
                    .clone()
                    .clamp_slice_bound(sifr_generated_slice_len);
                Vec::from_iter(
                    sifr_generated_slice_src
                        .iter()
                        .skip(sifr_generated_slice_start)
                        .take(sifr_generated_slice_stop.saturating_sub(sifr_generated_slice_start))
                        .cloned(),
                )
            };
            let tail_start: SifrInt = &self.cursor.clone() + &SifrInt::from(data.len());
            let right: Vec<u8> = if &tail_start < &SifrInt::from(self.buffer.len()) {
                {
                    let sifr_generated_slice_src = &self.buffer.clone();
                    let sifr_generated_slice_len = sifr_generated_slice_src.len();
                    let sifr_generated_slice_start =
                        tail_start.clamp_slice_bound(sifr_generated_slice_len);
                    let sifr_generated_slice_stop = sifr_generated_slice_len;
                    Vec::from_iter(
                        sifr_generated_slice_src
                            .iter()
                            .skip(sifr_generated_slice_start)
                            .take(
                                sifr_generated_slice_stop
                                    .saturating_sub(sifr_generated_slice_start),
                            )
                            .cloned(),
                    )
                }
            } else {
                Vec::new()
            };
            self.buffer = {
                let mut sifr_generated_v = {
                    let mut sifr_generated_v = left.to_vec();
                    sifr_generated_v.extend(data.iter().cloned());
                    sifr_generated_v
                }
                .to_vec();
                sifr_generated_v.extend(right.iter().cloned());
                sifr_generated_v
            };
            self.cursor = &self.cursor.clone() + &SifrInt::from(data.len());
            Ok(())
        }
    }
    impl SifrGeneratedStdlibSifrX2eioX2eBytesIO {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn seek(&mut self, offset: &SifrInt, whence: &SifrInt) -> Result<SifrInt, IOError> {
            if self.closed {
                return Err(IOError::new(sifr_generated_closed_stream_error()));
            }
            let origin: SifrInt = if whence == &SifrInt::from_i64(0) {
                SifrInt::from_i64(0)
            } else if whence == &SifrInt::from_i64(1) {
                self.cursor.clone()
            } else if whence == &SifrInt::from_i64(2) {
                SifrInt::from(self.buffer.len())
            } else {
                return Err(IOError::new(sifr_generated_invalid_whence_error(
                    whence.clone(),
                )));
            };
            let mut next_pos: SifrInt = &origin + offset;
            if &next_pos < &SifrInt::from_i64(0) {
                return Err(IOError::new(sifr_generated_negative_seek_error(
                    next_pos.clone(),
                )));
            }
            let end: SifrInt = SifrInt::from(self.buffer.len());
            if &next_pos > &end {
                next_pos = end.clone();
            }
            self.cursor = next_pos.clone();
            Ok(self.cursor.clone())
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
pub use sifr_generated_project_nominals::SifrGeneratedIoBinaryFileHandle;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eioX2eBytesIO;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eioX2eStringIO;
fn main() {
    let path: String = "/tmp/sifr_runtime_in_memory_streams.bin".to_string();
    let mut stringio_ok: bool = false;
    let mut stringio_negative_seek_ok: bool = false;
    let mut bytesio_ok: bool = false;
    let mut bytesio_negative_seek_ok: bool = false;
    let mut binary_file_ok: bool = false;
    let mut cleanup_ok: bool = false;
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let mut sio: SifrGeneratedStdlibSifrX2eioX2eStringIO =
            SifrGeneratedStdlibSifrX2eioX2eStringIO::new("sample".to_string());
        (&mut sio).write(&"1".to_string())?;
        let _seek: SifrInt = (&mut sio).seek(&SifrInt::from_i64(0), &SifrInt::from_i64(0))?;
        let text_value: String = (&mut sio).read(&None)?;
        stringio_ok = text_value == "1ample";
        let sifr_generated_try_res: Result<(), IOError> = (|| {
            let sifr_generated_bad_seek: SifrInt =
                (&mut sio).seek(&-SifrInt::from_i64(1), &SifrInt::from_i64(0))?;
            let _ = sifr_generated_bad_seek;
            Ok(())
        })();
        if let Err(sifr_generated_try_err) = sifr_generated_try_res {
            let e = sifr_generated_try_err.clone();
            let _ = e.message.clone().to_string();
            stringio_negative_seek_ok = true;
        }
        let mut bio: SifrGeneratedStdlibSifrX2eioX2eBytesIO =
            SifrGeneratedStdlibSifrX2eioX2eBytesIO::new(vec![97_u8, 98_u8, 99_u8]);
        let _seek_b_value_78f19d0c500eec0b: SifrInt =
            (&mut bio).seek(&SifrInt::from_i64(3), &SifrInt::from_i64(0))?;
        (&mut bio).write_bytes(&vec![100_u8])?;
        let _seek_b0: SifrInt = (&mut bio).seek(&SifrInt::from_i64(0), &SifrInt::from_i64(0))?;
        let bytes_value: Vec<u8> = (&mut bio).read_bytes(&None)?;
        bytesio_ok = bytes_value == vec![97_u8, 98_u8, 99_u8, 100_u8];
        let sifr_generated_try_res: Result<(), IOError> = (|| {
            let sifr_generated_bad_seek_b: SifrInt =
                (&mut bio).seek(&-SifrInt::from_i64(1), &SifrInt::from_i64(0))?;
            let _ = sifr_generated_bad_seek_b;
            Ok(())
        })();
        if let Err(sifr_generated_try_err) = sifr_generated_try_res {
            let e = sifr_generated_try_err.clone();
            let _ = e.message.clone().to_string();
            bytesio_negative_seek_ok = true;
        }
        let mut writer: SifrGeneratedIoBinaryFileHandle = open_binary(&path, &"wb".to_string())?;
        writer.write_bytes(&vec![
            114_u8, 117_u8, 110_u8, 116_u8, 105_u8, 109_u8, 101_u8, 45_u8, 105_u8, 110_u8, 95_u8,
            109_u8, 101_u8, 109_u8, 111_u8, 114_u8, 121_u8, 95_u8, 115_u8, 116_u8, 114_u8, 101_u8,
            97_u8, 109_u8, 115_u8,
        ])?;
        (&mut writer).close();
        let mut reader: SifrGeneratedIoBinaryFileHandle = open_binary(&path, &"rb".to_string())?;
        let loaded: Vec<u8> = reader.read_bytes(&None)?;
        (&mut reader).close();
        binary_file_ok = loaded
            == vec![
                114_u8, 117_u8, 110_u8, 116_u8, 105_u8, 109_u8, 101_u8, 45_u8, 105_u8, 110_u8,
                95_u8, 109_u8, 101_u8, 109_u8, 111_u8, 114_u8, 121_u8, 95_u8, 115_u8, 116_u8,
                114_u8, 101_u8, 97_u8, 109_u8, 115_u8,
            ];
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
    assert!(stringio_ok);
    assert!(stringio_negative_seek_ok);
    assert!(bytesio_ok);
    assert!(bytesio_negative_seek_ok);
    assert!(binary_file_ok);
    assert!(cleanup_ok);
    println!("runtime_in_memory_streams_in_memory_hierarchy_demo: ok");
}
