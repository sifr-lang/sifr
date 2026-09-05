// src/main.rs
pub mod sifr_generated_generated_support {
    use crate::{
        JSONDecodeError, ParseError, RegexError, SifrGeneratedStdlibSifrX2ejsonX2eJsonValue,
        SifrGeneratedStdlibSifrX2etomllibX2eTomlValue, TOMLDecodeError,
    };
    pub(super) use ::sifr_runtime::SifrInt;
    pub(super) fn base64_encode(s: &str) -> String {
        ::sifr_stdlib::base64::base64_encode(s)
    }
    pub(super) fn base64_decode(s: &str) -> Result<String, ParseError> {
        ::sifr_stdlib::base64::base64_decode(s).map_err(|sifr_generated_bridge_error| ParseError {
            message: sifr_generated_bridge_error,
        })
    }
    pub(super) fn b64encode(s: &str) -> String {
        base64_encode(s)
    }
    pub(super) fn b64decode(s: &str) -> Result<String, ParseError> {
        base64_decode(s)
    }
    pub(super) fn json_load_tokens(text: &str) -> Result<Vec<String>, JSONDecodeError> {
        ::sifr_stdlib::json::json_load_tokens(text).map_err(|sifr_generated_bridge_error| {
            JSONDecodeError {
                message: sifr_generated_bridge_error.message().to_string(),
                line: SifrInt::from(sifr_generated_bridge_error.line()),
                column: SifrInt::from(sifr_generated_bridge_error.column()),
            }
        })
    }
    pub(super) fn json_dump_tokens(tokens: &[String]) -> String {
        ::sifr_stdlib::json::json_dump_tokens(tokens)
    }
    #[derive(Debug, Clone)]
    pub(super) enum SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass28X3asifrX2ebuiltinX2eJSONDecodeError1X3a0(
            JSONDecodeError,
        ),
        SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a0(ParseError),
    }
    impl From<JSONDecodeError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0 {
        fn from(value: JSONDecodeError) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass28X3asifrX2ebuiltinX2eJSONDecodeError1X3a0(
                value,
            )
        }
    }
    impl From<ParseError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0 {
        fn from(value: ParseError) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                Self::SifrGeneratedUnionVariant5X3aclass28X3asifrX2ebuiltinX2eJSONDecodeError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                Self::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub(super) enum SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0
    {
        SifrGeneratedUnionVariant4X3aatom3X3aint(SifrInt),
        SifrGeneratedUnionVariant5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0(
            SifrGeneratedStdlibSifrX2ejsonX2eJsonValue,
        ),
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                Self::SifrGeneratedUnionVariant4X3aatom3X3aint(v) => write!(f, "{v}"),
                Self::SifrGeneratedUnionVariant5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
    pub(super) fn from_int(value: &SifrInt) -> SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        let int_value: Option<SifrInt> = Some((*value).clone());
        SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
            "int".to_string(),
            None,
            int_value.as_ref(),
            None,
            None,
        )
    }
    pub(super) fn sifr_generated_json_token_at(
        tokens: &[String],
        index: &SifrInt,
    ) -> Result<String, JSONDecodeError> {
        let value: Option<String> = {
            let sifr_generated_checked_read_collection = &tokens;
            let sifr_generated_checked_read_index = (*index).clone();
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        };
        let Some(value_value_7ce4fd9430e80cea) = value else {
            return Err(JSONDecodeError::new(
                "JSON bridge payload ended unexpectedly".to_string(),
            ));
        };
        Ok(value_value_7ce4fd9430e80cea)
    }
    pub(super) fn sifr_generated_json_token_int(
        tokens: &[String],
        index: &SifrInt,
    ) -> Result<SifrInt, JSONDecodeError> {
        let sifr_generated_try_res: Result<
            Result<SifrInt, JSONDecodeError>,
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0,
        > = (|| {
            let token_value_26c4b17d50b3c152: String = sifr_generated_json_token_at(
                    tokens,
                    index,
                )
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass28X3asifrX2ebuiltinX2eJSONDecodeError1X3a0,
                )?;
            let parsed: SifrInt = SifrInt::parse_decimal(
                    token_value_26c4b17d50b3c152.as_str(),
                    ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
                )
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a0,
                )?;
            Ok(Ok(parsed))
        })();
        sifr_generated_try_res
            .unwrap_or_else(|sifr_generated_try_err| match sifr_generated_try_err {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass28X3asifrX2ebuiltinX2eJSONDecodeError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let e = sifr_generated_try_variant_error;
                    Err(JSONDecodeError::new(e.message))
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let _e = sifr_generated_try_variant_error;
                    Err(
                        JSONDecodeError::new(
                            "JSON bridge payload has invalid integer metadata"
                                .to_string(),
                        ),
                    )
                }
            })
    }
    pub(super) fn sifr_generated_json_token_float(
        tokens: &[String],
        index: &SifrInt,
    ) -> Result<f64, JSONDecodeError> {
        let sifr_generated_try_res: Result<
            Result<f64, JSONDecodeError>,
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0,
        > = (|| {
            let token_value_26c4b17d50b3c152: String = sifr_generated_json_token_at(
                    tokens,
                    index,
                )
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass28X3asifrX2ebuiltinX2eJSONDecodeError1X3a0,
                )?;
            let parsed: f64 = token_value_26c4b17d50b3c152
                .parse::<f64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a0,
                )?;
            Ok(Ok(parsed))
        })();
        sifr_generated_try_res
            .unwrap_or_else(|sifr_generated_try_err| match sifr_generated_try_err {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass28X3asifrX2ebuiltinX2eJSONDecodeError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let e = sifr_generated_try_variant_error;
                    Err(JSONDecodeError::new(e.message))
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let _e = sifr_generated_try_variant_error;
                    Err(
                        JSONDecodeError::new(
                            "JSON bridge payload has invalid float metadata".to_string(),
                        ),
                    )
                }
            })
    }
    pub(super) fn sifr_generated_json_decode_bool_token(
        value: &str,
    ) -> Result<bool, JSONDecodeError> {
        if value == "true" {
            return Ok(true);
        }
        if value == "false" {
            return Ok(false);
        }
        Err(JSONDecodeError::new(
            "JSON bridge payload has invalid bool metadata".to_string(),
        ))
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub(super) fn sifr_generated_json_decode_value_at(
        tokens: &[String],
        index: &SifrInt,
    ) -> Result<(SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, SifrInt), JSONDecodeError> {
        let sifr_generated_try_res: Result<
            Result<(SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, SifrInt), JSONDecodeError>,
            JSONDecodeError,
        > = (|| {
            let tag: String = sifr_generated_json_token_at(tokens, index)?;
            let payload_index: SifrInt = ::std::ops::Add::add(index, &SifrInt::from_i64(1));
            if tag == "null" {
                return Ok(Ok((
                    SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
                        "null".to_string(),
                        None,
                        None,
                        None,
                        None,
                    ),
                    payload_index,
                )));
            }
            if tag == "bool" {
                let bool_token: String = sifr_generated_json_token_at(tokens, &payload_index)?;
                let bool_value: bool = sifr_generated_json_decode_bool_token(bool_token.as_str())?;
                return Ok(Ok((
                    SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
                        "bool".to_string(),
                        Some(bool_value),
                        None,
                        None,
                        None,
                    ),
                    ::std::ops::Add::add(&payload_index, &SifrInt::from_i64(1)),
                )));
            }
            if tag == "int" {
                let int_value: SifrInt = sifr_generated_json_token_int(tokens, &payload_index)?;
                return Ok(Ok((
                    SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
                        "int".to_string(),
                        None,
                        Some(&int_value),
                        None,
                        None,
                    ),
                    ::std::ops::Add::add(&payload_index, &SifrInt::from_i64(1)),
                )));
            }
            if tag == "float" {
                let float_value: f64 = sifr_generated_json_token_float(tokens, &payload_index)?;
                return Ok(Ok((
                    SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
                        "float".to_string(),
                        None,
                        None,
                        Some(float_value),
                        None,
                    ),
                    ::std::ops::Add::add(&payload_index, &SifrInt::from_i64(1)),
                )));
            }
            if tag == "str" {
                let str_value: String = sifr_generated_json_token_at(tokens, &payload_index)?;
                return Ok(Ok((
                    SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
                        "str".to_string(),
                        None,
                        None,
                        None,
                        Some(str_value),
                    ),
                    ::std::ops::Add::add(&payload_index, &SifrInt::from_i64(1)),
                )));
            }
            if tag == "array" {
                let array_count: SifrInt = sifr_generated_json_token_int(tokens, &payload_index)?;
                if array_count < SifrInt::from_i64(0) {
                    return Err(JSONDecodeError::new(
                        "JSON bridge payload has invalid array length".to_string(),
                    ));
                }
                let mut array_value: SifrGeneratedStdlibSifrX2ejsonX2eJsonValue =
                    SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
                        "array".to_string(),
                        None,
                        None,
                        None,
                        None,
                    );
                let mut next_index: SifrInt =
                    ::std::ops::Add::add(&payload_index, &SifrInt::from_i64(1));
                let mut consumed: SifrInt = SifrInt::from_i64(0);
                while consumed < array_count {
                    let item_result: (SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, SifrInt) =
                        sifr_generated_json_decode_value_at(tokens, &next_index)?;
                    array_value.array_items.push(item_result.0);
                    next_index.clone_from(&item_result.1);
                    consumed = ::std::ops::Add::add(&consumed, &SifrInt::from_i64(1));
                }
                return Ok(Ok((array_value, next_index)));
            }
            if tag == "object" {
                let object_count: SifrInt = sifr_generated_json_token_int(tokens, &payload_index)?;
                if object_count < SifrInt::from_i64(0) {
                    return Err(JSONDecodeError::new(
                        "JSON bridge payload has invalid object length".to_string(),
                    ));
                }
                let mut object_value: SifrGeneratedStdlibSifrX2ejsonX2eJsonValue =
                    SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
                        "object".to_string(),
                        None,
                        None,
                        None,
                        None,
                    );
                let mut next_index: SifrInt =
                    ::std::ops::Add::add(&payload_index, &SifrInt::from_i64(1));
                let mut consumed: SifrInt = SifrInt::from_i64(0);
                while consumed < object_count {
                    let key: String = sifr_generated_json_token_at(tokens, &next_index)?;
                    let item_result: (SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, SifrInt) =
                        sifr_generated_json_decode_value_at(
                            tokens,
                            &::std::ops::Add::add(&next_index, &SifrInt::from_i64(1)),
                        )?;
                    object_value.object_items.push((key, item_result.0));
                    next_index.clone_from(&item_result.1);
                    consumed = ::std::ops::Add::add(&consumed, &SifrInt::from_i64(1));
                }
                return Ok(Ok((object_value, next_index)));
            }
            Err(JSONDecodeError::new({
                let mut sifr_generated_concat: String =
                    String::with_capacity(43usize.saturating_add(tag.len()));
                sifr_generated_concat.push_str("JSON bridge payload has unknown value tag: ");
                sifr_generated_concat.push_str(tag.as_str());
                sifr_generated_concat
            }))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err;
            Err(JSONDecodeError::new(e.message))
        })
    }
    pub(super) fn sifr_generated_json_decode_tokens(
        tokens: &[String],
    ) -> Result<SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, JSONDecodeError> {
        let sifr_generated_try_res: Result<
            Result<SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, JSONDecodeError>,
            JSONDecodeError,
        > = (|| {
            let decoded: (SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, SifrInt) =
                sifr_generated_json_decode_value_at(tokens, &SifrInt::from_i64(0))?;
            if decoded.1 != tokens.len() {
                return Err(JSONDecodeError::new(
                    "JSON bridge payload has trailing data".to_string(),
                ));
            }
            Ok(Ok(decoded.0))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err;
            Err(JSONDecodeError::new(e.message))
        })
    }
    pub(super) fn sifr_generated_json_append_tokens(
        mut tokens: Vec<String>,
        value: &SifrGeneratedStdlibSifrX2ejsonX2eJsonValue,
    ) -> Vec<String> {
        tokens.push(value.kind.clone());
        if value.kind == "bool" {
            let bool_value: Option<bool> = value.bool_value;
            if bool_value.is_none() {
                tokens.push("false".to_string());
            } else if let Some(bool_value) = bool_value {
                tokens.push(bool_value.to_string().to_lowercase());
            }
        } else if value.kind == "int" {
            let int_value: Option<SifrInt> = value.int_value.clone();
            if int_value.is_none() {
                tokens.push("0".to_string());
            } else if let Some(int_value) = int_value {
                tokens.push(int_value.to_string());
            }
        } else if value.kind == "float" {
            let float_value: Option<f64> = value.float_value;
            if float_value.is_none() {
                tokens.push("0.0".to_string());
            } else if let Some(float_value) = float_value {
                tokens.push(float_value.to_string());
            }
        } else if value.kind == "str" {
            let str_value: Option<String> = value.as_str();
            if str_value.is_none() {
                tokens.push(String::new());
            } else if let Some(str_value) = str_value {
                tokens.push(str_value);
            }
        } else if value.kind == "array" {
            tokens.push(SifrInt::from(value.array_items.len()).to_string());
            for item in value.array_items.iter() {
                tokens = sifr_generated_json_append_tokens(tokens, item);
            }
        } else if value.kind == "object" {
            tokens.push(SifrInt::from(value.object_items.len()).to_string());
            for (key, item_value) in value.object_items.iter() {
                tokens.push(key.clone());
                tokens = sifr_generated_json_append_tokens(tokens, item_value);
            }
        }
        tokens
    }
    pub(super) fn sifr_generated_json_bridge_tokens(
        value: &SifrGeneratedStdlibSifrX2ejsonX2eJsonValue,
    ) -> Vec<String> {
        let tokens: Vec<String> = Vec::new();
        sifr_generated_json_append_tokens(tokens, value)
    }
    pub(super) fn sifr_generated_decode_json(
        s: &str,
    ) -> Result<SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, JSONDecodeError> {
        let sifr_generated_try_res: Result<
            Result<SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, JSONDecodeError>,
            JSONDecodeError,
        > = (|| {
            let tokens: Vec<String> = json_load_tokens(s)?;
            Ok(sifr_generated_json_decode_tokens(&tokens))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err;
            Err(e)
        })
    }
    pub(super) fn dumps(
        value: &SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0,
    ) -> String {
        match value {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0::SifrGeneratedUnionVariant5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0(
                value,
            ) => json_dump_tokens(&sifr_generated_json_bridge_tokens(value)),
            SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0::SifrGeneratedUnionVariant4X3aatom3X3aint(
                value,
            ) => json_dump_tokens(&sifr_generated_json_bridge_tokens(&from_int(value))),
        }
    }
    pub(super) fn re_find(pattern: &str, text: &str) -> Result<Option<String>, RegexError> {
        ::sifr_stdlib::regex::re_find(pattern, text).map_err(|sifr_generated_bridge_error| {
            RegexError {
                message: sifr_generated_bridge_error.to_string(),
                detail: sifr_generated_bridge_error.to_string(),
            }
        })
    }
    pub(super) fn re_replace(
        pattern: &str,
        replacement: &str,
        text: &str,
    ) -> Result<String, RegexError> {
        ::sifr_stdlib::regex::re_replace(pattern, replacement, text).map_err(
            |sifr_generated_bridge_error| RegexError {
                message: sifr_generated_bridge_error.to_string(),
                detail: sifr_generated_bridge_error.to_string(),
            },
        )
    }
    pub(super) fn re_findall(pattern: &str, text: &str) -> Result<Vec<String>, RegexError> {
        ::sifr_stdlib::regex::re_findall(pattern, text).map_err(|sifr_generated_bridge_error| {
            RegexError {
                message: sifr_generated_bridge_error.to_string(),
                detail: sifr_generated_bridge_error.to_string(),
            }
        })
    }
    pub(super) fn re_split(pattern: &str, text: &str) -> Result<Vec<String>, RegexError> {
        ::sifr_stdlib::regex::re_split(pattern, text).map_err(|sifr_generated_bridge_error| {
            RegexError {
                message: sifr_generated_bridge_error.to_string(),
                detail: sifr_generated_bridge_error.to_string(),
            }
        })
    }
    pub(super) fn search(pattern: &str, text: &str) -> Result<Option<String>, RegexError> {
        re_find(pattern, text)
    }
    pub(super) fn sub(pattern: &str, replacement: &str, text: &str) -> Result<String, RegexError> {
        re_replace(pattern, replacement, text)
    }
    pub(super) fn findall(pattern: &str, text: &str) -> Result<Vec<String>, RegexError> {
        re_findall(pattern, text)
    }
    pub(super) fn split(pattern: &str, text: &str) -> Result<Vec<String>, RegexError> {
        re_split(pattern, text)
    }
    pub(super) fn toml_parse_tokens(text: &str) -> Result<Vec<String>, ParseError> {
        ::sifr_stdlib::toml::toml_parse_tokens(text).map_err(|sifr_generated_bridge_error| {
            ParseError {
                message: sifr_generated_bridge_error,
            }
        })
    }
    #[derive(Debug, Clone)]
    pub(super) enum SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a0(ParseError),
        SifrGeneratedUnionVariant5X3aclass28X3asifrX2ebuiltinX2eTOMLDecodeError1X3a0(
            TOMLDecodeError,
        ),
    }
    impl From<ParseError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0 {
        fn from(value: ParseError) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a0(
                value,
            )
        }
    }
    impl From<TOMLDecodeError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0 {
        fn from(value: TOMLDecodeError) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass28X3asifrX2ebuiltinX2eTOMLDecodeError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                Self::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                Self::SifrGeneratedUnionVariant5X3aclass28X3asifrX2ebuiltinX2eTOMLDecodeError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
    pub(super) fn sifr_generated_token_at(
        tokens: &[String],
        index: &SifrInt,
    ) -> Result<String, TOMLDecodeError> {
        let value: Option<String> = {
            let sifr_generated_checked_read_collection = &tokens;
            let sifr_generated_checked_read_index = (*index).clone();
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        };
        let Some(value_value_7ce4fd9430e80cea) = value else {
            return Err(TOMLDecodeError::new(
                "TOML bridge payload ended unexpectedly".to_string(),
            ));
        };
        Ok(value_value_7ce4fd9430e80cea)
    }
    pub(super) fn sifr_generated_token_int(
        tokens: &[String],
        index: &SifrInt,
    ) -> Result<SifrInt, TOMLDecodeError> {
        let sifr_generated_try_res: Result<
            Result<SifrInt, TOMLDecodeError>,
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0,
        > = (|| {
            let token_value_26c4b17d50b3c152: String = sifr_generated_token_at(
                    tokens,
                    index,
                )
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass28X3asifrX2ebuiltinX2eTOMLDecodeError1X3a0,
                )?;
            let parsed: SifrInt = SifrInt::parse_decimal(
                    token_value_26c4b17d50b3c152.as_str(),
                    ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
                )
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a0,
                )?;
            Ok(Ok(parsed))
        })();
        sifr_generated_try_res
            .unwrap_or_else(|sifr_generated_try_err| match sifr_generated_try_err {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let _e = sifr_generated_try_variant_error;
                    Err(
                        TOMLDecodeError::new(
                            "TOML bridge payload has invalid integer metadata"
                                .to_string(),
                        ),
                    )
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass28X3asifrX2ebuiltinX2eTOMLDecodeError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let e = sifr_generated_try_variant_error;
                    Err(TOMLDecodeError::new(e.message))
                }
            })
    }
    pub(super) fn sifr_generated_token_float(
        tokens: &[String],
        index: &SifrInt,
    ) -> Result<f64, TOMLDecodeError> {
        let sifr_generated_try_res: Result<
            Result<f64, TOMLDecodeError>,
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0,
        > = (|| {
            let token_value_26c4b17d50b3c152: String = sifr_generated_token_at(
                    tokens,
                    index,
                )
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass28X3asifrX2ebuiltinX2eTOMLDecodeError1X3a0,
                )?;
            let parsed: f64 = token_value_26c4b17d50b3c152
                .parse::<f64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a0,
                )?;
            Ok(Ok(parsed))
        })();
        sifr_generated_try_res
            .unwrap_or_else(|sifr_generated_try_err| match sifr_generated_try_err {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eParseError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let _e = sifr_generated_try_variant_error;
                    Err(
                        TOMLDecodeError::new(
                            "TOML bridge payload has invalid float metadata".to_string(),
                        ),
                    )
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass28X3asifrX2ebuiltinX2eTOMLDecodeError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let e = sifr_generated_try_variant_error;
                    Err(TOMLDecodeError::new(e.message))
                }
            })
    }
    pub(super) fn sifr_generated_decode_bool_token(value: &str) -> Result<bool, TOMLDecodeError> {
        if value == "true" {
            return Ok(true);
        }
        if value == "false" {
            return Ok(false);
        }
        Err(TOMLDecodeError::new(
            "TOML bridge payload has invalid bool metadata".to_string(),
        ))
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub(super) fn sifr_generated_decode_toml_value_at(
        tokens: &[String],
        index: &SifrInt,
    ) -> Result<(SifrGeneratedStdlibSifrX2etomllibX2eTomlValue, SifrInt), TOMLDecodeError> {
        let sifr_generated_try_res: Result<
            Result<(SifrGeneratedStdlibSifrX2etomllibX2eTomlValue, SifrInt), TOMLDecodeError>,
            TOMLDecodeError,
        > = (|| {
            let tag: String = sifr_generated_token_at(tokens, index)?;
            let payload_index: SifrInt = ::std::ops::Add::add(index, &SifrInt::from_i64(1));
            if tag == "bool" {
                let bool_token: String = sifr_generated_token_at(tokens, &payload_index)?;
                let bool_value: bool = sifr_generated_decode_bool_token(bool_token.as_str())?;
                return Ok(Ok((
                    SifrGeneratedStdlibSifrX2etomllibX2eTomlValue::new(
                        "bool".to_string(),
                        Some(bool_value),
                        None,
                        None,
                        None,
                        None,
                    ),
                    ::std::ops::Add::add(&payload_index, &SifrInt::from_i64(1)),
                )));
            }
            if tag == "int" {
                let int_value: SifrInt = sifr_generated_token_int(tokens, &payload_index)?;
                return Ok(Ok((
                    SifrGeneratedStdlibSifrX2etomllibX2eTomlValue::new(
                        "int".to_string(),
                        None,
                        Some(&int_value),
                        None,
                        None,
                        None,
                    ),
                    ::std::ops::Add::add(&payload_index, &SifrInt::from_i64(1)),
                )));
            }
            if tag == "float" {
                let float_value: f64 = sifr_generated_token_float(tokens, &payload_index)?;
                return Ok(Ok((
                    SifrGeneratedStdlibSifrX2etomllibX2eTomlValue::new(
                        "float".to_string(),
                        None,
                        None,
                        Some(float_value),
                        None,
                        None,
                    ),
                    ::std::ops::Add::add(&payload_index, &SifrInt::from_i64(1)),
                )));
            }
            if tag == "str" {
                let str_value: String = sifr_generated_token_at(tokens, &payload_index)?;
                return Ok(Ok((
                    SifrGeneratedStdlibSifrX2etomllibX2eTomlValue::new(
                        "str".to_string(),
                        None,
                        None,
                        None,
                        Some(str_value),
                        None,
                    ),
                    ::std::ops::Add::add(&payload_index, &SifrInt::from_i64(1)),
                )));
            }
            if tag == "datetime" {
                let datetime_value: String = sifr_generated_token_at(tokens, &payload_index)?;
                return Ok(Ok((
                    SifrGeneratedStdlibSifrX2etomllibX2eTomlValue::new(
                        "datetime".to_string(),
                        None,
                        None,
                        None,
                        None,
                        Some(datetime_value),
                    ),
                    ::std::ops::Add::add(&payload_index, &SifrInt::from_i64(1)),
                )));
            }
            if tag == "array" {
                let array_count: SifrInt = sifr_generated_token_int(tokens, &payload_index)?;
                if array_count < SifrInt::from_i64(0) {
                    return Err(TOMLDecodeError::new(
                        "TOML bridge payload has invalid array length".to_string(),
                    ));
                }
                let mut array_value: SifrGeneratedStdlibSifrX2etomllibX2eTomlValue =
                    SifrGeneratedStdlibSifrX2etomllibX2eTomlValue::new(
                        "array".to_string(),
                        None,
                        None,
                        None,
                        None,
                        None,
                    );
                let mut next_index: SifrInt =
                    ::std::ops::Add::add(&payload_index, &SifrInt::from_i64(1));
                let mut consumed: SifrInt = SifrInt::from_i64(0);
                while consumed < array_count {
                    let item_result: (SifrGeneratedStdlibSifrX2etomllibX2eTomlValue, SifrInt) =
                        sifr_generated_decode_toml_value_at(tokens, &next_index)?;
                    array_value.array_items.push(item_result.0);
                    next_index.clone_from(&item_result.1);
                    consumed = ::std::ops::Add::add(&consumed, &SifrInt::from_i64(1));
                }
                return Ok(Ok((array_value, next_index)));
            }
            if tag == "table" {
                let table_count: SifrInt = sifr_generated_token_int(tokens, &payload_index)?;
                if table_count < SifrInt::from_i64(0) {
                    return Err(TOMLDecodeError::new(
                        "TOML bridge payload has invalid table length".to_string(),
                    ));
                }
                let mut table_value: SifrGeneratedStdlibSifrX2etomllibX2eTomlValue =
                    SifrGeneratedStdlibSifrX2etomllibX2eTomlValue::new(
                        "table".to_string(),
                        None,
                        None,
                        None,
                        None,
                        None,
                    );
                let mut next_index: SifrInt =
                    ::std::ops::Add::add(&payload_index, &SifrInt::from_i64(1));
                let mut consumed: SifrInt = SifrInt::from_i64(0);
                while consumed < table_count {
                    let key: String = sifr_generated_token_at(tokens, &next_index)?;
                    let item_result: (SifrGeneratedStdlibSifrX2etomllibX2eTomlValue, SifrInt) =
                        sifr_generated_decode_toml_value_at(
                            tokens,
                            &::std::ops::Add::add(&next_index, &SifrInt::from_i64(1)),
                        )?;
                    table_value.table_items.push((key, item_result.0));
                    next_index.clone_from(&item_result.1);
                    consumed = ::std::ops::Add::add(&consumed, &SifrInt::from_i64(1));
                }
                return Ok(Ok((table_value, next_index)));
            }
            Err(TOMLDecodeError::new({
                let mut sifr_generated_concat: String =
                    String::with_capacity(43usize.saturating_add(tag.len()));
                sifr_generated_concat.push_str("TOML bridge payload has unknown value tag: ");
                sifr_generated_concat.push_str(tag.as_str());
                sifr_generated_concat
            }))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err;
            Err(TOMLDecodeError::new(e.message))
        })
    }
    pub(super) fn sifr_generated_decode_toml_tokens(
        tokens: &[String],
    ) -> Result<SifrGeneratedStdlibSifrX2etomllibX2eTomlValue, TOMLDecodeError> {
        let sifr_generated_try_res: Result<
            Result<SifrGeneratedStdlibSifrX2etomllibX2eTomlValue, TOMLDecodeError>,
            TOMLDecodeError,
        > = (|| {
            let decoded: (SifrGeneratedStdlibSifrX2etomllibX2eTomlValue, SifrInt) =
                sifr_generated_decode_toml_value_at(tokens, &SifrInt::from_i64(0))?;
            if decoded.1 != tokens.len() {
                return Err(TOMLDecodeError::new(
                    "TOML bridge payload has trailing data".to_string(),
                ));
            }
            Ok(Ok(decoded.0))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err;
            Err(TOMLDecodeError::new(e.message))
        })
    }
    pub(super) fn loads(
        text: &str,
    ) -> Result<SifrGeneratedStdlibSifrX2etomllibX2eTomlValue, TOMLDecodeError> {
        let sifr_generated_try_res: Result<
            Result<SifrGeneratedStdlibSifrX2etomllibX2eTomlValue, TOMLDecodeError>,
            ParseError,
        > = (|| {
            let tokens: Vec<String> = toml_parse_tokens(text)?;
            Ok(sifr_generated_decode_toml_tokens(&tokens))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err;
            Err(TOMLDecodeError::new(e.message))
        })
    }
}
mod sifr_generated_project_nominals {
    use crate::sifr_generated_generated_support::{
        SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0,
        dumps, sifr_generated_decode_json,
    };
    use ::sifr_runtime::SifrInt;
    #[derive(Debug, Clone, PartialEq)]
    pub struct SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        pub kind: String,
        pub bool_value: Option<bool>,
        pub int_value: Option<SifrInt>,
        pub float_value: Option<f64>,
        pub str_value: Option<String>,
        pub array_items: Box<Vec<Self>>,
        pub object_items: Box<Vec<(String, Self)>>,
    }
    impl SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        #[must_use]
        pub fn new(
            kind: String,
            bool_value: Option<bool>,
            int_value: Option<&SifrInt>,
            float_value: Option<f64>,
            str_value: Option<String>,
        ) -> Self {
            let int_value: Option<SifrInt> = int_value.cloned();
            let sifr_generated_field_value_ef9c96d721673243_6b696e64: String = kind;
            let sifr_generated_field_value_49c3632d5fc42247_626f6f6c5f76616c7565: Option<bool> =
                bool_value;
            let sifr_generated_field_value_3e267a8f73b9f8b0_696e745f76616c7565: Option<SifrInt> =
                int_value;
            let sifr_generated_field_value_08384ece94446e4f_666c6f61745f76616c7565: Option<f64> =
                float_value;
            let sifr_generated_field_value_100b36b139835e22_7374725f76616c7565: Option<String> =
                str_value;
            let sifr_generated_field_value_45232d46c202975d_61727261795f6974656d73: Box<Vec<Self>> =
                Box::default();
            let sifr_generated_field_value_4b0f6d30620fe831_6f626a6563745f6974656d73: Box<
                Vec<(String, Self)>,
            > = Box::default();
            Self {
                kind: sifr_generated_field_value_ef9c96d721673243_6b696e64,
                bool_value: sifr_generated_field_value_49c3632d5fc42247_626f6f6c5f76616c7565,
                int_value: sifr_generated_field_value_3e267a8f73b9f8b0_696e745f76616c7565,
                float_value: sifr_generated_field_value_08384ece94446e4f_666c6f61745f76616c7565,
                str_value: sifr_generated_field_value_100b36b139835e22_7374725f76616c7565,
                array_items: sifr_generated_field_value_45232d46c202975d_61727261795f6974656d73,
                object_items: sifr_generated_field_value_4b0f6d30620fe831_6f626a6563745f6974656d73,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        #[must_use]
        pub fn is_object(&self) -> bool {
            self.kind == "object"
        }
    }
    impl SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        #[must_use]
        pub fn as_str(&self) -> Option<String> {
            self.str_value.clone()
        }
    }
    impl SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        #[must_use]
        pub fn get(&self, key: &str) -> Option<Self> {
            if !self.is_object() {
                return None;
            }
            for (item_key, item_value) in self.object_items.iter().cloned() {
                if item_key == *key {
                    return Some(item_value);
                }
            }
            None
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f, "{}", dumps(&
                SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0::SifrGeneratedUnionVariant5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0(self
                .clone()))
            )
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2ejsonX2eJSONDecoder {}
    impl SifrGeneratedStdlibSifrX2ejsonX2eJSONDecoder {
        #[must_use]
        pub const fn new() -> Self {
            Self {}
        }
    }
    impl ::std::default::Default for SifrGeneratedStdlibSifrX2ejsonX2eJSONDecoder {
        fn default() -> Self {
            Self::new()
        }
    }
    impl SifrGeneratedStdlibSifrX2ejsonX2eJSONDecoder {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn decode(
            &self,
            s: &str,
        ) -> Result<SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, JSONDecodeError> {
            sifr_generated_decode_json(s)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct SifrGeneratedStdlibSifrX2etomllibX2eTomlValue {
        pub kind: String,
        pub bool_value: Option<bool>,
        pub int_value: Option<SifrInt>,
        pub float_value: Option<f64>,
        pub str_value: Option<String>,
        pub datetime_value: Option<String>,
        pub array_items: Box<Vec<Self>>,
        pub table_items: Box<Vec<(String, Self)>>,
    }
    impl SifrGeneratedStdlibSifrX2etomllibX2eTomlValue {
        #[must_use]
        pub fn new(
            kind: String,
            bool_value: Option<bool>,
            int_value: Option<&SifrInt>,
            float_value: Option<f64>,
            str_value: Option<String>,
            datetime_value: Option<String>,
        ) -> Self {
            let int_value: Option<SifrInt> = int_value.cloned();
            let sifr_generated_field_value_ef9c96d721673243_6b696e64: String = kind;
            let sifr_generated_field_value_49c3632d5fc42247_626f6f6c5f76616c7565: Option<bool> =
                bool_value;
            let sifr_generated_field_value_3e267a8f73b9f8b0_696e745f76616c7565: Option<SifrInt> =
                int_value;
            let sifr_generated_field_value_08384ece94446e4f_666c6f61745f76616c7565: Option<f64> =
                float_value;
            let sifr_generated_field_value_100b36b139835e22_7374725f76616c7565: Option<String> =
                str_value;
            let sifr_generated_field_value_4fa57db663ec4ee2_6461746574696d655f76616c7565: Option<
                String,
            > = datetime_value;
            let sifr_generated_field_value_45232d46c202975d_61727261795f6974656d73: Box<Vec<Self>> =
                Box::default();
            let sifr_generated_field_value_cd3902008971ddd4_7461626c655f6974656d73: Box<
                Vec<(String, Self)>,
            > = Box::default();
            Self {
                kind: sifr_generated_field_value_ef9c96d721673243_6b696e64,
                bool_value: sifr_generated_field_value_49c3632d5fc42247_626f6f6c5f76616c7565,
                int_value: sifr_generated_field_value_3e267a8f73b9f8b0_696e745f76616c7565,
                float_value: sifr_generated_field_value_08384ece94446e4f_666c6f61745f76616c7565,
                str_value: sifr_generated_field_value_100b36b139835e22_7374725f76616c7565,
                datetime_value:
                    sifr_generated_field_value_4fa57db663ec4ee2_6461746574696d655f76616c7565,
                array_items: sifr_generated_field_value_45232d46c202975d_61727261795f6974656d73,
                table_items: sifr_generated_field_value_cd3902008971ddd4_7461626c655f6974656d73,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2etomllibX2eTomlValue {
        #[must_use]
        pub fn is_table(&self) -> bool {
            self.kind == "table"
        }
    }
    impl SifrGeneratedStdlibSifrX2etomllibX2eTomlValue {
        #[must_use]
        pub fn as_str(&self) -> Option<String> {
            self.str_value.clone()
        }
    }
    impl SifrGeneratedStdlibSifrX2etomllibX2eTomlValue {
        #[must_use]
        pub fn get(&self, key: &str) -> Option<Self> {
            if !self.is_table() {
                return None;
            }
            for (item_key, item_value) in self.table_items.iter().cloned() {
                if item_key == *key {
                    return Some(item_value);
                }
            }
            None
        }
    }
    impl SifrGeneratedStdlibSifrX2etomllibX2eTomlValue {
        #[must_use]
        pub fn keys(&self) -> Vec<String> {
            let mut result: Vec<String> = Vec::new();
            if !self.is_table() {
                return result;
            }
            for (item_key, _item_value) in self.table_items.iter() {
                result.push(item_key.clone());
            }
            result
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
    pub struct JSONDecodeError {
        pub message: String,
        pub line: SifrInt,
        pub column: SifrInt,
    }
    impl JSONDecodeError {
        #[must_use]
        pub const fn new(message: String) -> Self {
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
    pub struct TOMLDecodeError {
        pub message: String,
        pub line: SifrInt,
        pub column: SifrInt,
    }
    impl TOMLDecodeError {
        #[must_use]
        pub const fn new(message: String) -> Self {
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
    SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0,
    b64decode, b64encode, dumps, findall, loads, search, split, sub,
};
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::JSONDecodeError;
pub use sifr_generated_project_nominals::ParseError;
pub use sifr_generated_project_nominals::RegexError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ejsonX2eJSONDecoder;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ejsonX2eJsonValue;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2etomllibX2eTomlValue;
pub use sifr_generated_project_nominals::TOMLDecodeError;
fn sifr_generated_load_json(
    text: &str,
) -> Result<SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, JSONDecodeError> {
    let decoder: SifrGeneratedStdlibSifrX2ejsonX2eJSONDecoder =
        SifrGeneratedStdlibSifrX2ejsonX2eJSONDecoder::new();
    decoder.decode(text)
}
fn has_match(pattern: &str, text: &str) -> Result<bool, RegexError> {
    let sifr_generated_try_res: Result<Result<bool, RegexError>, RegexError> = (|| {
        let found: Option<String> = search(pattern, text)?;
        Ok(Ok(found.is_some()))
    })();
    sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
        let error = sifr_generated_try_err;
        Err(RegexError::new(error.message))
    })
}
fn demo_json() {
    println!("=== JSON Parse Safety ===");
    let sifr_generated_try_res: Result<(), JSONDecodeError> = (|| {
        let data: SifrGeneratedStdlibSifrX2ejsonX2eJsonValue =
            sifr_generated_load_json("{\"language\":\"sifr\",\"safe\":true}")?;
        println!("parsed: {data}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("error: {}", e.message);
    }
    let sifr_generated_try_res: Result<(), JSONDecodeError> = (|| {
        let bad: SifrGeneratedStdlibSifrX2ejsonX2eJsonValue =
            sifr_generated_load_json("{not valid json")?;
        let _ = bad;
        println!("should not reach here");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("caught JSONDecodeError: {}", e.message);
    }
    let dumped: String = dumps(
        &SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0::SifrGeneratedUnionVariant4X3aatom3X3aint(
            SifrInt::from_i64(42),
        ),
    );
    println!("dumped: {dumped}");
}
fn demo_toml() {
    println!("=== TOML Parse Safety ===");
    let sifr_generated_try_res: Result<(), TOMLDecodeError> = (|| {
        let toml_data: SifrGeneratedStdlibSifrX2etomllibX2eTomlValue =
            loads("name = \"sifr\"\nversion = 1")?;
        println!(
            "toml parsed: {}",
            toml_data.keys().len() > SifrInt::from_i64(0)
        );
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("error: {}", e.message);
    }
    let sifr_generated_try_res: Result<(), TOMLDecodeError> = (|| {
        let bad_toml: SifrGeneratedStdlibSifrX2etomllibX2eTomlValue = loads("[broken toml ===")?;
        let _ = bad_toml;
        println!("should not reach here");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("caught TOMLDecodeError: {}", e.message);
    }
}
fn demo_regex() {
    println!("=== Regex Safety ===");
    let sifr_generated_try_res: Result<(), RegexError> = (|| {
        let matched: bool = has_match("\\d+", "abc123")?;
        println!("match found: {matched}");
        let found: Option<String> = search("\\d+", "hello 42 world")?;
        if let Some(found) = found {
            println!("found: {found}");
        }
        let replaced: String = sub("\\d+", "NUM", "test 1 2 3")?;
        println!("replaced: {replaced}");
        let all_matches: Vec<String> = findall("[a-z]+", "Hello World Sifr")?;
        println!("findall count: {}", SifrInt::from(all_matches.len()));
        let parts: Vec<String> = split(",", "a,b,c")?;
        println!("split count: {}", SifrInt::from(parts.len()));
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("unexpected: {}", e.message);
    }
    let sifr_generated_try_res: Result<(), RegexError> = (|| {
        let _bad_match: bool = has_match("[unclosed", "text")?;
        println!("should not reach here");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("caught RegexError: {}", e.message);
    }
}
fn demo_base64() {
    println!("=== Base64 Safety ===");
    let encoded: String = b64encode("safe decoding!");
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let decoded: String = b64decode(encoded.as_str())?;
        println!("decoded: {decoded}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("unexpected: {}", e.message);
    }
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let _bad_decoded: String = b64decode("!!!not-base64!!!")?;
        println!("should not reach here");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("caught base64 ParseError: {}", e.message);
    }
}
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn demo_bytes() {
    println!("=== Bytes Safety ===");
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let text: String = ::sifr_runtime::encoding::decode_text(
            &[
                104_u8, 101_u8, 108_u8, 108_u8, 111_u8, 32_u8, 115_u8, 105_u8, 102_u8, 114_u8,
            ],
            "utf-8",
            "strict",
        )
        .map_err(|sifr_generated_message| ParseError {
            message: sifr_generated_message,
        })?;
        println!("utf8: {text}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("unexpected: {}", e.message);
    }
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let bad_bytes: Vec<u8> = vec![255u8, 254u8, 253u8];
        let _bad_text: String = ::sifr_runtime::encoding::decode_text(
            &bad_bytes, "utf-8", "strict",
        )
        .map_err(|sifr_generated_message| ParseError {
            message: sifr_generated_message,
        })?;
        println!("should not reach here");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("caught utf8 ParseError: {}", e.message);
    }
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let hex_data: Vec<u8> = {
            let s: &str = "48656c6c6f";
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
            if !cleaned.len().is_multiple_of(2) {
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
        let decoded_hex: String = ::sifr_runtime::encoding::decode_text(
            &hex_data, "utf-8", "strict",
        )
        .map_err(|sifr_generated_message| ParseError {
            message: sifr_generated_message,
        })?;
        println!("from hex: {decoded_hex}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("unexpected: {}", e.message);
    }
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let _bad_hex: Vec<u8> = {
            let s: &str = "ZZZZ";
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
            if !cleaned.len().is_multiple_of(2) {
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
        println!("should not reach here");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("caught hex ParseError: {}", e.message);
    }
}
fn main() {
    demo_json();
    demo_toml();
    demo_regex();
    demo_base64();
    demo_bytes();
    println!("=== All parse safety demos passed ===");
}
