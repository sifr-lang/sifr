// src/main.rs
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
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::IOError;
fn write_text(path: &str, content: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::write_text(path, content).map_err(sifr_generated_io_err)
}
fn listdir(path: &str) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::listdir(path).map_err(sifr_generated_io_err)
}
fn fnmatch(name: &str, pattern: &str) -> bool {
    sifr_generated_match(name, SifrInt::from_i64(0), pattern, SifrInt::from_i64(0))
}
fn sifr_generated_match(name: &str, mut ni: SifrInt, pattern: &str, mut pi: SifrInt) -> bool {
    while &pi < &SifrInt::from(pattern.chars().count()) {
        let pc: Option<String> = {
            let sifr_generated_string_source = &pattern;
            let sifr_generated_string_index = pi.clone();
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_string_source.chars().count());
            sifr_generated_string_source
                .chars()
                .nth(sifr_generated_string_index_normalized)
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
            let sifr_generated_shared_branch_condition = pc == "?";
            if &ni >= &SifrInt::from(name.chars().count()) {
                return false;
            }
            {
                if sifr_generated_shared_branch_condition {
                } else {
                    let nc: Option<String> = {
                        let sifr_generated_string_source = &name;
                        let sifr_generated_string_index = ni.clone();
                        let sifr_generated_string_index_normalized = sifr_generated_string_index
                            .normalize_index_or_len(sifr_generated_string_source.chars().count());
                        sifr_generated_string_source
                            .chars()
                            .nth(sifr_generated_string_index_normalized)
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
            }
        } else {
            return false;
        }
    }
    &ni == &SifrInt::from(name.chars().count())
}
fn glob(directory: &str, pattern: &str) -> Vec<String> {
    let sifr_generated_chars_pattern: Vec<char> = pattern.chars().collect::<Vec<char>>();
    let include_hidden: bool =
        &SifrInt::from(sifr_generated_chars_pattern.len()) > &SifrInt::from_i64(0) && {
            let sifr_generated_string_index = SifrInt::from_i64(0);
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_pattern.len());
            sifr_generated_chars_pattern.get(sifr_generated_string_index_normalized)
        }
        .map(::std::string::ToString::to_string)
        .is_some_and(|sifr_generated_checked_value_0| {
            sifr_generated_checked_value_0.clone() == "."
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
                sifr_generated_chars_entry.get(sifr_generated_string_index_normalized)
            }
            .map(::std::string::ToString::to_string)
            .is_some_and(|sifr_generated_checked_value_1| {
                sifr_generated_checked_value_1.clone() == "."
            }) {
                continue;
            }
            if fnmatch(&entry, pattern) {
                matches.push(entry.to_owned());
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
        let mut sifr_generated_sorted_v = matches.iter().cloned().collect::<Vec<_>>();
        sifr_generated_sorted_v.sort();
        sifr_generated_sorted_v
    }
}
fn run_command(cmd: &str) -> Result<String, IOError> {
    ::sifr_stdlib::sys::run_command(cmd).map_err(sifr_generated_io_err)
}
fn getpid() -> SifrInt {
    ::sifr_stdlib::sys::getpid().into_sifr_int()
}
fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
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
fn sifr_generated_io_err<E: ::std::fmt::Display + 'static>(e: E) -> IOError {
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
fn collect_glob_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let base: String = {
        let mut sifr_generated_concat: String = String::with_capacity(25usize);
        sifr_generated_concat.push_str("/tmp/sifr_glob_glob_demo_");
        sifr_generated_concat.push_str(getpid().to_string().as_str());
        sifr_generated_concat
    };
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let _mk: String = run_command(&format!("mkdir -p {base}"))?;
        write_text(&format!("{base}/a.txt"), &"a".to_string())?;
        write_text(&format!("{base}/b.txt"), &"b".to_string())?;
        write_text(&format!("{base}/.hidden.txt"), &"h".to_string())?;
        let txt: Vec<String> = glob(&base, &"*.txt".to_string());
        let txt_ok: bool = &SifrInt::from(txt.len()) == &SifrInt::from_i64(2)
            && {
                let sifr_generated_checked_read_collection = &txt;
                let sifr_generated_checked_read_index = SifrInt::from_i64(0);
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            }
            .is_some_and(|sifr_generated_checked_value_0| {
                sifr_generated_checked_value_0.clone() == "a.txt"
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
            .is_some_and(|sifr_generated_checked_value_1| {
                sifr_generated_checked_value_1.clone() == "b.txt"
            });
        actual.push(txt_ok);
        let hidden: Vec<String> = glob(&base, &".*.txt".to_string());
        let hidden_ok: bool = &SifrInt::from(hidden.len()) == &SifrInt::from_i64(1) && {
            let sifr_generated_checked_read_collection = &hidden;
            let sifr_generated_checked_read_index = SifrInt::from_i64(0);
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        }
        .is_some_and(|sifr_generated_checked_value_2| {
            sifr_generated_checked_value_2.clone() == ".hidden.txt"
        });
        actual.push(hidden_ok);
        let wildcard_q: Vec<String> = glob(&base, &"?.txt".to_string());
        let wildcard_q_ok: bool = &SifrInt::from(wildcard_q.len()) == &SifrInt::from_i64(2)
            && {
                let sifr_generated_checked_read_collection = &wildcard_q;
                let sifr_generated_checked_read_index = SifrInt::from_i64(0);
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            }
            .is_some_and(|sifr_generated_checked_value_3| {
                sifr_generated_checked_value_3.clone() == "a.txt"
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
            .is_some_and(|sifr_generated_checked_value_4| {
                sifr_generated_checked_value_4.clone() == "b.txt"
            });
        actual.push(wildcard_q_ok);
        let none: Vec<String> = glob(&base, &"*.csv".to_string());
        actual.push(SifrInt::from(none.len()) == SifrInt::from_i64(0));
        let missing: Vec<String> = glob(&format!("{base}_missing"), &"*.txt".to_string());
        actual.push(SifrInt::from(missing.len()) == SifrInt::from_i64(0));
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
        actual = vec![false, false, false, false, false];
    }
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let _clean: String = run_command(&format!("rm -rf {base}"))?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
    }
    actual
}
fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true];
    let actual: Vec<bool> = collect_glob_actual();
    assert_bool_vector_eq(&actual, &expected);
    println!("glob glob parity demo: pass");
}
