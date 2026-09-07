// src/main.rs
mod sifr_generated_generated_support {
    use crate::{
        FloatOverflowError, FloatPrecisionLossError, IOError, ParseError, RegexError,
        SifrGeneratedStdlibSifrX2edatetimeX2edatetime,
        SifrGeneratedStdlibSifrX2edatetimeX2etimezone,
        SifrGeneratedStdlibSifrX2egraphlibX2eCycleError,
        SifrGeneratedStdlibSifrX2etomllibX2eTomlValue,
        SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0,
        TOMLDecodeError, ValueError,
    };
    pub(crate) use ::sifr_runtime::SifrInt;
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
    pub(crate) fn perf_counter() -> f64 {
        ::sifr_stdlib::time::perf_counter()
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
    pub(crate) fn from_timestamp(
        ts: f64,
        tz: &Option<SifrGeneratedStdlibSifrX2edatetimeX2etimezone>,
    ) -> Result<SifrGeneratedStdlibSifrX2edatetimeX2edatetime, ValueError> {
        sifr_generated_from_timestamp_with_tz(ts, tz)
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub(crate) fn get_close_matches(
        word: &str,
        possibilities: &[String],
        n: SifrInt,
        cutoff: f64,
    ) -> Result<
        Vec<String>,
        SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0,
    >{
        let mut result: Vec<String> = Vec::new();
        let mut scores: Vec<f64> = Vec::new();
        for candidate in possibilities.iter().cloned() {
            let sifr_generated_try_res: Result<
                (f64,),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0,
            > = (|| {
                let score_value_c10e63fbe7f624f5: f64 = sifr_generated_similarity(
                    word,
                    &candidate,
                )?;
                Ok((score_value_c10e63fbe7f624f5,))
            })();
            let (score_value_c10e63fbe7f624f5_binding,) = match sifr_generated_try_res {
                Ok(sifr_generated_try_bindings) => sifr_generated_try_bindings,
                Err(sifr_generated_try_err) => {
                    match sifr_generated_try_err {
                        SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                            sifr_generated_try_variant_error,
                        ) => {
                            let error = sifr_generated_try_variant_error.clone();
                            return Err(
                                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                                    FloatOverflowError::new(error.message.clone()),
                                ),
                            );
                        }
                        SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                            sifr_generated_try_variant_error,
                        ) => {
                            let error = sifr_generated_try_variant_error.clone();
                            return Err(
                                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                                    FloatPrecisionLossError::new(error.message.clone()),
                                ),
                            );
                        }
                    }
                }
            };
            if score_value_c10e63fbe7f624f5_binding >= cutoff {
                result.push(candidate);
                scores.push(score_value_c10e63fbe7f624f5_binding);
            }
        }
        if &SifrInt::from(result.len()) <= &n {
            return Ok(result);
        }
        let mut top: Vec<String> = Vec::new();
        let mut used: Vec<SifrInt> = Vec::new();
        let mut count: SifrInt = SifrInt::from_i64(0);
        while &count < &n {
            let mut best_idx: SifrInt = -&SifrInt::from_i64(1);
            let mut best_score: f64 = -1.0_f64;
            let mut i: SifrInt = SifrInt::from_i64(0);
            while &i < &SifrInt::from(scores.len()) {
                let mut skip: bool = false;
                for u in used.iter().cloned() {
                    if &u == &i {
                        skip = true;
                    }
                }
                if !skip {
                    let s: Option<f64> = {
                        let sifr_generated_checked_read_collection = &scores;
                        let sifr_generated_checked_read_index = i.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(s) = s
                        && s > best_score
                    {
                        best_score = s;
                        best_idx = i.clone();
                    }
                }
                i = &i + &SifrInt::from_i64(1);
            }
            if &best_idx >= &SifrInt::from_i64(0) {
                used.push(best_idx.clone());
                let val: Option<String> = {
                    let sifr_generated_checked_read_collection = &result;
                    let sifr_generated_checked_read_index = best_idx.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(val) = val {
                    top.push(val);
                }
            }
            count = &count + &SifrInt::from_i64(1);
        }
        Ok(top)
    }
    pub(crate) fn sifr_generated_similarity(
        a: &str,
        b: &str,
    ) -> Result<
        f64,
        SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0,
    >{
        let sifr_generated_chars_a: Vec<char> = a.chars().collect::<Vec<char>>();
        let sifr_generated_chars_b: Vec<char> = b.chars().collect::<Vec<char>>();
        let total: SifrInt = &SifrInt::from(sifr_generated_chars_a.len())
            + &SifrInt::from(sifr_generated_chars_b.len());
        if &total == &SifrInt::from_i64(0) {
            return Ok(1.0_f64);
        }
        let mut matches: SifrInt = SifrInt::from_i64(0);
        let blocks: Vec<(SifrInt, SifrInt, SifrInt)> = sifr_generated_matching_blocks(a, b);
        for block in blocks.iter().cloned() {
            let (_, _, block_size) = block;
            matches = &matches + &block_size;
        }
        let sifr_generated_try_res: Result<
            Result<
                f64,
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0,
            >,
            SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0,
        > = (|| {
            let numerator: f64 = (&SifrInt::from_i64(2) * &matches)
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
                })?;
            let denominator: f64 = total
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
                })?;
            Ok(Ok(numerator / denominator))
        })();
        sifr_generated_try_res
            .unwrap_or_else(|sifr_generated_try_err| match sifr_generated_try_err {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let error = sifr_generated_try_variant_error.clone();
                    Err(
                        SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                            FloatOverflowError::new(error.message.clone()),
                        ),
                    )
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let error = sifr_generated_try_variant_error.clone();
                    Err(
                        SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                            FloatPrecisionLossError::new(error.message.clone()),
                        ),
                    )
                }
            })
    }
    #[expect(
        clippy::many_single_char_names,
        reason = "generated Rust preserves this exact typed Sifr source contract"
    )]
    pub(crate) fn sifr_generated_longest_common_substring_range(
        a: &str,
        b: &str,
        a_start: SifrInt,
        a_end: SifrInt,
        b_start_argument_c9091ffae73223be: SifrInt,
        b_end_argument_847b7941bf1533ab: SifrInt,
    ) -> (SifrInt, SifrInt, SifrInt) {
        let sifr_generated_chars_a: Vec<char> = a.chars().collect::<Vec<char>>();
        let sifr_generated_chars_b: Vec<char> = b.chars().collect::<Vec<char>>();
        let mut best_i: SifrInt = SifrInt::from_i64(0);
        let mut best_j_value_1e1620cd9a699d86: SifrInt = SifrInt::from_i64(0);
        let mut best_len_value_af9487aca555f05f: SifrInt = SifrInt::from_i64(0);
        let mut i: SifrInt = a_start.clone();
        while &i < &a_end {
            let mut j: SifrInt = b_start_argument_c9091ffae73223be.clone();
            while &j < &b_end_argument_847b7941bf1533ab {
                let mut k: SifrInt = SifrInt::from_i64(0);
                while &(&i + &k) < &a_end && &(&j + &k) < &b_end_argument_847b7941bf1533ab {
                    let ai: Option<String> = {
                        let sifr_generated_string_index = &i + &k;
                        let sifr_generated_string_index_normalized = sifr_generated_string_index
                            .normalize_index_or_len(sifr_generated_chars_a.len());
                        sifr_generated_chars_a
                            .get(sifr_generated_string_index_normalized)
                            .copied()
                    }
                    .map(|character| character.to_string());
                    let bj: Option<String> = {
                        let sifr_generated_string_index = &j + &k;
                        let sifr_generated_string_index_normalized = sifr_generated_string_index
                            .normalize_index_or_len(sifr_generated_chars_b.len());
                        sifr_generated_chars_b
                            .get(sifr_generated_string_index_normalized)
                            .copied()
                    }
                    .map(|character| character.to_string());
                    let (Some(ai), Some(bj)) = (ai, bj) else {
                        k = &k + &SifrInt::from_i64(1);
                        continue;
                    };
                    if ai != bj {
                        break;
                    }
                    k = &k + &SifrInt::from_i64(1);
                }
                if &k > &best_len_value_af9487aca555f05f {
                    best_len_value_af9487aca555f05f = k;
                    best_i = i.clone();
                    best_j_value_1e1620cd9a699d86 = j.clone();
                }
                j = &j + &SifrInt::from_i64(1);
            }
            i = &i + &SifrInt::from_i64(1);
        }
        (
            best_i.clone(),
            best_j_value_1e1620cd9a699d86.clone(),
            best_len_value_af9487aca555f05f.clone(),
        )
    }
    pub(crate) fn sifr_generated_sort_blocks(
        blocks: &[(SifrInt, SifrInt, SifrInt)],
    ) -> Vec<(SifrInt, SifrInt, SifrInt)> {
        let mut sorted_blocks: Vec<(SifrInt, SifrInt, SifrInt)> = Vec::new();
        for block in blocks.iter().cloned() {
            let (bl_a, bl_b_value_c53dd39bc263efba, _) = block.clone();
            let mut found_insert_at: bool = false;
            let mut insert_at: SifrInt = SifrInt::from_i64(0);
            let mut i: SifrInt = SifrInt::from_i64(0);
            for existing in sorted_blocks.iter().cloned() {
                if !found_insert_at {
                    let (ex_a, ex_b_value_e8565f608f1d5555, _) = existing;
                    let comes_before: bool = if &bl_a < &ex_a
                        || &bl_a == &ex_a
                            && &bl_b_value_c53dd39bc263efba < &ex_b_value_e8565f608f1d5555
                    {
                        true
                    } else {
                        false
                    };
                    if comes_before {
                        insert_at = i.clone();
                        found_insert_at = true;
                    }
                }
                i = &i + &SifrInt::from_i64(1);
            }
            if found_insert_at {
                sorted_blocks.insert(
                    insert_at.clamp_slice_bound(sorted_blocks.len()),
                    block.clone(),
                );
            } else {
                sorted_blocks.push(block.clone());
            }
        }
        sorted_blocks
    }
    pub(crate) fn sifr_generated_matching_blocks(
        a: &str,
        b: &str,
    ) -> Vec<(SifrInt, SifrInt, SifrInt)> {
        let sifr_generated_chars_a: Vec<char> = a.chars().collect::<Vec<char>>();
        let sifr_generated_chars_b: Vec<char> = b.chars().collect::<Vec<char>>();
        let mut pending_a_start: Vec<SifrInt> = vec![SifrInt::from_i64(0)];
        let mut pending_a_end: Vec<SifrInt> = vec![SifrInt::from(sifr_generated_chars_a.len())];
        let mut pending_b_start_value_5010e609c75d1d22: Vec<SifrInt> = vec![SifrInt::from_i64(0)];
        let mut pending_b_end_value_9589c6af9c1daa47: Vec<SifrInt> =
            vec![SifrInt::from(sifr_generated_chars_b.len())];
        let mut unsorted_blocks: Vec<(SifrInt, SifrInt, SifrInt)> = Vec::new();
        while &SifrInt::from(pending_a_start.len()) > &SifrInt::from_i64(0) {
            let a_start_value: Option<SifrInt> =
                Some(pending_a_start.remove(pending_a_start.len() - 1_usize));
            let a_end_value: Option<SifrInt> = pending_a_end.pop();
            let b_start_value: Option<SifrInt> = pending_b_start_value_5010e609c75d1d22.pop();
            let b_end_value: Option<SifrInt> = pending_b_end_value_9589c6af9c1daa47.pop();
            if let Some(a_start_value) = a_start_value.clone()
                && let Some(a_end_value) = a_end_value.clone()
                && let Some(b_start_value) = b_start_value.clone()
                && let Some(b_end_value) = b_end_value.clone()
            {
                let (ai, bj, size) = sifr_generated_longest_common_substring_range(
                    a,
                    b,
                    a_start_value.clone(),
                    a_end_value.clone(),
                    b_start_value.clone(),
                    b_end_value.clone(),
                );
                if &size == &SifrInt::from_i64(0) {
                    continue;
                }
                unsorted_blocks.push((ai.clone(), bj.clone(), size.clone()));
                let left_a_end: SifrInt = ai.clone();
                let left_b_end_value_2d7948a8a27a7433: SifrInt = bj.clone();
                if &a_start_value < &left_a_end
                    && &b_start_value < &left_b_end_value_2d7948a8a27a7433
                {
                    pending_a_start.push(a_start_value);
                    pending_a_end.push(left_a_end);
                    pending_b_start_value_5010e609c75d1d22.push(b_start_value);
                    pending_b_end_value_9589c6af9c1daa47.push(left_b_end_value_2d7948a8a27a7433);
                }
                let right_a_start: SifrInt = &ai + &size;
                let right_b_start_value_acd2b29c16778c53: SifrInt = &bj + &size;
                if &right_a_start < &a_end_value
                    && &right_b_start_value_acd2b29c16778c53 < &b_end_value
                {
                    pending_a_start.push(right_a_start);
                    pending_a_end.push(a_end_value);
                    pending_b_start_value_5010e609c75d1d22
                        .push(right_b_start_value_acd2b29c16778c53);
                    pending_b_end_value_9589c6af9c1daa47.push(b_end_value);
                }
            }
        }
        let sorted_blocks: Vec<(SifrInt, SifrInt, SifrInt)> =
            sifr_generated_sort_blocks(&unsorted_blocks);
        let mut merged_blocks: Vec<(SifrInt, SifrInt, SifrInt)> = Vec::new();
        let mut have_previous: bool = false;
        let mut prev_a: SifrInt = SifrInt::from_i64(0);
        let mut prev_b_value_471dcfb5c284856d: SifrInt = SifrInt::from_i64(0);
        let mut prev_size: SifrInt = SifrInt::from_i64(0);
        for block in sorted_blocks.iter().cloned() {
            let (bl_a, bl_b_value_c53dd39bc263efba, bl_size) = block.clone();
            if !have_previous {
                prev_a = bl_a.clone();
                prev_b_value_471dcfb5c284856d = bl_b_value_c53dd39bc263efba.clone();
                prev_size = bl_size.clone();
                have_previous = true;
                continue;
            }
            if &(&prev_a + &prev_size) == &bl_a
                && &(&prev_b_value_471dcfb5c284856d + &prev_size) == &bl_b_value_c53dd39bc263efba
            {
                prev_size = &prev_size + &bl_size;
            } else {
                merged_blocks.push((
                    prev_a.clone(),
                    prev_b_value_471dcfb5c284856d.clone(),
                    prev_size.clone(),
                ));
                prev_a = bl_a.clone();
                prev_b_value_471dcfb5c284856d = bl_b_value_c53dd39bc263efba.clone();
                prev_size = bl_size.clone();
            }
        }
        if have_previous {
            merged_blocks.push((
                prev_a.clone(),
                prev_b_value_471dcfb5c284856d.clone(),
                prev_size.clone(),
            ));
        }
        merged_blocks.push((
            SifrInt::from(a.chars().count()),
            SifrInt::from(b.chars().count()),
            SifrInt::from_i64(0),
        ));
        merged_blocks
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
    pub(crate) fn is_valid_ipv4(addr: &str) -> bool {
        let parts: Vec<String> = addr
            .split('.')
            .map(::std::string::ToString::to_string)
            .collect::<Vec<String>>();
        if &SifrInt::from(parts.len()) != &SifrInt::from_i64(4) {
            return false;
        }
        for part in parts.iter().cloned() {
            let sifr_generated_chars_part: Vec<char> = part.chars().collect::<Vec<char>>();
            if &SifrInt::from(sifr_generated_chars_part.len()) == &SifrInt::from_i64(0) {
                return false;
            }
            if &SifrInt::from(sifr_generated_chars_part.len()) > &SifrInt::from_i64(3) {
                return false;
            }
            if &SifrInt::from(sifr_generated_chars_part.len()) > &SifrInt::from_i64(1) {
                let first_digit: Option<String> = {
                    let sifr_generated_string_index = SifrInt::from_i64(0);
                    let sifr_generated_string_index_normalized = sifr_generated_string_index
                        .normalize_index_or_len(sifr_generated_chars_part.len());
                    sifr_generated_chars_part
                        .get(sifr_generated_string_index_normalized)
                        .copied()
                }
                .map(|character| character.to_string());
                if first_digit.is_some() && first_digit == Some("0".to_string()) {
                    return false;
                }
            }
            let val: SifrInt = sifr_generated_parse_int(&part);
            if &val < &SifrInt::from_i64(0) {
                return false;
            }
            if &val > &SifrInt::from_i64(255) {
                return false;
            }
        }
        true
    }
    pub(crate) fn sifr_generated_parse_int(s: &str) -> SifrInt {
        let sifr_generated_chars_s: Vec<char> = s.chars().collect::<Vec<char>>();
        let mut result: SifrInt = SifrInt::from_i64(0);
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &SifrInt::from(sifr_generated_chars_s.len()) {
            let ch: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_s.len());
                sifr_generated_chars_s
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(ch) = ch {
                if ch == "0" {
                    result = &result * &SifrInt::from_i64(10);
                } else if ch == "1" {
                    result = &(&result * &SifrInt::from_i64(10)) + &SifrInt::from_i64(1);
                } else if ch == "2" {
                    result = &(&result * &SifrInt::from_i64(10)) + &SifrInt::from_i64(2);
                } else if ch == "3" {
                    result = &(&result * &SifrInt::from_i64(10)) + &SifrInt::from_i64(3);
                } else if ch == "4" {
                    result = &(&result * &SifrInt::from_i64(10)) + &SifrInt::from_i64(4);
                } else if ch == "5" {
                    result = &(&result * &SifrInt::from_i64(10)) + &SifrInt::from_i64(5);
                } else if ch == "6" {
                    result = &(&result * &SifrInt::from_i64(10)) + &SifrInt::from_i64(6);
                } else if ch == "7" {
                    result = &(&result * &SifrInt::from_i64(10)) + &SifrInt::from_i64(7);
                } else if ch == "8" {
                    result = &(&result * &SifrInt::from_i64(10)) + &SifrInt::from_i64(8);
                } else if ch == "9" {
                    result = &(&result * &SifrInt::from_i64(10)) + &SifrInt::from_i64(9);
                } else {
                    return -&SifrInt::from_i64(1);
                }
            }
            i = &i + &SifrInt::from_i64(1);
        }
        result.clone()
    }
    pub(crate) fn sifr_generated_ip_to_int_raw(addr: &str) -> SifrInt {
        let parts: Vec<String> = addr
            .split('.')
            .map(::std::string::ToString::to_string)
            .collect::<Vec<String>>();
        let mut result: SifrInt = SifrInt::from_i64(0);
        for part in parts.iter().cloned() {
            let val: SifrInt = sifr_generated_parse_int(&part);
            result = &(&result * &SifrInt::from_i64(256)) + &val;
        }
        result.clone()
    }
    pub(crate) fn sifr_generated_in_ipv4_range(
        value: SifrInt,
        start: SifrInt,
        end: SifrInt,
    ) -> bool {
        if &value < &start {
            return false;
        }
        if &value > &end {
            return false;
        }
        true
    }
    pub(crate) fn sifr_generated_is_private_ipv4_value(value: SifrInt) -> bool {
        let private_hit: bool = if sifr_generated_in_ipv4_range(
            value.clone(),
            SifrInt::from_i64(0),
            SifrInt::from_i64(16_777_215),
        ) || (sifr_generated_in_ipv4_range(
            value.clone(),
            SifrInt::from_i64(167_772_160),
            SifrInt::from_i64(184_549_375),
        ) || (sifr_generated_in_ipv4_range(
            value.clone(),
            SifrInt::from_i64(2_130_706_432),
            SifrInt::from_i64(2_147_483_647),
        ) || (sifr_generated_in_ipv4_range(
            value.clone(),
            SifrInt::from_i64(2_851_995_648),
            SifrInt::from_i64(2_852_061_183),
        ) || (sifr_generated_in_ipv4_range(
            value.clone(),
            SifrInt::from_i64(2_886_729_728),
            SifrInt::from_i64(2_887_778_303),
        ) || (sifr_generated_in_ipv4_range(
            value.clone(),
            SifrInt::from_i64(3_221_225_472),
            SifrInt::from_i64(3_221_225_727),
        ) || (sifr_generated_in_ipv4_range(
            value.clone(),
            SifrInt::from_i64(3_221_225_642),
            SifrInt::from_i64(3_221_225_643),
        )
            || (sifr_generated_in_ipv4_range(
                value.clone(),
                SifrInt::from_i64(3_221_225_984),
                SifrInt::from_i64(3_221_226_239),
            ) || (sifr_generated_in_ipv4_range(
                value.clone(),
                SifrInt::from_i64(3_232_235_520),
                SifrInt::from_i64(3_232_301_055),
            ) || (sifr_generated_in_ipv4_range(
                value.clone(),
                SifrInt::from_i64(3_323_068_416),
                SifrInt::from_i64(3_323_199_487),
            ) || (sifr_generated_in_ipv4_range(
                value.clone(),
                SifrInt::from_i64(3_325_256_704),
                SifrInt::from_i64(3_325_256_959),
            ) || (sifr_generated_in_ipv4_range(
                value.clone(),
                SifrInt::from_i64(3_405_803_776),
                SifrInt::from_i64(3_405_804_031),
            ) || (sifr_generated_in_ipv4_range(
                value.clone(),
                SifrInt::from_i64(4_026_531_840),
                SifrInt::from_i64(4_294_967_295),
            ) || &value
                == &SifrInt::from_i64(4_294_967_295)))))))))))))
        {
            true
        } else {
            false
        };
        if private_hit {
            if &value == &SifrInt::from_i64(3_221_225_481) {
                return false;
            }
            if &value == &SifrInt::from_i64(3_221_225_482) {
                return false;
            }
        }
        private_hit
    }
    pub(crate) fn is_private(addr: &str) -> bool {
        if !is_valid_ipv4(addr) {
            return false;
        }
        let val: SifrInt = sifr_generated_ip_to_int_raw(addr);
        sifr_generated_is_private_ipv4_value(val.clone())
    }
    pub(crate) fn is_loopback(addr: &str) -> bool {
        if !is_valid_ipv4(addr) {
            return false;
        }
        let parts: Vec<String> = addr
            .split('.')
            .map(::std::string::ToString::to_string)
            .collect::<Vec<String>>();
        if &SifrInt::from(parts.len()) == &SifrInt::from_i64(4) {
            let first: Option<String> = {
                let sifr_generated_checked_read_collection = &parts;
                let sifr_generated_checked_read_index = SifrInt::from_i64(0);
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            if let Some(first) = first
                && first == "127"
            {
                return true;
            }
        }
        false
    }
    pub(crate) fn getcwd() -> Result<String, IOError> {
        ::sifr_stdlib::fs::getcwd().map_err(sifr_generated_io_err)
    }
    #[expect(
        clippy::approx_constant,
        reason = "generated Rust preserves this exact typed Sifr source contract"
    )]
    pub(crate) const TAU: f64 = 6.283_185_307_179_586_f64;
    pub(crate) const NAN: f64 = f64::NAN;
    pub(crate) const fn isnan(x: f64) -> bool {
        ::sifr_stdlib::math::isnan(x)
    }
    pub(crate) fn join_path(base: &str, child: &str) -> String {
        let sifr_generated_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
        if &SifrInt::from(sifr_generated_chars_base.len()) == &SifrInt::from_i64(0) {
            return {
                let mut sifr_generated_concat: String = String::with_capacity(child.len());
                sifr_generated_concat.push_str(child);
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
        }
        let last: Option<String> = {
            let sifr_generated_string_index =
                &SifrInt::from(base.chars().count()) - &SifrInt::from_i64(1);
            let sifr_generated_string_index_normalized =
                sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_base.len());
            sifr_generated_chars_base
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string());
        if let Some(last) = last
            && last.as_str() == "/".to_string().as_str()
        {
            return {
                let mut sifr_generated_concat: String =
                    String::with_capacity(base.len() + child.len());
                sifr_generated_concat.push_str(base.as_ref());
                sifr_generated_concat.push_str(child.as_ref());
                sifr_generated_concat
            };
        }
        {
            let mut sifr_generated_concat: String =
                String::with_capacity(base.len() + 1usize + child.len());
            sifr_generated_concat.push_str(base);
            sifr_generated_concat.push('/');
            sifr_generated_concat.push_str(child);
            sifr_generated_concat
        }
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
    pub(crate) fn platform_system() -> String {
        ::sifr_stdlib::platform::platform_system()
    }
    pub(crate) fn platform_arch() -> String {
        ::sifr_stdlib::platform::platform_arch()
    }
    pub(crate) fn system() -> String {
        platform_system()
    }
    pub(crate) fn machine() -> String {
        platform_arch()
    }
    pub(crate) fn re_findall(pattern: &str, text: &str) -> Result<Vec<String>, RegexError> {
        ::sifr_stdlib::regex::re_findall(pattern, text).map_err(|sifr_generated_bridge_error| {
            RegexError {
                message: sifr_generated_bridge_error.to_string(),
                detail: sifr_generated_bridge_error.to_string(),
            }
        })
    }
    pub(crate) fn findall(pattern: &str, text: &str) -> Result<Vec<String>, RegexError> {
        re_findall(pattern, text)
    }
    pub(crate) fn default_timer() -> f64 {
        perf_counter()
    }
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
                    let _e_5f65 = sifr_generated_try_variant_error.clone();
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
                    let _e_5f65 = sifr_generated_try_variant_error.clone();
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
    pub(crate) fn uuid4() -> String {
        ::sifr_stdlib::uuid::uuid4()
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
mod sifr_generated_project_nominals {
    use crate::sifr_generated_generated_support::*;
    use ::sifr_runtime::SifrInt;
    #[derive(Debug, Clone)]
    pub struct SifrGeneratedStdlibSifrX2edatetimeX2etimezone {
        pub offset: SifrInt,
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
    impl SifrGeneratedStdlibSifrX2etomllibX2eTomlValue {
        #[must_use]
        pub fn keys(&self) -> Vec<String> {
            let mut result: Vec<String> = Vec::new();
            if !self.is_table() {
                return result;
            }
            for (item_key, _item_value) in self.table_items.iter().cloned() {
                result.push(item_key.to_owned());
            }
            result
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
    pub struct JSONDecodeError {
        pub message: String,
        pub line: SifrInt,
        pub column: SifrInt,
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct TimeoutError {
        pub message: String,
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
    impl ::std::fmt::Display for ScopeFailure {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ScopeFailure {}
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
    impl From<JSONDecodeError> for Error {
        fn from(err: JSONDecodeError) -> Self {
            Self::new(err.message)
        }
    }
    impl From<JsonIntegerRangeError> for Error {
        fn from(err: JsonIntegerRangeError) -> Self {
            Self::new(err.message)
        }
    }
    impl From<JsonLimitError> for Error {
        fn from(err: JsonLimitError) -> Self {
            Self::new(err.message)
        }
    }
    impl From<TOMLDecodeError> for Error {
        fn from(err: TOMLDecodeError) -> Self {
            Self::new(err.message)
        }
    }
    impl From<RegexError> for Error {
        fn from(err: RegexError) -> Self {
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
    impl From<TimeoutError> for Error {
        fn from(err: TimeoutError) -> Self {
            Self::new(err.message)
        }
    }
    impl From<ScopeFailure> for Error {
        fn from(err: ScopeFailure) -> Self {
            Self::new(err.message)
        }
    }
    impl
        From<
            crate::sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2egraphlibX2eCycleError,
        > for Error
    {
        fn from(
            err: crate::sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2egraphlibX2eCycleError,
        ) -> Self {
            Self::new(err.message)
        }
    }
}
pub use sifr_generated_project_nominals::Error;
pub use sifr_generated_project_nominals::FloatOverflowError;
pub use sifr_generated_project_nominals::FloatPrecisionLossError;
pub use sifr_generated_project_nominals::IOError;
pub use sifr_generated_project_nominals::ParseError;
pub use sifr_generated_project_nominals::RegexError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2edatetimeX2edatetime;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2edatetimeX2etimezone;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2egraphlibX2eCycleError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2etomllibX2eTomlValue;
pub use sifr_generated_project_nominals::TOMLDecodeError;
pub use sifr_generated_project_nominals::ValueError;
mod sifr_generated_project_unions {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
            crate::sifr_generated_project_nominals::FloatOverflowError,
        ),
        SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
            crate::sifr_generated_project_nominals::FloatPrecisionLossError,
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
    #[derive(Debug, Clone)]
    pub enum SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0
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
    }
    impl From<crate::sifr_generated_project_nominals::Error>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0 {
        fn from(value: crate::sifr_generated_project_nominals::Error) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass5X3aError1X3a0(
                value,
            )
        }
    }
    impl From<crate::sifr_generated_project_nominals::FloatOverflowError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0 {
        fn from(
            value: crate::sifr_generated_project_nominals::FloatOverflowError,
        ) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                value,
            )
        }
    }
    impl From<crate::sifr_generated_project_nominals::FloatPrecisionLossError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0 {
        fn from(
            value: crate::sifr_generated_project_nominals::FloatPrecisionLossError,
        ) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass5X3aError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
}
use crate::sifr_generated_generated_support::*;
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_unions::SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0;
pub use sifr_generated_project_unions::SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0;
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
#[expect(
    clippy::assertions_on_constants,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
fn main() {
    assert!(TAU > 6.0_f64);
    assert!(isnan(NAN));
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let cwd: String = getcwd()?;
        let _chars_cwd: Vec<char> = cwd.chars().collect::<Vec<char>>();
        assert!(SifrInt::from(cwd.chars().count()) > SifrInt::from_i64(0));
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let err = sifr_generated_try_err.clone();
        println!("getcwd error: {}", err.message.clone());
        assert_eq!(
            format!("getcwd error: {}", err.message.clone()),
            "stdlib_parity demo: all checks passed!"
        );
    }
    let sifr_generated_try_res: Result<(), RegexError> = (|| {
        let matches: Vec<String> = findall(&"[0-9]+".to_string(), &"abc123def456".to_string())?;
        assert_eq!(SifrInt::from(matches.len()), SifrInt::from_i64(2));
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let err = sifr_generated_try_err.clone();
        println!("regex error: {}", err.message.clone());
        assert_eq!(
            format!("regex error: {}", err.message.clone()),
            "Total stdlib modules: 37"
        );
    }
    let from_nodes: Vec<SifrInt> = vec![
        SifrInt::from_i64(0),
        SifrInt::from_i64(0),
        SifrInt::from_i64(1),
    ];
    let to_nodes: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(2),
    ];
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2egraphlibX2eCycleError> =
        (|| {
            let order: Vec<SifrInt> =
                topological_sort(SifrInt::from_i64(3), &from_nodes, &to_nodes)?;
            assert_eq!(SifrInt::from(order.len()), SifrInt::from_i64(3));
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("error: {}", e.message.clone());
    }
    let id: String = uuid4();
    let _chars_id: Vec<char> = id.chars().collect::<Vec<char>>();
    assert!(&SifrInt::from(id.chars().count()) > &SifrInt::from_i64(0));
    let sys: String = system();
    let _chars_sys_value_83aa7b308a4fdc45: Vec<char> = sys.chars().collect::<Vec<char>>();
    assert!(&SifrInt::from(sys.chars().count()) > &SifrInt::from_i64(0));
    let arch: String = machine();
    let _chars_arch_value_27b041ed1c99580c: Vec<char> = arch.chars().collect::<Vec<char>>();
    assert!(&SifrInt::from(arch.chars().count()) > &SifrInt::from_i64(0));
    let p: String = join_path(&"/usr".to_string(), &"local".to_string());
    assert_eq!(p, "/usr/local");
    assert_eq!(basename(&"/home/user/file.txt".to_string()), "file.txt");
    assert_eq!(extension(&"file.tar.gz".to_string()), ".gz");
    let words: Vec<String> = vec![
        "apple".to_string(),
        "ape".to_string(),
        "application".to_string(),
    ];
    let sifr_generated_try_res: Result<
        (),
        SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0,
    > = (|| {
        let close: Vec<String> = get_close_matches(
                &"app".to_string(),
                &words,
                SifrInt::from_i64(2),
                0.3_f64,
            )
            .map_err(|sifr_generated_e| match sifr_generated_e {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                    sifr_generated_union_value,
                ) => {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                        sifr_generated_union_value,
                    )
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                    sifr_generated_union_value,
                ) => {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                        sifr_generated_union_value,
                    )
                }
            })?;
        assert!(SifrInt::from(close.len()) > SifrInt::from_i64(0));
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        match sifr_generated_try_err {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass5X3aError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let _e_5f65 = sifr_generated_try_variant_error.clone();
                assert!(false);
            }
            SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let _e_5f65 = Error::new(
                    sifr_generated_try_variant_error.clone().message,
                );
                assert!(false);
            }
            SifrGeneratedUnion8X3asequence5X3aunion1X3a331X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let _e_5f65 = Error::new(
                    sifr_generated_try_variant_error.clone().message,
                );
                assert!(false);
            }
        }
    }
    assert!(is_valid_ipv4(&"192.168.1.1".to_string()));
    assert!(!is_valid_ipv4(&"999.1.1.1".to_string()));
    assert!(is_private(&"10.0.0.1".to_string()));
    assert!(is_loopback(&"127.0.0.1".to_string()));
    let start: f64 = default_timer();
    let end: f64 = default_timer();
    assert!(end >= start);
    let sifr_generated_try_res: Result<(), TOMLDecodeError> = (|| {
        let toml_result: SifrGeneratedStdlibSifrX2etomllibX2eTomlValue =
            loads(&"key = \"value\"".to_string())?;
        assert!(SifrInt::from(toml_result.keys().len()) > SifrInt::from_i64(0));
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let err = sifr_generated_try_err.clone();
        println!("toml error: {}", err.message.clone());
    }
    let dt_now: SifrGeneratedStdlibSifrX2edatetimeX2edatetime = now(&None);
    assert!(&SifrInt::from(dt_now.to_string().chars().count()) > &SifrInt::from_i64(0));
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let dt_epoch: SifrGeneratedStdlibSifrX2edatetimeX2edatetime =
            from_timestamp(0.0_f64, &None)?;
        assert!(SifrInt::from(dt_epoch.isoformat().chars().count()) > SifrInt::from_i64(0));
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("error: {}", e.message.clone());
    }
    println!("stdlib_parity demo: all checks passed!");
    println!("Total stdlib modules: 37");
}
