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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueError {
        pub message: String,
    }
    impl ::std::fmt::Display for ValueError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ValueError {}
}
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::ParseError;
pub use sifr_generated_project_nominals::ValueError;
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    let mut bad_size: bool = false;
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let _ = {
            let sifr_generated_size = -&SifrInt::from_i64(1);
            if &sifr_generated_size < &0 {
                return Err(ValueError {
                    message: "bytes(size) requires a non-negative size".to_string(),
                });
            }
            let Ok(sifr_generated_size) = sifr_generated_size.try_to_usize() else {
                return Err(ValueError {
                    message: "bytes(size) exceeds the addressable size".to_string(),
                });
            };
            Ok::<Vec<u8>, ValueError>((0..sifr_generated_size).map(|_| 0_u8).collect::<Vec<u8>>())
        };
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        bad_size = true;
    }
    let mut bad_values: bool = false;
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let _ = {
            let sifr_generated_vals = vec![SifrInt::from_i64(0), SifrInt::from_i64(999)];
            let mut sifr_generated_out = Vec::new();
            for sifr_generated_pair in sifr_generated_vals.iter().enumerate() {
                sifr_generated_out.push(sifr_generated_pair.1.try_to_u8().map_err(|_error| {
                    ValueError {
                        message: format!(
                            "byte out of range at index {}: {}",
                            sifr_generated_pair.0, *sifr_generated_pair.1
                        ),
                    }
                })?);
            }
            Ok::<Vec<u8>, ValueError>(sifr_generated_out)
        };
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        bad_values = true;
    }
    let mut bad_hex: bool = false;
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let _ = {
            let s: String = "GG".to_string();
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
        };
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        bad_hex = true;
    }
    let mut bad_codec: bool = false;
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let codec: String = "definitely-not-a-codec".to_string();
        let _encoded: Vec<u8> = ::sifr_runtime::encoding::encode_bytes(
            &"abc".to_string(),
            &codec,
            &"strict".to_string(),
        )
        .map_err(|sifr_generated_message| ParseError {
            message: sifr_generated_message,
        })?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        bad_codec = true;
    }
    let mut bad_utf8: bool = false;
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let _invalid_utf8: String = ::sifr_runtime::encoding::decode_text(
            &vec![255u8],
            &"utf-8".to_string(),
            &"strict".to_string(),
        )
        .map_err(|sifr_generated_message| ParseError {
            message: sifr_generated_message,
        })?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        bad_utf8 = true;
    }
    assert!(bad_size);
    assert!(bad_values);
    assert!(bad_hex);
    assert!(bad_codec);
    assert!(bad_utf8);
    println!("bytes_bytes_errors_boundary_demo: ok");
}
