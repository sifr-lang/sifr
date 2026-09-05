// src/main.rs
pub mod sifr_generated_generated_support {
    use crate::{
        IOError, SifrGeneratedIoNativeFileHandle, SifrGeneratedStdlibSifrX2ehashlibX2eHashObject,
        SifrGeneratedStdlibSifrX2ehashlibX2eHashlibError, ValueError,
    };
    pub(super) use ::sifr_runtime::SifrInt;
    pub(super) fn sha256_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::sha256_bytes(data)
    }
    pub(super) fn md5_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::md5_bytes(data)
    }
    pub(super) fn sha1_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::sha1_bytes(data)
    }
    pub(super) fn sha224_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::sha224_bytes(data)
    }
    pub(super) fn sha384_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::sha384_bytes(data)
    }
    pub(super) fn sha512_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::sha512_bytes(data)
    }
    pub(super) fn blake2b_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::blake2b_bytes(data)
    }
    pub(super) fn blake2s_bytes(data: &[u8]) -> Vec<u8> {
        ::sifr_stdlib::hash::blake2s_bytes(data)
    }
    pub(super) fn write_text(path: &str, content: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::write_text(path, content).map_err(sifr_generated_io_err)
    }
    pub(super) fn sifr_generated_open_file(path: &str, mode: &str) -> Result<String, IOError> {
        ::sifr_stdlib::fs::open_file(path, mode).map_err(sifr_generated_io_err)
    }
    pub(super) fn sifr_generated_file_close(handle: &str) {
        ::sifr_stdlib::fs::file_close(handle);
    }
    pub(super) fn sifr_generated_file_read_bytes(
        handle: &str,
        size: Option<&SifrInt>,
    ) -> Result<Vec<u8>, IOError> {
        let size: Option<SifrInt> = size.cloned();
        ::sifr_stdlib::fs::file_read_bytes(
            handle,
            size.map(::sifr_runtime::interop::SifrIntBridge::from),
        )
        .map_err(sifr_generated_io_err)
    }
    pub(super) fn open_file(
        path: &str,
        mode: &str,
    ) -> Result<SifrGeneratedIoNativeFileHandle, IOError> {
        let sifr_generated_try_res: Result<
            Result<SifrGeneratedIoNativeFileHandle, IOError>,
            IOError,
        > = (|| {
            let handle_id: String = sifr_generated_open_file(path, mode)?;
            Ok(Ok(SifrGeneratedIoNativeFileHandle::new(handle_id)))
        })();
        sifr_generated_try_res.unwrap_or_else(
            |sifr_generated_try_err_user_736966725f67656e6572617465645f7472795f657272| {
                let e = sifr_generated_try_err_user_736966725f67656e6572617465645f7472795f657272;
                Err(e)
            },
        )
    }
    pub(super) fn file_close(handle: &SifrGeneratedIoNativeFileHandle) {
        sifr_generated_file_close(handle.id.as_str());
    }
    pub(super) fn file_read_bytes(
        handle: &SifrGeneratedIoNativeFileHandle,
        size: Option<&SifrInt>,
    ) -> Result<Vec<u8>, IOError> {
        let size: Option<SifrInt> = size.cloned();
        sifr_generated_file_read_bytes(handle.id.as_str(), size.as_ref())
    }
    pub(super) fn sifr_generated_build_hash(
        algorithm: &str,
        data: &[u8],
    ) -> SifrGeneratedStdlibSifrX2ehashlibX2eHashObject {
        let alg: String = algorithm.to_lowercase();
        if alg == "md5" {
            return SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
                alg,
                data.to_vec(),
                "md5".to_string(),
                &SifrInt::from_i64(16),
                &SifrInt::from_i64(64),
            );
        } else if alg == "sha1" {
            return SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
                alg,
                data.to_vec(),
                "sha1".to_string(),
                &SifrInt::from_i64(20),
                &SifrInt::from_i64(64),
            );
        } else if alg == "sha224" {
            return SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
                alg,
                data.to_vec(),
                "sha224".to_string(),
                &SifrInt::from_i64(28),
                &SifrInt::from_i64(64),
            );
        } else if alg == "sha256" {
            return SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
                alg,
                data.to_vec(),
                "sha256".to_string(),
                &SifrInt::from_i64(32),
                &SifrInt::from_i64(64),
            );
        } else if alg == "sha384" {
            return SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
                alg,
                data.to_vec(),
                "sha384".to_string(),
                &SifrInt::from_i64(48),
                &SifrInt::from_i64(128),
            );
        } else if alg == "sha512" {
            return SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
                alg,
                data.to_vec(),
                "sha512".to_string(),
                &SifrInt::from_i64(64),
                &SifrInt::from_i64(128),
            );
        } else if alg == "blake2b" {
            return SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
                alg,
                data.to_vec(),
                "blake2b".to_string(),
                &SifrInt::from_i64(64),
                &SifrInt::from_i64(128),
            );
        } else if alg == "blake2s" {
            return SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
                alg,
                data.to_vec(),
                "blake2s".to_string(),
                &SifrInt::from_i64(32),
                &SifrInt::from_i64(64),
            );
        }
        SifrGeneratedStdlibSifrX2ehashlibX2eHashObject::new(
            alg,
            data.to_vec(),
            "unknown".to_string(),
            &SifrInt::from_i64(0),
            &SifrInt::from_i64(0),
        )
    }
    pub(super) fn copy_hash(
        h: &SifrGeneratedStdlibSifrX2ehashlibX2eHashObject,
    ) -> SifrGeneratedStdlibSifrX2ehashlibX2eHashObject {
        sifr_generated_build_hash(h.algorithm.as_str(), &h.data.clone())
    }
    pub(super) fn sifr_generated_is_supported_algorithm(name: &str) -> bool {
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
    pub(super) fn sifr_generated_hash_bytes(algorithm: &str, data: &[u8]) -> Vec<u8> {
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
        Vec::<u8>::new()
    }
    pub(super) fn sifr_generated_hash_hex(algorithm: &str, data: &[u8]) -> String {
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
    pub(super) fn new(
        name: &str,
        data: &[u8],
    ) -> Result<SifrGeneratedStdlibSifrX2ehashlibX2eHashObject, ValueError> {
        if !sifr_generated_is_supported_algorithm(name) {
            return Err(ValueError::new({
                let mut sifr_generated_concat: String =
                    String::with_capacity(28usize.saturating_add(name.len()));
                sifr_generated_concat.push_str("unsupported hash algorithm: ");
                sifr_generated_concat.push_str(name);
                sifr_generated_concat
            }));
        }
        Ok(sifr_generated_build_hash(name, data))
    }
    pub(super) fn algorithms_guaranteed() -> Vec<String> {
        vec![
            "md5".to_string(),
            "sha1".to_string(),
            "sha224".to_string(),
            "sha256".to_string(),
            "sha384".to_string(),
            "sha512".to_string(),
            "blake2b".to_string(),
            "blake2s".to_string(),
        ]
    }
    pub(super) fn file_digest(
        path: &str,
        name: &str,
    ) -> Result<String, SifrGeneratedStdlibSifrX2ehashlibX2eHashlibError> {
        let sifr_generated_try_res: Result<(SifrGeneratedIoNativeFileHandle,), IOError> = (|| {
            let handle: SifrGeneratedIoNativeFileHandle = open_file(path, "rb")?;
            Ok((handle,))
        })(
        );
        let (handle,) = match sifr_generated_try_res {
            Ok(sifr_generated_try_bindings) => sifr_generated_try_bindings,
            Err(sifr_generated_try_err_user_736966725f67656e6572617465645f7472795f657272) => {
                let e = sifr_generated_try_err_user_736966725f67656e6572617465645f7472795f657272;
                return Err(SifrGeneratedStdlibSifrX2ehashlibX2eHashlibError::new(
                    e.message,
                ));
            }
        };
        let sifr_generated_try_res: Result<(Vec<u8>,), IOError> = (|| {
            let data: Vec<u8> = file_read_bytes(&handle, None)?;
            Ok((data,))
        })();
        let (data,) = match sifr_generated_try_res {
            Ok(sifr_generated_try_bindings) => sifr_generated_try_bindings,
            Err(sifr_generated_try_err_user_736966725f67656e6572617465645f7472795f657272) => {
                let e = sifr_generated_try_err_user_736966725f67656e6572617465645f7472795f657272;
                file_close(&handle);
                return Err(SifrGeneratedStdlibSifrX2ehashlibX2eHashlibError::new(
                    e.message,
                ));
            }
        };
        file_close(&handle);
        let sifr_generated_try_res: Result<
            Result<String, SifrGeneratedStdlibSifrX2ehashlibX2eHashlibError>,
            ValueError,
        > = (|| {
            let h: SifrGeneratedStdlibSifrX2ehashlibX2eHashObject = new(name, &data)?;
            Ok(Ok(h.hexdigest()))
        })();
        sifr_generated_try_res.unwrap_or_else(
            |sifr_generated_try_err_user_736966725f67656e6572617465645f7472795f657272| {
                let e = sifr_generated_try_err_user_736966725f67656e6572617465645f7472795f657272;
                Err(SifrGeneratedStdlibSifrX2ehashlibX2eHashlibError::new(
                    e.message,
                ))
            },
        )
    }
    pub(super) fn md5(data: &[u8]) -> SifrGeneratedStdlibSifrX2ehashlibX2eHashObject {
        sifr_generated_build_hash("md5", data)
    }
    pub(super) fn sha256(data: &[u8]) -> SifrGeneratedStdlibSifrX2ehashlibX2eHashObject {
        sifr_generated_build_hash("sha256", data)
    }
    pub(super) fn assert_vector_eq(actual: &[String], expected: &[String]) {
        assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < actual.len() {
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
            i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
        }
    }
    pub(super) fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
        assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < actual.len() {
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
            i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
        }
    }
    pub(super) fn sifr_generated_io_err<E: ::std::fmt::Display + 'static>(e: E) -> IOError {
        let msg = e.to_string();
        let kind = {
            let sifr_generated_io_kind = (&e as &dyn ::std::any::Any)
                .downcast_ref::<std::io::Error>()
                .map(::std::io::Error::kind);
            match sifr_generated_io_kind {
                Some(::std::io::ErrorKind::NotFound) => "FileNotFound".to_string(),
                Some(::std::io::ErrorKind::PermissionDenied) => "PermissionDenied".to_string(),
                Some(::std::io::ErrorKind::AlreadyExists) => "FileExists".to_string(),
                Some(::std::io::ErrorKind::IsADirectory) => "IsADirectory".to_string(),
                Some(::std::io::ErrorKind::NotADirectory) => "NotADirectory".to_string(),
                Some(::std::io::ErrorKind::DirectoryNotEmpty) => "DirectoryNotEmpty".to_string(),
                _ => "Other".to_string(),
            }
        };
        IOError { message: msg, kind }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SifrGeneratedIoNativeFileHandle {
    pub id: String,
}
impl SifrGeneratedIoNativeFileHandle {
    #[must_use]
    pub const fn new(id: String) -> Self {
        let sifr_generated_field_value_b90e3b1a0ca5e613_5f6964: String = id;
        Self {
            id: sifr_generated_field_value_b90e3b1a0ca5e613_5f6964,
        }
    }
}
impl ::std::fmt::Display for SifrGeneratedIoNativeFileHandle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "NativeFileHandle(_id={})", self.id)
    }
}
mod sifr_generated_project_nominals {
    use crate::sifr_generated_generated_support::{
        sifr_generated_hash_bytes, sifr_generated_hash_hex,
    };
    use ::sifr_runtime::SifrInt;
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2ehashlibX2eHashlibError {
        pub message: String,
    }
    impl SifrGeneratedStdlibSifrX2ehashlibX2eHashlibError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Debug for SifrGeneratedStdlibSifrX2ehashlibX2eHashlibError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            f.debug_struct("HashlibError")
                .field("message", &self.message)
                .finish()
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2ehashlibX2eHashlibError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }
    impl ::std::error::Error for SifrGeneratedStdlibSifrX2ehashlibX2eHashlibError {}
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
            digest_size: &SifrInt,
            block_size: &SifrInt,
        ) -> Self {
            let sifr_generated_field_value_ddb1f39e0a66bbbb_5f616c676f726974686d: String =
                algorithm;
            let sifr_generated_field_value_90770dc80a1c57ce_5f64617461: Vec<u8> = data;
            let sifr_generated_field_value_c4bcadba8e631b86_6e616d65: String = name;
            let sifr_generated_field_value_6344303e03c9f7c7_6469676573745f73697a65: SifrInt =
                (*digest_size).clone();
            let sifr_generated_field_value_e190162752f8783e_626c6f636b5f73697a65: SifrInt =
                (*block_size).clone();
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
        pub fn update(&mut self, data: &[u8]) {
            self.data = {
                let mut sifr_generated_v = self.data.clone().clone();
                sifr_generated_v.extend(data.iter().copied());
                sifr_generated_v
            };
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct IOError {
        pub message: String,
        pub kind: String,
    }
    impl ::std::fmt::Display for IOError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for IOError {}
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
use crate::sifr_generated_generated_support::{
    algorithms_guaranteed, assert_bool_vector_eq, assert_vector_eq, copy_hash, file_digest, md5,
    new, sha256, write_text,
};
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::IOError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ehashlibX2eHashObject;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ehashlibX2eHashlibError;
pub use sifr_generated_project_nominals::ValueError;
fn contains(values: &[String], needle: &str) -> bool {
    #[expect(
        clippy::explicit_iter_loop,
        reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
    )]
    for v in values.iter() {
        if v == needle {
            return true;
        }
    }
    false
}
fn collect_positive_actual(tmp_path: &str) -> Vec<String> {
    let mut actual: Vec<String> = Vec::new();
    let mut h: SifrGeneratedStdlibSifrX2ehashlibX2eHashObject = sha256(&Vec::<u8>::new());
    h.update(&[97_u8]);
    h.update(&[98_u8, 99_u8]);
    actual.push(
        (h.hexdigest().as_str() == sha256(&[97_u8, 98_u8, 99_u8]).hexdigest().as_str()).to_string(),
    );
    actual.push((h.digest().len() == SifrInt::from_i64(32)).to_string());
    let mut c: SifrGeneratedStdlibSifrX2ehashlibX2eHashObject = copy_hash(&h);
    c.update(&[120_u8]);
    actual.push(
        (c.hexdigest().as_str() == sha256(&[97_u8, 98_u8, 99_u8, 120_u8]).hexdigest().as_str())
            .to_string(),
    );
    let m: SifrGeneratedStdlibSifrX2ehashlibX2eHashObject =
        md5(&[104_u8, 101_u8, 108_u8, 108_u8, 111_u8]);
    actual.push(
        (m.hexdigest().as_str()
            == md5(&[104_u8, 101_u8, 108_u8, 108_u8, 111_u8])
                .hexdigest()
                .as_str())
        .to_string(),
    );
    actual.push(contains(&algorithms_guaranteed(), "sha256").to_string());
    actual.push(m.hexdigest());
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2ehashlibX2eHashlibError> =
        (|| {
            let out: String = file_digest(tmp_path, "sha256")?;
            actual.push(out);
            Ok(())
        })();
    if let Err(_try_err) = sifr_generated_try_res {
        actual.push("ERR".to_string());
    }
    actual
}
fn collect_negative_actual_ok() -> Vec<bool> {
    let mut actual_ok: Vec<bool> = Vec::new();
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let bad: SifrGeneratedStdlibSifrX2ehashlibX2eHashObject =
            new("sha3_256", &Vec::<u8>::new())?;
        let _ = bad.name;
        actual_ok.push(true);
        Ok(())
    })();
    if let Err(_try_err) = sifr_generated_try_res {
        actual_ok.push(false);
    }
    actual_ok
}
fn main() {
    let expected: Vec<String> = vec![
        "true".to_string(),
        "true".to_string(),
        "true".to_string(),
        "true".to_string(),
        "true".to_string(),
        "5d41402abc4b2a76b9719d911017c592".to_string(),
        "8e6537b695ff181bc341e32d8b8970485ac3513408e5eb1e8ba9fc5af1cd3f57".to_string(),
    ];
    let tmp_path: String = "tmp_hashlib_hashlib_demo.txt".to_string();
    let _: Result<(), IOError> = write_text(&tmp_path, "file-data");
    let actual: Vec<String> = collect_positive_actual(&tmp_path);
    assert_vector_eq(&actual, &expected);
    let expected_ok: Vec<bool> = vec![false];
    let actual_ok: Vec<bool> = collect_negative_actual_ok();
    assert_bool_vector_eq(&actual_ok, &expected_ok);
    println!("hashlib hashlib parity demo: pass");
}
