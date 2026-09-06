// src/main.rs
mod sifr_generated_generated_support {
    use crate::{ParseError, SifrGeneratedStdlibSifrX2etomllibX2eTomlValue, TOMLDecodeError};
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) fn toml_parse_tokens(text: &str) -> Result<Vec<String>, ParseError> {
        ::sifr_stdlib::toml::toml_parse_tokens(text).map_err(|sifr_generated_bridge_error| {
            ParseError {
                message: sifr_generated_bridge_error.to_string(),
            }
        })
    }
    #[derive(Debug, Clone)]
    pub(crate) enum SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass10X3aParseError1X3a0(ParseError),
        SifrGeneratedUnionVariant5X3aclass15X3aTOMLDecodeError1X3a0(TOMLDecodeError),
    }
    impl From<ParseError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0 {
        fn from(value: ParseError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aParseError1X3a0(
                value,
            )
        }
    }
    impl From<TOMLDecodeError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0 {
        fn from(value: TOMLDecodeError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass15X3aTOMLDecodeError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aParseError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass15X3aTOMLDecodeError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
    pub(crate) fn sifr_generated_token_at(
        tokens: &[String],
        index: SifrInt,
    ) -> Result<String, TOMLDecodeError> {
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
            return Err(TOMLDecodeError::new(
                "TOML bridge payload ended unexpectedly".to_string(),
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
    pub(crate) fn sifr_generated_token_int(
        tokens: &[String],
        index: SifrInt,
    ) -> Result<SifrInt, TOMLDecodeError> {
        let sifr_generated_try_res: Result<
            Result<SifrInt, TOMLDecodeError>,
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0,
        > = (|| {
            let token_value_26c4b17d50b3c152: String = sifr_generated_token_at(
                    tokens,
                    index.clone(),
                )
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass15X3aTOMLDecodeError1X3a0,
                )?;
            let parsed: SifrInt = SifrInt::parse_decimal(
                    &token_value_26c4b17d50b3c152,
                    ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
                )
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aParseError1X3a0,
                )?;
            Ok(Ok(parsed))
        })();
        sifr_generated_try_res
            .unwrap_or_else(|sifr_generated_try_err| match sifr_generated_try_err {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aParseError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let _e = sifr_generated_try_variant_error.clone();
                    Err(
                        TOMLDecodeError::new(
                            "TOML bridge payload has invalid integer metadata"
                                .to_string(),
                        ),
                    )
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass15X3aTOMLDecodeError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let e = sifr_generated_try_variant_error.clone();
                    Err(TOMLDecodeError::new(e.message.clone()))
                }
            })
    }
    pub(crate) fn sifr_generated_token_float(
        tokens: &[String],
        index: SifrInt,
    ) -> Result<f64, TOMLDecodeError> {
        let sifr_generated_try_res: Result<
            Result<f64, TOMLDecodeError>,
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0,
        > = (|| {
            let token_value_26c4b17d50b3c152: String = sifr_generated_token_at(
                    tokens,
                    index.clone(),
                )
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass15X3aTOMLDecodeError1X3a0,
                )?;
            let parsed: f64 = token_value_26c4b17d50b3c152
                .parse::<f64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aParseError1X3a0,
                )?;
            Ok(Ok(parsed))
        })();
        sifr_generated_try_res
            .unwrap_or_else(|sifr_generated_try_err| match sifr_generated_try_err {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aParseError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let _e = sifr_generated_try_variant_error.clone();
                    Err(
                        TOMLDecodeError::new(
                            "TOML bridge payload has invalid float metadata".to_string(),
                        ),
                    )
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aParseError1X3a028X3a5X3aclass15X3aTOMLDecodeError1X3a0::SifrGeneratedUnionVariant5X3aclass15X3aTOMLDecodeError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let e = sifr_generated_try_variant_error.clone();
                    Err(TOMLDecodeError::new(e.message.clone()))
                }
            })
    }
    pub(crate) fn sifr_generated_decode_bool_token(value: &str) -> Result<bool, TOMLDecodeError> {
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
    pub(crate) fn sifr_generated_decode_toml_value_at(
        tokens: &[String],
        index: SifrInt,
    ) -> Result<(SifrGeneratedStdlibSifrX2etomllibX2eTomlValue, SifrInt), TOMLDecodeError> {
        let sifr_generated_try_res: Result<
            Result<(SifrGeneratedStdlibSifrX2etomllibX2eTomlValue, SifrInt), TOMLDecodeError>,
            TOMLDecodeError,
        > = (|| {
            let tag: String = sifr_generated_token_at(tokens, index.clone())?;
            let payload_index: SifrInt = &index + &SifrInt::from_i64(1);
            if tag == "bool" {
                let bool_token: String = sifr_generated_token_at(tokens, payload_index.clone())?;
                let bool_value: bool = sifr_generated_decode_bool_token(&bool_token)?;
                return Ok(Ok((
                    SifrGeneratedStdlibSifrX2etomllibX2eTomlValue::new(
                        "bool".to_string(),
                        Some(bool_value),
                        None,
                        None,
                        None,
                        None,
                    ),
                    &payload_index + &SifrInt::from_i64(1),
                )));
            }
            if tag == "int" {
                let int_value: SifrInt = sifr_generated_token_int(tokens, payload_index.clone())?;
                return Ok(Ok((
                    SifrGeneratedStdlibSifrX2etomllibX2eTomlValue::new(
                        "int".to_string(),
                        None,
                        Some(int_value),
                        None,
                        None,
                        None,
                    ),
                    &payload_index + &SifrInt::from_i64(1),
                )));
            }
            if tag == "float" {
                let float_value: f64 = sifr_generated_token_float(tokens, payload_index.clone())?;
                return Ok(Ok((
                    SifrGeneratedStdlibSifrX2etomllibX2eTomlValue::new(
                        "float".to_string(),
                        None,
                        None,
                        Some(float_value),
                        None,
                        None,
                    ),
                    &payload_index + &SifrInt::from_i64(1),
                )));
            }
            if tag == "str" {
                let str_value: String = sifr_generated_token_at(tokens, payload_index.clone())?;
                return Ok(Ok((
                    SifrGeneratedStdlibSifrX2etomllibX2eTomlValue::new(
                        "str".to_string(),
                        None,
                        None,
                        None,
                        Some(str_value),
                        None,
                    ),
                    &payload_index + &SifrInt::from_i64(1),
                )));
            }
            if tag == "datetime" {
                let datetime_value: String =
                    sifr_generated_token_at(tokens, payload_index.clone())?;
                return Ok(Ok((
                    SifrGeneratedStdlibSifrX2etomllibX2eTomlValue::new(
                        "datetime".to_string(),
                        None,
                        None,
                        None,
                        None,
                        Some(datetime_value),
                    ),
                    &payload_index + &SifrInt::from_i64(1),
                )));
            }
            if tag == "array" {
                let array_count: SifrInt = sifr_generated_token_int(tokens, payload_index.clone())?;
                if &array_count < &SifrInt::from_i64(0) {
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
                let mut next_index: SifrInt = &payload_index + &SifrInt::from_i64(1);
                let mut consumed: SifrInt = SifrInt::from_i64(0);
                while &consumed < &array_count {
                    let item_result: (SifrGeneratedStdlibSifrX2etomllibX2eTomlValue, SifrInt) =
                        sifr_generated_decode_toml_value_at(tokens, next_index.clone())?;
                    array_value.array_items.push(item_result.0.clone());
                    next_index = item_result.1.clone();
                    consumed = &consumed + &SifrInt::from_i64(1);
                }
                return Ok(Ok((array_value, next_index.clone())));
            }
            if tag == "table" {
                let table_count: SifrInt = sifr_generated_token_int(tokens, payload_index.clone())?;
                if &table_count < &SifrInt::from_i64(0) {
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
                let mut next_index: SifrInt = &payload_index + &SifrInt::from_i64(1);
                let mut consumed: SifrInt = SifrInt::from_i64(0);
                while &consumed < &table_count {
                    let key: String = sifr_generated_token_at(tokens, next_index.clone())?;
                    let item_result: (SifrGeneratedStdlibSifrX2etomllibX2eTomlValue, SifrInt) =
                        sifr_generated_decode_toml_value_at(
                            tokens,
                            &next_index + &SifrInt::from_i64(1),
                        )?;
                    table_value.table_items.push((key, item_result.0.clone()));
                    next_index = item_result.1.clone();
                    consumed = &consumed + &SifrInt::from_i64(1);
                }
                return Ok(Ok((table_value, next_index)));
            }
            Err(TOMLDecodeError::new({
                let mut sifr_generated_concat: String = String::with_capacity(43usize + tag.len());
                sifr_generated_concat.push_str("TOML bridge payload has unknown value tag: ");
                sifr_generated_concat.push_str(tag.as_str());
                sifr_generated_concat
            }))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err.clone();
            Err(TOMLDecodeError::new(e.message.clone()))
        })
    }
    pub(crate) fn sifr_generated_decode_toml_tokens(
        tokens: &[String],
    ) -> Result<SifrGeneratedStdlibSifrX2etomllibX2eTomlValue, TOMLDecodeError> {
        let sifr_generated_try_res: Result<
            Result<SifrGeneratedStdlibSifrX2etomllibX2eTomlValue, TOMLDecodeError>,
            TOMLDecodeError,
        > = (|| {
            let decoded: (SifrGeneratedStdlibSifrX2etomllibX2eTomlValue, SifrInt) =
                sifr_generated_decode_toml_value_at(tokens, SifrInt::from_i64(0))?;
            if &decoded.1.clone() != &SifrInt::from(tokens.len()) {
                return Err(TOMLDecodeError::new(
                    "TOML bridge payload has trailing data".to_string(),
                ));
            }
            Ok(Ok(decoded.0.clone()))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err.clone();
            Err(TOMLDecodeError::new(e.message.clone()))
        })
    }
    pub(crate) fn loads(
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
            let e = sifr_generated_try_err.clone();
            Err(TOMLDecodeError::new(e.message.clone()))
        })
    }
}
mod sifr_generated_project_nominals {
    use ::sifr_runtime::SifrInt;
    #[derive(Debug, Clone, PartialEq)]
    pub struct SifrGeneratedStdlibSifrX2etomllibX2eTomlValue {
        pub kind: String,
        pub bool_value: Option<bool>,
        pub int_value: Option<SifrInt>,
        pub float_value: Option<f64>,
        pub str_value: Option<String>,
        pub datetime_value: Option<String>,
        pub array_items: Box<Vec<SifrGeneratedStdlibSifrX2etomllibX2eTomlValue>>,
        pub table_items: Box<Vec<(String, SifrGeneratedStdlibSifrX2etomllibX2eTomlValue)>>,
    }
    impl SifrGeneratedStdlibSifrX2etomllibX2eTomlValue {
        #[must_use]
        pub fn new(
            kind: String,
            bool_value: Option<bool>,
            int_value: Option<SifrInt>,
            float_value: Option<f64>,
            str_value: Option<String>,
            datetime_value: Option<String>,
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
            let sifr_generated_field_value_4fa57db663ec4ee2_6461746574696d655f76616c7565: Option<
                String,
            > = datetime_value;
            let sifr_generated_field_value_45232d46c202975d_61727261795f6974656d73: Box<
                Vec<SifrGeneratedStdlibSifrX2etomllibX2eTomlValue>,
            > = Box::default();
            let sifr_generated_field_value_cd3902008971ddd4_7461626c655f6974656d73: Box<
                Vec<(String, SifrGeneratedStdlibSifrX2etomllibX2eTomlValue)>,
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
            self.kind.clone() == "table"
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
        pub fn get(&self, key: &str) -> Option<SifrGeneratedStdlibSifrX2etomllibX2eTomlValue> {
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
}
pub use sifr_generated_project_nominals::ParseError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2etomllibX2eTomlValue;
pub use sifr_generated_project_nominals::TOMLDecodeError;
pub mod helper;
use crate::helper::render;
fn main() {
    println!("{}", render());
}

// src/helper.rs
pub use crate::SifrGeneratedStdlibSifrX2etomllibX2eTomlValue;
pub use crate::TOMLDecodeError;
use crate::sifr_generated_generated_support::*;
#[must_use]
pub fn render() -> String {
    let sifr_generated_try_res: Result<String, TOMLDecodeError> = (|| {
        let parsed: SifrGeneratedStdlibSifrX2etomllibX2eTomlValue =
            loads(&"name = \"five\"\nvalue = 5".to_string())?;
        let name_value: Option<SifrGeneratedStdlibSifrX2etomllibX2eTomlValue> =
            parsed.get(&"name".to_string());
        let value_value: Option<SifrGeneratedStdlibSifrX2etomllibX2eTomlValue> =
            parsed.get(&"value".to_string());
        if let Some(_name_value) = name_value
            && let Some(_value_value) = value_value
        {
            return Ok("dependency closure demo: pass".to_string());
        }
        Ok("dependency closure demo: empty".to_string())
    })();
    sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
        let e = sifr_generated_try_err.clone();
        e.message.clone()
    })
}
