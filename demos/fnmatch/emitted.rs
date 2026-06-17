// --- stdlib: sifr.fnmatch ---
fn fnmatch(name: &String, pattern: &String) -> bool {
    return _match(name, 0 as i64, pattern, 0 as i64);
}
fn _match(name: &String, mut ni: i64, pattern: &String, mut pi: i64) -> bool {
    while pi < (pattern.chars().count() as i64) {
        let pc: Option<String> = Some({
            let Some(__indexed_char) = pattern.chars().nth(pi as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(pc) = pc {
            if pc == "*".to_string() {
                pi = pi + (1 as i64);
                if pi == (pattern.len() as i64) {
                    return true;
                }
                let mut j: i64 = ni;
                while j <= (name.chars().count() as i64) {
                    if _match(name, j, pattern, pi) {
                        return true;
                    }
                    j = j + (1 as i64);
                }
                return false;
            } else {
                if pc == "?".to_string() {
                    if ni >= (name.len() as i64) {
                        return false;
                    }
                    ni = ni + (1 as i64);
                    pi = pi + (1 as i64);
                } else {
                    if ni >= (name.len() as i64) {
                        return false;
                    }
                    let nc: Option<String> = Some({
                        let Some(__indexed_char) = name.chars().nth(ni as usize) else {
                            unreachable!(
                                "compiler-verified string index should be in range"
                            );
                        };
                        __indexed_char.to_string()
                    });
                    if let Some(nc) = nc {
                        if nc != pc {
                            return false;
                        }
                    } else {
                        return false;
                    }
                    ni = ni + (1 as i64);
                    pi = pi + (1 as i64);
                }
            }
        } else {
            return false;
        }
    }
    return ni == (name.chars().count() as i64);
}
fn fnmatch_filter(names: &Vec<String>, pattern: &String) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    for name in names.iter().cloned() {
        if fnmatch(&name, pattern) {
            result.push(name);
        }
    }
    return result;
}
fn fnmatchcase(name: &String, pattern: &String) -> bool {
    return _match(name, 0 as i64, pattern, 0 as i64);
}

// --- stdlib: sifr.test ---
fn assert_eq<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    actual: &T,
    expected: &T,
) {
    assert!(* actual == * expected);
}
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0 as i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize]) == expected.get(i as usize).copied());
        i = i + (1 as i64);
    }
}

fn collect_match_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    actual.push(fnmatch(&"hello.txt".to_string(), &"*.txt".to_string()));
    let no_txt_match: bool = !(fnmatch(&"hello.py".to_string(), &"*.txt".to_string()));
    actual.push(no_txt_match);
    actual.push(fnmatch(&"abc".to_string(), &"a?c".to_string()));
    let case_sensitive_miss: bool = !(fnmatchcase(&"AbC".to_string(), &"abc".to_string()));
    actual.push(case_sensitive_miss);
    return actual;
}

fn collect_filter_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let names: Vec<String> = vec!["main.py".to_string(), "notes.txt".to_string(), "lib.py".to_string()];
    actual.push((format!("{:?}", fnmatch_filter(&names, &"*.py".to_string()))).as_str() == ("[\"main.py\", \"lib.py\"]".to_string()).as_str());
    actual.push((format!("{:?}", fnmatch_filter(&names, &"README*".to_string()))).as_str() == ("[]".to_string()).as_str());
    return actual;
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
