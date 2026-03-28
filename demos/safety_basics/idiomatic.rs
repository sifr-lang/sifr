// --- stdlib: sifr.test ---
fn assert_eq<T: Clone + std::fmt::Display + PartialOrd + 'static>(actual: &T, expected: &T) {
    assert!(*actual == *expected);
}
fn assert_vector_eq(actual: &Vec<String>, expected: &Vec<String>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0 as i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize].clone()) == expected.get(i as usize).cloned());
        i = i + (1 as i64);
    }
}

// --- stdlib: sifr.bytes ---
fn decode_utf8(data: &Vec<u8>) -> Result<String, ParseError> {
    return String::from_utf8(data.iter().copied().collect::<Vec<u8>>()).map_err(|e| ParseError {
        message: e.to_string(),
    });
}

// --- stdlib: sifr.base64 ---
fn b64encode(s: &String) -> String {
    return base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &s.as_bytes());
}

#[derive(Debug, Clone)]
struct ParseError {
    message: String,
}

impl ParseError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for ParseError {}

fn main() {
    let __sifr_try_res: Result<(), ParseError> = (|| {
        let bad: String = String::from_utf8(
            vec![(255 as i64) as u8]
                .iter()
                .copied()
                .collect::<Vec<u8>>(),
        )
        .map_err(|e| ParseError {
            message: e.to_string(),
        })?;
        println!("{}", false);
        assert!(format!("{}", false) == "true".to_string());
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", true);
        assert!(format!("{}", true) == "true".to_string());
    }
    let inputs: Vec<String> = vec![
        "".to_string(),
        "f".to_string(),
        "fo".to_string(),
        "foo".to_string(),
    ];
    let expected: Vec<String> = vec![
        "".to_string(),
        "Zg==".to_string(),
        "Zm8=".to_string(),
        "Zm9v".to_string(),
    ];
    let mut actual: Vec<String> = vec![];
    for s in inputs.iter().cloned() {
        actual.push(b64encode(&s));
    }
    assert_vector_eq(&actual, &expected);
    println!("{}", true);
}
