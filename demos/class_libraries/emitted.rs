// src/main.rs
mod sifr_generated_generated_support {
    use crate::{
        IOError, ParseError, SifrGeneratedIoBinaryFileHandle, SifrGeneratedIoNativeFileHandle,
        SifrGeneratedIoTextFileHandle, SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler,
        SifrGeneratedStdlibSifrX2eencodingX2eEncodeError,
        SifrGeneratedStdlibSifrX2eencodingX2eEncodeErrorHandler,
        SifrGeneratedStdlibSifrX2eencodingX2eEncodeOutcome,
        SifrGeneratedStdlibSifrX2eencodingX2eEncoding,
        SifrGeneratedStdlibSifrX2egraphlibX2eCycleError,
        SifrGeneratedStdlibSifrX2eloggingX2eLogger,
    };
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) fn sifr_generated_contains_int(values: &[SifrInt], target: SifrInt) -> bool {
        for value in values.iter().cloned() {
            if &value == &target {
                return true;
            }
        }
        false
    }
    pub(crate) fn topological_sort(
        num_nodes: SifrInt,
        from_nodes: &[SifrInt],
        to_nodes: &[SifrInt],
    ) -> Result<Vec<SifrInt>, SifrGeneratedStdlibSifrX2egraphlibX2eCycleError> {
        let mut result: Vec<SifrInt> = Vec::new();
        let mut visited: Vec<SifrInt> = Vec::new();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &num_nodes {
            visited.push(SifrInt::from_i64(0));
            i = &i + &SifrInt::from_i64(1);
        }
        let mut processed: SifrInt = SifrInt::from_i64(0);
        while &processed < &num_nodes {
            let mut found_any: bool = false;
            let mut node: SifrInt = SifrInt::from_i64(0);
            while &SifrInt::from_i64(0) <= &node && &node < &SifrInt::from(visited.len()) {
                let v: Option<SifrInt> = {
                    let sifr_generated_checked_read_collection = &visited;
                    let sifr_generated_checked_read_index = node.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(v) = v.clone()
                    && &v == &SifrInt::from_i64(0)
                {
                    let mut has_dep: bool = false;
                    let mut j: SifrInt = SifrInt::from_i64(0);
                    while &j < &SifrInt::from(to_nodes.len()) {
                        let to_val: Option<SifrInt> = {
                            let sifr_generated_checked_read_collection = &to_nodes;
                            let sifr_generated_checked_read_index = j.clone();
                            let sifr_generated_checked_read_normalized =
                                sifr_generated_checked_read_index.normalize_index_or_len(
                                    sifr_generated_checked_read_collection.len(),
                                );
                            sifr_generated_checked_read_collection
                                .get(sifr_generated_checked_read_normalized)
                                .cloned()
                        };
                        let from_val: Option<SifrInt> = {
                            let sifr_generated_checked_read_collection = &from_nodes;
                            let sifr_generated_checked_read_index = j.clone();
                            let sifr_generated_checked_read_normalized =
                                sifr_generated_checked_read_index.normalize_index_or_len(
                                    sifr_generated_checked_read_collection.len(),
                                );
                            sifr_generated_checked_read_collection
                                .get(sifr_generated_checked_read_normalized)
                                .cloned()
                        };
                        if let Some(to_val) = to_val.clone()
                            && let Some(from_val) = from_val.clone()
                            && &to_val == &node
                        {
                            let dep_v: Option<SifrInt> = {
                                let sifr_generated_checked_read_collection = &visited;
                                let sifr_generated_checked_read_index = from_val.clone();
                                let sifr_generated_checked_read_normalized =
                                    sifr_generated_checked_read_index.normalize_index_or_len(
                                        sifr_generated_checked_read_collection.len(),
                                    );
                                sifr_generated_checked_read_collection
                                    .get(sifr_generated_checked_read_normalized)
                                    .cloned()
                            };
                            if let Some(dep_v) = dep_v.clone()
                                && &dep_v == &SifrInt::from_i64(0)
                            {
                                has_dep = true;
                            }
                        }
                        j = &j + &SifrInt::from_i64(1);
                    }
                    if !has_dep {
                        result.push(node.clone());
                        {
                            let sifr_generated_assign_value = SifrInt::from_i64(1);
                            {
                                let sifr_generated_index_raw = node.clone();
                                let sifr_generated_index_normalized =
                                    sifr_generated_index_raw.normalize_index_or_len(visited.len());
                                if let Some(sifr_generated_elem) =
                                    visited.get_mut(sifr_generated_index_normalized)
                                {
                                    *sifr_generated_elem = sifr_generated_assign_value;
                                }
                            }
                        }
                        processed = &processed + &SifrInt::from_i64(1);
                        found_any = true;
                    }
                }
                node = &node + &SifrInt::from_i64(1);
            }
            if !found_any {
                return Err(SifrGeneratedStdlibSifrX2egraphlibX2eCycleError::new(
                    "cycle detected in graph".to_string(),
                ));
            }
        }
        Ok(result)
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
    pub(crate) fn get_global_level() -> SifrInt {
        ::sifr_stdlib::logging::get_global_level().into_sifr_int()
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
    pub(crate) const fn sifr_generated_const_4445425547() -> SifrInt {
        SifrInt::from_i64(10)
    }
    pub(crate) const fn sifr_generated_const_494e464f() -> SifrInt {
        SifrInt::from_i64(20)
    }
    pub(crate) const fn sifr_generated_const_5741524e494e47() -> SifrInt {
        SifrInt::from_i64(30)
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
    pub(crate) fn basename(path: &str) -> String {
        let sifr_generated_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
        let mut i: SifrInt =
            &SifrInt::from(sifr_generated_chars_path.len()) - &SifrInt::from_i64(1);
        while &i >= &SifrInt::from_i64(0) {
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
                        (&i + &SifrInt::from_i64(1)).clamp_slice_bound(sifr_generated_slice_len);
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
            i = &i - &SifrInt::from_i64(1);
        }
        {
            let mut sifr_generated_concat: String = String::with_capacity(path.len());
            sifr_generated_concat.push_str(path);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        }
    }
    pub(crate) fn dirname(path: &str) -> String {
        let sifr_generated_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
        let mut i: SifrInt =
            &SifrInt::from(sifr_generated_chars_path.len()) - &SifrInt::from_i64(1);
        while &i >= &SifrInt::from_i64(0) {
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
            i = &i - &SifrInt::from_i64(1);
        }
        String::new()
    }
    pub(crate) fn extension(path: &str) -> String {
        let sifr_generated_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
        let mut i: SifrInt =
            &SifrInt::from(sifr_generated_chars_path.len()) - &SifrInt::from_i64(1);
        while &i >= &SifrInt::from_i64(0) {
            let ch: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_path.len());
                sifr_generated_chars_path
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(ch) = ch {
                if ch == "." {
                    return {
                        let sifr_generated_slice_src = &sifr_generated_chars_path;
                        let sifr_generated_slice_len = sifr_generated_slice_src.len();
                        let sifr_generated_slice_start =
                            i.clamp_slice_bound(sifr_generated_slice_len);
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
                if ch == "/" {
                    return String::new();
                }
            }
            i = &i - &SifrInt::from_i64(1);
        }
        String::new()
    }
    pub(crate) fn stem(path: &str) -> String {
        let base: String = basename(path);
        let sifr_generated_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
        let mut i: SifrInt =
            &SifrInt::from(sifr_generated_chars_base.len()) - &SifrInt::from_i64(1);
        while &i > &SifrInt::from_i64(0) {
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
            i = &i - &SifrInt::from_i64(1);
        }
        {
            let mut sifr_generated_concat: String = String::with_capacity(base.len());
            sifr_generated_concat.push_str(base.as_str());
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        }
    }
    pub(crate) fn is_absolute(path: &str) -> bool {
        let sifr_generated_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
        if &SifrInt::from(sifr_generated_chars_path.len()) == &SifrInt::from_i64(0) {
            return false;
        }
        if &SifrInt::from(sifr_generated_chars_path.len()) >= &SifrInt::from_i64(3) {
            let colon: Option<String> = {
                let sifr_generated_string_index = SifrInt::from_i64(1);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_path.len());
                sifr_generated_chars_path
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            let sep: Option<String> = {
                let sifr_generated_string_index = SifrInt::from_i64(2);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_path.len());
                sifr_generated_chars_path
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(colon) = colon
                && let Some(sep) = sep
                && colon == ":"
                && (sep == "/" || sep == "\\")
            {
                return true;
            }
        }
        let first: Option<String> = {
            let sifr_generated_string_index = SifrInt::from_i64(0);
            let sifr_generated_string_index_normalized =
                sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_path.len());
            sifr_generated_chars_path
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string());
        if let Some(first) = first
            && (first == "/" || first == "\\")
        {
            return true;
        }
        false
    }
    pub(crate) fn sifr_generated_hex_digit_value(ch: &str) -> SifrInt {
        if ch == "0" {
            return SifrInt::from_i64(0);
        }
        if ch == "1" {
            return SifrInt::from_i64(1);
        }
        if ch == "2" {
            return SifrInt::from_i64(2);
        }
        if ch == "3" {
            return SifrInt::from_i64(3);
        }
        if ch == "4" {
            return SifrInt::from_i64(4);
        }
        if ch == "5" {
            return SifrInt::from_i64(5);
        }
        if ch == "6" {
            return SifrInt::from_i64(6);
        }
        if ch == "7" {
            return SifrInt::from_i64(7);
        }
        if ch == "8" {
            return SifrInt::from_i64(8);
        }
        if ch == "9" {
            return SifrInt::from_i64(9);
        }
        if ch == "a" || ch == "A" {
            return SifrInt::from_i64(10);
        }
        if ch == "b" || ch == "B" {
            return SifrInt::from_i64(11);
        }
        if ch == "c" || ch == "C" {
            return SifrInt::from_i64(12);
        }
        if ch == "d" || ch == "D" {
            return SifrInt::from_i64(13);
        }
        if ch == "e" || ch == "E" {
            return SifrInt::from_i64(14);
        }
        if ch == "f" || ch == "F" {
            return SifrInt::from_i64(15);
        }
        -&SifrInt::from_i64(1)
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
    #[derive(Debug, Clone)]
    pub struct SifrGeneratedStdlibSifrX2edatetimeX2etimedelta {
        pub days: SifrInt,
        pub seconds: SifrInt,
        pub microseconds: SifrInt,
    }
    impl SifrGeneratedStdlibSifrX2edatetimeX2etimedelta {
        #[must_use]
        pub fn new(days: SifrInt, seconds: SifrInt, microseconds: SifrInt) -> Self {
            let sifr_generated_field_value_906603c80a0dd39d_5f64617973: SifrInt = days.clone();
            let sifr_generated_field_value_7cbedb13c5d2304b_5f7365636f6e6473: SifrInt =
                seconds.clone();
            let sifr_generated_field_value_fb3e1ecc2972a7bf_5f6d6963726f7365636f6e6473: SifrInt =
                microseconds.clone();
            Self {
                days: sifr_generated_field_value_906603c80a0dd39d_5f64617973,
                seconds: sifr_generated_field_value_7cbedb13c5d2304b_5f7365636f6e6473,
                microseconds:
                    sifr_generated_field_value_fb3e1ecc2972a7bf_5f6d6963726f7365636f6e6473,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2edatetimeX2etimedelta {
        #[must_use]
        pub fn total_seconds(&self) -> SifrInt {
            &(&self.days.clone() * &SifrInt::from_i64(86400)) + &self.seconds.clone()
        }
    }
    impl SifrGeneratedStdlibSifrX2edatetimeX2etimedelta {
        #[must_use]
        pub fn days(&self) -> SifrInt {
            self.days.clone()
        }
    }
    impl SifrGeneratedStdlibSifrX2edatetimeX2etimedelta {
        #[must_use]
        pub fn total_microseconds(&self) -> SifrInt {
            &(&(&(&self.days.clone() * &SifrInt::from_i64(86400)) + &self.seconds.clone())
                * &SifrInt::from_i64(1_000_000))
                + &self.microseconds.clone()
        }
    }
    impl ::std::ops::Add<&SifrGeneratedStdlibSifrX2edatetimeX2etimedelta>
        for &SifrGeneratedStdlibSifrX2edatetimeX2etimedelta
    {
        type Output = SifrGeneratedStdlibSifrX2edatetimeX2etimedelta;
        fn add(self, other: &SifrGeneratedStdlibSifrX2edatetimeX2etimedelta) -> Self::Output {
            let total: SifrInt = &self.total_microseconds() + &other.total_microseconds();
            let d: SifrInt = total.floor_div_known_nonzero(&SifrInt::from_i64(86_400_000_000));
            let remaining: SifrInt =
                total.floor_mod_known_nonzero(&SifrInt::from_i64(86_400_000_000));
            let s: SifrInt = remaining.floor_div_known_nonzero(&SifrInt::from_i64(1_000_000));
            let us: SifrInt = remaining.floor_mod_known_nonzero(&SifrInt::from_i64(1_000_000));
            SifrGeneratedStdlibSifrX2edatetimeX2etimedelta::new(d.clone(), s.clone(), us.clone())
        }
    }
    impl ::std::ops::Sub<&SifrGeneratedStdlibSifrX2edatetimeX2etimedelta>
        for &SifrGeneratedStdlibSifrX2edatetimeX2etimedelta
    {
        type Output = SifrGeneratedStdlibSifrX2edatetimeX2etimedelta;
        fn sub(self, other: &SifrGeneratedStdlibSifrX2edatetimeX2etimedelta) -> Self::Output {
            let total: SifrInt = &self.total_microseconds() - &other.total_microseconds();
            let d: SifrInt = total.floor_div_known_nonzero(&SifrInt::from_i64(86_400_000_000));
            let remaining: SifrInt =
                total.floor_mod_known_nonzero(&SifrInt::from_i64(86_400_000_000));
            let s: SifrInt = remaining.floor_div_known_nonzero(&SifrInt::from_i64(1_000_000));
            let us: SifrInt = remaining.floor_mod_known_nonzero(&SifrInt::from_i64(1_000_000));
            SifrGeneratedStdlibSifrX2edatetimeX2etimedelta::new(d.clone(), s.clone(), us.clone())
        }
    }
    impl PartialEq for SifrGeneratedStdlibSifrX2edatetimeX2etimedelta {
        fn eq(&self, other: &SifrGeneratedStdlibSifrX2edatetimeX2etimedelta) -> bool {
            self.total_microseconds() == other.total_microseconds()
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2edatetimeX2etimedelta {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f,
                "timedelta(_days={}, _seconds={}, _microseconds={})",
                self.days, self.seconds, self.microseconds
            )
        }
    }
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2egraphlibX2eCycleError {
        pub message: String,
    }
    impl SifrGeneratedStdlibSifrX2egraphlibX2eCycleError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Debug for SifrGeneratedStdlibSifrX2egraphlibX2eCycleError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            f.debug_struct("CycleError")
                .field("message", &self.message)
                .finish()
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2egraphlibX2eCycleError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }
    impl ::std::error::Error for SifrGeneratedStdlibSifrX2egraphlibX2eCycleError {}
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SifrGeneratedStdlibSifrX2egraphlibX2eTopologicalSorter {
        pub nodes: Vec<SifrInt>,
        pub from_nodes: Vec<SifrInt>,
        pub to_nodes: Vec<SifrInt>,
        pub max_node: SifrInt,
        pub prepared: bool,
        pub ready_order: Vec<SifrInt>,
        pub next_index: SifrInt,
    }
    impl SifrGeneratedStdlibSifrX2egraphlibX2eTopologicalSorter {
        #[must_use]
        pub fn new() -> Self {
            let sifr_generated_field_value_ca4efc7207239f3a_6e6f646573: Vec<SifrInt> = Vec::new();
            let sifr_generated_field_value_e6fd5a19a19860db_66726f6d5f6e6f646573: Vec<SifrInt> =
                Vec::new();
            let sifr_generated_field_value_10a7723d02448bee_746f5f6e6f646573: Vec<SifrInt> =
                Vec::new();
            let sifr_generated_field_value_329212388287f8ee_6d61785f6e6f6465: SifrInt =
                -SifrInt::from_i64(1);
            let sifr_generated_field_value_d2fc88caa16ddddb_5f7072657061726564: bool = false;
            let sifr_generated_field_value_735ddaefc73fa22e_5f72656164795f6f72646572: Vec<SifrInt> =
                Vec::new();
            let sifr_generated_field_value_6b760d8d62496bd0_5f6e6578745f696e646578: SifrInt =
                SifrInt::from_i64(0);
            Self {
                nodes: sifr_generated_field_value_ca4efc7207239f3a_6e6f646573,
                from_nodes: sifr_generated_field_value_e6fd5a19a19860db_66726f6d5f6e6f646573,
                to_nodes: sifr_generated_field_value_10a7723d02448bee_746f5f6e6f646573,
                max_node: sifr_generated_field_value_329212388287f8ee_6d61785f6e6f6465,
                prepared: sifr_generated_field_value_d2fc88caa16ddddb_5f7072657061726564,
                ready_order: sifr_generated_field_value_735ddaefc73fa22e_5f72656164795f6f72646572,
                next_index: sifr_generated_field_value_6b760d8d62496bd0_5f6e6578745f696e646578,
            }
        }
    }
    impl ::std::default::Default for SifrGeneratedStdlibSifrX2egraphlibX2eTopologicalSorter {
        fn default() -> Self {
            Self::new()
        }
    }
    impl SifrGeneratedStdlibSifrX2egraphlibX2eTopologicalSorter {
        pub fn sifr_generated_record_node(&mut self, node: &SifrInt) {
            if !sifr_generated_contains_int(&self.nodes, node.clone()) {
                self.nodes.push(node.clone());
            }
            if node > &self.max_node {
                self.max_node = node.clone();
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2egraphlibX2eTopologicalSorter {
        pub fn add(&mut self, node: &SifrInt, predecessor: &SifrInt) {
            self.sifr_generated_record_node(node);
            self.sifr_generated_record_node(predecessor);
            self.from_nodes.push(predecessor.clone());
            self.to_nodes.push(node.clone());
            self.prepared = false;
            self.ready_order = Vec::new();
            self.next_index = SifrInt::from_i64(0);
        }
    }
    impl SifrGeneratedStdlibSifrX2egraphlibX2eTopologicalSorter {
        #[must_use]
        pub fn sifr_generated_filter_order(&self, order: &[SifrInt]) -> Vec<SifrInt> {
            let mut filtered: Vec<SifrInt> = Vec::new();
            for candidate in order.iter().cloned() {
                if sifr_generated_contains_int(&self.nodes, candidate.clone()) {
                    filtered.push(candidate.clone());
                }
            }
            filtered
        }
    }
    impl SifrGeneratedStdlibSifrX2egraphlibX2eTopologicalSorter {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn static_order(
            &self,
        ) -> Result<Vec<SifrInt>, SifrGeneratedStdlibSifrX2egraphlibX2eCycleError> {
            if &self.max_node.clone() < &SifrInt::from_i64(0) {
                return Ok(Vec::new());
            }
            let sifr_generated_try_res: Result<
                Result<Vec<SifrInt>, SifrGeneratedStdlibSifrX2egraphlibX2eCycleError>,
                SifrGeneratedStdlibSifrX2egraphlibX2eCycleError,
            > = (|| {
                let full_order: Vec<SifrInt> = topological_sort(
                    &self.max_node.clone() + &SifrInt::from_i64(1),
                    &self.from_nodes,
                    &self.to_nodes,
                )?;
                Ok(Ok(self.sifr_generated_filter_order(&full_order)))
            })();
            sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
                let e = sifr_generated_try_err.clone();
                Err(SifrGeneratedStdlibSifrX2egraphlibX2eCycleError::new(
                    e.message.clone(),
                ))
            })
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
        pub fn set_level(&mut self, level: &SifrInt) {
            self.level = level.clone();
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
        pub fn debug(&self, msg: &str) {
            self.sifr_generated_emit(
                &"DEBUG".to_string(),
                &sifr_generated_const_4445425547(),
                msg,
            );
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
        #[must_use]
        pub fn name(&self) -> String {
            basename(&self.path)
        }
    }
    impl SifrGeneratedStdlibSifrX2epathlibX2ePath {
        #[must_use]
        pub fn parent(&self) -> SifrGeneratedStdlibSifrX2epathlibX2ePath {
            SifrGeneratedStdlibSifrX2epathlibX2ePath::new(dirname(&self.path))
        }
    }
    impl SifrGeneratedStdlibSifrX2epathlibX2ePath {
        #[must_use]
        pub fn suffix(&self) -> String {
            extension(&self.path)
        }
    }
    impl SifrGeneratedStdlibSifrX2epathlibX2ePath {
        #[must_use]
        pub fn stem(&self) -> String {
            stem(&self.path)
        }
    }
    impl SifrGeneratedStdlibSifrX2epathlibX2ePath {
        #[must_use]
        pub fn is_absolute(&self) -> bool {
            is_absolute(&self.path)
        }
    }
    impl SifrGeneratedStdlibSifrX2epathlibX2ePath {
        #[must_use]
        pub fn to_str(&self) -> String {
            {
                let mut sifr_generated_concat: String = String::new();
                sifr_generated_concat.push_str(self.path.clone().as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            }
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2epathlibX2ePath {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "Path(_path={})", self.path)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2ereX2eMatch {
        pub matched: String,
        pub start: SifrInt,
        pub end: SifrInt,
    }
    impl SifrGeneratedStdlibSifrX2ereX2eMatch {
        #[must_use]
        pub fn new(matched: String, start: SifrInt, end: SifrInt) -> Self {
            let sifr_generated_field_value_057487a7ae39ff66_5f6d617463686564: String = matched;
            let sifr_generated_field_value_f46e9a817c26293e_5f7374617274: SifrInt = start.clone();
            let sifr_generated_field_value_3daa7443932b3d2b_5f656e64: SifrInt = end.clone();
            Self {
                matched: sifr_generated_field_value_057487a7ae39ff66_5f6d617463686564,
                start: sifr_generated_field_value_f46e9a817c26293e_5f7374617274,
                end: sifr_generated_field_value_3daa7443932b3d2b_5f656e64,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2ereX2eMatch {
        #[must_use]
        pub fn group(&self) -> String {
            {
                let mut sifr_generated_concat: String = String::new();
                sifr_generated_concat.push_str(self.matched.clone().as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2ereX2eMatch {
        #[must_use]
        pub fn start(&self) -> SifrInt {
            self.start.clone()
        }
    }
    impl SifrGeneratedStdlibSifrX2ereX2eMatch {
        #[must_use]
        pub fn end(&self) -> SifrInt {
            self.end.clone()
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2ereX2eMatch {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f,
                "Match(_matched={}, _start={}, _end={})",
                self.matched, self.start, self.end
            )
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2euuidX2eUUID {
        pub hex: String,
    }
    impl SifrGeneratedStdlibSifrX2euuidX2eUUID {
        #[must_use]
        pub fn new(hex_str: String) -> Self {
            let sifr_generated_field_value_123cb3437a89ad57_5f686578: String = {
                let mut sifr_generated_concat: String = String::with_capacity(hex_str.len());
                sifr_generated_concat.push_str(hex_str.as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            Self {
                hex: sifr_generated_field_value_123cb3437a89ad57_5f686578,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2euuidX2eUUID {
        #[must_use]
        pub fn hex(&self) -> String {
            let mut result: String = String::new();
            let mut i: SifrInt = SifrInt::from_i64(0);
            while &i < &SifrInt::from(self.hex.chars().count()) {
                let ch: Option<String> = {
                    let sifr_generated_string_chars = self.hex.chars().collect::<Vec<char>>();
                    let sifr_generated_string_index = i.clone();
                    let sifr_generated_string_index_normalized = sifr_generated_string_index
                        .normalize_index_or_len(sifr_generated_string_chars.len());
                    sifr_generated_string_chars
                        .get(sifr_generated_string_index_normalized)
                        .copied()
                }
                .map(|character| character.to_string());
                if let Some(ch) = ch
                    && ch != "-"
                {
                    result.push_str(ch.as_str());
                }
                i = &i + &SifrInt::from_i64(1);
            }
            result
        }
    }
    impl SifrGeneratedStdlibSifrX2euuidX2eUUID {
        #[must_use]
        pub fn version(&self) -> SifrInt {
            let marker: Option<String> = {
                let sifr_generated_string_chars = self.hex.chars().collect::<Vec<char>>();
                let sifr_generated_string_index = SifrInt::from_i64(14);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_string_chars.len());
                sifr_generated_string_chars
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            let Some(marker_value_eddcb72b15486e77) = marker else {
                return -&SifrInt::from_i64(1);
            };
            sifr_generated_hex_digit_value(&marker_value_eddcb72b15486e77)
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2euuidX2eUUID {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "UUID(_hex={})", self.hex)
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
}
use crate::sifr_generated_generated_support::*;
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::IOError;
pub use sifr_generated_project_nominals::ParseError;
pub use sifr_generated_project_nominals::SifrGeneratedIoBinaryFileHandle;
pub use sifr_generated_project_nominals::SifrGeneratedIoTextFileHandle;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2edatetimeX2etimedelta;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eencodingX2eDecodeErrorHandler;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eencodingX2eEncodeError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eencodingX2eEncodeErrorHandler;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eencodingX2eEncodeOutcome;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eencodingX2eEncoding;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2egraphlibX2eCycleError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2egraphlibX2eTopologicalSorter;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eloggingX2eLogger;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2epathlibX2ePath;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ereX2eMatch;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2euuidX2eUUID;
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    println!("=== TopologicalSorter ===");
    let mut ts: SifrGeneratedStdlibSifrX2egraphlibX2eTopologicalSorter =
        SifrGeneratedStdlibSifrX2egraphlibX2eTopologicalSorter::new();
    (&mut ts).add(&SifrInt::from_i64(1), &SifrInt::from_i64(0));
    (&mut ts).add(&SifrInt::from_i64(2), &SifrInt::from_i64(1));
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2egraphlibX2eCycleError> =
        (|| {
            let order: Vec<SifrInt> = ts.static_order()?;
            let first: Option<SifrInt> = {
                let sifr_generated_checked_read_collection = &order;
                let sifr_generated_checked_read_index = SifrInt::from_i64(0);
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            let second: Option<SifrInt> = {
                let sifr_generated_checked_read_collection = &order;
                let sifr_generated_checked_read_index = SifrInt::from_i64(1);
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            let third: Option<SifrInt> = {
                let sifr_generated_checked_read_collection = &order;
                let sifr_generated_checked_read_index = SifrInt::from_i64(2);
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            if let Some(first) = first.clone() {
                println!("{first}");
            }
            if let Some(second) = second.clone() {
                println!("{second}");
            }
            if let Some(third) = third.clone() {
                println!("{third}");
            }
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let err = sifr_generated_try_err.clone();
        println!("cycle error: {}", err.message.clone());
    }
    println!("=== Path ===");
    let p: SifrGeneratedStdlibSifrX2epathlibX2ePath =
        SifrGeneratedStdlibSifrX2epathlibX2ePath::new("/home/user/docs/report.pdf".to_string());
    println!("{}", p.name());
    println!("{}", p.parent().to_str());
    println!("{}", p.suffix());
    println!("{}", p.stem());
    println!("{}", p.is_absolute());
    println!("=== Logger ===");
    let mut log: SifrGeneratedStdlibSifrX2eloggingX2eLogger = getLogger(&"demo".to_string());
    log.info(&"application started".to_string());
    log.warning(&"disk space low".to_string());
    log.debug(&"this should not appear at INFO level".to_string());
    (&mut log).set_level(&SifrInt::from_i64(10));
    log.debug(&"now visible after level change".to_string());
    println!("=== Match ===");
    let m: SifrGeneratedStdlibSifrX2ereX2eMatch = SifrGeneratedStdlibSifrX2ereX2eMatch::new(
        "world".to_string(),
        SifrInt::from_i64(6),
        SifrInt::from_i64(11),
    );
    println!("{}", m.group());
    println!("{}", m.start());
    println!("{}", m.end());
    println!("=== UUID ===");
    let u: SifrGeneratedStdlibSifrX2euuidX2eUUID = SifrGeneratedStdlibSifrX2euuidX2eUUID::new(
        "550e8400-e29b-41d4-a716-446655440000".to_string(),
    );
    println!("{}", u.hex());
    println!("{}", u.version());
    println!("=== timedelta ===");
    let one_day: SifrGeneratedStdlibSifrX2edatetimeX2etimedelta =
        SifrGeneratedStdlibSifrX2edatetimeX2etimedelta::new(
            SifrInt::from_i64(1),
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
        );
    let two_hours: SifrGeneratedStdlibSifrX2edatetimeX2etimedelta =
        SifrGeneratedStdlibSifrX2edatetimeX2etimedelta::new(
            SifrInt::from_i64(0),
            SifrInt::from_i64(7200),
            SifrInt::from_i64(0),
        );
    let combined: SifrGeneratedStdlibSifrX2edatetimeX2etimedelta = &one_day + &two_hours;
    println!("{}", combined.total_seconds());
    println!("{}", combined.days());
    let diff: SifrGeneratedStdlibSifrX2edatetimeX2etimedelta = &one_day - &two_hours;
    println!("{}", diff.total_seconds());
    println!(
        "{}",
        one_day
            == SifrGeneratedStdlibSifrX2edatetimeX2etimedelta::new(
                SifrInt::from_i64(1),
                SifrInt::from_i64(0),
                SifrInt::from_i64(0)
            )
    );
}
