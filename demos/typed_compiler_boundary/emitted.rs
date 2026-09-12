// src/main.rs
mod sifr_generated_generated_support {
    use crate::{
        SifrGeneratedStdlibSifrX2ehashlibX2eHashObject, SifrGeneratedStdlibSifrX2etaskX2eContext,
    };
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) fn sha256_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::sha256_bytes(data)
    }
    pub(crate) fn md5_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::md5_bytes(data)
    }
    pub(crate) fn sha1_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::sha1_bytes(data)
    }
    pub(crate) fn sha224_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::sha224_bytes(data)
    }
    pub(crate) fn sha384_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::sha384_bytes(data)
    }
    pub(crate) fn sha512_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::sha512_bytes(data)
    }
    pub(crate) fn blake2b_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::blake2b_bytes(data)
    }
    pub(crate) fn blake2s_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::blake2s_bytes(data)
    }
    pub(crate) fn sifr_generated_build_hash(
        algorithm: &str,
        data: &[u8],
    ) -> SifrGeneratedStdlibSifrX2ehashlibX2eHashObject {
        let alg: String = algorithm.to_lowercase();
        if alg == "md5" {
            return SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
                alg,
                data.to_vec(),
                "md5".to_string(),
                SifrInt::from_i64(16),
                SifrInt::from_i64(64),
            );
        } else if alg == "sha1" {
            return SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
                alg,
                data.to_vec(),
                "sha1".to_string(),
                SifrInt::from_i64(20),
                SifrInt::from_i64(64),
            );
        } else if alg == "sha224" {
            return SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
                alg,
                data.to_vec(),
                "sha224".to_string(),
                SifrInt::from_i64(28),
                SifrInt::from_i64(64),
            );
        } else if alg == "sha256" {
            return SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
                alg,
                data.to_vec(),
                "sha256".to_string(),
                SifrInt::from_i64(32),
                SifrInt::from_i64(64),
            );
        } else if alg == "sha384" {
            return SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
                alg,
                data.to_vec(),
                "sha384".to_string(),
                SifrInt::from_i64(48),
                SifrInt::from_i64(128),
            );
        } else if alg == "sha512" {
            return SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
                alg,
                data.to_vec(),
                "sha512".to_string(),
                SifrInt::from_i64(64),
                SifrInt::from_i64(128),
            );
        } else if alg == "blake2b" {
            return SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
                alg,
                data.to_vec(),
                "blake2b".to_string(),
                SifrInt::from_i64(64),
                SifrInt::from_i64(128),
            );
        } else if alg == "blake2s" {
            return SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
                alg,
                data.to_vec(),
                "blake2s".to_string(),
                SifrInt::from_i64(32),
                SifrInt::from_i64(64),
            );
        }
        SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
            alg,
            data.to_vec(),
            "unknown".to_string(),
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
        )
    }
    pub(crate) fn sifr_generated_hash_bytes(algorithm: &str, data: &[u8]) -> Vec<u8> {
        if algorithm == "md5" {
            return md5_bytes(data);
        } else if algorithm == "sha1" {
            return sha1_bytes(data);
        } else if algorithm == "sha224" {
            return sha224_bytes(data);
        } else if algorithm == "sha256" {
            return sha256_bytes(data);
        } else if algorithm == "sha384" {
            return sha384_bytes(data);
        } else if algorithm == "sha512" {
            return sha512_bytes(data);
        } else if algorithm == "blake2b" {
            return blake2b_bytes(data);
        } else if algorithm == "blake2s" {
            return blake2s_bytes(data);
        }
        Vec::new()
    }
    pub(crate) fn sifr_generated_hash_hex(algorithm: &str, data: &[u8]) -> String {
        {
            let sifr_generated_bytes_receiver: &[u8] = &sifr_generated_hash_bytes(algorithm, data);
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
    }
    pub(crate) fn sha224(data: &[u8]) -> SifrGeneratedStdlibSifrX2ehashlibX2eHashObject {
        sifr_generated_build_hash(&"sha224".to_string(), data)
    }
    ::tokio::task_local! {
        pub (crate) static SIFR_GENERATED_SIFR_TASK_CONTEXT_LABEL : String;
    }
    pub(crate) fn sifr_generated_task_current_context() -> SifrGeneratedStdlibSifrX2etaskX2eContext
    {
        SifrGeneratedStdlibSifrX2etaskX2eContext::new(
            SIFR_GENERATED_SIFR_TASK_CONTEXT_LABEL
                .try_with(Clone::clone)
                .unwrap_or("Context".to_string()),
        )
    }
}
mod sifr_generated_project_nominals {
    use crate::sifr_generated_generated_support::*;
    use ::sifr_runtime::SifrInt;
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2ehashlibX2eHashObject {
        pub algorithm: String,
        pub data: Vec<u8>,
        pub name: String,
        pub digest_size: SifrInt,
        pub block_size: SifrInt,
    }
    impl SifrGeneratedStdlibSifrX2ehashlibX2eHashObject {
        #[must_use]
        pub fn new(
            algorithm: String,
            data: Vec<u8>,
            name: String,
            digest_size: SifrInt,
            block_size: SifrInt,
        ) -> Self {
            let sifr_generated_field_value_ddb1f39e0a66bbbb_5f616c676f726974686d: String =
                algorithm;
            let sifr_generated_field_value_90770dc80a1c57ce_5f64617461: Vec<u8> = data;
            let sifr_generated_field_value_c4bcadba8e631b86_6e616d65: String = name;
            let sifr_generated_field_value_6344303e03c9f7c7_6469676573745f73697a65: SifrInt =
                digest_size.clone();
            let sifr_generated_field_value_e190162752f8783e_626c6f636b5f73697a65: SifrInt =
                block_size.clone();
            Self {
                algorithm: sifr_generated_field_value_ddb1f39e0a66bbbb_5f616c676f726974686d,
                data: sifr_generated_field_value_90770dc80a1c57ce_5f64617461,
                name: sifr_generated_field_value_c4bcadba8e631b86_6e616d65,
                digest_size: sifr_generated_field_value_6344303e03c9f7c7_6469676573745f73697a65,
                block_size: sifr_generated_field_value_e190162752f8783e_626c6f636b5f73697a65,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2ehashlibX2eHashObject {
        #[must_use]
        pub fn hexdigest(&self) -> String {
            sifr_generated_hash_hex(&self.algorithm, &self.data)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2etaskX2eContext {
        pub name: String,
    }
    impl SifrGeneratedStdlibSifrX2etaskX2eContext {
        #[must_use]
        pub const fn new(name: String) -> Self {
            let sifr_generated_field_value_c4bcadba8e631b86_6e616d65: String = name;
            Self {
                name: sifr_generated_field_value_c4bcadba8e631b86_6e616d65,
            }
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2etaskX2eContext {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.name.clone())
        }
    }
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
use crate::sifr_generated_generated_support::*;
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::ParseError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ehashlibX2eHashObject;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2etaskX2eContext;
fn assert_eq(left: SifrInt, right: SifrInt) -> SifrInt {
    &left + &right
}
#[expect(
    clippy::assertions_on_constants,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
fn main() {
    assert_eq!(
        assert_eq(SifrInt::from_i64(20), SifrInt::from_i64(22)),
        SifrInt::from_i64(42)
    );
    assert_eq!(sifr_generated_task_current_context().to_string(), "Context");
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let payload: Vec<u8> = {
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
        let text: String = ::sifr_runtime::encoding::decode_text(
            &payload,
            &"utf-8".to_string(),
            &"strict".to_string(),
        )
        .map_err(|sifr_generated_message| ParseError {
            message: sifr_generated_message,
        })?;
        assert_eq!(text, "Sifr");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        assert!(false);
    }
    assert_eq!(
        SifrInt::from(
            sha224(&vec![
                116_u8, 121_u8, 112_u8, 101_u8, 100_u8, 32_u8, 98_u8, 111_u8, 117_u8, 110_u8,
                100_u8, 97_u8, 114_u8, 121_u8
            ])
            .hexdigest()
            .chars()
            .count()
        ),
        SifrInt::from_i64(56)
    );
    println!("typed compiler boundary: ok");
}
