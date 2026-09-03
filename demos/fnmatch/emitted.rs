// src/main.rs
use ::sifr_runtime::SifrInt;
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
            if sifr_generated_shared_branch_condition {
                ni = &ni + &SifrInt::from_i64(1);
                pi = &pi + &SifrInt::from_i64(1);
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
                ni = &ni + &SifrInt::from_i64(1);
                pi = &pi + &SifrInt::from_i64(1);
            }
        } else {
            return false;
        }
    }
    &ni == &SifrInt::from(name.chars().count())
}
fn filter(names: &[String], pattern: &str) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for name in names.iter().cloned() {
        if fnmatch(&name, pattern) {
            result.push(name.to_owned());
        }
    }
    result
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
fn collect_match_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![fnmatch(&"hello.txt".to_string(), &"*.txt".to_string())];
    let no_txt_match: bool = !fnmatch(&"hello.py".to_string(), &"*.txt".to_string());
    actual.push(no_txt_match);
    actual.push(fnmatch(&"abc".to_string(), &"a?c".to_string()));
    let case_sensitive_miss: bool = !fnmatch(&"AbC".to_string(), &"abc".to_string());
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
    actual.push(
        format!("{:?}", filter(&names, &"*.py".to_string())).as_str()
            == "[\"main.py\", \"lib.py\"]".to_string().as_str(),
    );
    actual.push(
        format!("{:?}", filter(&names, &"README*".to_string())).as_str()
            == "[]".to_string().as_str(),
    );
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
