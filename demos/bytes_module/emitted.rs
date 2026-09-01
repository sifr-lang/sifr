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
fn render_opt_int(value: Option<SifrInt>) -> String {
    let Some(value) = value.clone() else {
        return "None".to_string();
    };
    value.to_string()
}
fn collect_primary_actual(payload: &[u8]) -> Vec<String> {
    let actual: Vec<String> = vec![
        {
            let sifr_generated_bytes_receiver = &payload;
            {
                let sifr_generated_needle = SifrInt::from_i64(115);
                match sifr_generated_needle.try_to_u8() {
                    Ok(sifr_generated_needle_u8) => SifrInt::from(
                        sifr_generated_bytes_receiver
                            .iter()
                            .filter(|sifr_generated_x| {
                                **sifr_generated_x == sifr_generated_needle_u8
                            })
                            .count(),
                    ),
                    Err(_) => SifrInt::from_i64(0),
                }
            }
        }
        .to_string(),
        render_opt_int({
            let sifr_generated_bytes_receiver = &payload;
            {
                let sifr_generated_needle = SifrInt::from_i64(45);
                match sifr_generated_needle.try_to_u8() {
                    Ok(sifr_generated_needle_u8) => {
                        let sifr_generated_len = sifr_generated_bytes_receiver.len();
                        let sifr_generated_start = 0_usize;
                        let sifr_generated_stop = sifr_generated_len;
                        let mut sifr_generated_i = sifr_generated_start;
                        let mut sifr_generated_result = None;
                        while sifr_generated_i < sifr_generated_stop
                            && sifr_generated_result == None
                        {
                            if let Some(sifr_generated_x) =
                                sifr_generated_bytes_receiver.get(sifr_generated_i)
                                && *sifr_generated_x == sifr_generated_needle_u8
                            {
                                sifr_generated_result = Some(SifrInt::from(sifr_generated_i));
                            }
                            sifr_generated_i += 1_usize;
                        }
                        sifr_generated_result
                    }
                    Err(_) => None,
                }
            }
        }),
        payload
            .starts_with(&vec![98u8, 121u8, 116u8, 101u8, 115u8])
            .to_string(),
        payload.ends_with(&vec![101u8, 51u8, 48u8]).to_string(),
    ];
    actual
}
fn bytes_to_hex_or_empty(payload: &[u8]) -> String {
    let sifr_generated_try_res: Result<String, ParseError> = {
        let hx: String = {
            let sifr_generated_bytes_receiver = &payload;
            let mut sifr_generated_hex =
                String::with_capacity(sifr_generated_bytes_receiver.len().saturating_mul(2_usize));
            for sifr_generated_byte in sifr_generated_bytes_receiver.iter() {
                sifr_generated_hex.push_str(&format!("{:02x}", *sifr_generated_byte));
            }
            sifr_generated_hex
        };
        Ok(hx)
    };
    sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
        String::new()
    })
}
fn bytes_from_hex_to_text_or_empty(payload: &str) -> String {
    let sifr_generated_try_res: Result<String, ParseError> = (|| {
        let parsed: Vec<u8> = {
            let s: String = payload.to_string();
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
        let txt: String = ::sifr_runtime::encoding::decode_text(
            &parsed,
            &"utf-8".to_string(),
            &"strict".to_string(),
        )
        .map_err(|sifr_generated_message| ParseError {
            message: sifr_generated_message,
        })?;
        Ok(txt)
    })();
    sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
        String::new()
    })
}
fn collect_invalid_actual_ok() -> Vec<bool> {
    let mut invalid_actual_ok: Vec<bool> = Vec::new();
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let odd: Vec<u8> = {
            let s: String = "abc".to_string();
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
        let _ = SifrInt::from(odd.len()).to_string();
        invalid_actual_ok.push(true);
        Ok(())
    })();
    if sifr_generated_try_res.is_err() {
        invalid_actual_ok.push(false);
    }
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let bad_utf8: String = ::sifr_runtime::encoding::decode_text(
            &vec![255u8],
            &"utf-8".to_string(),
            &"strict".to_string(),
        )
        .map_err(|sifr_generated_message| ParseError {
            message: sifr_generated_message,
        })?;
        let _ = bad_utf8.to_string();
        invalid_actual_ok.push(true);
        Ok(())
    })();
    if sifr_generated_try_res.is_err() {
        invalid_actual_ok.push(false);
    }
    invalid_actual_ok
}
fn main() {
    let payload: Vec<u8> = vec![
        98u8, 121u8, 116u8, 101u8, 115u8, 45u8, 98u8, 121u8, 116u8, 101u8, 115u8, 95u8, 109u8,
        111u8, 100u8, 117u8, 108u8, 101u8,
    ];
    let expected: Vec<String> = vec![
        "2".to_string(),
        "5".to_string(),
        "true".to_string(),
        "false".to_string(),
    ];
    let actual: Vec<String> = collect_primary_actual(&payload);
    assert_vector_eq(&actual, &expected);
    let hex_text: String = bytes_to_hex_or_empty(&vec![72u8, 105u8]);
    let _ = hex_text.chars().collect::<Vec<char>>();
    assert_eq!(
        (&SifrInt::from(hex_text.chars().count()) > &SifrInt::from_i64(0)).to_string(),
        "true"
    );
    assert_eq!(hex_text.to_string(), "4869");
    let roundtrip_text: String = bytes_from_hex_to_text_or_empty(&"48 69".to_string());
    let _ = roundtrip_text.chars().collect::<Vec<char>>();
    assert_eq!(
        (&SifrInt::from(roundtrip_text.chars().count()) > &SifrInt::from_i64(0)).to_string(),
        "true"
    );
    assert_eq!(roundtrip_text.to_string(), "Hi");
    let invalid_expected_ok: Vec<bool> = vec![false, false];
    let invalid_actual_ok: Vec<bool> = collect_invalid_actual_ok();
    assert_bool_vector_eq(&invalid_actual_ok, &invalid_expected_ok);
    println!("bytes_module bytes parity demo: pass");
}
