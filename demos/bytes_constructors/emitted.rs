// src/main.rs
mod __sifr_project_nominals {
    pub use ::sifr_runtime::SifrInt;
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct IOError {
        pub message: String,
        pub kind: String,
    }
    impl IOError {
        pub fn new(message: String) -> Self {
            Self {
                message,
                kind: "Other".to_string(),
            }
        }
    }
    impl ::std::fmt::Display for IOError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for IOError {}
    pub fn __io_err<E: ::std::fmt::Display + 'static>(e: E) -> IOError {
        let msg = e.to_string();
        let kind = {
            let __sifr_io_kind = (&e as &dyn ::std::any::Any)
                .downcast_ref::<std::io::Error>()
                .map(::std::io::Error::kind);
            match __sifr_io_kind {
                Some(::std::io::ErrorKind::NotFound) => "FileNotFound".to_string(),
                Some(::std::io::ErrorKind::PermissionDenied) => {
                    "PermissionDenied".to_string()
                }
                Some(::std::io::ErrorKind::AlreadyExists) => "FileExists".to_string(),
                Some(::std::io::ErrorKind::IsADirectory) => "IsADirectory".to_string(),
                Some(::std::io::ErrorKind::NotADirectory) => "NotADirectory".to_string(),
                Some(::std::io::ErrorKind::DirectoryNotEmpty) => {
                    "DirectoryNotEmpty".to_string()
                }
                _ => "Other".to_string(),
            }
        };
        IOError { message: msg, kind }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ParseError {
        pub message: String,
    }
    impl ParseError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
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
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for ValueError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ValueError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct JSONDecodeError {
        pub message: String,
        pub line: SifrInt,
        pub column: SifrInt,
    }
    impl JSONDecodeError {
        pub fn new(message: String) -> Self {
            Self {
                message,
                line: SifrInt::from_i64(0),
                column: SifrInt::from_i64(0),
            }
        }
    }
    impl ::std::fmt::Display for JSONDecodeError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for JSONDecodeError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct JsonIntegerRangeError {
        pub message: String,
        pub path: String,
        pub profile: String,
    }
    impl JsonIntegerRangeError {
        pub fn new(message: String) -> Self {
            Self {
                message,
                path: String::new(),
                profile: String::new(),
            }
        }
    }
    impl ::std::fmt::Display for JsonIntegerRangeError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for JsonIntegerRangeError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct JsonLimitError {
        pub message: String,
        pub limit: SifrInt,
    }
    impl JsonLimitError {
        pub fn new(message: String) -> Self {
            Self {
                message,
                limit: SifrInt::from_i64(0),
            }
        }
    }
    impl ::std::fmt::Display for JsonLimitError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for JsonLimitError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct TOMLDecodeError {
        pub message: String,
        pub line: SifrInt,
        pub column: SifrInt,
    }
    impl TOMLDecodeError {
        pub fn new(message: String) -> Self {
            Self {
                message,
                line: SifrInt::from_i64(0),
                column: SifrInt::from_i64(0),
            }
        }
    }
    impl ::std::fmt::Display for TOMLDecodeError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for TOMLDecodeError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct RegexError {
        pub message: String,
        pub detail: String,
    }
    impl RegexError {
        pub fn new(message: String) -> Self {
            Self {
                message,
                detail: String::new(),
            }
        }
    }
    impl ::std::fmt::Display for RegexError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for RegexError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct TimeoutError {
        pub message: String,
    }
    impl TimeoutError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for TimeoutError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for TimeoutError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ScopeFailure {
        pub message: String,
    }
    impl ScopeFailure {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for ScopeFailure {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ScopeFailure {}
}
pub use __sifr_project_nominals::IOError;
pub use __sifr_project_nominals::JSONDecodeError;
pub use __sifr_project_nominals::JsonIntegerRangeError;
pub use __sifr_project_nominals::JsonLimitError;
pub use __sifr_project_nominals::ParseError;
pub use __sifr_project_nominals::RegexError;
pub use __sifr_project_nominals::ScopeFailure;
pub use __sifr_project_nominals::TOMLDecodeError;
pub use __sifr_project_nominals::TimeoutError;
pub use __sifr_project_nominals::ValueError;
use ::sifr_runtime::SifrInt;
fn __io_err<E: ::std::fmt::Display + 'static>(e: E) -> IOError {
    let msg = e.to_string();
    let kind = {
        let __sifr_io_kind = (&e as &dyn ::std::any::Any)
            .downcast_ref::<std::io::Error>()
            .map(::std::io::Error::kind);
        match __sifr_io_kind {
            Some(::std::io::ErrorKind::NotFound) => "FileNotFound".to_string(),
            Some(::std::io::ErrorKind::PermissionDenied) => {
                "PermissionDenied".to_string()
            }
            Some(::std::io::ErrorKind::AlreadyExists) => "FileExists".to_string(),
            Some(::std::io::ErrorKind::IsADirectory) => "IsADirectory".to_string(),
            Some(::std::io::ErrorKind::NotADirectory) => "NotADirectory".to_string(),
            Some(::std::io::ErrorKind::DirectoryNotEmpty) => {
                "DirectoryNotEmpty".to_string()
            }
            _ => "Other".to_string(),
        }
    };
    IOError { message: msg, kind }
}
fn main() {
    let mut size_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let zeros: Vec<u8> = ({
            let __size = SifrInt::from_i64(6);
            if &__size < &0 {
                return Err(ValueError {
                    message: "bytes(size) requires a non-negative size"
                        .to_string()
                        .to_string(),
                });
            }
            let __size = match __size.try_to_usize() {
                Ok(__size) => __size,
                Err(_) => {
                    return Err(ValueError {
                        message: "bytes(size) exceeds the addressable size"
                            .to_string()
                            .to_string(),
                    });
                }
            };
            Ok::<Vec<u8>, ValueError>((0..__size).map(|_| 0_u8).collect::<Vec<u8>>())
        })?;
        assert!(& SifrInt::from(zeros.len()) == & SifrInt::from_i64(6));
        size_ok = true;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(23usize +
            0usize); __sifr_concat.push_str("unexpected ValueError: "); __sifr_concat
            .push_str((e.message.clone()).as_str()); __sifr_concat }
        );
    }
    assert!(size_ok);
    let mut from_ints_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let from_list: Vec<u8> = ({
            let __vals = vec![
                SifrInt::from_i64(83), SifrInt::from_i64(105), SifrInt::from_i64(102),
                SifrInt::from_i64(114)
            ];
            let mut __out = Vec::new();
            for __pair in __vals.iter().enumerate() {
                if (*__pair.1 < 0) || (*__pair.1 > 255) {
                    return Err(ValueError {
                        message: format!(
                            "byte out of range at index {}: {}", __pair.0, * __pair.1
                        ),
                    });
                }
                __out.push(__pair.1.to_u8_proven_in_range());
            }
            Ok::<Vec<u8>, ValueError>(__out)
        })?;
        let first: Option<u8> = from_list
            .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(0))))
            .map(|__byte| *__byte as u8);
        let last: Option<u8> = from_list
            .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(3))))
            .map(|__byte| *__byte as u8);
        if let Some(first) = first {
            let expected_first: u8 = 83u8;
            assert!(first == expected_first);
        } else {
            assert!(false);
        }
        if let Some(last) = last {
            let expected_last: u8 = 114u8;
            assert!(last == expected_last);
        } else {
            assert!(false);
        }
        from_ints_ok = true;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(23usize +
            0usize); __sifr_concat.push_str("unexpected ValueError: "); __sifr_concat
            .push_str((e.message.clone()).as_str()); __sifr_concat }
        );
    }
    assert!(from_ints_ok);
    let mut from_hex_ok: bool = false;
    let __sifr_try_res: Result<(), ParseError> = (|| {
        let from_hex: Vec<u8> = ({
            let s: String = "53 69 66 72".to_string().to_string();
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
        let from_hex_text: String = ::sifr_runtime::encoding::decode_text(
                &from_hex,
                &"utf-8".to_string(),
                &"strict".to_string(),
            )
            .map_err(|__message| ParseError { message: __message })?;
        assert!((from_hex_text).as_str() == ("Sifr".to_string()).as_str());
        from_hex_ok = true;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(23usize +
            0usize); __sifr_concat.push_str("unexpected ParseError: "); __sifr_concat
            .push_str((e.message.clone()).as_str()); __sifr_concat }
        );
    }
    assert!(from_hex_ok);
    let mut encode_ok: bool = false;
    let __sifr_try_res: Result<(), ParseError> = (|| {
        let encoded: Vec<u8> = ::sifr_runtime::encoding::encode_bytes(
                &"bytes_constructors-demo".to_string(),
                &"utf-8".to_string(),
                &"strict".to_string(),
            )
            .map_err(|__message| ParseError { message: __message })?;
        let decoded: String = ::sifr_runtime::encoding::decode_text(
                &encoded,
                &"utf-8".to_string(),
                &"strict".to_string(),
            )
            .map_err(|__message| ParseError { message: __message })?;
        assert!((decoded).as_str() == ("bytes_constructors-demo".to_string()).as_str());
        encode_ok = true;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(23usize +
            0usize); __sifr_concat.push_str("unexpected ParseError: "); __sifr_concat
            .push_str((e.message.clone()).as_str()); __sifr_concat }
        );
    }
    assert!(encode_ok);
    println!("bytes_bytes_constructors_surface_demo: ok");
}
