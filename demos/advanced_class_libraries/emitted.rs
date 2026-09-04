// src/main.rs
pub mod sifr_generated_generated_support {
    use crate::{
        IOError, ParseError, RegexError, SifrGeneratedIoBinaryFileHandle,
        SifrGeneratedIoNativeFileHandle, SifrGeneratedIoTextFileHandle,
        SifrGeneratedStdlibSifrX2ecsvX2eDialect,
        SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler,
        SifrGeneratedStdlibSifrX2eencodingX2eEncodeError,
        SifrGeneratedStdlibSifrX2eencodingX2eEncodeErrorHandler,
        SifrGeneratedStdlibSifrX2eencodingX2eEncodeOutcome,
        SifrGeneratedStdlibSifrX2eencodingX2eEncoding, SifrGeneratedStdlibSifrX2eloggingX2eLogger,
        SifrGeneratedStdlibSifrX2ereX2ePattern, SifrGeneratedStdlibSifrX2eregexX2eCompiledPattern,
    };
    pub(super) use ::sifr_runtime::SifrInt;
    pub(super) use ::std::collections::HashMap;
    pub(super) fn sifr_generated_encoding_encode_bytes_impl(
        text: &str,
        encoding: &str,
        errors: &str,
    ) -> Result<Vec<u8>, ParseError> {
        ::sifr_stdlib::encoding::encoding_encode_bytes(text, encoding, errors).map_err(
            |sifr_generated_bridge_error| ParseError {
                message: sifr_generated_bridge_error,
            },
        )
    }
    pub(super) fn sifr_generated_encoding_encode_recoveries_impl(
        text: &str,
        encoding: &str,
        errors: &str,
    ) -> Result<Vec<String>, ParseError> {
        ::sifr_stdlib::encoding::encoding_encode_recoveries(text, encoding, errors).map_err(
            |sifr_generated_bridge_error| ParseError {
                message: sifr_generated_bridge_error,
            },
        )
    }
    pub(super) fn exists(path: &str) -> bool {
        ::sifr_stdlib::fs::exists(path)
    }
    pub(super) fn sifr_generated_open_file(path: &str, mode: &str) -> Result<String, IOError> {
        ::sifr_stdlib::fs::open_file(path, mode).map_err(sifr_generated_io_err)
    }
    pub(super) fn sifr_generated_file_close(handle: &str) {
        ::sifr_stdlib::fs::file_close(handle);
    }
    pub(super) fn sifr_generated_file_write_bytes(
        handle: &str,
        data: &[u8],
    ) -> Result<(), IOError> {
        ::sifr_stdlib::fs::file_write_bytes(handle, data).map_err(sifr_generated_io_err)
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
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err;
            Err(e)
        })
    }
    pub(super) fn file_close(handle: &SifrGeneratedIoNativeFileHandle) {
        sifr_generated_file_close(&handle.id.clone());
    }
    pub(super) fn file_write_bytes(
        handle: &SifrGeneratedIoNativeFileHandle,
        data: &[u8],
    ) -> Result<(), IOError> {
        sifr_generated_file_write_bytes(&handle.id.clone(), data)
    }
    pub(super) fn remove_file(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::remove_file(path).map_err(sifr_generated_io_err)
    }
    pub(super) fn touch(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::touch(path).map_err(sifr_generated_io_err)
    }
    pub(super) fn sifr_generated_const_454e434f44494e475f55544638() -> String {
        "utf-8".to_string()
    }
    pub(super) fn sifr_generated_const_4445434f44455f4552524f52535f535452494354() -> String {
        "strict".to_string()
    }
    pub(super) fn sifr_generated_const_454e434f44455f4552524f52535f535452494354() -> String {
        "strict".to_string()
    }
    pub(super) fn sifr_generated_encoding_encode_outcome(
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
            let e = sifr_generated_try_err;
            Err(SifrGeneratedStdlibSifrX2eencodingX2eEncodeError::new(
                e.message,
            ))
        })
    }
    pub(super) fn utf8() -> SifrGeneratedStdlibSifrX2eencodingX2eEncoding {
        SifrGeneratedStdlibSifrX2eencodingX2eEncoding::new(
            sifr_generated_const_454e434f44494e475f55544638(),
        )
    }
    pub(super) fn strict_decode_handler() -> SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler
    {
        SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler::new(
            sifr_generated_const_4445434f44455f4552524f52535f535452494354(),
        )
    }
    #[expect(
        clippy::ref_option,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn sifr_generated_encode_handler_name(
        errors: &Option<SifrGeneratedStdlibSifrX2eencodingX2eEncodeErrorHandler>,
    ) -> String {
        let Some(errors) = errors.as_ref() else {
            return sifr_generated_const_454e434f44455f4552524f52535f535452494354();
        };
        errors.name.clone()
    }
    #[expect(
        clippy::ref_option,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn encode_outcome(
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
    #[expect(
        clippy::ref_option,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn encode(
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
            let e = sifr_generated_try_err;
            Err(SifrGeneratedStdlibSifrX2eencodingX2eEncodeError::new(
                e.message,
            ))
        })
    }
    #[derive(Debug, Clone)]
    pub(super) enum SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a019X3a5X3aclass7X3aIOError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass20X3asifrX2ebuiltinX2eIOError1X3a0(IOError),
        SifrGeneratedUnionVariant5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a0(
            SifrGeneratedStdlibSifrX2eencodingX2eEncodeError,
        ),
    }
    impl From<IOError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a019X3a5X3aclass7X3aIOError1X3a0 {
        fn from(value: IOError) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass20X3asifrX2ebuiltinX2eIOError1X3a0(
                value,
            )
        }
    }
    impl From<SifrGeneratedStdlibSifrX2eencodingX2eEncodeError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a019X3a5X3aclass7X3aIOError1X3a0 {
        fn from(value: SifrGeneratedStdlibSifrX2eencodingX2eEncodeError) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a019X3a5X3aclass7X3aIOError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                Self::SifrGeneratedUnionVariant5X3aclass20X3asifrX2ebuiltinX2eIOError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                Self::SifrGeneratedUnionVariant5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
    pub(super) fn sifr_generated_closed_stream_error() -> String {
        "I/O operation on closed stream".to_string()
    }
    pub(super) fn sifr_generated_mode_is_writable(mode: &str) -> bool {
        mode.contains(&"w".to_string())
            || mode.contains(&"a".to_string())
            || mode.contains(&"+".to_string())
    }
    pub(super) fn sifr_generated_text_binary_mode(mode: &str) -> Result<String, IOError> {
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
            let mut sifr_generated_concat: String =
                String::with_capacity(19usize.saturating_add(mode.len()));
            sifr_generated_concat.push_str("invalid text mode: ");
            sifr_generated_concat.push_str(mode);
            sifr_generated_concat
        }))
    }
    #[expect(
        clippy::ref_option,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn sifr_generated_text_encoding_or_default(
        enc: &Option<SifrGeneratedStdlibSifrX2eencodingX2eEncoding>,
    ) -> SifrGeneratedStdlibSifrX2eencodingX2eEncoding {
        let Some(enc) = enc.as_ref() else {
            return SifrGeneratedStdlibSifrX2eencodingX2eEncoding::new("utf-8".to_string());
        };
        SifrGeneratedStdlibSifrX2eencodingX2eEncoding::new(enc.label.clone())
    }
    #[expect(
        clippy::ref_option,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn sifr_generated_decode_errors_or_default(
        errors: &Option<SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler>,
    ) -> SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler {
        let Some(errors) = errors.as_ref() else {
            return strict_decode_handler();
        };
        SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler::new(errors.name.clone())
    }
    pub(super) fn sifr_generated_encode_errors_from_decode_errors(
        errors: &SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler,
    ) -> SifrGeneratedStdlibSifrX2eencodingX2eEncodeErrorHandler {
        SifrGeneratedStdlibSifrX2eencodingX2eEncodeErrorHandler::new(errors.name.clone())
    }
    pub(super) fn open_binary(
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
            let e = sifr_generated_try_err;
            Err(e)
        })
    }
    #[expect(
        clippy::ref_option,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn open_text(
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
            let e = sifr_generated_try_err;
            Err(e)
        })
    }
    pub(super) const fn sifr_generated_const_51554f54455f414c4c() -> SifrInt {
        SifrInt::from_i64(1)
    }
    pub(super) const fn sifr_generated_const_51554f54455f4e4f4e4e554d45524943() -> SifrInt {
        SifrInt::from_i64(2)
    }
    pub(super) const fn sifr_generated_const_51554f54455f4e4f4e45() -> SifrInt {
        SifrInt::from_i64(3)
    }
    pub(super) const fn sifr_generated_const_51554f54455f535452494e4753() -> SifrInt {
        SifrInt::from_i64(4)
    }
    pub(super) const fn sifr_generated_const_51554f54455f4e4f544e554c4c() -> SifrInt {
        SifrInt::from_i64(5)
    }
    pub(super) fn sifr_generated_copy_dialect(
        dialect: &SifrGeneratedStdlibSifrX2ecsvX2eDialect,
    ) -> SifrGeneratedStdlibSifrX2ecsvX2eDialect {
        SifrGeneratedStdlibSifrX2ecsvX2eDialect::new(
            dialect.delimiter.to_string(),
            dialect.quotechar.to_string(),
            dialect.escapechar.to_string(),
            dialect.doublequote,
            dialect.skipinitialspace,
            dialect.lineterminator.to_string(),
            dialect.quoting.clone(),
        )
    }
    pub(super) fn sifr_generated_validate_char(name: &str, value: &str) {
        let _ = name.to_owned();
        let _ = value.to_owned();
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "generated signature preserves the typed Sifr callable contract"
    )]
    #[expect(
        clippy::ref_option,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn sifr_generated_resolve_dialect(
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
                quoting,
            );
        };
        sifr_generated_copy_dialect(dialect)
    }
    pub(super) fn sifr_generated_quotechar_value(
        dialect: &SifrGeneratedStdlibSifrX2ecsvX2eDialect,
    ) -> String {
        let quotechar: String = dialect.quotechar.clone();
        if quotechar.as_str() == String::new().as_str() {
            return "\"".to_string();
        }
        quotechar
    }
    pub(super) fn sifr_generated_append_field(row: &mut Vec<String>, field: String) {
        row.push(field);
    }
    pub(super) fn sifr_generated_append_row(rows: &mut Vec<Vec<String>>, row: Vec<String>) {
        rows.push(row);
    }
    pub(super) fn sifr_generated_char_at(text: &str, index: SifrInt) -> String {
        let sifr_generated_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
        if index < SifrInt::from_i64(0) || index >= sifr_generated_chars_text.len() {
            return String::new();
        }
        let ch: Option<String> = {
            let sifr_generated_string_index = index;
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
        clippy::needless_pass_by_value,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn sifr_generated_list_value_at(values: &[String], index: SifrInt) -> String {
        if index < SifrInt::from_i64(0) || index >= values.len() {
            return String::new();
        }
        for (current_index, value) in Box::new(values.iter().cloned().enumerate().map(
            |sifr_generated_pair| {
                (
                    ::std::ops::Add::add(
                        SifrInt::from(sifr_generated_pair.0),
                        SifrInt::from_i64(0),
                    ),
                    sifr_generated_pair.1,
                )
            },
        )) {
            if current_index == index {
                return {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(value.len().saturating_add(0usize));
                    sifr_generated_concat.push_str(value.as_str());
                    sifr_generated_concat.push_str("");
                    sifr_generated_concat
                };
            }
        }
        String::new()
    }
    pub(super) fn sifr_generated_dict_value_at(
        values: &HashMap<String, String>,
        key: &str,
    ) -> String {
        for item_key in values.keys().cloned() {
            if item_key != *key {
                continue;
            }
            let value_value_7ce4fd9430e80cea: Option<String> = values.get(&item_key).cloned();
            let Some(value_value_7ce4fd9430e80cea) = value_value_7ce4fd9430e80cea else {
                return String::new();
            };
            return {
                let mut sifr_generated_concat: String = String::with_capacity(
                    value_value_7ce4fd9430e80cea.len().saturating_add(0usize),
                );
                sifr_generated_concat.push_str(value_value_7ce4fd9430e80cea.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
        }
        String::new()
    }
    pub(super) fn sifr_generated_first_char(text: &str) -> String {
        sifr_generated_char_at(text, SifrInt::from_i64(0))
    }
    pub(super) fn sifr_generated_last_char(text: &str) -> String {
        let _ = text.chars().collect::<Vec<char>>();
        sifr_generated_char_at(
            text,
            ::std::ops::Sub::sub(SifrInt::from(text.chars().count()), SifrInt::from_i64(1)),
        )
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "generated signature preserves the typed Sifr callable contract"
    )]
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    #[expect(
        clippy::ref_option,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    #[expect(
        clippy::needless_pass_by_value,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn parse_csv(
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
            quotechar,
            escapechar,
            doublequote,
            skipinitialspace,
            "\n",
            quoting.clone(),
        );
        let mut rows: Vec<Vec<String>> = Vec::new();
        let mut row: Vec<String> = Vec::new();
        let mut field: String = String::new();
        let mut in_quotes: bool = false;
        let mut field_started: bool = false;
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < sifr_generated_chars_text.len() {
            let ch_value: String = sifr_generated_char_at(text, i.clone());
            if in_quotes {
                if !resolved.escapechar.clone().is_empty() && ch_value == resolved.escapechar {
                    if ::std::ops::Add::add(&i, &SifrInt::from_i64(1))
                        < sifr_generated_chars_text.len()
                    {
                        let escaped_value: String = sifr_generated_char_at(
                            text,
                            ::std::ops::Add::add(&i, &SifrInt::from_i64(1)),
                        );
                        field.push_str(escaped_value.as_str());
                        i = ::std::ops::Add::add(&i, &SifrInt::from_i64(2));
                        continue;
                    }
                    field.push_str(ch_value.as_str());
                    i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
                    continue;
                }
                if !resolved.quotechar.clone().is_empty() && ch_value == resolved.quotechar {
                    let quotechar: String = sifr_generated_quotechar_value(&resolved);
                    if resolved.doublequote
                        && ::std::ops::Add::add(&i, &SifrInt::from_i64(1))
                            < sifr_generated_chars_text.len()
                        && sifr_generated_char_at(
                            text,
                            ::std::ops::Add::add(&i, &SifrInt::from_i64(1)),
                        ) == quotechar
                    {
                        field.push_str(quotechar.as_str());
                        i = ::std::ops::Add::add(&i, &SifrInt::from_i64(2));
                        continue;
                    }
                    in_quotes = false;
                    i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
                    continue;
                }
                field.push_str(ch_value.as_str());
                i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
                continue;
            }
            if !field_started && resolved.skipinitialspace && ch_value == " " {
                i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
                continue;
            }
            if !resolved.escapechar.clone().is_empty() && ch_value == resolved.escapechar {
                if ::std::ops::Add::add(&i, &SifrInt::from_i64(1)) < sifr_generated_chars_text.len()
                {
                    let escaped_plain_value: String = sifr_generated_char_at(
                        text,
                        ::std::ops::Add::add(&i, &SifrInt::from_i64(1)),
                    );
                    field.push_str(escaped_plain_value.as_str());
                    field_started = true;
                    i = ::std::ops::Add::add(&i, &SifrInt::from_i64(2));
                    continue;
                }
                field.push_str(ch_value.as_str());
                field_started = true;
                i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
                continue;
            }
            if resolved.quoting != sifr_generated_const_51554f54455f4e4f4e45()
                && !resolved.quotechar.clone().is_empty()
            {
                let quotechar2_value_123324c155e57c27: String =
                    sifr_generated_quotechar_value(&resolved);
                if ch_value == quotechar2_value_123324c155e57c27 {
                    in_quotes = true;
                    field_started = true;
                    i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
                    continue;
                }
            }
            if ch_value == resolved.delimiter {
                sifr_generated_append_field(&mut row, field);
                field = String::new();
                field_started = false;
                i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
                continue;
            }
            if ch_value == "\n" || ch_value == "\r" {
                if ch_value == "\r"
                    && ::std::ops::Add::add(&i, &SifrInt::from_i64(1))
                        < sifr_generated_chars_text.len()
                    && sifr_generated_char_at(text, ::std::ops::Add::add(&i, &SifrInt::from_i64(1)))
                        == "\n"
                {
                    i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
                }
                if row.len() == SifrInt::from_i64(0) && field.is_empty() {
                    sifr_generated_append_row(&mut rows, Vec::new());
                } else {
                    sifr_generated_append_field(&mut row, field);
                    sifr_generated_append_row(&mut rows, row);
                }
                row = Vec::new();
                field = String::new();
                field_started = false;
                i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
                continue;
            }
            field.push_str(ch_value.as_str());
            field_started = true;
            i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
        }
        let _ = in_quotes;
        if row.len() > SifrInt::from_i64(0) || !field.is_empty() {
            sifr_generated_append_field(&mut row, field);
            sifr_generated_append_row(&mut rows, row);
        }
        rows
    }
    pub(super) fn sifr_generated_needs_quote(
        field: &str,
        dialect: &SifrGeneratedStdlibSifrX2ecsvX2eDialect,
    ) -> bool {
        if dialect.quoting == sifr_generated_const_51554f54455f414c4c() {
            return true;
        }
        if dialect.quoting == sifr_generated_const_51554f54455f4e4f4e4e554d45524943() {
            return true;
        }
        if dialect.quoting == sifr_generated_const_51554f54455f535452494e4753() {
            return true;
        }
        if dialect.quoting == sifr_generated_const_51554f54455f4e4f544e554c4c() {
            return true;
        }
        if dialect.quoting == sifr_generated_const_51554f54455f4e4f4e45() {
            return false;
        }
        if field.contains(dialect.delimiter.clone().as_str()) {
            return true;
        }
        if field.contains(&"\n".to_string()) || field.contains(&"\r".to_string()) {
            return true;
        }
        if !dialect.quotechar.clone().is_empty() {
            let quotechar: String = sifr_generated_quotechar_value(dialect);
            if field.contains(&quotechar) {
                return true;
            }
        }
        if field.chars().count() > SifrInt::from_i64(0) {
            let first: String = sifr_generated_first_char(field);
            let last: String = sifr_generated_last_char(field);
            if first == " " {
                return true;
            }
            if last == " " {
                return true;
            }
        }
        false
    }
    pub(super) fn sifr_generated_quote_field(
        field: &str,
        dialect: &SifrGeneratedStdlibSifrX2ecsvX2eDialect,
    ) -> String {
        let quotechar: String = sifr_generated_quotechar_value(dialect);
        let mut escaped: String = {
            let mut sifr_generated_concat: String =
                String::with_capacity(field.len().saturating_add(0usize));
            sifr_generated_concat.push_str(field);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        };
        if escaped.contains(&quotechar) {
            if dialect.doublequote || dialect.escapechar.clone().is_empty() {
                escaped = escaped.replace(&quotechar, &format!("{quotechar}{quotechar}"));
            } else {
                let escapechar_value: String = dialect.escapechar.clone();
                escaped = escaped.replace(&quotechar, &format!("{escapechar_value}{quotechar}"));
            }
        }
        {
            let mut sifr_generated_concat: String = String::with_capacity(
                quotechar
                    .len()
                    .saturating_add(escaped.len())
                    .saturating_add(quotechar.len()),
            );
            sifr_generated_concat.push_str(quotechar.as_str());
            sifr_generated_concat.push_str(escaped.as_str());
            sifr_generated_concat.push_str(quotechar.as_str());
            sifr_generated_concat
        }
    }
    pub(super) fn sifr_generated_escape_unquoted_field(
        field: &str,
        dialect: &SifrGeneratedStdlibSifrX2ecsvX2eDialect,
    ) -> String {
        let mut result: String = {
            let mut sifr_generated_concat: String =
                String::with_capacity(field.len().saturating_add(0usize));
            sifr_generated_concat.push_str(field);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        };
        if result.contains(dialect.delimiter.clone().as_str())
            && !dialect.escapechar.clone().is_empty()
        {
            result = result.replace(
                &dialect.delimiter.clone(),
                &format!("{}{}", dialect.escapechar, dialect.delimiter),
            );
        }
        if result.contains(&"\n".to_string()) && !dialect.escapechar.clone().is_empty() {
            result = result.replace('\n', &format!("{}\n", dialect.escapechar));
        }
        if result.contains(&"\r".to_string()) && !dialect.escapechar.clone().is_empty() {
            result = result.replace('\r', &format!("{}\r", dialect.escapechar));
        }
        if !dialect.quotechar.clone().is_empty() {
            let quotechar2: String = sifr_generated_quotechar_value(dialect);
            if result.contains(&quotechar2) {
                if dialect.escapechar.clone().is_empty() {
                    result = result.replace(&quotechar2, &format!("{quotechar2}{quotechar2}"));
                } else {
                    result = result.replace(
                        &quotechar2,
                        &format!("{}{}", dialect.escapechar, quotechar2),
                    );
                }
            }
        }
        result
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "generated signature preserves the typed Sifr callable contract"
    )]
    #[expect(
        clippy::ref_option,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn format_row(
        fields: &[String],
        dialect: &Option<SifrGeneratedStdlibSifrX2ecsvX2eDialect>,
        delimiter: &str,
        quotechar: &str,
        escapechar: &str,
        doublequote: bool,
        skipinitialspace: bool,
        quoting: SifrInt,
    ) -> String {
        let resolved: SifrGeneratedStdlibSifrX2ecsvX2eDialect = sifr_generated_resolve_dialect(
            dialect,
            delimiter,
            quotechar,
            escapechar,
            doublequote,
            skipinitialspace,
            "\n",
            quoting,
        );
        let mut parts: Vec<String> = Vec::new();
        #[expect(
            clippy::explicit_iter_loop,
            reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
        )]
        for field in fields.iter() {
            if sifr_generated_needs_quote(field, &resolved) {
                parts.push(sifr_generated_quote_field(field, &resolved));
            } else {
                parts.push(sifr_generated_escape_unquoted_field(field, &resolved));
            }
        }
        parts.join(&resolved.delimiter)
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "generated signature preserves the typed Sifr callable contract"
    )]
    #[expect(
        clippy::ref_option,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn format_csv(
        rows: &[Vec<String>],
        dialect: &Option<SifrGeneratedStdlibSifrX2ecsvX2eDialect>,
        delimiter: &str,
        quotechar: &str,
        escapechar: &str,
        doublequote: bool,
        skipinitialspace: bool,
        lineterminator: &str,
        quoting: SifrInt,
    ) -> String {
        let resolved: SifrGeneratedStdlibSifrX2ecsvX2eDialect = sifr_generated_resolve_dialect(
            dialect,
            delimiter,
            quotechar,
            escapechar,
            doublequote,
            skipinitialspace,
            lineterminator,
            quoting,
        );
        let mut rendered: Vec<String> = Vec::new();
        let resolved_delimiter: String = resolved.delimiter.clone();
        let resolved_quotechar: String = resolved.quotechar.clone();
        let resolved_escapechar: String = resolved.escapechar.clone();
        let resolved_lineterminator: String = resolved.lineterminator.clone();
        #[expect(
            clippy::explicit_iter_loop,
            reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
        )]
        for row in rows.iter() {
            rendered.push(format_row(
                row,
                &None,
                &resolved_delimiter,
                &resolved_quotechar,
                &resolved_escapechar,
                resolved.doublequote,
                resolved.skipinitialspace,
                resolved.quoting.clone(),
            ));
        }
        rendered.join(&resolved_lineterminator)
    }
    pub(super) fn sifr_generated_dict_reader_row(
        fieldnames: &[String],
        row: &[String],
        restkey: &str,
        restval: &str,
    ) -> HashMap<String, String> {
        let mut result: HashMap<String, String> = HashMap::from([]);
        for (i, key) in Box::new(fieldnames.iter().cloned().enumerate().map(
            |sifr_generated_pair| {
                (
                    ::std::ops::Add::add(
                        SifrInt::from(sifr_generated_pair.0),
                        SifrInt::from_i64(0),
                    ),
                    sifr_generated_pair.1,
                )
            },
        )) {
            if i < row.len() {
                {
                    let sifr_generated_assign_value = sifr_generated_list_value_at(row, i.clone());
                    {
                        let sifr_generated_assign_key = key.to_owned();
                        result.insert(sifr_generated_assign_key, sifr_generated_assign_value);
                    }
                }
            } else {
                let sifr_generated_assign_value = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(restval.len().saturating_add(0usize));
                    sifr_generated_concat.push_str(restval);
                    sifr_generated_concat.push_str("");
                    sifr_generated_concat
                };
                {
                    let sifr_generated_assign_key = key.to_owned();
                    result.insert(sifr_generated_assign_key, sifr_generated_assign_value);
                }
            }
        }
        if !restkey.is_empty() && row.len() > fieldnames.len() {
            let mut extras: Vec<String> = Vec::new();
            let mut j: SifrInt = SifrInt::from(fieldnames.len());
            while j < row.len() {
                extras.push(sifr_generated_list_value_at(row, j.clone()));
                j = ::std::ops::Add::add(&j, &SifrInt::from_i64(1));
            }
            {
                let sifr_generated_assign_value = format!("{extras:?}");
                {
                    let sifr_generated_assign_key = restkey.to_owned();
                    result.insert(sifr_generated_assign_key, sifr_generated_assign_value);
                }
            }
        }
        result
    }
    #[expect(
        clippy::needless_pass_by_value,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn sifr_generated_six_digits(value: SifrInt) -> String {
        let mut rendered: String = value.to_string();
        let mut sifr_generated_chars_rendered: Vec<char> = rendered.chars().collect::<Vec<char>>();
        while sifr_generated_chars_rendered.len() < SifrInt::from_i64(6) {
            rendered = {
                let mut sifr_generated_concat: String =
                    String::with_capacity(1usize.saturating_add(rendered.len()));
                sifr_generated_concat.push('0');
                sifr_generated_concat.push_str(rendered.as_str());
                sifr_generated_concat
            };
            sifr_generated_chars_rendered = rendered.chars().collect::<Vec<char>>();
        }
        rendered
    }
    pub(super) fn get_global_level() -> SifrInt {
        ::sifr_stdlib::logging::get_global_level().into_sifr_int()
    }
    pub(super) const fn sifr_generated_const_4445425547() -> SifrInt {
        SifrInt::from_i64(10)
    }
    pub(super) const fn sifr_generated_const_494e464f() -> SifrInt {
        SifrInt::from_i64(20)
    }
    pub(super) const fn sifr_generated_const_5741524e494e47() -> SifrInt {
        SifrInt::from_i64(30)
    }
    pub(super) const fn sifr_generated_const_4e4f54534554() -> SifrInt {
        SifrInt::from_i64(0)
    }
    #[expect(
        non_snake_case,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn getLogger(name: &str) -> SifrGeneratedStdlibSifrX2eloggingX2eLogger {
        let level: SifrInt = get_global_level();
        SifrGeneratedStdlibSifrX2eloggingX2eLogger::new(name.to_owned(), level)
    }
    pub(super) fn basename(path: &str) -> String {
        let sifr_generated_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
        let mut i: SifrInt = ::std::ops::Sub::sub(
            &SifrInt::from(sifr_generated_chars_path.len()),
            &SifrInt::from_i64(1),
        );
        while i >= SifrInt::from_i64(0) {
            let ch: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_path.len());
                sifr_generated_chars_path
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(ch) = ch
                && ch == "/"
            {
                return {
                    let sifr_generated_slice_src = &sifr_generated_chars_path;
                    let sifr_generated_slice_len = sifr_generated_slice_src.len();
                    let sifr_generated_slice_start =
                        ::std::ops::Add::add(&i, &SifrInt::from_i64(1))
                            .clamp_slice_bound(sifr_generated_slice_len);
                    let sifr_generated_slice_stop = sifr_generated_slice_len;
                    String::from_iter(
                        sifr_generated_slice_src
                            .iter()
                            .skip(sifr_generated_slice_start)
                            .take(
                                sifr_generated_slice_stop
                                    .saturating_sub(sifr_generated_slice_start),
                            )
                            .copied(),
                    )
                };
            }
            i = ::std::ops::Sub::sub(&i, &SifrInt::from_i64(1));
        }
        {
            let mut sifr_generated_concat: String =
                String::with_capacity(path.len().saturating_add(0usize));
            sifr_generated_concat.push_str(path);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        }
    }
    pub(super) fn dirname(path: &str) -> String {
        let sifr_generated_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
        let mut i: SifrInt = ::std::ops::Sub::sub(
            &SifrInt::from(sifr_generated_chars_path.len()),
            &SifrInt::from_i64(1),
        );
        while i >= SifrInt::from_i64(0) {
            let ch: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_path.len());
                sifr_generated_chars_path
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(ch) = ch
                && ch == "/"
            {
                return {
                    let sifr_generated_slice_src = &sifr_generated_chars_path;
                    let sifr_generated_slice_len = sifr_generated_slice_src.len();
                    let sifr_generated_slice_start = 0;
                    let sifr_generated_slice_stop = i.clamp_slice_bound(sifr_generated_slice_len);
                    String::from_iter(
                        sifr_generated_slice_src
                            .iter()
                            .skip(sifr_generated_slice_start)
                            .take(
                                sifr_generated_slice_stop
                                    .saturating_sub(sifr_generated_slice_start),
                            )
                            .copied(),
                    )
                };
            }
            i = ::std::ops::Sub::sub(&i, &SifrInt::from_i64(1));
        }
        String::new()
    }
    pub(super) fn stem(path: &str) -> String {
        let base: String = basename(path);
        let sifr_generated_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
        let mut i: SifrInt = ::std::ops::Sub::sub(
            &SifrInt::from(sifr_generated_chars_base.len()),
            &SifrInt::from_i64(1),
        );
        while i > SifrInt::from_i64(0) {
            let ch: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_base.len());
                sifr_generated_chars_base
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(ch) = ch
                && ch == "."
            {
                return {
                    let sifr_generated_slice_src = &sifr_generated_chars_base;
                    let sifr_generated_slice_len = sifr_generated_slice_src.len();
                    let sifr_generated_slice_start = 0;
                    let sifr_generated_slice_stop = i.clamp_slice_bound(sifr_generated_slice_len);
                    String::from_iter(
                        sifr_generated_slice_src
                            .iter()
                            .skip(sifr_generated_slice_start)
                            .take(
                                sifr_generated_slice_stop
                                    .saturating_sub(sifr_generated_slice_start),
                            )
                            .copied(),
                    )
                };
            }
            i = ::std::ops::Sub::sub(&i, &SifrInt::from_i64(1));
        }
        base
    }
    pub(super) trait SifrGeneratedOpaqueSifrStdlibSifrX2eregexX2eCompiledPatternMethods {
        fn search(&self, text: &str) -> Result<Option<String>, RegexError>;
        fn is_match(&self, text: &str) -> Result<bool, RegexError>;
        fn findall(&self, text: &str) -> Result<Vec<String>, RegexError>;
    }
    pub(super) fn compile_pattern(
        pattern: &str,
    ) -> Result<SifrGeneratedStdlibSifrX2eregexX2eCompiledPattern, RegexError> {
        ::sifr_stdlib::regex::compile_pattern(pattern).map_err(|sifr_generated_bridge_error| {
            RegexError {
                message: sifr_generated_bridge_error.to_string(),
                detail: sifr_generated_bridge_error.to_string(),
            }
        })
    }
    pub(super) fn re_match(pattern: &str, text: &str) -> Result<bool, RegexError> {
        ::sifr_stdlib::regex::re_match(pattern, text).map_err(|sifr_generated_bridge_error| {
            RegexError {
                message: sifr_generated_bridge_error.to_string(),
                detail: sifr_generated_bridge_error.to_string(),
            }
        })
    }
    pub(super) fn compile(
        pattern: &str,
    ) -> Result<SifrGeneratedStdlibSifrX2ereX2ePattern, RegexError> {
        let sifr_generated_try_res: Result<
            Result<SifrGeneratedStdlibSifrX2ereX2ePattern, RegexError>,
            RegexError,
        > = (|| {
            let compiled: SifrGeneratedStdlibSifrX2eregexX2eCompiledPattern =
                compile_pattern(pattern)?;
            Ok(Ok(SifrGeneratedStdlibSifrX2ereX2ePattern::new(
                compiled,
                pattern.to_owned(),
                SifrInt::from_i64(0),
            )))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let error = sifr_generated_try_err;
            Err(RegexError::new(error.message))
        })
    }
    pub(super) fn fullmatch(pattern: &str, text: &str) -> Result<bool, RegexError> {
        let anchored: String = {
            let mut sifr_generated_concat: String =
                String::with_capacity(1usize.saturating_add(pattern.len()).saturating_add(1usize));
            sifr_generated_concat.push('^');
            sifr_generated_concat.push_str(pattern);
            sifr_generated_concat.push('$');
            sifr_generated_concat
        };
        re_match(&anchored, text)
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
    use crate::SifrGeneratedIoNativeFileHandle;
    use crate::sifr_generated_generated_support::{
        SifrGeneratedOpaqueSifrStdlibSifrX2eregexX2eCompiledPatternMethods,
        SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a019X3a5X3aclass7X3aIOError1X3a0,
        dirname, encode, exists, file_close, file_write_bytes, format_csv, open_text, parse_csv,
        remove_file, sifr_generated_closed_stream_error, sifr_generated_const_4e4f54534554,
        sifr_generated_const_494e464f, sifr_generated_const_51554f54455f4e4f4e45,
        sifr_generated_const_5741524e494e47, sifr_generated_const_4445425547,
        sifr_generated_dict_reader_row, sifr_generated_dict_value_at,
        sifr_generated_mode_is_writable, sifr_generated_resolve_dialect, sifr_generated_six_digits,
        sifr_generated_validate_char, stem, touch, utf8,
    };
    use ::sifr_runtime::SifrInt;
    use ::std::collections::HashMap;
    use ::std::collections::VecDeque;
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        pub data_field: VecDeque<T>,
        pub maxlen: Option<SifrInt>,
    }
    impl<T: Clone> SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        #[must_use]
        pub fn new(items: Option<Vec<T>>, maxlen: Option<SifrInt>) -> Self {
            let mut data: Vec<T> = Vec::new();
            if let Some(items) = items {
                let start: SifrInt = if let Some(maxlen) = maxlen.clone()
                    && items.len() > maxlen
                {
                    ::std::ops::Sub::sub(&SifrInt::from(items.len()), &maxlen)
                } else {
                    SifrInt::from_i64(0)
                };
                let mut i: SifrInt = start;
                while i < items.len() {
                    let item_value_2841a0c596d6f426: Option<T> = {
                        let sifr_generated_checked_read_collection = &items;
                        let sifr_generated_checked_read_index = &i;
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(item) = item_value_2841a0c596d6f426 {
                        data.push(item);
                    }
                    i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
                }
            }
            let sifr_generated_field_value_169953f6befb0270_6d61786c656e: Option<SifrInt> = maxlen;
            let sifr_generated_field_value_90770dc80a1c57ce_5f64617461: VecDeque<T> =
                VecDeque::from(data);
            Self {
                maxlen: sifr_generated_field_value_169953f6befb0270_6d61786c656e,
                data_field: sifr_generated_field_value_90770dc80a1c57ce_5f64617461,
            }
        }
    }
    impl<T: Clone> SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        pub fn append(&mut self, val: &T) {
            self.data_field.push_back(val.clone());
            let maxlen_opt: Option<SifrInt> = self.maxlen.clone();
            if let Some(maxlen_opt) = maxlen_opt {
                let maxlen: SifrInt = maxlen_opt;
                if self.data_field.len() > maxlen {
                    self.data_field.pop_front();
                }
            }
        }
    }
    impl<T> SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        #[must_use]
        pub fn popleft(&mut self) -> Option<T> {
            if self.data_field.len() == SifrInt::from_i64(0) {
                return None;
            }
            Some({
                let sifr_generated_nonempty_pop_index = 0_usize;
                let mut sifr_generated_nonempty_pop_values = self
                    .data_field
                    .drain(sifr_generated_nonempty_pop_index..=sifr_generated_nonempty_pop_index)
                    .collect::<Vec<_>>();
                sifr_generated_nonempty_pop_values.remove(0_usize)
            })
        }
    }
    impl<T> SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        #[must_use]
        pub fn len(&self) -> SifrInt {
            SifrInt::from(self.data_field.len())
        }
    }
    impl<T> SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        pub fn clear(&mut self) {
            self.data_field.clear();
        }
    }
    impl<T: Clone + PartialEq> SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        #[must_use]
        pub fn count(&self, value: &T) -> SifrInt {
            let mut total: SifrInt = SifrInt::from_i64(0);
            for item in self.data_field.iter() {
                if item == value {
                    total = ::std::ops::Add::add(&total, &SifrInt::from_i64(1));
                }
            }
            total
        }
    }
    impl<T: Clone + PartialEq> SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        #[must_use]
        pub fn index(&self, value: &T, start: &SifrInt, stop: &Option<SifrInt>) -> Option<SifrInt> {
            let size: SifrInt = SifrInt::from(self.data_field.len());
            let mut begin: SifrInt = start.clone();
            if begin < SifrInt::from_i64(0) {
                begin = ::std::ops::Add::add(&size, &begin);
                if begin < SifrInt::from_i64(0) {
                    begin = SifrInt::from_i64(0);
                }
            }
            let mut end: SifrInt = size.clone();
            if let Some(stop) = stop.as_ref() {
                end.clone_from(stop);
                if end < SifrInt::from_i64(0) {
                    end = ::std::ops::Add::add(&size, &end);
                }
                if end < SifrInt::from_i64(0) {
                    end = SifrInt::from_i64(0);
                }
                if end > size {
                    end = size;
                }
            }
            let mut i: SifrInt = begin;
            while i < end {
                let current: Option<T> = {
                    let sifr_generated_checked_read_collection = &self.data_field;
                    let sifr_generated_checked_read_index = &i;
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(current) = current
                    && current == *value
                {
                    return Some(i);
                }
                i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
            }
            None
        }
    }
    impl<T: Clone + PartialEq> SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        pub fn remove(&mut self, value: &T) {
            let idx: Option<SifrInt> = self.index(value, &SifrInt::from_i64(0), &None);
            if let Some(idx) = idx {
                let mut rebuilt: Vec<T> = Vec::new();
                let mut i: SifrInt = SifrInt::from_i64(0);
                while i < self.data_field.len() {
                    let current: Option<T> = {
                        let sifr_generated_checked_read_collection = &self.data_field;
                        let sifr_generated_checked_read_index = &i;
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(current) = current
                        && i != idx
                    {
                        rebuilt.push(current);
                    }
                    i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
                }
                self.data_field.clear();
                #[expect(
                    clippy::explicit_iter_loop,
                    reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
                )]
                for item in rebuilt.iter() {
                    self.data_field.push_back(item.clone());
                }
            }
        }
    }
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
        pub const fn new(label: String) -> Self {
            let sifr_generated_field_value_39f7fcec8fcb623d_6c6162656c: String = label;
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
        pub const fn new(name: String) -> Self {
            let sifr_generated_field_value_c4bcadba8e631b86_6e616d65: String = name;
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
        pub const fn new(name: String) -> Self {
            let sifr_generated_field_value_c4bcadba8e631b86_6e616d65: String = name;
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
                        SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass20X3asifrX2ebuiltinX2eIOError1X3a0,
                    )?;
                Ok(Ok(()))
            })();
            sifr_generated_try_res
                .unwrap_or_else(|sifr_generated_try_err| match sifr_generated_try_err {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass20X3asifrX2ebuiltinX2eIOError1X3a0(
                        sifr_generated_try_variant_error,
                    ) => {
                        let e = sifr_generated_try_variant_error;
                        Err(e)
                    }
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a238X3a5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass25X3asifrX2eencodingX2eEncodeError1X3a0(
                        sifr_generated_try_variant_error,
                    ) => {
                        let e = sifr_generated_try_variant_error;
                        Err(
                            IOError::new({
                                let mut sifr_generated_concat: String = String::with_capacity(
                                    20usize.saturating_add(0usize),
                                );
                                sifr_generated_concat.push_str("text encode failed: ");
                                sifr_generated_concat.push_str(e.message.as_str());
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
            let mut resolved_quoting: SifrInt = quoting;
            sifr_generated_validate_char("delimiter", &delimiter);
            if !quotechar.is_empty() {
                sifr_generated_validate_char("quotechar", &quotechar);
            }
            if !escapechar.is_empty() {
                sifr_generated_validate_char("escapechar", &escapechar);
            }
            if quotechar.is_empty()
                && resolved_quoting != sifr_generated_const_51554f54455f4e4f4e45()
            {
                resolved_quoting.clone_from(&sifr_generated_const_51554f54455f4e4f4e45());
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
                resolved_quoting;
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
        #[expect(
            clippy::needless_pass_by_value,
            reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
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
                    "\n",
                    quoting,
                );
            let rows: Vec<Vec<String>> = parse_csv(
                &text,
                &None,
                &resolved_dialect.delimiter.to_string(),
                &resolved_dialect.quotechar.to_string(),
                &resolved_dialect.escapechar.to_string(),
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
            for row in self.rows.iter() {
                let mut copied: Vec<String> = Vec::new();
                #[expect(
                    clippy::explicit_iter_loop,
                    reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
                )]
                for field in row.iter() {
                    copied.push(field.clone());
                }
                result.push(copied);
            }
            result
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SifrGeneratedStdlibSifrX2ecsvX2ewriter {
        pub rows: Vec<Vec<String>>,
        pub dialect: SifrGeneratedStdlibSifrX2ecsvX2eDialect,
    }
    impl SifrGeneratedStdlibSifrX2ecsvX2ewriter {
        #[must_use]
        #[expect(
            clippy::too_many_arguments,
            reason = "generated signature preserves the typed Sifr callable contract"
        )]
        #[expect(
            clippy::needless_pass_by_value,
            reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
        )]
        pub fn new(
            dialect: Option<SifrGeneratedStdlibSifrX2ecsvX2eDialect>,
            delimiter: String,
            quotechar: String,
            escapechar: String,
            doublequote: bool,
            skipinitialspace: bool,
            lineterminator: String,
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
                    &lineterminator,
                    quoting,
                );
            let sifr_generated_field_value_ac4a5fa27eb34095_6469616c656374: SifrGeneratedStdlibSifrX2ecsvX2eDialect = resolved_dialect;
            let sifr_generated_field_value_d742ae5cfb4259e3_5f726f7773: Vec<Vec<String>> =
                Vec::new();
            Self {
                dialect: sifr_generated_field_value_ac4a5fa27eb34095_6469616c656374,
                rows: sifr_generated_field_value_d742ae5cfb4259e3_5f726f7773,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2ecsvX2ewriter {
        pub fn writerow(&mut self, row: &[String]) {
            let mut copied: Vec<String> = Vec::new();
            #[expect(
                clippy::explicit_iter_loop,
                reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
            )]
            for value in row.iter() {
                copied.push(value.clone());
            }
            self.rows.push(copied);
        }
    }
    impl SifrGeneratedStdlibSifrX2ecsvX2ewriter {
        #[must_use]
        pub fn getvalue(&self) -> String {
            format_csv(
                &self.rows,
                &Some(self.dialect.clone()),
                ",",
                "\"",
                "",
                true,
                false,
                "\n",
                SifrInt::from_i64(0),
            )
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SifrGeneratedStdlibSifrX2ecsvX2eDictReader {
        pub fieldnames_field: Vec<String>,
        pub rows: Vec<Vec<String>>,
        pub pos: SifrInt,
        pub restkey: String,
        pub restval: String,
        pub dialect: SifrGeneratedStdlibSifrX2ecsvX2eDialect,
    }
    impl SifrGeneratedStdlibSifrX2ecsvX2eDictReader {
        #[must_use]
        #[expect(
            clippy::too_many_arguments,
            reason = "generated signature preserves the typed Sifr callable contract"
        )]
        #[expect(
            clippy::needless_pass_by_value,
            reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
        )]
        pub fn new(
            text: String,
            fieldnames: Option<Vec<String>>,
            restkey: String,
            restval: String,
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
                    "\n",
                    quoting,
                );
            let all_rows: Vec<Vec<String>> = parse_csv(
                &text,
                &None,
                &resolved_dialect.delimiter.to_string(),
                &resolved_dialect.quotechar.to_string(),
                &resolved_dialect.escapechar.to_string(),
                resolved_dialect.doublequote,
                resolved_dialect.skipinitialspace,
                resolved_dialect.quoting.clone(),
            );
            let mut fieldnames_data: Vec<String> = Vec::new();
            let mut rows_data: Vec<Vec<String>> = Vec::new();
            if let Some(fieldnames) = fieldnames {
                #[expect(
                    clippy::explicit_iter_loop,
                    reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
                )]
                for field in fieldnames.iter() {
                    fieldnames_data.push(field.clone());
                }
                #[expect(
                    clippy::explicit_iter_loop,
                    reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
                )]
                for row in all_rows.iter() {
                    let mut copied_row: Vec<String> = Vec::new();
                    #[expect(
                        clippy::explicit_iter_loop,
                        reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
                    )]
                    for value in row.iter() {
                        copied_row.push(value.clone());
                    }
                    rows_data.push(copied_row);
                }
            } else {
                for (index, row) in Box::new(all_rows.iter().cloned().enumerate().map(
                    |sifr_generated_pair| {
                        (
                            ::std::ops::Add::add(
                                SifrInt::from(sifr_generated_pair.0),
                                SifrInt::from_i64(0),
                            ),
                            sifr_generated_pair.1,
                        )
                    },
                )) {
                    if index == SifrInt::from_i64(0) {
                        #[expect(
                            clippy::explicit_iter_loop,
                            reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
                        )]
                        for field in row.iter() {
                            fieldnames_data.push(field.clone());
                        }
                    } else {
                        let mut copied_row2_value_34fd90463ec69210: Vec<String> = Vec::new();
                        #[expect(
                            clippy::explicit_iter_loop,
                            reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
                        )]
                        for value in row.iter() {
                            copied_row2_value_34fd90463ec69210.push(value.clone());
                        }
                        rows_data.push(copied_row2_value_34fd90463ec69210);
                    }
                }
            }
            let sifr_generated_field_value_ac4a5fa27eb34095_6469616c656374: SifrGeneratedStdlibSifrX2ecsvX2eDialect = resolved_dialect;
            let sifr_generated_field_value_d10323292550fbae_726573746b6579: String = restkey;
            let sifr_generated_field_value_e9b6e328a309fdee_7265737476616c: String = restval;
            let sifr_generated_field_value_e04b9443eebba9b4_5f706f73: SifrInt =
                SifrInt::from_i64(0);
            let sifr_generated_field_value_efdb691f099ff036_5f6669656c646e616d6573: Vec<String> =
                fieldnames_data;
            let sifr_generated_field_value_d742ae5cfb4259e3_5f726f7773: Vec<Vec<String>> =
                rows_data;
            Self {
                dialect: sifr_generated_field_value_ac4a5fa27eb34095_6469616c656374,
                restkey: sifr_generated_field_value_d10323292550fbae_726573746b6579,
                restval: sifr_generated_field_value_e9b6e328a309fdee_7265737476616c,
                pos: sifr_generated_field_value_e04b9443eebba9b4_5f706f73,
                fieldnames_field:
                    sifr_generated_field_value_efdb691f099ff036_5f6669656c646e616d6573,
                rows: sifr_generated_field_value_d742ae5cfb4259e3_5f726f7773,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2ecsvX2eDictReader {
        #[must_use]
        pub fn fieldnames(&self) -> Vec<String> {
            let mut copied: Vec<String> = Vec::new();
            for field in self.fieldnames_field.iter() {
                copied.push(field.clone());
            }
            copied
        }
    }
    impl SifrGeneratedStdlibSifrX2ecsvX2eDictReader {
        #[must_use]
        pub fn rows(&self) -> Vec<HashMap<String, String>> {
            let mut result: Vec<HashMap<String, String>> = Vec::new();
            for row in self.rows.iter() {
                if row.len() == SifrInt::from_i64(0) {
                    continue;
                }
                result.push(sifr_generated_dict_reader_row(
                    &self.fieldnames_field,
                    row,
                    &self.restkey,
                    &self.restval,
                ));
            }
            result
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SifrGeneratedStdlibSifrX2ecsvX2eDictWriter {
        pub fieldnames: Vec<String>,
        pub restval: String,
        pub extrasaction: String,
        pub writer: SifrGeneratedStdlibSifrX2ecsvX2ewriter,
    }
    impl SifrGeneratedStdlibSifrX2ecsvX2eDictWriter {
        #[must_use]
        #[expect(
            clippy::too_many_arguments,
            reason = "generated signature preserves the typed Sifr callable contract"
        )]
        #[expect(
            clippy::needless_pass_by_value,
            reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
        )]
        pub fn new(
            fieldnames: Vec<String>,
            restval: String,
            extrasaction: String,
            dialect: Option<SifrGeneratedStdlibSifrX2ecsvX2eDialect>,
            delimiter: String,
            quotechar: String,
            escapechar: String,
            doublequote: bool,
            skipinitialspace: bool,
            lineterminator: String,
            quoting: SifrInt,
        ) -> Self {
            let mut fieldnames_data: Vec<String> = Vec::new();
            #[expect(
                clippy::explicit_iter_loop,
                reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
            )]
            for field in fieldnames.iter() {
                fieldnames_data.push(field.clone());
            }
            let mut action: String = extrasaction.to_lowercase();
            if action != "raise" && action != "ignore" {
                action = "raise".to_string();
            }
            let writer_value: SifrGeneratedStdlibSifrX2ecsvX2ewriter =
                SifrGeneratedStdlibSifrX2ecsvX2ewriter::new(
                    dialect,
                    delimiter,
                    quotechar,
                    escapechar,
                    doublequote,
                    skipinitialspace,
                    lineterminator,
                    quoting,
                );
            let sifr_generated_field_value_53f0b4be9e002e35_6669656c646e616d6573: Vec<String> =
                fieldnames_data;
            let sifr_generated_field_value_e9b6e328a309fdee_7265737476616c: String = restval;
            let sifr_generated_field_value_9e8e4a59c8cf9f24_657874726173616374696f6e: String =
                action;
            let sifr_generated_field_value_459f614879d36021_5f777269746572: SifrGeneratedStdlibSifrX2ecsvX2ewriter = writer_value;
            Self {
                fieldnames: sifr_generated_field_value_53f0b4be9e002e35_6669656c646e616d6573,
                restval: sifr_generated_field_value_e9b6e328a309fdee_7265737476616c,
                extrasaction: sifr_generated_field_value_9e8e4a59c8cf9f24_657874726173616374696f6e,
                writer: sifr_generated_field_value_459f614879d36021_5f777269746572,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2ecsvX2eDictWriter {
        pub fn writeheader(&mut self) {
            let mut current_writer: SifrGeneratedStdlibSifrX2ecsvX2ewriter = self.writer.clone();
            current_writer.writerow(&self.fieldnames.clone());
            self.writer = current_writer;
        }
    }
    impl SifrGeneratedStdlibSifrX2ecsvX2eDictWriter {
        pub fn writerow(&mut self, row: &HashMap<String, String>) {
            let mut ordered: Vec<String> = Vec::new();
            for fieldname in self.fieldnames.iter() {
                if row.contains_key(fieldname) {
                    ordered.push(sifr_generated_dict_value_at(row, fieldname));
                } else {
                    ordered.push(self.restval.clone());
                }
            }
            let mut current_writer: SifrGeneratedStdlibSifrX2ecsvX2ewriter = self.writer.clone();
            current_writer.writerow(&ordered);
            self.writer = current_writer;
        }
    }
    impl SifrGeneratedStdlibSifrX2ecsvX2eDictWriter {
        #[must_use]
        pub fn getvalue(&self) -> String {
            self.writer.getvalue()
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
        pub const fn new(
            year: SifrInt,
            month: SifrInt,
            day: SifrInt,
            hour: SifrInt,
            minute: SifrInt,
            second: SifrInt,
            microsecond: SifrInt,
            tz_offset: Option<SifrInt>,
        ) -> Self {
            let sifr_generated_field_value_7c64634977425edc_79656172: SifrInt = year;
            let sifr_generated_field_value_f4bdc3936faf56a5_6d6f6e7468: SifrInt = month;
            let sifr_generated_field_value_ca8d3918f4578f1d_646179: SifrInt = day;
            let sifr_generated_field_value_407efecc7eb5764f_686f7572: SifrInt = hour;
            let sifr_generated_field_value_5bb2f9bdf2fad1e9_6d696e757465: SifrInt = minute;
            let sifr_generated_field_value_a49985ef4cee20bd_7365636f6e64: SifrInt = second;
            let sifr_generated_field_value_27f934ab879dcfa3_6d6963726f7365636f6e64: SifrInt =
                microsecond;
            let sifr_generated_field_value_17964c5d1d2f9a66_5f747a5f6f6666736574: Option<SifrInt> =
                tz_offset;
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
            if mo.chars().count() < SifrInt::from_i64(2) {
                mo = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize.saturating_add(mo.len()));
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(mo.as_str());
                    sifr_generated_concat
                };
            }
            let mut d: String = self.day.clone().to_string();
            if d.chars().count() < SifrInt::from_i64(2) {
                d = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize.saturating_add(d.len()));
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(d.as_str());
                    sifr_generated_concat
                };
            }
            let mut h: String = self.hour.clone().to_string();
            if h.chars().count() < SifrInt::from_i64(2) {
                h = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize.saturating_add(h.len()));
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(h.as_str());
                    sifr_generated_concat
                };
            }
            let mut mi: String = self.minute.clone().to_string();
            if mi.chars().count() < SifrInt::from_i64(2) {
                mi = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize.saturating_add(mi.len()));
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(mi.as_str());
                    sifr_generated_concat
                };
            }
            let mut s: String = self.second.clone().to_string();
            if s.chars().count() < SifrInt::from_i64(2) {
                s = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize.saturating_add(s.len()));
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(s.as_str());
                    sifr_generated_concat
                };
            }
            let mut base: String = {
                let mut sifr_generated_concat: String = String::with_capacity(
                    y.len()
                        .saturating_add(1usize)
                        .saturating_add(mo.len())
                        .saturating_add(1usize)
                        .saturating_add(d.len())
                        .saturating_add(1usize)
                        .saturating_add(h.len())
                        .saturating_add(1usize)
                        .saturating_add(mi.len())
                        .saturating_add(1usize)
                        .saturating_add(s.len()),
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
            if self.microsecond != SifrInt::from_i64(0) {
                base.push('.');
                base.push_str(sifr_generated_six_digits(self.microsecond.clone()).as_str());
            }
            let tz_offset_opt: Option<SifrInt> = self.tz_offset.clone();
            let Some(tz_offset_opt_value_af7a59df393dc871) = tz_offset_opt else {
                return base;
            };
            let offset: SifrInt = tz_offset_opt_value_af7a59df393dc871;
            let mut sign: String = "+".to_string();
            let mut abs_offset: SifrInt = offset;
            if abs_offset < SifrInt::from_i64(0) {
                sign = "-".to_string();
                abs_offset = ::std::ops::Neg::neg(&abs_offset);
            }
            let h_off: SifrInt = abs_offset.floor_div_known_nonzero(&SifrInt::from_i64(3600));
            let m_off_value_ecbb7903406895aa: SifrInt = abs_offset
                .floor_mod_known_nonzero(&SifrInt::from_i64(3600))
                .floor_div_known_nonzero(&SifrInt::from_i64(60));
            let mut hs_off_value_cdfc32c6642466ee: String = h_off.to_string();
            if hs_off_value_cdfc32c6642466ee.chars().count() < SifrInt::from_i64(2) {
                hs_off_value_cdfc32c6642466ee = {
                    let mut sifr_generated_concat: String = String::with_capacity(
                        1usize.saturating_add(hs_off_value_cdfc32c6642466ee.len()),
                    );
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(hs_off_value_cdfc32c6642466ee.as_str());
                    sifr_generated_concat
                };
            }
            let mut ms_off_value_f9e2b676f4ffcfe7: String =
                m_off_value_ecbb7903406895aa.to_string();
            if ms_off_value_f9e2b676f4ffcfe7.chars().count() < SifrInt::from_i64(2) {
                ms_off_value_f9e2b676f4ffcfe7 = {
                    let mut sifr_generated_concat: String = String::with_capacity(
                        1usize.saturating_add(ms_off_value_f9e2b676f4ffcfe7.len()),
                    );
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(ms_off_value_f9e2b676f4ffcfe7.as_str());
                    sifr_generated_concat
                };
            }
            {
                let mut sifr_generated_concat: String = String::with_capacity(
                    base.len()
                        .saturating_add(sign.len())
                        .saturating_add(hs_off_value_cdfc32c6642466ee.len())
                        .saturating_add(1usize)
                        .saturating_add(ms_off_value_f9e2b676f4ffcfe7.len()),
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
        fn eq(&self, other: &Self) -> bool {
            let same_tz: bool = self.tz_offset == other.tz_offset;
            self.year == other.year
                && self.month == other.month
                && self.day == other.day
                && self.hour == other.hour
                && self.minute == other.minute
                && self.second == other.second
                && self.microsecond == other.microsecond
                && same_tz
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2edatetimeX2edatetime {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.isoformat())
        }
    }
    #[derive(Debug, Clone)]
    pub struct SifrGeneratedStdlibSifrX2edatetimeX2edate {
        pub year: SifrInt,
        pub month: SifrInt,
        pub day: SifrInt,
    }
    impl SifrGeneratedStdlibSifrX2edatetimeX2edate {
        #[must_use]
        pub const fn new(year: SifrInt, month: SifrInt, day: SifrInt) -> Self {
            Self { year, month, day }
        }
    }
    impl SifrGeneratedStdlibSifrX2edatetimeX2edate {
        #[must_use]
        pub fn isoformat(&self) -> String {
            let y: String = self.year.clone().to_string();
            let mut mo: String = self.month.clone().to_string();
            if mo.chars().count() < SifrInt::from_i64(2) {
                mo = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize.saturating_add(mo.len()));
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(mo.as_str());
                    sifr_generated_concat
                };
            }
            let mut d: String = self.day.clone().to_string();
            if d.chars().count() < SifrInt::from_i64(2) {
                d = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize.saturating_add(d.len()));
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(d.as_str());
                    sifr_generated_concat
                };
            }
            {
                let mut sifr_generated_concat: String = String::with_capacity(
                    y.len()
                        .saturating_add(1usize)
                        .saturating_add(mo.len())
                        .saturating_add(1usize)
                        .saturating_add(d.len()),
                );
                sifr_generated_concat.push_str(y.as_str());
                sifr_generated_concat.push('-');
                sifr_generated_concat.push_str(mo.as_str());
                sifr_generated_concat.push('-');
                sifr_generated_concat.push_str(d.as_str());
                sifr_generated_concat
            }
        }
    }
    impl PartialEq for SifrGeneratedStdlibSifrX2edatetimeX2edate {
        fn eq(&self, other: &Self) -> bool {
            self.year == other.year && self.month == other.month && self.day == other.day
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2edatetimeX2edate {
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
            result = result.replace("%(levelname)s", level);
            result = result.replace("%(name)s", name);
            result = result.replace("%(message)s", msg);
            result
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2eloggingX2eFormatter {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "Formatter(_fmt={})", self.fmt)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2eloggingX2eLogger {
        pub name_field: String,
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
            let sifr_generated_field_value_70fb616fceb1e22c_5f6c6576656c: SifrInt = level;
            let sifr_generated_field_value_1fb1dcbc22de0cba_5f6c6f675f70617468: String =
                String::new();
            let sifr_generated_field_value_15ca476e60592199_5f68616e646c65725f6b696e64: String =
                String::new();
            let sifr_generated_field_value_7c86d4c1dd53d2b4_5f68616e646c65725f70617468: String =
                String::new();
            let sifr_generated_field_value_f71817da89e71523_5f68616e646c65725f6c6576656c: SifrInt =
                sifr_generated_const_4e4f54534554();
            let sifr_generated_field_value_98e9bbb8fd5643d6_5f68616e646c65725f666d74: String =
                "%(levelname)s:%(name)s:%(message)s".to_string();
            Self {
                name_field: sifr_generated_field_value_2570757371473f6d_5f6e616d65,
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
        pub fn set_level(&mut self, level: &SifrInt) {
            self.level.clone_from(level);
        }
    }
    impl SifrGeneratedStdlibSifrX2eloggingX2eLogger {
        #[must_use]
        pub fn sifr_generated_handler_allows(&self, level_num: &SifrInt) -> bool {
            if self.handler_level == sifr_generated_const_4e4f54534554() {
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
            formatter.format(level, &self.name_field.clone(), msg)
        }
    }
    impl SifrGeneratedStdlibSifrX2eloggingX2eLogger {
        pub fn sifr_generated_emit(&self, level: &str, level_num: &SifrInt, msg: &str) {
            if &self.level > level_num {
                return;
            }
            if self.handler_kind == "null" {
                return;
            }
            if self.handler_kind == "stream" {
                if self.sifr_generated_handler_allows(level_num) {
                    println!("{}", self.sifr_generated_handler_line(level, msg));
                }
                return;
            }
            if self.handler_kind == "file" {
                if self.sifr_generated_handler_allows(level_num)
                    && !self.handler_path.clone().is_empty()
                {
                    let line: String = {
                        let mut sifr_generated_concat: String =
                            String::with_capacity(0usize.saturating_add(1usize));
                        sifr_generated_concat
                            .push_str(self.sifr_generated_handler_line(level, msg).as_str());
                        sifr_generated_concat.push('\n');
                        sifr_generated_concat
                    };
                    let sifr_generated_try_res: Result<(), IOError> = (|| {
                        let mut fh: SifrGeneratedIoTextFileHandle =
                            open_text(&self.handler_path, "a", &Some(utf8()), &None)?;
                        let sifr_generated_try_res: Result<(), IOError> = (|| {
                            fh.write(&line)?;
                            Ok(())
                        })(
                        );
                        if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                            let e2 = sifr_generated_try_err;
                            let _ = e2.message;
                        }
                        fh.close();
                        Ok(())
                    })();
                    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                        let e = sifr_generated_try_err;
                        let _ = e.message;
                    }
                }
                return;
            }
            let line: String = {
                let mut sifr_generated_concat: String = String::with_capacity(
                    1usize
                        .saturating_add(level.len())
                        .saturating_add(2usize)
                        .saturating_add(0usize)
                        .saturating_add(2usize)
                        .saturating_add(msg.len()),
                );
                sifr_generated_concat.push('[');
                sifr_generated_concat.push_str(level);
                sifr_generated_concat.push_str("] ");
                sifr_generated_concat.push_str(self.name_field.clone().as_str());
                sifr_generated_concat.push_str(": ");
                sifr_generated_concat.push_str(msg);
                sifr_generated_concat
            };
            println!("{line}");
            if !self.log_path.clone().is_empty() {
                let sifr_generated_try_res: Result<(), IOError> = (|| {
                    let mut fh: SifrGeneratedIoTextFileHandle =
                        open_text(&self.log_path, "a", &Some(utf8()), &None)?;
                    let sifr_generated_try_res: Result<(), IOError> = (|| {
                        fh.write(&{
                            let mut sifr_generated_concat: String =
                                String::with_capacity(line.len().saturating_add(1usize));
                            sifr_generated_concat.push_str(line.as_str());
                            sifr_generated_concat.push('\n');
                            sifr_generated_concat
                        })?;
                        Ok(())
                    })();
                    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                        let e2 = sifr_generated_try_err;
                        let _ = e2.message;
                    }
                    fh.close();
                    Ok(())
                })();
                if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                    let e = sifr_generated_try_err;
                    let _ = e.message;
                }
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2eloggingX2eLogger {
        pub fn debug(&self, msg: &str) {
            self.sifr_generated_emit("DEBUG", &sifr_generated_const_4445425547(), msg);
        }
    }
    impl SifrGeneratedStdlibSifrX2eloggingX2eLogger {
        pub fn info(&self, msg: &str) {
            self.sifr_generated_emit("INFO", &sifr_generated_const_494e464f(), msg);
        }
    }
    impl SifrGeneratedStdlibSifrX2eloggingX2eLogger {
        pub fn warning(&self, msg: &str) {
            self.sifr_generated_emit("WARNING", &sifr_generated_const_5741524e494e47(), msg);
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2eloggingX2eLogger {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f,
                "Logger(_name={}, _level={}, _log_path={}, _handler_kind={}, _handler_path={}, _handler_level={}, _handler_fmt={})",
                self.name_field,
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
        #[must_use]
        pub fn exists(&self) -> bool {
            exists(&self.path)
        }
    }
    impl SifrGeneratedStdlibSifrX2epathlibX2ePath {
        #[must_use]
        pub fn to_str(&self) -> String {
            self.path.clone()
        }
    }
    impl SifrGeneratedStdlibSifrX2epathlibX2ePath {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn touch(&self) -> Result<(), IOError> {
            touch(&self.path)
        }
    }
    impl SifrGeneratedStdlibSifrX2epathlibX2ePath {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn unlink(&self) -> Result<(), IOError> {
            remove_file(&self.path)
        }
    }
    impl SifrGeneratedStdlibSifrX2epathlibX2ePath {
        #[must_use]
        pub fn with_name(&self, name: &str) -> Self {
            let parent: String = dirname(&self.path);
            if parent.is_empty() {
                return Self::new(name.to_string());
            }
            Self::new(format!("{parent}/{name}"))
        }
    }
    impl SifrGeneratedStdlibSifrX2epathlibX2ePath {
        #[must_use]
        pub fn with_suffix(&self, suffix: &str) -> Self {
            let s: String = stem(&self.path);
            let parent: String = dirname(&self.path);
            if parent.is_empty() {
                return Self::new(format!("{s}{suffix}"));
            }
            Self::new(format!("{parent}/{s}{suffix}"))
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2epathlibX2ePath {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "Path(_path={})", self.path)
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
        fn findall(&self, text: &str) -> Result<Vec<String>, RegexError> {
            ::sifr_stdlib::regex::compiled_pattern_findall(self, text).map_err(
                |sifr_generated_bridge_error| RegexError {
                    message: sifr_generated_bridge_error.to_string(),
                    detail: sifr_generated_bridge_error.to_string(),
                },
            )
        }
    }
    pub struct SifrGeneratedStdlibSifrX2ereX2ePattern {
        pub compiled: SifrGeneratedStdlibSifrX2eregexX2eCompiledPattern,
        pub pattern: String,
        pub flags: SifrInt,
    }
    impl SifrGeneratedStdlibSifrX2ereX2ePattern {
        #[must_use]
        pub const fn new(
            compiled: SifrGeneratedStdlibSifrX2eregexX2eCompiledPattern,
            pattern: String,
            flags: SifrInt,
        ) -> Self {
            let sifr_generated_field_value_fc19778909466d91_5f636f6d70696c6564: SifrGeneratedStdlibSifrX2eregexX2eCompiledPattern = compiled;
            let sifr_generated_field_value_24bd37eb3fd1d0fc_5f7061747465726e: String = pattern;
            let sifr_generated_field_value_7e89da5111942f49_5f666c616773: SifrInt = flags;
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
    impl SifrGeneratedStdlibSifrX2ereX2ePattern {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn is_match(&self, text: &str) -> Result<bool, RegexError> {
            self.compiled.is_match(text)
        }
    }
    impl SifrGeneratedStdlibSifrX2ereX2ePattern {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn findall(&self, text: &str) -> Result<Vec<String>, RegexError> {
            self.compiled.findall(text)
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
}
use crate::sifr_generated_generated_support::{
    compile, fullmatch, getLogger, sifr_generated_const_4445425547,
};
use ::sifr_runtime::SifrInt;
use ::std::collections::HashMap;
pub use sifr_generated_project_nominals::IOError;
pub use sifr_generated_project_nominals::ParseError;
pub use sifr_generated_project_nominals::RegexError;
pub use sifr_generated_project_nominals::SifrGeneratedIoBinaryFileHandle;
pub use sifr_generated_project_nominals::SifrGeneratedIoTextFileHandle;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ecollectionsX2edeque;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ecsvX2eDialect;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ecsvX2eDictReader;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ecsvX2eDictWriter;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ecsvX2ereader;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ecsvX2ewriter;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2edatetimeX2edate;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2edatetimeX2edatetime;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eencodingX2eEncodeError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eencodingX2eEncodeErrorHandler;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eencodingX2eEncodeOutcome;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eencodingX2eEncoding;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eloggingX2eLogger;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2epathlibX2ePath;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ereX2ePattern;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eregexX2eCompiledPattern;
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    let mut d: SifrGeneratedStdlibSifrX2ecollectionsX2edeque<SifrInt> =
        SifrGeneratedStdlibSifrX2ecollectionsX2edeque::new(None, Some(SifrInt::from_i64(3)));
    d.append(&SifrInt::from_i64(1));
    d.append(&SifrInt::from_i64(2));
    d.append(&SifrInt::from_i64(3));
    d.append(&SifrInt::from_i64(4));
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(23usize.saturating_add(0usize));
        sifr_generated_concat.push_str("deque len (maxlen=3) = ");
        sifr_generated_concat.push_str(d.len().to_string().as_str());
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(16usize.saturating_add(0usize));
        sifr_generated_concat.push_str("deque popleft = ");
        sifr_generated_concat.push_str(
            d.popleft()
                .map_or_else(
                    || "None".to_string(),
                    |sifr_generated_v| sifr_generated_v.to_string(),
                )
                .as_str(),
        );
        sifr_generated_concat
    });
    let dt: SifrGeneratedStdlibSifrX2edatetimeX2edatetime =
        SifrGeneratedStdlibSifrX2edatetimeX2edatetime::new(
            SifrInt::from_i64(2024),
            SifrInt::from_i64(6),
            SifrInt::from_i64(15),
            SifrInt::from_i64(9),
            SifrInt::from_i64(30),
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
            None,
        );
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(21usize.saturating_add(0usize));
        sifr_generated_concat.push_str("datetime isoformat = ");
        sifr_generated_concat.push_str(dt.isoformat().as_str());
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(16usize.saturating_add(0usize));
        sifr_generated_concat.push_str("datetime year = ");
        sifr_generated_concat.push_str(dt.year.to_string().as_str());
        sifr_generated_concat
    });
    let today: SifrGeneratedStdlibSifrX2edatetimeX2edate =
        SifrGeneratedStdlibSifrX2edatetimeX2edate::new(
            SifrInt::from_i64(2024),
            SifrInt::from_i64(6),
            SifrInt::from_i64(15),
        );
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(17usize.saturating_add(0usize));
        sifr_generated_concat.push_str("date isoformat = ");
        sifr_generated_concat.push_str(today.isoformat().as_str());
        sifr_generated_concat
    });
    let p: SifrGeneratedStdlibSifrX2epathlibX2ePath =
        SifrGeneratedStdlibSifrX2epathlibX2ePath::new("/tmp/demo_file.txt".to_string());
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        p.touch()?;
        println!("path touch ok = true");
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(14usize.saturating_add(0usize));
            sifr_generated_concat.push_str("path exists = ");
            sifr_generated_concat.push_str(p.exists().to_string().as_str());
            sifr_generated_concat
        });
        p.unlink()?;
        println!("path unlink ok = true");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(12usize.saturating_add(0usize));
            sifr_generated_concat.push_str("path error: ");
            sifr_generated_concat.push_str(e.message.as_str());
            sifr_generated_concat
        });
    }
    let p2: SifrGeneratedStdlibSifrX2epathlibX2ePath =
        SifrGeneratedStdlibSifrX2epathlibX2ePath::new("/tmp/myfile.txt".to_string());
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(14usize.saturating_add(0usize));
        sifr_generated_concat.push_str("with_suffix = ");
        sifr_generated_concat.push_str(p2.with_suffix(".csv").to_str().as_str());
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(12usize.saturating_add(0usize));
        sifr_generated_concat.push_str("with_name = ");
        sifr_generated_concat.push_str(p2.with_name("other.txt").to_str().as_str());
        sifr_generated_concat
    });
    let sifr_generated_try_res: Result<(), RegexError> = (|| {
        let pat: SifrGeneratedStdlibSifrX2ereX2ePattern = compile("\\d+")?;
        let m: bool = pat.is_match("abc123")?;
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(19usize.saturating_add(0usize));
            sifr_generated_concat.push_str("pattern is_match = ");
            sifr_generated_concat.push_str(m.to_string().as_str());
            sifr_generated_concat
        });
        let found: Option<String> = pat.search("hello 42 world")?;
        if let Some(found) = found {
            println!("{}", {
                let mut sifr_generated_concat: String =
                    String::with_capacity(23usize.saturating_add(0usize));
                sifr_generated_concat.push_str("pattern search found = ");
                sifr_generated_concat.push_str(
                    (found.chars().count() > SifrInt::from_i64(0))
                        .to_string()
                        .as_str(),
                );
                sifr_generated_concat
            });
        }
        let nums: Vec<String> = pat.findall("1 plus 2 equals 3")?;
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(24usize.saturating_add(0usize));
            sifr_generated_concat.push_str("pattern findall count = ");
            sifr_generated_concat.push_str(SifrInt::from(nums.len()).to_string().as_str());
            sifr_generated_concat
        });
        let sifr_generated_try_res: Result<(), RegexError> = (|| {
            let fm_val: bool = fullmatch("\\d+", "12345")?;
            println!("{}", {
                let mut sifr_generated_concat: String =
                    String::with_capacity(19usize.saturating_add(0usize));
                sifr_generated_concat.push_str("fullmatch digits = ");
                sifr_generated_concat.push_str(fm_val.to_string().as_str());
                sifr_generated_concat
            });
            Ok(())
        })();
        if let Err(sifr_generated_try_err) = sifr_generated_try_res {
            let e2 = sifr_generated_try_err;
            println!("{}", {
                let mut sifr_generated_concat: String =
                    String::with_capacity(17usize.saturating_add(0usize));
                sifr_generated_concat.push_str("fullmatch error: ");
                sifr_generated_concat.push_str(e2.message.as_str());
                sifr_generated_concat
            });
        }
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(13usize.saturating_add(0usize));
            sifr_generated_concat.push_str("regex error: ");
            sifr_generated_concat.push_str(e.message.as_str());
            sifr_generated_concat
        });
    }
    let mut log: SifrGeneratedStdlibSifrX2eloggingX2eLogger = getLogger("demo");
    log.set_level(&sifr_generated_const_4445425547());
    log.debug("debug message");
    log.info("info message");
    log.warning("warning message");
    let csv_text: String = "name,age\nalice,30\nbob,25".to_string();
    let r: SifrGeneratedStdlibSifrX2ecsvX2ereader = SifrGeneratedStdlibSifrX2ecsvX2ereader::new(
        csv_text,
        None,
        ",".to_string(),
        "\"".to_string(),
        String::new(),
        true,
        false,
        SifrInt::from_i64(0),
    );
    let all_rows: Vec<Vec<String>> = r.rows();
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(11usize.saturating_add(0usize));
        sifr_generated_concat.push_str("csv rows = ");
        sifr_generated_concat.push_str(SifrInt::from(all_rows.len()).to_string().as_str());
        sifr_generated_concat
    });
    let mut w: SifrGeneratedStdlibSifrX2ecsvX2ewriter = SifrGeneratedStdlibSifrX2ecsvX2ewriter::new(
        None,
        ",".to_string(),
        "\"".to_string(),
        String::new(),
        true,
        false,
        "\n".to_string(),
        SifrInt::from_i64(0),
    );
    let row1: Vec<String> = vec!["x".to_string(), "y".to_string()];
    let row2_value_a3a6c71ff10a162b: Vec<String> = vec!["1".to_string(), "2".to_string()];
    w.writerow(&row1);
    w.writerow(&row2_value_a3a6c71ff10a162b);
    let out: String = w.getvalue();
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(20usize.saturating_add(out.len()));
        sifr_generated_concat.push_str("csv writer output = ");
        sifr_generated_concat.push_str(out.as_str());
        sifr_generated_concat
    });
    let dr: SifrGeneratedStdlibSifrX2ecsvX2eDictReader =
        SifrGeneratedStdlibSifrX2ecsvX2eDictReader::new(
            "name,score\nalice,95\nbob,87".to_string(),
            None,
            String::new(),
            String::new(),
            None,
            ",".to_string(),
            "\"".to_string(),
            String::new(),
            true,
            false,
            SifrInt::from_i64(0),
        );
    let headers: Vec<String> = dr.fieldnames();
    let first_header: Option<String> = {
        let sifr_generated_checked_read_collection = &headers;
        let sifr_generated_checked_read_index = SifrInt::from_i64(0);
        let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
            .normalize_index_or_len(sifr_generated_checked_read_collection.len());
        sifr_generated_checked_read_collection
            .get(sifr_generated_checked_read_normalized)
            .cloned()
    };
    if let Some(first_header) = first_header {
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(21usize.saturating_add(first_header.len()));
            sifr_generated_concat.push_str("dictreader headers = ");
            sifr_generated_concat.push_str(first_header.as_str());
            sifr_generated_concat
        });
    }
    let dict_rows: Vec<HashMap<String, String>> = dr.rows();
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(23usize.saturating_add(0usize));
        sifr_generated_concat.push_str("dictreader row count = ");
        sifr_generated_concat.push_str(SifrInt::from(dict_rows.len()).to_string().as_str());
        sifr_generated_concat
    });
    let mut dw: SifrGeneratedStdlibSifrX2ecsvX2eDictWriter =
        SifrGeneratedStdlibSifrX2ecsvX2eDictWriter::new(
            vec!["name".to_string(), "score".to_string()],
            String::new(),
            "raise".to_string(),
            None,
            ",".to_string(),
            "\"".to_string(),
            String::new(),
            true,
            false,
            "\n".to_string(),
            SifrInt::from_i64(0),
        );
    dw.writeheader();
    let row_data: HashMap<String, String> = HashMap::from([
        ("name".to_string(), "charlie".to_string()),
        ("score".to_string(), "91".to_string()),
    ]);
    dw.writerow(&row_data);
    let dw_out: String = dw.getvalue();
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(20usize.saturating_add(dw_out.len()));
        sifr_generated_concat.push_str("dictwriter output = ");
        sifr_generated_concat.push_str(dw_out.as_str());
        sifr_generated_concat
    });
}
