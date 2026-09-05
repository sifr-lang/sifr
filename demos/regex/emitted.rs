// src/main.rs
pub mod sifr_generated_generated_support {
    use crate::RegexError;
    pub(super) use ::sifr_runtime::SifrInt;
    pub(super) fn re_find(pattern: &str, text: &str) -> Result<Option<String>, RegexError> {
        ::sifr_stdlib::regex::re_find(pattern, text).map_err(|sifr_generated_bridge_error| {
            RegexError {
                message: sifr_generated_bridge_error.to_string(),
                detail: sifr_generated_bridge_error.to_string(),
            }
        })
    }
    pub(super) fn re_replace(
        pattern: &str,
        replacement: &str,
        text: &str,
    ) -> Result<String, RegexError> {
        ::sifr_stdlib::regex::re_replace(pattern, replacement, text).map_err(
            |sifr_generated_bridge_error| RegexError {
                message: sifr_generated_bridge_error.to_string(),
                detail: sifr_generated_bridge_error.to_string(),
            },
        )
    }
    pub(super) fn re_findall(pattern: &str, text: &str) -> Result<Vec<String>, RegexError> {
        ::sifr_stdlib::regex::re_findall(pattern, text).map_err(|sifr_generated_bridge_error| {
            RegexError {
                message: sifr_generated_bridge_error.to_string(),
                detail: sifr_generated_bridge_error.to_string(),
            }
        })
    }
    pub(super) fn re_split(pattern: &str, text: &str) -> Result<Vec<String>, RegexError> {
        ::sifr_stdlib::regex::re_split(pattern, text).map_err(|sifr_generated_bridge_error| {
            RegexError {
                message: sifr_generated_bridge_error.to_string(),
                detail: sifr_generated_bridge_error.to_string(),
            }
        })
    }
    pub(super) fn re_find_flags(
        pattern: &str,
        text: &str,
        flags: SifrInt,
    ) -> Result<Option<String>, RegexError> {
        ::sifr_stdlib::regex::re_find_flags(
            pattern,
            text,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map_err(|sifr_generated_bridge_error| RegexError {
            message: sifr_generated_bridge_error.to_string(),
            detail: sifr_generated_bridge_error.to_string(),
        })
    }
    pub(super) const fn sifr_generated_const_49474e4f524543415345() -> SifrInt {
        SifrInt::from_i64(2)
    }
    pub(super) fn search(pattern: &str, text: &str) -> Result<Option<String>, RegexError> {
        re_find(pattern, text)
    }
    pub(super) fn search_flags(
        pattern: &str,
        text: &str,
        flags: &SifrInt,
    ) -> Result<Option<String>, RegexError> {
        re_find_flags(pattern, text, (*flags).clone())
    }
    pub(super) fn sub(pattern: &str, replacement: &str, text: &str) -> Result<String, RegexError> {
        re_replace(pattern, replacement, text)
    }
    pub(super) fn findall(pattern: &str, text: &str) -> Result<Vec<String>, RegexError> {
        re_findall(pattern, text)
    }
    pub(super) fn split(pattern: &str, text: &str) -> Result<Vec<String>, RegexError> {
        re_split(pattern, text)
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
}
mod sifr_generated_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct RegexError {
        pub message: String,
        pub detail: String,
    }
    impl RegexError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self {
                message,
                detail: String::new(),
            }
        }
    }
    impl ::std::fmt::Display for RegexError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for RegexError {}
}
use crate::sifr_generated_generated_support::{
    assert_bool_vector_eq, findall, search, search_flags,
    sifr_generated_const_49474e4f524543415345, split, sub,
};
pub use sifr_generated_project_nominals::RegexError;
fn has_match(pattern: &str, text: &str) -> Result<bool, RegexError> {
    let sifr_generated_try_res: Result<Result<bool, RegexError>, RegexError> = (|| {
        let found: Option<String> = search(pattern, text)?;
        Ok(Ok(found.is_some()))
    })();
    sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
        let error = sifr_generated_try_err;
        Err(RegexError::new(error.message))
    })
}
fn collect_primary_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let mut match_ok: bool = false;
    let mut find_ok: bool = false;
    let mut replace_ok: bool = false;
    let mut findall_ok: bool = false;
    let mut split_ok: bool = false;
    let mut case_fold_ok: bool = false;
    let sifr_generated_try_res: Result<(), RegexError> = (|| {
        let m: bool = has_match("[0-9]+", "42 bottles")?;
        match_ok = m;
        let found_num: Option<String> = search("[0-9]+", "id=9000")?;
        find_ok = found_num.unwrap_or_else(|| "None".to_string()) == "9000";
        let replaced_value_e7f16644073d5f45: String = sub("\\s+", "-", "hello   world")?;
        replace_ok = replaced_value_e7f16644073d5f45 == "hello-world";
        let all_alpha: Vec<String> = findall("[a-z]+", "ab 12 cd")?;
        findall_ok = format!("{all_alpha:?}") == "[\"ab\", \"cd\"]";
        let split_parts: Vec<String> = split(":+", "a:b::c")?;
        split_ok = format!("{split_parts:?}") == "[\"a\", \"b\", \"c\"]";
        let case_fold: Option<String> = search_flags(
            "hello",
            "HELLO",
            &sifr_generated_const_49474e4f524543415345(),
        )?;
        case_fold_ok = case_fold.is_some();
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        let _ = e.message;
    }
    actual.push(match_ok);
    actual.push(find_ok);
    actual.push(replace_ok);
    actual.push(findall_ok);
    actual.push(split_ok);
    actual.push(case_fold_ok);
    actual
}
fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true];
    let actual: Vec<bool> = collect_primary_actual();
    assert_bool_vector_eq(&actual, &expected);
    println!("regex re parity demo: pass");
}
