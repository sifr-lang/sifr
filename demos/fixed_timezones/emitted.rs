// src/main.rs
mod sifr_generated_project_nominals {
    pub use ::sifr_runtime::SifrInt;
    #[derive(Debug, Clone)]
    pub struct SifrGeneratedStdlibSifrX2edatetimeX2etimezone {
        pub offset: SifrInt,
    }
    impl SifrGeneratedStdlibSifrX2edatetimeX2etimezone {
        #[must_use]
        pub fn new(offset: SifrInt) -> Self {
            let sifr_generated_field_value_d85dd81618b4c959_5f6f6666736574: SifrInt =
                offset.clone();
            Self {
                offset: sifr_generated_field_value_d85dd81618b4c959_5f6f6666736574,
            }
        }
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
    #[must_use]
    pub fn sifr_generated_six_digits(value: SifrInt) -> String {
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
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::FloatOverflowError;
pub use sifr_generated_project_nominals::FloatPrecisionLossError;
pub use sifr_generated_project_nominals::ParseError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2edatetimeX2edatetime;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2edatetimeX2etimezone;
pub use sifr_generated_project_nominals::ValueError;
fn datetime_now_struct() -> Vec<SifrInt> {
    ::sifr_stdlib::time::datetime_now_struct()
        .into_iter()
        .map(::sifr_runtime::interop::SifrIntBridge::into_sifr_int)
        .collect()
}
fn datetime_from_timestamp(ts: f64) -> Result<String, ValueError> {
    ::sifr_stdlib::time::datetime_from_timestamp(ts).map_err(|sifr_generated_bridge_error| {
        ValueError {
            message: sifr_generated_bridge_error.to_string(),
        }
    })
}
fn time_now() -> f64 {
    ::sifr_stdlib::time::time_now()
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0
{
    SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(FloatOverflowError),
    SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(FloatPrecisionLossError),
}
impl From<FloatOverflowError>
for SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
    fn from(value: FloatOverflowError) -> Self {
        SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
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
enum SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0
{
    SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(FloatOverflowError),
    SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(FloatPrecisionLossError),
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
fn sifr_generated_substring(value: &str, start: SifrInt, end: SifrInt) -> String {
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
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn sifr_generated_parse_datetime_iso(
    value: &str,
) -> Result<(SifrInt, SifrInt, SifrInt, SifrInt, SifrInt, SifrInt), ValueError> {
    let sifr_generated_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
    let Some(_checked_value_2) = {
        let sifr_generated_string_index = SifrInt::from_i64(4);
        let sifr_generated_string_index_normalized =
            sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_value.len());
        sifr_generated_chars_value
            .get(sifr_generated_string_index_normalized)
            .copied()
    }
    .map(|character| character.to_string()) else {
        return Err(ValueError::new("invalid datetime string".to_string()));
    };
    let Some(_checked_value_3) = {
        let sifr_generated_string_index = SifrInt::from_i64(7);
        let sifr_generated_string_index_normalized =
            sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_value.len());
        sifr_generated_chars_value
            .get(sifr_generated_string_index_normalized)
            .copied()
    }
    .map(|character| character.to_string()) else {
        return Err(ValueError::new("invalid datetime string".to_string()));
    };
    let Some(_checked_value_4) = {
        let sifr_generated_string_index = SifrInt::from_i64(10);
        let sifr_generated_string_index_normalized =
            sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_value.len());
        sifr_generated_chars_value
            .get(sifr_generated_string_index_normalized)
            .copied()
    }
    .map(|character| character.to_string()) else {
        return Err(ValueError::new("invalid datetime string".to_string()));
    };
    let Some(_checked_value_5) = {
        let sifr_generated_string_index = SifrInt::from_i64(13);
        let sifr_generated_string_index_normalized =
            sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_value.len());
        sifr_generated_chars_value
            .get(sifr_generated_string_index_normalized)
            .copied()
    }
    .map(|character| character.to_string()) else {
        return Err(ValueError::new("invalid datetime string".to_string()));
    };
    let Some(_checked_value_6) = {
        let sifr_generated_string_index = SifrInt::from_i64(16);
        let sifr_generated_string_index_normalized =
            sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_value.len());
        sifr_generated_chars_value
            .get(sifr_generated_string_index_normalized)
            .copied()
    }
    .map(|character| character.to_string()) else {
        return Err(ValueError::new("invalid datetime string".to_string()));
    };
    if {
        let sifr_generated_string_index = SifrInt::from_i64(4);
        let sifr_generated_string_index_normalized =
            sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_value.len());
        sifr_generated_chars_value
            .get(sifr_generated_string_index_normalized)
            .copied()
    } != Some("-").and_then(|sifr_generated_cmp_s| {
        let mut sifr_generated_cmp_chars = sifr_generated_cmp_s.chars();
        let sifr_generated_cmp_first = sifr_generated_cmp_chars.next();
        if sifr_generated_cmp_chars.next().is_some() {
            None
        } else {
            sifr_generated_cmp_first
        }
    }) || {
        let sifr_generated_string_index = SifrInt::from_i64(7);
        let sifr_generated_string_index_normalized =
            sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_value.len());
        sifr_generated_chars_value
            .get(sifr_generated_string_index_normalized)
            .copied()
    } != Some("-").and_then(|sifr_generated_cmp_s| {
        let mut sifr_generated_cmp_chars = sifr_generated_cmp_s.chars();
        let sifr_generated_cmp_first = sifr_generated_cmp_chars.next();
        if sifr_generated_cmp_chars.next().is_some() {
            None
        } else {
            sifr_generated_cmp_first
        }
    }) || {
        let sifr_generated_string_index = SifrInt::from_i64(10);
        let sifr_generated_string_index_normalized =
            sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_value.len());
        sifr_generated_chars_value
            .get(sifr_generated_string_index_normalized)
            .copied()
    } != Some("T").and_then(|sifr_generated_cmp_s| {
        let mut sifr_generated_cmp_chars = sifr_generated_cmp_s.chars();
        let sifr_generated_cmp_first = sifr_generated_cmp_chars.next();
        if sifr_generated_cmp_chars.next().is_some() {
            None
        } else {
            sifr_generated_cmp_first
        }
    }) || {
        let sifr_generated_string_index = SifrInt::from_i64(13);
        let sifr_generated_string_index_normalized =
            sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_value.len());
        sifr_generated_chars_value
            .get(sifr_generated_string_index_normalized)
            .copied()
    } != Some(":").and_then(|sifr_generated_cmp_s| {
        let mut sifr_generated_cmp_chars = sifr_generated_cmp_s.chars();
        let sifr_generated_cmp_first = sifr_generated_cmp_chars.next();
        if sifr_generated_cmp_chars.next().is_some() {
            None
        } else {
            sifr_generated_cmp_first
        }
    }) || {
        let sifr_generated_string_index = SifrInt::from_i64(16);
        let sifr_generated_string_index_normalized =
            sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_value.len());
        sifr_generated_chars_value
            .get(sifr_generated_string_index_normalized)
            .copied()
    } != Some(":").and_then(|sifr_generated_cmp_s| {
        let mut sifr_generated_cmp_chars = sifr_generated_cmp_s.chars();
        let sifr_generated_cmp_first = sifr_generated_cmp_chars.next();
        if sifr_generated_cmp_chars.next().is_some() {
            None
        } else {
            sifr_generated_cmp_first
        }
    }) {
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
fn sifr_generated_timezone_offset_from_text(text: &str) -> Result<SifrInt, ValueError> {
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
    } != Some(":").and_then(|sifr_generated_cmp_s| {
        let mut sifr_generated_cmp_chars = sifr_generated_cmp_s.chars();
        let sifr_generated_cmp_first = sifr_generated_cmp_chars.next();
        if sifr_generated_cmp_chars.next().is_some() {
            None
        } else {
            sifr_generated_cmp_first
        }
    }) {
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
fn sifr_generated_from_timestamp_with_tz(
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
                            "exact integer is outside the finite float range".to_string(),
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
                            "exact integer is outside the finite float range".to_string(),
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
fn now(
    tz: &Option<SifrGeneratedStdlibSifrX2edatetimeX2etimezone>,
) -> SifrGeneratedStdlibSifrX2edatetimeX2edatetime {
    let current_epoch: f64 = time_now();
    let sifr_generated_try_res: Result<SifrGeneratedStdlibSifrX2edatetimeX2edatetime, ValueError> =
        (|| {
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
            for (i, v) in Box::new(
                parts
                    .iter()
                    .cloned()
                    .enumerate()
                    .map(|sifr_generated_pair| {
                        (
                            SifrInt::from(sifr_generated_pair.0) + SifrInt::from_i64(0),
                            sifr_generated_pair.1,
                        )
                    }),
            ) {
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
fn main() {
    let zero: SifrGeneratedStdlibSifrX2edatetimeX2etimezone =
        SifrGeneratedStdlibSifrX2edatetimeX2etimezone::new(SifrInt::from_i64(0));
    assert_eq!(zero.to_string(), "UTC");
    let plus_two_thirty: SifrGeneratedStdlibSifrX2edatetimeX2etimezone =
        SifrGeneratedStdlibSifrX2edatetimeX2etimezone::new(SifrInt::from_i64(9000));
    let minus_five: SifrGeneratedStdlibSifrX2edatetimeX2etimezone =
        SifrGeneratedStdlibSifrX2edatetimeX2etimezone::new(-&SifrInt::from_i64(18000));
    assert_eq!(plus_two_thirty.to_string(), "UTC+02:30");
    assert_eq!(minus_five.to_string(), "UTC-05:00");
    let current: SifrGeneratedStdlibSifrX2edatetimeX2edatetime = now(&None);
    assert!(&current.year.clone() >= &SifrInt::from_i64(1970));
    assert!(
        &current.month.clone() >= &SifrInt::from_i64(1)
            && &current.month.clone() <= &SifrInt::from_i64(12)
    );
    assert!(
        &current.day.clone() >= &SifrInt::from_i64(1)
            && &current.day.clone() <= &SifrInt::from_i64(31)
    );
    assert_eq!(
        &SifrInt::from(current.isoformat().chars().count()),
        &SifrInt::from_i64(19)
    );
}
