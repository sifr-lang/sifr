// src/main.rs
mod sifr_generated_generated_support {
    use crate::{JSONDecodeError, ParseError, SifrGeneratedStdlibSifrX2ejsonX2eJsonValue};
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) fn json_load_tokens(text: &str) -> Result<Vec<String>, JSONDecodeError> {
        ::sifr_stdlib::json::json_load_tokens(text).map_err(|sifr_generated_bridge_error| {
            JSONDecodeError {
                message: sifr_generated_bridge_error.message().to_string(),
                line: SifrInt::from(sifr_generated_bridge_error.line()),
                column: SifrInt::from(sifr_generated_bridge_error.column()),
            }
        })
    }
    pub(crate) fn json_dump_tokens(tokens: &[String]) -> String {
        ::sifr_stdlib::json::json_dump_tokens(tokens)
    }
    #[derive(Debug, Clone)]
    pub(crate) enum SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass15X3aJSONDecodeError1X3a0(JSONDecodeError),
        SifrGeneratedUnionVariant5X3aclass10X3aParseError1X3a0(ParseError),
    }
    impl From<JSONDecodeError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0 {
        fn from(value: JSONDecodeError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass15X3aJSONDecodeError1X3a0(
                value,
            )
        }
    }
    impl From<ParseError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0 {
        fn from(value: ParseError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aParseError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass15X3aJSONDecodeError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aParseError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub(crate) enum SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0(
            SifrGeneratedStdlibSifrX2ejsonX2eJsonValue,
        ),
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0::SifrGeneratedUnionVariant5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
    pub(crate) fn from_int(value: SifrInt) -> SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        let int_value: Option<SifrInt> = Some(value.clone());
        SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
            "int".to_string(),
            None,
            int_value.clone(),
            None,
            None,
        )
    }
    pub(crate) fn from_str(value: &str) -> SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        let str_value: Option<String> = Some({
            let mut sifr_generated_concat: String = String::with_capacity(value.len());
            sifr_generated_concat.push_str(value);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        });
        SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
            "str".to_string(),
            None,
            None,
            None,
            str_value,
        )
    }
    pub(crate) fn sifr_generated_append_array_item(
        mut value: SifrGeneratedStdlibSifrX2ejsonX2eJsonValue,
        item: SifrGeneratedStdlibSifrX2ejsonX2eJsonValue,
    ) -> SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        value.array_items.push(item);
        value
    }
    pub(crate) fn sifr_generated_append_object_item(
        mut value: SifrGeneratedStdlibSifrX2ejsonX2eJsonValue,
        key: String,
        item_value: SifrGeneratedStdlibSifrX2ejsonX2eJsonValue,
    ) -> SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        value.object_items.push((key, item_value));
        value
    }
    pub(crate) fn from_array(
        items: &[SifrGeneratedStdlibSifrX2ejsonX2eJsonValue],
    ) -> SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        let mut value: SifrGeneratedStdlibSifrX2ejsonX2eJsonValue =
            SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
                "array".to_string(),
                None,
                None,
                None,
                None,
            );
        for item in items.iter().cloned() {
            value = sifr_generated_append_array_item(value, item);
        }
        value
    }
    pub(crate) fn from_object(
        items: &[(String, SifrGeneratedStdlibSifrX2ejsonX2eJsonValue)],
    ) -> SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        let mut value: SifrGeneratedStdlibSifrX2ejsonX2eJsonValue =
            SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
                "object".to_string(),
                None,
                None,
                None,
                None,
            );
        for (key, item_value) in items.iter().cloned() {
            value = sifr_generated_append_object_item(value, key, item_value);
        }
        value
    }
    pub(crate) fn sifr_generated_json_token_at(
        tokens: &[String],
        index: SifrInt,
    ) -> Result<String, JSONDecodeError> {
        let value: Option<String> = {
            let sifr_generated_checked_read_collection = &tokens;
            let sifr_generated_checked_read_index = index.clone();
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
        Ok({
            let mut sifr_generated_concat: String =
                String::with_capacity(value_value_7ce4fd9430e80cea.len());
            sifr_generated_concat.push_str(value_value_7ce4fd9430e80cea.as_str());
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        })
    }
    pub(crate) fn sifr_generated_json_token_int(
        tokens: &[String],
        index: SifrInt,
    ) -> Result<SifrInt, JSONDecodeError> {
        let sifr_generated_try_res: Result<
            Result<SifrInt, JSONDecodeError>,
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0,
        > = (|| {
            let token_value_26c4b17d50b3c152: String = sifr_generated_json_token_at(
                    tokens,
                    index.clone(),
                )
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass15X3aJSONDecodeError1X3a0,
                )?;
            let parsed: SifrInt = SifrInt::parse_decimal(
                    &token_value_26c4b17d50b3c152,
                    ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
                )
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aParseError1X3a0,
                )?;
            Ok(Ok(parsed))
        })();
        sifr_generated_try_res
            .unwrap_or_else(|sifr_generated_try_err| match sifr_generated_try_err {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass15X3aJSONDecodeError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let e = sifr_generated_try_variant_error.clone();
                    Err(JSONDecodeError::new(e.message.clone()))
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aParseError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let _e = sifr_generated_try_variant_error.clone();
                    Err(
                        JSONDecodeError::new(
                            "JSON bridge payload has invalid integer metadata"
                                .to_string(),
                        ),
                    )
                }
            })
    }
    pub(crate) fn sifr_generated_json_token_float(
        tokens: &[String],
        index: SifrInt,
    ) -> Result<f64, JSONDecodeError> {
        let sifr_generated_try_res: Result<
            Result<f64, JSONDecodeError>,
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0,
        > = (|| {
            let token_value_26c4b17d50b3c152: String = sifr_generated_json_token_at(
                    tokens,
                    index.clone(),
                )
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass15X3aJSONDecodeError1X3a0,
                )?;
            let parsed: f64 = token_value_26c4b17d50b3c152
                .parse::<f64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aParseError1X3a0,
                )?;
            Ok(Ok(parsed))
        })();
        sifr_generated_try_res
            .unwrap_or_else(|sifr_generated_try_err| match sifr_generated_try_err {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass15X3aJSONDecodeError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let e = sifr_generated_try_variant_error.clone();
                    Err(JSONDecodeError::new(e.message.clone()))
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aJSONDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aParseError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let _e = sifr_generated_try_variant_error.clone();
                    Err(
                        JSONDecodeError::new(
                            "JSON bridge payload has invalid float metadata".to_string(),
                        ),
                    )
                }
            })
    }
    pub(crate) fn sifr_generated_json_decode_bool_token(
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
    pub(crate) fn sifr_generated_json_decode_value_at(
        tokens: &[String],
        index: SifrInt,
    ) -> Result<(SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, SifrInt), JSONDecodeError> {
        let sifr_generated_try_res: Result<
            Result<(SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, SifrInt), JSONDecodeError>,
            JSONDecodeError,
        > = (|| {
            let tag: String = sifr_generated_json_token_at(tokens, index.clone())?;
            let payload_index: SifrInt = &index + &SifrInt::from_i64(1);
            if tag == "null" {
                return Ok(Ok((
                    SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
                        "null".to_string(),
                        None,
                        None,
                        None,
                        None,
                    ),
                    payload_index.clone(),
                )));
            }
            if tag == "bool" {
                let bool_token: String =
                    sifr_generated_json_token_at(tokens, payload_index.clone())?;
                let bool_value: bool = sifr_generated_json_decode_bool_token(&bool_token)?;
                return Ok(Ok((
                    SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
                        "bool".to_string(),
                        Some(bool_value),
                        None,
                        None,
                        None,
                    ),
                    &payload_index + &SifrInt::from_i64(1),
                )));
            }
            if tag == "int" {
                let int_value: SifrInt =
                    sifr_generated_json_token_int(tokens, payload_index.clone())?;
                return Ok(Ok((
                    SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
                        "int".to_string(),
                        None,
                        Some(int_value),
                        None,
                        None,
                    ),
                    &payload_index + &SifrInt::from_i64(1),
                )));
            }
            if tag == "float" {
                let float_value: f64 =
                    sifr_generated_json_token_float(tokens, payload_index.clone())?;
                return Ok(Ok((
                    SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
                        "float".to_string(),
                        None,
                        None,
                        Some(float_value),
                        None,
                    ),
                    &payload_index + &SifrInt::from_i64(1),
                )));
            }
            if tag == "str" {
                let str_value: String =
                    sifr_generated_json_token_at(tokens, payload_index.clone())?;
                return Ok(Ok((
                    SifrGeneratedStdlibSifrX2ejsonX2eJsonValue::new(
                        "str".to_string(),
                        None,
                        None,
                        None,
                        Some(str_value),
                    ),
                    &payload_index + &SifrInt::from_i64(1),
                )));
            }
            if tag == "array" {
                let array_count: SifrInt =
                    sifr_generated_json_token_int(tokens, payload_index.clone())?;
                if &array_count < &SifrInt::from_i64(0) {
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
                let mut next_index: SifrInt = &payload_index + &SifrInt::from_i64(1);
                let mut consumed: SifrInt = SifrInt::from_i64(0);
                while &consumed < &array_count {
                    let item_result: (SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, SifrInt) =
                        sifr_generated_json_decode_value_at(tokens, next_index.clone())?;
                    array_value.array_items.push(item_result.0.clone());
                    next_index = item_result.1.clone();
                    consumed = &consumed + &SifrInt::from_i64(1);
                }
                return Ok(Ok((array_value, next_index.clone())));
            }
            if tag == "object" {
                let object_count: SifrInt =
                    sifr_generated_json_token_int(tokens, payload_index.clone())?;
                if &object_count < &SifrInt::from_i64(0) {
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
                let mut next_index: SifrInt = &payload_index + &SifrInt::from_i64(1);
                let mut consumed: SifrInt = SifrInt::from_i64(0);
                while &consumed < &object_count {
                    let key: String = sifr_generated_json_token_at(tokens, next_index.clone())?;
                    let item_result: (SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, SifrInt) =
                        sifr_generated_json_decode_value_at(
                            tokens,
                            &next_index + &SifrInt::from_i64(1),
                        )?;
                    object_value.object_items.push((key, item_result.0.clone()));
                    next_index = item_result.1.clone();
                    consumed = &consumed + &SifrInt::from_i64(1);
                }
                return Ok(Ok((object_value, next_index)));
            }
            Err(JSONDecodeError::new({
                let mut sifr_generated_concat: String = String::with_capacity(43usize + tag.len());
                sifr_generated_concat.push_str("JSON bridge payload has unknown value tag: ");
                sifr_generated_concat.push_str(tag.as_str());
                sifr_generated_concat
            }))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err.clone();
            Err(JSONDecodeError::new(e.message.clone()))
        })
    }
    pub(crate) fn sifr_generated_json_decode_tokens(
        tokens: &[String],
    ) -> Result<SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, JSONDecodeError> {
        let sifr_generated_try_res: Result<
            Result<SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, JSONDecodeError>,
            JSONDecodeError,
        > = (|| {
            let decoded: (SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, SifrInt) =
                sifr_generated_json_decode_value_at(tokens, SifrInt::from_i64(0))?;
            if &decoded.1.clone() != &SifrInt::from(tokens.len()) {
                return Err(JSONDecodeError::new(
                    "JSON bridge payload has trailing data".to_string(),
                ));
            }
            Ok(Ok(decoded.0.clone()))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err.clone();
            Err(JSONDecodeError::new(e.message.clone()))
        })
    }
    pub(crate) fn sifr_generated_json_append_tokens(
        mut tokens: Vec<String>,
        value: &SifrGeneratedStdlibSifrX2ejsonX2eJsonValue,
    ) -> Vec<String> {
        tokens.push(value.kind.clone().to_string());
        if value.kind.clone() == "bool" {
            let bool_value: Option<bool> = value.bool_value;
            if bool_value.is_none() {
                tokens.push("false".to_string());
            } else if let Some(bool_value) = bool_value {
                tokens.push(bool_value.to_string().to_lowercase());
            }
        } else if value.kind.clone() == "int" {
            let int_value: Option<SifrInt> = value.int_value.clone();
            if int_value.is_none() {
                tokens.push("0".to_string());
            } else if let Some(int_value) = int_value.clone() {
                tokens.push(int_value.to_string());
            }
        } else if value.kind.clone() == "float" {
            let float_value: Option<f64> = value.float_value;
            if float_value.is_none() {
                tokens.push("0.0".to_string());
            } else if let Some(float_value) = float_value {
                tokens.push(float_value.to_string());
            }
        } else if value.kind.clone() == "str" {
            let str_value: Option<String> = value.as_str();
            if str_value.is_none() {
                tokens.push(String::new());
            } else if let Some(str_value) = str_value {
                tokens.push(str_value);
            }
        } else if value.kind.clone() == "array" {
            tokens.push(SifrInt::from(value.array_items.len()).to_string());
            for item in value.array_items.iter().cloned() {
                tokens = sifr_generated_json_append_tokens(tokens, &item);
            }
        } else if value.kind.clone() == "object" {
            tokens.push(SifrInt::from(value.object_items.len()).to_string());
            for (key, item_value) in value.object_items.iter().cloned() {
                tokens.push(key.to_owned());
                tokens = sifr_generated_json_append_tokens(tokens, &item_value);
            }
        }
        tokens
    }
    pub(crate) fn sifr_generated_json_bridge_tokens(
        value: &SifrGeneratedStdlibSifrX2ejsonX2eJsonValue,
    ) -> Vec<String> {
        let tokens: Vec<String> = Vec::new();
        sifr_generated_json_append_tokens(tokens, value)
    }
    pub(crate) fn sifr_generated_decode_json(
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
            let e = sifr_generated_try_err.clone();
            Err(e)
        })
    }
    pub(crate) fn loads(
        s: &str,
    ) -> Result<SifrGeneratedStdlibSifrX2ejsonX2eJsonValue, JSONDecodeError> {
        sifr_generated_decode_json(s)
    }
    pub(crate) fn dumps(
        value: &SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0,
    ) -> String {
        match value {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0::SifrGeneratedUnionVariant5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0(
                value,
            ) => json_dump_tokens(&sifr_generated_json_bridge_tokens(value)),
        }
    }
}
mod sifr_generated_project_nominals {
    use crate::sifr_generated_generated_support::*;
    use ::sifr_runtime::SifrInt;
    #[derive(Debug, Clone, PartialEq)]
    pub struct SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        pub kind: String,
        pub bool_value: Option<bool>,
        pub int_value: Option<SifrInt>,
        pub float_value: Option<f64>,
        pub str_value: Option<String>,
        pub array_items: Box<Vec<SifrGeneratedStdlibSifrX2ejsonX2eJsonValue>>,
        pub object_items: Box<Vec<(String, SifrGeneratedStdlibSifrX2ejsonX2eJsonValue)>>,
    }
    impl SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        #[must_use]
        pub fn new(
            kind: String,
            bool_value: Option<bool>,
            int_value: Option<SifrInt>,
            float_value: Option<f64>,
            str_value: Option<String>,
        ) -> Self {
            let sifr_generated_field_value_ef9c96d721673243_6b696e64: String = kind;
            let sifr_generated_field_value_49c3632d5fc42247_626f6f6c5f76616c7565: Option<bool> =
                bool_value;
            let sifr_generated_field_value_3e267a8f73b9f8b0_696e745f76616c7565: Option<SifrInt> =
                int_value.clone();
            let sifr_generated_field_value_08384ece94446e4f_666c6f61745f76616c7565: Option<f64> =
                float_value;
            let sifr_generated_field_value_100b36b139835e22_7374725f76616c7565: Option<String> =
                str_value;
            let sifr_generated_field_value_45232d46c202975d_61727261795f6974656d73: Box<
                Vec<SifrGeneratedStdlibSifrX2ejsonX2eJsonValue>,
            > = Box::default();
            let sifr_generated_field_value_4b0f6d30620fe831_6f626a6563745f6974656d73: Box<
                Vec<(String, SifrGeneratedStdlibSifrX2ejsonX2eJsonValue)>,
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
        pub fn is_array(&self) -> bool {
            self.kind.clone() == "array"
        }
    }
    impl SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        #[must_use]
        pub fn is_object(&self) -> bool {
            self.kind.clone() == "object"
        }
    }
    impl SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        #[must_use]
        pub fn as_int(&self) -> Option<SifrInt> {
            self.int_value.clone()
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
        pub fn at(&self, index: &SifrInt) -> Option<SifrGeneratedStdlibSifrX2ejsonX2eJsonValue> {
            if !self.is_array() {
                return None;
            }
            if index < &SifrInt::from_i64(0) || index >= &SifrInt::from(self.array_items.len()) {
                return None;
            }
            {
                let sifr_generated_checked_read_collection = &self.array_items;
                let sifr_generated_checked_read_index = index.clone();
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        #[must_use]
        pub fn get(&self, key: &str) -> Option<SifrGeneratedStdlibSifrX2ejsonX2eJsonValue> {
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
}
use crate::sifr_generated_generated_support::*;
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::JSONDecodeError;
pub use sifr_generated_project_nominals::ParseError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ejsonX2eJsonValue;
fn main() {
    let wrapper_payload: SifrGeneratedStdlibSifrX2ejsonX2eJsonValue = from_object(&vec![
        ("module".to_string(), from_str(&"json".to_string())),
        (
            "items".to_string(),
            from_array(&vec![
                from_int(SifrInt::from_i64(1)),
                from_int(SifrInt::from_i64(2)),
                from_int(SifrInt::from_i64(3)),
            ]),
        ),
    ]);
    let encoded: String = dumps(
        &SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0::SifrGeneratedUnionVariant5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0(
            wrapper_payload.clone(),
        ),
    );
    assert_eq!(
        encoded.as_str(),
        "{\"module\":\"json\",\"items\":[1,2,3]}"
            .to_string()
            .as_str()
    );
    let mut decoded_ok: bool = false;
    let sifr_generated_try_res: Result<(), JSONDecodeError> = (|| {
        let decoded: SifrGeneratedStdlibSifrX2ejsonX2eJsonValue = loads(&encoded)?;
        let module_value: Option<SifrGeneratedStdlibSifrX2ejsonX2eJsonValue> =
            decoded.get(&"module".to_string());
        let items_value: Option<SifrGeneratedStdlibSifrX2ejsonX2eJsonValue> =
            decoded.get(&"items".to_string());
        decoded_ok = module_value.is_some() && items_value.is_some();
        if let Some(module_value) = module_value {
            assert_eq!(module_value.as_str(), Some("json".to_string()));
        }
        if let Some(items_value) = items_value {
            let second: Option<SifrGeneratedStdlibSifrX2ejsonX2eJsonValue> =
                items_value.at(&SifrInt::from_i64(1));
            assert!(second.is_some());
            if let Some(second) = second {
                assert_eq!(second.as_int(), Some(SifrInt::from_i64(2)));
            }
        }
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
    }
    assert!(decoded_ok);
}
