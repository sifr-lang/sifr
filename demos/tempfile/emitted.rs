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
    pub(crate) fn gettempdir() -> String {
        ::sifr_stdlib::fs::gettempdir()
    }
    pub(crate) fn run_command(cmd: &str) -> Result<String, IOError> {
        ::sifr_stdlib::sys::run_command(cmd).map_err(sifr_generated_io_err)
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
    pub(crate) fn dirname(path: &str) -> String {
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
        String::new()
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
    pub(crate) fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
        assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &SifrInt::from(actual.len()) {
            assert_eq!(
                {
                    let sifr_generated_condition_list = &actual;
                    let sifr_generated_condition_index = i.clone();
                    let sifr_generated_condition_normalized = sifr_generated_condition_index
                        .normalize_index_or_len(sifr_generated_condition_list.len());
                    sifr_generated_condition_list
                        .get(sifr_generated_condition_normalized)
                        .copied()
                },
                {
                    let sifr_generated_condition_list = &expected;
                    let sifr_generated_condition_index = i.clone();
                    let sifr_generated_condition_normalized = sifr_generated_condition_index
                        .normalize_index_or_len(sifr_generated_condition_list.len());
                    sifr_generated_condition_list
                        .get(sifr_generated_condition_normalized)
                        .copied()
                }
            );
            i = &i + &SifrInt::from_i64(1);
        }
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
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn collect_tempfile_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let preview_path: String = mktemp_path(&"sifr_tempfile_preview_".to_string());
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let file_path: String = mkstemp(&"sifr_tempfile_tmp_".to_string())?;
        let dir_path: String = mkdtemp(&"sifr_tempfile_tmpd_".to_string())?;
        actual.push(exists(&file_path));
        actual.push(exists(&dir_path));
        let preview_name: String = basename(&preview_path);
        let sifr_generated_chars_preview_name: Vec<char> =
            preview_name.chars().collect::<Vec<char>>();
        let file_name: String = basename(&file_path);
        let sifr_generated_chars_file_name: Vec<char> = file_name.chars().collect::<Vec<char>>();
        let dir_name: String = basename(&dir_path);
        let sifr_generated_chars_dir_name: Vec<char> = dir_name.chars().collect::<Vec<char>>();
        let preview_has_prefix: bool = &SifrInt::from(sifr_generated_chars_preview_name.len())
            > &SifrInt::from("sifr_tempfile_preview_".to_string().chars().count())
            && {
                let sifr_generated_slice_src = &sifr_generated_chars_preview_name;
                let sifr_generated_slice_len = sifr_generated_slice_src.len();
                let sifr_generated_slice_start =
                    SifrInt::from_i64(0).clamp_slice_bound(sifr_generated_slice_len);
                let sifr_generated_slice_stop =
                    SifrInt::from("sifr_tempfile_preview_".to_string().chars().count())
                        .clamp_slice_bound(sifr_generated_slice_len);
                String::from_iter(
                    sifr_generated_slice_src
                        .iter()
                        .skip(sifr_generated_slice_start)
                        .take(sifr_generated_slice_stop.saturating_sub(sifr_generated_slice_start))
                        .copied(),
                )
            } == "sifr_tempfile_preview_";
        let file_has_prefix: bool = &SifrInt::from(sifr_generated_chars_file_name.len())
            > &SifrInt::from("sifr_tempfile_tmp_".to_string().chars().count())
            && {
                let sifr_generated_slice_src = &sifr_generated_chars_file_name;
                let sifr_generated_slice_len = sifr_generated_slice_src.len();
                let sifr_generated_slice_start =
                    SifrInt::from_i64(0).clamp_slice_bound(sifr_generated_slice_len);
                let sifr_generated_slice_stop =
                    SifrInt::from("sifr_tempfile_tmp_".to_string().chars().count())
                        .clamp_slice_bound(sifr_generated_slice_len);
                String::from_iter(
                    sifr_generated_slice_src
                        .iter()
                        .skip(sifr_generated_slice_start)
                        .take(sifr_generated_slice_stop.saturating_sub(sifr_generated_slice_start))
                        .copied(),
                )
            } == "sifr_tempfile_tmp_";
        let dir_has_prefix: bool = &SifrInt::from(sifr_generated_chars_dir_name.len())
            > &SifrInt::from("sifr_tempfile_tmpd_".to_string().chars().count())
            && {
                let sifr_generated_slice_src = &sifr_generated_chars_dir_name;
                let sifr_generated_slice_len = sifr_generated_slice_src.len();
                let sifr_generated_slice_start =
                    SifrInt::from_i64(0).clamp_slice_bound(sifr_generated_slice_len);
                let sifr_generated_slice_stop =
                    SifrInt::from("sifr_tempfile_tmpd_".to_string().chars().count())
                        .clamp_slice_bound(sifr_generated_slice_len);
                String::from_iter(
                    sifr_generated_slice_src
                        .iter()
                        .skip(sifr_generated_slice_start)
                        .take(sifr_generated_slice_stop.saturating_sub(sifr_generated_slice_start))
                        .copied(),
                )
            } == "sifr_tempfile_tmpd_";
        actual.push(preview_has_prefix && file_has_prefix && dir_has_prefix);
        let temp_root: String = dirname(&preview_path);
        let missing_parent_name: String = "__sifr_tempfile_missing_parent__".to_string();
        let missing_parent_path: String = {
            let mut sifr_generated_concat: String =
                String::with_capacity(temp_root.len() + 1usize + missing_parent_name.len());
            sifr_generated_concat.push_str(temp_root.as_str());
            sifr_generated_concat.push('/');
            sifr_generated_concat.push_str(missing_parent_name.as_str());
            sifr_generated_concat
        };
        let missing_prefix: String = {
            let mut sifr_generated_concat: String =
                String::with_capacity(missing_parent_name.len() + 5usize);
            sifr_generated_concat.push_str(missing_parent_name.as_str());
            sifr_generated_concat.push_str("/bad_");
            sifr_generated_concat
        };
        let _rm_missing: String = run_command(&format!("rm -rf {missing_parent_path}"))?;
        let mut missing_error: bool = false;
        let sifr_generated_try_res: Result<(), IOError> = (|| {
            let unexpected_file: String = mkstemp(&missing_prefix)?;
            let _rm_unexpected: String = run_command(&format!("rm -f {unexpected_file}"))?;
            missing_error = false;
            Ok(())
        })();
        if let Err(sifr_generated_try_err) = sifr_generated_try_res {
            let e = sifr_generated_try_err.clone();
            let _ = e.message.clone().to_string();
            missing_error = true;
        }
        actual.push(missing_error);
        let _c1: String = run_command(&format!("rm -f {file_path}"))?;
        let _c2: String = run_command(&format!("rm -rf {dir_path}"))?;
        let _c3: String = run_command(&format!("rm -rf {missing_parent_path}"))?;
        let cleaned: bool = !exists(&file_path) && !exists(&dir_path);
        actual.push(cleaned);
        let next_path: String = mkstemp(&"sifr_tempfile_tmp_".to_string())?;
        actual.push(next_path != file_path);
        let _c4: String = run_command(&format!("rm -f {next_path}"))?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
        actual = vec![false, false, false, false, false, false];
    }
    actual
}
fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true];
    let actual: Vec<bool> = collect_tempfile_actual();
    assert_bool_vector_eq(&actual, &expected);
    println!("tempfile tempfile parity demo: pass");
}
