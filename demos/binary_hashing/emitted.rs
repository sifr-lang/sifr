// src/main.rs
mod sifr_generated_project_nominals {
    pub use ::sifr_runtime::SifrInt;
    #[must_use]
    pub fn sha256_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::sha256_bytes(data)
    }
    #[must_use]
    pub fn md5_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::md5_bytes(data)
    }
    #[must_use]
    pub fn sha1_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::sha1_bytes(data)
    }
    #[must_use]
    pub fn sha224_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::sha224_bytes(data)
    }
    #[must_use]
    pub fn sha384_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::sha384_bytes(data)
    }
    #[must_use]
    pub fn sha512_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::sha512_bytes(data)
    }
    #[must_use]
    pub fn blake2b_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::blake2b_bytes(data)
    }
    #[must_use]
    pub fn blake2s_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::blake2s_bytes(data)
    }
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
    impl SifrGeneratedStdlibSifrX2ehashlibX2eHashObject {
        #[must_use]
        pub fn digest(&self) -> Vec<u8> {
            sifr_generated_hash_bytes(&self.algorithm, &self.data)
        }
    }
    #[must_use]
    pub fn sifr_generated_hash_bytes(algorithm: &str, data: &[u8]) -> Vec<u8> {
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
    #[must_use]
    pub fn sifr_generated_hash_hex(algorithm: &str, data: &[u8]) -> String {
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
    impl ValueError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for ValueError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ValueError {}
}
pub use sifr_generated_project_nominals::ParseError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ehashlibX2eHashObject;
pub use sifr_generated_project_nominals::ValueError;
mod sifr_generated_project_unions {
    #[derive(Debug, Clone)]
    pub enum SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a023X3a5X3aclass10X3aValueError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass10X3aParseError1X3a0(
            crate::sifr_generated_project_nominals::ParseError,
        ),
        SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
            crate::sifr_generated_project_nominals::ValueError,
        ),
    }
    impl From<crate::sifr_generated_project_nominals::ParseError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a023X3a5X3aclass10X3aValueError1X3a0 {
        fn from(value: crate::sifr_generated_project_nominals::ParseError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a023X3a5X3aclass10X3aValueError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aParseError1X3a0(
                value,
            )
        }
    }
    impl From<crate::sifr_generated_project_nominals::ValueError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a023X3a5X3aclass10X3aValueError1X3a0 {
        fn from(value: crate::sifr_generated_project_nominals::ValueError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a023X3a5X3aclass10X3aValueError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a023X3a5X3aclass10X3aValueError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a023X3a5X3aclass10X3aValueError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aParseError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a023X3a5X3aclass10X3aValueError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
}
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_unions::SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a023X3a5X3aclass10X3aValueError1X3a0;
fn base64_encode_bytes(data: &[u8]) -> Vec<u8> {
    ::sifr_stdlib::base64::base64_encode_bytes(data)
}
fn base64_decode_bytes(data: &[u8]) -> Result<Vec<u8>, ParseError> {
    ::sifr_stdlib::base64::base64_decode_bytes(data).map_err(|sifr_generated_bridge_error| {
        ParseError {
            message: sifr_generated_bridge_error.to_string(),
        }
    })
}
fn b64encode_bytes(data: &[u8]) -> Vec<u8> {
    base64_encode_bytes(data)
}
fn b64decode_bytes(data: &[u8]) -> Result<Vec<u8>, ParseError> {
    base64_decode_bytes(data)
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
fn sifr_generated_is_supported_algorithm(name: &str) -> bool {
    let n: String = name.to_lowercase();
    n == "md5"
        || n == "sha1"
        || n == "sha224"
        || n == "sha256"
        || n == "sha384"
        || n == "sha512"
        || n == "blake2b"
        || n == "blake2s"
}
fn new(
    name: &str,
    data: &[u8],
) -> Result<SifrGeneratedStdlibSifrX2ehashlibX2eHashObject, ValueError> {
    if !sifr_generated_is_supported_algorithm(name) {
        return Err(ValueError::new({
            let mut sifr_generated_concat: String = String::with_capacity(28usize + name.len());
            sifr_generated_concat.push_str("unsupported hash algorithm: ");
            sifr_generated_concat.push_str(name);
            sifr_generated_concat
        }));
    }
    Ok(sifr_generated_build_hash(name, data))
}
fn main() {
    let sifr_generated_try_res: Result<
        (),
        SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a023X3a5X3aclass10X3aValueError1X3a0,
    > = (|| {
        let data: Vec<u8> = vec![
            98u8, 105u8, 110u8, 97u8, 114u8, 121u8, 95u8, 104u8, 97u8, 115u8, 104u8,
            105u8, 110u8, 103u8, 45u8, 98u8, 121u8, 116u8, 101u8, 115u8, 45u8, 100u8,
            101u8, 109u8, 111u8
        ];
        let h: SifrGeneratedStdlibSifrX2ehashlibX2eHashObject = new(
                &"sha256".to_string(),
                &data,
            )
            .map_err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a023X3a5X3aclass10X3aValueError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0,
            )?;
        assert_eq!(& SifrInt::from(h.digest().len()), & SifrInt::from_i64(32));
        assert_eq!(
            & SifrInt::from(h.hexdigest().chars().count()), & SifrInt::from_i64(64)
        );
        let enc: Vec<u8> = b64encode_bytes(&data);
        let dec: Vec<u8> = b64decode_bytes(&enc)
            .map_err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a023X3a5X3aclass10X3aValueError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aParseError1X3a0,
            )?;
        assert_eq!(dec, data);
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        match sifr_generated_try_err {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a023X3a5X3aclass10X3aValueError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aParseError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let e = sifr_generated_try_variant_error.clone();
                assert_eq!(
                    format!("unexpected parse error: {}", e.message.clone()),
                    "rng_binary_hashing_base64_bytes_demo: pass"
                );
            }
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a023X3a5X3aclass10X3aValueError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let e = sifr_generated_try_variant_error.clone();
                assert_eq!(
                    format!("unexpected value error: {}", e.message.clone()),
                    "rng_binary_hashing_base64_bytes_demo: pass"
                );
            }
        }
    }
    assert_eq!(
        "rng_binary_hashing_base64_bytes_demo: pass".to_string(),
        "rng_binary_hashing_base64_bytes_demo: pass"
    );
}
