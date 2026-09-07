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
    pub(crate) fn mkdir(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::mkdir(path).map_err(sifr_generated_io_err)
    }
    pub(crate) fn rmdir(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::rmdir(path).map_err(sifr_generated_io_err)
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
    pub(crate) fn sifr_generated_next_candidate(prefix: &str) -> String {
        mktemp_path(prefix)
    }
    pub(crate) fn sifr_generated_collision_message(kind: &str, attempts: SifrInt) -> String {
        {
            let mut sifr_generated_concat: String =
                String::with_capacity(9usize + kind.len() + 37usize + 9usize);
            sifr_generated_concat.push_str("tempfile.");
            sifr_generated_concat.push_str(kind);
            sifr_generated_concat.push_str(": failed to create unique path after ");
            sifr_generated_concat.push_str(attempts.to_string().as_str());
            sifr_generated_concat.push_str(" attempts");
            sifr_generated_concat
        }
    }
    pub(crate) fn mkstemp(prefix: &str) -> Result<String, IOError> {
        let mut attempts: SifrInt = SifrInt::from_i64(0);
        let max_attempts: SifrInt = SifrInt::from_i64(64);
        while &attempts < &max_attempts {
            let path: String = sifr_generated_next_candidate(prefix);
            let path_for_check: String = {
                let mut sifr_generated_concat: String = String::with_capacity(path.len());
                sifr_generated_concat.push_str(path.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            if exists(&path) {
                attempts = &attempts + &SifrInt::from_i64(1);
                continue;
            }
            let sifr_generated_try_res: Result<Result<String, IOError>, IOError> = (|| {
                write_text(&path, &String::new())?;
                Ok(Ok(path))
            })();
            match sifr_generated_try_res {
                Ok(sifr_generated_ret_val) => {
                    return sifr_generated_ret_val;
                }
                Err(sifr_generated_try_err) => {
                    let e = sifr_generated_try_err.clone();
                    if exists(&path_for_check) {
                        attempts = &attempts + &SifrInt::from_i64(1);
                        continue;
                    }
                    return Err(e);
                }
            }
        }
        Err(IOError::new(sifr_generated_collision_message(
            &"mkstemp".to_string(),
            max_attempts.clone(),
        )))
    }
    pub(crate) fn mkdtemp(prefix: &str) -> Result<String, IOError> {
        let mut attempts: SifrInt = SifrInt::from_i64(0);
        let max_attempts: SifrInt = SifrInt::from_i64(64);
        while &attempts < &max_attempts {
            let path: String = sifr_generated_next_candidate(prefix);
            let path_for_check: String = {
                let mut sifr_generated_concat: String = String::with_capacity(path.len());
                sifr_generated_concat.push_str(path.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            if exists(&path) {
                attempts = &attempts + &SifrInt::from_i64(1);
                continue;
            }
            let sifr_generated_try_res: Result<Result<String, IOError>, IOError> = (|| {
                mkdir(&path)?;
                Ok(Ok(path))
            })();
            match sifr_generated_try_res {
                Ok(sifr_generated_ret_val) => {
                    return sifr_generated_ret_val;
                }
                Err(sifr_generated_try_err) => {
                    let e = sifr_generated_try_err.clone();
                    if exists(&path_for_check) {
                        attempts = &attempts + &SifrInt::from_i64(1);
                        continue;
                    }
                    return Err(e);
                }
            }
        }
        Err(IOError::new(sifr_generated_collision_message(
            &"mkdtemp".to_string(),
            max_attempts.clone(),
        )))
    }
    pub(crate) fn zip_create(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::zipfile::zip_create(path).map_err(sifr_generated_io_err)
    }
    pub(crate) fn zip_add_file(zip_path: &str, name: &str, content: &str) -> Result<(), IOError> {
        ::sifr_stdlib::zipfile::zip_add_file(zip_path, name, content).map_err(sifr_generated_io_err)
    }
    pub(crate) fn zip_read_file(zip_path: &str, name: &str) -> Result<String, IOError> {
        ::sifr_stdlib::zipfile::zip_read_file(zip_path, name).map_err(sifr_generated_io_err)
    }
    pub(crate) fn zip_namelist(zip_path: &str) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::zipfile::zip_namelist(zip_path).map_err(sifr_generated_io_err)
    }
    pub(crate) fn sifr_generated_zip_read_only_error() -> String {
        "zipfile operation requires write or append mode".to_string()
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
        pub fn read(&self, name: &str) -> Result<String, IOError> {
            zip_read_file(&self.path, name)
        }
    }
    impl SifrGeneratedStdlibSifrX2ezipfileX2eZipFile {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn namelist(&self) -> Result<Vec<String>, IOError> {
            zip_namelist(&self.path)
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
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ezipfileX2eZipFile;
fn main() {
    let mut temp_file: String = String::new();
    let mut temp_dir: String = String::new();
    let mut zip_path: String = String::new();
    let mut tempfile_ok: bool = false;
    let mut zip_ok_value_27b0758fe4899c03: bool = false;
    let mut cleanup_ok: bool = false;
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let temp_file_created: String = mkstemp(&"sifr_runtime_tempfiles_and_zip_".to_string())?;
        let temp_dir_created: String = mkdtemp(&"sifr_runtime_tempfiles_and_zip_".to_string())?;
        temp_file = {
            let mut sifr_generated_concat: String = String::with_capacity(temp_file_created.len());
            sifr_generated_concat.push_str(temp_file_created.as_str());
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        };
        temp_dir = {
            let mut sifr_generated_concat: String = String::with_capacity(temp_dir_created.len());
            sifr_generated_concat.push_str(temp_dir_created.as_str());
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        };
        tempfile_ok = exists(&temp_file) && exists(&temp_dir);
        write_text(&temp_file, &"payload".to_string())?;
        zip_path = {
            let mut sifr_generated_concat: String = String::with_capacity(temp_file.len() + 4usize);
            sifr_generated_concat.push_str(temp_file.as_str());
            sifr_generated_concat.push_str(".zip");
            sifr_generated_concat
        };
        let archive: SifrGeneratedStdlibSifrX2ezipfileX2eZipFile =
            SifrGeneratedStdlibSifrX2ezipfileX2eZipFile::new(
                zip_path.to_string(),
                "a".to_string(),
                SifrInt::from_i64(0),
            );
        archive.create()?;
        archive.write(&"entry.txt".to_string(), &"payload".to_string())?;
        let names: Vec<String> = archive.namelist()?;
        let content: String = archive.read(&"entry.txt".to_string())?;
        zip_ok_value_27b0758fe4899c03 = &SifrInt::from(names.len()) == &SifrInt::from_i64(1)
            && {
                let sifr_generated_checked_read_collection = &names;
                let sifr_generated_checked_read_index = SifrInt::from_i64(0);
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            }
            .is_some_and(|_checked_value_0| {
                ({
                    let sifr_generated_cmp_list = &names;
                    let sifr_generated_cmp_i = SifrInt::from_i64(0);
                    let sifr_generated_cmp_norm =
                        sifr_generated_cmp_i.normalize_index_or_len(sifr_generated_cmp_list.len());
                    sifr_generated_cmp_list
                        .get(sifr_generated_cmp_norm)
                        .map(::std::string::String::as_str)
                } == Some("entry.txt"))
            })
            && content == "payload";
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
    }
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        if &SifrInt::from(zip_path.chars().count()) > &SifrInt::from_i64(0) && exists(&zip_path) {
            remove_file(&zip_path)?;
        }
        if &SifrInt::from(temp_file.chars().count()) > &SifrInt::from_i64(0) && exists(&temp_file) {
            remove_file(&temp_file)?;
        }
        if &SifrInt::from(temp_dir.chars().count()) > &SifrInt::from_i64(0) && exists(&temp_dir) {
            rmdir(&temp_dir)?;
        }
        cleanup_ok = (&SifrInt::from(temp_file.chars().count()) == &SifrInt::from_i64(0)
            || !exists(&temp_file))
            && (&SifrInt::from(temp_dir.chars().count()) == &SifrInt::from_i64(0)
                || !exists(&temp_dir))
            && (&SifrInt::from(zip_path.chars().count()) == &SifrInt::from_i64(0)
                || !exists(&zip_path));
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
    }
    assert!(tempfile_ok);
    assert!(zip_ok_value_27b0758fe4899c03);
    assert!(cleanup_ok);
    println!("runtime_tempfiles_and_zip_zip_lifecycle_demo: ok");
}
