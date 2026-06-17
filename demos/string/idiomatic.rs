const ASCII_LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const WHITESPACE: &str = " \t\n\r\u{b}\u{c}";

fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(actual, expected);
}

fn capwords(text: &str) -> String {
    text.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => {
                    let mut result = first.to_uppercase().collect::<String>();
                    result.push_str(&chars.as_str().to_lowercase());
                    result
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn collect_capwords_actual() -> Vec<bool> {
    vec![
        capwords("hello world") == "Hello World",
        capwords("hello\tworld") == "Hello World",
        capwords("hello\nworld") == "Hello World",
        capwords("one\u{b}two\u{c}three") == "One Two Three",
        capwords("  one   two  ") == "One Two",
    ]
}

fn collect_constants_actual() -> Vec<bool> {
    vec![
        ASCII_LOWERCASE == "abcdefghijklmnopqrstuvwxyz",
        DIGITS == "0123456789",
        WHITESPACE.chars().count() == 6,
    ]
}

fn append_all(target: &mut Vec<bool>, values: &[bool]) {
    target.extend_from_slice(values);
}

fn main() {
    let mut actual = Vec::new();
    append_all(&mut actual, &collect_capwords_actual());
    append_all(&mut actual, &collect_constants_actual());

    assert_bool_vector_eq(&actual, &[true, true, true, true, true, true, true, true]);
    println!("string string parity demo: pass");
}
