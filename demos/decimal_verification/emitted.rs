// src/main.rs
pub mod sifr_generated_generated_support {
    use crate::SifrGeneratedStdlibSifrX2ejsonX2eJsonValue;
    pub(super) use ::bigdecimal::BigDecimal;
    pub(super) use ::rust_decimal::Decimal;
    pub(super) use ::sifr_runtime::SifrInt;
    pub(super) fn json_dump_tokens(tokens: &[String]) -> String {
        ::sifr_stdlib::json::json_dump_tokens(tokens)
    }
    #[derive(Debug, Clone, PartialEq)]
    pub(super) enum SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0(
            SifrGeneratedStdlibSifrX2ejsonX2eJsonValue,
        ),
        SifrGeneratedUnionVariant4X3aatom7X3adecimal(Decimal),
        SifrGeneratedUnionVariant4X3aatom10X3abigdecimal(BigDecimal),
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                Self::SifrGeneratedUnionVariant5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0(
                    v,
                ) => write!(f, "{v}"),
                Self::SifrGeneratedUnionVariant4X3aatom7X3adecimal(v) => write!(f, "{v}"),
                Self::SifrGeneratedUnionVariant4X3aatom10X3abigdecimal(v) => {
                    write!(f, "{v}")
                }
            }
        }
    }
    pub(super) fn from_str(value: &str) -> SifrGeneratedStdlibSifrX2ejsonX2eJsonValue {
        let str_value: Option<String> = Some({
            let mut sifr_generated_concat: String =
                String::with_capacity(value.len().saturating_add(0usize));
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
    pub(super) fn dumps(
        value: &SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0,
    ) -> String {
        match value {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0::SifrGeneratedUnionVariant5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0(
                value,
            ) => json_dump_tokens(&sifr_generated_json_bridge_tokens(value)),
            value => {
                json_dump_tokens(
                    &sifr_generated_json_bridge_tokens(&from_str(&value.to_string())),
                )
            }
        }
    }
}
mod sifr_generated_project_nominals {
    use crate::sifr_generated_generated_support::{
        SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0,
        dumps,
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
            int_value: Option<SifrInt>,
            float_value: Option<f64>,
            str_value: Option<String>,
        ) -> Self {
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
        pub fn as_str(&self) -> Option<String> {
            self.str_value.clone()
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
    pub struct DivisionError {
        pub message: String,
    }
    impl DivisionError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for DivisionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for DivisionError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct DecimalConversionError {
        pub message: String,
    }
    impl DecimalConversionError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for DecimalConversionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for DecimalConversionError {}
}
pub use sifr_generated_project_nominals::DecimalConversionError;
pub use sifr_generated_project_nominals::DivisionError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ejsonX2eJsonValue;
mod sifr_generated_project_unions {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0(
            crate::sifr_generated_project_nominals::DecimalConversionError,
        ),
        SifrGeneratedUnionVariant5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a0(
            crate::sifr_generated_project_nominals::DivisionError,
        ),
    }
    impl From<crate::sifr_generated_project_nominals::DecimalConversionError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0 {
        fn from(
            value: crate::sifr_generated_project_nominals::DecimalConversionError,
        ) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0(
                value,
            )
        }
    }
    impl From<crate::sifr_generated_project_nominals::DivisionError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0 {
        fn from(value: crate::sifr_generated_project_nominals::DivisionError) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                Self::SifrGeneratedUnionVariant5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                Self::SifrGeneratedUnionVariant5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
}
use crate::sifr_generated_generated_support::{
    SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0,
    dumps,
};
use ::bigdecimal::BigDecimal;
use ::rust_decimal::Decimal;
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_unions::SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0;
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
#[expect(
    clippy::assertions_on_constants,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn main() {
    println!("decimal_verification verification corpus and determinism gates demo");
    let d: Decimal = Decimal::from_i128_with_scale(-75_i128, 1);
    let bd: BigDecimal = BigDecimal::new(
        ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&[181]),
        1,
    );
    println!(
        "{}", dumps(&
        SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0::SifrGeneratedUnionVariant4X3aatom7X3adecimal(Decimal::from_i128_with_scale(12300_i128,
        4)))
    );
    println!(
        "{}", dumps(&
        SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0::SifrGeneratedUnionVariant4X3aatom10X3abigdecimal(BigDecimal::new(::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&
        [48, 12]), 4)))
    );
    let sifr_generated_try_res: Result<
        (),
        SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0,
    > = (|| {
        let d_floor: Decimal = {
            let sifr_generated_decimal_left_result = Ok(d);
            let sifr_generated_decimal_right_result = Ok(
                Decimal::from_i128_with_scale(2_i128, 0),
            );
            sifr_generated_decimal_left_result
                .and_then(move |sifr_generated_decimal_left| {
                    sifr_generated_decimal_right_result
                        .and_then(move |sifr_generated_decimal_right| {
                            Decimal::checked_div(
                                    sifr_generated_decimal_left,
                                    sifr_generated_decimal_right,
                                )
                                .map(|sifr_generated_decimal_quotient| {
                                    sifr_generated_decimal_quotient.floor()
                                })
                                .map_or_else(
                                    || Err(
                                        if sifr_generated_decimal_right.is_zero() {
                                            SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0::SifrGeneratedUnionVariant5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a0(
                                                DivisionError::new("division by zero".to_string()),
                                            )
                                        } else {
                                            SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0::SifrGeneratedUnionVariant5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0(
                                                DecimalConversionError::new(
                                                    "decimal // operation overflowed its exact representation"
                                                        .to_string(),
                                                ),
                                            )
                                        },
                                    ),
                                    Ok,
                                )
                        })
                })
        }?;
        let d_remainder: Decimal = {
            let sifr_generated_decimal_left_result = Ok(d);
            let sifr_generated_decimal_right_result = Ok(
                Decimal::from_i128_with_scale(2_i128, 0),
            );
            sifr_generated_decimal_left_result
                .and_then(move |sifr_generated_decimal_left| {
                    sifr_generated_decimal_right_result
                        .and_then(move |sifr_generated_decimal_right| {
                            Decimal::checked_div(
                                    sifr_generated_decimal_left,
                                    sifr_generated_decimal_right,
                                )
                                .and_then(|sifr_generated_decimal_quotient| {
                                    Decimal::checked_mul(
                                            sifr_generated_decimal_quotient.floor(),
                                            sifr_generated_decimal_right,
                                        )
                                        .and_then(|sifr_generated_decimal_product| Decimal::checked_sub(
                                            sifr_generated_decimal_left,
                                            sifr_generated_decimal_product,
                                        ))
                                })
                                .map_or_else(
                                    || Err(
                                        if sifr_generated_decimal_right.is_zero() {
                                            SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0::SifrGeneratedUnionVariant5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a0(
                                                DivisionError::new("division by zero".to_string()),
                                            )
                                        } else {
                                            SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0::SifrGeneratedUnionVariant5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0(
                                                DecimalConversionError::new(
                                                    "decimal % operation overflowed its exact representation"
                                                        .to_string(),
                                                ),
                                            )
                                        },
                                    ),
                                    Ok,
                                )
                        })
                })
        }?;
        let bd_floor_value_78a84cd7a18822c2: BigDecimal = {
            let sifr_generated_bigdecimal_left = bd.clone();
            let sifr_generated_bigdecimal_right = BigDecimal::new(
                    ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&[2]),
                    0,
                )
                .clone();
            if ::bigdecimal::Zero::is_zero(&sifr_generated_bigdecimal_right) {
                Err(DivisionError::new("division by zero".to_string()))
            } else {
                Ok(
                    ::bigdecimal::Context::new(
                            ::std::num::NonZeroU64::MIN.saturating_add(27),
                            ::bigdecimal::RoundingMode::HalfEven,
                        )
                        .round_decimal_ref(
                            &(&sifr_generated_bigdecimal_left
                                / &sifr_generated_bigdecimal_right)
                                .with_scale_round(0, ::bigdecimal::RoundingMode::Floor),
                        ),
                )
            }
        }
            .map_err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0::SifrGeneratedUnionVariant5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a0,
            )?;
        let bd_remainder_value_c770fb6e5eee30f3: BigDecimal = {
            let sifr_generated_bigdecimal_left = bd.clone();
            let sifr_generated_bigdecimal_right = BigDecimal::new(
                    ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&[2]),
                    0,
                )
                .clone();
            if ::bigdecimal::Zero::is_zero(&sifr_generated_bigdecimal_right) {
                Err(DivisionError::new("division by zero".to_string()))
            } else {
                Ok(
                    ::bigdecimal::Context::new(
                            ::std::num::NonZeroU64::MIN.saturating_add(27),
                            ::bigdecimal::RoundingMode::HalfEven,
                        )
                        .round_decimal_ref(
                            &(&sifr_generated_bigdecimal_left
                                - (&sifr_generated_bigdecimal_left
                                    / &sifr_generated_bigdecimal_right)
                                    .with_scale_round(0, ::bigdecimal::RoundingMode::Floor)
                                    * &sifr_generated_bigdecimal_right),
                        ),
                )
            }
        }
            .map_err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0::SifrGeneratedUnionVariant5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a0,
            )?;
        println!("{d_floor}");
        println!("{d_remainder}");
        println!("{bd_floor_value_78a84cd7a18822c2}");
        println!("{bd_remainder_value_c770fb6e5eee30f3}");
        let baseline_tmp_d: Decimal = {
            let sifr_generated_decimal_left_result = Ok(
                Decimal::from_i128_with_scale(12345_i128, 4),
            );
            let sifr_generated_decimal_right_result = Ok(
                Decimal::from_i128_with_scale(30_i128, 1),
            );
            sifr_generated_decimal_left_result
                .and_then(move |sifr_generated_decimal_left| {
                    sifr_generated_decimal_right_result
                        .and_then(move |sifr_generated_decimal_right| {
                            Decimal::checked_mul(
                                    sifr_generated_decimal_left,
                                    sifr_generated_decimal_right,
                                )
                                .map_or_else(
                                    || Err(
                                        DecimalConversionError::new(
                                            "decimal * operation overflowed its exact representation"
                                                .to_string(),
                                        ),
                                    ),
                                    Ok,
                                )
                        })
                })
        }
            .map_err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0::SifrGeneratedUnionVariant5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0,
            )?;
        let baseline_d: String = baseline_tmp_d
            .round_dp_with_strategy(
                {
                    let sifr_generated_scale = 3;
                    (if sifr_generated_scale < 0 { 0 } else { sifr_generated_scale })
                        as u32
                },
                ::rust_decimal::RoundingStrategy::MidpointNearestEven,
            )
            .to_string();
        let baseline_bd_value_ad11946794caa821: String = ::bigdecimal::Context::new(
                ::std::num::NonZeroU64::MIN.saturating_add(27),
                ::bigdecimal::RoundingMode::HalfEven,
            )
            .round_decimal_ref(
                &(BigDecimal::new(
                        ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(
                            &[39, 228, 27, 50, 70, 190, 201, 177, 110, 57, 129, 21],
                        ),
                        28,
                    )
                    .clone()
                    + BigDecimal::new(
                            ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&[0]),
                            0,
                        )
                        .clone()),
            )
            .round(6)
            .to_string();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < SifrInt::from_i64(20) {
            let loop_tmp_d: Decimal = {
                let sifr_generated_decimal_left_result = Ok(
                    Decimal::from_i128_with_scale(12345_i128, 4),
                );
                let sifr_generated_decimal_right_result = Ok(
                    Decimal::from_i128_with_scale(30_i128, 1),
                );
                sifr_generated_decimal_left_result
                    .and_then(move |sifr_generated_decimal_left| {
                        sifr_generated_decimal_right_result
                            .and_then(move |sifr_generated_decimal_right| {
                                Decimal::checked_mul(
                                        sifr_generated_decimal_left,
                                        sifr_generated_decimal_right,
                                    )
                                    .map_or_else(
                                        || Err(
                                            DecimalConversionError::new(
                                                "decimal * operation overflowed its exact representation"
                                                    .to_string(),
                                            ),
                                        ),
                                        Ok,
                                    )
                            })
                    })
            }
                .map_err(
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0::SifrGeneratedUnionVariant5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0,
                )?;
            assert_eq!(
                loop_tmp_d.round_dp_with_strategy({ let sifr_generated_scale = 3; (if
                sifr_generated_scale < 0 { 0 } else { sifr_generated_scale }) as u32 },
                ::rust_decimal::RoundingStrategy::MidpointNearestEven).to_string(),
                baseline_d
            );
            assert_eq!(
                ::bigdecimal::Context::new(::std::num::NonZeroU64::MIN
                .saturating_add(27), ::bigdecimal::RoundingMode::HalfEven)
                .round_decimal_ref(&
                (BigDecimal::new(::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&
                [39, 228, 27, 50, 70, 190, 201, 177, 110, 57, 129, 21]), 28).clone() +
                BigDecimal::new(::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&
                [0]), 0).clone())).round(6).to_string(),
                baseline_bd_value_ad11946794caa821
            );
            i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
        }
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        match sifr_generated_try_err {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0::SifrGeneratedUnionVariant5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let error = sifr_generated_try_variant_error;
                assert!(false, "{error}");
            }
            SifrGeneratedUnion8X3asequence5X3aunion1X3a239X3a5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a048X3a5X3aclass35X3asifrX2ebuiltinX2eDecimalConversionError1X3a0::SifrGeneratedUnionVariant5X3aclass26X3asifrX2ebuiltinX2eDivisionError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let error = sifr_generated_try_variant_error;
                assert!(false, "{error}");
            }
        }
    }
    println!("deterministic decimal and bigdecimal corpus checks passed");
}
