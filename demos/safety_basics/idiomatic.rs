use base64::{engine::general_purpose::STANDARD, Engine as _};

fn assert_vector_eq(actual: &[String], expected: &[String]) {
    assert_eq!(actual, expected);
}

fn decode_utf8(data: &[u8]) -> Result<String, std::string::FromUtf8Error> {
    String::from_utf8(data.to_vec())
}

fn b64encode(text: &str) -> String {
    STANDARD.encode(text)
}

fn main() {
    match decode_utf8(&[0xff]) {
        Ok(_) => {
            println!("false");
            assert_eq!(false.to_string(), "true");
        }
        Err(_) => {
            println!("true");
            assert_eq!(true.to_string(), "true");
        }
    }

    let inputs = ["", "f", "fo", "foo"];
    let expected = vec![
        "".to_string(),
        "Zg==".to_string(),
        "Zm8=".to_string(),
        "Zm9v".to_string(),
    ];
    let actual = inputs
        .iter()
        .map(|value| b64encode(value))
        .collect::<Vec<_>>();
    assert_vector_eq(&actual, &expected);
    println!("true");
}
