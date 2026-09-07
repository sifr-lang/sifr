// src/main.rs
mod sifr_generated_generated_support {
    use crate::{
        FloatOverflowError, FloatPrecisionLossError, IOError, ParseError,
        SifrGeneratedIoBinaryFileHandle, SifrGeneratedIoNativeFileHandle,
        SifrGeneratedIoTextFileHandle, SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler,
        SifrGeneratedStdlibSifrX2eencodingX2eEncodeError,
        SifrGeneratedStdlibSifrX2eencodingX2eEncodeErrorHandler,
        SifrGeneratedStdlibSifrX2eencodingX2eEncodeOutcome,
        SifrGeneratedStdlibSifrX2eencodingX2eEncoding, SifrGeneratedStdlibSifrX2eloggingX2eLogger,
        SifrGeneratedStdlibSifrX2etimeX2estructTime,
        SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0,
        ValueError,
    };
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) fn sifr_generated_encoding_encode_bytes_impl(
        text: &str,
        encoding: &str,
        errors: &str,
    ) -> Result<Vec<u8>, ParseError> {
        ::sifr_stdlib::encoding::encoding_encode_bytes(text, encoding, errors).map_err(
            |sifr_generated_bridge_error| ParseError {
                message: sifr_generated_bridge_error.to_string(),
            },
        )
    }
    pub(crate) fn sifr_generated_encoding_encode_recoveries_impl(
        text: &str,
        encoding: &str,
        errors: &str,
    ) -> Result<Vec<String>, ParseError> {
        ::sifr_stdlib::encoding::encoding_encode_recoveries(text, encoding, errors).map_err(
            |sifr_generated_bridge_error| ParseError {
                message: sifr_generated_bridge_error.to_string(),
            },
        )
    }
    pub(crate) fn read_text(path: &str) -> Result<String, IOError> {
        ::sifr_stdlib::fs::read_text(path).map_err(sifr_generated_io_err)
    }
    pub(crate) fn write_text(path: &str, content: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::write_text(path, content).map_err(sifr_generated_io_err)
    }
    pub(crate) fn exists(path: &str) -> bool {
        ::sifr_stdlib::fs::exists(path)
    }
    pub(crate) fn sifr_generated_open_file(path: &str, mode: &str) -> Result<String, IOError> {
        ::sifr_stdlib::fs::open_file(path, mode).map_err(sifr_generated_io_err)
    }
    pub(crate) fn sifr_generated_file_close(handle: &str) {
        ::sifr_stdlib::fs::file_close(handle);
    }
    pub(crate) fn sifr_generated_file_write_bytes(
        handle: &str,
        data: &[u8],
    ) -> Result<(), IOError> {
        ::sifr_stdlib::fs::file_write_bytes(handle, data).map_err(sifr_generated_io_err)
    }
    pub(crate) fn open_file(
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
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err.clone();
            Err(e)
        })
    }
    pub(crate) fn file_close(handle: &SifrGeneratedIoNativeFileHandle) {
        sifr_generated_file_close(&handle.id.clone());
    }
    pub(crate) fn file_write_bytes(
        handle: &SifrGeneratedIoNativeFileHandle,
        data: &[u8],
    ) -> Result<(), IOError> {
        sifr_generated_file_write_bytes(&handle.id.clone(), data)
    }
    pub(crate) fn remove_file(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::remove_file(path).map_err(sifr_generated_io_err)
    }
    pub(crate) fn sifr_generated_const_454e434f44494e475f55544638() -> String {
        "utf-8".to_string()
    }
    pub(crate) fn sifr_generated_const_4445434f44455f4552524f52535f535452494354() -> String {
        "strict".to_string()
    }
    pub(crate) fn sifr_generated_const_454e434f44455f4552524f52535f535452494354() -> String {
        "strict".to_string()
    }
    pub(crate) fn sifr_generated_encoding_encode_outcome(
        text: &str,
        encoding: &str,
        errors: &str,
    ) -> Result<
        SifrGeneratedStdlibSifrX2eencodingX2eEncodeOutcome,
        SifrGeneratedStdlibSifrX2eencodingX2eEncodeError,
    > {
        let sifr_generated_try_res: Result<
            Result<
                SifrGeneratedStdlibSifrX2eencodingX2eEncodeOutcome,
                SifrGeneratedStdlibSifrX2eencodingX2eEncodeError,
            >,
            ParseError,
        > = (|| {
            let data: Vec<u8> = sifr_generated_encoding_encode_bytes_impl(text, encoding, errors)?;
            let recoveries: Vec<String> =
                sifr_generated_encoding_encode_recoveries_impl(text, encoding, errors)?;
            Ok(Ok(SifrGeneratedStdlibSifrX2eencodingX2eEncodeOutcome::new(
                data, recoveries,
            )))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err.clone();
            Err(SifrGeneratedStdlibSifrX2eencodingX2eEncodeError::new(
                e.message.clone(),
            ))
        })
    }
    pub(crate) fn utf8() -> SifrGeneratedStdlibSifrX2eencodingX2eEncoding {
        SifrGeneratedStdlibSifrX2eencodingX2eEncoding::new(
            sifr_generated_const_454e434f44494e475f55544638(),
        )
    }
    pub(crate) fn strict_decode_handler() -> SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler
    {
        SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler::new(
            sifr_generated_const_4445434f44455f4552524f52535f535452494354(),
        )
    }
    pub(crate) fn sifr_generated_encode_handler_name(
        errors: &Option<SifrGeneratedStdlibSifrX2eencodingX2eEncodeErrorHandler>,
    ) -> String {
        let Some(errors) = errors.as_ref() else {
            return sifr_generated_const_454e434f44455f4552524f52535f535452494354();
        };
        {
            let mut sifr_generated_concat: String = String::new();
            sifr_generated_concat.push_str(errors.name.clone().as_str());
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        }
    }
    pub(crate) fn encode_outcome(
        text: &str,
        enc: &SifrGeneratedStdlibSifrX2eencodingX2eEncoding,
        errors: &Option<SifrGeneratedStdlibSifrX2eencodingX2eEncodeErrorHandler>,
    ) -> Result<
        SifrGeneratedStdlibSifrX2eencodingX2eEncodeOutcome,
        SifrGeneratedStdlibSifrX2eencodingX2eEncodeError,
    > {
        let handler_name: String = sifr_generated_encode_handler_name(errors);
        sifr_generated_encoding_encode_outcome(text, &enc.label.clone(), &handler_name)
    }
    pub(crate) fn encode(
        text: &str,
        enc: &SifrGeneratedStdlibSifrX2eencodingX2eEncoding,
        errors: &Option<SifrGeneratedStdlibSifrX2eencodingX2eEncodeErrorHandler>,
    ) -> Result<Vec<u8>, SifrGeneratedStdlibSifrX2eencodingX2eEncodeError> {
        let sifr_generated_try_res: Result<
            Result<Vec<u8>, SifrGeneratedStdlibSifrX2eencodingX2eEncodeError>,
            SifrGeneratedStdlibSifrX2eencodingX2eEncodeError,
        > = (|| {
            let outcome: SifrGeneratedStdlibSifrX2eencodingX2eEncodeOutcome =
                encode_outcome(text, enc, errors)?;
            Ok(Ok(outcome.get_data()))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err.clone();
            Err(SifrGeneratedStdlibSifrX2eencodingX2eEncodeError::new(
                e.message.clone(),
            ))
        })
    }
    #[derive(Debug, Clone)]
    pub(crate) enum SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a019X3a5X3aclass7X3aIOError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass7X3aIOError1X3a0(IOError),
        SifrGeneratedUnionVariant5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a0(
            SifrGeneratedStdlibSifrX2eencodingX2eEncodeError,
        ),
    }
    impl From<IOError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a019X3a5X3aclass7X3aIOError1X3a0 {
        fn from(value: IOError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass7X3aIOError1X3a0(
                value,
            )
        }
    }
    impl From<SifrGeneratedStdlibSifrX2eencodingX2eEncodeError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a019X3a5X3aclass7X3aIOError1X3a0 {
        fn from(value: SifrGeneratedStdlibSifrX2eencodingX2eEncodeError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a019X3a5X3aclass7X3aIOError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass7X3aIOError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
    pub(crate) fn sifr_generated_closed_stream_error() -> String {
        "I/O operation on closed stream".to_string()
    }
    pub(crate) fn sifr_generated_mode_is_writable(mode: &str) -> bool {
        mode.contains(&"w".to_string())
            || mode.contains(&"a".to_string())
            || mode.contains(&"+".to_string())
    }
    pub(crate) fn sifr_generated_text_binary_mode(mode: &str) -> Result<String, IOError> {
        if mode.contains(&"b".to_string()) {
            return Err(IOError::new(
                "open_text requires a text mode without \'b\'".to_string(),
            ));
        }
        if mode == "r" || mode == "rt" {
            return Ok("rb".to_string());
        }
        if mode == "w" || mode == "wt" {
            return Ok("wb".to_string());
        }
        if mode == "a" || mode == "at" {
            return Ok("ab".to_string());
        }
        Err(IOError::new({
            let mut sifr_generated_concat: String = String::with_capacity(19usize + mode.len());
            sifr_generated_concat.push_str("invalid text mode: ");
            sifr_generated_concat.push_str(mode);
            sifr_generated_concat
        }))
    }
    pub(crate) fn sifr_generated_text_encoding_or_default(
        enc: &Option<SifrGeneratedStdlibSifrX2eencodingX2eEncoding>,
    ) -> SifrGeneratedStdlibSifrX2eencodingX2eEncoding {
        let Some(enc) = enc.as_ref() else {
            return SifrGeneratedStdlibSifrX2eencodingX2eEncoding::new("utf-8".to_string());
        };
        SifrGeneratedStdlibSifrX2eencodingX2eEncoding::new(enc.label.clone().to_string())
    }
    pub(crate) fn sifr_generated_decode_errors_or_default(
        errors: &Option<SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler>,
    ) -> SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler {
        let Some(errors) = errors.as_ref() else {
            return strict_decode_handler();
        };
        SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler::new(
            errors.name.clone().to_string(),
        )
    }
    pub(crate) fn sifr_generated_encode_errors_from_decode_errors(
        errors: &SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler,
    ) -> SifrGeneratedStdlibSifrX2eencodingX2eEncodeErrorHandler {
        SifrGeneratedStdlibSifrX2eencodingX2eEncodeErrorHandler::new(
            errors.name.clone().to_string(),
        )
    }
    pub(crate) fn open_binary(
        path: &str,
        mode: &str,
    ) -> Result<SifrGeneratedIoBinaryFileHandle, IOError> {
        if !mode.contains(&"b".to_string()) {
            return Err(IOError::new("open_binary requires binary mode".to_string()));
        }
        let sifr_generated_try_res: Result<
            Result<SifrGeneratedIoBinaryFileHandle, IOError>,
            IOError,
        > = (|| {
            let handle: SifrGeneratedIoNativeFileHandle = open_file(path, mode)?;
            Ok(Ok(SifrGeneratedIoBinaryFileHandle::new(
                handle,
                mode.to_owned(),
            )))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err.clone();
            Err(e)
        })
    }
    pub(crate) fn open_text(
        path: &str,
        mode: &str,
        encoding: &Option<SifrGeneratedStdlibSifrX2eencodingX2eEncoding>,
        errors: &Option<SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler>,
    ) -> Result<SifrGeneratedIoTextFileHandle, IOError> {
        let sifr_generated_try_res: Result<
            Result<SifrGeneratedIoTextFileHandle, IOError>,
            IOError,
        > = (|| {
            let binary_mode: String = sifr_generated_text_binary_mode(mode)?;
            let text_encoding: SifrGeneratedStdlibSifrX2eencodingX2eEncoding =
                sifr_generated_text_encoding_or_default(encoding);
            let decode_errors: SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler =
                sifr_generated_decode_errors_or_default(errors);
            let encode_errors: SifrGeneratedStdlibSifrX2eencodingX2eEncodeErrorHandler =
                sifr_generated_encode_errors_from_decode_errors(&decode_errors);
            let binary: SifrGeneratedIoBinaryFileHandle = open_binary(path, &binary_mode)?;
            Ok(Ok(SifrGeneratedIoTextFileHandle::new(
                binary,
                text_encoding,
                decode_errors,
                encode_errors,
            )))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err.clone();
            Err(e)
        })
    }
    pub(crate) fn get_global_level() -> SifrInt {
        ::sifr_stdlib::logging::get_global_level().into_sifr_int()
    }
    pub(crate) const fn sifr_generated_const_494e464f() -> SifrInt {
        SifrInt::from_i64(20)
    }
    pub(crate) const fn sifr_generated_const_4e4f54534554() -> SifrInt {
        SifrInt::from_i64(0)
    }
    #[expect(
        non_snake_case,
        reason = "generated Rust preserves this exact typed Sifr source contract"
    )]
    pub(crate) fn getLogger(name: &str) -> SifrGeneratedStdlibSifrX2eloggingX2eLogger {
        let level: SifrInt = get_global_level();
        SifrGeneratedStdlibSifrX2eloggingX2eLogger::new(name.to_owned(), level.clone())
    }
    pub(crate) fn perf_counter() -> f64 {
        ::sifr_stdlib::time::perf_counter()
    }
    pub(crate) fn sifr_generated_gmtime_intrinsic(epoch: f64) -> String {
        ::sifr_stdlib::time::gmtime(epoch)
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub(crate) enum SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(FloatOverflowError),
        SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
            FloatPrecisionLossError,
        ),
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
    pub(crate) const fn sifr_generated_const_54494d455a4f4e45() -> SifrInt {
        SifrInt::from_i64(0)
    }
    pub(crate) fn sifr_generated_const_545a4e414d45() -> (String, String) {
        ("UTC".to_string(), "UTC".to_string())
    }
    pub(crate) fn sifr_generated_is_leap_year(year: SifrInt) -> bool {
        &year.floor_mod_known_nonzero(&SifrInt::from_i64(4)) == &SifrInt::from_i64(0)
            && &year.floor_mod_known_nonzero(&SifrInt::from_i64(100)) != &SifrInt::from_i64(0)
            || &year.floor_mod_known_nonzero(&SifrInt::from_i64(400)) == &SifrInt::from_i64(0)
    }
    pub(crate) fn sifr_generated_days_in_year(year: SifrInt) -> SifrInt {
        if sifr_generated_is_leap_year(year.clone()) {
            return SifrInt::from_i64(366);
        }
        SifrInt::from_i64(365)
    }
    pub(crate) fn sifr_generated_days_in_month(year: SifrInt, month: SifrInt) -> SifrInt {
        let month_days: Vec<SifrInt> = vec![
            SifrInt::from_i64(31),
            SifrInt::from_i64(28),
            SifrInt::from_i64(31),
            SifrInt::from_i64(30),
            SifrInt::from_i64(31),
            SifrInt::from_i64(30),
            SifrInt::from_i64(31),
            SifrInt::from_i64(31),
            SifrInt::from_i64(30),
            SifrInt::from_i64(31),
            SifrInt::from_i64(30),
            SifrInt::from_i64(31),
        ];
        let idx: SifrInt = &month - &SifrInt::from_i64(1);
        let d: Option<SifrInt> = {
            let sifr_generated_checked_read_collection = &month_days;
            let sifr_generated_checked_read_index = idx.clone();
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        };
        if &month == &SifrInt::from_i64(2) && sifr_generated_is_leap_year(year.clone()) {
            return SifrInt::from_i64(29);
        }
        let Some(d) = d.clone() else {
            return SifrInt::from_i64(0);
        };
        d
    }
    pub(crate) fn sifr_generated_substring(value: &str, start: SifrInt, end: SifrInt) -> String {
        let sifr_generated_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
        let mut result: String = String::new();
        let mut i: SifrInt = start.clone();
        while &i < &end {
            let ch: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_value.len());
                sifr_generated_chars_value
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(ch) = ch {
                result.push_str(ch.as_str());
            }
            i = &i + &SifrInt::from_i64(1);
        }
        result
    }
    pub(crate) fn sifr_generated_digit_value(ch: &str) -> Option<SifrInt> {
        if ch == "0" {
            return Some(SifrInt::from_i64(0));
        }
        if ch == "1" {
            return Some(SifrInt::from_i64(1));
        }
        if ch == "2" {
            return Some(SifrInt::from_i64(2));
        }
        if ch == "3" {
            return Some(SifrInt::from_i64(3));
        }
        if ch == "4" {
            return Some(SifrInt::from_i64(4));
        }
        if ch == "5" {
            return Some(SifrInt::from_i64(5));
        }
        if ch == "6" {
            return Some(SifrInt::from_i64(6));
        }
        if ch == "7" {
            return Some(SifrInt::from_i64(7));
        }
        if ch == "8" {
            return Some(SifrInt::from_i64(8));
        }
        if ch == "9" {
            return Some(SifrInt::from_i64(9));
        }
        None
    }
    pub(crate) fn sifr_generated_parse_decimal(text: &str) -> Option<SifrInt> {
        let sifr_generated_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
        if &SifrInt::from(sifr_generated_chars_text.len()) == &SifrInt::from_i64(0) {
            return None;
        }
        let mut out: SifrInt = SifrInt::from_i64(0);
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &SifrInt::from(sifr_generated_chars_text.len()) {
            let ch_opt: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_text.len());
                sifr_generated_chars_text
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            let ch_opt_value_58c5362056f71db8 = ch_opt?;
            let ch: String = ch_opt_value_58c5362056f71db8;
            let digit_opt: Option<SifrInt> = sifr_generated_digit_value(&ch);
            let digit_opt_value_c39685cb2782ed00 = digit_opt.clone()?;
            let digit: SifrInt = digit_opt_value_c39685cb2782ed00.clone();
            out = &(&out * &SifrInt::from_i64(10)) + &digit;
            i = &i + &SifrInt::from_i64(1);
        }
        Some(out)
    }
    pub(crate) fn sifr_generated_int_or_negative_one(value: Option<SifrInt>) -> SifrInt {
        let Some(value) = value.clone() else {
            return -&SifrInt::from_i64(1);
        };
        value.clone()
    }
    pub(crate) fn sifr_generated_day_of_year(
        year: SifrInt,
        month: SifrInt,
        day: SifrInt,
    ) -> SifrInt {
        let mut yday: SifrInt = SifrInt::from_i64(0);
        let mut m: SifrInt = SifrInt::from_i64(1);
        while &m < &month {
            yday = &yday + &sifr_generated_days_in_month(year.clone(), m.clone());
            m = &m + &SifrInt::from_i64(1);
        }
        &yday + &day
    }
    pub(crate) fn sifr_generated_weekday(year: SifrInt, month: SifrInt, day: SifrInt) -> SifrInt {
        let mut days_since_epoch: SifrInt = SifrInt::from_i64(0);
        if &year >= &SifrInt::from_i64(1970) {
            let mut y: SifrInt = SifrInt::from_i64(1970);
            while &y < &year {
                days_since_epoch = &days_since_epoch + &sifr_generated_days_in_year(y.clone());
                y = &y + &SifrInt::from_i64(1);
            }
        } else {
            let mut y: SifrInt = SifrInt::from_i64(1969);
            while &y >= &year {
                days_since_epoch = &days_since_epoch - &sifr_generated_days_in_year(y.clone());
                y = &y - &SifrInt::from_i64(1);
            }
        }
        let mut m: SifrInt = SifrInt::from_i64(1);
        while &m < &month {
            days_since_epoch =
                &days_since_epoch + &sifr_generated_days_in_month(year.clone(), m.clone());
            m = &m + &SifrInt::from_i64(1);
        }
        days_since_epoch = &(&days_since_epoch + &day) - &SifrInt::from_i64(1);
        let mut wd: SifrInt = (&SifrInt::from_i64(3) + &days_since_epoch)
            .floor_mod_known_nonzero(&SifrInt::from_i64(7));
        if &wd < &SifrInt::from_i64(0) {
            wd = &wd + &SifrInt::from_i64(7);
        }
        wd.clone()
    }
    pub(crate) fn sifr_generated_valid_date(year: SifrInt, month: SifrInt, day: SifrInt) -> bool {
        if &year <= &SifrInt::from_i64(0) {
            return false;
        }
        if &month < &SifrInt::from_i64(1) || &month > &SifrInt::from_i64(12) {
            return false;
        }
        let max_day: SifrInt = sifr_generated_days_in_month(year.clone(), month.clone());
        &day >= &SifrInt::from_i64(1) && &day <= &max_day
    }
    pub(crate) fn sifr_generated_invalid_struct_time() -> SifrGeneratedStdlibSifrX2etimeX2estructTime
    {
        SifrGeneratedStdlibSifrX2etimeX2estructTime::new(
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
        )
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub(crate) fn sifr_generated_to_struct_time(
        rendered: &str,
    ) -> SifrGeneratedStdlibSifrX2etimeX2estructTime {
        let sifr_generated_chars_rendered: Vec<char> = rendered.chars().collect::<Vec<char>>();
        let Some(_checked_value_3) = {
            let sifr_generated_string_index = SifrInt::from_i64(4);
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_rendered.len());
            sifr_generated_chars_rendered
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string()) else {
            return sifr_generated_invalid_struct_time();
        };
        let Some(_checked_value_4) = {
            let sifr_generated_string_index = SifrInt::from_i64(7);
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_rendered.len());
            sifr_generated_chars_rendered
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string()) else {
            return sifr_generated_invalid_struct_time();
        };
        let Some(_checked_value_5) = {
            let sifr_generated_string_index = SifrInt::from_i64(10);
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_rendered.len());
            sifr_generated_chars_rendered
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string()) else {
            return sifr_generated_invalid_struct_time();
        };
        let Some(_checked_value_6) = {
            let sifr_generated_string_index = SifrInt::from_i64(13);
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_rendered.len());
            sifr_generated_chars_rendered
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string()) else {
            return sifr_generated_invalid_struct_time();
        };
        let Some(_checked_value_7) = {
            let sifr_generated_string_index = SifrInt::from_i64(16);
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_rendered.len());
            sifr_generated_chars_rendered
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string()) else {
            return sifr_generated_invalid_struct_time();
        };
        if {
            let sifr_generated_string_index = SifrInt::from_i64(4);
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_rendered.len());
            sifr_generated_chars_rendered
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(Some)
            != Some(Some('-'))
            || {
                let sifr_generated_string_index = SifrInt::from_i64(7);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_rendered.len());
                sifr_generated_chars_rendered
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(Some)
                != Some(Some('-'))
            || {
                let sifr_generated_string_index = SifrInt::from_i64(10);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_rendered.len());
                sifr_generated_chars_rendered
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(Some)
                != Some(Some('T'))
            || {
                let sifr_generated_string_index = SifrInt::from_i64(13);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_rendered.len());
                sifr_generated_chars_rendered
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(Some)
                != Some(Some(':'))
            || {
                let sifr_generated_string_index = SifrInt::from_i64(16);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_rendered.len());
                sifr_generated_chars_rendered
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(Some)
                != Some(Some(':'))
        {
            return sifr_generated_invalid_struct_time();
        }
        let year: SifrInt = sifr_generated_int_or_negative_one(sifr_generated_parse_decimal(
            &sifr_generated_substring(rendered, SifrInt::from_i64(0), SifrInt::from_i64(4)),
        ));
        let month: SifrInt = sifr_generated_int_or_negative_one(sifr_generated_parse_decimal(
            &sifr_generated_substring(rendered, SifrInt::from_i64(5), SifrInt::from_i64(7)),
        ));
        let day: SifrInt = sifr_generated_int_or_negative_one(sifr_generated_parse_decimal(
            &sifr_generated_substring(rendered, SifrInt::from_i64(8), SifrInt::from_i64(10)),
        ));
        let hour: SifrInt = sifr_generated_int_or_negative_one(sifr_generated_parse_decimal(
            &sifr_generated_substring(rendered, SifrInt::from_i64(11), SifrInt::from_i64(13)),
        ));
        let minute: SifrInt = sifr_generated_int_or_negative_one(sifr_generated_parse_decimal(
            &sifr_generated_substring(rendered, SifrInt::from_i64(14), SifrInt::from_i64(16)),
        ));
        let second: SifrInt = sifr_generated_int_or_negative_one(sifr_generated_parse_decimal(
            &sifr_generated_substring(rendered, SifrInt::from_i64(17), SifrInt::from_i64(19)),
        ));
        if &year < &SifrInt::from_i64(0)
            || &month < &SifrInt::from_i64(0)
            || &day < &SifrInt::from_i64(0)
            || &hour < &SifrInt::from_i64(0)
            || &minute < &SifrInt::from_i64(0)
            || &second < &SifrInt::from_i64(0)
        {
            return sifr_generated_invalid_struct_time();
        }
        if !sifr_generated_valid_date(year.clone(), month.clone(), day.clone()) {
            return sifr_generated_invalid_struct_time();
        }
        let wday: SifrInt = sifr_generated_weekday(year.clone(), month.clone(), day.clone());
        let yday_value_75753d4973d2a3ce: SifrInt =
            sifr_generated_day_of_year(year.clone(), month.clone(), day.clone());
        SifrGeneratedStdlibSifrX2etimeX2estructTime::new(
            year.clone(),
            month.clone(),
            day.clone(),
            hour.clone(),
            minute.clone(),
            second.clone(),
            wday.clone(),
            yday_value_75753d4973d2a3ce.clone(),
            SifrInt::from_i64(0),
        )
    }
    pub(crate) fn gmtime_struct(epoch: f64) -> SifrGeneratedStdlibSifrX2etimeX2estructTime {
        let rendered: String = sifr_generated_gmtime_intrinsic(epoch);
        sifr_generated_to_struct_time(&rendered)
    }
    pub(crate) fn mktime(
        t: &SifrGeneratedStdlibSifrX2etimeX2estructTime,
    ) -> Result<
        f64,
        SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0,
    >{
        if !sifr_generated_valid_date(t.tm_year.clone(), t.tm_mon.clone(), t.tm_mday.clone()) {
            return Err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
                    ValueError::new(
                        "mktime() received an invalid calendar date".to_string(),
                    ),
                ),
            );
        }
        let mut days: SifrInt = SifrInt::from_i64(0);
        if &t.tm_year.clone() >= &SifrInt::from_i64(1970) {
            let mut y: SifrInt = SifrInt::from_i64(1970);
            while &y < &t.tm_year.clone() {
                days = &days + &sifr_generated_days_in_year(y.clone());
                y = &y + &SifrInt::from_i64(1);
            }
        } else {
            let mut y: SifrInt = SifrInt::from_i64(1969);
            while &y >= &t.tm_year.clone() {
                days = &days - &sifr_generated_days_in_year(y.clone());
                y = &y - &SifrInt::from_i64(1);
            }
        }
        let mut m: SifrInt = SifrInt::from_i64(1);
        while &m < &t.tm_mon.clone() {
            days = &days + &sifr_generated_days_in_month(t.tm_year.clone(), m.clone());
            m = &m + &SifrInt::from_i64(1);
        }
        days = &(&days + &t.tm_mday.clone()) - &SifrInt::from_i64(1);
        let stamp: SifrInt = &(&(&(&days * &SifrInt::from_i64(86400))
            + &(&t.tm_hour.clone() * &SifrInt::from_i64(3600)))
            + &(&t.tm_min.clone() * &SifrInt::from_i64(60)))
            + &t.tm_sec.clone();
        stamp
            .clone()
            .checked_to_f64()
            .map_err(|sifr_generated_float_error| match sifr_generated_float_error {
                ::sifr_runtime::IntegerFloatConversionError::Overflow => {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                        FloatOverflowError::new(
                            "exact integer is outside the finite float range".to_string(),
                        ),
                    )
                }
                ::sifr_runtime::IntegerFloatConversionError::PrecisionLoss => {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                        FloatPrecisionLossError::new(
                            "exact integer cannot be represented without float precision loss"
                                .to_string(),
                        ),
                    )
                }
            })
            .map_err(|sifr_generated_error_value| match sifr_generated_error_value {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                    sifr_generated_union_value,
                ) => {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                        sifr_generated_union_value,
                    )
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                    sifr_generated_union_value,
                ) => {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                        sifr_generated_union_value,
                    )
                }
            })
    }
    pub(crate) fn sifr_generated_elapsed_non_negative(start: f64, end: f64) -> f64 {
        let elapsed: f64 = end - start;
        if elapsed < 0.0_f64 {
            return 0.0_f64;
        }
        elapsed
    }
    pub(crate) fn timeit(stmt: impl Fn(), number: SifrInt) -> f64 {
        let start: f64 = perf_counter();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &number {
            stmt();
            i = &i + &SifrInt::from_i64(1);
        }
        let end: f64 = perf_counter();
        sifr_generated_elapsed_non_negative(start, end)
    }
    pub(crate) fn sifr_generated_io_err<E: ::std::fmt::Display + 'static>(e: E) -> IOError {
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
    use crate::SifrGeneratedIoNativeFileHandle;
    use crate::sifr_generated_generated_support::*;
    use ::sifr_runtime::SifrInt;
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2eencodingX2eEncodeError {
        pub message: String,
    }
    impl SifrGeneratedStdlibSifrX2eencodingX2eEncodeError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Debug for SifrGeneratedStdlibSifrX2eencodingX2eEncodeError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            f.debug_struct("EncodeError")
                .field("message", &self.message)
                .finish()
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2eencodingX2eEncodeError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }
    impl ::std::error::Error for SifrGeneratedStdlibSifrX2eencodingX2eEncodeError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2eencodingX2eEncoding {
        pub label: String,
    }
    impl SifrGeneratedStdlibSifrX2eencodingX2eEncoding {
        #[must_use]
        pub fn new(label: String) -> Self {
            let sifr_generated_field_value_39f7fcec8fcb623d_6c6162656c: String = {
                let mut sifr_generated_concat: String = String::with_capacity(label.len());
                sifr_generated_concat.push_str(label.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            Self {
                label: sifr_generated_field_value_39f7fcec8fcb623d_6c6162656c,
            }
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2eencodingX2eEncoding {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "Encoding(label={})", self.label)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler {
        pub name: String,
    }
    impl SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler {
        #[must_use]
        pub fn new(name: String) -> Self {
            let sifr_generated_field_value_c4bcadba8e631b86_6e616d65: String = {
                let mut sifr_generated_concat: String = String::with_capacity(name.len());
                sifr_generated_concat.push_str(name.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            Self {
                name: sifr_generated_field_value_c4bcadba8e631b86_6e616d65,
            }
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "DecodeErrorHandler(name={})", self.name)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2eencodingX2eEncodeErrorHandler {
        pub name: String,
    }
    impl SifrGeneratedStdlibSifrX2eencodingX2eEncodeErrorHandler {
        #[must_use]
        pub fn new(name: String) -> Self {
            let sifr_generated_field_value_c4bcadba8e631b86_6e616d65: String = {
                let mut sifr_generated_concat: String = String::with_capacity(name.len());
                sifr_generated_concat.push_str(name.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            Self {
                name: sifr_generated_field_value_c4bcadba8e631b86_6e616d65,
            }
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2eencodingX2eEncodeErrorHandler {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "EncodeErrorHandler(name={})", self.name)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SifrGeneratedStdlibSifrX2eencodingX2eEncodeOutcome {
        pub data: Vec<u8>,
        pub recoveries: Vec<String>,
    }
    impl SifrGeneratedStdlibSifrX2eencodingX2eEncodeOutcome {
        #[must_use]
        pub const fn new(data: Vec<u8>, recoveries: Vec<String>) -> Self {
            let sifr_generated_field_value_855b556730a34a05_64617461: Vec<u8> = data;
            let sifr_generated_field_value_eb53194d835eec2e_7265636f766572696573: Vec<String> =
                recoveries;
            Self {
                data: sifr_generated_field_value_855b556730a34a05_64617461,
                recoveries: sifr_generated_field_value_eb53194d835eec2e_7265636f766572696573,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2eencodingX2eEncodeOutcome {
        #[must_use]
        pub fn get_data(&self) -> Vec<u8> {
            self.data.clone()
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedIoBinaryFileHandle {
        pub handle: SifrGeneratedIoNativeFileHandle,
        pub mode: String,
        pub closed: bool,
    }
    impl SifrGeneratedIoBinaryFileHandle {
        #[must_use]
        pub const fn new(handle: SifrGeneratedIoNativeFileHandle, mode: String) -> Self {
            let sifr_generated_field_value_b31dc5f344797918_5f68616e646c65: SifrGeneratedIoNativeFileHandle = handle;
            let sifr_generated_field_value_e0efc38c5ec2afd5_5f6d6f6465: String = mode;
            let sifr_generated_field_value_8bc7f577e5ffacda_5f636c6f736564: bool = false;
            Self {
                handle: sifr_generated_field_value_b31dc5f344797918_5f68616e646c65,
                mode: sifr_generated_field_value_e0efc38c5ec2afd5_5f6d6f6465,
                closed: sifr_generated_field_value_8bc7f577e5ffacda_5f636c6f736564,
            }
        }
    }
    impl SifrGeneratedIoBinaryFileHandle {
        pub fn close(&mut self) {
            if self.closed {
                return;
            }
            file_close(&self.handle);
            self.closed = true;
        }
    }
    impl SifrGeneratedIoBinaryFileHandle {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn write_bytes(&self, data: &[u8]) -> Result<(), IOError> {
            if self.closed {
                return Err(IOError::new(sifr_generated_closed_stream_error()));
            }
            if !self.writable() {
                return Err(IOError::new("stream is not writable".to_string()));
            }
            file_write_bytes(&self.handle, data)
        }
    }
    impl SifrGeneratedIoBinaryFileHandle {
        #[must_use]
        pub fn writable(&self) -> bool {
            sifr_generated_mode_is_writable(&self.mode)
        }
    }
    impl ::std::fmt::Display for SifrGeneratedIoBinaryFileHandle {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f,
                "BinaryFileHandle(_handle={:?}, _mode={}, _closed={})",
                self.handle, self.mode, self.closed
            )
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedIoTextFileHandle {
        pub binary: SifrGeneratedIoBinaryFileHandle,
        pub encoding: SifrGeneratedStdlibSifrX2eencodingX2eEncoding,
        pub decode_errors: SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler,
        pub encode_errors: SifrGeneratedStdlibSifrX2eencodingX2eEncodeErrorHandler,
    }
    impl SifrGeneratedIoTextFileHandle {
        #[must_use]
        pub const fn new(
            binary: SifrGeneratedIoBinaryFileHandle,
            enc: SifrGeneratedStdlibSifrX2eencodingX2eEncoding,
            decode_errors: SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler,
            encode_errors: SifrGeneratedStdlibSifrX2eencodingX2eEncodeErrorHandler,
        ) -> Self {
            let sifr_generated_field_value_8697ce5b827f5df7_5f62696e617279: SifrGeneratedIoBinaryFileHandle = binary;
            let sifr_generated_field_value_d67f71b9ba409c5f_5f656e636f64696e67: SifrGeneratedStdlibSifrX2eencodingX2eEncoding = enc;
            let sifr_generated_field_value_51b881e9a05bdf16_5f6465636f64655f6572726f7273: SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler = decode_errors;
            let sifr_generated_field_value_a66abc614f69ca5a_5f656e636f64655f6572726f7273: SifrGeneratedStdlibSifrX2eencodingX2eEncodeErrorHandler = encode_errors;
            Self {
                binary: sifr_generated_field_value_8697ce5b827f5df7_5f62696e617279,
                encoding: sifr_generated_field_value_d67f71b9ba409c5f_5f656e636f64696e67,
                decode_errors:
                    sifr_generated_field_value_51b881e9a05bdf16_5f6465636f64655f6572726f7273,
                encode_errors:
                    sifr_generated_field_value_a66abc614f69ca5a_5f656e636f64655f6572726f7273,
            }
        }
    }
    impl SifrGeneratedIoTextFileHandle {
        pub fn close(&mut self) {
            self.binary.close();
        }
    }
    impl SifrGeneratedIoTextFileHandle {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn write(&self, text: &str) -> Result<(), IOError> {
            let sifr_generated_try_res: Result<
                Result<(), IOError>,
                SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a019X3a5X3aclass7X3aIOError1X3a0,
            > = (|| {
                let data: Vec<u8> = encode(
                        text,
                        &self.encoding,
                        &Some(self.encode_errors.clone()),
                    )
                    .map_err(
                        SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a0,
                    )?;
                self.binary
                    .write_bytes(&data)
                    .map_err(
                        SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass7X3aIOError1X3a0,
                    )?;
                Ok(Ok(()))
            })();
            sifr_generated_try_res
                .unwrap_or_else(|sifr_generated_try_err| match sifr_generated_try_err {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass7X3aIOError1X3a0(
                        sifr_generated_try_variant_error,
                    ) => {
                        let e = sifr_generated_try_variant_error.clone();
                        Err(e)
                    }
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a0(
                        sifr_generated_try_variant_error,
                    ) => {
                        let e = sifr_generated_try_variant_error.clone();
                        Err(
                            IOError::new({
                                let mut sifr_generated_concat: String = String::with_capacity(
                                    20usize,
                                );
                                sifr_generated_concat.push_str("text encode failed: ");
                                sifr_generated_concat.push_str(e.message.clone().as_str());
                                sifr_generated_concat
                            }),
                        )
                    }
                })
        }
    }
    impl ::std::fmt::Display for SifrGeneratedIoTextFileHandle {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f,
                "TextFileHandle(_binary={}, _encoding={:?}, _decode_errors={:?}, _encode_errors={:?})",
                self.binary, self.encoding, self.decode_errors, self.encode_errors
            )
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2eloggingX2eFormatter {
        pub fmt: String,
    }
    impl SifrGeneratedStdlibSifrX2eloggingX2eFormatter {
        #[must_use]
        pub const fn new(fmt: String) -> Self {
            let sifr_generated_field_value_20e80d43821854a1_5f666d74: String = fmt;
            Self {
                fmt: sifr_generated_field_value_20e80d43821854a1_5f666d74,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2eloggingX2eFormatter {
        #[must_use]
        pub fn format(&self, level: &str, name: &str, msg: &str) -> String {
            let mut result: String = self.fmt.clone();
            result = result.replace("%(levelname)s", &level);
            result = result.replace("%(name)s", &name);
            result = result.replace("%(message)s", &msg);
            result
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2eloggingX2eFormatter {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "Formatter(_fmt={})", self.fmt)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2eloggingX2eFileHandler {
        pub path: String,
        pub level: SifrInt,
        pub formatter: SifrGeneratedStdlibSifrX2eloggingX2eFormatter,
    }
    impl SifrGeneratedStdlibSifrX2eloggingX2eFileHandler {
        #[must_use]
        pub fn new(path: String, level: SifrInt) -> Self {
            let sifr_generated_field_value_0e74a76ec4f48c05_5f70617468: String = {
                let mut sifr_generated_concat: String = String::with_capacity(path.len());
                sifr_generated_concat.push_str(path.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            let sifr_generated_field_value_70fb616fceb1e22c_5f6c6576656c: SifrInt = level.clone();
            let sifr_generated_field_value_07cfd6c6e0ac9648_5f666f726d6174746572: SifrGeneratedStdlibSifrX2eloggingX2eFormatter = SifrGeneratedStdlibSifrX2eloggingX2eFormatter::new(
                "%(levelname)s:%(name)s:%(message)s".to_string(),
            );
            Self {
                path: sifr_generated_field_value_0e74a76ec4f48c05_5f70617468,
                level: sifr_generated_field_value_70fb616fceb1e22c_5f6c6576656c,
                formatter: sifr_generated_field_value_07cfd6c6e0ac9648_5f666f726d6174746572,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2eloggingX2eFileHandler {
        #[must_use]
        pub fn path(&self) -> String {
            self.path.clone()
        }
    }
    impl SifrGeneratedStdlibSifrX2eloggingX2eFileHandler {
        #[must_use]
        pub fn level(&self) -> SifrInt {
            self.level.clone()
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2eloggingX2eFileHandler {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f,
                "FileHandler(_path={}, _level={}, _formatter={})",
                self.path, self.level, self.formatter
            )
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2eloggingX2eLogger {
        pub name: String,
        pub level: SifrInt,
        pub log_path: String,
        pub handler_kind: String,
        pub handler_path: String,
        pub handler_level: SifrInt,
        pub handler_fmt: String,
    }
    impl SifrGeneratedStdlibSifrX2eloggingX2eLogger {
        #[must_use]
        pub fn new(name: String, level: SifrInt) -> Self {
            let sifr_generated_field_value_2570757371473f6d_5f6e616d65: String = name;
            let sifr_generated_field_value_70fb616fceb1e22c_5f6c6576656c: SifrInt = level.clone();
            let sifr_generated_field_value_1fb1dcbc22de0cba_5f6c6f675f70617468: String =
                String::new();
            let sifr_generated_field_value_15ca476e60592199_5f68616e646c65725f6b696e64: String =
                String::new();
            let sifr_generated_field_value_7c86d4c1dd53d2b4_5f68616e646c65725f70617468: String =
                String::new();
            let sifr_generated_field_value_f71817da89e71523_5f68616e646c65725f6c6576656c: SifrInt =
                sifr_generated_const_4e4f54534554().clone();
            let sifr_generated_field_value_98e9bbb8fd5643d6_5f68616e646c65725f666d74: String =
                "%(levelname)s:%(name)s:%(message)s".to_string();
            Self {
                name: sifr_generated_field_value_2570757371473f6d_5f6e616d65,
                level: sifr_generated_field_value_70fb616fceb1e22c_5f6c6576656c,
                log_path: sifr_generated_field_value_1fb1dcbc22de0cba_5f6c6f675f70617468,
                handler_kind:
                    sifr_generated_field_value_15ca476e60592199_5f68616e646c65725f6b696e64,
                handler_path:
                    sifr_generated_field_value_7c86d4c1dd53d2b4_5f68616e646c65725f70617468,
                handler_level:
                    sifr_generated_field_value_f71817da89e71523_5f68616e646c65725f6c6576656c,
                handler_fmt: sifr_generated_field_value_98e9bbb8fd5643d6_5f68616e646c65725f666d74,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2eloggingX2eLogger {
        pub fn set_file(&mut self, path: &str) {
            self.log_path = {
                let mut sifr_generated_concat: String = String::with_capacity(path.len());
                sifr_generated_concat.push_str(path);
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
        }
    }
    impl SifrGeneratedStdlibSifrX2eloggingX2eLogger {
        pub fn add_handler(&mut self, handler: &SifrGeneratedStdlibSifrX2eloggingX2eFileHandler) {
            self.handler_kind = "file".to_string();
            self.handler_path = handler.path();
            self.handler_level = handler.level();
            self.handler_fmt = "%(levelname)s:%(name)s:%(message)s".to_string();
        }
    }
    impl SifrGeneratedStdlibSifrX2eloggingX2eLogger {
        pub fn clear_handler(&mut self) {
            self.handler_kind = String::new();
            self.handler_path = String::new();
            self.handler_level = sifr_generated_const_4e4f54534554().clone();
            self.handler_fmt = "%(levelname)s:%(name)s:%(message)s".to_string();
        }
    }
    impl SifrGeneratedStdlibSifrX2eloggingX2eLogger {
        #[must_use]
        pub fn sifr_generated_handler_allows(&self, level_num: &SifrInt) -> bool {
            if &self.handler_level.clone() == &sifr_generated_const_4e4f54534554() {
                return true;
            }
            level_num >= &self.handler_level
        }
    }
    impl SifrGeneratedStdlibSifrX2eloggingX2eLogger {
        #[must_use]
        pub fn sifr_generated_handler_line(&self, level: &str, msg: &str) -> String {
            let formatter: SifrGeneratedStdlibSifrX2eloggingX2eFormatter =
                SifrGeneratedStdlibSifrX2eloggingX2eFormatter::new(self.handler_fmt.clone());
            formatter.format(level, &self.name.clone(), msg)
        }
    }
    impl SifrGeneratedStdlibSifrX2eloggingX2eLogger {
        pub fn sifr_generated_emit(&self, level: &str, level_num: &SifrInt, msg: &str) {
            if &self.level > level_num {
                return;
            }
            if self.handler_kind.clone() == "null" {
                return;
            }
            if self.handler_kind.clone() == "stream" {
                if self.sifr_generated_handler_allows(level_num) {
                    println!("{}", self.sifr_generated_handler_line(level, msg));
                }
                return;
            }
            if self.handler_kind.clone() == "file" {
                if self.sifr_generated_handler_allows(level_num)
                    && !self.handler_path.clone().is_empty()
                {
                    let line: String = {
                        let mut sifr_generated_concat: String = String::with_capacity(1usize);
                        sifr_generated_concat
                            .push_str(self.sifr_generated_handler_line(level, msg).as_str());
                        sifr_generated_concat.push('\n');
                        sifr_generated_concat
                    };
                    let sifr_generated_try_res: Result<(), IOError> = (|| {
                        let mut fh: SifrGeneratedIoTextFileHandle = open_text(
                            &self.handler_path,
                            &"a".to_string(),
                            &Some(utf8().clone()),
                            &None,
                        )?;
                        let sifr_generated_try_res: Result<(), IOError> = (|| {
                            fh.write(&line)?;
                            Ok(())
                        })(
                        );
                        if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                            let e2 = sifr_generated_try_err.clone();
                            let _ = e2.message.clone();
                        }
                        (&mut fh).close();
                        Ok(())
                    })();
                    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                        let e = sifr_generated_try_err.clone();
                        let _ = e.message.clone();
                    }
                }
                return;
            }
            let line: String = {
                let mut sifr_generated_concat: String =
                    String::with_capacity(1usize + level.len() + 2usize + 2usize + msg.len());
                sifr_generated_concat.push('[');
                sifr_generated_concat.push_str(level);
                sifr_generated_concat.push_str("] ");
                sifr_generated_concat.push_str(self.name.clone().as_str());
                sifr_generated_concat.push_str(": ");
                sifr_generated_concat.push_str(msg);
                sifr_generated_concat
            };
            println!("{line}");
            if !self.log_path.clone().is_empty() {
                let sifr_generated_try_res: Result<(), IOError> = (|| {
                    let mut fh: SifrGeneratedIoTextFileHandle = open_text(
                        &self.log_path,
                        &"a".to_string(),
                        &Some(utf8().clone()),
                        &None,
                    )?;
                    let sifr_generated_try_res: Result<(), IOError> = (|| {
                        fh.write(&{
                            let mut sifr_generated_concat: String =
                                String::with_capacity(line.len() + 1usize);
                            sifr_generated_concat.push_str(line.as_str());
                            sifr_generated_concat.push('\n');
                            sifr_generated_concat
                        })?;
                        Ok(())
                    })();
                    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                        let e2 = sifr_generated_try_err.clone();
                        let _ = e2.message.clone();
                    }
                    (&mut fh).close();
                    Ok(())
                })();
                if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                    let e = sifr_generated_try_err.clone();
                    let _ = e.message.clone();
                }
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2eloggingX2eLogger {
        pub fn info(&self, msg: &str) {
            self.sifr_generated_emit(&"INFO".to_string(), &sifr_generated_const_494e464f(), msg);
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2eloggingX2eLogger {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f,
                "Logger(_name={}, _level={}, _log_path={}, _handler_kind={}, _handler_path={}, _handler_level={}, _handler_fmt={})",
                self.name,
                self.level,
                self.log_path,
                self.handler_kind,
                self.handler_path,
                self.handler_level,
                self.handler_fmt
            )
        }
    }
    #[derive(Debug, Clone)]
    pub struct SifrGeneratedStdlibSifrX2etimeX2estructTime {
        pub tm_year: SifrInt,
        pub tm_mon: SifrInt,
        pub tm_mday: SifrInt,
        pub tm_hour: SifrInt,
        pub tm_min: SifrInt,
        pub tm_sec: SifrInt,
        pub tm_wday: SifrInt,
        pub tm_yday: SifrInt,
        pub tm_isdst: SifrInt,
    }
    impl SifrGeneratedStdlibSifrX2etimeX2estructTime {
        #[must_use]
        #[expect(
            clippy::too_many_arguments,
            reason = "generated signature preserves the typed Sifr callable contract"
        )]
        pub fn new(
            tm_year: SifrInt,
            tm_mon: SifrInt,
            tm_mday_argument_a505494cd43c9214: SifrInt,
            tm_hour: SifrInt,
            tm_min_argument_103d514d457d4a49: SifrInt,
            tm_sec: SifrInt,
            tm_wday_argument_d5143a059ed34c12: SifrInt,
            tm_yday_argument_6b9a41f3b9220250: SifrInt,
            tm_isdst: SifrInt,
        ) -> Self {
            let sifr_generated_field_value_72897bf3bc91df5a_746d5f79656172: SifrInt =
                tm_year.clone();
            let sifr_generated_field_value_1029314d456c6adf_746d5f6d6f6e: SifrInt = tm_mon.clone();
            let sifr_generated_field_value_a505494cd43c9214_746d5f6d646179: SifrInt =
                tm_mday_argument_a505494cd43c9214.clone();
            let sifr_generated_field_value_129c5b76af381059_746d5f686f7572: SifrInt =
                tm_hour.clone();
            let sifr_generated_field_value_103d514d457d4a49_746d5f6d696e: SifrInt =
                tm_min_argument_103d514d457d4a49.clone();
            let sifr_generated_field_value_f3d84e4dc71632a0_746d5f736563: SifrInt = tm_sec.clone();
            let sifr_generated_field_value_d5143a059ed34c12_746d5f77646179: SifrInt =
                tm_wday_argument_d5143a059ed34c12.clone();
            let sifr_generated_field_value_6b9a41f3b9220250_746d5f79646179: SifrInt =
                tm_yday_argument_6b9a41f3b9220250.clone();
            let sifr_generated_field_value_d0ec16f562c1ee92_746d5f6973647374: SifrInt =
                tm_isdst.clone();
            Self {
                tm_year: sifr_generated_field_value_72897bf3bc91df5a_746d5f79656172,
                tm_mon: sifr_generated_field_value_1029314d456c6adf_746d5f6d6f6e,
                tm_mday: sifr_generated_field_value_a505494cd43c9214_746d5f6d646179,
                tm_hour: sifr_generated_field_value_129c5b76af381059_746d5f686f7572,
                tm_min: sifr_generated_field_value_103d514d457d4a49_746d5f6d696e,
                tm_sec: sifr_generated_field_value_f3d84e4dc71632a0_746d5f736563,
                tm_wday: sifr_generated_field_value_d5143a059ed34c12_746d5f77646179,
                tm_yday: sifr_generated_field_value_6b9a41f3b9220250_746d5f79646179,
                tm_isdst: sifr_generated_field_value_d0ec16f562c1ee92_746d5f6973647374,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2etimeX2estructTime {
        #[must_use]
        pub fn as_tuple(
            &self,
        ) -> (
            SifrInt,
            SifrInt,
            SifrInt,
            SifrInt,
            SifrInt,
            SifrInt,
            SifrInt,
            SifrInt,
            SifrInt,
        ) {
            (
                self.tm_year.clone(),
                self.tm_mon.clone(),
                self.tm_mday.clone(),
                self.tm_hour.clone(),
                self.tm_min.clone(),
                self.tm_sec.clone(),
                self.tm_wday.clone(),
                self.tm_yday.clone(),
                self.tm_isdst.clone(),
            )
        }
    }
    impl SifrGeneratedStdlibSifrX2etimeX2estructTime {
        #[must_use]
        pub fn isoformat(&self) -> String {
            let y: String = self.tm_year.clone().to_string();
            let mut mo: String = self.tm_mon.clone().to_string();
            if &SifrInt::from(mo.chars().count()) < &SifrInt::from_i64(2) {
                mo = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize + mo.len());
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(mo.as_str());
                    sifr_generated_concat
                };
            }
            let mut d: String = self.tm_mday.clone().to_string();
            if &SifrInt::from(d.chars().count()) < &SifrInt::from_i64(2) {
                d = {
                    let mut sifr_generated_concat: String = String::with_capacity(1usize + d.len());
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(d.as_str());
                    sifr_generated_concat
                };
            }
            let mut h: String = self.tm_hour.clone().to_string();
            if &SifrInt::from(h.chars().count()) < &SifrInt::from_i64(2) {
                h = {
                    let mut sifr_generated_concat: String = String::with_capacity(1usize + h.len());
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(h.as_str());
                    sifr_generated_concat
                };
            }
            let mut mi: String = self.tm_min.clone().to_string();
            if &SifrInt::from(mi.chars().count()) < &SifrInt::from_i64(2) {
                mi = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize + mi.len());
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(mi.as_str());
                    sifr_generated_concat
                };
            }
            let mut s: String = self.tm_sec.clone().to_string();
            if &SifrInt::from(s.chars().count()) < &SifrInt::from_i64(2) {
                s = {
                    let mut sifr_generated_concat: String = String::with_capacity(1usize + s.len());
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(s.as_str());
                    sifr_generated_concat
                };
            }
            {
                let mut sifr_generated_concat: String = String::with_capacity(
                    y.len()
                        + 1usize
                        + mo.len()
                        + 1usize
                        + d.len()
                        + 1usize
                        + h.len()
                        + 1usize
                        + mi.len()
                        + 1usize
                        + s.len(),
                );
                sifr_generated_concat.push_str(y.as_str());
                sifr_generated_concat.push('-');
                sifr_generated_concat.push_str(mo.as_str());
                sifr_generated_concat.push('-');
                sifr_generated_concat.push_str(d.as_str());
                sifr_generated_concat.push('T');
                sifr_generated_concat.push_str(h.as_str());
                sifr_generated_concat.push(':');
                sifr_generated_concat.push_str(mi.as_str());
                sifr_generated_concat.push(':');
                sifr_generated_concat.push_str(s.as_str());
                sifr_generated_concat
            }
        }
    }
    impl PartialEq for SifrGeneratedStdlibSifrX2etimeX2estructTime {
        fn eq(&self, other: &SifrGeneratedStdlibSifrX2etimeX2estructTime) -> bool {
            self.as_tuple() == other.as_tuple()
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2etimeX2estructTime {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.isoformat())
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2etimeitX2eTimer {}
    impl SifrGeneratedStdlibSifrX2etimeitX2eTimer {
        #[must_use]
        pub const fn new() -> Self {
            Self {}
        }
    }
    impl ::std::default::Default for SifrGeneratedStdlibSifrX2etimeitX2eTimer {
        fn default() -> Self {
            Self::new()
        }
    }
    impl SifrGeneratedStdlibSifrX2etimeitX2eTimer {
        #[must_use]
        pub fn timeit(&self, stmt: impl Fn(), number: &SifrInt) -> f64 {
            timeit(stmt, number.clone())
        }
    }
    impl SifrGeneratedStdlibSifrX2etimeitX2eTimer {
        #[must_use]
        pub fn sifr_generated_call__(&self, stmt: impl Fn(), number: &SifrInt) -> f64 {
            self.timeit(stmt, number)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct IOError {
        pub message: String,
        pub kind: String,
    }
    impl IOError {
        #[must_use]
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Error {
        pub message: String,
    }
    impl Error {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for Error {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for Error {}
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct FloatOverflowError {
        pub message: String,
    }
    impl FloatOverflowError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for FloatOverflowError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for FloatOverflowError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct FloatPrecisionLossError {
        pub message: String,
    }
    impl FloatPrecisionLossError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for FloatPrecisionLossError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for FloatPrecisionLossError {}
    impl From<IOError> for Error {
        fn from(err: IOError) -> Self {
            Self::new(err.message)
        }
    }
    impl From<ParseError> for Error {
        fn from(err: ParseError) -> Self {
            Self::new(err.message)
        }
    }
    impl From<ValueError> for Error {
        fn from(err: ValueError) -> Self {
            Self::new(err.message)
        }
    }
    impl From<FloatOverflowError> for Error {
        fn from(err: FloatOverflowError) -> Self {
            Self::new(err.message)
        }
    }
    impl From<FloatPrecisionLossError> for Error {
        fn from(err: FloatPrecisionLossError) -> Self {
            Self::new(err.message)
        }
    }
}
pub use sifr_generated_project_nominals::Error;
pub use sifr_generated_project_nominals::FloatOverflowError;
pub use sifr_generated_project_nominals::FloatPrecisionLossError;
pub use sifr_generated_project_nominals::IOError;
pub use sifr_generated_project_nominals::ParseError;
pub use sifr_generated_project_nominals::SifrGeneratedIoBinaryFileHandle;
pub use sifr_generated_project_nominals::SifrGeneratedIoTextFileHandle;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eencodingX2eEncodeError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eencodingX2eEncodeErrorHandler;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eencodingX2eEncodeOutcome;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eencodingX2eEncoding;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eloggingX2eFileHandler;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eloggingX2eLogger;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2etimeX2estructTime;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2etimeitX2eTimer;
pub use sifr_generated_project_nominals::ValueError;
mod sifr_generated_project_unions {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
            crate::sifr_generated_project_nominals::FloatOverflowError,
        ),
        SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
            crate::sifr_generated_project_nominals::FloatPrecisionLossError,
        ),
        SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
            crate::sifr_generated_project_nominals::ValueError,
        ),
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
    #[derive(Debug, Clone)]
    pub enum SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass5X3aError1X3a0(
            crate::sifr_generated_project_nominals::Error,
        ),
        SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
            crate::sifr_generated_project_nominals::FloatOverflowError,
        ),
        SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
            crate::sifr_generated_project_nominals::FloatPrecisionLossError,
        ),
        SifrGeneratedUnionVariant5X3aclass7X3aIOError1X3a0(
            crate::sifr_generated_project_nominals::IOError,
        ),
        SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
            crate::sifr_generated_project_nominals::ValueError,
        ),
    }
    impl From<crate::sifr_generated_project_nominals::Error>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0 {
        fn from(value: crate::sifr_generated_project_nominals::Error) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass5X3aError1X3a0(
                value,
            )
        }
    }
    impl From<crate::sifr_generated_project_nominals::FloatOverflowError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0 {
        fn from(
            value: crate::sifr_generated_project_nominals::FloatOverflowError,
        ) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                value,
            )
        }
    }
    impl From<crate::sifr_generated_project_nominals::FloatPrecisionLossError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0 {
        fn from(
            value: crate::sifr_generated_project_nominals::FloatPrecisionLossError,
        ) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                value,
            )
        }
    }
    impl From<crate::sifr_generated_project_nominals::IOError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0 {
        fn from(value: crate::sifr_generated_project_nominals::IOError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass7X3aIOError1X3a0(
                value,
            )
        }
    }
    impl From<crate::sifr_generated_project_nominals::ValueError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0 {
        fn from(value: crate::sifr_generated_project_nominals::ValueError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass5X3aError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass7X3aIOError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
}
use crate::sifr_generated_generated_support::*;
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_unions::SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0;
pub use sifr_generated_project_unions::SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0;
fn workload() {
    let mut i: SifrInt = SifrInt::from_i64(0);
    let mut total: SifrInt = SifrInt::from_i64(0);
    while &i < &SifrInt::from_i64(64) {
        total = &total + &i;
        i = &i + &SifrInt::from_i64(1);
    }
}
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    let mut demo_ok: bool = false;
    let log_path: String = "/tmp/sifr_runtime_logging_and_timers.log".to_string();
    let sifr_generated_try_res: Result<
        (),
        SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0,
    > = (|| {
        write_text(&log_path, &String::new())
            .map_err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass7X3aIOError1X3a0,
            )?;
        let mut logger: SifrGeneratedStdlibSifrX2eloggingX2eLogger = getLogger(
            &"logging_and_timers-demo".to_string(),
        );
        (&mut logger).set_file(&log_path);
        let fh: SifrGeneratedStdlibSifrX2eloggingX2eFileHandler = SifrGeneratedStdlibSifrX2eloggingX2eFileHandler::new(
            log_path.to_string(),
            sifr_generated_const_494e464f(),
        );
        (&mut logger).add_handler(&fh);
        logger.info(&"hello".to_string());
        (&mut logger).clear_handler();
        let gmt: SifrGeneratedStdlibSifrX2etimeX2estructTime = gmtime_struct(0.0_f64);
        let epoch_tm: SifrGeneratedStdlibSifrX2etimeX2estructTime = SifrGeneratedStdlibSifrX2etimeX2estructTime::new(
            SifrInt::from_i64(1970),
            SifrInt::from_i64(1),
            SifrInt::from_i64(1),
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
            SifrInt::from_i64(3),
            SifrInt::from_i64(1),
            SifrInt::from_i64(0),
        );
        let epoch_stamp: f64 = mktime(&epoch_tm)
            .map_err(|sifr_generated_e| match sifr_generated_e {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                    sifr_generated_union_value,
                ) => {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                        sifr_generated_union_value,
                    )
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                    sifr_generated_union_value,
                ) => {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                        sifr_generated_union_value,
                    )
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
                    sifr_generated_union_value,
                ) => {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
                        sifr_generated_union_value,
                    )
                }
            })?;
        let epoch_ok_value_83bd003ade93c135: bool = epoch_stamp == 0.0_f64;
        let timer: SifrGeneratedStdlibSifrX2etimeitX2eTimer = SifrGeneratedStdlibSifrX2etimeitX2eTimer::new();
        let elapsed: f64 = timer.sifr_generated_call__(workload, &SifrInt::from_i64(4));
        let content: String = read_text(&log_path)
            .map_err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass7X3aIOError1X3a0,
            )?;
        demo_ok = content == "INFO:logging_and_timers-demo:hello\n"
            && &gmt.tm_year.clone() == &SifrInt::from_i64(1970)
            && epoch_ok_value_83bd003ade93c135 && elapsed >= 0.0_f64
            && &sifr_generated_const_54494d455a4f4e45() == &SifrInt::from_i64(0)
            && sifr_generated_const_545a4e414d45().0.clone() == "UTC";
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        match sifr_generated_try_err {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass5X3aError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let e = sifr_generated_try_variant_error.clone();
                let _ = e.message.clone();
            }
            SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let e = Error::new(sifr_generated_try_variant_error.clone().message);
                let _ = e.message.clone();
            }
            SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let e = Error::new(sifr_generated_try_variant_error.clone().message);
                let _ = e.message.clone();
            }
            SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass7X3aIOError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let e = Error::new(sifr_generated_try_variant_error.clone().message);
                let _ = e.message.clone();
            }
            SifrGeneratedUnion8X3asequence5X3aunion1X3a523X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let e = Error::new(sifr_generated_try_variant_error.clone().message);
                let _ = e.message.clone();
            }
        }
    }
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        if exists(&log_path) {
            remove_file(&log_path)?;
        }
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone();
    }
    assert!(demo_ok);
    println!("runtime_logging_and_timers_time_timeit_object_surface_demo: ok");
}
