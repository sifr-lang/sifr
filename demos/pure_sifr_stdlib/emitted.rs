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
fn sha256_bytes(data: &[u8]) -> Vec<u8> {
    ::sifr_stdlib::hash::sha256_bytes(data)
}
fn md5_bytes(data: &[u8]) -> Vec<u8> {
    ::sifr_stdlib::hash::md5_bytes(data)
}
fn sha1_bytes(data: &[u8]) -> Vec<u8> {
    ::sifr_stdlib::hash::sha1_bytes(data)
}
fn sha224_bytes(data: &[u8]) -> Vec<u8> {
    ::sifr_stdlib::hash::sha224_bytes(data)
}
fn sha384_bytes(data: &[u8]) -> Vec<u8> {
    ::sifr_stdlib::hash::sha384_bytes(data)
}
fn sha512_bytes(data: &[u8]) -> Vec<u8> {
    ::sifr_stdlib::hash::sha512_bytes(data)
}
fn blake2b_bytes(data: &[u8]) -> Vec<u8> {
    ::sifr_stdlib::hash::blake2b_bytes(data)
}
fn blake2s_bytes(data: &[u8]) -> Vec<u8> {
    ::sifr_stdlib::hash::blake2s_bytes(data)
}
fn b64encode(s: &str) -> String {
    base64_encode(s)
}
fn b64decode(s: &str) -> Result<String, ParseError> {
    base64_decode(s)
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SifrGeneratedStdlibSifrX2ehashlibX2eHashObject {
    algorithm: String,
    data: Vec<u8>,
    name: String,
    digest_size: SifrInt,
    block_size: SifrInt,
}
impl SifrGeneratedStdlibSifrX2ehashlibX2eHashObject {
    fn new(
        algorithm: String,
        data: Vec<u8>,
        name: String,
        digest_size: SifrInt,
        block_size: SifrInt,
    ) -> Self {
        let sifr_generated_field_value_ddb1f39e0a66bbbb_5f616c676f726974686d: String = algorithm;
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
    fn hexdigest(&self) -> String {
        sifr_generated_hash_hex(&self.algorithm, &self.data)
    }
}
fn sifr_generated_build_hash(
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
fn sifr_generated_hash_bytes(algorithm: &str, data: &[u8]) -> Vec<u8> {
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
    {
        let sifr_generated_empty_bytes_literal: Vec<u8> = Vec::new();
        sifr_generated_empty_bytes_literal
    }
}
fn sifr_generated_hash_hex(algorithm: &str, data: &[u8]) -> String {
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
fn sha256(data: &[u8]) -> SifrGeneratedStdlibSifrX2ehashlibX2eHashObject {
    sifr_generated_build_hash(&"sha256".to_string(), data)
}
#[expect(
    clippy::approx_constant,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
const PI: f64 = 3.141_592_653_589_793_f64;
fn sqrt(x: f64) -> f64 {
    ::sifr_stdlib::math::sqrt(x)
}
#[expect(
    clippy::approx_constant,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
fn main() {
    assert_eq!(
        &SifrInt::from_i64(1) + &SifrInt::from_i64(1),
        SifrInt::from_i64(2)
    );
    assert!(true);
    let result: f64 = sqrt(9.0_f64);
    assert_eq!(result, 3.0_f64);
    assert!(PI > 3.14_f64);
    let h: String = sha256(&vec![104u8, 101u8, 108u8, 108u8, 111u8]).hexdigest();
    let _ = h.chars().collect::<Vec<char>>();
    assert_eq!(&SifrInt::from(h.chars().count()), &SifrInt::from_i64(64));
    let encoded: String = b64encode(&"Hello!".to_string());
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let decoded: String = b64decode(&encoded)?;
        assert_eq!(decoded, "Hello!");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let err = sifr_generated_try_err.clone();
        println!("base64 error: {}", err.message.clone());
        assert_eq!(
            format!("base64 error: {}", err.message.clone()),
            "stdlib_migration demo: all checks passed!"
        );
    }
    println!("stdlib_migration demo: all checks passed!");
}
