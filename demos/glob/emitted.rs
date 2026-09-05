// src/main.rs
pub mod sifr_generated_generated_support {
    use crate::IOError;
    pub(super) use ::sifr_runtime::SifrInt;
    pub(super) fn write_text(path: &str, content: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::write_text(path, content).map_err(sifr_generated_io_err)
    }
    pub(super) fn listdir(path: &str) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::listdir(path).map_err(sifr_generated_io_err)
    }
    pub(super) fn fnmatch(name: &str, pattern: &str) -> bool {
        sifr_generated_match(name, SifrInt::from_i64(0), pattern, SifrInt::from_i64(0))
    }
    pub(super) fn sifr_generated_match(
        name: &str,
        mut ni: SifrInt,
        pattern: &str,
        mut pi: SifrInt,
    ) -> bool {
        while pi < pattern.chars().count() {
            let pc: Option<String> = {
                let sifr_generated_string_chars = pattern.chars().collect::<Vec<char>>();
                let sifr_generated_string_index = &pi;
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_string_chars.len());
                sifr_generated_string_chars
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(pc) = pc {
                if pc == "*" {
                    pi = ::std::ops::Add::add(&pi, &SifrInt::from_i64(1));
                    if pi == pattern.chars().count() {
                        return true;
                    }
                    let mut j: SifrInt = ni;
                    while j <= name.chars().count() {
                        if sifr_generated_match(name, j.clone(), pattern, pi.clone()) {
                            return true;
                        }
                        j = ::std::ops::Add::add(&j, &SifrInt::from_i64(1));
                    }
                    return false;
                }
                if ni >= name.chars().count() {
                    return false;
                }
                if pc != "?" {
                    let nc: Option<String> = {
                        let sifr_generated_string_chars = name.chars().collect::<Vec<char>>();
                        let sifr_generated_string_index = &ni;
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
                ni = ::std::ops::Add::add(&ni, &SifrInt::from_i64(1));
                pi = ::std::ops::Add::add(&pi, &SifrInt::from_i64(1));
            } else {
                return false;
            }
        }
        ni == name.chars().count()
    }
    pub(super) fn glob(directory: &str, pattern: &str) -> Vec<String> {
        let sifr_generated_chars_pattern: Vec<char> = pattern.chars().collect::<Vec<char>>();
        let include_hidden: bool = sifr_generated_chars_pattern.len() > SifrInt::from_i64(0) && {
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
                if sifr_generated_chars_entry.len() == SifrInt::from_i64(0) {
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
            let e = sifr_generated_try_err;
            let _ = e.message;
            return Vec::new();
        }
        {
            let mut sifr_generated_sorted_values = matches;
            sifr_generated_sorted_values.sort_by(
                |sifr_generated_sorted_left, sifr_generated_sorted_right| {
                    sifr_generated_sorted_left.cmp(sifr_generated_sorted_right)
                },
            );
            sifr_generated_sorted_values
        }
    }
    pub(super) fn run_command(cmd: &str) -> Result<String, IOError> {
        ::sifr_stdlib::sys::run_command(cmd).map_err(sifr_generated_io_err)
    }
    pub(super) fn getpid() -> SifrInt {
        ::sifr_stdlib::sys::getpid().into_sifr_int()
    }
    pub(super) fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
        assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < actual.len() {
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
            i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
        }
    }
    pub(super) fn sifr_generated_io_err<E: ::std::fmt::Display + 'static>(e: E) -> IOError {
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
    impl ::std::fmt::Display for IOError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for IOError {}
}
use crate::sifr_generated_generated_support::{
    assert_bool_vector_eq, getpid, glob, run_command, write_text,
};
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::IOError;
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn collect_glob_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let base: String = {
        let mut sifr_generated_concat: String =
            String::with_capacity(25usize.saturating_add(0usize));
        sifr_generated_concat.push_str("/tmp/sifr_glob_glob_demo_");
        sifr_generated_concat.push_str(getpid().to_string().as_str());
        sifr_generated_concat
    };
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let _mk: String = run_command(&format!("mkdir -p {base}"))?;
        write_text(&format!("{base}/a.txt"), "a")?;
        write_text(&format!("{base}/b.txt"), "b")?;
        write_text(&format!("{base}/.hidden.txt"), "h")?;
        let txt: Vec<String> = glob(&base, "*.txt");
        let txt_ok: bool = txt.len() == SifrInt::from_i64(2)
            && {
                let sifr_generated_checked_read_collection = &txt;
                let sifr_generated_checked_read_index = SifrInt::from_i64(0);
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            }
            .is_some_and(|_checked_value_0| {
                ({
                    let sifr_generated_cmp_list = &txt;
                    let sifr_generated_cmp_i = SifrInt::from_i64(0);
                    let sifr_generated_cmp_norm =
                        sifr_generated_cmp_i.normalize_index_or_len(sifr_generated_cmp_list.len());
                    sifr_generated_cmp_list
                        .get(sifr_generated_cmp_norm)
                        .map(::std::string::String::as_str)
                } == Some("a.txt"))
            })
            && {
                let sifr_generated_checked_read_collection = &txt;
                let sifr_generated_checked_read_index = SifrInt::from_i64(1);
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            }
            .is_some_and(|_checked_value_1| {
                ({
                    let sifr_generated_cmp_list = &txt;
                    let sifr_generated_cmp_i = SifrInt::from_i64(1);
                    let sifr_generated_cmp_norm =
                        sifr_generated_cmp_i.normalize_index_or_len(sifr_generated_cmp_list.len());
                    sifr_generated_cmp_list
                        .get(sifr_generated_cmp_norm)
                        .map(::std::string::String::as_str)
                } == Some("b.txt"))
            });
        actual.push(txt_ok);
        let hidden: Vec<String> = glob(&base, ".*.txt");
        let hidden_ok: bool = hidden.len() == SifrInt::from_i64(1) && {
            let sifr_generated_checked_read_collection = &hidden;
            let sifr_generated_checked_read_index = SifrInt::from_i64(0);
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        }
        .is_some_and(|_checked_value_2| {
            ({
                let sifr_generated_cmp_list = &hidden;
                let sifr_generated_cmp_i = SifrInt::from_i64(0);
                let sifr_generated_cmp_norm =
                    sifr_generated_cmp_i.normalize_index_or_len(sifr_generated_cmp_list.len());
                sifr_generated_cmp_list
                    .get(sifr_generated_cmp_norm)
                    .map(::std::string::String::as_str)
            } == Some(".hidden.txt"))
        });
        actual.push(hidden_ok);
        let wildcard_q: Vec<String> = glob(&base, "?.txt");
        let wildcard_q_ok: bool = wildcard_q.len() == SifrInt::from_i64(2)
            && {
                let sifr_generated_checked_read_collection = &wildcard_q;
                let sifr_generated_checked_read_index = SifrInt::from_i64(0);
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            }
            .is_some_and(|_checked_value_3| {
                ({
                    let sifr_generated_cmp_list = &wildcard_q;
                    let sifr_generated_cmp_i = SifrInt::from_i64(0);
                    let sifr_generated_cmp_norm =
                        sifr_generated_cmp_i.normalize_index_or_len(sifr_generated_cmp_list.len());
                    sifr_generated_cmp_list
                        .get(sifr_generated_cmp_norm)
                        .map(::std::string::String::as_str)
                } == Some("a.txt"))
            })
            && {
                let sifr_generated_checked_read_collection = &wildcard_q;
                let sifr_generated_checked_read_index = SifrInt::from_i64(1);
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            }
            .is_some_and(|_checked_value_4| {
                ({
                    let sifr_generated_cmp_list = &wildcard_q;
                    let sifr_generated_cmp_i = SifrInt::from_i64(1);
                    let sifr_generated_cmp_norm =
                        sifr_generated_cmp_i.normalize_index_or_len(sifr_generated_cmp_list.len());
                    sifr_generated_cmp_list
                        .get(sifr_generated_cmp_norm)
                        .map(::std::string::String::as_str)
                } == Some("b.txt"))
            });
        actual.push(wildcard_q_ok);
        let none: Vec<String> = glob(&base, "*.csv");
        actual.push(none.len() == SifrInt::from_i64(0));
        let missing: Vec<String> = glob(&format!("{base}_missing"), "*.txt");
        actual.push(missing.len() == SifrInt::from_i64(0));
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        let _ = e.message.clone();
        actual = vec![false, false, false, false, false];
    }
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let _clean: String = run_command(&format!("rm -rf {base}"))?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        let _ = e.message;
    }
    actual
}
fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true];
    let actual: Vec<bool> = collect_glob_actual();
    assert_bool_vector_eq(&actual, &expected);
    println!("glob glob parity demo: pass");
}
