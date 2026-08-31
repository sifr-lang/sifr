// src/main.rs
use ::sifr_runtime::SifrInt;

// --- stdlib: sifr.string ---
fn __const_ascii_lowercase() -> String {
    "abcdefghijklmnopqrstuvwxyz".to_string().to_string()
}
fn __const_digits() -> String {
    "0123456789".to_string().to_string()
}
fn __const_whitespace() -> String {
    " \t\n\r\u{b}\u{c}".to_string().to_string()
}
fn capwords(s: &str) -> String {
    let normalized: String = s
        .replace('\t', " ")
        .replace('\n', " ")
        .replace('\r', " ")
        .replace('\u{b}', " ")
        .replace('\u{c}', " ");
    let words: Vec<String> = normalized
        .split(' ')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let mut result: String = "".to_string();
    let mut first: bool = true;
    for word in words.iter().cloned() {
        let __sifr_chars_word: Vec<char> = word.chars().collect::<Vec<char>>();
        if (&SifrInt::from(__sifr_chars_word.len()) > &SifrInt::from_i64(0)) {
            if !first {
                result.push(' ');
            }
            first = false;
            let cap: String = {
                let _s = word.clone();
                let mut _c = _s.chars();
                _c.next()
                    .map(|f| f.to_uppercase().to_string() + &_c.as_str().to_lowercase())
                    .unwrap_or_default()
            };
            result.push_str(cap.as_str());
        }
    }
    result
}

// --- stdlib: sifr.test ---
fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
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
// --- end stdlib ---

fn collect_capwords_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    actual.push(capwords(&"hello world".to_string()).as_str() == "Hello World".to_string().as_str());
    actual.push(capwords(&"hello\tworld".to_string()).as_str() == "Hello World".to_string().as_str());
    actual.push(capwords(&"hello\nworld".to_string()).as_str() == "Hello World".to_string().as_str());
    actual.push(capwords(&"one\u{b}two\u{c}three".to_string()).as_str() == "One Two Three".to_string().as_str());
    actual.push(capwords(&"  one   two  ".to_string()).as_str() == "One Two".to_string().as_str());
    actual
}

fn collect_constants_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    actual.push(__const_ascii_lowercase() == "abcdefghijklmnopqrstuvwxyz");
    actual.push(__const_digits() == "0123456789");
    actual.push(&SifrInt::from(__const_whitespace().chars().count()) == &SifrInt::from_i64(6));
    actual
}

fn append_all(target: &mut Vec<bool>, values: &[bool]) {
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
