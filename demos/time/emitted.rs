// src/main.rs
mod sifr_generated_generated_support {
    use crate::{
        FloatOverflowError, FloatPrecisionLossError, SifrGeneratedStdlibSifrX2etimeX2estructTime,
        SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0,
        ValueError,
    };
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
        assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &SifrInt::from(actual.len()) {
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
            i = &i + &SifrInt::from_i64(1);
        }
    }
    pub(crate) fn time_now() -> f64 {
        ::sifr_stdlib::time::time_now()
    }
    pub(crate) fn time_format(epoch: f64, fmt: &str) -> String {
        ::sifr_stdlib::time::time_format(epoch, fmt)
    }
    pub(crate) fn perf_counter() -> f64 {
        ::sifr_stdlib::time::perf_counter()
    }
    pub(crate) fn sleep(seconds: f64) {
        ::sifr_stdlib::time::sleep(seconds);
    }
    pub(crate) fn monotonic() -> f64 {
        ::sifr_stdlib::time::monotonic()
    }
    pub(crate) fn strptime(s: &str, fmt: &str) -> Result<String, ValueError> {
        ::sifr_stdlib::time::strptime(s, fmt).map_err(|sifr_generated_bridge_error| ValueError {
            message: sifr_generated_bridge_error.to_string(),
        })
    }
    pub(crate) fn sifr_generated_gmtime_intrinsic(epoch: f64) -> String {
        ::sifr_stdlib::time::gmtime(epoch)
    }
    pub(crate) fn sifr_generated_localtime_intrinsic(epoch: f64) -> String {
        ::sifr_stdlib::time::localtime(epoch)
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub(crate) enum SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(FloatOverflowError),
        SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
            FloatPrecisionLossError,
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
    pub(crate) fn sifr_generated_is_leap_year(year: SifrInt) -> bool {
        &year.floor_mod_known_nonzero(&SifrInt::from_i64(4)) == &SifrInt::from_i64(0)
            && &year.floor_mod_known_nonzero(&SifrInt::from_i64(100)) != &SifrInt::from_i64(0)
            || &year.floor_mod_known_nonzero(&SifrInt::from_i64(400)) == &SifrInt::from_i64(0)
    }
    pub(crate) fn sifr_generated_days_in_year(year: SifrInt) -> SifrInt {
        if sifr_generated_is_leap_year(year.clone()) {
            return SifrInt::from_i64(366);
        }
        SifrInt::from_i64(365)
    }
    pub(crate) fn sifr_generated_days_in_month(year: SifrInt, month: SifrInt) -> SifrInt {
        let month_days: Vec<SifrInt> = vec![
            SifrInt::from_i64(31),
            SifrInt::from_i64(28),
            SifrInt::from_i64(31),
            SifrInt::from_i64(30),
            SifrInt::from_i64(31),
            SifrInt::from_i64(30),
            SifrInt::from_i64(31),
            SifrInt::from_i64(31),
            SifrInt::from_i64(30),
            SifrInt::from_i64(31),
            SifrInt::from_i64(30),
            SifrInt::from_i64(31),
        ];
        let idx: SifrInt = &month - &SifrInt::from_i64(1);
        let d: Option<SifrInt> = {
            let sifr_generated_checked_read_collection = &month_days;
            let sifr_generated_checked_read_index = idx.clone();
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        };
        if &month == &SifrInt::from_i64(2) && sifr_generated_is_leap_year(year.clone()) {
            return SifrInt::from_i64(29);
        }
        let Some(d) = d.clone() else {
            return SifrInt::from_i64(0);
        };
        d
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
    pub(crate) fn sifr_generated_digit_value(ch: &str) -> Option<SifrInt> {
        if ch == "0" {
            return Some(SifrInt::from_i64(0));
        }
        if ch == "1" {
            return Some(SifrInt::from_i64(1));
        }
        if ch == "2" {
            return Some(SifrInt::from_i64(2));
        }
        if ch == "3" {
            return Some(SifrInt::from_i64(3));
        }
        if ch == "4" {
            return Some(SifrInt::from_i64(4));
        }
        if ch == "5" {
            return Some(SifrInt::from_i64(5));
        }
        if ch == "6" {
            return Some(SifrInt::from_i64(6));
        }
        if ch == "7" {
            return Some(SifrInt::from_i64(7));
        }
        if ch == "8" {
            return Some(SifrInt::from_i64(8));
        }
        if ch == "9" {
            return Some(SifrInt::from_i64(9));
        }
        None
    }
    pub(crate) fn sifr_generated_parse_decimal(text: &str) -> Option<SifrInt> {
        let sifr_generated_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
        if &SifrInt::from(sifr_generated_chars_text.len()) == &SifrInt::from_i64(0) {
            return None;
        }
        let mut out: SifrInt = SifrInt::from_i64(0);
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
            let ch_opt_value_58c5362056f71db8 = ch_opt?;
            let ch: String = ch_opt_value_58c5362056f71db8;
            let digit_opt: Option<SifrInt> = sifr_generated_digit_value(&ch);
            let digit_opt_value_c39685cb2782ed00 = digit_opt.clone()?;
            let digit: SifrInt = digit_opt_value_c39685cb2782ed00.clone();
            out = &(&out * &SifrInt::from_i64(10)) + &digit;
            i = &i + &SifrInt::from_i64(1);
        }
        Some(out)
    }
    pub(crate) fn sifr_generated_int_or_negative_one(value: Option<SifrInt>) -> SifrInt {
        let Some(value) = value.clone() else {
            return -&SifrInt::from_i64(1);
        };
        value.clone()
    }
    pub(crate) fn sifr_generated_day_of_year(
        year: SifrInt,
        month: SifrInt,
        day: SifrInt,
    ) -> SifrInt {
        let mut yday: SifrInt = SifrInt::from_i64(0);
        let mut m: SifrInt = SifrInt::from_i64(1);
        while &m < &month {
            yday = &yday + &sifr_generated_days_in_month(year.clone(), m.clone());
            m = &m + &SifrInt::from_i64(1);
        }
        &yday + &day
    }
    pub(crate) fn sifr_generated_weekday(year: SifrInt, month: SifrInt, day: SifrInt) -> SifrInt {
        let mut days_since_epoch: SifrInt = SifrInt::from_i64(0);
        if &year >= &SifrInt::from_i64(1970) {
            let mut y: SifrInt = SifrInt::from_i64(1970);
            while &y < &year {
                days_since_epoch = &days_since_epoch + &sifr_generated_days_in_year(y.clone());
                y = &y + &SifrInt::from_i64(1);
            }
        } else {
            let mut y: SifrInt = SifrInt::from_i64(1969);
            while &y >= &year {
                days_since_epoch = &days_since_epoch - &sifr_generated_days_in_year(y.clone());
                y = &y - &SifrInt::from_i64(1);
            }
        }
        let mut m: SifrInt = SifrInt::from_i64(1);
        while &m < &month {
            days_since_epoch =
                &days_since_epoch + &sifr_generated_days_in_month(year.clone(), m.clone());
            m = &m + &SifrInt::from_i64(1);
        }
        days_since_epoch = &(&days_since_epoch + &day) - &SifrInt::from_i64(1);
        let mut wd: SifrInt = (&SifrInt::from_i64(3) + &days_since_epoch)
            .floor_mod_known_nonzero(&SifrInt::from_i64(7));
        if &wd < &SifrInt::from_i64(0) {
            wd = &wd + &SifrInt::from_i64(7);
        }
        wd.clone()
    }
    pub(crate) fn sifr_generated_valid_date(year: SifrInt, month: SifrInt, day: SifrInt) -> bool {
        if &year <= &SifrInt::from_i64(0) {
            return false;
        }
        if &month < &SifrInt::from_i64(1) || &month > &SifrInt::from_i64(12) {
            return false;
        }
        let max_day: SifrInt = sifr_generated_days_in_month(year.clone(), month.clone());
        &day >= &SifrInt::from_i64(1) && &day <= &max_day
    }
    pub(crate) fn sifr_generated_invalid_struct_time() -> SifrGeneratedStdlibSifrX2etimeX2estructTime
    {
        SifrGeneratedStdlibSifrX2etimeX2estructTime::new(
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
            SifrInt::from_i64(0),
        )
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub(crate) fn sifr_generated_to_struct_time(
        rendered: &str,
    ) -> SifrGeneratedStdlibSifrX2etimeX2estructTime {
        let sifr_generated_chars_rendered: Vec<char> = rendered.chars().collect::<Vec<char>>();
        let Some(_checked_value_3) = {
            let sifr_generated_string_index = SifrInt::from_i64(4);
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_rendered.len());
            sifr_generated_chars_rendered
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string()) else {
            return sifr_generated_invalid_struct_time();
        };
        let Some(_checked_value_4) = {
            let sifr_generated_string_index = SifrInt::from_i64(7);
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_rendered.len());
            sifr_generated_chars_rendered
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string()) else {
            return sifr_generated_invalid_struct_time();
        };
        let Some(_checked_value_5) = {
            let sifr_generated_string_index = SifrInt::from_i64(10);
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_rendered.len());
            sifr_generated_chars_rendered
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string()) else {
            return sifr_generated_invalid_struct_time();
        };
        let Some(_checked_value_6) = {
            let sifr_generated_string_index = SifrInt::from_i64(13);
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_rendered.len());
            sifr_generated_chars_rendered
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string()) else {
            return sifr_generated_invalid_struct_time();
        };
        let Some(_checked_value_7) = {
            let sifr_generated_string_index = SifrInt::from_i64(16);
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_rendered.len());
            sifr_generated_chars_rendered
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string()) else {
            return sifr_generated_invalid_struct_time();
        };
        if {
            let sifr_generated_string_index = SifrInt::from_i64(4);
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_chars_rendered.len());
            sifr_generated_chars_rendered
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(Some)
            != Some(Some('-'))
            || {
                let sifr_generated_string_index = SifrInt::from_i64(7);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_rendered.len());
                sifr_generated_chars_rendered
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(Some)
                != Some(Some('-'))
            || {
                let sifr_generated_string_index = SifrInt::from_i64(10);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_rendered.len());
                sifr_generated_chars_rendered
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(Some)
                != Some(Some('T'))
            || {
                let sifr_generated_string_index = SifrInt::from_i64(13);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_rendered.len());
                sifr_generated_chars_rendered
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(Some)
                != Some(Some(':'))
            || {
                let sifr_generated_string_index = SifrInt::from_i64(16);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_rendered.len());
                sifr_generated_chars_rendered
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(Some)
                != Some(Some(':'))
        {
            return sifr_generated_invalid_struct_time();
        }
        let year: SifrInt = sifr_generated_int_or_negative_one(sifr_generated_parse_decimal(
            &sifr_generated_substring(rendered, SifrInt::from_i64(0), SifrInt::from_i64(4)),
        ));
        let month: SifrInt = sifr_generated_int_or_negative_one(sifr_generated_parse_decimal(
            &sifr_generated_substring(rendered, SifrInt::from_i64(5), SifrInt::from_i64(7)),
        ));
        let day: SifrInt = sifr_generated_int_or_negative_one(sifr_generated_parse_decimal(
            &sifr_generated_substring(rendered, SifrInt::from_i64(8), SifrInt::from_i64(10)),
        ));
        let hour: SifrInt = sifr_generated_int_or_negative_one(sifr_generated_parse_decimal(
            &sifr_generated_substring(rendered, SifrInt::from_i64(11), SifrInt::from_i64(13)),
        ));
        let minute: SifrInt = sifr_generated_int_or_negative_one(sifr_generated_parse_decimal(
            &sifr_generated_substring(rendered, SifrInt::from_i64(14), SifrInt::from_i64(16)),
        ));
        let second: SifrInt = sifr_generated_int_or_negative_one(sifr_generated_parse_decimal(
            &sifr_generated_substring(rendered, SifrInt::from_i64(17), SifrInt::from_i64(19)),
        ));
        if &year < &SifrInt::from_i64(0)
            || &month < &SifrInt::from_i64(0)
            || &day < &SifrInt::from_i64(0)
            || &hour < &SifrInt::from_i64(0)
            || &minute < &SifrInt::from_i64(0)
            || &second < &SifrInt::from_i64(0)
        {
            return sifr_generated_invalid_struct_time();
        }
        if !sifr_generated_valid_date(year.clone(), month.clone(), day.clone()) {
            return sifr_generated_invalid_struct_time();
        }
        let wday: SifrInt = sifr_generated_weekday(year.clone(), month.clone(), day.clone());
        let yday_value_75753d4973d2a3ce: SifrInt =
            sifr_generated_day_of_year(year.clone(), month.clone(), day.clone());
        SifrGeneratedStdlibSifrX2etimeX2estructTime::new(
            year.clone(),
            month.clone(),
            day.clone(),
            hour.clone(),
            minute.clone(),
            second.clone(),
            wday.clone(),
            yday_value_75753d4973d2a3ce.clone(),
            SifrInt::from_i64(0),
        )
    }
    pub(crate) fn time() -> f64 {
        time_now()
    }
    pub(crate) fn strftime(fmt: &str, epoch: f64) -> String {
        time_format(epoch, fmt)
    }
    pub(crate) fn gmtime_struct(epoch: f64) -> SifrGeneratedStdlibSifrX2etimeX2estructTime {
        let rendered: String = sifr_generated_gmtime_intrinsic(epoch);
        sifr_generated_to_struct_time(&rendered)
    }
    pub(crate) fn localtime_struct(epoch: f64) -> SifrGeneratedStdlibSifrX2etimeX2estructTime {
        let rendered: String = sifr_generated_localtime_intrinsic(epoch);
        sifr_generated_to_struct_time(&rendered)
    }
    pub(crate) fn mktime(
        t: &SifrGeneratedStdlibSifrX2etimeX2estructTime,
    ) -> Result<
        f64,
        SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0,
    >{
        if !sifr_generated_valid_date(t.tm_year.clone(), t.tm_mon.clone(), t.tm_mday.clone()) {
            return Err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
                    ValueError::new(
                        "mktime() received an invalid calendar date".to_string(),
                    ),
                ),
            );
        }
        let mut days: SifrInt = SifrInt::from_i64(0);
        if &t.tm_year.clone() >= &SifrInt::from_i64(1970) {
            let mut y: SifrInt = SifrInt::from_i64(1970);
            while &y < &t.tm_year.clone() {
                days = &days + &sifr_generated_days_in_year(y.clone());
                y = &y + &SifrInt::from_i64(1);
            }
        } else {
            let mut y: SifrInt = SifrInt::from_i64(1969);
            while &y >= &t.tm_year.clone() {
                days = &days - &sifr_generated_days_in_year(y.clone());
                y = &y - &SifrInt::from_i64(1);
            }
        }
        let mut m: SifrInt = SifrInt::from_i64(1);
        while &m < &t.tm_mon.clone() {
            days = &days + &sifr_generated_days_in_month(t.tm_year.clone(), m.clone());
            m = &m + &SifrInt::from_i64(1);
        }
        days = &(&days + &t.tm_mday.clone()) - &SifrInt::from_i64(1);
        let stamp: SifrInt = &(&(&(&days * &SifrInt::from_i64(86400))
            + &(&t.tm_hour.clone() * &SifrInt::from_i64(3600)))
            + &(&t.tm_min.clone() * &SifrInt::from_i64(60)))
            + &t.tm_sec.clone();
        stamp
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
            .map_err(|sifr_generated_error_value| match sifr_generated_error_value {
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
            })
    }
}
mod sifr_generated_project_nominals {
    use ::sifr_runtime::SifrInt;
    #[derive(Debug, Clone)]
    pub struct SifrGeneratedStdlibSifrX2etimeX2estructTime {
        pub tm_year: SifrInt,
        pub tm_mon: SifrInt,
        pub tm_mday: SifrInt,
        pub tm_hour: SifrInt,
        pub tm_min: SifrInt,
        pub tm_sec: SifrInt,
        pub tm_wday: SifrInt,
        pub tm_yday: SifrInt,
        pub tm_isdst: SifrInt,
    }
    impl SifrGeneratedStdlibSifrX2etimeX2estructTime {
        #[must_use]
        #[expect(
            clippy::too_many_arguments,
            reason = "generated signature preserves the typed Sifr callable contract"
        )]
        pub fn new(
            tm_year: SifrInt,
            tm_mon: SifrInt,
            tm_mday_argument_a505494cd43c9214: SifrInt,
            tm_hour: SifrInt,
            tm_min_argument_103d514d457d4a49: SifrInt,
            tm_sec: SifrInt,
            tm_wday_argument_d5143a059ed34c12: SifrInt,
            tm_yday_argument_6b9a41f3b9220250: SifrInt,
            tm_isdst: SifrInt,
        ) -> Self {
            let sifr_generated_field_value_72897bf3bc91df5a_746d5f79656172: SifrInt =
                tm_year.clone();
            let sifr_generated_field_value_1029314d456c6adf_746d5f6d6f6e: SifrInt = tm_mon.clone();
            let sifr_generated_field_value_a505494cd43c9214_746d5f6d646179: SifrInt =
                tm_mday_argument_a505494cd43c9214.clone();
            let sifr_generated_field_value_129c5b76af381059_746d5f686f7572: SifrInt =
                tm_hour.clone();
            let sifr_generated_field_value_103d514d457d4a49_746d5f6d696e: SifrInt =
                tm_min_argument_103d514d457d4a49.clone();
            let sifr_generated_field_value_f3d84e4dc71632a0_746d5f736563: SifrInt = tm_sec.clone();
            let sifr_generated_field_value_d5143a059ed34c12_746d5f77646179: SifrInt =
                tm_wday_argument_d5143a059ed34c12.clone();
            let sifr_generated_field_value_6b9a41f3b9220250_746d5f79646179: SifrInt =
                tm_yday_argument_6b9a41f3b9220250.clone();
            let sifr_generated_field_value_d0ec16f562c1ee92_746d5f6973647374: SifrInt =
                tm_isdst.clone();
            Self {
                tm_year: sifr_generated_field_value_72897bf3bc91df5a_746d5f79656172,
                tm_mon: sifr_generated_field_value_1029314d456c6adf_746d5f6d6f6e,
                tm_mday: sifr_generated_field_value_a505494cd43c9214_746d5f6d646179,
                tm_hour: sifr_generated_field_value_129c5b76af381059_746d5f686f7572,
                tm_min: sifr_generated_field_value_103d514d457d4a49_746d5f6d696e,
                tm_sec: sifr_generated_field_value_f3d84e4dc71632a0_746d5f736563,
                tm_wday: sifr_generated_field_value_d5143a059ed34c12_746d5f77646179,
                tm_yday: sifr_generated_field_value_6b9a41f3b9220250_746d5f79646179,
                tm_isdst: sifr_generated_field_value_d0ec16f562c1ee92_746d5f6973647374,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2etimeX2estructTime {
        #[must_use]
        pub fn as_tuple(
            &self,
        ) -> (
            SifrInt,
            SifrInt,
            SifrInt,
            SifrInt,
            SifrInt,
            SifrInt,
            SifrInt,
            SifrInt,
            SifrInt,
        ) {
            (
                self.tm_year.clone(),
                self.tm_mon.clone(),
                self.tm_mday.clone(),
                self.tm_hour.clone(),
                self.tm_min.clone(),
                self.tm_sec.clone(),
                self.tm_wday.clone(),
                self.tm_yday.clone(),
                self.tm_isdst.clone(),
            )
        }
    }
    impl SifrGeneratedStdlibSifrX2etimeX2estructTime {
        #[must_use]
        pub fn isoformat(&self) -> String {
            let y: String = self.tm_year.clone().to_string();
            let mut mo: String = self.tm_mon.clone().to_string();
            if &SifrInt::from(mo.chars().count()) < &SifrInt::from_i64(2) {
                mo = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize + mo.len());
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(mo.as_str());
                    sifr_generated_concat
                };
            }
            let mut d: String = self.tm_mday.clone().to_string();
            if &SifrInt::from(d.chars().count()) < &SifrInt::from_i64(2) {
                d = {
                    let mut sifr_generated_concat: String = String::with_capacity(1usize + d.len());
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(d.as_str());
                    sifr_generated_concat
                };
            }
            let mut h: String = self.tm_hour.clone().to_string();
            if &SifrInt::from(h.chars().count()) < &SifrInt::from_i64(2) {
                h = {
                    let mut sifr_generated_concat: String = String::with_capacity(1usize + h.len());
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(h.as_str());
                    sifr_generated_concat
                };
            }
            let mut mi: String = self.tm_min.clone().to_string();
            if &SifrInt::from(mi.chars().count()) < &SifrInt::from_i64(2) {
                mi = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize + mi.len());
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(mi.as_str());
                    sifr_generated_concat
                };
            }
            let mut s: String = self.tm_sec.clone().to_string();
            if &SifrInt::from(s.chars().count()) < &SifrInt::from_i64(2) {
                s = {
                    let mut sifr_generated_concat: String = String::with_capacity(1usize + s.len());
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(s.as_str());
                    sifr_generated_concat
                };
            }
            {
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
            }
        }
    }
    impl PartialEq for SifrGeneratedStdlibSifrX2etimeX2estructTime {
        fn eq(&self, other: &SifrGeneratedStdlibSifrX2etimeX2estructTime) -> bool {
            self.as_tuple() == other.as_tuple()
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2etimeX2estructTime {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.isoformat())
        }
    }
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
    impl From<ValueError> for Error {
        fn from(err: ValueError) -> Self {
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
}
pub use sifr_generated_project_nominals::Error;
pub use sifr_generated_project_nominals::FloatOverflowError;
pub use sifr_generated_project_nominals::FloatPrecisionLossError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2etimeX2estructTime;
pub use sifr_generated_project_nominals::ValueError;
mod sifr_generated_project_unions {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
            crate::sifr_generated_project_nominals::FloatOverflowError,
        ),
        SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
            crate::sifr_generated_project_nominals::FloatPrecisionLossError,
        ),
        SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
            crate::sifr_generated_project_nominals::ValueError,
        ),
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
    #[derive(Debug, Clone)]
    pub enum SifrGeneratedUnion8X3asequence5X3aunion1X3a423X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0
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
        SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
            crate::sifr_generated_project_nominals::ValueError,
        ),
    }
    impl From<crate::sifr_generated_project_nominals::Error>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a423X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0 {
        fn from(value: crate::sifr_generated_project_nominals::Error) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a423X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass5X3aError1X3a0(
                value,
            )
        }
    }
    impl From<crate::sifr_generated_project_nominals::FloatOverflowError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a423X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0 {
        fn from(
            value: crate::sifr_generated_project_nominals::FloatOverflowError,
        ) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a423X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                value,
            )
        }
    }
    impl From<crate::sifr_generated_project_nominals::FloatPrecisionLossError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a423X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0 {
        fn from(
            value: crate::sifr_generated_project_nominals::FloatPrecisionLossError,
        ) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a423X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                value,
            )
        }
    }
    impl From<crate::sifr_generated_project_nominals::ValueError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a423X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0 {
        fn from(value: crate::sifr_generated_project_nominals::ValueError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a423X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a423X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a423X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass5X3aError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a423X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a423X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a423X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
}
use crate::sifr_generated_generated_support::*;
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_unions::SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0;
pub use sifr_generated_project_unions::SifrGeneratedUnion8X3asequence5X3aunion1X3a423X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0;
fn collect_clock_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![time() > 0.0_f64];
    let perf_before: f64 = perf_counter();
    let mono_before: f64 = monotonic();
    sleep(0.01_f64);
    let perf_after: f64 = perf_counter();
    let mono_after: f64 = monotonic();
    actual.push(perf_after >= perf_before && mono_after >= mono_before);
    actual
}
fn collect_format_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![
        strftime(&"%Y-%m-%d %H:%M:%S".to_string(), 0.0_f64).as_str()
            == "1970-01-01 00:00:00".to_string().as_str(),
    ];
    let gmt: SifrGeneratedStdlibSifrX2etimeX2estructTime = gmtime_struct(0.0_f64);
    actual.push(
        &gmt.tm_year.clone() == &SifrInt::from_i64(1970)
            && &gmt.tm_mon.clone() == &SifrInt::from_i64(1)
            && &gmt.tm_mday.clone() == &SifrInt::from_i64(1)
            && &gmt.tm_hour.clone() == &SifrInt::from_i64(0)
            && &gmt.tm_min.clone() == &SifrInt::from_i64(0)
            && &gmt.tm_sec.clone() == &SifrInt::from_i64(0),
    );
    let local: SifrGeneratedStdlibSifrX2etimeX2estructTime = localtime_struct(0.0_f64);
    actual.push(
        &local.tm_year.clone() > &SifrInt::from_i64(0)
            && &local.tm_yday.clone() >= &SifrInt::from_i64(1),
    );
    actual
}
fn collect_parse_and_safety_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let mut parsed_ok: bool = false;
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let parsed: String = strptime(
            &"2024-01-15 10:30:00".to_string(),
            &"%Y-%m-%d %H:%M:%S".to_string(),
        )?;
        parsed_ok = parsed == "2024-01-15T10:30:00";
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
        parsed_ok = false;
    }
    actual.push(parsed_ok);
    let mut parse_error_ok: bool = false;
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let _bad: String = strptime(&"bad".to_string(), &"%Y-%m-%d %H:%M:%S".to_string())?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
        parse_error_ok = true;
    }
    actual.push(parse_error_ok);
    sleep(-0.05_f64);
    actual.push(true);
    let epoch_tm: SifrGeneratedStdlibSifrX2etimeX2estructTime = gmtime_struct(0.0_f64);
    let sifr_generated_try_res: Result<
        (),
        SifrGeneratedUnion8X3asequence5X3aunion1X3a423X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0,
    > = (|| {
        let epoch_stamp: f64 = mktime(&epoch_tm)
            .map_err(|sifr_generated_e| match sifr_generated_e {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                    sifr_generated_union_value,
                ) => {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a423X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                        sifr_generated_union_value,
                    )
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                    sifr_generated_union_value,
                ) => {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a423X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                        sifr_generated_union_value,
                    )
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a323X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
                    sifr_generated_union_value,
                ) => {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a423X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
                        sifr_generated_union_value,
                    )
                }
            })?;
        actual.push(epoch_stamp == 0.0_f64);
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        match sifr_generated_try_err {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a423X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass5X3aError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let _e_5f65 = sifr_generated_try_variant_error.clone();
                actual.push(false);
            }
            SifrGeneratedUnion8X3asequence5X3aunion1X3a423X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let _e_5f65 = Error::new(
                    sifr_generated_try_variant_error.clone().message,
                );
                actual.push(false);
            }
            SifrGeneratedUnion8X3asequence5X3aunion1X3a423X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let _e_5f65 = Error::new(
                    sifr_generated_try_variant_error.clone().message,
                );
                actual.push(false);
            }
            SifrGeneratedUnion8X3asequence5X3aunion1X3a423X3a5X3aclass10X3aValueError1X3a031X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a017X3a5X3aclass5X3aError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let _e_5f65 = Error::new(
                    sifr_generated_try_variant_error.clone().message,
                );
                actual.push(false);
            }
        }
    }
    actual
}
fn append_all(target: &mut Vec<bool>, values: &[bool]) {
    for value in values.iter().copied() {
        target.push(value);
    }
}
fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true, true, true, true];
    let mut actual: Vec<bool> = Vec::new();
    append_all(&mut actual, &collect_clock_actual());
    append_all(&mut actual, &collect_format_actual());
    append_all(&mut actual, &collect_parse_and_safety_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("time time parity demo: pass");
}
