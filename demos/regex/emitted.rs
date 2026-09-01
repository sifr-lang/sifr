// src/main.rs
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
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::RegexError;
fn re_find(pattern: &str, text: &str) -> Result<Option<String>, RegexError> {
    ::sifr_stdlib::regex::re_find(pattern, text).map_err(|sifr_generated_bridge_error| RegexError {
        message: sifr_generated_bridge_error.to_string(),
        detail: sifr_generated_bridge_error.to_string(),
    })
}
fn re_replace(pattern: &str, replacement: &str, text: &str) -> Result<String, RegexError> {
    ::sifr_stdlib::regex::re_replace(pattern, replacement, text).map_err(
        |sifr_generated_bridge_error| RegexError {
            message: sifr_generated_bridge_error.to_string(),
            detail: sifr_generated_bridge_error.to_string(),
        },
    )
}
fn re_findall(pattern: &str, text: &str) -> Result<Vec<String>, RegexError> {
    ::sifr_stdlib::regex::re_findall(pattern, text).map_err(|sifr_generated_bridge_error| {
        RegexError {
            message: sifr_generated_bridge_error.to_string(),
            detail: sifr_generated_bridge_error.to_string(),
        }
    })
}
fn re_split(pattern: &str, text: &str) -> Result<Vec<String>, RegexError> {
    ::sifr_stdlib::regex::re_split(pattern, text).map_err(|sifr_generated_bridge_error| {
        RegexError {
            message: sifr_generated_bridge_error.to_string(),
            detail: sifr_generated_bridge_error.to_string(),
        }
    })
}
fn re_find_flags(pattern: &str, text: &str, flags: SifrInt) -> Result<Option<String>, RegexError> {
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
const fn sifr_generated_const_49474e4f524543415345() -> SifrInt {
    SifrInt::from_i64(2)
}
fn search(pattern: &str, text: &str) -> Result<Option<String>, RegexError> {
    re_find(pattern, text)
}
fn search_flags(pattern: &str, text: &str, flags: SifrInt) -> Result<Option<String>, RegexError> {
    re_find_flags(pattern, text, flags.clone())
}
fn sub(pattern: &str, replacement: &str, text: &str) -> Result<String, RegexError> {
    re_replace(pattern, replacement, text)
}
fn findall(pattern: &str, text: &str) -> Result<Vec<String>, RegexError> {
    re_findall(pattern, text)
}
fn split(pattern: &str, text: &str) -> Result<Vec<String>, RegexError> {
    re_split(pattern, text)
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
fn has_match(pattern: &str, text: &str) -> Result<bool, RegexError> {
    let sifr_generated_try_res: Result<Result<bool, RegexError>, RegexError> = (|| {
        let found: Option<String> = search(pattern, text)?;
        Ok(Ok(found.is_some()))
    })();
    sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
        let error = sifr_generated_try_err.clone();
        Err(RegexError::new(error.message.clone()))
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
        let m: bool = has_match(&"[0-9]+".to_string(), &"42 bottles".to_string())?;
        match_ok = m;
        let found_num: Option<String> = search(&"[0-9]+".to_string(), &"id=9000".to_string())?;
        find_ok = found_num.map_or_else(
            || "None".to_string(),
            |sifr_generated_v| sifr_generated_v.to_string(),
        ) == "9000";
        let replaced_value_e7f16644073d5f45: String = sub(
            &"\\s+".to_string(),
            &"-".to_string(),
            &"hello   world".to_string(),
        )?;
        replace_ok = replaced_value_e7f16644073d5f45 == "hello-world";
        let all_alpha: Vec<String> = findall(&"[a-z]+".to_string(), &"ab 12 cd".to_string())?;
        findall_ok = format!("{all_alpha:?}") == "[\"ab\", \"cd\"]";
        let split_parts: Vec<String> = split(&":+".to_string(), &"a:b::c".to_string())?;
        split_ok = format!("{split_parts:?}") == "[\"a\", \"b\", \"c\"]";
        let case_fold: Option<String> = search_flags(
            &"hello".to_string(),
            &"HELLO".to_string(),
            sifr_generated_const_49474e4f524543415345(),
        )?;
        case_fold_ok = case_fold.is_some();
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
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
