// src/main.rs
use ::sifr_runtime::SifrInt;
fn sifr_generated_const_61736369695f6c6f77657263617365() -> String {
    "abcdefghijklmnopqrstuvwxyz".to_string()
}
fn sifr_generated_const_646967697473() -> String {
    "0123456789".to_string()
}
fn sifr_generated_const_77686974657370616365() -> String {
    " \t\n\r\u{b}\u{c}".to_string()
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
        .map(::std::string::ToString::to_string)
        .collect::<Vec<String>>();
    let mut result: String = String::new();
    let mut first: bool = true;
    for word in words.iter().cloned() {
        let sifr_generated_chars_word: Vec<char> = word.chars().collect::<Vec<char>>();
        if &SifrInt::from(sifr_generated_chars_word.len()) > &SifrInt::from_i64(0) {
            if !first {
                result.push(' ');
            }
            first = false;
            let cap: String = {
                let sifr_generated_s = word.clone();
                let mut sifr_generated_c = sifr_generated_s.chars();
                sifr_generated_c
                    .next()
                    .map(|f| {
                        f.to_uppercase().to_string() + &sifr_generated_c.as_str().to_lowercase()
                    })
                    .unwrap_or_default()
            };
            result.push_str(cap.as_str());
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
fn collect_capwords_actual() -> Vec<bool> {
    vec![
        capwords(&"hello world".to_string()).as_str() == "Hello World".to_string().as_str(),
        capwords(&"hello\tworld".to_string()).as_str() == "Hello World".to_string().as_str(),
        capwords(&"hello\nworld".to_string()).as_str() == "Hello World".to_string().as_str(),
        capwords(&"one\u{b}two\u{c}three".to_string()).as_str()
            == "One Two Three".to_string().as_str(),
        capwords(&"  one   two  ".to_string()).as_str() == "One Two".to_string().as_str(),
    ]
}
fn collect_constants_actual() -> Vec<bool> {
    vec![
        sifr_generated_const_61736369695f6c6f77657263617365() == "abcdefghijklmnopqrstuvwxyz",
        sifr_generated_const_646967697473() == "0123456789",
        &SifrInt::from(sifr_generated_const_77686974657370616365().chars().count())
            == &SifrInt::from_i64(6),
    ]
}
fn append_all(target: &mut Vec<bool>, values: &[bool]) {
    for value in values.iter().copied() {
        target.push(value);
    }
}
fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true, true, true];
    let mut actual: Vec<bool> = Vec::new();
    append_all(&mut actual, &collect_capwords_actual());
    append_all(&mut actual, &collect_constants_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("string string parity demo: pass");
}
