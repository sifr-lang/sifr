// src/main.rs
mod sifr_generated_generated_support {
    use crate::{
        FloatOverflowError, FloatPrecisionLossError, ParseError,
        SifrGeneratedStdlibSifrX2edatetimeX2edatetime,
        SifrGeneratedStdlibSifrX2edatetimeX2etimezone,
        SifrGeneratedStdlibSifrX2egraphlibX2eCycleError, SifrGeneratedStdlibSifrX2erandomX2eRandom,
        SifrGeneratedStdlibSifrX2erandomX2eRandomState, SifrGeneratedStdlibSifrX2euuidX2eUUID,
        ValueError,
    };
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) fn datetime_from_timestamp(ts: f64) -> Result<String, ValueError> {
        ::sifr_stdlib::time::datetime_from_timestamp(ts).map_err(|sifr_generated_bridge_error| {
            ValueError {
                message: sifr_generated_bridge_error.to_string(),
            }
        })
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
    pub(crate) fn from_timestamp(
        ts: f64,
        tz: &Option<SifrGeneratedStdlibSifrX2edatetimeX2etimezone>,
    ) -> Result<SifrGeneratedStdlibSifrX2edatetimeX2edatetime, ValueError> {
        sifr_generated_from_timestamp_with_tz(ts, tz)
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
    pub(crate) fn ip_to_int(addr: &str) -> Result<SifrInt, ValueError> {
        if !is_valid_ipv4(addr) {
            return Err(ValueError::new("invalid IPv4 address".to_string()));
        }
        Ok(sifr_generated_ip_to_int_raw(addr))
    }
    pub(crate) trait SifrGeneratedAdd: Sized {}
    impl SifrGeneratedAdd for ::sifr_runtime::SifrInt {}
    impl SifrGeneratedAdd for f64 {}
    impl SifrGeneratedAdd for String {}
    pub(crate) fn batched<T: Clone + 'static>(
        data: &[T],
        n: SifrInt,
    ) -> Result<Vec<Vec<T>>, ValueError> {
        if &n <= &SifrInt::from_i64(0) {
            return Err(ValueError::new("batched: n must be > 0".to_string()));
        }
        let mut result: Vec<Vec<T>> = Vec::new();
        let mut current_batch: Vec<T> = Vec::new();
        for value in data.iter().cloned() {
            current_batch.push(value);
            if &SifrInt::from(current_batch.len()) == &n {
                result.push(current_batch.to_vec());
                current_batch = Vec::new();
            }
        }
        if &SifrInt::from(current_batch.len()) > &SifrInt::from_i64(0) {
            result.push(current_batch);
        }
        Ok(result)
    }
    pub(crate) fn random_int(min: SifrInt, max: SifrInt) -> SifrInt {
        ::sifr_stdlib::random::random_int(
            ::sifr_runtime::interop::SifrIntBridge::from(min),
            ::sifr_runtime::interop::SifrIntBridge::from(max),
        )
        .into_sifr_int()
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
    pub(crate) fn randint(minimum: SifrInt, maximum: SifrInt) -> Result<SifrInt, ValueError> {
        let mut generator: SifrGeneratedStdlibSifrX2erandomX2eRandom =
            sifr_generated_module_random();
        let value: Result<SifrInt, ValueError> = generator.randint(&minimum, &maximum);
        sifr_generated_sync_module_random(&mut generator);
        value
    }
    pub(crate) fn randbelow(n: SifrInt) -> Result<SifrInt, ValueError> {
        if &n <= &SifrInt::from_i64(0) {
            return Err(ValueError::new("randbelow: n must be > 0".to_string()));
        }
        Ok(random_int(SifrInt::from_i64(0), &n - &SifrInt::from_i64(1)))
    }
    pub(crate) fn sifr_generated_replace_whitespace_chars(
        text: &str,
        replace_tabs: bool,
    ) -> String {
        let normalized: String = text
            .replace('\n', " ")
            .replace('\r', " ")
            .replace('\u{b}', " ")
            .replace('\u{c}', " ");
        if replace_tabs {
            return normalized.replace('\t', " ");
        }
        normalized
    }
    pub(crate) fn sifr_generated_expand_tabs_impl(text: &str, tabsize: SifrInt) -> String {
        let sifr_generated_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
        let mut effective_tabsize: SifrInt = tabsize.clone();
        if &effective_tabsize <= &SifrInt::from_i64(0) {
            effective_tabsize = SifrInt::from_i64(1);
        }
        if &effective_tabsize == &SifrInt::from_i64(0) {
            return text.to_owned();
        }
        let mut result: String = String::new();
        let mut column: SifrInt = SifrInt::from_i64(0);
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
            if let Some(ch_opt) = ch_opt {
                let ch: String = ch_opt;
                if ch == "\t" {
                    let mut spaces: SifrInt =
                        &effective_tabsize - &column.floor_mod_known_nonzero(&effective_tabsize);
                    if &spaces <= &SifrInt::from_i64(0) {
                        spaces = effective_tabsize.clone();
                    }
                    let mut j: SifrInt = SifrInt::from_i64(0);
                    while &j < &spaces {
                        result.push(' ');
                        j = &j + &SifrInt::from_i64(1);
                    }
                    column = &column + &spaces;
                } else {
                    let sifr_generated_shared_branch_condition = ch == "\n" || ch == "\r";
                    result.push_str(ch.as_str());
                    if sifr_generated_shared_branch_condition {
                        column = SifrInt::from_i64(0);
                    } else {
                        column = &column + &SifrInt::from_i64(1);
                    }
                }
            }
            i = &i + &SifrInt::from_i64(1);
        }
        result
    }
    pub(crate) fn sifr_generated_prepare_text(
        text: &str,
        expand_tabs: bool,
        tabsize: SifrInt,
        replace_whitespace: bool,
    ) -> String {
        let mut prepared: String = {
            let mut sifr_generated_concat: String = String::with_capacity(text.len());
            sifr_generated_concat.push_str(text);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        };
        if expand_tabs {
            prepared = sifr_generated_expand_tabs_impl(&prepared, tabsize.clone());
        }
        if replace_whitespace {
            prepared = sifr_generated_replace_whitespace_chars(&prepared, true);
        }
        prepared
    }
    pub(crate) fn sifr_generated_normalize_whitespace(text: &str) -> String {
        sifr_generated_prepare_text(text, true, SifrInt::from_i64(8), true)
    }
    pub(crate) fn sifr_generated_split_word_units(
        word: &str,
        break_on_hyphens: bool,
    ) -> Vec<String> {
        if !break_on_hyphens {
            return vec![{
                let mut sifr_generated_concat: String = String::with_capacity(word.len());
                sifr_generated_concat.push_str(word);
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            }];
        }
        let parts: Vec<String> = word
            .split('-')
            .map(::std::string::ToString::to_string)
            .collect::<Vec<String>>();
        if &SifrInt::from(parts.len()) <= &SifrInt::from_i64(1) {
            return vec![{
                let mut sifr_generated_concat: String = String::with_capacity(word.len());
                sifr_generated_concat.push_str(word);
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            }];
        }
        let mut units: Vec<String> = Vec::new();
        let mut index: SifrInt = SifrInt::from_i64(0);
        for part in parts.iter().cloned() {
            let sifr_generated_chars_part: Vec<char> = part.chars().collect::<Vec<char>>();
            let is_last: bool = &index == &(&SifrInt::from(parts.len()) - &SifrInt::from_i64(1));
            if is_last {
                if &SifrInt::from(sifr_generated_chars_part.len()) > &SifrInt::from_i64(0) {
                    units.push(part);
                }
            } else if &SifrInt::from(sifr_generated_chars_part.len()) == &SifrInt::from_i64(0) {
                units.push("-".to_string());
            } else {
                units.push(format!("{part}-"));
            }
            index = &index + &SifrInt::from_i64(1);
        }
        if &SifrInt::from(units.len()) == &SifrInt::from_i64(0) {
            units.push(word.to_string());
        }
        units
    }
    pub(crate) fn sifr_generated_trim_line(line: &str) -> String {
        let sifr_generated_chars_line: Vec<char> = line.chars().collect::<Vec<char>>();
        let mut start: SifrInt = SifrInt::from_i64(0);
        while &start < &SifrInt::from(sifr_generated_chars_line.len()) && {
            let sifr_generated_string_index = start.clone();
            let sifr_generated_string_index_normalized =
                sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_line.len());
            sifr_generated_chars_line
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string())
        .is_some_and(|_checked_value_2| {
            {
                let sifr_generated_string_index = start.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_line.len());
                sifr_generated_chars_line
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(Some)
                == Some(Some(' '))
        }) {
            start = &start + &SifrInt::from_i64(1);
        }
        let mut end: SifrInt = SifrInt::from(sifr_generated_chars_line.len());
        while &end > &start && {
            let sifr_generated_string_index = &end - &SifrInt::from_i64(1);
            let sifr_generated_string_index_normalized =
                sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_line.len());
            sifr_generated_chars_line
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(Some)
            == Some(Some(' '))
        {
            end = &end - &SifrInt::from_i64(1);
        }
        {
            let sifr_generated_slice_src = &sifr_generated_chars_line;
            let sifr_generated_slice_len = sifr_generated_slice_src.len();
            let sifr_generated_slice_start = start.clamp_slice_bound(sifr_generated_slice_len);
            let sifr_generated_slice_stop = end.clamp_slice_bound(sifr_generated_slice_len);
            String::from_iter(
                sifr_generated_slice_src
                    .iter()
                    .skip(sifr_generated_slice_start)
                    .take(sifr_generated_slice_stop.saturating_sub(sifr_generated_slice_start))
                    .copied(),
            )
        }
    }
    pub(crate) fn sifr_generated_finalize_line(line: &str, drop_whitespace: bool) -> String {
        if drop_whitespace {
            return sifr_generated_trim_line(line);
        }
        {
            let mut sifr_generated_concat: String = String::with_capacity(line.len());
            sifr_generated_concat.push_str(line);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        }
    }
    pub(crate) fn sifr_generated_wrap_impl(text: &str, width: SifrInt) -> Vec<String> {
        let normalized: String = sifr_generated_normalize_whitespace(text);
        sifr_generated_wrap_with_indents(
            &normalized,
            width.clone(),
            &String::new(),
            &String::new(),
            true,
            true,
        )
    }
    pub(crate) fn sifr_generated_effective_content_width(
        total_width: SifrInt,
        indent: &str,
    ) -> SifrInt {
        let sifr_generated_chars_indent: Vec<char> = indent.chars().collect::<Vec<char>>();
        let available: SifrInt = &total_width - &SifrInt::from(sifr_generated_chars_indent.len());
        if &available <= &SifrInt::from_i64(0) {
            return SifrInt::from_i64(1);
        }
        available.clone()
    }
    pub(crate) fn sifr_generated_push_current_line(
        result: &mut Vec<String>,
        line: &str,
        indent: &str,
        drop_whitespace: bool,
    ) {
        let candidate: String =
            sifr_generated_finalize_line(&format!("{indent}{line}"), drop_whitespace);
        let sifr_generated_chars_candidate: Vec<char> = candidate.chars().collect::<Vec<char>>();
        if drop_whitespace {
            if &SifrInt::from(sifr_generated_chars_candidate.len()) > &SifrInt::from_i64(0) {
                result.push(candidate);
            }
        } else {
            result.push(candidate);
        }
    }
    pub(crate) fn sifr_generated_wrap_with_indents(
        text: &str,
        total_width: SifrInt,
        initial_indent: &str,
        subsequent_indent: &str,
        break_on_hyphens: bool,
        drop_whitespace: bool,
    ) -> Vec<String> {
        let words: Vec<String> = text
            .split(' ')
            .map(::std::string::ToString::to_string)
            .collect::<Vec<String>>();
        let mut result: Vec<String> = Vec::new();
        let mut current: String = String::new();
        let mut sifr_generated_chars_current: Vec<char> = current.chars().collect::<Vec<char>>();
        let mut first_line: bool = true;
        let mut current_limit: SifrInt =
            sifr_generated_effective_content_width(total_width.clone(), initial_indent);
        for raw_word in words.iter().cloned() {
            let units: Vec<String> = sifr_generated_split_word_units(&raw_word, break_on_hyphens);
            for word in units.iter().cloned() {
                let sifr_generated_chars_word: Vec<char> = word.chars().collect::<Vec<char>>();
                if &SifrInt::from(sifr_generated_chars_word.len()) == &SifrInt::from_i64(0) {
                    if drop_whitespace {
                        continue;
                    }
                    if &SifrInt::from(sifr_generated_chars_current.len()) > &SifrInt::from_i64(0)
                        && &(&SifrInt::from(sifr_generated_chars_current.len())
                            + &SifrInt::from_i64(1))
                            <= &current_limit
                    {
                        current.push(' ');
                        sifr_generated_chars_current.push(' ');
                    }
                    continue;
                }
                if &SifrInt::from(sifr_generated_chars_current.len()) == &SifrInt::from_i64(0) {
                    current = word;
                    sifr_generated_chars_current = current.chars().collect::<Vec<char>>();
                } else if &(&(&SifrInt::from(sifr_generated_chars_current.len())
                    + &SifrInt::from_i64(1))
                    + &SifrInt::from(sifr_generated_chars_word.len()))
                    <= &current_limit
                {
                    current.push(' ');
                    sifr_generated_chars_current.push(' ');
                    let sifr_generated_string_concat_current_1 = word;
                    current.push_str(sifr_generated_string_concat_current_1.as_str());
                    sifr_generated_chars_current
                        .extend(sifr_generated_string_concat_current_1.as_str().chars());
                } else {
                    if first_line {
                        sifr_generated_push_current_line(
                            &mut result,
                            &current,
                            initial_indent,
                            drop_whitespace,
                        );
                        first_line = false;
                        current_limit = sifr_generated_effective_content_width(
                            total_width.clone(),
                            subsequent_indent,
                        );
                    } else {
                        sifr_generated_push_current_line(
                            &mut result,
                            &current,
                            subsequent_indent,
                            drop_whitespace,
                        );
                    }
                    current = word;
                    sifr_generated_chars_current = current.chars().collect::<Vec<char>>();
                }
            }
        }
        if &SifrInt::from(sifr_generated_chars_current.len()) > &SifrInt::from_i64(0) {
            if first_line {
                sifr_generated_push_current_line(
                    &mut result,
                    &current,
                    initial_indent,
                    drop_whitespace,
                );
            } else {
                sifr_generated_push_current_line(
                    &mut result,
                    &current,
                    subsequent_indent,
                    drop_whitespace,
                );
            }
        }
        result
    }
    pub(crate) fn wrap(text: &str, width: SifrInt) -> Result<Vec<String>, ValueError> {
        if &width <= &SifrInt::from_i64(0) {
            return Err(ValueError::new("wrap: width must be > 0".to_string()));
        }
        Ok(sifr_generated_wrap_impl(text, width.clone()))
    }
    pub(crate) fn sifr_generated_to_lower_hex_char(ch: &str) -> String {
        if ch == "A" {
            return "a".to_string();
        }
        if ch == "B" {
            return "b".to_string();
        }
        if ch == "C" {
            return "c".to_string();
        }
        if ch == "D" {
            return "d".to_string();
        }
        if ch == "E" {
            return "e".to_string();
        }
        if ch == "F" {
            return "f".to_string();
        }
        {
            let mut sifr_generated_concat: String = String::with_capacity(ch.len());
            sifr_generated_concat.push_str(ch);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        }
    }
    pub(crate) fn sifr_generated_is_hex_char(ch: &str) -> bool {
        if ch == "0" {
            return true;
        }
        if ch == "1" {
            return true;
        }
        if ch == "2" {
            return true;
        }
        if ch == "3" {
            return true;
        }
        if ch == "4" {
            return true;
        }
        if ch == "5" {
            return true;
        }
        if ch == "6" {
            return true;
        }
        if ch == "7" {
            return true;
        }
        if ch == "8" {
            return true;
        }
        if ch == "9" {
            return true;
        }
        if ch == "a" {
            return true;
        }
        if ch == "b" {
            return true;
        }
        if ch == "c" {
            return true;
        }
        if ch == "d" {
            return true;
        }
        if ch == "e" {
            return true;
        }
        if ch == "f" {
            return true;
        }
        if ch == "A" {
            return true;
        }
        if ch == "B" {
            return true;
        }
        if ch == "C" {
            return true;
        }
        if ch == "D" {
            return true;
        }
        if ch == "E" {
            return true;
        }
        if ch == "F" {
            return true;
        }
        false
    }
    pub(crate) fn sifr_generated_starts_with(value: &str, prefix: &str) -> bool {
        let sifr_generated_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
        let sifr_generated_chars_prefix: Vec<char> = prefix.chars().collect::<Vec<char>>();
        if &SifrInt::from(sifr_generated_chars_value.len())
            < &SifrInt::from(sifr_generated_chars_prefix.len())
        {
            return false;
        }
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &SifrInt::from(sifr_generated_chars_prefix.len()) {
            let left: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_value.len());
                sifr_generated_chars_value
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            let right: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_prefix.len());
                sifr_generated_chars_prefix
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if left != right {
                return false;
            }
            i = &i + &SifrInt::from_i64(1);
        }
        true
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub(crate) fn sifr_generated_canonical_uuid_text(
        input_text: &str,
    ) -> Result<String, ValueError> {
        let mut normalized_input: String = {
            let mut sifr_generated_concat: String = String::with_capacity(input_text.len());
            sifr_generated_concat.push_str(input_text);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        };
        let mut sifr_generated_chars_normalized_input: Vec<char> =
            if sifr_generated_starts_with(&normalized_input, &"urn:uuid:".to_string()) {
                {
                    normalized_input = sifr_generated_substring(
                        &normalized_input,
                        SifrInt::from_i64(9),
                        SifrInt::from(normalized_input.chars().count()),
                    );
                    normalized_input.chars().collect::<Vec<char>>()
                }
            } else {
                normalized_input.chars().collect::<Vec<char>>()
            };
        if &SifrInt::from(sifr_generated_chars_normalized_input.len()) >= &SifrInt::from_i64(2) {
            let first: Option<String> = {
                let sifr_generated_string_index = SifrInt::from_i64(0);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_normalized_input.len());
                sifr_generated_chars_normalized_input
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            let last: Option<String> = {
                let sifr_generated_string_index =
                    &SifrInt::from(normalized_input.chars().count()) - &SifrInt::from_i64(1);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_normalized_input.len());
                sifr_generated_chars_normalized_input
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if first == Some("{".to_string()) && last == Some("}".to_string()) {
                normalized_input = sifr_generated_substring(
                    &normalized_input,
                    SifrInt::from_i64(1),
                    &SifrInt::from(normalized_input.chars().count()) - &SifrInt::from_i64(1),
                );
                sifr_generated_chars_normalized_input =
                    normalized_input.chars().collect::<Vec<char>>();
            }
        }
        let input_len: SifrInt = SifrInt::from(sifr_generated_chars_normalized_input.len());
        let mut hex_only: String = String::new();
        let mut sifr_generated_chars_hex_only: Vec<char> = hex_only.chars().collect::<Vec<char>>();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &input_len {
            let ch_opt: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_normalized_input.len());
                sifr_generated_chars_normalized_input
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(ch_opt) = ch_opt {
                let ch: String = ch_opt;
                if ch == "-" {
                } else {
                    if !sifr_generated_is_hex_char(&ch) {
                        return Err(ValueError::new("invalid UUID hex string".to_string()));
                    }
                    let sifr_generated_string_concat_hex_only_0 =
                        sifr_generated_to_lower_hex_char(&ch);
                    hex_only.push_str(sifr_generated_string_concat_hex_only_0.as_str());
                    sifr_generated_chars_hex_only
                        .extend(sifr_generated_string_concat_hex_only_0.as_str().chars());
                }
            }
            i = &i + &SifrInt::from_i64(1);
        }
        if &SifrInt::from(sifr_generated_chars_hex_only.len()) != &SifrInt::from_i64(32) {
            return Err(ValueError::new(
                "UUID hex string must be 32 hex characters".to_string(),
            ));
        }
        if &input_len == &SifrInt::from_i64(36) {
            let h1: Option<String> = {
                let sifr_generated_string_index = SifrInt::from_i64(8);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_normalized_input.len());
                sifr_generated_chars_normalized_input
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            let h2: Option<String> = {
                let sifr_generated_string_index = SifrInt::from_i64(13);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_normalized_input.len());
                sifr_generated_chars_normalized_input
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            let h3: Option<String> = {
                let sifr_generated_string_index = SifrInt::from_i64(18);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_normalized_input.len());
                sifr_generated_chars_normalized_input
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            let h4: Option<String> = {
                let sifr_generated_string_index = SifrInt::from_i64(23);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_normalized_input.len());
                sifr_generated_chars_normalized_input
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if h1 != Some("-".to_string())
                || h2 != Some("-".to_string())
                || h3 != Some("-".to_string())
                || h4 != Some("-".to_string())
            {
                return Err(ValueError::new("invalid UUID hex string".to_string()));
            }
        } else if &input_len != &SifrInt::from_i64(32) {
            return Err(ValueError::new("invalid UUID hex string".to_string()));
        }
        let mut canonical: String = String::new();
        let mut j: SifrInt = SifrInt::from_i64(0);
        while &j < &SifrInt::from(sifr_generated_chars_hex_only.len()) {
            if &j == &SifrInt::from_i64(8)
                || &j == &SifrInt::from_i64(12)
                || &j == &SifrInt::from_i64(16)
                || &j == &SifrInt::from_i64(20)
            {
                canonical.push('-');
            }
            let part: Option<String> = {
                let sifr_generated_string_index = j.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_hex_only.len());
                sifr_generated_chars_hex_only
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(part) = part {
                canonical.push_str(part.as_str());
            }
            j = &j + &SifrInt::from_i64(1);
        }
        Ok(canonical)
    }
    pub(crate) fn uuid_from_hex(
        hex_str: &str,
    ) -> Result<SifrGeneratedStdlibSifrX2euuidX2eUUID, ValueError> {
        let sifr_generated_try_res: Result<
            Result<SifrGeneratedStdlibSifrX2euuidX2eUUID, ValueError>,
            ValueError,
        > = (|| {
            let canonical: String = sifr_generated_canonical_uuid_text(hex_str)?;
            Ok(Ok(SifrGeneratedStdlibSifrX2euuidX2eUUID::new(canonical)))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err.clone();
            Err(ValueError::new(e.message.clone()))
        })
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
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn randrange(
            &mut self,
            start: &SifrInt,
            stop: &Option<SifrInt>,
            step_argument_af0b4e191da20cef: &SifrInt,
        ) -> Result<SifrInt, ValueError> {
            if step_argument_af0b4e191da20cef == &SifrInt::from_i64(0) {
                return Err(ValueError::new(
                    "randrange: step must not be zero".to_string(),
                ));
            }
            let mut actual_start: SifrInt = start.clone();
            let mut actual_stop_value_351bdef5a4961be0: SifrInt = start.clone();
            if stop.is_none() {
                actual_start = SifrInt::from_i64(0);
            } else if let Some(stop) = stop.as_ref() {
                actual_stop_value_351bdef5a4961be0 = stop.clone();
            }
            let width: SifrInt = &actual_stop_value_351bdef5a4961be0 - &actual_start;
            if step_argument_af0b4e191da20cef > &SifrInt::from_i64(0) {
                if &width <= &SifrInt::from_i64(0) {
                    return Err(ValueError::new("randrange: empty range".to_string()));
                }
            } else if &width >= &SifrInt::from_i64(0) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
            let mut abs_width: SifrInt = width.clone();
            if &abs_width < &SifrInt::from_i64(0) {
                abs_width = &SifrInt::from_i64(0) - &abs_width;
            }
            let mut abs_step: SifrInt = step_argument_af0b4e191da20cef.clone();
            if &abs_step < &SifrInt::from_i64(0) {
                abs_step = &SifrInt::from_i64(0) - &abs_step;
            }
            if &abs_step == &SifrInt::from_i64(0) {
                return Err(ValueError::new(
                    "randrange: step must not be zero".to_string(),
                ));
            }
            let count: SifrInt = (&(&abs_width + &abs_step) - &SifrInt::from_i64(1))
                .floor_div_known_nonzero(&abs_step);
            if &count <= &SifrInt::from_i64(0) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
            if &count == &SifrInt::from_i64(0) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
            let pick: SifrInt = self
                .sifr_generated_next_u32()
                .floor_mod_known_nonzero(&count);
            Ok(&actual_start + &(&pick * step_argument_af0b4e191da20cef))
        }
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn randint(
            &mut self,
            minimum: &SifrInt,
            maximum: &SifrInt,
        ) -> Result<SifrInt, ValueError> {
            if *minimum > *maximum {
                return Err(ValueError::new("randint: min must be <= max".to_string()));
            }
            self.randrange(
                minimum,
                &Some((maximum + &SifrInt::from_i64(1)).clone()),
                &SifrInt::from_i64(1),
            )
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
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2euuidX2eUUID {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "UUID(_hex={})", self.hex)
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct IndexError {
        pub message: String,
    }
    impl IndexError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for IndexError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for IndexError {}
}
use crate::sifr_generated_generated_support::*;
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::FloatOverflowError;
pub use sifr_generated_project_nominals::FloatPrecisionLossError;
pub use sifr_generated_project_nominals::IndexError;
pub use sifr_generated_project_nominals::ParseError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2edatetimeX2edatetime;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2edatetimeX2etimezone;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2egraphlibX2eCycleError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2erandomX2eRandom;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2erandomX2eRandomState;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2euuidX2eUUID;
pub use sifr_generated_project_nominals::ValueError;
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    println!("=== 1. random.randint: Validates a <= b ===");
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let _r: SifrInt = randint(SifrInt::from_i64(1), SifrInt::from_i64(10))?;
        println!("randint(1, 10) = ok");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("error: {}", e.message.clone());
    }
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let _r2: SifrInt = randint(SifrInt::from_i64(5), SifrInt::from_i64(3))?;
        println!("should not reach here");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("randint(5, 3) -> ValueError: {}", e.message.clone());
    }
    println!("=== 2. secrets.randbelow: Validates n > 0 ===");
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let _s: SifrInt = randbelow(SifrInt::from_i64(100))?;
        println!("randbelow(100) = ok");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("error: {}", e.message.clone());
    }
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let _s2: SifrInt = randbelow(SifrInt::from_i64(0))?;
        println!("should not reach here");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("randbelow(0) -> ValueError: {}", e.message.clone());
    }
    println!("=== 3. textwrap.wrap: Validates width > 0 ===");
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let lines: Vec<String> = wrap(&"hello world".to_string(), SifrInt::from_i64(5))?;
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(27usize + 7usize);
            sifr_generated_concat.push_str("wrap(hello world, 5) = ok (");
            sifr_generated_concat.push_str(SifrInt::from(lines.len()).to_string().as_str());
            sifr_generated_concat.push_str(" lines)");
            sifr_generated_concat
        });
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(7usize);
            sifr_generated_concat.push_str("error: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let _lines2: Vec<String> = wrap(&"hello".to_string(), SifrInt::from_i64(0))?;
        println!("should not reach here");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(30usize);
            sifr_generated_concat.push_str("wrap(hello, 0) -> ValueError: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    println!("=== 4. itertools.batched: Validates n > 0 ===");
    let data: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
        SifrInt::from_i64(5),
    ];
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let b: Vec<Vec<SifrInt>> = batched(
            &data.iter().cloned().collect::<Vec<_>>(),
            SifrInt::from_i64(2),
        )?;
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(30usize + 9usize);
            sifr_generated_concat.push_str("batched([1,2,3,4,5], 2) = ok (");
            sifr_generated_concat.push_str(SifrInt::from(b.len()).to_string().as_str());
            sifr_generated_concat.push_str(" batches)");
            sifr_generated_concat
        });
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(7usize);
            sifr_generated_concat.push_str("error: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let _b2: Vec<Vec<SifrInt>> = batched(
            &data.iter().cloned().collect::<Vec<_>>(),
            SifrInt::from_i64(0),
        )?;
        println!("should not reach here");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(32usize);
            sifr_generated_concat.push_str("batched(data, 0) -> ValueError: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    println!("=== 5. graphlib.topological_sort: Cycle Detection ===");
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2egraphlibX2eCycleError> =
        (|| {
            let order: Vec<SifrInt> = topological_sort(
                SifrInt::from_i64(3),
                &vec![SifrInt::from_i64(0), SifrInt::from_i64(0)],
                &vec![SifrInt::from_i64(1), SifrInt::from_i64(2)],
            )?;
            println!("{}", {
                let mut sifr_generated_concat: String = String::with_capacity(15usize);
                sifr_generated_concat.push_str("acyclic graph: ");
                sifr_generated_concat.push_str(format!("{order:?}").as_str());
                sifr_generated_concat
            });
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(7usize);
            sifr_generated_concat.push_str("error: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2egraphlibX2eCycleError> =
        (|| {
            let _order2: Vec<SifrInt> = topological_sort(
                SifrInt::from_i64(2),
                &vec![SifrInt::from_i64(0), SifrInt::from_i64(1)],
                &vec![SifrInt::from_i64(1), SifrInt::from_i64(0)],
            )?;
            println!("should not reach here");
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("cyclic graph -> CycleError: {}", e.message.clone());
    }
    println!("=== 6. uuid.uuid_from_hex: Validates hex format ===");
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let _u: SifrGeneratedStdlibSifrX2euuidX2eUUID =
            uuid_from_hex(&"550e8400e29b41d4a716446655440000".to_string())?;
        println!("valid UUID hex: ok");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("error: {}", e.message.clone());
    }
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let _u2: SifrGeneratedStdlibSifrX2euuidX2eUUID =
            uuid_from_hex(&"xyz-invalid!".to_string())?;
        println!("should not reach here");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("invalid chars -> ValueError: {}", e.message.clone());
    }
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let _u3: SifrGeneratedStdlibSifrX2euuidX2eUUID = uuid_from_hex(&"abcd1234".to_string())?;
        println!("should not reach here");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("wrong length -> ValueError: {}", e.message.clone());
    }
    println!("=== 7. ipaddress.ip_to_int: Validates IPv4 format ===");
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let _ip: SifrInt = ip_to_int(&"192.168.1.1".to_string())?;
        println!("ip_to_int(192.168.1.1) = ok");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(7usize);
            sifr_generated_concat.push_str("error: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let _ip2: SifrInt = ip_to_int(&"bad".to_string())?;
        println!("should not reach here");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(30usize);
            sifr_generated_concat.push_str("ip_to_int(bad) -> ValueError: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    println!("=== 8. datetime.from_timestamp: Validates timestamp ===");
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let _dt: SifrGeneratedStdlibSifrX2edatetimeX2edatetime = from_timestamp(0.0_f64, &None)?;
        println!("from_timestamp(0.0) = ok");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("error: {}", e.message.clone());
    }
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let _dt2: SifrGeneratedStdlibSifrX2edatetimeX2edatetime =
            from_timestamp(-99_999_999_999_999.0_f64, &None)?;
        println!("should not reach here");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!(
            "from_timestamp(invalid) -> ValueError: {}",
            e.message.clone()
        );
    }
    println!("=== 9. SubscriptAssign: Bounds-checked (IndexError) ===");
    let mut nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(10),
        SifrInt::from_i64(20),
        SifrInt::from_i64(30),
    ];
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(8usize);
        sifr_generated_concat.push_str("before: ");
        sifr_generated_concat.push_str(format!("{nums:?}").as_str());
        sifr_generated_concat
    });
    let sifr_generated_try_res: Result<(), IndexError> = (|| {
        {
            let sifr_generated_assign_value = SifrInt::from_i64(999);
            {
                let sifr_generated_index_raw = SifrInt::from_i64(99);
                let sifr_generated_index_normalized =
                    sifr_generated_index_raw.normalize_index_or_len(nums.len());
                if let Some(sifr_generated_elem) = nums.get_mut(sifr_generated_index_normalized) {
                    *sifr_generated_elem = sifr_generated_assign_value;
                } else {
                    return Err(IndexError::new("collection index out of range".to_string()));
                }
            }
        }
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e_5f65 = sifr_generated_try_err.clone();
        println!("out-of-bounds assign -> IndexError");
    }
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(27usize);
        sifr_generated_concat.push_str("after out-of-bounds error: ");
        sifr_generated_concat.push_str(format!("{nums:?}").as_str());
        sifr_generated_concat
    });
    if &SifrInt::from(nums.len()) > &SifrInt::from_i64(1) {
        {
            let sifr_generated_assign_value = SifrInt::from_i64(99);
            {
                let sifr_generated_index_raw = SifrInt::from_i64(1);
                let sifr_generated_index_normalized =
                    sifr_generated_index_raw.normalize_index_or_len(nums.len());
                if let Some(sifr_generated_elem) = nums.get_mut(sifr_generated_index_normalized) {
                    *sifr_generated_elem = sifr_generated_assign_value;
                }
            }
        }
    }
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(20usize);
        sifr_generated_concat.push_str("after valid assign: ");
        sifr_generated_concat.push_str(format!("{nums:?}").as_str());
        sifr_generated_concat
    });
    println!("demo complete!");
}
