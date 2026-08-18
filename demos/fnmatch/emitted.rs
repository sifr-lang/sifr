// src/main.rs
// --- stdlib: sifr.fnmatch ---
fn fnmatch(name: &String, pattern: &String) -> bool {
    _match(name, 0_i64, pattern, 0_i64)
}
fn _match(name: &String, mut ni: i64, pattern: &String, mut pi: i64) -> bool {
    while (pi < (pattern.chars().count() as i64)) {
        let pc: Option<String> = Some({
            let Some(__indexed_char) = pattern
                .chars()
                .nth(pi as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(pc) = pc {
            if pc == "*" {
                pi += 1_i64;
                if (pi == (pattern.chars().count() as i64)) {
                    return true;
                }
                let mut j: i64 = ni;
                while (j <= (name.chars().count() as i64)) {
                    if _match(name, j, pattern, pi) {
                        return true;
                    }
                    j += 1_i64;
                }
                return false;
            } else {
                if pc == "?" {
                    if (ni >= (name.chars().count() as i64)) {
                        return false;
                    }
                    ni += 1_i64;
                    pi += 1_i64;
                } else {
                    if (ni >= (name.chars().count() as i64)) {
                        return false;
                    }
                    let nc: Option<String> = Some({
                        let Some(__indexed_char) = name
                            .chars()
                            .nth(ni as usize)
                            .map(|c| c.to_string()) else {
                            unreachable!(
                                "compiler-verified string index should be in range"
                            );
                        };
                        __indexed_char
                    });
                    if let Some(nc) = nc {
                        if nc != pc {
                            return false;
                        }
                    } else {
                        return false;
                    }
                    ni += 1_i64;
                    pi += 1_i64;
                }
            }
        } else {
            return false;
        }
    }
    (ni == (name.chars().count() as i64))
}
fn fnmatchcase(name: &String, pattern: &String) -> bool {
    _match(name, 0_i64, pattern, 0_i64)
}
fn filter(names: &Vec<String>, pattern: &String) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    for name in names.iter().cloned() {
        if fnmatch(&name, pattern) {
            result.push(name.clone());
        }
    }
    result
}

// --- stdlib: sifr.test ---
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0_i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize]) == expected.get(i as usize).copied());
        i += 1_i64;
    }
}
// --- end stdlib ---

fn collect_match_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    actual.push(fnmatch(&"hello.txt".to_string(), &"*.txt".to_string()));
    let no_txt_match: bool = !(fnmatch(&"hello.py".to_string(), &"*.txt".to_string()));
    actual.push(no_txt_match);
    actual.push(fnmatch(&"abc".to_string(), &"a?c".to_string()));
    let case_sensitive_miss: bool = !(fnmatchcase(&"AbC".to_string(), &"abc".to_string()));
    actual.push(case_sensitive_miss);
    actual
}

fn collect_filter_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let names: Vec<String> = vec!["main.py".to_string(), "notes.txt".to_string(), "lib.py".to_string()];
    actual.push((format!("{:?}", filter(&names, &"*.py".to_string()))).as_str() == ("[\"main.py\", \"lib.py\"]".to_string()).as_str());
    actual.push((format!("{:?}", filter(&names, &"README*".to_string()))).as_str() == ("[]".to_string()).as_str());
    actual
}

fn append_all(target: &mut Vec<bool>, values: &Vec<bool>) {
    for value in values.iter().copied() {
        target.push(value);
    }
}

fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true];
    let mut actual: Vec<bool> = vec![];
    append_all(&mut actual, &collect_match_actual());
    append_all(&mut actual, &collect_filter_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("fnmatch fnmatch parity demo: pass");
}
