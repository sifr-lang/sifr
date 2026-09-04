// src/main.rs
pub mod sifr_generated_generated_support {
    use crate::{
        FloatOverflowError, FloatPrecisionLossError, ParseError,
        SifrGeneratedStdlibSifrX2edatetimeX2edatetime,
        SifrGeneratedStdlibSifrX2edatetimeX2etimezone, ValueError,
    };
    pub(super) use ::sifr_runtime::SifrInt;
    pub(super) fn datetime_now_struct() -> Vec<SifrInt> {
        ::sifr_stdlib::time::datetime_now_struct()
            .into_iter()
            .map(::sifr_runtime::interop::SifrIntBridge::into_sifr_int)
            .collect()
    }
    pub(super) fn datetime_from_timestamp(ts: f64) -> Result<String, ValueError> {
        ::sifr_stdlib::time::datetime_from_timestamp(ts).map_err(|sifr_generated_bridge_error| {
            ValueError {
                message: sifr_generated_bridge_error.to_string(),
            }
        })
    }
    pub(super) fn time_now() -> f64 {
        ::sifr_stdlib::time::time_now()
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub(super) enum SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
            FloatOverflowError,
        ),
        SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
            FloatPrecisionLossError,
        ),
    }
    impl From<FloatOverflowError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
        fn from(value: FloatOverflowError) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
                value,
            )
        }
    }
    impl From<FloatPrecisionLossError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
        fn from(value: FloatPrecisionLossError) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                Self::SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                Self::SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
    #[derive(Debug, Clone)]
    pub(super) enum SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
            FloatOverflowError,
        ),
        SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
            FloatPrecisionLossError,
        ),
        SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0(ValueError),
    }
    impl From<FloatOverflowError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
        fn from(value: FloatOverflowError) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
                value,
            )
        }
    }
    impl From<FloatPrecisionLossError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
        fn from(value: FloatPrecisionLossError) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
                value,
            )
        }
    }
    impl From<ValueError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
        fn from(value: ValueError) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                Self::SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                Self::SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                Self::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
    #[expect(
        clippy::needless_pass_by_value,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn sifr_generated_substring(value: &str, start: SifrInt, end: SifrInt) -> String {
        let sifr_generated_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
        let mut result: String = String::new();
        let mut i: SifrInt = start;
        while i < end {
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
            i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
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
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub(super) fn sifr_generated_parse_datetime_iso(
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
        #[expect(
            clippy::type_complexity,
            reason = "language necessity: this generated carrier preserves nested typed Sifr error and tuple structure; owner Item 12; remove when the carrier representation changes"
        )]
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
            let _ = sifr_generated_try_err;
            Err(ValueError::new("invalid datetime string".to_string()))
        })
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    #[expect(
        clippy::ref_option,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn sifr_generated_from_timestamp_with_tz(
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
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0,
                )?;
            let whole_seconds_float: f64 = whole_seconds
                .checked_to_f64()
                .map_err(|sifr_generated_float_error| match sifr_generated_float_error {
                    ::sifr_runtime::IntegerFloatConversionError::Overflow => {
                        SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
                            FloatOverflowError::new(
                                "exact integer is outside the finite float range"
                                    .to_string(),
                            ),
                        )
                    }
                    ::sifr_runtime::IntegerFloatConversionError::PrecisionLoss => {
                        SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
                            FloatPrecisionLossError::new(
                                "exact integer cannot be represented without float precision loss"
                                    .to_string(),
                            ),
                        )
                    }
                })
                .map_err(|sifr_generated_e| match sifr_generated_e {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
                        sifr_generated_union_value,
                    ) => {
                        SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
                            sifr_generated_union_value,
                        )
                    }
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
                        sifr_generated_union_value,
                    ) => {
                        SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
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
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0,
                )?;
            if microsecond < SifrInt::from_i64(0) {
                microsecond = ::std::ops::Neg::neg(&microsecond);
            }
            let mut adjusted_seconds: SifrInt = whole_seconds.clone();
            let mut tz_offset_value: SifrInt = SifrInt::from_i64(0);
            let tz_has_offset: bool = tz
                .as_ref()
                .is_some_and(|tz| {
                    let tz_offset: SifrInt = tz.offset();
                    adjusted_seconds = ::std::ops::Add::add(&whole_seconds, &tz_offset);
                    tz_offset_value = tz_offset;
                    true
                });
            let adjusted_seconds_float: f64 = adjusted_seconds
                .checked_to_f64()
                .map_err(|sifr_generated_float_error| match sifr_generated_float_error {
                    ::sifr_runtime::IntegerFloatConversionError::Overflow => {
                        SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
                            FloatOverflowError::new(
                                "exact integer is outside the finite float range"
                                    .to_string(),
                            ),
                        )
                    }
                    ::sifr_runtime::IntegerFloatConversionError::PrecisionLoss => {
                        SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
                            FloatPrecisionLossError::new(
                                "exact integer cannot be represented without float precision loss"
                                    .to_string(),
                            ),
                        )
                    }
                })
                .map_err(|sifr_generated_e| match sifr_generated_e {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
                        sifr_generated_union_value,
                    ) => {
                        SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
                            sifr_generated_union_value,
                        )
                    }
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
                        sifr_generated_union_value,
                    ) => {
                        SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
                            sifr_generated_union_value,
                        )
                    }
                })?;
            let rendered: String = datetime_from_timestamp(adjusted_seconds_float)
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0,
                )?;
            let parts: (SifrInt, SifrInt, SifrInt, SifrInt, SifrInt, SifrInt) = sifr_generated_parse_datetime_iso(
                    &rendered,
                )
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0,
                )?;
            let year_part: Option<SifrInt> = Some(parts.0.clone());
            let month_part: Option<SifrInt> = Some(parts.1.clone());
            let day_part: Option<SifrInt> = Some(parts.2.clone());
            let hour_part: Option<SifrInt> = Some(parts.3.clone());
            let minute_part: Option<SifrInt> = Some(parts.4.clone());
            let second_part: Option<SifrInt> = Some(parts.5);
            let mut year: SifrInt = SifrInt::from_i64(0);
            let mut month: SifrInt = SifrInt::from_i64(1);
            let mut day: SifrInt = SifrInt::from_i64(1);
            let mut hour: SifrInt = SifrInt::from_i64(0);
            let mut minute: SifrInt = SifrInt::from_i64(0);
            let mut second: SifrInt = SifrInt::from_i64(0);
            if let Some(year_part) = year_part {
                year = year_part;
            }
            if let Some(month_part) = month_part {
                month = month_part;
            }
            if let Some(day_part) = day_part {
                day = day_part;
            }
            if let Some(hour_part) = hour_part {
                hour = hour_part;
            }
            if let Some(minute_part) = minute_part {
                minute = minute_part;
            }
            if let Some(second_part) = second_part {
                second = second_part;
            }
            if tz_has_offset {
                return Ok(
                    Ok(
                        SifrGeneratedStdlibSifrX2edatetimeX2edatetime::new(
                            year,
                            month,
                            day,
                            hour,
                            minute,
                            second,
                            microsecond,
                            Some(tz_offset_value),
                        ),
                    ),
                );
            }
            Ok(
                Ok(
                    SifrGeneratedStdlibSifrX2edatetimeX2edatetime::new(
                        year,
                        month,
                        day,
                        hour,
                        minute,
                        second,
                        microsecond,
                        None,
                    ),
                ),
            )
        })();
        sifr_generated_try_res
            .unwrap_or_else(|sifr_generated_try_err| match sifr_generated_try_err {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let e = sifr_generated_try_variant_error;
                    Err(ValueError::new(e.message))
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let e = sifr_generated_try_variant_error;
                    Err(ValueError::new(e.message))
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let e = sifr_generated_try_variant_error;
                    Err(ValueError::new(e.message))
                }
            })
    }
    #[expect(
        clippy::ref_option,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn now(
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
                let _ = sifr_generated_try_err;
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
                            ::std::ops::Add::add(
                                SifrInt::from(sifr_generated_pair.0),
                                SifrInt::from_i64(0),
                            ),
                            sifr_generated_pair.1,
                        )
                    },
                )) {
                    if i == SifrInt::from_i64(0) {
                        yr.clone_from(&v);
                    }
                    if i == SifrInt::from_i64(1) {
                        mo.clone_from(&v);
                    }
                    if i == SifrInt::from_i64(2) {
                        dy.clone_from(&v);
                    }
                    if i == SifrInt::from_i64(3) {
                        hr.clone_from(&v);
                    }
                    if i == SifrInt::from_i64(4) {
                        mn.clone_from(&v);
                    }
                    if i == SifrInt::from_i64(5) {
                        sc.clone_from(&v);
                    }
                }
                let Some(tz) = tz.as_ref() else {
                    return SifrGeneratedStdlibSifrX2edatetimeX2edatetime::new(
                        yr,
                        mo,
                        dy,
                        hr,
                        mn,
                        sc,
                        SifrInt::from_i64(0),
                        None,
                    );
                };
                SifrGeneratedStdlibSifrX2edatetimeX2edatetime::new(
                    yr,
                    mo,
                    dy,
                    hr,
                    mn,
                    sc,
                    SifrInt::from_i64(0),
                    Some(tz.offset()),
                )
            }
        }
    }
    #[expect(
        clippy::ref_option,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn from_timestamp(
        ts: f64,
        tz: &Option<SifrGeneratedStdlibSifrX2edatetimeX2etimezone>,
    ) -> Result<SifrGeneratedStdlibSifrX2edatetimeX2edatetime, ValueError> {
        sifr_generated_from_timestamp_with_tz(ts, tz)
    }
    pub(super) fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
        assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < actual.len() {
            assert_eq!(
                {
                    let sifr_generated_condition_list = &actual;
                    let sifr_generated_condition_index = i.clone();
                    let sifr_generated_condition_normalized = sifr_generated_condition_index
                        .normalize_index_or_len(sifr_generated_condition_list.len());
                    sifr_generated_condition_list
                        .get(sifr_generated_condition_normalized)
                        .copied()
                },
                {
                    let sifr_generated_condition_list = &expected;
                    let sifr_generated_condition_index = i.clone();
                    let sifr_generated_condition_normalized = sifr_generated_condition_index
                        .normalize_index_or_len(sifr_generated_condition_list.len());
                    sifr_generated_condition_list
                        .get(sifr_generated_condition_normalized)
                        .copied()
                }
            );
            i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
        }
    }
}
mod sifr_generated_project_nominals {
    use crate::sifr_generated_generated_support::sifr_generated_six_digits;
    use ::sifr_runtime::SifrInt;
    #[derive(Debug, Clone)]
    pub struct SifrGeneratedStdlibSifrX2edatetimeX2etimedelta {
        pub days: SifrInt,
        pub seconds: SifrInt,
        pub microseconds: SifrInt,
    }
    impl SifrGeneratedStdlibSifrX2edatetimeX2etimedelta {
        #[must_use]
        pub const fn new(days: SifrInt, seconds: SifrInt, microseconds: SifrInt) -> Self {
            let sifr_generated_field_value_906603c80a0dd39d_5f64617973: SifrInt = days;
            let sifr_generated_field_value_7cbedb13c5d2304b_5f7365636f6e6473: SifrInt = seconds;
            let sifr_generated_field_value_fb3e1ecc2972a7bf_5f6d6963726f7365636f6e6473: SifrInt =
                microseconds;
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
            ::std::ops::Add::add(
                &::std::ops::Mul::mul(&self.days.clone(), &SifrInt::from_i64(86400)),
                &self.seconds.clone(),
            )
        }
    }
    impl SifrGeneratedStdlibSifrX2edatetimeX2etimedelta {
        #[must_use]
        pub fn total_microseconds(&self) -> SifrInt {
            ::std::ops::Add::add(
                &::std::ops::Mul::mul(
                    &::std::ops::Add::add(
                        &::std::ops::Mul::mul(&self.days.clone(), &SifrInt::from_i64(86400)),
                        &self.seconds.clone(),
                    ),
                    &SifrInt::from_i64(1_000_000),
                ),
                &self.microseconds.clone(),
            )
        }
    }
    impl ::std::ops::Add<&SifrGeneratedStdlibSifrX2edatetimeX2etimedelta>
        for &SifrGeneratedStdlibSifrX2edatetimeX2etimedelta
    {
        type Output = SifrGeneratedStdlibSifrX2edatetimeX2etimedelta;
        fn add(self, other: &SifrGeneratedStdlibSifrX2edatetimeX2etimedelta) -> Self::Output {
            let total: SifrInt =
                ::std::ops::Add::add(&self.total_microseconds(), &other.total_microseconds());
            let d: SifrInt = total.floor_div_known_nonzero(&SifrInt::from_i64(86_400_000_000));
            let remaining: SifrInt =
                total.floor_mod_known_nonzero(&SifrInt::from_i64(86_400_000_000));
            let s: SifrInt = remaining.floor_div_known_nonzero(&SifrInt::from_i64(1_000_000));
            let us: SifrInt = remaining.floor_mod_known_nonzero(&SifrInt::from_i64(1_000_000));
            SifrGeneratedStdlibSifrX2edatetimeX2etimedelta::new(d, s, us)
        }
    }
    impl ::std::ops::Sub<&SifrGeneratedStdlibSifrX2edatetimeX2etimedelta>
        for &SifrGeneratedStdlibSifrX2edatetimeX2etimedelta
    {
        type Output = SifrGeneratedStdlibSifrX2edatetimeX2etimedelta;
        fn sub(self, other: &SifrGeneratedStdlibSifrX2edatetimeX2etimedelta) -> Self::Output {
            let total: SifrInt =
                ::std::ops::Sub::sub(&self.total_microseconds(), &other.total_microseconds());
            let d: SifrInt = total.floor_div_known_nonzero(&SifrInt::from_i64(86_400_000_000));
            let remaining: SifrInt =
                total.floor_mod_known_nonzero(&SifrInt::from_i64(86_400_000_000));
            let s: SifrInt = remaining.floor_div_known_nonzero(&SifrInt::from_i64(1_000_000));
            let us: SifrInt = remaining.floor_mod_known_nonzero(&SifrInt::from_i64(1_000_000));
            SifrGeneratedStdlibSifrX2edatetimeX2etimedelta::new(d, s, us)
        }
    }
    impl PartialEq for SifrGeneratedStdlibSifrX2edatetimeX2etimedelta {
        fn eq(&self, other: &Self) -> bool {
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
    #[derive(Debug, Clone)]
    pub struct SifrGeneratedStdlibSifrX2edatetimeX2etimezone {
        pub offset: SifrInt,
    }
    impl SifrGeneratedStdlibSifrX2edatetimeX2etimezone {
        #[must_use]
        pub const fn new(offset: SifrInt) -> Self {
            let sifr_generated_field_value_d85dd81618b4c959_5f6f6666736574: SifrInt = offset;
            Self {
                offset: sifr_generated_field_value_d85dd81618b4c959_5f6f6666736574,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2edatetimeX2etimezone {
        #[must_use]
        pub fn offset(&self) -> SifrInt {
            self.offset.clone()
        }
    }
    impl SifrGeneratedStdlibSifrX2edatetimeX2etimezone {
        #[must_use]
        pub fn iso_suffix(&self) -> String {
            let sign: String = if self.offset < SifrInt::from_i64(0) {
                "-".to_string()
            } else {
                "+".to_string()
            };
            let mut abs_offset: SifrInt = self.offset.clone();
            if abs_offset < SifrInt::from_i64(0) {
                abs_offset = ::std::ops::Neg::neg(&abs_offset);
            }
            let h: SifrInt = abs_offset.floor_div_known_nonzero(&SifrInt::from_i64(3600));
            let m: SifrInt = abs_offset
                .floor_mod_known_nonzero(&SifrInt::from_i64(3600))
                .floor_div_known_nonzero(&SifrInt::from_i64(60));
            let mut hs: String = h.to_string();
            if hs.chars().count() < SifrInt::from_i64(2) {
                hs = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize.saturating_add(hs.len()));
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(hs.as_str());
                    sifr_generated_concat
                };
            }
            let mut ms: String = m.to_string();
            if ms.chars().count() < SifrInt::from_i64(2) {
                ms = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize.saturating_add(ms.len()));
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(ms.as_str());
                    sifr_generated_concat
                };
            }
            {
                let mut sifr_generated_concat: String = String::with_capacity(
                    sign.len()
                        .saturating_add(hs.len())
                        .saturating_add(1usize)
                        .saturating_add(ms.len()),
                );
                sifr_generated_concat.push_str(sign.as_str());
                sifr_generated_concat.push_str(hs.as_str());
                sifr_generated_concat.push(':');
                sifr_generated_concat.push_str(ms.as_str());
                sifr_generated_concat
            }
        }
    }
    impl PartialEq for SifrGeneratedStdlibSifrX2edatetimeX2etimezone {
        fn eq(&self, other: &Self) -> bool {
            self.offset == other.offset
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2edatetimeX2etimezone {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            if self.offset == SifrInt::from_i64(0) {
                return write!(f, "UTC");
            }
            write!(f, "{}", {
                let mut sifr_generated_concat: String =
                    String::with_capacity(3usize.saturating_add(0usize));
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
}
use crate::sifr_generated_generated_support::{assert_bool_vector_eq, from_timestamp, now};
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::FloatOverflowError;
pub use sifr_generated_project_nominals::FloatPrecisionLossError;
pub use sifr_generated_project_nominals::ParseError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2edatetimeX2edatetime;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2edatetimeX2etimedelta;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2edatetimeX2etimezone;
pub use sifr_generated_project_nominals::ValueError;
fn collect_positive_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let dt: SifrGeneratedStdlibSifrX2edatetimeX2edatetime =
        SifrGeneratedStdlibSifrX2edatetimeX2edatetime::new(
            SifrInt::from_i64(2024),
            SifrInt::from_i64(1),
            SifrInt::from_i64(15),
            SifrInt::from_i64(10),
            SifrInt::from_i64(30),
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
            None,
        );
    actual.push(dt.isoformat().as_str() == "2024-01-15T10:30:00".to_string().as_str());
    let base_td: SifrGeneratedStdlibSifrX2edatetimeX2etimedelta =
        SifrGeneratedStdlibSifrX2edatetimeX2etimedelta::new(
            SifrInt::from_i64(0),
            SifrInt::from_i64(3600),
            SifrInt::from_i64(0),
        );
    let extra_td: SifrGeneratedStdlibSifrX2edatetimeX2etimedelta =
        SifrGeneratedStdlibSifrX2edatetimeX2etimedelta::new(
            SifrInt::from_i64(0),
            SifrInt::from_i64(1800),
            SifrInt::from_i64(0),
        );
    let td: SifrGeneratedStdlibSifrX2edatetimeX2etimedelta =
        ::std::ops::Add::add(&base_td, &extra_td);
    actual.push(td.total_seconds() == SifrInt::from_i64(5400));
    let current: SifrGeneratedStdlibSifrX2edatetimeX2edatetime = now(&None);
    actual.push(
        current.year > SifrInt::from_i64(2020)
            && current.month >= SifrInt::from_i64(1)
            && current.month <= SifrInt::from_i64(12),
    );
    let mut epoch_ok: bool = false;
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let epoch: SifrGeneratedStdlibSifrX2edatetimeX2edatetime = from_timestamp(0.0_f64, &None)?;
        epoch_ok = epoch.isoformat() == "1970-01-01T00:00:00";
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        let _ = e.message;
    }
    actual.push(epoch_ok);
    actual.push(
        SifrGeneratedStdlibSifrX2edatetimeX2etimezone::new(::std::ops::Neg::neg(
            &SifrInt::from_i64(19800),
        ))
        .to_string()
        .as_str()
            == "UTC-05:30".to_string().as_str(),
    );
    actual
}
fn collect_negative_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let mut invalid_rejected: bool = false;
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let sifr_generated_bad: SifrGeneratedStdlibSifrX2edatetimeX2edatetime =
            from_timestamp(100_000_000_000_000_000_000.0_f64, &None)?;
        let _ = sifr_generated_bad;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        invalid_rejected = e.message.chars().count() > SifrInt::from_i64(0);
    }
    actual.push(invalid_rejected);
    actual
}
fn append_all(target: &mut Vec<bool>, values: &[bool]) {
    for value in values.iter().copied() {
        target.push(value);
    }
}
fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true];
    let mut actual: Vec<bool> = Vec::new();
    append_all(&mut actual, &collect_positive_actual());
    append_all(&mut actual, &collect_negative_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("datetime datetime parity demo: pass");
}
