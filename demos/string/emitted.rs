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

// --- stdlib: sifr.string ---
fn __const_ascii_lowercase() -> String {
    return "abcdefghijklmnopqrstuvwxyz".to_string().to_string();
}
fn __const_digits() -> String {
    return "0123456789".to_string().to_string();
}
fn __const_whitespace() -> String {
    return " \t\n\r\u{b}\u{c}".to_string().to_string();
}
fn capwords(s: &String) -> String {
    let normalized: String = s
        .replace(&"\t".to_string(), &" ".to_string())
        .replace(&"\n".to_string(), &" ".to_string())
        .replace(&"\r".to_string(), &" ".to_string())
        .replace(&"\u{b}".to_string(), &" ".to_string())
        .replace(&"\u{c}".to_string(), &" ".to_string());
    let words: Vec<String> = normalized
        .split(&" ".to_string())
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let mut result: String = "".to_string();
    let mut first: bool = true;
    for word in words.iter().cloned() {
        if (word.chars().count() as i64) > (0 as i64) {
            if !first {
                result = format!("{}{}", result, " ".to_string());
            }
            first = false;
            let cap: String = {
                let _s = word.clone();
                let mut _c = _s.chars();
                _c.next()
                    .map(|f| f.to_uppercase().to_string() + &_c.as_str().to_lowercase())
                    .unwrap_or_default()
            };
            result = format!("{}{}", result, cap);
        }
    }
    return result;
}

fn collect_capwords_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    actual.push((capwords(&"hello world".to_string())).as_str() == ("Hello World".to_string()).as_str());
    actual.push((capwords(&"hello\tworld".to_string())).as_str() == ("Hello World".to_string()).as_str());
    actual.push((capwords(&"hello\nworld".to_string())).as_str() == ("Hello World".to_string()).as_str());
    actual.push((capwords(&"one\u{b}two\u{c}three".to_string())).as_str() == ("One Two Three".to_string()).as_str());
    actual.push((capwords(&"  one   two  ".to_string())).as_str() == ("One Two".to_string()).as_str());
    return actual;
}

fn collect_constants_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    actual.push(__const_ascii_lowercase() == "abcdefghijklmnopqrstuvwxyz".to_string());
    actual.push(__const_digits() == "0123456789".to_string());
    actual.push((__const_whitespace().chars().count() as i64) == (6 as i64));
    return actual;
}

fn append_all(target: &mut Vec<bool>, values: &Vec<bool>) {
    for value in values.iter().copied() {
        target.push(value);
    }
}

fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true, true, true];
    let mut actual: Vec<bool> = vec![];
    append_all(&mut actual, &collect_capwords_actual());
    append_all(&mut actual, &collect_constants_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("string string parity demo: pass");
}
