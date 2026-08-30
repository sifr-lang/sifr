// src/main.rs
mod __sifr_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct RegexError {
        pub message: String,
        pub detail: String,
    }
    impl RegexError {
        pub fn new(message: String) -> Self {
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
pub use __sifr_project_nominals::RegexError;
use ::sifr_runtime::SifrInt;
type __SifrStdlib___sifr_x2eregex_x2eCompiledPattern = ::sifr_runtime::interop::Handle<
    ::sifr_stdlib::regex::CompiledPattern,
>;
trait __SifrOpaque__SifrStdlib___sifr_x2eregex_x2eCompiledPatternMethods {
    fn search(&self, text: &String) -> Result<Option<String>, RegexError>;
    fn is_match(&self, text: &String) -> Result<bool, RegexError>;
    fn sub(&self, replacement: &String, text: &String) -> Result<String, RegexError>;
    fn findall(&self, text: &String) -> Result<Vec<String>, RegexError>;
    fn split(&self, text: &String) -> Result<Vec<String>, RegexError>;
    fn pattern(&self) -> Result<String, RegexError>;
    fn flags(&self) -> Result<SifrInt, RegexError>;
}
impl __SifrOpaque__SifrStdlib___sifr_x2eregex_x2eCompiledPatternMethods
for __SifrStdlib___sifr_x2eregex_x2eCompiledPattern {
    fn search(&self, text: &String) -> Result<Option<String>, RegexError> {
        ::sifr_stdlib::regex::compiled_pattern_search(self, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    fn is_match(&self, text: &String) -> Result<bool, RegexError> {
        ::sifr_stdlib::regex::compiled_pattern_is_match(self, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    fn sub(&self, replacement: &String, text: &String) -> Result<String, RegexError> {
        ::sifr_stdlib::regex::compiled_pattern_replace(self, replacement, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    fn findall(&self, text: &String) -> Result<Vec<String>, RegexError> {
        ::sifr_stdlib::regex::compiled_pattern_findall(self, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    fn split(&self, text: &String) -> Result<Vec<String>, RegexError> {
        ::sifr_stdlib::regex::compiled_pattern_split(self, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    fn pattern(&self) -> Result<String, RegexError> {
        ::sifr_stdlib::regex::compiled_pattern_source(self)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    fn flags(&self) -> Result<SifrInt, RegexError> {
        ::sifr_stdlib::regex::compiled_pattern_flags(self)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
}
fn compile_pattern(
    pattern: &String,
) -> Result<__SifrStdlib___sifr_x2eregex_x2eCompiledPattern, RegexError> {
    ::sifr_stdlib::regex::compile_pattern(pattern)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn compile_pattern_flags(
    pattern: &String,
    flags: SifrInt,
) -> Result<__SifrStdlib___sifr_x2eregex_x2eCompiledPattern, RegexError> {
    ::sifr_stdlib::regex::compile_pattern_flags(
            pattern,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_match(pattern: &String, text: &String) -> Result<bool, RegexError> {
    ::sifr_stdlib::regex::re_match(pattern, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_find(pattern: &String, text: &String) -> Result<Option<String>, RegexError> {
    ::sifr_stdlib::regex::re_find(pattern, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_replace(
    pattern: &String,
    replacement: &String,
    text: &String,
) -> Result<String, RegexError> {
    ::sifr_stdlib::regex::re_replace(pattern, replacement, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_findall(pattern: &String, text: &String) -> Result<Vec<String>, RegexError> {
    ::sifr_stdlib::regex::re_findall(pattern, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_split(pattern: &String, text: &String) -> Result<Vec<String>, RegexError> {
    ::sifr_stdlib::regex::re_split(pattern, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_find_start(pattern: &String, text: &String) -> Result<SifrInt, RegexError> {
    ::sifr_stdlib::regex::re_find_start(pattern, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_find_end(pattern: &String, text: &String) -> Result<SifrInt, RegexError> {
    ::sifr_stdlib::regex::re_find_end(pattern, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_match_flags(
    pattern: &String,
    text: &String,
    flags: SifrInt,
) -> Result<bool, RegexError> {
    ::sifr_stdlib::regex::re_match_flags(
            pattern,
            text,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_find_flags(
    pattern: &String,
    text: &String,
    flags: SifrInt,
) -> Result<Option<String>, RegexError> {
    ::sifr_stdlib::regex::re_find_flags(
            pattern,
            text,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_replace_flags(
    pattern: &String,
    replacement: &String,
    text: &String,
    flags: SifrInt,
) -> Result<String, RegexError> {
    ::sifr_stdlib::regex::re_replace_flags(
            pattern,
            replacement,
            text,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_findall_flags(
    pattern: &String,
    text: &String,
    flags: SifrInt,
) -> Result<Vec<String>, RegexError> {
    ::sifr_stdlib::regex::re_findall_flags(
            pattern,
            text,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_split_flags(
    pattern: &String,
    text: &String,
    flags: SifrInt,
) -> Result<Vec<String>, RegexError> {
    ::sifr_stdlib::regex::re_split_flags(
            pattern,
            text,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn __const_IGNORECASE() -> SifrInt {
    SifrInt::from_i64(2)
}
fn search(pattern: &String, text: &String) -> Result<Option<String>, RegexError> {
    re_find(pattern, text)
}
fn search_flags(
    pattern: &String,
    text: &String,
    flags: SifrInt,
) -> Result<Option<String>, RegexError> {
    re_find_flags(pattern, text, (flags).clone())
}
fn sub(
    pattern: &String,
    replacement: &String,
    text: &String,
) -> Result<String, RegexError> {
    re_replace(pattern, replacement, text)
}
fn findall(pattern: &String, text: &String) -> Result<Vec<String>, RegexError> {
    re_findall(pattern, text)
}
fn split(pattern: &String, text: &String) -> Result<Vec<String>, RegexError> {
    re_split(pattern, text)
}
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &SifrInt::from(actual.len()) {
        assert!(
            ({ let __sifr_condition_list = & actual; let __sifr_condition_index = i
            .clone(); let __sifr_condition_normalized = __sifr_condition_index
            .normalize_index_or_len(__sifr_condition_list.len()); __sifr_condition_list
            .get(__sifr_condition_normalized).copied() }) == ({ let __sifr_condition_list
            = & expected; let __sifr_condition_index = i.clone(); let
            __sifr_condition_normalized = __sifr_condition_index
            .normalize_index_or_len(__sifr_condition_list.len()); __sifr_condition_list
            .get(__sifr_condition_normalized).copied() })
        );
        i = &i + &SifrInt::from_i64(1);
    }
}
fn has_match(pattern: &String, text: &String) -> Result<bool, RegexError> {
    let __sifr_try_res: Result<Result<bool, RegexError>, RegexError> = (|| {
        let found: Option<String> = search(pattern, text)?;
        Ok(Ok((found != None)))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let error = __sifr_try_err.clone();
            return Err(RegexError::new(error.message.clone()));
        }
    }
}
fn collect_primary_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let mut match_ok: bool = false;
    let mut find_ok: bool = false;
    let mut replace_ok: bool = false;
    let mut findall_ok: bool = false;
    let mut split_ok: bool = false;
    let mut case_fold_ok: bool = false;
    let __sifr_try_res: Result<(), RegexError> = (|| {
        let m: bool = has_match(&"[0-9]+".to_string(), &"42 bottles".to_string())?;
        match_ok = m;
        let found_num: Option<String> = search(
            &"[0-9]+".to_string(),
            &"id=9000".to_string(),
        )?;
        find_ok = ((found_num)
            .map_or("None".to_string().to_string(), |__v| format!("{}", __v)) == "9000");
        let replaced: String = sub(
            &"\\s+".to_string(),
            &"-".to_string(),
            &"hello   world".to_string(),
        )?;
        replace_ok = (replaced == "hello-world");
        let all_alpha: Vec<String> = findall(
            &"[a-z]+".to_string(),
            &"ab 12 cd".to_string(),
        )?;
        findall_ok = (format!("{:?}", all_alpha) == "[\"ab\", \"cd\"]");
        let split_parts: Vec<String> = split(&":+".to_string(), &"a:b::c".to_string())?;
        split_ok = (format!("{:?}", split_parts) == "[\"a\", \"b\", \"c\"]");
        let case_fold: Option<String> = search_flags(
            &"hello".to_string(),
            &"HELLO".to_string(),
            __const_IGNORECASE(),
        )?;
        case_fold_ok = (case_fold != None);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message.clone());
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
