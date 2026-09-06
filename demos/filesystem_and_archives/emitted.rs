// src/main.rs
mod sifr_generated_generated_support {
    use crate::IOError;
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) fn read_text(path: &str) -> Result<String, IOError> {
        ::sifr_stdlib::fs::read_text(path).map_err(sifr_generated_io_err)
    }
    pub(crate) fn write_text(path: &str, content: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::write_text(path, content).map_err(sifr_generated_io_err)
    }
    pub(crate) fn exists(path: &str) -> bool {
        ::sifr_stdlib::fs::exists(path)
    }
    pub(crate) fn listdir(path: &str) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::listdir(path).map_err(sifr_generated_io_err)
    }
    pub(crate) fn mkdir(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::mkdir(path).map_err(sifr_generated_io_err)
    }
    pub(crate) fn rename(src: &str, dst: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::rename(src, dst).map_err(sifr_generated_io_err)
    }
    pub(crate) fn copy_file(src: &str, dst: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::copy_file(src, dst).map_err(sifr_generated_io_err)
    }
    pub(crate) fn rmdir_all(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::rmdir_all(path).map_err(sifr_generated_io_err)
    }
    pub(crate) fn gettempdir() -> String {
        ::sifr_stdlib::fs::gettempdir()
    }
    pub(crate) fn fnmatch(name: &str, pattern: &str) -> bool {
        sifr_generated_match(name, SifrInt::from_i64(0), pattern, SifrInt::from_i64(0))
    }
    pub(crate) fn sifr_generated_match(
        name: &str,
        mut ni: SifrInt,
        pattern: &str,
        mut pi: SifrInt,
    ) -> bool {
        while &pi < &SifrInt::from(pattern.chars().count()) {
            let pc: Option<String> = {
                let sifr_generated_string_chars = pattern.chars().collect::<Vec<char>>();
                let sifr_generated_string_index = pi.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_string_chars.len());
                sifr_generated_string_chars
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(pc) = pc {
                if pc == "*" {
                    pi = &pi + &SifrInt::from_i64(1);
                    if &pi == &SifrInt::from(pattern.chars().count()) {
                        return true;
                    }
                    let mut j: SifrInt = ni.clone();
                    while &j <= &SifrInt::from(name.chars().count()) {
                        if sifr_generated_match(name, j.clone(), pattern, pi.clone()) {
                            return true;
                        }
                        j = &j + &SifrInt::from_i64(1);
                    }
                    return false;
                }
                if &ni >= &SifrInt::from(name.chars().count()) {
                    return false;
                }
                if pc != "?" {
                    let nc: Option<String> = {
                        let sifr_generated_string_chars = name.chars().collect::<Vec<char>>();
                        let sifr_generated_string_index = ni.clone();
                        let sifr_generated_string_index_normalized = sifr_generated_string_index
                            .normalize_index_or_len(sifr_generated_string_chars.len());
                        sifr_generated_string_chars
                            .get(sifr_generated_string_index_normalized)
                            .copied()
                    }
                    .map(|character| character.to_string());
                    if let Some(nc) = nc {
                        if nc != pc {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                ni = &ni + &SifrInt::from_i64(1);
                pi = &pi + &SifrInt::from_i64(1);
            } else {
                return false;
            }
        }
        &ni == &SifrInt::from(name.chars().count())
    }
    pub(crate) fn glob(directory: &str, pattern: &str) -> Vec<String> {
        let sifr_generated_chars_pattern: Vec<char> = pattern.chars().collect::<Vec<char>>();
        let include_hidden: bool =
            &SifrInt::from(sifr_generated_chars_pattern.len()) > &SifrInt::from_i64(0) && {
                let sifr_generated_string_index = SifrInt::from_i64(0);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_pattern.len());
                sifr_generated_chars_pattern
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string())
            .is_some_and(|_checked_value_0| {
                {
                    let sifr_generated_string_index = SifrInt::from_i64(0);
                    let sifr_generated_string_index_normalized = sifr_generated_string_index
                        .normalize_index_or_len(sifr_generated_chars_pattern.len());
                    sifr_generated_chars_pattern
                        .get(sifr_generated_string_index_normalized)
                        .copied()
                }
                .map(Some)
                    == Some(Some('.'))
            });
        let mut matches: Vec<String> = Vec::new();
        let sifr_generated_try_res: Result<(), IOError> = (|| {
            let entries: Vec<String> = listdir(directory)?;
            for entry in entries.iter().cloned() {
                let sifr_generated_chars_entry: Vec<char> = entry.chars().collect::<Vec<char>>();
                if &SifrInt::from(sifr_generated_chars_entry.len()) == &SifrInt::from_i64(0) {
                    continue;
                }
                if !include_hidden && {
                    let sifr_generated_string_index = SifrInt::from_i64(0);
                    let sifr_generated_string_index_normalized = sifr_generated_string_index
                        .normalize_index_or_len(sifr_generated_chars_entry.len());
                    sifr_generated_chars_entry
                        .get(sifr_generated_string_index_normalized)
                        .copied()
                }
                .map(|character| character.to_string())
                .is_some_and(|_checked_value_1| {
                    {
                        let sifr_generated_string_index = SifrInt::from_i64(0);
                        let sifr_generated_string_index_normalized = sifr_generated_string_index
                            .normalize_index_or_len(sifr_generated_chars_entry.len());
                        sifr_generated_chars_entry
                            .get(sifr_generated_string_index_normalized)
                            .copied()
                    }
                    .map(Some)
                        == Some(Some('.'))
                }) {
                    continue;
                }
                if fnmatch(&entry, pattern) {
                    matches.push(entry);
                }
            }
            Ok(())
        })();
        if let Err(sifr_generated_try_err) = sifr_generated_try_res {
            let e = sifr_generated_try_err.clone();
            let _ = e.message.clone().to_string();
            return Vec::new();
        }
        {
            let mut sifr_generated_sorted_values = matches.iter().cloned().collect::<Vec<_>>();
            sifr_generated_sorted_values.sort_by(
                |sifr_generated_sorted_left, sifr_generated_sorted_right| {
                    sifr_generated_sorted_left.cmp(&sifr_generated_sorted_right)
                },
            );
            sifr_generated_sorted_values
        }
    }
    pub(crate) fn sifr_generated_gzip_compress_bytes_impl(data: &str) -> Vec<u8> {
        ::sifr_stdlib::gzip::gzip_compress_bytes(data)
    }
    pub(crate) fn sifr_generated_gzip_decompress_bytes_impl(
        data: &[u8],
    ) -> Result<String, IOError> {
        ::sifr_stdlib::gzip::gzip_decompress_bytes(data).map_err(sifr_generated_io_err)
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
    pub(crate) fn compress(data: &str) -> Vec<u8> {
        sifr_generated_gzip_compress_bytes_impl(data)
    }
    pub(crate) fn decompress(data: &[u8]) -> Result<String, IOError> {
        sifr_generated_gzip_decompress_bytes_impl(data)
    }
    pub(crate) fn run_command(cmd: &str) -> Result<String, IOError> {
        ::sifr_stdlib::sys::run_command(cmd).map_err(sifr_generated_io_err)
    }
    pub(crate) fn getpid() -> SifrInt {
        ::sifr_stdlib::sys::getpid().into_sifr_int()
    }
    pub(crate) fn basename(path: &str) -> String {
        let sifr_generated_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
        let mut i: SifrInt =
            &SifrInt::from(sifr_generated_chars_path.len()) - &SifrInt::from_i64(1);
        while &i >= &SifrInt::from_i64(0) {
            let ch: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_path.len());
                sifr_generated_chars_path
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(ch) = ch
                && ch == "/"
            {
                return {
                    let sifr_generated_slice_src = &sifr_generated_chars_path;
                    let sifr_generated_slice_len = sifr_generated_slice_src.len();
                    let sifr_generated_slice_start =
                        (&i + &SifrInt::from_i64(1)).clamp_slice_bound(sifr_generated_slice_len);
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
                };
            }
            i = &i - &SifrInt::from_i64(1);
        }
        {
            let mut sifr_generated_concat: String = String::with_capacity(path.len());
            sifr_generated_concat.push_str(path);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        }
    }
    pub(crate) fn stem(path: &str) -> String {
        let base: String = basename(path);
        let sifr_generated_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
        let mut i: SifrInt =
            &SifrInt::from(sifr_generated_chars_base.len()) - &SifrInt::from_i64(1);
        while &i > &SifrInt::from_i64(0) {
            let ch: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_base.len());
                sifr_generated_chars_base
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(ch) = ch
                && ch == "."
            {
                return {
                    let sifr_generated_slice_src = &sifr_generated_chars_base;
                    let sifr_generated_slice_len = sifr_generated_slice_src.len();
                    let sifr_generated_slice_start = 0;
                    let sifr_generated_slice_stop = i.clamp_slice_bound(sifr_generated_slice_len);
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
                };
            }
            i = &i - &SifrInt::from_i64(1);
        }
        {
            let mut sifr_generated_concat: String = String::with_capacity(base.len());
            sifr_generated_concat.push_str(base.as_str());
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        }
    }
    pub(crate) fn copy(src: &str, dst: &str) -> Result<(), IOError> {
        copy_file(src, dst)
    }
    pub(crate) fn move_file(src: &str, dst: &str) -> Result<(), IOError> {
        rename(src, dst)
    }
    pub(crate) fn rmtree(path: &str) -> Result<(), IOError> {
        rmdir_all(path)
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
    pub struct SifrGeneratedStdlibSifrX2epathlibX2ePath {
        pub path: String,
    }
    impl SifrGeneratedStdlibSifrX2epathlibX2ePath {
        #[must_use]
        pub const fn new(path: String) -> Self {
            let sifr_generated_field_value_0e74a76ec4f48c05_5f70617468: String = path;
            Self {
                path: sifr_generated_field_value_0e74a76ec4f48c05_5f70617468,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2epathlibX2ePath {
        #[must_use]
        pub fn stem(&self) -> String {
            stem(&self.path)
        }
    }
    impl SifrGeneratedStdlibSifrX2epathlibX2ePath {
        #[must_use]
        pub fn exists(&self) -> bool {
            exists(&self.path)
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2epathlibX2ePath {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "Path(_path={})", self.path)
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
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2epathlibX2ePath;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ezipfileX2eZipFile;
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    let base: String = {
        let mut sifr_generated_concat: String = String::with_capacity(42usize);
        sifr_generated_concat.push_str("/tmp/sifr_filesystem_archive_surface_demo_");
        sifr_generated_concat.push_str(getpid().to_string().as_str());
        sifr_generated_concat
    };
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let _mk: String = run_command(&format!("mkdir -p {base}"))?;
        let source: String = {
            let mut sifr_generated_concat: String = String::with_capacity(base.len() + 9usize);
            sifr_generated_concat.push_str(base.as_str());
            sifr_generated_concat.push_str("/note.txt");
            sifr_generated_concat
        };
        write_text(&source, &"hello d1".to_string())?;
        let sifr_generated_try_res: Result<(), IOError> = (|| {
            let note_content: String = read_text(&source)?;
            println!("{}", {
                let mut sifr_generated_concat: String =
                    String::with_capacity(15usize + note_content.len());
                sifr_generated_concat.push_str("io.read_text = ");
                sifr_generated_concat.push_str(note_content.as_str());
                sifr_generated_concat
            });
            Ok(())
        })();
        if let Err(sifr_generated_try_err) = sifr_generated_try_res {
            let e = sifr_generated_try_err.clone();
            println!("{}", {
                let mut sifr_generated_concat: String = String::with_capacity(20usize);
                sifr_generated_concat.push_str("io.read_text error: ");
                sifr_generated_concat.push_str(e.message.clone().as_str());
                sifr_generated_concat
            });
        }
        let note_path: SifrGeneratedStdlibSifrX2epathlibX2ePath =
            SifrGeneratedStdlibSifrX2epathlibX2ePath::new(source.to_string());
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(15usize);
            sifr_generated_concat.push_str("pathlib.stem = ");
            sifr_generated_concat.push_str(note_path.stem().as_str());
            sifr_generated_concat
        });
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(16usize);
            sifr_generated_concat.push_str("glob(\"*.txt\") = ");
            sifr_generated_concat
                .push_str(format!("{:?}", glob(&base, &"*.txt".to_string())).as_str());
            sifr_generated_concat
        });
        let copied: String = {
            let mut sifr_generated_concat: String = String::with_capacity(base.len() + 11usize);
            sifr_generated_concat.push_str(base.as_str());
            sifr_generated_concat.push_str("/copied.txt");
            sifr_generated_concat
        };
        let moved: String = {
            let mut sifr_generated_concat: String = String::with_capacity(base.len() + 10usize);
            sifr_generated_concat.push_str(base.as_str());
            sifr_generated_concat.push_str("/moved.txt");
            sifr_generated_concat
        };
        copy(&source, &copied)?;
        move_file(&copied, &moved)?;
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(26usize);
            sifr_generated_concat.push_str("shutil.move_file exists = ");
            sifr_generated_concat.push_str(
                SifrGeneratedStdlibSifrX2epathlibX2ePath::new(moved)
                    .exists()
                    .to_string()
                    .as_str(),
            );
            sifr_generated_concat
        });
        let temp_file: String = mkstemp(&"sifr_filesystem_archive_surface_demo_".to_string())?;
        let temp_dir: String = mkdtemp(&"sifr_filesystem_archive_surface_demo_".to_string())?;
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(19usize + temp_file.len());
            sifr_generated_concat.push_str("tempfile.mkstemp = ");
            sifr_generated_concat.push_str(temp_file.as_str());
            sifr_generated_concat
        });
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(19usize + temp_dir.len());
            sifr_generated_concat.push_str("tempfile.mkdtemp = ");
            sifr_generated_concat.push_str(temp_dir.as_str());
            sifr_generated_concat
        });
        let compressed: Vec<u8> = compress(&"archive sample".to_string());
        let sifr_generated_try_res: Result<(), IOError> = (|| {
            let restored: String = decompress(&compressed)?;
            println!("{}", {
                let mut sifr_generated_concat: String =
                    String::with_capacity(17usize + restored.len());
                sifr_generated_concat.push_str("gzip roundtrip = ");
                sifr_generated_concat.push_str(restored.as_str());
                sifr_generated_concat
            });
            Ok(())
        })();
        if let Err(sifr_generated_try_err) = sifr_generated_try_res {
            let e = sifr_generated_try_err.clone();
            println!("{}", {
                let mut sifr_generated_concat: String = String::with_capacity(12usize);
                sifr_generated_concat.push_str("gzip error: ");
                sifr_generated_concat.push_str(e.message.clone().as_str());
                sifr_generated_concat
            });
        }
        let zip_path: String = {
            let mut sifr_generated_concat: String = String::with_capacity(base.len() + 9usize);
            sifr_generated_concat.push_str(base.as_str());
            sifr_generated_concat.push_str("/demo.zip");
            sifr_generated_concat
        };
        let archive: SifrGeneratedStdlibSifrX2ezipfileX2eZipFile =
            SifrGeneratedStdlibSifrX2ezipfileX2eZipFile::new(
                zip_path,
                "a".to_string(),
                SifrInt::from_i64(0),
            );
        let sifr_generated_try_res: Result<(), IOError> = (|| {
            archive.create()?;
            archive.write(&"inside.txt".to_string(), &"inside-zip".to_string())?;
            let inside: String = archive.read(&"inside.txt".to_string())?;
            println!("{}", {
                let mut sifr_generated_concat: String =
                    String::with_capacity(15usize + inside.len());
                sifr_generated_concat.push_str("zipfile.read = ");
                sifr_generated_concat.push_str(inside.as_str());
                sifr_generated_concat
            });
            println!("{}", {
                let mut sifr_generated_concat: String = String::with_capacity(19usize);
                sifr_generated_concat.push_str("zipfile.namelist = ");
                sifr_generated_concat.push_str(format!("{:?}", archive.namelist()).as_str());
                sifr_generated_concat
            });
            Ok(())
        })();
        if let Err(sifr_generated_try_err) = sifr_generated_try_res {
            let e = sifr_generated_try_err.clone();
            println!("{}", {
                let mut sifr_generated_concat: String = String::with_capacity(15usize);
                sifr_generated_concat.push_str("zipfile error: ");
                sifr_generated_concat.push_str(e.message.clone().as_str());
                sifr_generated_concat
            });
        }
        let _rm_temp_file: String = run_command(&format!("rm -f {temp_file}"))?;
        rmtree(&temp_dir)?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(39usize);
            sifr_generated_concat.push_str("filesystem_archive_surface demo error: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let _cleanup: String = run_command(&format!("rm -rf {base}"))?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(42usize);
            sifr_generated_concat.push_str("filesystem_archive_surface cleanup error: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
}
