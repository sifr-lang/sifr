// src/main.rs
mod sifr_generated_generated_support {
    use crate::IOError;
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) fn write_text(path: &str, content: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::write_text(path, content).map_err(sifr_generated_io_err)
    }
    pub(crate) fn exists(path: &str) -> bool {
        ::sifr_stdlib::fs::exists(path)
    }
    pub(crate) fn remove_file(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::remove_file(path).map_err(sifr_generated_io_err)
    }
    pub(crate) fn gettempdir() -> String {
        ::sifr_stdlib::fs::gettempdir()
    }
    pub(crate) fn random_int(min: SifrInt, max: SifrInt) -> SifrInt {
        ::sifr_stdlib::random::random_int(
            ::sifr_runtime::interop::SifrIntBridge::from(min),
            ::sifr_runtime::interop::SifrIntBridge::from(max),
        )
        .into_sifr_int()
    }
    pub(crate) fn sifr_generated_random_suffix() -> String {
        let n: SifrInt = random_int(SifrInt::from_i64(100_000), SifrInt::from_i64(999_999));
        n.to_string()
    }
    pub(crate) fn mktemp_path(prefix: &str) -> String {
        let suffix: String = sifr_generated_random_suffix();
        let mut root: String = gettempdir();
        let sifr_generated_chars_root: Vec<char> = root.chars().collect::<Vec<char>>();
        if &SifrInt::from(sifr_generated_chars_root.len()) == &SifrInt::from_i64(0) {
            root = "/tmp".to_string();
        } else {
            let last: Option<String> = {
                let sifr_generated_string_index =
                    &SifrInt::from(root.chars().count()) - &SifrInt::from_i64(1);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_root.len());
                sifr_generated_chars_root
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(last) = last
                && last == "/"
            {
                return {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(root.len() + prefix.len() + suffix.len());
                    sifr_generated_concat.push_str(root.as_str());
                    sifr_generated_concat.push_str(prefix);
                    sifr_generated_concat.push_str(suffix.as_str());
                    sifr_generated_concat
                };
            }
        }
        {
            let mut sifr_generated_concat: String =
                String::with_capacity(root.len() + 1usize + prefix.len() + suffix.len());
            sifr_generated_concat.push_str(root.as_str());
            sifr_generated_concat.push('/');
            sifr_generated_concat.push_str(prefix);
            sifr_generated_concat.push_str(suffix.as_str());
            sifr_generated_concat
        }
    }
    pub(crate) fn zip_create(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::zipfile::zip_create(path).map_err(sifr_generated_io_err)
    }
    pub(crate) fn zip_add_file(zip_path: &str, name: &str, content: &str) -> Result<(), IOError> {
        ::sifr_stdlib::zipfile::zip_add_file(zip_path, name, content).map_err(sifr_generated_io_err)
    }
    pub(crate) fn zip_add_file_bytes(
        zip_path: &str,
        name: &str,
        content: &[u8],
    ) -> Result<(), IOError> {
        ::sifr_stdlib::zipfile::zip_add_file_bytes(zip_path, name, content)
            .map_err(sifr_generated_io_err)
    }
    pub(crate) fn zip_read_file_bytes(zip_path: &str, name: &str) -> Result<Vec<u8>, IOError> {
        ::sifr_stdlib::zipfile::zip_read_file_bytes(zip_path, name).map_err(sifr_generated_io_err)
    }
    pub(crate) fn zip_namelist(zip_path: &str) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::zipfile::zip_namelist(zip_path).map_err(sifr_generated_io_err)
    }
    pub(crate) const fn sifr_generated_const_5a49505f53544f524544() -> SifrInt {
        SifrInt::from_i64(0)
    }
    pub(crate) fn sifr_generated_zip_read_only_error() -> String {
        "zipfile operation requires write or append mode".to_string()
    }
    pub(crate) fn sifr_generated_zip_open_mode_error(mode: &str) -> String {
        {
            let mut sifr_generated_concat: String = String::with_capacity(48usize + mode.len());
            sifr_generated_concat.push_str("zipfile open supports read-only mode only, got: ");
            sifr_generated_concat.push_str(mode);
            sifr_generated_concat
        }
    }
    pub(crate) fn sifr_generated_closed_stream_error() -> String {
        "I/O operation on closed stream".to_string()
    }
    pub(crate) fn sifr_generated_zip_unimplemented_error(feature: &str) -> String {
        {
            let mut sifr_generated_concat: String =
                String::with_capacity(8usize + feature.len() + 49usize);
            sifr_generated_concat.push_str("zipfile ");
            sifr_generated_concat.push_str(feature);
            sifr_generated_concat.push_str(" is not implemented in this compatibility surface");
            sifr_generated_concat
        }
    }
    pub(crate) fn is_zipfile(path: &str) -> bool {
        let sifr_generated_try_res: Result<bool, IOError> = (|| {
            let _names: Vec<String> = zip_namelist(path)?;
            Ok(true)
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err.clone();
            let _ = e.message.clone();
            false
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
mod sifr_generated_project_nominals {
    use crate::sifr_generated_generated_support::*;
    use ::sifr_runtime::SifrInt;
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2etempfileX2eNamedTemporaryFile {
        pub path: String,
        pub mode: String,
        pub delete: bool,
        pub closed: bool,
        pub cleaned: bool,
    }
    impl SifrGeneratedStdlibSifrX2etempfileX2eNamedTemporaryFile {
        #[must_use]
        pub fn new(mode: String, delete: bool, prefix: String) -> Self {
            let mut candidate: String = mktemp_path(&prefix);
            while exists(&candidate) {
                candidate = mktemp_path(&prefix);
            }
            let _created_result: Result<(), IOError> = write_text(&candidate, &String::new());
            let sifr_generated_field_value_0e74a76ec4f48c05_5f70617468: String = {
                let mut sifr_generated_concat: String = String::with_capacity(candidate.len());
                sifr_generated_concat.push_str(candidate.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            let sifr_generated_field_value_e0efc38c5ec2afd5_5f6d6f6465: String = {
                let mut sifr_generated_concat: String = String::with_capacity(mode.len());
                sifr_generated_concat.push_str(mode.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            let sifr_generated_field_value_516ea6609f22db39_5f64656c657465: bool = delete;
            let sifr_generated_field_value_8bc7f577e5ffacda_5f636c6f736564: bool = false;
            let sifr_generated_field_value_12032f9cf5c44b7a_5f636c65616e6564: bool = false;
            Self {
                path: sifr_generated_field_value_0e74a76ec4f48c05_5f70617468,
                mode: sifr_generated_field_value_e0efc38c5ec2afd5_5f6d6f6465,
                delete: sifr_generated_field_value_516ea6609f22db39_5f64656c657465,
                closed: sifr_generated_field_value_8bc7f577e5ffacda_5f636c6f736564,
                cleaned: sifr_generated_field_value_12032f9cf5c44b7a_5f636c65616e6564,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2etempfileX2eNamedTemporaryFile {
        #[must_use]
        pub fn name(&self) -> String {
            {
                let mut sifr_generated_concat: String = String::new();
                sifr_generated_concat.push_str(self.path.clone().as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2etempfileX2eNamedTemporaryFile {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn sifr_generated_cleanup_path(&mut self) -> Result<(), IOError> {
            if self.cleaned {
                return Ok(());
            }
            if exists(&self.path) {
                let sifr_generated_try_res: Result<(), IOError> = (|| {
                    remove_file(&self.path)?;
                    Ok(())
                })();
                if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                    let e = sifr_generated_try_err.clone();
                    return Err(e);
                }
            }
            self.cleaned = true;
            Ok(())
        }
    }
    impl SifrGeneratedStdlibSifrX2etempfileX2eNamedTemporaryFile {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn close(&mut self) -> Result<(), IOError> {
            self.closed = true;
            if self.delete {
                return self.sifr_generated_cleanup_path();
            }
            Ok(())
        }
    }
    impl SifrGeneratedStdlibSifrX2etempfileX2eNamedTemporaryFile {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn cleanup(&mut self) -> Result<(), IOError> {
            self.closed = true;
            self.sifr_generated_cleanup_path()
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2etempfileX2eNamedTemporaryFile {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f,
                "NamedTemporaryFile(_path={}, _mode={}, _delete={}, _closed={}, _cleaned={})",
                self.path, self.mode, self.delete, self.closed, self.cleaned
            )
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2ezipfileX2eZipReadHandle {
        pub data: Vec<u8>,
        pub cursor: SifrInt,
        pub closed: bool,
    }
    impl SifrGeneratedStdlibSifrX2ezipfileX2eZipReadHandle {
        #[must_use]
        pub const fn new(data: Vec<u8>) -> Self {
            let sifr_generated_field_value_90770dc80a1c57ce_5f64617461: Vec<u8> = data;
            let sifr_generated_field_value_d0bd94583b33fdec_5f637572736f72: SifrInt =
                SifrInt::from_i64(0);
            let sifr_generated_field_value_8bc7f577e5ffacda_5f636c6f736564: bool = false;
            Self {
                data: sifr_generated_field_value_90770dc80a1c57ce_5f64617461,
                cursor: sifr_generated_field_value_d0bd94583b33fdec_5f637572736f72,
                closed: sifr_generated_field_value_8bc7f577e5ffacda_5f636c6f736564,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2ezipfileX2eZipReadHandle {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn read_bytes(&mut self, size: &Option<SifrInt>) -> Result<Vec<u8>, IOError> {
            if self.closed {
                return Err(IOError::new(sifr_generated_closed_stream_error()));
            }
            let mut end: SifrInt = SifrInt::from(self.data.len());
            if let Some(size) = size.as_ref() {
                let requested_size: SifrInt = size.clone();
                if &requested_size < &SifrInt::from_i64(0) {
                    end = SifrInt::from(self.data.len());
                } else {
                    let requested_end: SifrInt = &self.cursor.clone() + &requested_size;
                    if &requested_end < &end {
                        end = requested_end.clone();
                    }
                }
            }
            let out: Vec<u8> = {
                let sifr_generated_slice_src = &self.data.clone();
                let sifr_generated_slice_len = sifr_generated_slice_src.len();
                let sifr_generated_slice_start = self
                    .cursor
                    .clone()
                    .clamp_slice_bound(sifr_generated_slice_len);
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
            Ok(out)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2ezipfileX2eZipFile {
        pub path: String,
        pub mode: String,
        pub compression: SifrInt,
    }
    impl SifrGeneratedStdlibSifrX2ezipfileX2eZipFile {
        #[must_use]
        pub fn new(path: String, mode: String, compression: SifrInt) -> Self {
            let sifr_generated_field_value_03c52d0debd70676_70617468: String = {
                let mut sifr_generated_concat: String = String::with_capacity(path.len());
                sifr_generated_concat.push_str(path.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            let sifr_generated_field_value_0d3deba2c41dadb2_6d6f6465: String = {
                let mut sifr_generated_concat: String = String::with_capacity(mode.len());
                sifr_generated_concat.push_str(mode.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            let sifr_generated_field_value_fb545b3ab0be00f5_636f6d7072657373696f6e: SifrInt =
                compression.clone();
            Self {
                path: sifr_generated_field_value_03c52d0debd70676_70617468,
                mode: sifr_generated_field_value_0d3deba2c41dadb2_6d6f6465,
                compression: sifr_generated_field_value_fb545b3ab0be00f5_636f6d7072657373696f6e,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2ezipfileX2eZipFile {
        #[must_use]
        pub fn sifr_generated_writable_mode(&self) -> bool {
            self.mode.clone() == "w"
                || self.mode.clone() == "a"
                || self.mode.clone() == "wb"
                || self.mode.clone() == "ab"
        }
    }
    impl SifrGeneratedStdlibSifrX2ezipfileX2eZipFile {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn create(&self) -> Result<(), IOError> {
            zip_create(&self.path)
        }
    }
    impl SifrGeneratedStdlibSifrX2ezipfileX2eZipFile {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn write(&self, name: &str, content: &str) -> Result<(), IOError> {
            if !self.sifr_generated_writable_mode() {
                return Err(IOError::new(sifr_generated_zip_read_only_error()));
            }
            zip_add_file(&self.path, name, content)
        }
    }
    impl SifrGeneratedStdlibSifrX2ezipfileX2eZipFile {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn write_bytes(&self, name: &str, content: &[u8]) -> Result<(), IOError> {
            if !self.sifr_generated_writable_mode() {
                return Err(IOError::new(sifr_generated_zip_read_only_error()));
            }
            zip_add_file_bytes(&self.path, name, content)
        }
    }
    impl SifrGeneratedStdlibSifrX2ezipfileX2eZipFile {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn read_bytes(&self, name: &str) -> Result<Vec<u8>, IOError> {
            zip_read_file_bytes(&self.path, name)
        }
    }
    impl SifrGeneratedStdlibSifrX2ezipfileX2eZipFile {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn open(
            &self,
            name: &str,
            mode: &str,
        ) -> Result<SifrGeneratedStdlibSifrX2ezipfileX2eZipReadHandle, IOError> {
            let _ = name.to_owned();
            if mode != "r" && mode != "rb" {
                return Err(IOError::new(sifr_generated_zip_open_mode_error(mode)));
            }
            Err(IOError::new(sifr_generated_zip_unimplemented_error(
                &"open".to_string(),
            )))
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2ezipfileX2eZipFile {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f,
                "ZipFile(path={}, mode={}, compression={})",
                self.path, self.mode, self.compression
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
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2etempfileX2eNamedTemporaryFile;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ezipfileX2eZipFile;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ezipfileX2eZipReadHandle;
fn main() {
    let zip_path: String = "/tmp/sifr_runtime_zipfile_io.zip".to_string();
    let mut demo_ok: bool = false;
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let mut temp_file: SifrGeneratedStdlibSifrX2etempfileX2eNamedTemporaryFile =
            SifrGeneratedStdlibSifrX2etempfileX2eNamedTemporaryFile::new(
                "wb".to_string(),
                false,
                "sifr_runtime_zipfile_io_".to_string(),
            );
        let tmp_path: String = temp_file.name();
        (&mut temp_file).close()?;
        (&mut temp_file).cleanup()?;
        let tempfile_ok: bool = !exists(&tmp_path);
        if exists(&zip_path) {
            remove_file(&zip_path)?;
        }
        let writer: SifrGeneratedStdlibSifrX2ezipfileX2eZipFile =
            SifrGeneratedStdlibSifrX2ezipfileX2eZipFile::new(
                zip_path.to_string(),
                "w".to_string(),
                sifr_generated_const_5a49505f53544f524544(),
            );
        writer.create()?;
        writer.write(&"note.txt".to_string(), &"runtime-zipfile_io".to_string())?;
        writer.write_bytes(&"bin/raw.bin".to_string(), &vec![0_u8, 1_u8, 2_u8])?;
        let reader: SifrGeneratedStdlibSifrX2ezipfileX2eZipFile =
            SifrGeneratedStdlibSifrX2ezipfileX2eZipFile::new(
                zip_path.to_string(),
                "r".to_string(),
                sifr_generated_const_5a49505f53544f524544(),
            );
        let payload: Vec<u8> = reader.read_bytes(&"bin/raw.bin".to_string())?;
        let mut handle: SifrGeneratedStdlibSifrX2ezipfileX2eZipReadHandle =
            SifrGeneratedStdlibSifrX2ezipfileX2eZipReadHandle::new(vec![97_u8, 98_u8, 99_u8]);
        let read_all_value_a00acea05f629b7d: Vec<u8> =
            (&mut handle).read_bytes(&Some((-SifrInt::from_i64(1)).clone()))?;
        let handle_negative_ok: bool = read_all_value_a00acea05f629b7d == vec![97_u8, 98_u8, 99_u8];
        let sifr_generated_open_handle_result: Result<
            SifrGeneratedStdlibSifrX2ezipfileX2eZipReadHandle,
            IOError,
        > = reader.open(&"bin/raw.bin".to_string(), &"rb".to_string());
        let mut open_rejected: bool = false;
        let sifr_generated_try_res: Result<(), IOError> = (|| {
            let _open_handle: SifrGeneratedStdlibSifrX2ezipfileX2eZipReadHandle =
                sifr_generated_open_handle_result?;
            Ok(())
        })();
        if let Err(sifr_generated_try_err) = sifr_generated_try_res {
            let e = sifr_generated_try_err.clone();
            let _ = e.message.clone();
            open_rejected = true;
        }
        let bad_mode_writer: SifrGeneratedStdlibSifrX2ezipfileX2eZipFile =
            SifrGeneratedStdlibSifrX2ezipfileX2eZipFile::new(
                zip_path.to_string(),
                "rw".to_string(),
                sifr_generated_const_5a49505f53544f524544(),
            );
        let sifr_generated_bad_mode_write_result: Result<(), IOError> =
            bad_mode_writer.write(&"bad.txt".to_string(), &"bad-mode".to_string());
        let mut bad_mode_rejected: bool = false;
        let sifr_generated_try_res: Result<(), IOError> = (|| {
            sifr_generated_bad_mode_write_result?;
            Ok(())
        })();
        if let Err(sifr_generated_try_err) = sifr_generated_try_res {
            let e = sifr_generated_try_err.clone();
            let _ = e.message.clone();
            bad_mode_rejected = true;
        }
        demo_ok = tempfile_ok
            && is_zipfile(&zip_path)
            && payload == vec![0_u8, 1_u8, 2_u8]
            && handle_negative_ok
            && open_rejected
            && bad_mode_rejected;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone();
    }
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        if exists(&zip_path) {
            remove_file(&zip_path)?;
        }
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone();
    }
    assert!(demo_ok);
    println!("runtime_zipfile_io_zipfile_lifecycle_demo: ok");
}
