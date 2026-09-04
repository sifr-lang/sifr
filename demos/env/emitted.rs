// src/main.rs
pub mod sifr_generated_generated_support {
    pub(super) use ::sifr_runtime::SifrInt;
    pub(super) fn env_get(key: &str) -> Option<String> {
        ::sifr_stdlib::sys::env_get(key)
    }
    pub(super) fn env_keys() -> Vec<String> {
        ::sifr_stdlib::sys::env_keys()
    }
    pub(super) fn env_values() -> Vec<String> {
        ::sifr_stdlib::sys::env_values()
    }
    pub(super) fn env_items() -> Vec<String> {
        ::sifr_stdlib::sys::env_items()
    }
    pub(super) fn getenv_opt(key: &str) -> Option<String> {
        env_get(key)
    }
    pub(super) fn getenv(key: &str, default_value: &str) -> String {
        let val: Option<String> = env_get(key);
        let Some(val) = val else {
            return {
                let mut sifr_generated_concat: String =
                    String::with_capacity(default_value.len().saturating_add(0usize));
                sifr_generated_concat.push_str(default_value.as_ref());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
        };
        val
    }
    pub(super) fn keys() -> Vec<String> {
        env_keys()
    }
    pub(super) fn values() -> Vec<String> {
        env_values()
    }
    pub(super) fn items() -> Vec<String> {
        env_items()
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
use crate::sifr_generated_generated_support::{
    assert_bool_vector_eq, getenv, getenv_opt, items, keys, values,
};
fn main() {
    let with_default: String = getenv("SIFR_ENV_SAMPLE_MISSING", "fallback");
    println!("{with_default}");
    assert_eq!(with_default, "fallback");
    let without_default: Option<String> = getenv_opt("SIFR_ENV_SAMPLE_MISSING");
    assert_eq!(without_default.is_none().to_string(), "true");
    let invalid_expected_lookup_found: Vec<bool> = vec![false, false];
    let invalid_actual_lookup_found: Vec<bool> =
        vec![getenv_opt("").is_some(), getenv_opt("A=B").is_some()];
    assert_bool_vector_eq(&invalid_actual_lookup_found, &invalid_expected_lookup_found);
    assert_eq!((keys().len() == values().len()).to_string(), "true");
    assert_eq!((keys().len() == items().len()).to_string(), "true");
    println!("env read-only access demo: pass");
}
