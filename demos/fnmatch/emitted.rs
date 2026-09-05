// src/main.rs
pub mod sifr_generated_generated_support {
    pub(super) use ::sifr_runtime::SifrInt;
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
    pub(super) fn filter(names: &[String], pattern: &str) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        for name in names.iter().cloned() {
            if fnmatch(&name, pattern) {
                result.push(name);
            }
        }
        result
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
use crate::sifr_generated_generated_support::{assert_bool_vector_eq, filter, fnmatch};
fn collect_match_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![fnmatch("hello.txt", "*.txt")];
    let no_txt_match: bool = !fnmatch("hello.py", "*.txt");
    actual.push(no_txt_match);
    actual.push(fnmatch("abc", "a?c"));
    let case_sensitive_miss: bool = !fnmatch("AbC", "abc");
    actual.push(case_sensitive_miss);
    actual
}
fn collect_filter_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let names: Vec<String> = vec![
        "main.py".to_string(),
        "notes.txt".to_string(),
        "lib.py".to_string(),
    ];
    actual.push(format!("{:?}", filter(&names, "*.py")).as_str() == "[\"main.py\", \"lib.py\"]");
    actual.push(format!("{:?}", filter(&names, "README*")).as_str() == "[]");
    actual
}
fn append_all(target: &mut Vec<bool>, values: &[bool]) {
    for value in values.iter().copied() {
        target.push(value);
    }
}
fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true];
    let mut actual: Vec<bool> = Vec::new();
    append_all(&mut actual, &collect_match_actual());
    append_all(&mut actual, &collect_filter_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("fnmatch fnmatch parity demo: pass");
}
