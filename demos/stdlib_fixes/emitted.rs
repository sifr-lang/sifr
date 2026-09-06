// src/main.rs
mod sifr_generated_generated_support {
    use crate::{
        FloatOverflowError, FloatPrecisionLossError, IOError, ParseError, RegexError,
        SifrGeneratedIoBinaryFileHandle, SifrGeneratedIoNativeFileHandle,
        SifrGeneratedIoTextFileHandle, SifrGeneratedStdlibSifrX2ecsvX2eDialect,
        SifrGeneratedStdlibSifrX2ecsvX2ereader, SifrGeneratedStdlibSifrX2edatetimeX2edatetime,
        SifrGeneratedStdlibSifrX2edatetimeX2etimezone,
        SifrGeneratedStdlibSifrX2eencodingX2eDecodeError,
        SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler,
        SifrGeneratedStdlibSifrX2eencodingX2eDecodeOutcome,
        SifrGeneratedStdlibSifrX2eencodingX2eEncodeError,
        SifrGeneratedStdlibSifrX2eencodingX2eEncodeErrorHandler,
        SifrGeneratedStdlibSifrX2eencodingX2eEncodeOutcome,
        SifrGeneratedStdlibSifrX2eencodingX2eEncoding, SifrGeneratedStdlibSifrX2eloggingX2eLogger,
        SifrGeneratedStdlibSifrX2erandomX2eRandom, SifrGeneratedStdlibSifrX2erandomX2eRandomState,
        SifrGeneratedStdlibSifrX2ereX2ePattern, SifrGeneratedStdlibSifrX2eregexX2eCompiledPattern,
        ValueError,
    };
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) fn sifr_generated_encoding_decode_text_impl(
        data: &[u8],
        encoding: &str,
        errors: &str,
    ) -> Result<String, ParseError> {
        ::sifr_stdlib::encoding::encoding_decode_text(data, encoding, errors).map_err(
            |sifr_generated_bridge_error| ParseError {
                message: sifr_generated_bridge_error.to_string(),
            },
        )
    }
    pub(crate) fn sifr_generated_encoding_decode_recoveries_impl(
        data: &[u8],
        encoding: &str,
        errors: &str,
    ) -> Result<Vec<String>, ParseError> {
        ::sifr_stdlib::encoding::encoding_decode_recoveries(data, encoding, errors).map_err(
            |sifr_generated_bridge_error| ParseError {
                message: sifr_generated_bridge_error.to_string(),
            },
        )
    }
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
    pub(crate) fn sifr_generated_open_file(path: &str, mode: &str) -> Result<String, IOError> {
        ::sifr_stdlib::fs::open_file(path, mode).map_err(sifr_generated_io_err)
    }
    pub(crate) fn sifr_generated_file_close(handle: &str) {
        ::sifr_stdlib::fs::file_close(handle);
    }
    pub(crate) fn sifr_generated_file_read_bytes(
        handle: &str,
        size: Option<SifrInt>,
    ) -> Result<Vec<u8>, IOError> {
        ::sifr_stdlib::fs::file_read_bytes(
            handle,
            size.map(::sifr_runtime::interop::SifrIntBridge::from),
        )
        .map_err(sifr_generated_io_err)
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
    pub(crate) fn file_read_bytes(
        handle: &SifrGeneratedIoNativeFileHandle,
        size: Option<SifrInt>,
    ) -> Result<Vec<u8>, IOError> {
        sifr_generated_file_read_bytes(&handle.id.clone(), size.clone())
    }
    pub(crate) fn file_write_bytes(
        handle: &SifrGeneratedIoNativeFileHandle,
        data: &[u8],
    ) -> Result<(), IOError> {
        sifr_generated_file_write_bytes(&handle.id.clone(), data)
    }
    pub(crate) fn getcwd() -> Result<String, IOError> {
        ::sifr_stdlib::fs::getcwd().map_err(sifr_generated_io_err)
    }
    pub(crate) fn glob_pattern(dir: &str, pattern: &str) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::glob_pattern(dir, pattern).map_err(sifr_generated_io_err)
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
    pub(crate) fn sifr_generated_encoding_decode_outcome(
        data: &[u8],
        encoding: &str,
        errors: &str,
    ) -> Result<
        SifrGeneratedStdlibSifrX2eencodingX2eDecodeOutcome,
        SifrGeneratedStdlibSifrX2eencodingX2eDecodeError,
    > {
        let sifr_generated_try_res: Result<
            Result<
                SifrGeneratedStdlibSifrX2eencodingX2eDecodeOutcome,
                SifrGeneratedStdlibSifrX2eencodingX2eDecodeError,
            >,
            ParseError,
        > = (|| {
            let text: String = sifr_generated_encoding_decode_text_impl(data, encoding, errors)?;
            let recoveries: Vec<String> =
                sifr_generated_encoding_decode_recoveries_impl(data, encoding, errors)?;
            Ok(Ok(SifrGeneratedStdlibSifrX2eencodingX2eDecodeOutcome::new(
                text, recoveries,
            )))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err.clone();
            Err(SifrGeneratedStdlibSifrX2eencodingX2eDecodeError::new(
                e.message.clone(),
            ))
        })
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
    pub(crate) fn sifr_generated_decode_handler_name(
        errors: &Option<SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler>,
    ) -> String {
        let Some(errors) = errors.as_ref() else {
            return sifr_generated_const_4445434f44455f4552524f52535f535452494354();
        };
        {
            let mut sifr_generated_concat: String = String::new();
            sifr_generated_concat.push_str(errors.name.clone().as_str());
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        }
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
    pub(crate) fn decode_outcome(
        data: &[u8],
        enc: &SifrGeneratedStdlibSifrX2eencodingX2eEncoding,
        errors: &Option<SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler>,
    ) -> Result<
        SifrGeneratedStdlibSifrX2eencodingX2eDecodeOutcome,
        SifrGeneratedStdlibSifrX2eencodingX2eDecodeError,
    > {
        let handler_name: String = sifr_generated_decode_handler_name(errors);
        sifr_generated_encoding_decode_outcome(data, &enc.label.clone(), &handler_name)
    }
    pub(crate) fn decode(
        data: &[u8],
        enc: &SifrGeneratedStdlibSifrX2eencodingX2eEncoding,
        errors: &Option<SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler>,
    ) -> Result<String, SifrGeneratedStdlibSifrX2eencodingX2eDecodeError> {
        let sifr_generated_try_res: Result<
            Result<String, SifrGeneratedStdlibSifrX2eencodingX2eDecodeError>,
            SifrGeneratedStdlibSifrX2eencodingX2eDecodeError,
        > = (|| {
            let outcome: SifrGeneratedStdlibSifrX2eencodingX2eDecodeOutcome =
                decode_outcome(data, enc, errors)?;
            Ok(Ok(outcome.get_text()))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err.clone();
            Err(SifrGeneratedStdlibSifrX2eencodingX2eDecodeError::new(
                e.message.clone(),
            ))
        })
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
    pub(crate) enum SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eDecodeError1X3a019X3a5X3aclass7X3aIOError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass7X3aIOError1X3a0(IOError),
        SifrGeneratedUnionVariant5X3aclass25X3asifrX2eencodingX2eDecodeError1X3a0(
            SifrGeneratedStdlibSifrX2eencodingX2eDecodeError,
        ),
    }
    impl From<IOError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eDecodeError1X3a019X3a5X3aclass7X3aIOError1X3a0 {
        fn from(value: IOError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eDecodeError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass7X3aIOError1X3a0(
                value,
            )
        }
    }
    impl From<SifrGeneratedStdlibSifrX2eencodingX2eDecodeError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eDecodeError1X3a019X3a5X3aclass7X3aIOError1X3a0 {
        fn from(value: SifrGeneratedStdlibSifrX2eencodingX2eDecodeError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eDecodeError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass25X3asifrX2eencodingX2eDecodeError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eDecodeError1X3a019X3a5X3aclass7X3aIOError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eDecodeError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass7X3aIOError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eDecodeError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass25X3asifrX2eencodingX2eDecodeError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
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
    pub(crate) fn sifr_generated_mode_is_readable(mode: &str) -> bool {
        mode.contains(&"r".to_string()) || mode.contains(&"+".to_string())
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
    pub(crate) const fn sifr_generated_const_51554f54455f4e4f4e45() -> SifrInt {
        SifrInt::from_i64(3)
    }
    pub(crate) fn sifr_generated_copy_dialect(
        dialect: &SifrGeneratedStdlibSifrX2ecsvX2eDialect,
    ) -> SifrGeneratedStdlibSifrX2ecsvX2eDialect {
        SifrGeneratedStdlibSifrX2ecsvX2eDialect::new(
            dialect.delimiter.clone().to_string(),
            dialect.quotechar.clone().to_string(),
            dialect.escapechar.clone().to_string(),
            dialect.doublequote,
            dialect.skipinitialspace,
            dialect.lineterminator.clone().to_string(),
            dialect.quoting.clone(),
        )
    }
    pub(crate) fn sifr_generated_validate_char(name: &str, value: &str) {
        let _ = name.to_owned();
        let _ = value.to_owned();
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "generated signature preserves the typed Sifr callable contract"
    )]
    pub(crate) fn sifr_generated_resolve_dialect(
        dialect: &Option<SifrGeneratedStdlibSifrX2ecsvX2eDialect>,
        delimiter: &str,
        quotechar: &str,
        escapechar: &str,
        doublequote: bool,
        skipinitialspace: bool,
        lineterminator: &str,
        quoting: SifrInt,
    ) -> SifrGeneratedStdlibSifrX2ecsvX2eDialect {
        let Some(dialect) = dialect.as_ref() else {
            return SifrGeneratedStdlibSifrX2ecsvX2eDialect::new(
                delimiter.to_owned(),
                quotechar.to_owned(),
                escapechar.to_owned(),
                doublequote,
                skipinitialspace,
                lineterminator.to_owned(),
                quoting.clone(),
            );
        };
        sifr_generated_copy_dialect(dialect)
    }
    pub(crate) fn sifr_generated_quotechar_value(
        dialect: &SifrGeneratedStdlibSifrX2ecsvX2eDialect,
    ) -> String {
        let quotechar: String = {
            let mut sifr_generated_concat: String = String::new();
            sifr_generated_concat.push_str(dialect.quotechar.clone().as_str());
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        };
        if quotechar.as_str() == String::new().as_str() {
            return "\"".to_string();
        }
        quotechar
    }
    pub(crate) fn sifr_generated_append_field(row: &mut Vec<String>, field: String) {
        row.push(field.to_string());
    }
    pub(crate) fn sifr_generated_append_row(rows: &mut Vec<Vec<String>>, row: Vec<String>) {
        rows.push(row);
    }
    pub(crate) fn sifr_generated_char_at(text: &str, index: SifrInt) -> String {
        let sifr_generated_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
        if &index < &SifrInt::from_i64(0)
            || &index >= &SifrInt::from(sifr_generated_chars_text.len())
        {
            return String::new();
        }
        let ch: Option<String> = {
            let sifr_generated_string_index = index.clone();
            let sifr_generated_string_index_normalized =
                sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_text.len());
            sifr_generated_chars_text
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string());
        let Some(ch) = ch else {
            return String::new();
        };
        ch
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "generated signature preserves the typed Sifr callable contract"
    )]
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub(crate) fn parse_csv(
        text: &str,
        dialect: &Option<SifrGeneratedStdlibSifrX2ecsvX2eDialect>,
        delimiter: &str,
        quotechar: &str,
        escapechar: &str,
        doublequote: bool,
        skipinitialspace: bool,
        quoting: SifrInt,
    ) -> Vec<Vec<String>> {
        let quotechar = quotechar.to_owned();
        let sifr_generated_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
        let resolved: SifrGeneratedStdlibSifrX2ecsvX2eDialect = sifr_generated_resolve_dialect(
            dialect,
            delimiter,
            &quotechar,
            escapechar,
            doublequote,
            skipinitialspace,
            &"\n".to_string(),
            quoting.clone(),
        );
        let mut rows: Vec<Vec<String>> = Vec::new();
        let mut row: Vec<String> = Vec::new();
        let mut field: String = String::new();
        let mut in_quotes: bool = false;
        let mut field_started: bool = false;
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &SifrInt::from(sifr_generated_chars_text.len()) {
            let ch_value: String = sifr_generated_char_at(text, i.clone());
            if in_quotes {
                if !resolved.escapechar.clone().is_empty()
                    && ch_value == resolved.escapechar.clone()
                {
                    if &(&i + &SifrInt::from_i64(1))
                        < &SifrInt::from(sifr_generated_chars_text.len())
                    {
                        let escaped_value: String =
                            sifr_generated_char_at(text, &i + &SifrInt::from_i64(1));
                        field.push_str(escaped_value.as_str());
                        i = &i + &SifrInt::from_i64(2);
                        continue;
                    }
                    field.push_str(ch_value.as_str());
                    i = &i + &SifrInt::from_i64(1);
                    continue;
                }
                if !resolved.quotechar.clone().is_empty() && ch_value == resolved.quotechar.clone()
                {
                    let quotechar: String = sifr_generated_quotechar_value(&resolved);
                    if resolved.doublequote
                        && &(&i + &SifrInt::from_i64(1))
                            < &SifrInt::from(sifr_generated_chars_text.len())
                        && sifr_generated_char_at(text, &i + &SifrInt::from_i64(1)) == quotechar
                    {
                        field.push_str(quotechar.as_str());
                        i = &i + &SifrInt::from_i64(2);
                        continue;
                    }
                    in_quotes = false;
                    i = &i + &SifrInt::from_i64(1);
                    continue;
                }
                field.push_str(ch_value.as_str());
                i = &i + &SifrInt::from_i64(1);
                continue;
            }
            if !field_started && resolved.skipinitialspace && ch_value == " " {
                i = &i + &SifrInt::from_i64(1);
                continue;
            }
            if !resolved.escapechar.clone().is_empty() && ch_value == resolved.escapechar.clone() {
                if &(&i + &SifrInt::from_i64(1)) < &SifrInt::from(sifr_generated_chars_text.len()) {
                    let escaped_plain_value: String =
                        sifr_generated_char_at(text, &i + &SifrInt::from_i64(1));
                    field.push_str(escaped_plain_value.as_str());
                    field_started = true;
                    i = &i + &SifrInt::from_i64(2);
                    continue;
                }
                field.push_str(ch_value.as_str());
                field_started = true;
                i = &i + &SifrInt::from_i64(1);
                continue;
            }
            if &resolved.quoting.clone() != &sifr_generated_const_51554f54455f4e4f4e45()
                && !resolved.quotechar.clone().is_empty()
            {
                let quotechar2_value_123324c155e57c27: String =
                    sifr_generated_quotechar_value(&resolved);
                if ch_value == quotechar2_value_123324c155e57c27 {
                    in_quotes = true;
                    field_started = true;
                    i = &i + &SifrInt::from_i64(1);
                    continue;
                }
            }
            if ch_value == resolved.delimiter.clone() {
                sifr_generated_append_field(&mut row, field);
                field = String::new();
                field_started = false;
                i = &i + &SifrInt::from_i64(1);
                continue;
            }
            if ch_value == "\n" || ch_value == "\r" {
                if ch_value == "\r"
                    && &(&i + &SifrInt::from_i64(1))
                        < &SifrInt::from(sifr_generated_chars_text.len())
                    && sifr_generated_char_at(text, &i + &SifrInt::from_i64(1)) == "\n"
                {
                    i = &i + &SifrInt::from_i64(1);
                }
                if &SifrInt::from(row.len()) == &SifrInt::from_i64(0) && field.is_empty() {
                    sifr_generated_append_row(&mut rows, Vec::new());
                } else {
                    sifr_generated_append_field(&mut row, field);
                    sifr_generated_append_row(&mut rows, row);
                }
                row = Vec::new();
                field = String::new();
                field_started = false;
                i = &i + &SifrInt::from_i64(1);
                continue;
            }
            field.push_str(ch_value.as_str());
            field_started = true;
            i = &i + &SifrInt::from_i64(1);
        }
        let _ = in_quotes;
        if &SifrInt::from(row.len()) > &SifrInt::from_i64(0) || !field.is_empty() {
            sifr_generated_append_field(&mut row, field);
            sifr_generated_append_row(&mut rows, row);
        }
        rows
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "generated signature preserves the typed Sifr callable contract"
    )]
    pub(crate) fn reader_from_path(
        path: &str,
        dialect: &Option<SifrGeneratedStdlibSifrX2ecsvX2eDialect>,
        delimiter: &str,
        quotechar: &str,
        escapechar: &str,
        doublequote: bool,
        skipinitialspace: bool,
        quoting: SifrInt,
    ) -> Result<SifrGeneratedStdlibSifrX2ecsvX2ereader, IOError> {
        let sifr_generated_try_res: Result<
            Result<SifrGeneratedStdlibSifrX2ecsvX2ereader, IOError>,
            IOError,
        > = (|| {
            let text: String = read_text(path)?;
            Ok(Ok(SifrGeneratedStdlibSifrX2ecsvX2ereader::new(
                text,
                dialect.clone(),
                delimiter.to_owned(),
                quotechar.to_owned(),
                escapechar.to_owned(),
                doublequote,
                skipinitialspace,
                quoting.clone(),
            )))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err.clone();
            Err(e)
        })
    }
    pub(crate) fn datetime_now_struct() -> Vec<SifrInt> {
        ::sifr_stdlib::time::datetime_now_struct()
            .into_iter()
            .map(::sifr_runtime::interop::SifrIntBridge::into_sifr_int)
            .collect()
    }
    pub(crate) fn datetime_from_timestamp(ts: f64) -> Result<String, ValueError> {
        ::sifr_stdlib::time::datetime_from_timestamp(ts).map_err(|sifr_generated_bridge_error| {
            ValueError {
                message: sifr_generated_bridge_error.to_string(),
            }
        })
    }
    pub(crate) fn time_now() -> f64 {
        ::sifr_stdlib::time::time_now()
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub(crate) enum SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(FloatOverflowError),
        SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
            FloatPrecisionLossError,
        ),
    }
    impl From<FloatOverflowError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
        fn from(value: FloatOverflowError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                value,
            )
        }
    }
    impl From<FloatPrecisionLossError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
        fn from(value: FloatPrecisionLossError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                value,
            )
        }
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
    #[derive(Debug, Clone)]
    pub(crate) enum SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(FloatOverflowError),
        SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
            FloatPrecisionLossError,
        ),
        SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(ValueError),
    }
    impl From<FloatOverflowError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
        fn from(value: FloatOverflowError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                value,
            )
        }
    }
    impl From<FloatPrecisionLossError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
        fn from(value: FloatPrecisionLossError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                value,
            )
        }
    }
    impl From<ValueError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
        fn from(value: ValueError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
                value,
            )
        }
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
    pub(crate) fn sifr_generated_six_digits(value: SifrInt) -> String {
        let mut rendered: String = value.to_string();
        let mut sifr_generated_chars_rendered: Vec<char> = rendered.chars().collect::<Vec<char>>();
        while &SifrInt::from(sifr_generated_chars_rendered.len()) < &SifrInt::from_i64(6) {
            rendered = {
                let mut sifr_generated_concat: String =
                    String::with_capacity(1usize + rendered.len());
                sifr_generated_concat.push('0');
                sifr_generated_concat.push_str(rendered.as_str());
                sifr_generated_concat
            };
            sifr_generated_chars_rendered = rendered.chars().collect::<Vec<char>>();
        }
        rendered
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub(crate) fn sifr_generated_parse_datetime_iso(
        value: &str,
    ) -> Result<(SifrInt, SifrInt, SifrInt, SifrInt, SifrInt, SifrInt), ValueError> {
        let sifr_generated_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
        let Some(_checked_value_2) = {
            let sifr_generated_string_index = SifrInt::from_i64(4);
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_value.len());
            sifr_generated_chars_value
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string()) else {
            return Err(ValueError::new("invalid datetime string".to_string()));
        };
        let Some(_checked_value_3) = {
            let sifr_generated_string_index = SifrInt::from_i64(7);
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_value.len());
            sifr_generated_chars_value
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string()) else {
            return Err(ValueError::new("invalid datetime string".to_string()));
        };
        let Some(_checked_value_4) = {
            let sifr_generated_string_index = SifrInt::from_i64(10);
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_value.len());
            sifr_generated_chars_value
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string()) else {
            return Err(ValueError::new("invalid datetime string".to_string()));
        };
        let Some(_checked_value_5) = {
            let sifr_generated_string_index = SifrInt::from_i64(13);
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_value.len());
            sifr_generated_chars_value
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string()) else {
            return Err(ValueError::new("invalid datetime string".to_string()));
        };
        let Some(_checked_value_6) = {
            let sifr_generated_string_index = SifrInt::from_i64(16);
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_value.len());
            sifr_generated_chars_value
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string()) else {
            return Err(ValueError::new("invalid datetime string".to_string()));
        };
        if {
            let sifr_generated_string_index = SifrInt::from_i64(4);
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_value.len());
            sifr_generated_chars_value
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(Some)
            != Some(Some('-'))
            || {
                let sifr_generated_string_index = SifrInt::from_i64(7);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_value.len());
                sifr_generated_chars_value
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(Some)
                != Some(Some('-'))
            || {
                let sifr_generated_string_index = SifrInt::from_i64(10);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_value.len());
                sifr_generated_chars_value
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(Some)
                != Some(Some('T'))
            || {
                let sifr_generated_string_index = SifrInt::from_i64(13);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_value.len());
                sifr_generated_chars_value
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(Some)
                != Some(Some(':'))
            || {
                let sifr_generated_string_index = SifrInt::from_i64(16);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_value.len());
                sifr_generated_chars_value
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(Some)
                != Some(Some(':'))
        {
            return Err(ValueError::new("invalid datetime string".to_string()));
        }
        let sifr_generated_try_res: Result<
            Result<(SifrInt, SifrInt, SifrInt, SifrInt, SifrInt, SifrInt), ValueError>,
            ParseError,
        > = (|| {
            let year: SifrInt = SifrInt::parse_decimal(
                &sifr_generated_substring(value, SifrInt::from_i64(0), SifrInt::from_i64(4)),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
            let month: SifrInt = SifrInt::parse_decimal(
                &sifr_generated_substring(value, SifrInt::from_i64(5), SifrInt::from_i64(7)),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
            let day: SifrInt = SifrInt::parse_decimal(
                &sifr_generated_substring(value, SifrInt::from_i64(8), SifrInt::from_i64(10)),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
            let hour: SifrInt = SifrInt::parse_decimal(
                &sifr_generated_substring(value, SifrInt::from_i64(11), SifrInt::from_i64(13)),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
            let minute: SifrInt = SifrInt::parse_decimal(
                &sifr_generated_substring(value, SifrInt::from_i64(14), SifrInt::from_i64(16)),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
            let second: SifrInt = SifrInt::parse_decimal(
                &sifr_generated_substring(value, SifrInt::from_i64(17), SifrInt::from_i64(19)),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
            Ok(Ok((year, month, day, hour, minute, second)))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let _e_5f65 = sifr_generated_try_err.clone();
            Err(ValueError::new("invalid datetime string".to_string()))
        })
    }
    pub(crate) fn sifr_generated_timezone_offset_from_text(
        text: &str,
    ) -> Result<SifrInt, ValueError> {
        let sifr_generated_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
        if text == "UTC" {
            return Ok(SifrInt::from_i64(0));
        }
        if &SifrInt::from(sifr_generated_chars_text.len()) != &SifrInt::from_i64(9) {
            return Err(ValueError::new("invalid timezone string".to_string()));
        }
        if sifr_generated_substring(text, SifrInt::from_i64(0), SifrInt::from_i64(3)) != "UTC" {
            return Err(ValueError::new("invalid timezone string".to_string()));
        }
        let sign_value: String =
            sifr_generated_substring(text, SifrInt::from_i64(3), SifrInt::from_i64(4));
        if sign_value != "+" && sign_value != "-" {
            return Err(ValueError::new("invalid timezone string".to_string()));
        }
        if {
            let sifr_generated_string_index = SifrInt::from_i64(6);
            let sifr_generated_string_index_normalized =
                sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_text.len());
            sifr_generated_chars_text
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(Some)
            != Some(Some(':'))
        {
            return Err(ValueError::new("invalid timezone string".to_string()));
        }
        let sifr_generated_try_res: Result<Result<SifrInt, ValueError>, ParseError> = (|| {
            let hours: SifrInt = SifrInt::parse_decimal(
                &sifr_generated_substring(text, SifrInt::from_i64(4), SifrInt::from_i64(6)),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
            let minutes: SifrInt = SifrInt::parse_decimal(
                &sifr_generated_substring(text, SifrInt::from_i64(7), SifrInt::from_i64(9)),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
            let mut offset: SifrInt =
                &(&hours * &SifrInt::from_i64(3600)) + &(&minutes * &SifrInt::from_i64(60));
            if sign_value == "-" {
                offset = -&offset;
            }
            Ok(Ok(offset))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let _e_5f65 = sifr_generated_try_err.clone();
            Err(ValueError::new("invalid timezone string".to_string()))
        })
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub(crate) fn sifr_generated_from_timestamp_with_tz(
        ts: f64,
        tz: &Option<SifrGeneratedStdlibSifrX2edatetimeX2etimezone>,
    ) -> Result<SifrGeneratedStdlibSifrX2edatetimeX2edatetime, ValueError> {
        let sifr_generated_try_res: Result<
            Result<SifrGeneratedStdlibSifrX2edatetimeX2edatetime, ValueError>,
            SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0,
        > = (|| {
            let whole_seconds: SifrInt = SifrInt::from_f64_trunc(ts)
                .ok_or_else(|| ValueError {
                    message: "cannot convert non-finite float to int".to_string(),
                })
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0,
                )?;
            let whole_seconds_float: f64 = whole_seconds
                .clone()
                .checked_to_f64()
                .map_err(|sifr_generated_float_error| match sifr_generated_float_error {
                    ::sifr_runtime::IntegerFloatConversionError::Overflow => {
                        SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                            FloatOverflowError::new(
                                "exact integer is outside the finite float range"
                                    .to_string(),
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
                .map_err(|sifr_generated_e| match sifr_generated_e {
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
                })?;
            let fractional: f64 = ts - whole_seconds_float;
            let mut microsecond: SifrInt = SifrInt::from_f64_trunc(
                    fractional * 1_000_000.0_f64,
                )
                .ok_or_else(|| ValueError {
                    message: "cannot convert non-finite float to int".to_string(),
                })
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0,
                )?;
            if &microsecond < &SifrInt::from_i64(0) {
                microsecond = -&microsecond;
            }
            let mut adjusted_seconds: SifrInt = whole_seconds.clone();
            let mut tz_offset_value: SifrInt = SifrInt::from_i64(0);
            let tz_has_offset: bool = if let Some(tz) = tz.as_ref() {
                {
                    let tz_text: String = tz.to_string();
                    let tz_offset: SifrInt = sifr_generated_timezone_offset_from_text(
                            &tz_text,
                        )
                        .map_err(
                            SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0,
                        )?;
                    adjusted_seconds = &whole_seconds + &tz_offset;
                    tz_offset_value = tz_offset;
                    true
                }
            } else {
                false
            };
            let adjusted_seconds_float: f64 = adjusted_seconds
                .clone()
                .checked_to_f64()
                .map_err(|sifr_generated_float_error| match sifr_generated_float_error {
                    ::sifr_runtime::IntegerFloatConversionError::Overflow => {
                        SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                            FloatOverflowError::new(
                                "exact integer is outside the finite float range"
                                    .to_string(),
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
                .map_err(|sifr_generated_e| match sifr_generated_e {
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
                })?;
            let rendered: String = datetime_from_timestamp(adjusted_seconds_float)
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0,
                )?;
            let parts: (SifrInt, SifrInt, SifrInt, SifrInt, SifrInt, SifrInt) = sifr_generated_parse_datetime_iso(
                    &rendered,
                )
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0,
                )?;
            let year_part: Option<SifrInt> = Some(parts.0.clone());
            let month_part: Option<SifrInt> = Some(parts.1.clone());
            let day_part: Option<SifrInt> = Some(parts.2.clone());
            let hour_part: Option<SifrInt> = Some(parts.3.clone());
            let minute_part: Option<SifrInt> = Some(parts.4.clone());
            let second_part: Option<SifrInt> = Some(parts.5.clone());
            let mut year: SifrInt = SifrInt::from_i64(0);
            let mut month: SifrInt = SifrInt::from_i64(1);
            let mut day: SifrInt = SifrInt::from_i64(1);
            let mut hour: SifrInt = SifrInt::from_i64(0);
            let mut minute: SifrInt = SifrInt::from_i64(0);
            let mut second: SifrInt = SifrInt::from_i64(0);
            if let Some(year_part) = year_part.clone() {
                year = year_part;
            }
            if let Some(month_part) = month_part.clone() {
                month = month_part;
            }
            if let Some(day_part) = day_part.clone() {
                day = day_part;
            }
            if let Some(hour_part) = hour_part.clone() {
                hour = hour_part;
            }
            if let Some(minute_part) = minute_part.clone() {
                minute = minute_part;
            }
            if let Some(second_part) = second_part.clone() {
                second = second_part;
            }
            if tz_has_offset {
                return Ok(
                    Ok(
                        SifrGeneratedStdlibSifrX2edatetimeX2edatetime::new(
                            year.clone(),
                            month.clone(),
                            day.clone(),
                            hour.clone(),
                            minute.clone(),
                            second.clone(),
                            microsecond.clone(),
                            Some(tz_offset_value),
                        ),
                    ),
                );
            }
            Ok(
                Ok(
                    SifrGeneratedStdlibSifrX2edatetimeX2edatetime::new(
                        year.clone(),
                        month.clone(),
                        day.clone(),
                        hour.clone(),
                        minute.clone(),
                        second.clone(),
                        microsecond.clone(),
                        None,
                    ),
                ),
            )
        })();
        sifr_generated_try_res
            .unwrap_or_else(|sifr_generated_try_err| match sifr_generated_try_err {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let e = sifr_generated_try_variant_error.clone();
                    Err(ValueError::new(e.message.clone()))
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let e = sifr_generated_try_variant_error.clone();
                    Err(ValueError::new(e.message.clone()))
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let e = sifr_generated_try_variant_error.clone();
                    Err(ValueError::new(e.message.clone()))
                }
            })
    }
    pub(crate) fn now(
        tz: &Option<SifrGeneratedStdlibSifrX2edatetimeX2etimezone>,
    ) -> SifrGeneratedStdlibSifrX2edatetimeX2edatetime {
        let current_epoch: f64 = time_now();
        let sifr_generated_try_res: Result<
            SifrGeneratedStdlibSifrX2edatetimeX2edatetime,
            ValueError,
        > = (|| {
            let current: SifrGeneratedStdlibSifrX2edatetimeX2edatetime =
                sifr_generated_from_timestamp_with_tz(current_epoch, tz)?;
            Ok(current)
        })();
        match sifr_generated_try_res {
            Ok(sifr_generated_ret_val) => sifr_generated_ret_val,
            Err(sifr_generated_try_err) => {
                let _e_5f65 = sifr_generated_try_err.clone();
                let parts: Vec<SifrInt> = datetime_now_struct();
                let mut yr: SifrInt = SifrInt::from_i64(0);
                let mut mo: SifrInt = SifrInt::from_i64(1);
                let mut dy: SifrInt = SifrInt::from_i64(1);
                let mut hr: SifrInt = SifrInt::from_i64(0);
                let mut mn: SifrInt = SifrInt::from_i64(0);
                let mut sc: SifrInt = SifrInt::from_i64(0);
                for (i, v) in Box::new(parts.iter().cloned().enumerate().map(
                    |sifr_generated_pair| {
                        (
                            SifrInt::from(sifr_generated_pair.0) + SifrInt::from_i64(0),
                            sifr_generated_pair.1,
                        )
                    },
                )) {
                    if &i == &SifrInt::from_i64(0) {
                        yr = v.clone();
                    }
                    if &i == &SifrInt::from_i64(1) {
                        mo = v.clone();
                    }
                    if &i == &SifrInt::from_i64(2) {
                        dy = v.clone();
                    }
                    if &i == &SifrInt::from_i64(3) {
                        hr = v.clone();
                    }
                    if &i == &SifrInt::from_i64(4) {
                        mn = v.clone();
                    }
                    if &i == &SifrInt::from_i64(5) {
                        sc = v.clone();
                    }
                }
                if let Some(tz) = tz.as_ref() {
                    let sifr_generated_try_res: Result<
                        SifrGeneratedStdlibSifrX2edatetimeX2edatetime,
                        ValueError,
                    > = (|| {
                        let parsed_offset: SifrInt =
                            sifr_generated_timezone_offset_from_text(&tz.to_string())?;
                        Ok(SifrGeneratedStdlibSifrX2edatetimeX2edatetime::new(
                            yr.clone(),
                            mo.clone(),
                            dy.clone(),
                            hr.clone(),
                            mn.clone(),
                            sc.clone(),
                            SifrInt::from_i64(0),
                            Some(parsed_offset),
                        ))
                    })();
                    match sifr_generated_try_res {
                        Ok(sifr_generated_ret_val) => {
                            return sifr_generated_ret_val;
                        }
                        Err(sifr_generated_try_err) => {
                            let _e_5f65 = sifr_generated_try_err.clone();
                            return SifrGeneratedStdlibSifrX2edatetimeX2edatetime::new(
                                yr.clone(),
                                mo.clone(),
                                dy.clone(),
                                hr.clone(),
                                mn.clone(),
                                sc.clone(),
                                SifrInt::from_i64(0),
                                None,
                            );
                        }
                    }
                }
                SifrGeneratedStdlibSifrX2edatetimeX2edatetime::new(
                    yr.clone(),
                    mo.clone(),
                    dy.clone(),
                    hr.clone(),
                    mn.clone(),
                    sc.clone(),
                    SifrInt::from_i64(0),
                    None,
                )
            }
        }
    }
    pub(crate) fn set_global_level(level: SifrInt) {
        ::sifr_stdlib::logging::set_global_level(::sifr_runtime::interop::SifrIntBridge::from(
            level,
        ));
    }
    pub(crate) fn get_global_level() -> SifrInt {
        ::sifr_stdlib::logging::get_global_level().into_sifr_int()
    }
    pub(crate) const fn sifr_generated_const_4445425547() -> SifrInt {
        SifrInt::from_i64(10)
    }
    pub(crate) const fn sifr_generated_const_494e464f() -> SifrInt {
        SifrInt::from_i64(20)
    }
    pub(crate) const fn sifr_generated_const_5741524e494e47() -> SifrInt {
        SifrInt::from_i64(30)
    }
    pub(crate) const fn sifr_generated_const_4552524f52() -> SifrInt {
        SifrInt::from_i64(40)
    }
    pub(crate) const fn sifr_generated_const_435249544943414c() -> SifrInt {
        SifrInt::from_i64(50)
    }
    pub(crate) const fn sifr_generated_const_4e4f54534554() -> SifrInt {
        SifrInt::from_i64(0)
    }
    pub(crate) fn sifr_generated_level_name_to_num(level: &str) -> SifrInt {
        if level == "DEBUG" {
            return sifr_generated_const_4445425547();
        }
        if level == "INFO" {
            return sifr_generated_const_494e464f();
        }
        if level == "WARNING" {
            return sifr_generated_const_5741524e494e47();
        }
        if level == "ERROR" {
            return sifr_generated_const_4552524f52();
        }
        if level == "CRITICAL" {
            return sifr_generated_const_435249544943414c();
        }
        sifr_generated_const_4e4f54534554()
    }
    #[expect(
        non_snake_case,
        reason = "generated Rust preserves this exact typed Sifr source contract"
    )]
    pub(crate) fn basicConfig(level: SifrInt) -> SifrGeneratedStdlibSifrX2eloggingX2eLogger {
        set_global_level(level.clone());
        SifrGeneratedStdlibSifrX2eloggingX2eLogger::new("root".to_string(), level.clone())
    }
    #[expect(
        non_snake_case,
        reason = "generated Rust preserves this exact typed Sifr source contract"
    )]
    pub(crate) fn getLogger(name: &str) -> SifrGeneratedStdlibSifrX2eloggingX2eLogger {
        let level: SifrInt = get_global_level();
        SifrGeneratedStdlibSifrX2eloggingX2eLogger::new(name.to_owned(), level.clone())
    }
    pub(crate) struct SifrGeneratedYielder<T> {
        pub(crate) slot: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    }
    pub(crate) struct SifrGeneratedYieldFuture<T> {
        pub(crate) slot: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
        pub(crate) value: Option<T>,
    }
    impl<T> Unpin for SifrGeneratedYieldFuture<T> {}
    impl<T> ::std::future::Future for SifrGeneratedYieldFuture<T> {
        type Output = ();
        fn poll(
            self: ::std::pin::Pin<&mut Self>,
            _: &mut ::std::task::Context<'_>,
        ) -> ::std::task::Poll<()> {
            let state = self.get_mut();
            let Some(value) = state.value.take() else {
                return ::std::task::Poll::Ready(());
            };
            sifr_generated_store_suspended(&state.slot, value);
            ::std::task::Poll::Pending
        }
    }
    impl<T> SifrGeneratedYielder<T> {
        pub(crate) fn suspend(&self, value: T) -> SifrGeneratedYieldFuture<T> {
            SifrGeneratedYieldFuture {
                slot: ::std::sync::Arc::clone(&self.slot),
                value: Some(value),
            }
        }
    }
    pub(crate) fn sifr_generated_store_suspended<T>(
        slot: &::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
        value: T,
    ) {
        match slot.lock() {
            Ok(mut state) => *state = Some(value),
            Err(poisoned) => *poisoned.into_inner() = Some(value),
        }
    }
    pub(crate) fn sifr_generated_take_suspended<T>(
        slot: &::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    ) -> Option<T> {
        match slot.lock() {
            Ok(mut state) => state.take(),
            Err(poisoned) => poisoned.into_inner().take(),
        }
    }
    pub(crate) struct SifrGeneratedGenerator<T> {
        pub(crate) producer:
            Option<::std::pin::Pin<Box<dyn ::std::future::Future<Output = ()> + 'static>>>,
        pub(crate) yielded: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
        pub(crate) complete: bool,
    }
    impl<T> SifrGeneratedGenerator<T> {
        pub(crate) fn new<
            F: FnOnce(SifrGeneratedYielder<T>) -> Fut + 'static,
            Fut: ::std::future::Future<Output = ()> + 'static,
        >(
            factory: F,
        ) -> Self {
            let yielded = ::std::sync::Arc::new(::std::sync::Mutex::new(None));
            let producer = factory(SifrGeneratedYielder {
                slot: ::std::sync::Arc::clone(&yielded),
            });
            Self {
                producer: Some(Box::pin(producer)),
                yielded,
                complete: false,
            }
        }
    }
    impl<T> Iterator for SifrGeneratedGenerator<T> {
        type Item = T;
        fn next(&mut self) -> Option<T> {
            if self.complete {
                return None;
            }
            let completed = {
                let Some(producer) = self.producer.as_mut() else {
                    self.complete = true;
                    return None;
                };
                let mut context = ::std::task::Context::from_waker(::std::task::Waker::noop());
                ::std::future::Future::poll(producer.as_mut(), &mut context).is_ready()
            };
            let yielded = sifr_generated_take_suspended(&self.yielded);
            if completed {
                self.complete = true;
                self.producer = None;
            }
            yielded
        }
    }
    pub(crate) fn sifr_generated_iter_list_str(
        entries: Vec<String>,
    ) -> Box<dyn Iterator<Item = String>> {
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<String>| {
                let mut i: SifrInt = SifrInt::from_i64(0);
                while &i < &SifrInt::from(entries.len()) {
                    let Some(sifr_generated_checked_value_7) = ({
                        let sifr_generated_checked_read_collection = &entries;
                        let sifr_generated_checked_read_index = i.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    }) else {
                        break;
                    };
                    sifr_generated_yielder
                        .suspend(sifr_generated_checked_value_7.clone())
                        .await;
                    i = &i + &SifrInt::from_i64(1);
                }
            },
        ))
    }
    pub(crate) fn sifr_generated_glob_list(
        path: &str,
        pattern: &str,
    ) -> Result<Vec<String>, IOError> {
        glob_pattern(path, pattern)
    }
    pub(crate) fn sifr_generated_glob_to_iter(
        path: &str,
        pattern: &str,
    ) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
        let sifr_generated_try_res: Result<
            Result<Box<dyn Iterator<Item = String>>, IOError>,
            IOError,
        > = (|| {
            let entries: Vec<String> = sifr_generated_glob_list(path, pattern)?;
            Ok(Ok(sifr_generated_iter_list_str(entries)))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err.clone();
            Err(e)
        })
    }
    pub(crate) fn random_seed() -> SifrInt {
        ::sifr_stdlib::random::random_seed().into_sifr_int()
    }
    pub(crate) fn random_module_state_words() -> Vec<SifrInt> {
        ::sifr_stdlib::random::random_module_state_words()
            .into_iter()
            .map(::sifr_runtime::interop::SifrIntBridge::into_sifr_int)
            .collect()
    }
    pub(crate) fn random_module_state_index() -> SifrInt {
        ::sifr_stdlib::random::random_module_state_index().into_sifr_int()
    }
    pub(crate) fn random_module_state_gauss_next() -> Option<f64> {
        ::sifr_stdlib::random::random_module_state_gauss_next()
    }
    pub(crate) fn random_module_set_state(
        words: &[SifrInt],
        index: SifrInt,
        gauss_next: Option<f64>,
    ) -> Result<(), ValueError> {
        ::sifr_stdlib::random::random_module_set_state(
            &words
                .iter()
                .cloned()
                .map(::sifr_runtime::interop::SifrIntBridge::from)
                .collect::<Vec<_>>(),
            ::sifr_runtime::interop::SifrIntBridge::from(index),
            gauss_next,
        )
        .map_err(|sifr_generated_bridge_error| ValueError {
            message: sifr_generated_bridge_error.to_string(),
        })
    }
    pub(crate) const fn sifr_generated_const_5f4d545f4e() -> SifrInt {
        SifrInt::from_i64(624)
    }
    pub(crate) const fn sifr_generated_const_5f4d545f4d() -> SifrInt {
        SifrInt::from_i64(397)
    }
    pub(crate) const fn sifr_generated_const_5f4d545f4d41545249585f41() -> SifrInt {
        SifrInt::from_i64(2_567_483_615)
    }
    pub(crate) const fn sifr_generated_const_5f4d545f55505045525f4d41534b() -> SifrInt {
        SifrInt::from_i64(2_147_483_648)
    }
    pub(crate) const fn sifr_generated_const_5f4d545f4c4f5745525f4d41534b() -> SifrInt {
        SifrInt::from_i64(2_147_483_647)
    }
    pub(crate) const fn sifr_generated_const_5f4d545f46() -> SifrInt {
        SifrInt::from_i64(1_812_433_253)
    }
    pub(crate) const fn sifr_generated_const_5f4d545f574f52445f4d41534b() -> SifrInt {
        SifrInt::from_i64(4_294_967_295)
    }
    pub(crate) fn sifr_generated_state_word_at(words: &[SifrInt], index: SifrInt) -> SifrInt {
        let value: Option<SifrInt> = {
            let sifr_generated_checked_read_collection = &words;
            let sifr_generated_checked_read_index = index.clone();
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        };
        let Some(value_value_7ce4fd9430e80cea) = value.clone() else {
            return SifrInt::from_i64(0);
        };
        value_value_7ce4fd9430e80cea
    }
    pub(crate) fn sifr_generated_clone_words(words: &[SifrInt]) -> Vec<SifrInt> {
        let mut copied: Vec<SifrInt> = Vec::new();
        for word in words.iter().cloned() {
            copied.push(word);
        }
        copied
    }
    pub(crate) fn sifr_generated_normalize_seed_input(seed_value: Option<SifrInt>) -> SifrInt {
        let Some(seed_value) = seed_value.clone() else {
            return random_seed();
        };
        seed_value.clone()
    }
    pub(crate) fn sifr_generated_seed_words_from_seed(seed_value: SifrInt) -> Vec<SifrInt> {
        let mut words: Vec<SifrInt> =
            vec![&seed_value & &sifr_generated_const_5f4d545f574f52445f4d41534b()];
        let mut i: SifrInt = SifrInt::from_i64(1);
        while &i < &sifr_generated_const_5f4d545f4e() {
            let prev: SifrInt = sifr_generated_state_word_at(&words, &i - &SifrInt::from_i64(1));
            let next_word: SifrInt = &(&(&sifr_generated_const_5f4d545f46()
                * &(&prev ^ &prev.floor_div_known_nonzero(&SifrInt::from_i64(1_073_741_824))))
                + &i)
                & &sifr_generated_const_5f4d545f574f52445f4d41534b();
            words.push(next_word);
            i = &i + &SifrInt::from_i64(1);
        }
        words
    }
    pub(crate) fn sifr_generated_build_state_from_module_storage()
    -> SifrGeneratedStdlibSifrX2erandomX2eRandomState {
        SifrGeneratedStdlibSifrX2erandomX2eRandomState::new(
            SifrInt::from_i64(3),
            random_module_state_words(),
            random_module_state_index(),
            random_module_state_gauss_next(),
        )
    }
    pub(crate) fn sifr_generated_store_state_into_module_storage(
        state: &SifrGeneratedStdlibSifrX2erandomX2eRandomState,
    ) {
        let sifr_generated_set_result: Result<(), ValueError> = random_module_set_state(
            &sifr_generated_clone_words(&state.state_words.clone()),
            state.index.clone(),
            state.gauss_next,
        );
        let _ = sifr_generated_set_result;
    }
    pub(crate) fn sifr_generated_ensure_module_state_initialized() {
        let words: Vec<SifrInt> = random_module_state_words();
        if &SifrInt::from(words.len()) == &sifr_generated_const_5f4d545f4e() {
            return;
        }
        let bootstrap: SifrGeneratedStdlibSifrX2erandomX2eRandom =
            SifrGeneratedStdlibSifrX2erandomX2eRandom::new(Some(SifrInt::from_i64(5489)));
        sifr_generated_store_state_into_module_storage(&bootstrap.getstate());
    }
    pub(crate) fn sifr_generated_module_random() -> SifrGeneratedStdlibSifrX2erandomX2eRandom {
        sifr_generated_ensure_module_state_initialized();
        let mut r: SifrGeneratedStdlibSifrX2erandomX2eRandom =
            SifrGeneratedStdlibSifrX2erandomX2eRandom::new(Some(SifrInt::from_i64(0)));
        let sifr_generated_set_result: Result<(), ValueError> =
            r.setstate(&sifr_generated_build_state_from_module_storage());
        let _ = sifr_generated_set_result;
        r
    }
    pub(crate) fn sifr_generated_sync_module_random(
        generator: &mut SifrGeneratedStdlibSifrX2erandomX2eRandom,
    ) {
        sifr_generated_store_state_into_module_storage(&generator.getstate());
    }
    pub(crate) fn choice<T: Clone + 'static>(items: &[T]) -> Result<T, ValueError> {
        let item_count: SifrInt = SifrInt::from(items.len());
        if &item_count == &SifrInt::from_i64(0) {
            return Err(ValueError::new(
                "choice: items must not be empty".to_string(),
            ));
        }
        let mut generator: SifrGeneratedStdlibSifrX2erandomX2eRandom =
            sifr_generated_module_random();
        let index: SifrInt = generator
            .sifr_generated_next_u32()
            .floor_mod_known_nonzero(&item_count);
        let picked: Option<T> = {
            let sifr_generated_checked_read_collection = &items;
            let sifr_generated_checked_read_index = index.clone();
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        };
        sifr_generated_sync_module_random(&mut generator);
        let Some(picked_value_9fda901c871bd7d9) = picked else {
            return Err(ValueError::new("choice: index out of range".to_string()));
        };
        Ok(picked_value_9fda901c871bd7d9)
    }
    pub(crate) trait SifrGeneratedOpaqueSifrStdlibSifrX2eregexX2eCompiledPatternMethods {
        fn search(&self, text: &str) -> Result<Option<String>, RegexError>;
        fn is_match(&self, text: &str) -> Result<bool, RegexError>;
        fn sub(&self, replacement: &str, text: &str) -> Result<String, RegexError>;
        fn findall(&self, text: &str) -> Result<Vec<String>, RegexError>;
        fn split(&self, text: &str) -> Result<Vec<String>, RegexError>;
        fn pattern(&self) -> Result<String, RegexError>;
        fn flags(&self) -> Result<SifrInt, RegexError>;
    }
    pub(crate) fn compile_pattern_flags(
        pattern: &str,
        flags: SifrInt,
    ) -> Result<SifrGeneratedStdlibSifrX2eregexX2eCompiledPattern, RegexError> {
        ::sifr_stdlib::regex::compile_pattern_flags(
            pattern,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map_err(|sifr_generated_bridge_error| RegexError {
            message: sifr_generated_bridge_error.to_string(),
            detail: sifr_generated_bridge_error.to_string(),
        })
    }
    pub(crate) fn re_find_flags(
        pattern: &str,
        text: &str,
        flags: SifrInt,
    ) -> Result<Option<String>, RegexError> {
        ::sifr_stdlib::regex::re_find_flags(
            pattern,
            text,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map_err(|sifr_generated_bridge_error| RegexError {
            message: sifr_generated_bridge_error.to_string(),
            detail: sifr_generated_bridge_error.to_string(),
        })
    }
    pub(crate) const fn sifr_generated_const_49474e4f524543415345() -> SifrInt {
        SifrInt::from_i64(2)
    }
    pub(crate) const fn sifr_generated_const_4d554c54494c494e45() -> SifrInt {
        SifrInt::from_i64(8)
    }
    pub(crate) fn search_flags(
        pattern: &str,
        text: &str,
        flags: SifrInt,
    ) -> Result<Option<String>, RegexError> {
        re_find_flags(pattern, text, flags.clone())
    }
    pub(crate) fn compile_flags(
        pattern: &str,
        flags: SifrInt,
    ) -> Result<SifrGeneratedStdlibSifrX2ereX2ePattern, RegexError> {
        let sifr_generated_try_res: Result<
            Result<SifrGeneratedStdlibSifrX2ereX2ePattern, RegexError>,
            RegexError,
        > = (|| {
            let compiled: SifrGeneratedStdlibSifrX2eregexX2eCompiledPattern =
                compile_pattern_flags(pattern, flags.clone())?;
            Ok(Ok(SifrGeneratedStdlibSifrX2ereX2ePattern::new(
                compiled,
                pattern.to_owned(),
                flags.clone(),
            )))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let error = sifr_generated_try_err.clone();
            Err(RegexError::new(error.message.clone()))
        })
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
    pub struct SifrGeneratedStdlibSifrX2eencodingX2eDecodeError {
        pub message: String,
    }
    impl SifrGeneratedStdlibSifrX2eencodingX2eDecodeError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Debug for SifrGeneratedStdlibSifrX2eencodingX2eDecodeError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            f.debug_struct("DecodeError")
                .field("message", &self.message)
                .finish()
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2eencodingX2eDecodeError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }
    impl ::std::error::Error for SifrGeneratedStdlibSifrX2eencodingX2eDecodeError {}
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
    pub struct SifrGeneratedStdlibSifrX2eencodingX2eDecodeOutcome {
        pub text: String,
        pub recoveries: Vec<String>,
    }
    impl SifrGeneratedStdlibSifrX2eencodingX2eDecodeOutcome {
        #[must_use]
        pub fn new(text: String, recoveries: Vec<String>) -> Self {
            let sifr_generated_field_value_fa04f4ef1995407e_74657874: String = {
                let mut sifr_generated_concat: String = String::with_capacity(text.len());
                sifr_generated_concat.push_str(text.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            let sifr_generated_field_value_eb53194d835eec2e_7265636f766572696573: Vec<String> =
                recoveries;
            Self {
                text: sifr_generated_field_value_fa04f4ef1995407e_74657874,
                recoveries: sifr_generated_field_value_eb53194d835eec2e_7265636f766572696573,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2eencodingX2eDecodeOutcome {
        #[must_use]
        pub fn get_text(&self) -> String {
            {
                let mut sifr_generated_concat: String = String::new();
                sifr_generated_concat.push_str(self.text.clone().as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            }
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
        pub fn read_bytes(&self, size: &Option<SifrInt>) -> Result<Vec<u8>, IOError> {
            if self.closed {
                return Err(IOError::new(sifr_generated_closed_stream_error()));
            }
            if !self.readable() {
                return Err(IOError::new("stream is not readable".to_string()));
            }
            file_read_bytes(&self.handle, size.clone())
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
        pub fn readable(&self) -> bool {
            sifr_generated_mode_is_readable(&self.mode)
        }
    }
    impl SifrGeneratedIoBinaryFileHandle {
        #[must_use]
        pub fn writable(&self) -> bool {
            sifr_generated_mode_is_writable(&self.mode)
        }
    }
    impl SifrGeneratedIoBinaryFileHandle {
        #[must_use]
        pub fn sifr_generated_enter__(&self) -> SifrGeneratedIoBinaryFileHandle {
            self.clone()
        }
    }
    impl SifrGeneratedIoBinaryFileHandle {
        pub fn sifr_generated_exit__(&mut self) {
            self.close();
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
        pub fn read(&self) -> Result<String, IOError> {
            let sifr_generated_try_res: Result<
                Result<String, IOError>,
                SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eDecodeError1X3a019X3a5X3aclass7X3aIOError1X3a0,
            > = (|| {
                let data: Vec<u8> = self
                    .binary
                    .read_bytes(&None)
                    .map_err(
                        SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eDecodeError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass7X3aIOError1X3a0,
                    )?;
                let text: String = decode(
                        &data,
                        &self.encoding,
                        &Some(self.decode_errors.clone()),
                    )
                    .map_err(
                        SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eDecodeError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass25X3asifrX2eencodingX2eDecodeError1X3a0,
                    )?;
                Ok(Ok(text))
            })();
            sifr_generated_try_res
                .unwrap_or_else(|sifr_generated_try_err| match sifr_generated_try_err {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eDecodeError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass7X3aIOError1X3a0(
                        sifr_generated_try_variant_error,
                    ) => {
                        let e = sifr_generated_try_variant_error.clone();
                        Err(e)
                    }
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eDecodeError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass25X3asifrX2eencodingX2eDecodeError1X3a0(
                        sifr_generated_try_variant_error,
                    ) => {
                        let e = sifr_generated_try_variant_error.clone();
                        Err(
                            IOError::new({
                                let mut sifr_generated_concat: String = String::with_capacity(
                                    20usize,
                                );
                                sifr_generated_concat.push_str("text decode failed: ");
                                sifr_generated_concat.push_str(e.message.clone().as_str());
                                sifr_generated_concat
                            }),
                        )
                    }
                })
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
    impl SifrGeneratedIoTextFileHandle {
        #[must_use]
        pub fn sifr_generated_enter__(&self) -> SifrGeneratedIoTextFileHandle {
            self.clone()
        }
    }
    impl SifrGeneratedIoTextFileHandle {
        pub fn sifr_generated_exit__(&mut self) {
            self.close();
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
    pub struct SifrGeneratedStdlibSifrX2ecsvX2eDialect {
        pub delimiter: String,
        pub quotechar: String,
        pub escapechar: String,
        pub doublequote: bool,
        pub skipinitialspace: bool,
        pub lineterminator: String,
        pub quoting: SifrInt,
    }
    impl SifrGeneratedStdlibSifrX2ecsvX2eDialect {
        #[must_use]
        pub fn new(
            delimiter: String,
            quotechar: String,
            escapechar: String,
            doublequote: bool,
            skipinitialspace: bool,
            lineterminator: String,
            quoting: SifrInt,
        ) -> Self {
            let mut resolved_quoting: SifrInt = quoting.clone();
            sifr_generated_validate_char(&"delimiter".to_string(), &delimiter);
            if !quotechar.is_empty() {
                sifr_generated_validate_char(&"quotechar".to_string(), &quotechar);
            }
            if !escapechar.is_empty() {
                sifr_generated_validate_char(&"escapechar".to_string(), &escapechar);
            }
            if quotechar.is_empty()
                && &resolved_quoting != &sifr_generated_const_51554f54455f4e4f4e45()
            {
                resolved_quoting = sifr_generated_const_51554f54455f4e4f4e45().clone();
            }
            let sifr_generated_field_value_894f6deb0b90819a_64656c696d69746572: String = delimiter;
            let sifr_generated_field_value_071afb87ccff598f_71756f746563686172: String = quotechar;
            let sifr_generated_field_value_35712447096491ca_65736361706563686172: String =
                escapechar;
            let sifr_generated_field_value_0c828b579bd5cc5c_646f75626c6571756f7465: bool =
                doublequote;
            let sifr_generated_field_value_aed440ff683599d8_736b6970696e697469616c7370616365: bool =
                skipinitialspace;
            let sifr_generated_field_value_5421666eeec5d0d2_6c696e657465726d696e61746f72: String =
                lineterminator;
            let sifr_generated_field_value_7f757e185a85e280_71756f74696e67: SifrInt =
                resolved_quoting.clone();
            Self {
                delimiter: sifr_generated_field_value_894f6deb0b90819a_64656c696d69746572,
                quotechar: sifr_generated_field_value_071afb87ccff598f_71756f746563686172,
                escapechar: sifr_generated_field_value_35712447096491ca_65736361706563686172,
                doublequote: sifr_generated_field_value_0c828b579bd5cc5c_646f75626c6571756f7465,
                skipinitialspace:
                    sifr_generated_field_value_aed440ff683599d8_736b6970696e697469616c7370616365,
                lineterminator:
                    sifr_generated_field_value_5421666eeec5d0d2_6c696e657465726d696e61746f72,
                quoting: sifr_generated_field_value_7f757e185a85e280_71756f74696e67,
            }
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2ecsvX2eDialect {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f,
                "Dialect(delimiter={}, quotechar={}, escapechar={}, doublequote={}, skipinitialspace={}, lineterminator={}, quoting={})",
                self.delimiter,
                self.quotechar,
                self.escapechar,
                self.doublequote,
                self.skipinitialspace,
                self.lineterminator,
                self.quoting
            )
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SifrGeneratedStdlibSifrX2ecsvX2ereader {
        pub rows: Vec<Vec<String>>,
        pub pos: SifrInt,
        pub dialect: SifrGeneratedStdlibSifrX2ecsvX2eDialect,
    }
    impl SifrGeneratedStdlibSifrX2ecsvX2ereader {
        #[must_use]
        #[expect(
            clippy::too_many_arguments,
            reason = "generated signature preserves the typed Sifr callable contract"
        )]
        pub fn new(
            text: String,
            dialect: Option<SifrGeneratedStdlibSifrX2ecsvX2eDialect>,
            delimiter: String,
            quotechar: String,
            escapechar: String,
            doublequote: bool,
            skipinitialspace: bool,
            quoting: SifrInt,
        ) -> Self {
            let resolved_dialect: SifrGeneratedStdlibSifrX2ecsvX2eDialect =
                sifr_generated_resolve_dialect(
                    &dialect,
                    &delimiter,
                    &quotechar,
                    &escapechar,
                    doublequote,
                    skipinitialspace,
                    &"\n".to_string(),
                    quoting.clone(),
                );
            let rows: Vec<Vec<String>> = parse_csv(
                &text,
                &None,
                &resolved_dialect.delimiter.clone().to_string(),
                &resolved_dialect.quotechar.clone().to_string(),
                &resolved_dialect.escapechar.clone().to_string(),
                resolved_dialect.doublequote,
                resolved_dialect.skipinitialspace,
                resolved_dialect.quoting.clone(),
            );
            let sifr_generated_field_value_ac4a5fa27eb34095_6469616c656374: SifrGeneratedStdlibSifrX2ecsvX2eDialect = resolved_dialect;
            let sifr_generated_field_value_d742ae5cfb4259e3_5f726f7773: Vec<Vec<String>> = rows;
            let sifr_generated_field_value_e04b9443eebba9b4_5f706f73: SifrInt =
                SifrInt::from_i64(0);
            Self {
                dialect: sifr_generated_field_value_ac4a5fa27eb34095_6469616c656374,
                rows: sifr_generated_field_value_d742ae5cfb4259e3_5f726f7773,
                pos: sifr_generated_field_value_e04b9443eebba9b4_5f706f73,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2ecsvX2ereader {
        #[must_use]
        pub fn rows(&self) -> Vec<Vec<String>> {
            let mut result: Vec<Vec<String>> = Vec::new();
            for row in self.rows.iter().cloned() {
                let mut copied: Vec<String> = Vec::new();
                for field in row.iter().cloned() {
                    copied.push(field.to_string());
                }
                result.push(copied.to_vec());
            }
            result
        }
    }
    #[derive(Debug, Clone)]
    pub struct SifrGeneratedStdlibSifrX2edatetimeX2etimezone {
        pub offset: SifrInt,
    }
    impl SifrGeneratedStdlibSifrX2edatetimeX2etimezone {
        #[must_use]
        pub fn new(offset: SifrInt) -> Self {
            let sifr_generated_field_value_d85dd81618b4c959_5f6f6666736574: SifrInt =
                offset.clone();
            Self {
                offset: sifr_generated_field_value_d85dd81618b4c959_5f6f6666736574,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2edatetimeX2etimezone {
        #[must_use]
        pub fn iso_suffix(&self) -> String {
            let sign: String = if &self.offset.clone() < &SifrInt::from_i64(0) {
                "-".to_string()
            } else {
                "+".to_string()
            };
            let mut abs_offset: SifrInt = self.offset.clone();
            if &abs_offset < &SifrInt::from_i64(0) {
                abs_offset = -&abs_offset;
            }
            let h: SifrInt = abs_offset.floor_div_known_nonzero(&SifrInt::from_i64(3600));
            let m: SifrInt = abs_offset
                .floor_mod_known_nonzero(&SifrInt::from_i64(3600))
                .floor_div_known_nonzero(&SifrInt::from_i64(60));
            let mut hs: String = h.to_string();
            if &SifrInt::from(hs.chars().count()) < &SifrInt::from_i64(2) {
                hs = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize + hs.len());
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(hs.as_str());
                    sifr_generated_concat
                };
            }
            let mut ms: String = m.to_string();
            if &SifrInt::from(ms.chars().count()) < &SifrInt::from_i64(2) {
                ms = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize + ms.len());
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(ms.as_str());
                    sifr_generated_concat
                };
            }
            {
                let mut sifr_generated_concat: String =
                    String::with_capacity(sign.len() + hs.len() + 1usize + ms.len());
                sifr_generated_concat.push_str(sign.as_str());
                sifr_generated_concat.push_str(hs.as_str());
                sifr_generated_concat.push(':');
                sifr_generated_concat.push_str(ms.as_str());
                sifr_generated_concat
            }
        }
    }
    impl PartialEq for SifrGeneratedStdlibSifrX2edatetimeX2etimezone {
        fn eq(&self, other: &SifrGeneratedStdlibSifrX2edatetimeX2etimezone) -> bool {
            self.offset == other.offset
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2edatetimeX2etimezone {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            if &self.offset.clone() == &SifrInt::from_i64(0) {
                return write!(f, "UTC");
            }
            write!(f, "{}", {
                let mut sifr_generated_concat: String = String::with_capacity(3usize);
                sifr_generated_concat.push_str("UTC");
                sifr_generated_concat.push_str(self.iso_suffix().as_str());
                sifr_generated_concat
            })
        }
    }
    #[derive(Debug, Clone)]
    pub struct SifrGeneratedStdlibSifrX2edatetimeX2edatetime {
        pub year: SifrInt,
        pub month: SifrInt,
        pub day: SifrInt,
        pub hour: SifrInt,
        pub minute: SifrInt,
        pub second: SifrInt,
        pub microsecond: SifrInt,
        pub tz_offset: Option<SifrInt>,
    }
    impl SifrGeneratedStdlibSifrX2edatetimeX2edatetime {
        #[must_use]
        #[expect(
            clippy::too_many_arguments,
            reason = "generated signature preserves the typed Sifr callable contract"
        )]
        pub fn new(
            year: SifrInt,
            month: SifrInt,
            day: SifrInt,
            hour: SifrInt,
            minute: SifrInt,
            second: SifrInt,
            microsecond: SifrInt,
            tz_offset: Option<SifrInt>,
        ) -> Self {
            let sifr_generated_field_value_7c64634977425edc_79656172: SifrInt = year.clone();
            let sifr_generated_field_value_f4bdc3936faf56a5_6d6f6e7468: SifrInt = month.clone();
            let sifr_generated_field_value_ca8d3918f4578f1d_646179: SifrInt = day.clone();
            let sifr_generated_field_value_407efecc7eb5764f_686f7572: SifrInt = hour.clone();
            let sifr_generated_field_value_5bb2f9bdf2fad1e9_6d696e757465: SifrInt = minute.clone();
            let sifr_generated_field_value_a49985ef4cee20bd_7365636f6e64: SifrInt = second.clone();
            let sifr_generated_field_value_27f934ab879dcfa3_6d6963726f7365636f6e64: SifrInt =
                microsecond.clone();
            let sifr_generated_field_value_17964c5d1d2f9a66_5f747a5f6f6666736574: Option<SifrInt> =
                tz_offset.clone();
            Self {
                year: sifr_generated_field_value_7c64634977425edc_79656172,
                month: sifr_generated_field_value_f4bdc3936faf56a5_6d6f6e7468,
                day: sifr_generated_field_value_ca8d3918f4578f1d_646179,
                hour: sifr_generated_field_value_407efecc7eb5764f_686f7572,
                minute: sifr_generated_field_value_5bb2f9bdf2fad1e9_6d696e757465,
                second: sifr_generated_field_value_a49985ef4cee20bd_7365636f6e64,
                microsecond: sifr_generated_field_value_27f934ab879dcfa3_6d6963726f7365636f6e64,
                tz_offset: sifr_generated_field_value_17964c5d1d2f9a66_5f747a5f6f6666736574,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2edatetimeX2edatetime {
        #[must_use]
        #[expect(
            clippy::too_many_lines,
            reason = "one generated Rust function preserves one typed Sifr function"
        )]
        pub fn isoformat(&self) -> String {
            let y: String = self.year.clone().to_string();
            let mut mo: String = self.month.clone().to_string();
            if &SifrInt::from(mo.chars().count()) < &SifrInt::from_i64(2) {
                mo = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize + mo.len());
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(mo.as_str());
                    sifr_generated_concat
                };
            }
            let mut d: String = self.day.clone().to_string();
            if &SifrInt::from(d.chars().count()) < &SifrInt::from_i64(2) {
                d = {
                    let mut sifr_generated_concat: String = String::with_capacity(1usize + d.len());
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(d.as_str());
                    sifr_generated_concat
                };
            }
            let mut h: String = self.hour.clone().to_string();
            if &SifrInt::from(h.chars().count()) < &SifrInt::from_i64(2) {
                h = {
                    let mut sifr_generated_concat: String = String::with_capacity(1usize + h.len());
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(h.as_str());
                    sifr_generated_concat
                };
            }
            let mut mi: String = self.minute.clone().to_string();
            if &SifrInt::from(mi.chars().count()) < &SifrInt::from_i64(2) {
                mi = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize + mi.len());
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(mi.as_str());
                    sifr_generated_concat
                };
            }
            let mut s: String = self.second.clone().to_string();
            if &SifrInt::from(s.chars().count()) < &SifrInt::from_i64(2) {
                s = {
                    let mut sifr_generated_concat: String = String::with_capacity(1usize + s.len());
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(s.as_str());
                    sifr_generated_concat
                };
            }
            let mut base: String = {
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
            };
            if &self.microsecond.clone() != &SifrInt::from_i64(0) {
                base.push('.');
                base.push_str(sifr_generated_six_digits(self.microsecond.clone()).as_str());
            }
            let tz_offset_opt: Option<SifrInt> = self.tz_offset.clone();
            let Some(tz_offset_opt_value_af7a59df393dc871) = tz_offset_opt.clone() else {
                return base;
            };
            let offset: SifrInt = tz_offset_opt_value_af7a59df393dc871.clone();
            let mut sign: String = "+".to_string();
            let mut abs_offset: SifrInt = offset.clone();
            if &abs_offset < &SifrInt::from_i64(0) {
                sign = "-".to_string();
                abs_offset = -&abs_offset;
            }
            let h_off: SifrInt = abs_offset.floor_div_known_nonzero(&SifrInt::from_i64(3600));
            let m_off_value_ecbb7903406895aa: SifrInt = abs_offset
                .floor_mod_known_nonzero(&SifrInt::from_i64(3600))
                .floor_div_known_nonzero(&SifrInt::from_i64(60));
            let mut hs_off_value_cdfc32c6642466ee: String = h_off.to_string();
            if &SifrInt::from(hs_off_value_cdfc32c6642466ee.chars().count()) < &SifrInt::from_i64(2)
            {
                hs_off_value_cdfc32c6642466ee = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize + hs_off_value_cdfc32c6642466ee.len());
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(hs_off_value_cdfc32c6642466ee.as_str());
                    sifr_generated_concat
                };
            }
            let mut ms_off_value_f9e2b676f4ffcfe7: String =
                m_off_value_ecbb7903406895aa.to_string();
            if &SifrInt::from(ms_off_value_f9e2b676f4ffcfe7.chars().count()) < &SifrInt::from_i64(2)
            {
                ms_off_value_f9e2b676f4ffcfe7 = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize + ms_off_value_f9e2b676f4ffcfe7.len());
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(ms_off_value_f9e2b676f4ffcfe7.as_str());
                    sifr_generated_concat
                };
            }
            {
                let mut sifr_generated_concat: String = String::with_capacity(
                    base.len()
                        + sign.len()
                        + hs_off_value_cdfc32c6642466ee.len()
                        + 1usize
                        + ms_off_value_f9e2b676f4ffcfe7.len(),
                );
                sifr_generated_concat.push_str(base.as_str());
                sifr_generated_concat.push_str(sign.as_str());
                sifr_generated_concat.push_str(hs_off_value_cdfc32c6642466ee.as_str());
                sifr_generated_concat.push(':');
                sifr_generated_concat.push_str(ms_off_value_f9e2b676f4ffcfe7.as_str());
                sifr_generated_concat
            }
        }
    }
    impl PartialEq for SifrGeneratedStdlibSifrX2edatetimeX2edatetime {
        fn eq(&self, other: &SifrGeneratedStdlibSifrX2edatetimeX2edatetime) -> bool {
            let same_tz: bool = self.tz_offset == other.tz_offset;
            self.year.clone() == other.year.clone()
                && self.month.clone() == other.month.clone()
                && self.day.clone() == other.day.clone()
                && self.hour.clone() == other.hour.clone()
                && self.minute.clone() == other.minute.clone()
                && self.second.clone() == other.second.clone()
                && self.microsecond.clone() == other.microsecond.clone()
                && same_tz
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2edatetimeX2edatetime {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.isoformat())
        }
    }
    #[derive(Debug, Clone)]
    pub struct SifrGeneratedStdlibSifrX2edatetimeX2etime {
        pub hour: SifrInt,
        pub minute: SifrInt,
        pub second: SifrInt,
        pub microsecond: SifrInt,
        pub tz_offset: Option<SifrInt>,
    }
    impl SifrGeneratedStdlibSifrX2edatetimeX2etime {
        #[must_use]
        pub const fn new(
            hour: SifrInt,
            minute: SifrInt,
            second: SifrInt,
            microsecond: SifrInt,
            sifr_generated_tz_offset: Option<SifrInt>,
        ) -> Self {
            Self {
                hour,
                minute,
                second,
                microsecond,
                tz_offset: sifr_generated_tz_offset,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2edatetimeX2etime {
        #[must_use]
        pub fn isoformat(&self) -> String {
            let mut h: String = self.hour.clone().to_string();
            if &SifrInt::from(h.chars().count()) < &SifrInt::from_i64(2) {
                h = {
                    let mut sifr_generated_concat: String = String::with_capacity(1usize + h.len());
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(h.as_str());
                    sifr_generated_concat
                };
            }
            let mut mi: String = self.minute.clone().to_string();
            if &SifrInt::from(mi.chars().count()) < &SifrInt::from_i64(2) {
                mi = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize + mi.len());
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(mi.as_str());
                    sifr_generated_concat
                };
            }
            let mut s: String = self.second.clone().to_string();
            if &SifrInt::from(s.chars().count()) < &SifrInt::from_i64(2) {
                s = {
                    let mut sifr_generated_concat: String = String::with_capacity(1usize + s.len());
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(s.as_str());
                    sifr_generated_concat
                };
            }
            let mut rendered: String = {
                let mut sifr_generated_concat: String =
                    String::with_capacity(h.len() + 1usize + mi.len() + 1usize + s.len());
                sifr_generated_concat.push_str(h.as_str());
                sifr_generated_concat.push(':');
                sifr_generated_concat.push_str(mi.as_str());
                sifr_generated_concat.push(':');
                sifr_generated_concat.push_str(s.as_str());
                sifr_generated_concat
            };
            if &self.microsecond.clone() != &SifrInt::from_i64(0) {
                rendered.push('.');
                rendered.push_str(sifr_generated_six_digits(self.microsecond.clone()).as_str());
            }
            let tz_offset_opt: Option<SifrInt> = self.tz_offset.clone();
            let Some(tz_offset_opt_value_af7a59df393dc871) = tz_offset_opt.clone() else {
                return rendered;
            };
            {
                let mut sifr_generated_concat: String = String::with_capacity(rendered.len());
                sifr_generated_concat.push_str(rendered.as_str());
                sifr_generated_concat.push_str(
                    SifrGeneratedStdlibSifrX2edatetimeX2etimezone::new(
                        tz_offset_opt_value_af7a59df393dc871.clone(),
                    )
                    .iso_suffix()
                    .as_str(),
                );
                sifr_generated_concat
            }
        }
    }
    impl PartialEq for SifrGeneratedStdlibSifrX2edatetimeX2etime {
        fn eq(&self, other: &SifrGeneratedStdlibSifrX2edatetimeX2etime) -> bool {
            self.hour.clone() == other.hour.clone()
                && self.minute.clone() == other.minute.clone()
                && self.second.clone() == other.second.clone()
                && self.microsecond.clone() == other.microsecond.clone()
                && self.tz_offset.clone() == other.tz_offset.clone()
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2edatetimeX2etime {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.isoformat())
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
        pub fn sifr_generated_allows(&self, level_num: &SifrInt) -> bool {
            if &self.level.clone() == &sifr_generated_const_4e4f54534554() {
                return true;
            }
            level_num >= &self.level
        }
    }
    impl SifrGeneratedStdlibSifrX2eloggingX2eFileHandler {
        pub fn emit(&self, level: &str, name: &str, msg: &str) {
            let level_num: SifrInt = sifr_generated_level_name_to_num(level);
            if !self.sifr_generated_allows(&level_num) {
                return;
            }
            let line: String = {
                let mut sifr_generated_concat: String = String::with_capacity(1usize);
                sifr_generated_concat.push_str(self.formatter.format(level, name, msg).as_str());
                sifr_generated_concat.push('\n');
                sifr_generated_concat
            };
            let sifr_generated_try_res: Result<(), IOError> = (|| {
                let mut fh: SifrGeneratedIoTextFileHandle =
                    open_text(&self.path, &"a".to_string(), &Some(utf8().clone()), &None)?;
                let sifr_generated_try_res: Result<(), IOError> = (|| {
                    fh.write(&line)?;
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
    impl SifrGeneratedStdlibSifrX2eloggingX2eLogger {
        pub fn warning(&self, msg: &str) {
            self.sifr_generated_emit(
                &"WARNING".to_string(),
                &sifr_generated_const_5741524e494e47(),
                msg,
            );
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2epathlibX2ePath {
        pub path: String,
    }
    impl SifrGeneratedStdlibSifrX2epathlibX2ePath {
        #[must_use]
        pub const fn new(path: String) -> Self {
            let sifr_generated_field_value_0e74a76ec4f48c05_5f70617468: String = path;
            Self {
                path: sifr_generated_field_value_0e74a76ec4f48c05_5f70617468,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2epathlibX2ePath {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn glob(&self, pattern: &str) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
            sifr_generated_glob_to_iter(&self.path, pattern)
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2epathlibX2ePath {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "Path(_path={})", self.path)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct SifrGeneratedStdlibSifrX2erandomX2eRandomState {
        pub version: SifrInt,
        pub state_words: Vec<SifrInt>,
        pub index: SifrInt,
        pub gauss_next: Option<f64>,
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandomState {
        #[must_use]
        pub fn new(
            version: SifrInt,
            state_words: Vec<SifrInt>,
            index: SifrInt,
            gauss_next: Option<f64>,
        ) -> Self {
            let sifr_generated_field_value_bb62c62c9808ea37_76657273696f6e: SifrInt =
                version.clone();
            let sifr_generated_field_value_8e62ac2dd7162e8c_73746174655f776f726473: Vec<SifrInt> =
                state_words;
            let sifr_generated_field_value_83cf8e8f9081468b_696e646578: SifrInt = index.clone();
            let sifr_generated_field_value_edec7000e7b3eeaa_67617573735f6e657874: Option<f64> =
                gauss_next;
            Self {
                version: sifr_generated_field_value_bb62c62c9808ea37_76657273696f6e,
                state_words: sifr_generated_field_value_8e62ac2dd7162e8c_73746174655f776f726473,
                index: sifr_generated_field_value_83cf8e8f9081468b_696e646578,
                gauss_next: sifr_generated_field_value_edec7000e7b3eeaa_67617573735f6e657874,
            }
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct SifrGeneratedStdlibSifrX2erandomX2eRandom {
        pub state_words: Vec<SifrInt>,
        pub index: SifrInt,
        pub gauss_next: Option<f64>,
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        #[must_use]
        pub fn new(seed_value: Option<SifrInt>) -> Self {
            let normalized_seed: SifrInt = sifr_generated_normalize_seed_input(seed_value.clone());
            let sifr_generated_field_value_7e372b502c45daad_5f73746174655f776f726473: Vec<SifrInt> =
                sifr_generated_seed_words_from_seed(normalized_seed.clone());
            let sifr_generated_field_value_497043933c8a2d12_5f696e646578: SifrInt =
                sifr_generated_const_5f4d545f4e().clone();
            let sifr_generated_field_value_88c1b3a412b57c41_5f67617573735f6e657874: Option<f64> =
                None;
            Self {
                state_words: sifr_generated_field_value_7e372b502c45daad_5f73746174655f776f726473,
                index: sifr_generated_field_value_497043933c8a2d12_5f696e646578,
                gauss_next: sifr_generated_field_value_88c1b3a412b57c41_5f67617573735f6e657874,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        pub fn sifr_generated_twist(&mut self) {
            let mut i: SifrInt = SifrInt::from_i64(0);
            while &SifrInt::from_i64(0) <= &i && &i < &SifrInt::from(self.state_words.len()) {
                let y: SifrInt = &(&sifr_generated_state_word_at(&self.state_words, i.clone())
                    & &sifr_generated_const_5f4d545f55505045525f4d41534b())
                    + &(&sifr_generated_state_word_at(
                        &self.state_words,
                        (&i + &SifrInt::from_i64(1))
                            .floor_mod_known_nonzero(&sifr_generated_const_5f4d545f4e()),
                    ) & &sifr_generated_const_5f4d545f4c4f5745525f4d41534b());
                let mut x_a: SifrInt = y.floor_div_known_nonzero(&SifrInt::from_i64(2));
                if &y.floor_mod_known_nonzero(&SifrInt::from_i64(2)) != &SifrInt::from_i64(0) {
                    x_a = &x_a ^ &sifr_generated_const_5f4d545f4d41545249585f41();
                }
                let new_word: SifrInt = &sifr_generated_state_word_at(
                    &self.state_words,
                    (&i + &sifr_generated_const_5f4d545f4d())
                        .floor_mod_known_nonzero(&sifr_generated_const_5f4d545f4e()),
                ) ^ &x_a;
                {
                    let sifr_generated_assign_value =
                        &new_word & &sifr_generated_const_5f4d545f574f52445f4d41534b();
                    {
                        let sifr_generated_index_raw = i.clone();
                        let sifr_generated_index_normalized =
                            sifr_generated_index_raw.normalize_index_or_len(self.state_words.len());
                        if let Some(sifr_generated_elem) =
                            self.state_words.get_mut(sifr_generated_index_normalized)
                        {
                            *sifr_generated_elem = sifr_generated_assign_value;
                        }
                    }
                }
                i = &i + &SifrInt::from_i64(1);
            }
            self.index = SifrInt::from_i64(0);
        }
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        #[must_use]
        pub fn sifr_generated_next_u32(&mut self) -> SifrInt {
            if &self.index.clone() >= &sifr_generated_const_5f4d545f4e() {
                self.sifr_generated_twist();
            }
            let mut y: SifrInt =
                sifr_generated_state_word_at(&self.state_words, self.index.clone());
            self.index = &self.index.clone() + &SifrInt::from_i64(1);
            y = &y ^ &y.floor_div_known_nonzero(&SifrInt::from_i64(2048));
            y = &y ^ &(&(&y * &SifrInt::from_i64(128)) & &SifrInt::from_i64(2_636_928_640));
            y = &y ^ &(&(&y * &SifrInt::from_i64(32768)) & &SifrInt::from_i64(4_022_730_752));
            y = &y ^ &y.floor_div_known_nonzero(&SifrInt::from_i64(262_144));
            &y & &sifr_generated_const_5f4d545f574f52445f4d41534b()
        }
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        #[must_use]
        pub fn getstate(&self) -> SifrGeneratedStdlibSifrX2erandomX2eRandomState {
            SifrGeneratedStdlibSifrX2erandomX2eRandomState::new(
                SifrInt::from_i64(3),
                sifr_generated_clone_words(&self.state_words),
                self.index.clone(),
                self.gauss_next,
            )
        }
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn setstate(
            &mut self,
            state: &SifrGeneratedStdlibSifrX2erandomX2eRandomState,
        ) -> Result<(), ValueError> {
            if &state.version.clone() != &SifrInt::from_i64(3) {
                return Err(ValueError::new("setstate: unsupported version".to_string()));
            }
            if &SifrInt::from(state.state_words.len()) != &sifr_generated_const_5f4d545f4e() {
                return Err(ValueError::new(
                    "setstate: state_words must have length 624".to_string(),
                ));
            }
            if &state.index.clone() < &SifrInt::from_i64(0)
                || &state.index.clone() > &sifr_generated_const_5f4d545f4e()
            {
                return Err(ValueError::new(
                    "setstate: index must be in range [0, 624]".to_string(),
                ));
            }
            let mut normalized: Vec<SifrInt> = Vec::new();
            for word in state.state_words.iter().cloned() {
                if &word < &SifrInt::from_i64(0)
                    || &word > &sifr_generated_const_5f4d545f574f52445f4d41534b()
                {
                    return Err(ValueError::new("setstate: word out of range".to_string()));
                }
                normalized.push(&word & &sifr_generated_const_5f4d545f574f52445f4d41534b());
            }
            self.state_words = normalized;
            self.index = state.index.clone();
            self.gauss_next = state.gauss_next;
            Ok(())
        }
    }
    pub type SifrGeneratedStdlibSifrX2eregexX2eCompiledPattern =
        ::sifr_runtime::interop::Handle<::sifr_stdlib::regex::CompiledPattern>;
    impl SifrGeneratedOpaqueSifrStdlibSifrX2eregexX2eCompiledPatternMethods
        for SifrGeneratedStdlibSifrX2eregexX2eCompiledPattern
    {
        fn search(&self, text: &str) -> Result<Option<String>, RegexError> {
            ::sifr_stdlib::regex::compiled_pattern_search(self, text).map_err(
                |sifr_generated_bridge_error| RegexError {
                    message: sifr_generated_bridge_error.to_string(),
                    detail: sifr_generated_bridge_error.to_string(),
                },
            )
        }
        fn is_match(&self, text: &str) -> Result<bool, RegexError> {
            ::sifr_stdlib::regex::compiled_pattern_is_match(self, text).map_err(
                |sifr_generated_bridge_error| RegexError {
                    message: sifr_generated_bridge_error.to_string(),
                    detail: sifr_generated_bridge_error.to_string(),
                },
            )
        }
        fn sub(&self, replacement: &str, text: &str) -> Result<String, RegexError> {
            ::sifr_stdlib::regex::compiled_pattern_replace(self, replacement, text).map_err(
                |sifr_generated_bridge_error| RegexError {
                    message: sifr_generated_bridge_error.to_string(),
                    detail: sifr_generated_bridge_error.to_string(),
                },
            )
        }
        fn findall(&self, text: &str) -> Result<Vec<String>, RegexError> {
            ::sifr_stdlib::regex::compiled_pattern_findall(self, text).map_err(
                |sifr_generated_bridge_error| RegexError {
                    message: sifr_generated_bridge_error.to_string(),
                    detail: sifr_generated_bridge_error.to_string(),
                },
            )
        }
        fn split(&self, text: &str) -> Result<Vec<String>, RegexError> {
            ::sifr_stdlib::regex::compiled_pattern_split(self, text).map_err(
                |sifr_generated_bridge_error| RegexError {
                    message: sifr_generated_bridge_error.to_string(),
                    detail: sifr_generated_bridge_error.to_string(),
                },
            )
        }
        fn pattern(&self) -> Result<String, RegexError> {
            ::sifr_stdlib::regex::compiled_pattern_source(self).map_err(
                |sifr_generated_bridge_error| RegexError {
                    message: sifr_generated_bridge_error.to_string(),
                    detail: sifr_generated_bridge_error.to_string(),
                },
            )
        }
        fn flags(&self) -> Result<SifrInt, RegexError> {
            ::sifr_stdlib::regex::compiled_pattern_flags(self)
                .map(::sifr_runtime::interop::SifrIntBridge::into_sifr_int)
                .map_err(|sifr_generated_bridge_error| RegexError {
                    message: sifr_generated_bridge_error.to_string(),
                    detail: sifr_generated_bridge_error.to_string(),
                })
        }
    }
    pub struct SifrGeneratedStdlibSifrX2ereX2ePattern {
        pub compiled: SifrGeneratedStdlibSifrX2eregexX2eCompiledPattern,
        pub pattern: String,
        pub flags: SifrInt,
    }
    impl SifrGeneratedStdlibSifrX2ereX2ePattern {
        #[must_use]
        pub fn new(
            compiled: SifrGeneratedStdlibSifrX2eregexX2eCompiledPattern,
            pattern: String,
            flags: SifrInt,
        ) -> Self {
            let sifr_generated_field_value_fc19778909466d91_5f636f6d70696c6564: SifrGeneratedStdlibSifrX2eregexX2eCompiledPattern = compiled;
            let sifr_generated_field_value_24bd37eb3fd1d0fc_5f7061747465726e: String = pattern;
            let sifr_generated_field_value_7e89da5111942f49_5f666c616773: SifrInt = flags.clone();
            Self {
                compiled: sifr_generated_field_value_fc19778909466d91_5f636f6d70696c6564,
                pattern: sifr_generated_field_value_24bd37eb3fd1d0fc_5f7061747465726e,
                flags: sifr_generated_field_value_7e89da5111942f49_5f666c616773,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2ereX2ePattern {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn search(&self, text: &str) -> Result<Option<String>, RegexError> {
            self.compiled.search(text)
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
    pub struct RegexError {
        pub message: String,
        pub detail: String,
    }
    impl RegexError {
        #[must_use]
        pub const fn new(message: String) -> Self {
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
}
use crate::sifr_generated_generated_support::*;
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::FloatOverflowError;
pub use sifr_generated_project_nominals::FloatPrecisionLossError;
pub use sifr_generated_project_nominals::IOError;
pub use sifr_generated_project_nominals::ParseError;
pub use sifr_generated_project_nominals::RegexError;
pub use sifr_generated_project_nominals::SifrGeneratedIoBinaryFileHandle;
pub use sifr_generated_project_nominals::SifrGeneratedIoTextFileHandle;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ecsvX2eDialect;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ecsvX2ereader;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2edatetimeX2edatetime;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2edatetimeX2etime;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2edatetimeX2etimezone;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eencodingX2eDecodeError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eencodingX2eDecodeOutcome;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eencodingX2eEncodeError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eencodingX2eEncodeErrorHandler;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eencodingX2eEncodeOutcome;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eencodingX2eEncoding;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eloggingX2eFileHandler;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eloggingX2eLogger;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2epathlibX2ePath;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2erandomX2eRandom;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2erandomX2eRandomState;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ereX2ePattern;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eregexX2eCompiledPattern;
pub use sifr_generated_project_nominals::ValueError;
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    let path: String = "/tmp/sifr_demo_remediation.txt".to_string();
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let mut f: SifrGeneratedIoTextFileHandle = (|| {
            let sifr_generated_path = path.to_string();
            let sifr_generated_mode = "w".to_string();
            let sifr_generated_encoding = "utf-8".to_string();
            let sifr_generated_errors = "strict".to_string();
            let sifr_generated_binary_mode = match sifr_generated_mode.as_str() {
                "r" | "rt" => "rb".to_string(),
                "w" | "wt" => "wb".to_string(),
                "a" | "at" => "ab".to_string(),
                _ => {
                    return Err(IOError {
                        message: format!("invalid mode: {sifr_generated_mode}"),
                        kind: "Other".to_string(),
                    });
                }
            };
            let sifr_generated_handle_id = ::sifr_stdlib::fs::open_file(
                sifr_generated_path.as_str(),
                sifr_generated_binary_mode.as_str(),
            )
            .map_err(sifr_generated_io_err)?;
            Ok::<SifrGeneratedIoTextFileHandle, IOError>(SifrGeneratedIoTextFileHandle::new(
                SifrGeneratedIoBinaryFileHandle::new(
                    SifrGeneratedIoNativeFileHandle::new(sifr_generated_handle_id),
                    sifr_generated_binary_mode.to_string(),
                ),
                SifrGeneratedStdlibSifrX2eencodingX2eEncoding::new(sifr_generated_encoding),
                SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler::new(
                    sifr_generated_errors.clone(),
                ),
                SifrGeneratedStdlibSifrX2eencodingX2eEncodeErrorHandler::new(sifr_generated_errors),
            ))
        })()?;
        (&mut f).write(&"hello from open()\n".to_string())?;
        (&mut f).write(&"second line\n".to_string())?;
        (&mut f).close();
        let content: String = read_text(&path)?;
        let _chars_content: Vec<char> = content.chars().collect::<Vec<char>>();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(16usize);
            sifr_generated_concat.push_str("open write ok = ");
            sifr_generated_concat.push_str(
                (SifrInt::from(content.chars().count()) > SifrInt::from_i64(0))
                    .to_string()
                    .as_str(),
            );
            sifr_generated_concat
        });
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(18usize);
            sifr_generated_concat.push_str("open write error: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    let path2_value_3f11d2a7be5fa58c: String = "/tmp/sifr_demo_ctx.txt".to_string();
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        {
            struct SifrGeneratedWithGuard0 {
                ctx: SifrGeneratedIoTextFileHandle,
            }
            impl Drop for SifrGeneratedWithGuard0 {
                fn drop(&mut self) {
                    self.ctx.sifr_generated_exit__();
                }
            }
            let sifr_generated_ctx_0 = (|| {
                let sifr_generated_path = path2_value_3f11d2a7be5fa58c.to_string();
                let sifr_generated_mode = "w".to_string();
                let sifr_generated_encoding = "utf-8".to_string();
                let sifr_generated_errors = "strict".to_string();
                let sifr_generated_binary_mode = match sifr_generated_mode.as_str() {
                    "r" | "rt" => "rb".to_string(),
                    "w" | "wt" => "wb".to_string(),
                    "a" | "at" => "ab".to_string(),
                    _ => {
                        return Err(IOError {
                            message: format!("invalid mode: {sifr_generated_mode}"),
                            kind: "Other".to_string(),
                        });
                    }
                };
                let sifr_generated_handle_id = ::sifr_stdlib::fs::open_file(
                    sifr_generated_path.as_str(),
                    sifr_generated_binary_mode.as_str(),
                )
                .map_err(sifr_generated_io_err)?;
                Ok::<SifrGeneratedIoTextFileHandle, IOError>(SifrGeneratedIoTextFileHandle::new(
                    SifrGeneratedIoBinaryFileHandle::new(
                        SifrGeneratedIoNativeFileHandle::new(sifr_generated_handle_id),
                        sifr_generated_binary_mode.to_string(),
                    ),
                    SifrGeneratedStdlibSifrX2eencodingX2eEncoding::new(sifr_generated_encoding),
                    SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler::new(
                        sifr_generated_errors.clone(),
                    ),
                    SifrGeneratedStdlibSifrX2eencodingX2eEncodeErrorHandler::new(
                        sifr_generated_errors,
                    ),
                ))
            })()?;
            let sifr_generated_guard_0 = SifrGeneratedWithGuard0 {
                ctx: sifr_generated_ctx_0,
            };
            let mut fw = sifr_generated_guard_0.ctx.sifr_generated_enter__();
            (&mut fw).write(&"context manager works".to_string())?;
        }
        let result: String = read_text(&path2_value_3f11d2a7be5fa58c)?;
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(21usize);
            sifr_generated_concat.push_str("context manager ok = ");
            sifr_generated_concat
                .push_str((result == "context manager works").to_string().as_str());
            sifr_generated_concat
        });
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(23usize);
            sifr_generated_concat.push_str("context manager error: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let mut fr: SifrGeneratedIoTextFileHandle = (|| {
            let sifr_generated_path = path.to_string();
            let sifr_generated_mode = "r".to_string();
            let sifr_generated_encoding = "utf-8".to_string();
            let sifr_generated_errors = "strict".to_string();
            let sifr_generated_binary_mode = match sifr_generated_mode.as_str() {
                "r" | "rt" => "rb".to_string(),
                "w" | "wt" => "wb".to_string(),
                "a" | "at" => "ab".to_string(),
                _ => {
                    return Err(IOError {
                        message: format!("invalid mode: {sifr_generated_mode}"),
                        kind: "Other".to_string(),
                    });
                }
            };
            let sifr_generated_handle_id = ::sifr_stdlib::fs::open_file(
                sifr_generated_path.as_str(),
                sifr_generated_binary_mode.as_str(),
            )
            .map_err(sifr_generated_io_err)?;
            Ok::<SifrGeneratedIoTextFileHandle, IOError>(SifrGeneratedIoTextFileHandle::new(
                SifrGeneratedIoBinaryFileHandle::new(
                    SifrGeneratedIoNativeFileHandle::new(sifr_generated_handle_id),
                    sifr_generated_binary_mode.to_string(),
                ),
                SifrGeneratedStdlibSifrX2eencodingX2eEncoding::new(sifr_generated_encoding),
                SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler::new(
                    sifr_generated_errors.clone(),
                ),
                SifrGeneratedStdlibSifrX2eencodingX2eEncodeErrorHandler::new(sifr_generated_errors),
            ))
        })()?;
        let content2_value_ee7eb2d0c2b58110: String = fr.read()?;
        let _chars_content2_value_6aa5e9a973436389: Vec<char> = content2_value_ee7eb2d0c2b58110
            .chars()
            .collect::<Vec<char>>();
        (&mut fr).close();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(15usize);
            sifr_generated_concat.push_str("open read ok = ");
            sifr_generated_concat.push_str(
                (SifrInt::from(content2_value_ee7eb2d0c2b58110.chars().count())
                    > SifrInt::from_i64(0))
                .to_string()
                .as_str(),
            );
            sifr_generated_concat
        });
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(17usize);
            sifr_generated_concat.push_str("open read error: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    let t: SifrGeneratedStdlibSifrX2edatetimeX2etime =
        SifrGeneratedStdlibSifrX2edatetimeX2etime::new(
            SifrInt::from_i64(10),
            SifrInt::from_i64(30),
            SifrInt::from_i64(45),
            SifrInt::from_i64(0),
            None,
        );
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(17usize);
        sifr_generated_concat.push_str("time isoformat = ");
        sifr_generated_concat.push_str(t.isoformat().as_str());
        sifr_generated_concat
    });
    let t2: SifrGeneratedStdlibSifrX2edatetimeX2etime =
        SifrGeneratedStdlibSifrX2edatetimeX2etime::new(
            SifrInt::from_i64(10),
            SifrInt::from_i64(30),
            SifrInt::from_i64(45),
            SifrInt::from_i64(0),
            None,
        );
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(10usize);
        sifr_generated_concat.push_str("time eq = ");
        sifr_generated_concat.push_str((t == t2).to_string().as_str());
        sifr_generated_concat
    });
    let tz: SifrGeneratedStdlibSifrX2edatetimeX2etimezone =
        SifrGeneratedStdlibSifrX2edatetimeX2etimezone::new(SifrInt::from_i64(0));
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(15usize);
        sifr_generated_concat.push_str("timezone utc = ");
        sifr_generated_concat.push_str(tz.to_string().as_str());
        sifr_generated_concat
    });
    let dt: SifrGeneratedStdlibSifrX2edatetimeX2edatetime = now(&None);
    let iso: String = dt.isoformat();
    let _chars_iso: Vec<char> = iso.chars().collect::<Vec<char>>();
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(19usize);
        sifr_generated_concat.push_str("now isoformat ok = ");
        sifr_generated_concat.push_str(
            (&SifrInt::from(iso.chars().count()) > &SifrInt::from_i64(0))
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    let tmp: SifrGeneratedStdlibSifrX2epathlibX2ePath =
        SifrGeneratedStdlibSifrX2epathlibX2ePath::new("/tmp".to_string());
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let matches_it: Box<dyn Iterator<Item = String>> = tmp.glob(&"sifr_demo_*".to_string())?;
        let matches: Vec<String> = matches_it.collect::<Vec<_>>();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(13usize);
            sifr_generated_concat.push_str("glob found = ");
            sifr_generated_concat.push_str(
                (SifrInt::from(matches.len()) > SifrInt::from_i64(0))
                    .to_string()
                    .as_str(),
            );
            sifr_generated_concat
        });
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(12usize);
            sifr_generated_concat.push_str("glob error: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    let sifr_generated_try_res: Result<(), RegexError> = (|| {
        let found: Option<String> = search_flags(
            &"hello".to_string(),
            &"HELLO WORLD".to_string(),
            sifr_generated_const_49474e4f524543415345(),
        )?;
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(16usize);
            sifr_generated_concat.push_str("re ignorecase = ");
            sifr_generated_concat.push_str(found.is_some().to_string().as_str());
            sifr_generated_concat
        });
        let pat: SifrGeneratedStdlibSifrX2ereX2ePattern = compile_flags(
            &"^line".to_string(),
            sifr_generated_const_4d554c54494c494e45(),
        )?;
        let found2_value_09082fc57b2e8657: Option<String> =
            pat.search(&"line1\nline2".to_string())?;
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(15usize);
            sifr_generated_concat.push_str("re multiline = ");
            sifr_generated_concat
                .push_str(found2_value_09082fc57b2e8657.is_some().to_string().as_str());
            sifr_generated_concat
        });
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(10usize);
            sifr_generated_concat.push_str("re error: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let cwd: String = getcwd()?;
        let _chars_cwd: Vec<char> = cwd.chars().collect::<Vec<char>>();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(15usize);
            sifr_generated_concat.push_str("os getcwd ok = ");
            sifr_generated_concat.push_str(
                (SifrInt::from(cwd.chars().count()) > SifrInt::from_i64(0))
                    .to_string()
                    .as_str(),
            );
            sifr_generated_concat
        });
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(17usize);
            sifr_generated_concat.push_str("os getcwd error: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    let items: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
        SifrInt::from_i64(5),
    ];
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let picked: SifrInt = choice(&items)?;
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(19usize);
            sifr_generated_concat.push_str("random choice ok = ");
            sifr_generated_concat.push_str(
                (&picked >= &SifrInt::from_i64(1) && &picked <= &SifrInt::from_i64(5))
                    .to_string()
                    .as_str(),
            );
            sifr_generated_concat
        });
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(21usize);
            sifr_generated_concat.push_str("random choice error: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    let root: SifrGeneratedStdlibSifrX2eloggingX2eLogger =
        basicConfig(sifr_generated_const_5741524e494e47());
    root.info(&"should not print".to_string());
    root.warning(&"root warning visible".to_string());
    let logger2: SifrGeneratedStdlibSifrX2eloggingX2eLogger = getLogger(&"myapp".to_string());
    logger2.info(&"should not print either".to_string());
    logger2.warning(&"myapp warning visible".to_string());
    println!("basicConfig global level ok");
    let handler: SifrGeneratedStdlibSifrX2eloggingX2eFileHandler =
        SifrGeneratedStdlibSifrX2eloggingX2eFileHandler::new(
            "/tmp/sifr_demo_fh_log.txt".to_string(),
            SifrInt::from_i64(0),
        );
    handler.emit(
        &"INFO".to_string(),
        &"demo".to_string(),
        &"file handler test".to_string(),
    );
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let log_content: String = read_text(&"/tmp/sifr_demo_fh_log.txt".to_string())?;
        let _chars_log_content: Vec<char> = log_content.chars().collect::<Vec<char>>();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(24usize);
            sifr_generated_concat.push_str("file handler wrote ok = ");
            sifr_generated_concat.push_str(
                (SifrInt::from(log_content.chars().count()) > SifrInt::from_i64(0))
                    .to_string()
                    .as_str(),
            );
            sifr_generated_concat
        });
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(20usize);
            sifr_generated_concat.push_str("file handler error: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    let csv_path: String = "/tmp/sifr_demo_csv.csv".to_string();
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        write_text(&csv_path, &"name,age\nalice,30\nbob,25".to_string())?;
        let r: SifrGeneratedStdlibSifrX2ecsvX2ereader = reader_from_path(
            &csv_path,
            &None,
            &",".to_string(),
            &"\"".to_string(),
            &String::new(),
            true,
            false,
            SifrInt::from_i64(0),
        )?;
        let rows: Vec<Vec<String>> = r.rows();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(28usize);
            sifr_generated_concat.push_str("csv reader_from_path rows = ");
            sifr_generated_concat.push_str(SifrInt::from(rows.len()).to_string().as_str());
            sifr_generated_concat
        });
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(11usize);
            sifr_generated_concat.push_str("csv error: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
}
