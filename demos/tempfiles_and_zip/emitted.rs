// src/main.rs
mod __sifr_project_nominals {
    pub fn _gzip_compress_bytes_impl(data: &String) -> Vec<u8> {
        ::sifr_stdlib::gzip::gzip_compress_bytes(data)
    }
    pub fn _gzip_decompress_bytes_impl(data: &Vec<u8>) -> Result<String, IOError> {
        ::sifr_stdlib::gzip::gzip_decompress_bytes(data)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn zip_create(path: &String) -> Result<(), IOError> {
        ::sifr_stdlib::zipfile::zip_create(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn zip_add_file(
        zip_path: &String,
        name: &String,
        content: &String,
    ) -> Result<(), IOError> {
        ::sifr_stdlib::zipfile::zip_add_file(zip_path, name, content)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn zip_add_file_bytes(
        zip_path: &String,
        name: &String,
        content: &Vec<u8>,
    ) -> Result<(), IOError> {
        ::sifr_stdlib::zipfile::zip_add_file_bytes(zip_path, name, content)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn zip_read_file(zip_path: &String, name: &String) -> Result<String, IOError> {
        ::sifr_stdlib::zipfile::zip_read_file(zip_path, name)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn zip_read_file_bytes(
        zip_path: &String,
        name: &String,
    ) -> Result<Vec<u8>, IOError> {
        ::sifr_stdlib::zipfile::zip_read_file_bytes(zip_path, name)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn zip_namelist(zip_path: &String) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::zipfile::zip_namelist(zip_path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2ezipfile_x2eZipInfo {
        pub filename: String,
        pub file_size: i64,
        pub compress_type: i64,
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipInfo {
        pub fn new(filename: String, file_size: i64, compress_type: i64) -> Self {
            let __sifr_field_init_0: String = {
                let mut __sifr_concat: String = String::with_capacity(
                    filename.len() + 0usize,
                );
                __sifr_concat.push_str((filename).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
            let __sifr_field_init_1: i64 = file_size;
            let __sifr_field_init_2: i64 = compress_type;
            Self {
                filename: __sifr_field_init_0,
                file_size: __sifr_field_init_1,
                compress_type: __sifr_field_init_2,
            }
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipInfo {}
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2ezipfile_x2eZipInfo {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f, "ZipInfo(filename={}, file_size={}, compress_type={})", self.filename,
                self.file_size, self.compress_type
            )
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2ezipfile_x2eZipReadHandle {
        pub _data: Vec<u8>,
        pub _cursor: i64,
        pub _closed: bool,
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipReadHandle {
        pub fn new(data: Vec<u8>) -> Self {
            let __sifr_field_init_0: Vec<u8> = data;
            let __sifr_field_init_1: i64 = 0_i64;
            let __sifr_field_init_2: bool = false;
            Self {
                _data: __sifr_field_init_0,
                _cursor: __sifr_field_init_1,
                _closed: __sifr_field_init_2,
            }
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipReadHandle {
        pub fn close(&mut self) {
            self._closed = true;
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipReadHandle {
        pub fn closed(&self) -> bool {
            self._closed
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipReadHandle {
        pub fn read_bytes(&mut self, size: Option<i64>) -> Result<Vec<u8>, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            let mut end: i64 = self._data.len() as i64;
            if let Some(size) = size {
                let requested_size: i64 = size;
                if requested_size < (0_i64) {
                    end = self._data.len() as i64;
                } else {
                    let requested_end: i64 = self._cursor + requested_size;
                    if requested_end < end {
                        end = requested_end;
                    }
                }
            }
            let out: Vec<u8> = {
                let _slice_src = &self._data.clone();
                let _slice_len_i64 = _slice_src.len() as i64;
                let _slice_start_i64 = if self._cursor < 0 {
                    (_slice_len_i64 + self._cursor).max(0)
                } else {
                    self._cursor.min(_slice_len_i64)
                };
                let _slice_stop_i64 = if end < 0 {
                    (_slice_len_i64 + end).max(0)
                } else {
                    end.min(_slice_len_i64)
                };
                Vec::from_iter(
                    _slice_src
                        .iter()
                        .skip(_slice_start_i64 as usize)
                        .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                        .cloned(),
                )
            };
            self._cursor = end;
            Ok(out)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub path: String,
        pub mode: String,
        pub compression: i64,
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn new(path: String, mode: String, compression: i64) -> Self {
            let __sifr_field_init_0: String = {
                let mut __sifr_concat: String = String::with_capacity(path.len() + 0usize);
                __sifr_concat.push_str((path).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
            let __sifr_field_init_1: String = {
                let mut __sifr_concat: String = String::with_capacity(mode.len() + 0usize);
                __sifr_concat.push_str((mode).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
            let __sifr_field_init_2: i64 = compression;
            Self {
                path: __sifr_field_init_0,
                mode: __sifr_field_init_1,
                compression: __sifr_field_init_2,
            }
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn _writable_mode(&self) -> bool {
            (((((self.mode.clone() == "w")) || ((self.mode.clone() == "a")))
                || ((self.mode.clone() == "wb"))) || ((self.mode.clone() == "ab")))
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn create(&self) -> Result<(), IOError> {
            zip_create(&self.path)
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn write(&self, name: &String, content: &String) -> Result<(), IOError> {
            if !(self._writable_mode()) {
                return Err(IOError::new(_zip_read_only_error()));
            }
            zip_add_file(&self.path, name, content)
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn write_bytes(&self, name: &String, content: &Vec<u8>) -> Result<(), IOError> {
            if !(self._writable_mode()) {
                return Err(IOError::new(_zip_read_only_error()));
            }
            zip_add_file_bytes(&self.path, name, content)
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn read(&self, name: &String) -> Result<String, IOError> {
            zip_read_file(&self.path, name)
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn read_bytes(&self, name: &String) -> Result<Vec<u8>, IOError> {
            zip_read_file_bytes(&self.path, name)
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn namelist(&self) -> Result<Vec<String>, IOError> {
            zip_namelist(&self.path)
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn infolist(
            &self,
        ) -> Result<Vec<__SifrStdlib_sifr_x2ezipfile_x2eZipInfo>, IOError> {
            Err(IOError::new(_zip_unimplemented_error(&"infolist".to_string())))
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn getinfo(
            &self,
            name: &String,
        ) -> Result<__SifrStdlib_sifr_x2ezipfile_x2eZipInfo, IOError> {
            let _ = (name).clone();
            Err(IOError::new(_zip_unimplemented_error(&"getinfo".to_string())))
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn open(
            &self,
            name: &String,
            mode: &String,
        ) -> Result<__SifrStdlib_sifr_x2ezipfile_x2eZipReadHandle, IOError> {
            let _ = (name).clone();
            if ((mode).as_str() != "r") && ((mode).as_str() != "rb") {
                return Err(IOError::new(_zip_open_mode_error(mode)));
            }
            Err(IOError::new(_zip_unimplemented_error(&"open".to_string())))
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn extract(&self, name: &String, path: &String) -> Result<String, IOError> {
            let _ = (name).clone();
            let _ = (path).clone();
            Err(IOError::new(_zip_unimplemented_error(&"extract".to_string())))
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn extractall(&self, path: &String) -> Result<Vec<String>, IOError> {
            let _ = (path).clone();
            Err(IOError::new(_zip_unimplemented_error(&"extractall".to_string())))
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn __enter__(&self) -> __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
            self.clone()
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn __exit__(&self) {}
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f, "ZipFile(path={}, mode={}, compression={})", self.path, self.mode, self
                .compression
            )
        }
    }
    pub fn _zip_read_only_error() -> String {
        "zipfile operation requires write or append mode".to_string()
    }
    pub fn _zip_open_mode_error(mode: &String) -> String {
        {
            let mut __sifr_concat: String = String::with_capacity(48usize + mode.len());
            __sifr_concat.push_str("zipfile open supports read-only mode only, got: ");
            __sifr_concat.push_str((mode).as_str());
            __sifr_concat
        }
    }
    pub fn _closed_stream_error() -> String {
        "I/O operation on closed stream".to_string()
    }
    pub fn _zip_unimplemented_error(feature: &String) -> String {
        {
            let mut __sifr_concat: String = String::with_capacity(
                (8usize + feature.len()) + 49usize,
            );
            __sifr_concat.push_str("zipfile ");
            __sifr_concat.push_str((feature).as_str());
            __sifr_concat.push_str(" is not implemented in this compatibility surface");
            __sifr_concat
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct IOError {
        pub message: String,
        pub kind: String,
    }
    impl IOError {
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
    pub fn __io_err<E: ::std::fmt::Display + 'static>(e: E) -> IOError {
        let msg = e.to_string();
        let kind = {
            let __sifr_io_kind = (&e as &dyn ::std::any::Any)
                .downcast_ref::<std::io::Error>()
                .map(::std::io::Error::kind);
            match __sifr_io_kind {
                Some(::std::io::ErrorKind::NotFound) => "FileNotFound".to_string(),
                Some(::std::io::ErrorKind::PermissionDenied) => {
                    "PermissionDenied".to_string()
                }
                Some(::std::io::ErrorKind::AlreadyExists) => "FileExists".to_string(),
                Some(::std::io::ErrorKind::IsADirectory) => "IsADirectory".to_string(),
                Some(::std::io::ErrorKind::NotADirectory) => "NotADirectory".to_string(),
                Some(::std::io::ErrorKind::DirectoryNotEmpty) => {
                    "DirectoryNotEmpty".to_string()
                }
                _ => "Other".to_string(),
            }
        };
        IOError { message: msg, kind }
    }
}
pub use __sifr_project_nominals::IOError;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ezipfile_x2eZipFile;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ezipfile_x2eZipInfo;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ezipfile_x2eZipReadHandle;
fn _encoding_is_supported_impl(label: &String) -> bool {
    ::sifr_stdlib::encoding::encoding_is_supported(label)
}
fn _encoding_canonical_label_impl(label: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::encoding::encoding_canonical_label(label)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _encoding_decode_text_impl(
    data: &Vec<u8>,
    encoding: &String,
    errors: &String,
) -> Result<String, ParseError> {
    ::sifr_stdlib::encoding::encoding_decode_text(data, encoding, errors)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _encoding_decode_recoveries_impl(
    data: &Vec<u8>,
    encoding: &String,
    errors: &String,
) -> Result<Vec<String>, ParseError> {
    ::sifr_stdlib::encoding::encoding_decode_recoveries(data, encoding, errors)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _encoding_decode_incremental_text_impl(
    data: &Vec<u8>,
    pending: &Vec<u8>,
    encoding: &String,
    errors: &String,
    r#final: bool,
) -> Result<String, ParseError> {
    ::sifr_stdlib::encoding::encoding_decode_incremental_text(
            data,
            pending,
            encoding,
            errors,
            r#final,
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _encoding_decode_incremental_recoveries_impl(
    data: &Vec<u8>,
    pending: &Vec<u8>,
    encoding: &String,
    errors: &String,
    r#final: bool,
) -> Result<Vec<String>, ParseError> {
    ::sifr_stdlib::encoding::encoding_decode_incremental_recoveries(
            data,
            pending,
            encoding,
            errors,
            r#final,
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _encoding_decode_incremental_pending_impl(
    data: &Vec<u8>,
    pending: &Vec<u8>,
    encoding: &String,
    r#final: bool,
) -> Result<Vec<u8>, ParseError> {
    ::sifr_stdlib::encoding::encoding_decode_incremental_pending(
            data,
            pending,
            encoding,
            r#final,
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _encoding_encode_bytes_impl(
    text: &String,
    encoding: &String,
    errors: &String,
) -> Result<Vec<u8>, ParseError> {
    ::sifr_stdlib::encoding::encoding_encode_bytes(text, encoding, errors)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _encoding_encode_recoveries_impl(
    text: &String,
    encoding: &String,
    errors: &String,
) -> Result<Vec<String>, ParseError> {
    ::sifr_stdlib::encoding::encoding_encode_recoveries(text, encoding, errors)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrIoNativeFileHandle {
    _id: String,
}
impl __SifrIoNativeFileHandle {
    fn new(id: String) -> Self {
        let __sifr_field_init_0: String = id;
        Self { _id: __sifr_field_init_0 }
    }
}
impl __SifrIoNativeFileHandle {}
impl ::std::fmt::Display for __SifrIoNativeFileHandle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "NativeFileHandle(_id={})", self._id)
    }
}
fn read_text(path: &String) -> Result<String, IOError> {
    ::sifr_stdlib::fs::read_text(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn write_text(path: &String, content: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::write_text(path, content)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn exists(path: &String) -> bool {
    ::sifr_stdlib::fs::exists(path)
}
fn read_lines(path: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::read_lines(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn append_text(path: &String, content: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::append_text(path, content)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _open_file(path: &String, mode: &String) -> Result<String, IOError> {
    ::sifr_stdlib::fs::open_file(path, mode)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_read(handle: &String) -> Result<String, IOError> {
    ::sifr_stdlib::fs::file_read(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_write(handle: &String, data: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::file_write(handle, data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_readline(handle: &String) -> Result<Option<String>, IOError> {
    ::sifr_stdlib::fs::file_readline(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_readlines(handle: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::file_readlines(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_close(handle: &String) {
    ::sifr_stdlib::fs::file_close(handle);
}
fn _file_read_bytes(handle: &String) -> Result<Vec<u8>, IOError> {
    ::sifr_stdlib::fs::file_read_bytes(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_write_bytes(handle: &String, data: &Vec<u8>) -> Result<(), IOError> {
    ::sifr_stdlib::fs::file_write_bytes(handle, data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn open_file(path: &String, mode: &String) -> Result<__SifrIoNativeFileHandle, IOError> {
    let __sifr_try_res: Result<Result<__SifrIoNativeFileHandle, IOError>, IOError> = (|| {
        let handle_id: String = _open_file(path, mode)?;
        return Ok(Ok(__SifrIoNativeFileHandle::new(handle_id)));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(IOError::new(e.message.clone()));
        }
    }
}
fn file_read(handle: &__SifrIoNativeFileHandle) -> Result<String, IOError> {
    _file_read(&handle._id.clone())
}
fn file_write(handle: &__SifrIoNativeFileHandle, data: &String) -> Result<(), IOError> {
    _file_write(&handle._id.clone(), data)
}
fn file_readline(handle: &__SifrIoNativeFileHandle) -> Result<Option<String>, IOError> {
    _file_readline(&handle._id.clone())
}
fn file_readlines(handle: &__SifrIoNativeFileHandle) -> Result<Vec<String>, IOError> {
    _file_readlines(&handle._id.clone())
}
fn file_close(handle: &__SifrIoNativeFileHandle) {
    _file_close(&handle._id.clone());
}
fn file_read_bytes(handle: &__SifrIoNativeFileHandle) -> Result<Vec<u8>, IOError> {
    _file_read_bytes(&handle._id.clone())
}
fn file_write_bytes(
    handle: &__SifrIoNativeFileHandle,
    data: &Vec<u8>,
) -> Result<(), IOError> {
    _file_write_bytes(&handle._id.clone(), data)
}
fn getcwd() -> Result<String, IOError> {
    ::sifr_stdlib::fs::getcwd()
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn listdir(path: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::listdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn mkdir(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::mkdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn rmdir(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::rmdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn remove_file(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::remove_file(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn rename(src: &String, dst: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::rename(src, dst)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn chdir(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::chdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn stat_size(path: &String) -> Result<i64, IOError> {
    ::sifr_stdlib::fs::stat_size(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.to_i64_saturating())
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn disk_usage(path: &String) -> Vec<i64> {
    ::sifr_stdlib::fs::disk_usage(path)
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}
fn is_file(path: &String) -> bool {
    ::sifr_stdlib::fs::is_file(path)
}
fn is_dir(path: &String) -> bool {
    ::sifr_stdlib::fs::is_dir(path)
}
fn copy_file(src: &String, dst: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::copy_file(src, dst)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn walk_dir(path: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::walk_dir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn rmdir_all(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::rmdir_all(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn gettempdir() -> String {
    ::sifr_stdlib::fs::gettempdir()
}
fn makedirs(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::makedirs(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn touch(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::touch(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn resolve_path(path: &String) -> Result<String, IOError> {
    ::sifr_stdlib::fs::resolve_path(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn iterdir(path: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::iterdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn glob_pattern(dir: &String, pattern: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::glob_pattern(dir, pattern)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn rglob_pattern(dir: &String, pattern: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::rglob_pattern(dir, pattern)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn __const_ENCODING_UTF8() -> String {
    "utf-8".to_string().to_string()
}
fn __const_ENCODING_UTF8_SIG() -> String {
    "utf-8-sig".to_string().to_string()
}
fn __const_ENCODING_ASCII() -> String {
    "ascii".to_string().to_string()
}
fn __const_ENCODING_LATIN1() -> String {
    "latin-1".to_string().to_string()
}
fn __const_ENCODING_UTF16_LE() -> String {
    "utf-16-le".to_string().to_string()
}
fn __const_ENCODING_UTF16_BE() -> String {
    "utf-16-be".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1250() -> String {
    "windows-1250".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1251() -> String {
    "windows-1251".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1252() -> String {
    "windows-1252".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1253() -> String {
    "windows-1253".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1254() -> String {
    "windows-1254".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1255() -> String {
    "windows-1255".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1256() -> String {
    "windows-1256".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1257() -> String {
    "windows-1257".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1258() -> String {
    "windows-1258".to_string().to_string()
}
fn __const_DECODE_ERRORS_STRICT() -> String {
    "strict".to_string().to_string()
}
fn __const_DECODE_ERRORS_REPLACE() -> String {
    "replace".to_string().to_string()
}
fn __const_DECODE_ERRORS_IGNORE() -> String {
    "ignore".to_string().to_string()
}
fn __const_DECODE_ERRORS_BACKSLASH_REPLACE() -> String {
    "backslashreplace".to_string().to_string()
}
fn __const_ENCODE_ERRORS_STRICT() -> String {
    "strict".to_string().to_string()
}
fn __const_ENCODE_ERRORS_REPLACE() -> String {
    "replace".to_string().to_string()
}
fn __const_ENCODE_ERRORS_IGNORE() -> String {
    "ignore".to_string().to_string()
}
fn __const_ENCODE_ERRORS_BACKSLASH_REPLACE() -> String {
    "backslashreplace".to_string().to_string()
}
fn __const_ENCODE_ERRORS_XMLCHARREF_REPLACE() -> String {
    "xmlcharrefreplace".to_string().to_string()
}
fn __const_ENCODE_ERRORS_NAME_REPLACE() -> String {
    "namereplace".to_string().to_string()
}
#[derive(Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eencoding_x2eDecodeError {
    message: String,
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecodeError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecodeError {}
impl ::std::fmt::Debug for __SifrStdlib_sifr_x2eencoding_x2eDecodeError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct("DecodeError").field("message", &self.message).finish()
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eencoding_x2eDecodeError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl ::std::error::Error for __SifrStdlib_sifr_x2eencoding_x2eDecodeError {}
#[derive(Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eencoding_x2eEncodeError {
    message: String,
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncodeError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncodeError {}
impl ::std::fmt::Debug for __SifrStdlib_sifr_x2eencoding_x2eEncodeError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct("EncodeError").field("message", &self.message).finish()
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eencoding_x2eEncodeError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl ::std::error::Error for __SifrStdlib_sifr_x2eencoding_x2eEncodeError {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    label: String,
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    fn new(label: String) -> Self {
        let __sifr_field_init_0: String = {
            let mut __sifr_concat: String = String::with_capacity(label.len() + 0usize);
            __sifr_concat.push_str((label).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        Self { label: __sifr_field_init_0 }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    fn canonical_label(
        &self,
    ) -> Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError> {
        _encoding_canonical_label(&self.label)
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    fn is_supported(&self) -> bool {
        _encoding_is_supported(&self.label)
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Encoding(label={})", self.label)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
    name: String,
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
    fn new(name: String) -> Self {
        let __sifr_field_init_0: String = {
            let mut __sifr_concat: String = String::with_capacity(name.len() + 0usize);
            __sifr_concat.push_str((name).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        Self { name: __sifr_field_init_0 }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "DecodeErrorHandler(name={})", self.name)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    name: String,
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    fn new(name: String) -> Self {
        let __sifr_field_init_0: String = {
            let mut __sifr_concat: String = String::with_capacity(name.len() + 0usize);
            __sifr_concat.push_str((name).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        Self { name: __sifr_field_init_0 }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "EncodeErrorHandler(name={})", self.name)
    }
}
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome {
    text: String,
    recoveries: Vec<String>,
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome {
    fn new(text: String, recoveries: Vec<String>) -> Self {
        let __sifr_field_init_0: String = {
            let mut __sifr_concat: String = String::with_capacity(text.len() + 0usize);
            __sifr_concat.push_str((text).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        let __sifr_field_init_1: Vec<String> = recoveries;
        Self {
            text: __sifr_field_init_0,
            recoveries: __sifr_field_init_1,
        }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome {
    fn get_text(&self) -> String {
        {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str((self.text.clone()).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome {
    fn get_recoveries(&self) -> Vec<String> {
        self.recoveries.clone()
    }
}
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome {
    data: Vec<u8>,
    recoveries: Vec<String>,
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome {
    fn new(data: Vec<u8>, recoveries: Vec<String>) -> Self {
        let __sifr_field_init_0: Vec<u8> = data;
        let __sifr_field_init_1: Vec<String> = recoveries;
        Self {
            data: __sifr_field_init_0,
            recoveries: __sifr_field_init_1,
        }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome {
    fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome {
    fn get_recoveries(&self) -> Vec<String> {
        self.recoveries.clone()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eencoding_x2eDecoder {
    _encoding: __SifrStdlib_sifr_x2eencoding_x2eEncoding,
    _errors: __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler,
    _exhausted: bool,
    _pending: Vec<u8>,
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecoder {
    fn new(
        enc: __SifrStdlib_sifr_x2eencoding_x2eEncoding,
        errors: Option<__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler>,
    ) -> Self {
        let __sifr_field_init_0: __SifrStdlib_sifr_x2eencoding_x2eEncoding = enc;
        let __sifr_field_init_1: __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler = _decode_handler_or_strict(
            &errors,
        );
        let __sifr_field_init_2: bool = false;
        let __sifr_field_init_3: Vec<u8> = vec![];
        Self {
            _encoding: __sifr_field_init_0,
            _errors: __sifr_field_init_1,
            _exhausted: __sifr_field_init_2,
            _pending: __sifr_field_init_3,
        }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecoder {
    fn decode(
        &mut self,
        data: &Vec<u8>,
        r#final: bool,
    ) -> Result<
        __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome,
        __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
    > {
        if self._exhausted {
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(
                    "decoder is exhausted".to_string(),
                ),
            );
        }
        let __sifr_try_res: Result<
            Result<
                __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome,
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
            >,
            __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
        > = (|| {
            let outcome: __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome = _encoding_decode_incremental_outcome(
                data,
                &self._pending,
                &self._encoding.clone().label.clone(),
                &self._errors.clone().name.clone(),
                r#final,
            )?;
            let next_pending: Vec<u8> = _encoding_decode_incremental_pending(
                data,
                &self._pending,
                &self._encoding.clone().label.clone(),
                r#final,
            )?;
            self._pending = next_pending;
            if r#final {
                self._pending = vec![];
                self._exhausted = true;
            }
            return Ok(Ok(outcome));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(
                    __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
                );
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eencoding_x2eEncoder {
    _encoding: __SifrStdlib_sifr_x2eencoding_x2eEncoding,
    _errors: __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler,
    _exhausted: bool,
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncoder {
    fn new(
        enc: __SifrStdlib_sifr_x2eencoding_x2eEncoding,
        errors: Option<__SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler>,
    ) -> Self {
        let __sifr_field_init_0: __SifrStdlib_sifr_x2eencoding_x2eEncoding = enc;
        let __sifr_field_init_1: __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler = _encode_handler_or_strict(
            &errors,
        );
        let __sifr_field_init_2: bool = false;
        Self {
            _encoding: __sifr_field_init_0,
            _errors: __sifr_field_init_1,
            _exhausted: __sifr_field_init_2,
        }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncoder {
    fn encode(
        &mut self,
        text: &String,
        r#final: bool,
    ) -> Result<
        __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome,
        __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
    > {
        if self._exhausted {
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(
                    "encoder is exhausted".to_string(),
                ),
            );
        }
        let __sifr_try_res: Result<
            Result<
                __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome,
                __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
            >,
            __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
        > = (|| {
            let outcome: __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome = encode_outcome(
                text,
                &self._encoding,
                &Some((self._errors.clone()).clone()),
            )?;
            if r#final {
                self._exhausted = true;
            }
            return Ok(Ok(outcome));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(
                    __SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message.clone()),
                );
            }
        }
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eencoding_x2eEncoder {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(
            f, "Encoder(_encoding={}, _errors={}, _exhausted={})", self._encoding, self
            ._errors, self._exhausted
        )
    }
}
fn _encoding_is_supported(label: &String) -> bool {
    _encoding_is_supported_impl(label)
}
fn _encoding_canonical_label(
    label: &String,
) -> Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError> {
    let __sifr_try_res: Result<
        Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError>,
        ParseError,
    > = (|| {
        let value: String = _encoding_canonical_label_impl(label)?;
        return Ok(Ok(value));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
            );
        }
    }
}
fn _encoding_decode_text(
    data: &Vec<u8>,
    encoding: &String,
    errors: &String,
) -> Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError> {
    let __sifr_try_res: Result<
        Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError>,
        ParseError,
    > = (|| {
        let text: String = _encoding_decode_text_impl(data, encoding, errors)?;
        return Ok(Ok(text));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
            );
        }
    }
}
fn _encoding_decode_recoveries(
    data: &Vec<u8>,
    encoding: &String,
    errors: &String,
) -> Result<Vec<String>, __SifrStdlib_sifr_x2eencoding_x2eDecodeError> {
    let __sifr_try_res: Result<
        Result<Vec<String>, __SifrStdlib_sifr_x2eencoding_x2eDecodeError>,
        ParseError,
    > = (|| {
        let recoveries: Vec<String> = _encoding_decode_recoveries_impl(
            data,
            encoding,
            errors,
        )?;
        return Ok(Ok(recoveries));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
            );
        }
    }
}
fn _encoding_decode_outcome(
    data: &Vec<u8>,
    encoding: &String,
    errors: &String,
) -> Result<
    __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome,
    __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
> {
    let __sifr_try_res: Result<
        Result<
            __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome,
            __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
        >,
        ParseError,
    > = (|| {
        let text: String = _encoding_decode_text_impl(data, encoding, errors)?;
        let recoveries: Vec<String> = _encoding_decode_recoveries_impl(
            data,
            encoding,
            errors,
        )?;
        return Ok(
            Ok(__SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome::new(text, recoveries)),
        );
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
            );
        }
    }
}
fn _encoding_decode_incremental_outcome(
    data: &Vec<u8>,
    pending: &Vec<u8>,
    encoding: &String,
    errors: &String,
    r#final: bool,
) -> Result<
    __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome,
    __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
> {
    let __sifr_try_res: Result<
        Result<
            __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome,
            __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
        >,
        ParseError,
    > = (|| {
        let text: String = _encoding_decode_incremental_text_impl(
            data,
            pending,
            encoding,
            errors,
            r#final,
        )?;
        let recoveries: Vec<String> = _encoding_decode_incremental_recoveries_impl(
            data,
            pending,
            encoding,
            errors,
            r#final,
        )?;
        return Ok(
            Ok(__SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome::new(text, recoveries)),
        );
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
            );
        }
    }
}
fn _encoding_decode_incremental_pending(
    data: &Vec<u8>,
    pending: &Vec<u8>,
    encoding: &String,
    r#final: bool,
) -> Result<Vec<u8>, __SifrStdlib_sifr_x2eencoding_x2eDecodeError> {
    let __sifr_try_res: Result<
        Result<Vec<u8>, __SifrStdlib_sifr_x2eencoding_x2eDecodeError>,
        ParseError,
    > = (|| {
        let next_pending: Vec<u8> = _encoding_decode_incremental_pending_impl(
            data,
            pending,
            encoding,
            r#final,
        )?;
        return Ok(Ok(next_pending));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
            );
        }
    }
}
fn _encoding_encode_bytes(
    text: &String,
    encoding: &String,
    errors: &String,
) -> Result<Vec<u8>, __SifrStdlib_sifr_x2eencoding_x2eEncodeError> {
    let __sifr_try_res: Result<
        Result<Vec<u8>, __SifrStdlib_sifr_x2eencoding_x2eEncodeError>,
        ParseError,
    > = (|| {
        let data: Vec<u8> = _encoding_encode_bytes_impl(text, encoding, errors)?;
        return Ok(Ok(data));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message.clone()),
            );
        }
    }
}
fn _encoding_encode_recoveries(
    text: &String,
    encoding: &String,
    errors: &String,
) -> Result<Vec<String>, __SifrStdlib_sifr_x2eencoding_x2eEncodeError> {
    let __sifr_try_res: Result<
        Result<Vec<String>, __SifrStdlib_sifr_x2eencoding_x2eEncodeError>,
        ParseError,
    > = (|| {
        let recoveries: Vec<String> = _encoding_encode_recoveries_impl(
            text,
            encoding,
            errors,
        )?;
        return Ok(Ok(recoveries));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message.clone()),
            );
        }
    }
}
fn _encoding_encode_outcome(
    text: &String,
    encoding: &String,
    errors: &String,
) -> Result<
    __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome,
    __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
> {
    let __sifr_try_res: Result<
        Result<
            __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome,
            __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
        >,
        ParseError,
    > = (|| {
        let data: Vec<u8> = _encoding_encode_bytes_impl(text, encoding, errors)?;
        let recoveries: Vec<String> = _encoding_encode_recoveries_impl(
            text,
            encoding,
            errors,
        )?;
        return Ok(
            Ok(__SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome::new(data, recoveries)),
        );
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message.clone()),
            );
        }
    }
}
fn encoding(label: &String) -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new((label).clone())
}
fn utf8() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_UTF8())
}
fn utf8_sig() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_UTF8_SIG())
}
fn ascii() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_ASCII())
}
fn latin1() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_LATIN1())
}
fn utf16_le() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_UTF16_LE())
}
fn utf16_be() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_UTF16_BE())
}
fn windows1252() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_WINDOWS_1252())
}
fn strict_decode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
        __const_DECODE_ERRORS_STRICT(),
    )
}
fn replace_decode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
        __const_DECODE_ERRORS_REPLACE(),
    )
}
fn ignore_decode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
        __const_DECODE_ERRORS_IGNORE(),
    )
}
fn backslash_replace_decode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
        __const_DECODE_ERRORS_BACKSLASH_REPLACE(),
    )
}
fn strict_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
        __const_ENCODE_ERRORS_STRICT(),
    )
}
fn replace_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
        __const_ENCODE_ERRORS_REPLACE(),
    )
}
fn ignore_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
        __const_ENCODE_ERRORS_IGNORE(),
    )
}
fn backslash_replace_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
        __const_ENCODE_ERRORS_BACKSLASH_REPLACE(),
    )
}
fn xmlcharref_replace_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
        __const_ENCODE_ERRORS_XMLCHARREF_REPLACE(),
    )
}
fn name_replace_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
        __const_ENCODE_ERRORS_NAME_REPLACE(),
    )
}
fn _decode_handler_name(
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler>,
) -> String {
    if let Some(errors) = errors.as_ref() {
        return {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str((errors.name.clone()).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    }
    __const_DECODE_ERRORS_STRICT()
}
fn _encode_handler_name(
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler>,
) -> String {
    if let Some(errors) = errors.as_ref() {
        return {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str((errors.name.clone()).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    }
    __const_ENCODE_ERRORS_STRICT()
}
fn _decode_handler_or_strict(
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler>,
) -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
    if let Some(errors) = errors.as_ref() {
        return __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
            format!("{}{}", errors.name.clone(), ""),
        );
    }
    strict_decode_handler()
}
fn _encode_handler_or_strict(
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler>,
) -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    if let Some(errors) = errors.as_ref() {
        return __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
            format!("{}{}", errors.name.clone(), ""),
        );
    }
    strict_encode_handler()
}
fn decode_outcome(
    data: &Vec<u8>,
    enc: &__SifrStdlib_sifr_x2eencoding_x2eEncoding,
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler>,
) -> Result<
    __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome,
    __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
> {
    let handler_name: String = _decode_handler_name(errors);
    let __sifr_try_res: Result<
        Result<
            __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome,
            __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
        >,
        __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
    > = (|| {
        return Ok(_encoding_decode_outcome(data, &enc.label.clone(), &handler_name));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
            );
        }
    }
}
fn decode(
    data: &Vec<u8>,
    enc: &__SifrStdlib_sifr_x2eencoding_x2eEncoding,
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler>,
) -> Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError> {
    let __sifr_try_res: Result<
        Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError>,
        __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
    > = (|| {
        let outcome: __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome = decode_outcome(
            data,
            enc,
            errors,
        )?;
        return Ok(Ok(outcome.get_text()));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
            );
        }
    }
}
fn encode_outcome(
    text: &String,
    enc: &__SifrStdlib_sifr_x2eencoding_x2eEncoding,
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler>,
) -> Result<
    __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome,
    __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
> {
    let handler_name: String = _encode_handler_name(errors);
    let __sifr_try_res: Result<
        Result<
            __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome,
            __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
        >,
        __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
    > = (|| {
        return Ok(_encoding_encode_outcome(text, &enc.label.clone(), &handler_name));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message.clone()),
            );
        }
    }
}
fn encode(
    text: &String,
    enc: &__SifrStdlib_sifr_x2eencoding_x2eEncoding,
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler>,
) -> Result<Vec<u8>, __SifrStdlib_sifr_x2eencoding_x2eEncodeError> {
    let __sifr_try_res: Result<
        Result<Vec<u8>, __SifrStdlib_sifr_x2eencoding_x2eEncodeError>,
        __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
    > = (|| {
        let outcome: __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome = encode_outcome(
            text,
            enc,
            errors,
        )?;
        return Ok(Ok(outcome.get_data()));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message.clone()),
            );
        }
    }
}
fn run_command(cmd: &String) -> Result<String, IOError> {
    ::sifr_stdlib::sys::run_command(cmd)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn env_get(key: &String) -> Option<String> {
    ::sifr_stdlib::sys::env_get(key)
}
fn env_keys() -> Vec<String> {
    ::sifr_stdlib::sys::env_keys()
}
fn env_values() -> Vec<String> {
    ::sifr_stdlib::sys::env_values()
}
fn env_items() -> Vec<String> {
    ::sifr_stdlib::sys::env_items()
}
fn get_args() -> Vec<String> {
    ::sifr_stdlib::sys::get_args()
}
fn sys_exit(code: i64) {
    ::sifr_stdlib::sys::sys_exit(::sifr_runtime::interop::SifrIntBridge::from(code));
}
fn sys_version() -> String {
    ::sifr_stdlib::sys::sys_version()
}
fn sys_platform() -> String {
    ::sifr_stdlib::sys::sys_platform()
}
fn sys_maxsize() -> i64 {
    ::sifr_stdlib::sys::sys_maxsize().to_i64_saturating()
}
fn getpid() -> i64 {
    ::sifr_stdlib::sys::getpid().to_i64_saturating()
}
fn cpu_count() -> i64 {
    ::sifr_stdlib::sys::cpu_count().to_i64_saturating()
}
fn which(name: &String) -> Option<String> {
    ::sifr_stdlib::sys::which(name)
}
fn os_sep() -> String {
    ::sifr_stdlib::sys::os_sep()
}
fn os_linesep() -> String {
    ::sifr_stdlib::sys::os_linesep()
}
fn os_name() -> String {
    ::sifr_stdlib::sys::os_name()
}
fn random_int(min: i64, max: i64) -> i64 {
    ::sifr_stdlib::random::random_int(
            ::sifr_runtime::interop::SifrIntBridge::from(min),
            ::sifr_runtime::interop::SifrIntBridge::from(max),
        )
        .to_i64_saturating()
}
fn random_float() -> f64 {
    ::sifr_stdlib::random::random_float()
}
fn random_uniform(min: f64, max: f64) -> f64 {
    ::sifr_stdlib::random::random_uniform(min, max)
}
fn random_randrange(start: i64, stop: i64, step: i64) -> Result<i64, ValueError> {
    ::sifr_stdlib::random::random_randrange(
            ::sifr_runtime::interop::SifrIntBridge::from(start),
            ::sifr_runtime::interop::SifrIntBridge::from(stop),
            ::sifr_runtime::interop::SifrIntBridge::from(step),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.to_i64_saturating())
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn random_gauss(mu: f64, sigma: f64) -> f64 {
    ::sifr_stdlib::random::random_gauss(mu, sigma)
}
fn random_module_state_words() -> Vec<i64> {
    ::sifr_stdlib::random::random_module_state_words()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}
fn random_module_state_index() -> i64 {
    ::sifr_stdlib::random::random_module_state_index().to_i64_saturating()
}
fn random_module_state_gauss_next() -> Option<f64> {
    ::sifr_stdlib::random::random_module_state_gauss_next()
}
fn random_module_set_state(
    words: &Vec<i64>,
    index: i64,
    gauss_next: Option<f64>,
) -> Result<(), ValueError> {
    ::sifr_stdlib::random::random_module_set_state(
            &words
                .iter()
                .copied()
                .map(::sifr_runtime::interop::SifrIntBridge::from)
                .collect::<Vec<_>>(),
            ::sifr_runtime::interop::SifrIntBridge::from(index),
            gauss_next.map(|__sifr_bridge_item_0| __sifr_bridge_item_0),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_encode(s: &String) -> String {
    ::sifr_stdlib::base64::base64_encode(s)
}
fn base64_encode_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::base64::base64_encode_bytes(data)
}
fn base64_decode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_decode_bytes(data: &Vec<u8>) -> Result<Vec<u8>, ParseError> {
    ::sifr_stdlib::base64::base64_decode_bytes(data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_encode_opts(
    s: &String,
    altchars: &String,
    wrapcol: i64,
) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_encode_opts(
            s,
            altchars,
            ::sifr_runtime::interop::SifrIntBridge::from(wrapcol),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_decode_opts(
    s: &String,
    altchars: &String,
    validate: bool,
    ignorechars: &String,
) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_decode_opts(s, altchars, validate, ignorechars)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn urlsafe_b64encode(s: &String) -> String {
    ::sifr_stdlib::base64::urlsafe_b64encode(s)
}
fn urlsafe_b64encode_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::base64::urlsafe_b64encode_bytes(data)
}
fn urlsafe_b64decode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::urlsafe_b64decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn urlsafe_b64decode_bytes(data: &Vec<u8>) -> Result<Vec<u8>, ParseError> {
    ::sifr_stdlib::base64::urlsafe_b64decode_bytes(data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn b32encode(s: &String) -> String {
    ::sifr_stdlib::base64::b32encode(s)
}
fn b32decode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::b32decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn b32hexencode(s: &String) -> String {
    ::sifr_stdlib::base64::b32hexencode(s)
}
fn b32hexdecode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::b32hexdecode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn sha256_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha256_bytes(data)
}
fn md5_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::md5_bytes(data)
}
fn sha1_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha1_bytes(data)
}
fn sha224_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha224_bytes(data)
}
fn sha384_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha384_bytes(data)
}
fn sha512_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha512_bytes(data)
}
fn blake2b_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::blake2b_bytes(data)
}
fn blake2s_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::blake2s_bytes(data)
}
fn _random_suffix() -> String {
    let n: i64 = random_int(100000_i64, 999999_i64);
    format!("{}", n)
}
fn mktemp_path(prefix: &String) -> String {
    let suffix: String = _random_suffix();
    let mut root: String = gettempdir();
    let mut __sifr_chars_root: Vec<char> = root.chars().collect::<Vec<char>>();
    if ((__sifr_chars_root.len() as i64) == (0_i64)) {
        root = "/tmp".to_string();
        __sifr_chars_root = root.chars().collect::<Vec<char>>();
    } else {
        let last: Option<String> = __sifr_chars_root
            .get(((root.chars().count() as i64) - (1_i64)) as usize)
            .map(|c| c.to_string());
        if let Some(last) = last {
            if last == "/" {
                return {
                    let mut __sifr_concat: String = String::with_capacity(
                        (root.len() + prefix.len()) + suffix.len(),
                    );
                    __sifr_concat.push_str((root).as_str());
                    __sifr_concat.push_str((prefix).as_str());
                    __sifr_concat.push_str((suffix).as_str());
                    __sifr_concat
                };
            }
        }
    }
    {
        let mut __sifr_concat: String = String::with_capacity(
            ((root.len() + 1usize) + prefix.len()) + suffix.len(),
        );
        __sifr_concat.push_str((root).as_str());
        __sifr_concat.push('/');
        __sifr_concat.push_str((prefix).as_str());
        __sifr_concat.push_str((suffix).as_str());
        __sifr_concat
    }
}
fn _next_candidate(prefix: &String) -> String {
    mktemp_path(prefix)
}
fn _collision_message(kind: &String, attempts: i64) -> String {
    {
        let mut __sifr_concat: String = String::with_capacity(
            (((9usize + kind.len()) + 37usize) + 0usize) + 9usize,
        );
        __sifr_concat.push_str("tempfile.");
        __sifr_concat.push_str((kind).as_str());
        __sifr_concat.push_str(": failed to create unique path after ");
        __sifr_concat.push_str((format!("{}", attempts)).as_str());
        __sifr_concat.push_str(" attempts");
        __sifr_concat
    }
}
fn mkstemp(prefix: &String) -> Result<String, IOError> {
    let mut attempts: i64 = 0_i64;
    let max_attempts: i64 = 64_i64;
    while attempts < max_attempts {
        let path: String = _next_candidate(prefix);
        let path_for_check: String = {
            let mut __sifr_concat: String = String::with_capacity(path.len() + 0usize);
            __sifr_concat.push_str((path).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        if exists(&path) {
            attempts += 1_i64;
            continue;
        }
        let __sifr_try_res: Result<Result<String, IOError>, IOError> = (|| {
            let wrt: () = write_text(&path, &"".to_string())?;
            return Ok(Ok(path));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                if exists(&path_for_check) {
                    attempts += 1_i64;
                    continue;
                }
                return Err(IOError::new(e.message.clone()));
            }
        }
    }
    Err(IOError::new(_collision_message(&"mkstemp".to_string(), max_attempts)))
}
fn mkdtemp(prefix: &String) -> Result<String, IOError> {
    let mut attempts: i64 = 0_i64;
    let max_attempts: i64 = 64_i64;
    while attempts < max_attempts {
        let path: String = _next_candidate(prefix);
        let path_for_check: String = {
            let mut __sifr_concat: String = String::with_capacity(path.len() + 0usize);
            __sifr_concat.push_str((path).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        if exists(&path) {
            attempts += 1_i64;
            continue;
        }
        let __sifr_try_res: Result<Result<String, IOError>, IOError> = (|| {
            let md: () = mkdir(&path)?;
            return Ok(Ok(path));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                if exists(&path_for_check) {
                    attempts += 1_i64;
                    continue;
                }
                return Err(IOError::new(e.message.clone()));
            }
        }
    }
    Err(IOError::new(_collision_message(&"mkdtemp".to_string(), max_attempts)))
}
fn _gzip_compress_bytes_impl(data: &String) -> Vec<u8> {
    ::sifr_stdlib::gzip::gzip_compress_bytes(data)
}
fn _gzip_decompress_bytes_impl(data: &Vec<u8>) -> Result<String, IOError> {
    ::sifr_stdlib::gzip::gzip_decompress_bytes(data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn zip_create(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::zipfile::zip_create(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn zip_add_file(
    zip_path: &String,
    name: &String,
    content: &String,
) -> Result<(), IOError> {
    ::sifr_stdlib::zipfile::zip_add_file(zip_path, name, content)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn zip_add_file_bytes(
    zip_path: &String,
    name: &String,
    content: &Vec<u8>,
) -> Result<(), IOError> {
    ::sifr_stdlib::zipfile::zip_add_file_bytes(zip_path, name, content)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn zip_read_file(zip_path: &String, name: &String) -> Result<String, IOError> {
    ::sifr_stdlib::zipfile::zip_read_file(zip_path, name)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn zip_read_file_bytes(zip_path: &String, name: &String) -> Result<Vec<u8>, IOError> {
    ::sifr_stdlib::zipfile::zip_read_file_bytes(zip_path, name)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn zip_namelist(zip_path: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::zipfile::zip_namelist(zip_path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _zip_read_only_error() -> String {
    "zipfile operation requires write or append mode".to_string()
}
fn _zip_open_mode_error(mode: &String) -> String {
    {
        let mut __sifr_concat: String = String::with_capacity(48usize + mode.len());
        __sifr_concat.push_str("zipfile open supports read-only mode only, got: ");
        __sifr_concat.push_str((mode).as_str());
        __sifr_concat
    }
}
fn _closed_stream_error() -> String {
    "I/O operation on closed stream".to_string()
}
fn _zip_unimplemented_error(feature: &String) -> String {
    {
        let mut __sifr_concat: String = String::with_capacity(
            (8usize + feature.len()) + 49usize,
        );
        __sifr_concat.push_str("zipfile ");
        __sifr_concat.push_str((feature).as_str());
        __sifr_concat.push_str(" is not implemented in this compatibility surface");
        __sifr_concat
    }
}
fn __io_err<E: ::std::fmt::Display + 'static>(e: E) -> IOError {
    let msg = e.to_string();
    let kind = {
        let __sifr_io_kind = (&e as &dyn ::std::any::Any)
            .downcast_ref::<std::io::Error>()
            .map(::std::io::Error::kind);
        match __sifr_io_kind {
            Some(::std::io::ErrorKind::NotFound) => "FileNotFound".to_string(),
            Some(::std::io::ErrorKind::PermissionDenied) => {
                "PermissionDenied".to_string()
            }
            Some(::std::io::ErrorKind::AlreadyExists) => "FileExists".to_string(),
            Some(::std::io::ErrorKind::IsADirectory) => "IsADirectory".to_string(),
            Some(::std::io::ErrorKind::NotADirectory) => "NotADirectory".to_string(),
            Some(::std::io::ErrorKind::DirectoryNotEmpty) => {
                "DirectoryNotEmpty".to_string()
            }
            _ => "Other".to_string(),
        }
    };
    IOError { message: msg, kind }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Error {
    message: String,
}
impl Error {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for Error {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ParseError {
    message: String,
}
impl ParseError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for ParseError {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ValueError {
    message: String,
}
impl ValueError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for ValueError {}
impl From<IOError> for Error {
    fn from(err: IOError) -> Self {
        Self::new(err.message)
    }
}
impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Self::new(err.message)
    }
}
impl From<ValueError> for Error {
    fn from(err: ValueError) -> Self {
        Self::new(err.message)
    }
}
fn main() {
    let mut temp_file: String = "".to_string();
    let mut temp_dir: String = "".to_string();
    let mut zip_path: String = "".to_string();
    let mut tempfile_ok: bool = false;
    let mut zip_ok: bool = false;
    let mut cleanup_ok: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
        let temp_file_created: String = mkstemp(
            &"sifr_runtime_tempfiles_and_zip_".to_string(),
        )?;
        let temp_dir_created: String = mkdtemp(
            &"sifr_runtime_tempfiles_and_zip_".to_string(),
        )?;
        temp_file = {
            let mut __sifr_concat: String = String::with_capacity(
                temp_file_created.len() + 0usize,
            );
            __sifr_concat.push_str((temp_file_created).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        temp_dir = {
            let mut __sifr_concat: String = String::with_capacity(
                temp_dir_created.len() + 0usize,
            );
            __sifr_concat.push_str((temp_dir_created).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        tempfile_ok = ((exists(&temp_file)) && (exists(&temp_dir)));
        let _w: () = write_text(&temp_file, &"payload".to_string())?;
        zip_path = {
            let mut __sifr_concat: String = String::with_capacity(
                temp_file.len() + 4usize,
            );
            __sifr_concat.push_str((temp_file).as_str());
            __sifr_concat.push_str(".zip");
            __sifr_concat
        };
        let archive: __SifrStdlib_sifr_x2ezipfile_x2eZipFile = __SifrStdlib_sifr_x2ezipfile_x2eZipFile::new(
            format!("{}{}", zip_path, ""),
            "a".to_string(),
            0_i64,
        );
        let _create: () = archive.create()?;
        let _add: () = archive.write(&"entry.txt".to_string(), &"payload".to_string())?;
        let names: Vec<String> = archive.namelist()?;
        let content: String = archive.read(&"entry.txt".to_string())?;
        zip_ok = (((((names.len() as i64) == (1_i64)))
            && ((names[(0_i64) as usize].clone() == "entry.txt")))
            && (content == "payload"));
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message.clone());
    }
    let __sifr_try_res: Result<(), IOError> = (|| {
        if ((zip_path.chars().count() as i64) > (0_i64)) && exists(&zip_path) {
            let _rm_zip: () = remove_file(&zip_path)?;
        }
        if ((temp_file.chars().count() as i64) > (0_i64)) && exists(&temp_file) {
            let _rm_file: () = remove_file(&temp_file)?;
        }
        if ((temp_dir.chars().count() as i64) > (0_i64)) && exists(&temp_dir) {
            let _rm_dir: () = rmdir(&temp_dir)?;
        }
        cleanup_ok = (((((temp_file.chars().count() as i64) == (0_i64))
            || !(exists(&temp_file)))
            && (((temp_dir.chars().count() as i64) == (0_i64)) || !(exists(&temp_dir))))
            && (((zip_path.chars().count() as i64) == (0_i64)) || !(exists(&zip_path))));
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message.clone());
    }
    assert!(tempfile_ok);
    assert!(zip_ok);
    assert!(cleanup_ok);
    println!("runtime_tempfiles_and_zip_zip_lifecycle_demo: ok");
}
