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
pub use sifr_generated_project_nominals::ParseError;
fn main() {
    let encoded: Vec<u8> = vec![
        115u8, 105u8, 102u8, 114u8, 45u8, 98u8, 121u8, 116u8, 101u8, 115u8,
    ];
    let mut decode_ok: bool = false;
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let decoded_value_84517ef0dce4ed91: String = ::sifr_runtime::encoding::decode_text(
            &encoded,
            &"utf-8".to_string(),
            &"strict".to_string(),
        )
        .map_err(|sifr_generated_message| ParseError {
            message: sifr_generated_message,
        })?;
        decode_ok = decoded_value_84517ef0dce4ed91 == "sifr-bytes";
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
    }
    assert!(decode_ok);
    let mut hex_ok: bool = false;
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let parsed_hex: Vec<u8> = {
            let s: String = "73696672".to_string();
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
        let decoded_hex: String = ::sifr_runtime::encoding::decode_text(
            &parsed_hex,
            &"utf-8".to_string(),
            &"strict".to_string(),
        )
        .map_err(|sifr_generated_message| ParseError {
            message: sifr_generated_message,
        })?;
        hex_ok = decoded_hex == "sifr";
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
    }
    assert!(hex_ok);
}
