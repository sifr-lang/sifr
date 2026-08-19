// src/main.rs
// --- stdlib: _sifr.crypto ---
fn random_int(min: i64, max: i64) -> i64 {
    ::sifr_stdlib::random::random_int(
            ::sifr_runtime::interop::SifrIntBridge::from(min),
            ::sifr_runtime::interop::SifrIntBridge::from(max),
        )
        .to_i64_saturating()
}
fn random_float() -> f64 {
    ::sifr_stdlib::random::random_float()
}
fn random_uniform(min: f64, max: f64) -> f64 {
    ::sifr_stdlib::random::random_uniform(min, max)
}
fn random_randrange(start: i64, stop: i64, step: i64) -> Result<i64, ValueError> {
    ::sifr_stdlib::random::random_randrange(
            ::sifr_runtime::interop::SifrIntBridge::from(start),
            ::sifr_runtime::interop::SifrIntBridge::from(stop),
            ::sifr_runtime::interop::SifrIntBridge::from(step),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.to_i64_saturating())
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn random_gauss(mu: f64, sigma: f64) -> f64 {
    ::sifr_stdlib::random::random_gauss(mu, sigma)
}
fn random_module_state_words() -> Vec<i64> {
    ::sifr_stdlib::random::random_module_state_words()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}
fn random_module_state_index() -> i64 {
    ::sifr_stdlib::random::random_module_state_index().to_i64_saturating()
}
fn random_module_state_gauss_next() -> Option<f64> {
    ::sifr_stdlib::random::random_module_state_gauss_next()
}
fn random_module_set_state(
    words: &Vec<i64>,
    index: i64,
    gauss_next: Option<f64>,
) -> Result<(), ValueError> {
    ::sifr_stdlib::random::random_module_set_state(
            &words
                .iter()
                .copied()
                .map(::sifr_runtime::interop::SifrIntBridge::from)
                .collect::<Vec<_>>(),
            ::sifr_runtime::interop::SifrIntBridge::from(index),
            gauss_next.map(|__sifr_bridge_item_0| __sifr_bridge_item_0),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_encode(s: &String) -> String {
    ::sifr_stdlib::base64::base64_encode(s)
}
fn base64_encode_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::base64::base64_encode_bytes(data)
}
fn base64_decode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_decode_bytes(data: &Vec<u8>) -> Result<Vec<u8>, ParseError> {
    ::sifr_stdlib::base64::base64_decode_bytes(data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_encode_opts(
    s: &String,
    altchars: &String,
    wrapcol: i64,
) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_encode_opts(
            s,
            altchars,
            ::sifr_runtime::interop::SifrIntBridge::from(wrapcol),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_decode_opts(
    s: &String,
    altchars: &String,
    validate: bool,
    ignorechars: &String,
) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_decode_opts(s, altchars, validate, ignorechars)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn urlsafe_b64encode(s: &String) -> String {
    ::sifr_stdlib::base64::urlsafe_b64encode(s)
}
fn urlsafe_b64encode_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::base64::urlsafe_b64encode_bytes(data)
}
fn urlsafe_b64decode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::urlsafe_b64decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn urlsafe_b64decode_bytes(data: &Vec<u8>) -> Result<Vec<u8>, ParseError> {
    ::sifr_stdlib::base64::urlsafe_b64decode_bytes(data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn b32encode(s: &String) -> String {
    ::sifr_stdlib::base64::b32encode(s)
}
fn b32decode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::b32decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn b32hexencode(s: &String) -> String {
    ::sifr_stdlib::base64::b32hexencode(s)
}
fn b32hexdecode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::b32hexdecode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn sha256_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha256_bytes(data)
}
fn md5_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::md5_bytes(data)
}
fn sha1_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha1_bytes(data)
}
fn sha224_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha224_bytes(data)
}
fn sha384_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha384_bytes(data)
}
fn sha512_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha512_bytes(data)
}
fn blake2b_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::blake2b_bytes(data)
}
fn blake2s_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::blake2s_bytes(data)
}

// --- stdlib: sifr.base64 ---
fn b64encode(s: &String) -> String {
    base64_encode(s)
}
fn b64decode(s: &String) -> Result<String, ParseError> {
    base64_decode(s)
}
fn b16encode(s: &String) -> Result<String, ParseError> {
    let __sifr_try_res: Result<Result<String, ParseError>, ParseError> = (|| {
        let data: Vec<u8> = ::sifr_runtime::encoding::encode_bytes(
                &s,
                &"utf-8".to_string(),
                &"strict".to_string(),
            )
            .map_err(|__message| ParseError { message: __message })?;
        return Ok(
            Ok(
                ({
                    let __bytes_receiver = &data;
                    let mut __hex = String::with_capacity(
                        __bytes_receiver.len().saturating_mul(2),
                    );
                    for __byte in __bytes_receiver.iter() {
                        __hex.push_str(&format!("{:02x}", * __byte));
                    }
                    __hex
                })
                    .to_uppercase(),
            ),
        );
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(e);
        }
    }
}
fn b16decode(s: &String) -> Result<String, ParseError> {
    let __sifr_try_res: Result<Result<String, ParseError>, ParseError> = (|| {
        let data: Vec<u8> = ({
            let s: String = s.to_string();
            let mut cleaned = String::new();
            for ch in s.chars() {
                if ch.is_ascii_whitespace() {
                    continue;
                }
                if !ch.is_ascii_hexdigit() {
                    return Err(ParseError {
                        message: format!("invalid hex character: {}", ch),
                    });
                }
                cleaned.push(ch);
            }
            if (cleaned.len() % 2) != 0 {
                return Err(ParseError {
                    message: "fromhex() arg must contain an even number of hexadecimal digits"
                        .to_string()
                        .to_string(),
                });
            }
            let mut result = Vec::new();
            for pair in cleaned.as_bytes().chunks(2) {
                let pair_str = ::std::str::from_utf8(pair)
                    .map_err(|e| ParseError {
                        message: e.to_string(),
                    })?;
                result
                    .push(
                        u8::from_str_radix(pair_str, 16)
                            .map_err(|e| ParseError {
                                message: e.to_string(),
                            })?,
                    );
            }
            Ok::<Vec<u8>, ParseError>(result)
        })?;
        return Ok(
            ::sifr_runtime::encoding::decode_text(
                    &data,
                    &"utf-8".to_string(),
                    &"strict".to_string(),
                )
                .map_err(|__message| ParseError { message: __message }),
        );
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(e);
        }
    }
}

// --- stdlib: sifr.test ---
fn assert_vector_eq(actual: &Vec<String>, expected: &Vec<String>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0_i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize].clone()) == expected.get(i as usize).cloned());
        i += 1_i64;
    }
}
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0_i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize]) == expected.get(i as usize).copied());
        i += 1_i64;
    }
}
// --- end stdlib ---

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ParseError {
    message: String,
}

impl ParseError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl ::std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for ParseError {
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl ::std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for ValueError {
}

fn encode_b64_or_empty(payload: &String) -> String {
    b64encode(payload)
}

fn encode_urlsafe_b64_or_empty(payload: &String) -> String {
    urlsafe_b64encode(payload)
}

fn decode_b64_or_empty(payload: &String) -> String {
    let __sifr_try_res: Result<String, ParseError> = (|| {
    let decoded: String = b64decode(payload)?;
    return Ok(decoded);
    unreachable!("sifr try/except return capture fell through");
})();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        },
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            let _ = format!("{}", format!("{}{}", "unexpected: ", e.message.clone()));
            return "".to_string();
        },
    }
}

fn decode_urlsafe_b64_or_empty(payload: &String) -> String {
    let __sifr_try_res: Result<String, ParseError> = (|| {
    let decoded: String = urlsafe_b64decode(payload)?;
    return Ok(decoded);
    unreachable!("sifr try/except return capture fell through");
})();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        },
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            let _ = format!("{}", format!("{}{}", "unexpected: ", e.message.clone()));
            return "".to_string();
        },
    }
}

fn b16_encode_or_empty(payload: &String) -> String {
    let mut encoded: String = "".to_string();
    let __sifr_try_res: Result<(), ParseError> = (|| {
    let out: String = b16encode(payload)?;
    encoded = out;
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", format!("{}{}", "unexpected: ", e.message.clone()));
    }
    encoded
}

fn b16_decode_or_empty(payload: &String) -> String {
    let mut decoded: String = "".to_string();
    let __sifr_try_res: Result<(), ParseError> = (|| {
    let out: String = b16decode(payload)?;
    decoded = out;
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", format!("{}{}", "unexpected: ", e.message.clone()));
    }
    decoded
}

fn collect_positive_actual() -> Vec<String> {
    let mut actual: Vec<String> = vec![];
    actual.push(encode_b64_or_empty(&"foo".to_string()));
    actual.push(decode_b64_or_empty(&"Zm9v".to_string()));
    let urlsafe_encoded: String = encode_urlsafe_b64_or_empty(&"hello".to_string());
    let urlsafe_encoded_for_decode: String = {
    let mut __sifr_concat: String = String::with_capacity(urlsafe_encoded.len() + 0usize);
    __sifr_concat.push_str((urlsafe_encoded).as_str());
    __sifr_concat.push_str("");
    __sifr_concat
};
    actual.push(urlsafe_encoded.clone());
    actual.push(decode_urlsafe_b64_or_empty(&urlsafe_encoded_for_decode));
    let b16_encoded: String = b16_encode_or_empty(&"Hi".to_string());
    let b16_encoded_for_decode: String = {
    let mut __sifr_concat: String = String::with_capacity(b16_encoded.len() + 0usize);
    __sifr_concat.push_str((b16_encoded).as_str());
    __sifr_concat.push_str("");
    __sifr_concat
};
    actual.push(b16_encoded.clone());
    actual.push(b16_decode_or_empty(&b16_encoded_for_decode));
    actual
}

fn collect_decode_actual_ok(inputs: &Vec<String>) -> Vec<bool> {
    let mut actual_ok: Vec<bool> = vec![];
    for payload in inputs.iter().cloned() {
        let __sifr_try_res: Result<(), ParseError> = (|| {
    let decoded: String = b64decode(&payload)?;
    let _ = format!("{}", decoded);
    actual_ok.push(true);
    Ok(())
})();
        if let Err(__sifr_try_err) = __sifr_try_res {
            let e = __sifr_try_err.clone();
            actual_ok.push(false);
        }
    }
    actual_ok
}

fn main() {
    let expected: Vec<String> = vec!["Zm9v".to_string(), "foo".to_string(), "aGVsbG8=".to_string(), "hello".to_string(), "4869".to_string(), "Hi".to_string()];
    let actual: Vec<String> = collect_positive_actual();
    assert_vector_eq(&actual, &expected);
    let decode_inputs: Vec<String> = vec!["not base64!!!".to_string(), "Zm9v".to_string()];
    let expected_ok: Vec<bool> = vec![false, true];
    let actual_ok: Vec<bool> = collect_decode_actual_ok(&decode_inputs);
    assert_bool_vector_eq(&actual_ok, &expected_ok);
    println!("base64 base64 parity demo: pass");
}
