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
#[expect(
    clippy::assertions_on_constants,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
fn main() {
    let mut size_ok: bool = false;
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let zeros: Vec<u8> = {
            let sifr_generated_size = SifrInt::from_i64(6);
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
        }?;
        assert_eq!(&SifrInt::from(zeros.len()), &SifrInt::from_i64(6));
        size_ok = true;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(23usize);
            sifr_generated_concat.push_str("unexpected ValueError: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    assert!(size_ok);
    let mut from_ints_ok: bool = false;
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let from_list: Vec<u8> = {
            let sifr_generated_vals = vec![
                SifrInt::from_i64(83),
                SifrInt::from_i64(105),
                SifrInt::from_i64(102),
                SifrInt::from_i64(114),
            ];
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
        }?;
        let first: Option<u8> = {
            let sifr_generated_checked_read_collection = &from_list;
            let sifr_generated_checked_read_index = SifrInt::from_i64(0);
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        };
        let last: Option<u8> = {
            let sifr_generated_checked_read_collection = &from_list;
            let sifr_generated_checked_read_index = SifrInt::from_i64(3);
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        };
        if let Some(first) = first {
            let expected_first: u8 = 83u8;
            assert_eq!(first, expected_first);
        } else {
            assert!(false);
        }
        if let Some(last) = last {
            let expected_last: u8 = 114u8;
            assert_eq!(last, expected_last);
        } else {
            assert!(false);
        }
        from_ints_ok = true;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(23usize);
            sifr_generated_concat.push_str("unexpected ValueError: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    assert!(from_ints_ok);
    let mut from_hex_ok: bool = false;
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let from_hex: Vec<u8> = {
            let s: String = "53 69 66 72".to_string();
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
        let from_hex_text_value_6272bce207218a0f: String = ::sifr_runtime::encoding::decode_text(
            &from_hex,
            &"utf-8".to_string(),
            &"strict".to_string(),
        )
        .map_err(|sifr_generated_message| ParseError {
            message: sifr_generated_message,
        })?;
        assert_eq!(
            from_hex_text_value_6272bce207218a0f.as_str(),
            "Sifr".to_string().as_str()
        );
        from_hex_ok = true;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(23usize);
            sifr_generated_concat.push_str("unexpected ParseError: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    assert!(from_hex_ok);
    let mut encode_ok: bool = false;
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let encoded_value_8b21351fd8aea299: Vec<u8> = ::sifr_runtime::encoding::encode_bytes(
            &"bytes_constructors-demo".to_string(),
            &"utf-8".to_string(),
            &"strict".to_string(),
        )
        .map_err(|sifr_generated_message| ParseError {
            message: sifr_generated_message,
        })?;
        let decoded: String = ::sifr_runtime::encoding::decode_text(
            &encoded_value_8b21351fd8aea299,
            &"utf-8".to_string(),
            &"strict".to_string(),
        )
        .map_err(|sifr_generated_message| ParseError {
            message: sifr_generated_message,
        })?;
        assert_eq!(
            decoded.as_str(),
            "bytes_constructors-demo".to_string().as_str()
        );
        encode_ok = true;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(23usize);
            sifr_generated_concat.push_str("unexpected ParseError: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    assert!(encode_ok);
    println!("bytes_bytes_constructors_surface_demo: ok");
}
