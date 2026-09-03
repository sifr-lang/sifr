// src/main.rs
mod sifr_generated_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ParseError {
        pub message: String,
    }
    impl ::std::fmt::Display for ParseError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ParseError {}
}
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::ParseError;
fn base64_encode(s: &str) -> String {
    ::sifr_stdlib::base64::base64_encode(s)
}
fn base64_decode(s: &str) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_decode(s).map_err(|sifr_generated_bridge_error| ParseError {
        message: sifr_generated_bridge_error.to_string(),
    })
}
fn urlsafe_b64encode(s: &str) -> String {
    ::sifr_stdlib::base64::urlsafe_b64encode(s)
}
fn urlsafe_b64decode(s: &str) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::urlsafe_b64decode(s).map_err(|sifr_generated_bridge_error| ParseError {
        message: sifr_generated_bridge_error.to_string(),
    })
}
fn b64encode(s: &str) -> String {
    base64_encode(s)
}
fn b64decode(s: &str) -> Result<String, ParseError> {
    base64_decode(s)
}
fn b16encode(s: &str) -> Result<String, ParseError> {
    let sifr_generated_try_res: Result<Result<String, ParseError>, ParseError> = (|| {
        let data: Vec<u8> =
            ::sifr_runtime::encoding::encode_bytes(&s, &"utf-8".to_string(), &"strict".to_string())
                .map_err(|sifr_generated_message| ParseError {
                    message: sifr_generated_message,
                })?;
        Ok(Ok({
            let sifr_generated_bytes_receiver: &[u8] = &data;
            let mut sifr_generated_hex =
                String::with_capacity(sifr_generated_bytes_receiver.len().saturating_mul(2_usize));
            for sifr_generated_byte in sifr_generated_bytes_receiver {
                let _ = ::std::fmt::Write::write_fmt(
                    &mut sifr_generated_hex,
                    format_args!("{:02x}", *sifr_generated_byte),
                );
            }
            sifr_generated_hex
        }
        .to_uppercase()))
    })();
    sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
        let e = sifr_generated_try_err.clone();
        Err(e)
    })
}
fn b16decode(s: &str) -> Result<String, ParseError> {
    let sifr_generated_try_res: Result<Result<String, ParseError>, ParseError> = (|| {
        let data: Vec<u8> = {
            let s: String = s.to_string();
            let mut cleaned = String::new();
            for ch in s.chars() {
                if ch.is_ascii_whitespace() {
                    continue;
                }
                if !ch.is_ascii_hexdigit() {
                    return Err(ParseError {
                        message: format!("invalid hex character: {ch}"),
                    });
                }
                cleaned.push(ch);
            }
            if cleaned.len() % 2 != 0 {
                return Err(ParseError {
                    message: "fromhex() arg must contain an even number of hexadecimal digits"
                        .to_string(),
                });
            }
            let mut result = Vec::new();
            for pair in cleaned.as_bytes().chunks(2) {
                let pair_str = ::std::str::from_utf8(pair).map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
                result.push(u8::from_str_radix(pair_str, 16).map_err(|e| ParseError {
                    message: e.to_string(),
                })?);
            }
            Ok::<Vec<u8>, ParseError>(result)
        }?;
        Ok(::sifr_runtime::encoding::decode_text(
            &data,
            &"utf-8".to_string(),
            &"strict".to_string(),
        )
        .map_err(|sifr_generated_message| ParseError {
            message: sifr_generated_message,
        }))
    })();
    sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
        let e = sifr_generated_try_err.clone();
        Err(e)
    })
}
fn assert_vector_eq(actual: &[String], expected: &[String]) {
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
                    .cloned()
            },
            {
                let sifr_generated_condition_list = &expected;
                let sifr_generated_condition_index = i.clone();
                let sifr_generated_condition_normalized = sifr_generated_condition_index
                    .normalize_index_or_len(sifr_generated_condition_list.len());
                sifr_generated_condition_list
                    .get(sifr_generated_condition_normalized)
                    .cloned()
            }
        );
        i = &i + &SifrInt::from_i64(1);
    }
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
fn encode_b64_or_empty(payload: &str) -> String {
    b64encode(payload)
}
fn encode_urlsafe_b64_or_empty(payload: &str) -> String {
    urlsafe_b64encode(payload)
}
fn decode_b64_or_empty(payload: &str) -> String {
    let sifr_generated_try_res: Result<String, ParseError> = (|| {
        let decoded: String = b64decode(payload)?;
        Ok(decoded)
    })();
    sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
        let e = sifr_generated_try_err.clone();
        let _ = format!("unexpected: {}", e.message.clone());
        String::new()
    })
}
fn decode_urlsafe_b64_or_empty(payload: &str) -> String {
    let sifr_generated_try_res: Result<String, ParseError> = (|| {
        let decoded: String = urlsafe_b64decode(payload)?;
        Ok(decoded)
    })();
    sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
        let e = sifr_generated_try_err.clone();
        let _ = format!("unexpected: {}", e.message.clone());
        String::new()
    })
}
fn b16_encode_or_empty(payload: &str) -> String {
    let mut encoded: String = String::new();
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let out: String = b16encode(payload)?;
        encoded = out;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = format!("unexpected: {}", e.message.clone());
    }
    encoded
}
fn b16_decode_or_empty(payload: &str) -> String {
    let mut decoded: String = String::new();
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let out: String = b16decode(payload)?;
        decoded = out;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = format!("unexpected: {}", e.message.clone());
    }
    decoded
}
fn collect_positive_actual() -> Vec<String> {
    let mut actual: Vec<String> = vec![
        encode_b64_or_empty(&"foo".to_string()),
        decode_b64_or_empty(&"Zm9v".to_string()),
    ];
    let urlsafe_encoded: String = encode_urlsafe_b64_or_empty(&"hello".to_string());
    let urlsafe_encoded_for_decode: String = {
        let mut sifr_generated_concat: String = String::with_capacity(urlsafe_encoded.len());
        sifr_generated_concat.push_str(urlsafe_encoded.as_str());
        sifr_generated_concat.push_str("");
        sifr_generated_concat
    };
    actual.push(urlsafe_encoded);
    actual.push(decode_urlsafe_b64_or_empty(&urlsafe_encoded_for_decode));
    let b16_encoded: String = b16_encode_or_empty(&"Hi".to_string());
    let b16_encoded_for_decode: String = {
        let mut sifr_generated_concat: String = String::with_capacity(b16_encoded.len());
        sifr_generated_concat.push_str(b16_encoded.as_str());
        sifr_generated_concat.push_str("");
        sifr_generated_concat
    };
    actual.push(b16_encoded);
    actual.push(b16_decode_or_empty(&b16_encoded_for_decode));
    actual
}
fn collect_decode_actual_ok(inputs: &[String]) -> Vec<bool> {
    let mut actual_ok: Vec<bool> = Vec::new();
    for payload in inputs.iter().cloned() {
        let sifr_generated_try_res: Result<(), ParseError> = (|| {
            let decoded: String = b64decode(&payload)?;
            let _ = decoded.to_string();
            actual_ok.push(true);
            Ok(())
        })();
        if let Err(sifr_generated_try_err) = sifr_generated_try_res {
            let _e = sifr_generated_try_err.clone();
            actual_ok.push(false);
        }
    }
    actual_ok
}
fn main() {
    let expected: Vec<String> = vec![
        "Zm9v".to_string(),
        "foo".to_string(),
        "aGVsbG8=".to_string(),
        "hello".to_string(),
        "4869".to_string(),
        "Hi".to_string(),
    ];
    let actual: Vec<String> = collect_positive_actual();
    assert_vector_eq(&actual, &expected);
    let decode_inputs: Vec<String> = vec!["not base64!!!".to_string(), "Zm9v".to_string()];
    let expected_ok: Vec<bool> = vec![false, true];
    let actual_ok: Vec<bool> = collect_decode_actual_ok(&decode_inputs);
    assert_bool_vector_eq(&actual_ok, &expected_ok);
    println!("base64 base64 parity demo: pass");
}
