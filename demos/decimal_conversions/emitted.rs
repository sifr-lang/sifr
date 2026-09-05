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
        let str_value: Option<String> = Some(value.to_string());
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
    pub struct DecimalConversionError {
        pub message: String,
    }
    impl ::std::fmt::Display for DecimalConversionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for DecimalConversionError {}
}
use crate::sifr_generated_generated_support::{
    SifrGeneratedUnion8X3asequence5X3aunion1X3a719X3a4X3aatom10X3abigdecimal11X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool13X3a4X3aatom5X3afloat15X3a4X3aatom7X3adecimal32X3a5X3aclass19X3asifrX2ejsonX2eJsonValue1X3a0,
    dumps,
};
use ::bigdecimal::BigDecimal;
use ::num_bigint::BigInt;
use ::rust_decimal::Decimal;
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::DecimalConversionError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ejsonX2eJsonValue;
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    println!("decimal_conversions conversion and boundary rules demo");
    let d: Decimal = Decimal::from_i128_with_scale(-19_i128, 1);
    let bd: BigDecimal = BigDecimal::new(
        ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&[237]),
        1,
    );
    let sifr_generated_try_res: Result<(), DecimalConversionError> = (|| {
        let i_from_decimal: SifrInt = {
            let sifr_generated_decimal_bigint = BigInt::from(d.trunc().mantissa());
            Ok(SifrInt::from_bigint(sifr_generated_decimal_bigint))
        }?;
        let i_from_bigdecimal: SifrInt = SifrInt::parse_decimal(
            &bd.with_scale(0).to_string(),
            ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
        )
        .map_err(|sifr_generated_e| DecimalConversionError {
            message: sifr_generated_e.to_string(),
        })?;
        println!("{i_from_decimal}");
        println!("{i_from_bigdecimal}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(31usize.saturating_add(0usize));
            sifr_generated_concat.push_str("unexpected conversion failure: ");
            sifr_generated_concat.push_str(e.message.as_str());
            sifr_generated_concat
        });
    }
    let bd_from_decimal: BigDecimal = {
        let sifr_generated_decimal = Decimal::from_i128_with_scale(123_400_i128, 4);
        BigDecimal::new(
            ::bigdecimal::num_bigint::BigInt::from(sifr_generated_decimal.mantissa()),
            i64::from(sifr_generated_decimal.scale()),
        )
    };
    println!("{bd_from_decimal}");
    let sifr_generated_try_res: Result<(), DecimalConversionError> = (|| {
        let d_from_bigdecimal_value_3c82d2c6b38e3364: Decimal = Decimal::from_str_exact(
            BigDecimal::new(
                ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&[1, 36, 248]),
                4,
            )
            .to_string()
            .as_str(),
        )
        .map_err(|e| DecimalConversionError {
            message: e.to_string(),
        })?;
        println!("{d_from_bigdecimal_value_3c82d2c6b38e3364}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(39usize.saturating_add(0usize));
            sifr_generated_concat.push_str("unexpected decimal conversion failure: ");
            sifr_generated_concat.push_str(e.message.as_str());
            sifr_generated_concat
        });
    }
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
    let sifr_generated_try_res: Result<(), DecimalConversionError> = (|| {
        let out_of_range: SifrInt = SifrInt::parse_decimal(
            &BigDecimal::new(
                ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&[
                    0, 192, 151, 206, 123, 201, 7, 21, 179, 75, 159, 15, 255, 255, 255, 255,
                ]),
                0,
            )
            .with_scale(0)
            .to_string(),
            ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
        )
        .map_err(|sifr_generated_e| DecimalConversionError {
            message: sifr_generated_e.to_string(),
        })?;
        println!("{out_of_range}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(25usize.saturating_add(0usize));
            sifr_generated_concat.push_str("caught conversion error: ");
            sifr_generated_concat.push_str(e.message.as_str());
            sifr_generated_concat
        });
    }
}
