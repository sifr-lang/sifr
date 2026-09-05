// src/main.rs
pub mod sifr_generated_generated_support {
    use crate::{
        FloatOverflowError, FloatPrecisionLossError, SifrGeneratedStdlibSifrX2etimeX2estructTime,
        SifrGeneratedUnion8X3asequence5X3aunion1X3a336X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a044X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0,
        ValueError,
    };
    pub(super) use ::sifr_runtime::SifrInt;
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
    pub(super) fn time_now() -> f64 {
        ::sifr_stdlib::time::time_now()
    }
    pub(super) fn time_format(epoch: f64, fmt: &str) -> String {
        ::sifr_stdlib::time::time_format(epoch, fmt)
    }
    pub(super) fn perf_counter() -> f64 {
        ::sifr_stdlib::time::perf_counter()
    }
    pub(super) fn sleep(seconds: f64) {
        ::sifr_stdlib::time::sleep(seconds);
    }
    pub(super) fn monotonic() -> f64 {
        ::sifr_stdlib::time::monotonic()
    }
    pub(super) fn strptime(s: &str, fmt: &str) -> Result<String, ValueError> {
        ::sifr_stdlib::time::strptime(s, fmt).map_err(|sifr_generated_bridge_error| ValueError {
            message: sifr_generated_bridge_error,
        })
    }
    pub(super) fn sifr_generated_gmtime_intrinsic(epoch: f64) -> String {
        ::sifr_stdlib::time::gmtime(epoch)
    }
    pub(super) fn sifr_generated_localtime_intrinsic(epoch: f64) -> String {
        ::sifr_stdlib::time::localtime(epoch)
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
    pub(super) fn sifr_generated_is_leap_year(year: &SifrInt) -> bool {
        year.floor_mod_known_nonzero(&SifrInt::from_i64(4)) == SifrInt::from_i64(0)
            && year.floor_mod_known_nonzero(&SifrInt::from_i64(100)) != SifrInt::from_i64(0)
            || year.floor_mod_known_nonzero(&SifrInt::from_i64(400)) == SifrInt::from_i64(0)
    }
    pub(super) fn sifr_generated_days_in_year(year: &SifrInt) -> SifrInt {
        if sifr_generated_is_leap_year(year) {
            return SifrInt::from_i64(366);
        }
        SifrInt::from_i64(365)
    }
    pub(super) fn sifr_generated_days_in_month(year: &SifrInt, month: &SifrInt) -> SifrInt {
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
        let idx: SifrInt = ::std::ops::Sub::sub(month, &SifrInt::from_i64(1));
        let d: Option<SifrInt> = {
            let sifr_generated_checked_read_collection = &month_days;
            let sifr_generated_checked_read_index = &idx;
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        };
        if month == &SifrInt::from_i64(2) && sifr_generated_is_leap_year(year) {
            return SifrInt::from_i64(29);
        }
        let Some(d) = d else {
            return SifrInt::from_i64(0);
        };
        d
    }
    pub(super) fn sifr_generated_substring(value: &str, start: &SifrInt, end: &SifrInt) -> String {
        let sifr_generated_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
        let mut result: String = String::new();
        let mut i: SifrInt = (*start).clone();
        while &i < end {
            let ch: Option<String> = {
                let sifr_generated_string_index = &i;
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
    pub(super) fn sifr_generated_digit_value(ch: &str) -> Option<SifrInt> {
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
    pub(super) fn sifr_generated_parse_decimal(text: &str) -> Option<SifrInt> {
        let sifr_generated_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
        if sifr_generated_chars_text.len() == SifrInt::from_i64(0) {
            return None;
        }
        let mut out: SifrInt = SifrInt::from_i64(0);
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < sifr_generated_chars_text.len() {
            let ch_opt: Option<String> = {
                let sifr_generated_string_index = &i;
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
            let digit_opt_value_c39685cb2782ed00 = digit_opt?;
            let digit: SifrInt = digit_opt_value_c39685cb2782ed00;
            out = ::std::ops::Add::add(&::std::ops::Mul::mul(&out, &SifrInt::from_i64(10)), &digit);
            i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
        }
        Some(out)
    }
    pub(super) fn sifr_generated_int_or_negative_one(value: Option<&SifrInt>) -> SifrInt {
        let value: Option<SifrInt> = value.cloned();
        let Some(value_value_7ce4fd9430e80cea) = value else {
            return ::std::ops::Neg::neg(&SifrInt::from_i64(1));
        };
        value_value_7ce4fd9430e80cea
    }
    pub(super) fn sifr_generated_day_of_year(
        year: &SifrInt,
        month: &SifrInt,
        day: &SifrInt,
    ) -> SifrInt {
        let mut yday: SifrInt = SifrInt::from_i64(0);
        let mut m: SifrInt = SifrInt::from_i64(1);
        while &m < month {
            yday = ::std::ops::Add::add(&yday, &sifr_generated_days_in_month(year, &m));
            m = ::std::ops::Add::add(&m, &SifrInt::from_i64(1));
        }
        ::std::ops::Add::add(&yday, day)
    }
    pub(super) fn sifr_generated_weekday(
        year: &SifrInt,
        month: &SifrInt,
        day: &SifrInt,
    ) -> SifrInt {
        let mut days_since_epoch: SifrInt = SifrInt::from_i64(0);
        if year >= &SifrInt::from_i64(1970) {
            let mut y: SifrInt = SifrInt::from_i64(1970);
            while &y < year {
                days_since_epoch =
                    ::std::ops::Add::add(&days_since_epoch, &sifr_generated_days_in_year(&y));
                y = ::std::ops::Add::add(&y, &SifrInt::from_i64(1));
            }
        } else {
            let mut y: SifrInt = SifrInt::from_i64(1969);
            while &y >= year {
                days_since_epoch =
                    ::std::ops::Sub::sub(&days_since_epoch, &sifr_generated_days_in_year(&y));
                y = ::std::ops::Sub::sub(&y, &SifrInt::from_i64(1));
            }
        }
        let mut m: SifrInt = SifrInt::from_i64(1);
        while &m < month {
            days_since_epoch =
                ::std::ops::Add::add(&days_since_epoch, &sifr_generated_days_in_month(year, &m));
            m = ::std::ops::Add::add(&m, &SifrInt::from_i64(1));
        }
        days_since_epoch = ::std::ops::Sub::sub(
            &::std::ops::Add::add(&days_since_epoch, day),
            &SifrInt::from_i64(1),
        );
        let mut wd: SifrInt = ::std::ops::Add::add(&SifrInt::from_i64(3), &days_since_epoch)
            .floor_mod_known_nonzero(&SifrInt::from_i64(7));
        if wd < SifrInt::from_i64(0) {
            wd = ::std::ops::Add::add(&wd, &SifrInt::from_i64(7));
        }
        wd
    }
    pub(super) fn sifr_generated_valid_date(
        year: &SifrInt,
        month: &SifrInt,
        day: &SifrInt,
    ) -> bool {
        if year <= &SifrInt::from_i64(0) {
            return false;
        }
        if month < &SifrInt::from_i64(1) || month > &SifrInt::from_i64(12) {
            return false;
        }
        let max_day: SifrInt = sifr_generated_days_in_month(year, month);
        day >= &SifrInt::from_i64(1) && day <= &max_day
    }
    pub(super) fn sifr_generated_invalid_struct_time() -> SifrGeneratedStdlibSifrX2etimeX2estructTime
    {
        SifrGeneratedStdlibSifrX2etimeX2estructTime::new(
            &SifrInt::from_i64(0),
            &SifrInt::from_i64(0),
            &SifrInt::from_i64(0),
            &SifrInt::from_i64(0),
            &SifrInt::from_i64(0),
            &SifrInt::from_i64(0),
            &SifrInt::from_i64(0),
            &SifrInt::from_i64(0),
            &SifrInt::from_i64(0),
        )
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub(super) fn sifr_generated_to_struct_time(
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
        let year: SifrInt = sifr_generated_int_or_negative_one(
            sifr_generated_parse_decimal(&sifr_generated_substring(
                rendered,
                &SifrInt::from_i64(0),
                &SifrInt::from_i64(4),
            ))
            .as_ref(),
        );
        let month: SifrInt = sifr_generated_int_or_negative_one(
            sifr_generated_parse_decimal(&sifr_generated_substring(
                rendered,
                &SifrInt::from_i64(5),
                &SifrInt::from_i64(7),
            ))
            .as_ref(),
        );
        let day: SifrInt = sifr_generated_int_or_negative_one(
            sifr_generated_parse_decimal(&sifr_generated_substring(
                rendered,
                &SifrInt::from_i64(8),
                &SifrInt::from_i64(10),
            ))
            .as_ref(),
        );
        let hour: SifrInt = sifr_generated_int_or_negative_one(
            sifr_generated_parse_decimal(&sifr_generated_substring(
                rendered,
                &SifrInt::from_i64(11),
                &SifrInt::from_i64(13),
            ))
            .as_ref(),
        );
        let minute: SifrInt = sifr_generated_int_or_negative_one(
            sifr_generated_parse_decimal(&sifr_generated_substring(
                rendered,
                &SifrInt::from_i64(14),
                &SifrInt::from_i64(16),
            ))
            .as_ref(),
        );
        let second: SifrInt = sifr_generated_int_or_negative_one(
            sifr_generated_parse_decimal(&sifr_generated_substring(
                rendered,
                &SifrInt::from_i64(17),
                &SifrInt::from_i64(19),
            ))
            .as_ref(),
        );
        if year < SifrInt::from_i64(0)
            || month < SifrInt::from_i64(0)
            || day < SifrInt::from_i64(0)
            || hour < SifrInt::from_i64(0)
            || minute < SifrInt::from_i64(0)
            || second < SifrInt::from_i64(0)
        {
            return sifr_generated_invalid_struct_time();
        }
        if !sifr_generated_valid_date(&year, &month, &day) {
            return sifr_generated_invalid_struct_time();
        }
        let wday: SifrInt = sifr_generated_weekday(&year, &month, &day);
        let yday_value_75753d4973d2a3ce: SifrInt = sifr_generated_day_of_year(&year, &month, &day);
        SifrGeneratedStdlibSifrX2etimeX2estructTime::new(
            &year,
            &month,
            &day,
            &hour,
            &minute,
            &second,
            &wday,
            &yday_value_75753d4973d2a3ce,
            &SifrInt::from_i64(0),
        )
    }
    pub(super) fn time() -> f64 {
        time_now()
    }
    pub(super) fn strftime(fmt: &str, epoch: f64) -> String {
        time_format(epoch, fmt)
    }
    pub(super) fn gmtime_struct(epoch: f64) -> SifrGeneratedStdlibSifrX2etimeX2estructTime {
        let rendered: String = sifr_generated_gmtime_intrinsic(epoch);
        sifr_generated_to_struct_time(&rendered)
    }
    pub(super) fn localtime_struct(epoch: f64) -> SifrGeneratedStdlibSifrX2etimeX2estructTime {
        let rendered: String = sifr_generated_localtime_intrinsic(epoch);
        sifr_generated_to_struct_time(&rendered)
    }
    pub(super) fn mktime(
        t: &SifrGeneratedStdlibSifrX2etimeX2estructTime,
    ) -> Result<
        f64,
        SifrGeneratedUnion8X3asequence5X3aunion1X3a336X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a044X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0,
    >{
        if !sifr_generated_valid_date(&t.tm_year, &t.tm_mon, &t.tm_mday) {
            return Err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a336X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a044X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0(
                    ValueError::new(
                        "mktime() received an invalid calendar date".to_string(),
                    ),
                ),
            );
        }
        let mut days: SifrInt = SifrInt::from_i64(0);
        if t.tm_year >= SifrInt::from_i64(1970) {
            let mut y: SifrInt = SifrInt::from_i64(1970);
            while y < t.tm_year {
                days = ::std::ops::Add::add(&days, &sifr_generated_days_in_year(&y));
                y = ::std::ops::Add::add(&y, &SifrInt::from_i64(1));
            }
        } else {
            let mut y: SifrInt = SifrInt::from_i64(1969);
            while y >= t.tm_year {
                days = ::std::ops::Sub::sub(&days, &sifr_generated_days_in_year(&y));
                y = ::std::ops::Sub::sub(&y, &SifrInt::from_i64(1));
            }
        }
        let mut m: SifrInt = SifrInt::from_i64(1);
        while m < t.tm_mon {
            days = ::std::ops::Add::add(&days, &sifr_generated_days_in_month(&t.tm_year, &m));
            m = ::std::ops::Add::add(&m, &SifrInt::from_i64(1));
        }
        days = ::std::ops::Sub::sub(
            &::std::ops::Add::add(&days, &t.tm_mday.clone()),
            &SifrInt::from_i64(1),
        );
        let stamp: SifrInt = ::std::ops::Add::add(
            &::std::ops::Add::add(
                &::std::ops::Add::add(
                    &::std::ops::Mul::mul(&days, &SifrInt::from_i64(86400)),
                    &::std::ops::Mul::mul(&t.tm_hour.clone(), &SifrInt::from_i64(3600)),
                ),
                &::std::ops::Mul::mul(&t.tm_min.clone(), &SifrInt::from_i64(60)),
            ),
            &t.tm_sec.clone(),
        );
        stamp
            .checked_to_f64()
            .map_err(|sifr_generated_float_error| match sifr_generated_float_error {
                ::sifr_runtime::IntegerFloatConversionError::Overflow => {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
                        FloatOverflowError::new(
                            "exact integer is outside the finite float range".to_string(),
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
            .map_err(|sifr_generated_error_value| match sifr_generated_error_value {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
                    sifr_generated_union_value,
                ) => {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a336X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a044X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
                        sifr_generated_union_value,
                    )
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
                    sifr_generated_union_value,
                ) => {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a336X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a044X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
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
            tm_year: &SifrInt,
            tm_mon: &SifrInt,
            tm_mday_argument_a505494cd43c9214: &SifrInt,
            tm_hour: &SifrInt,
            tm_min_argument_103d514d457d4a49: &SifrInt,
            tm_sec: &SifrInt,
            tm_wday_argument_d5143a059ed34c12: &SifrInt,
            tm_yday_argument_6b9a41f3b9220250: &SifrInt,
            tm_isdst: &SifrInt,
        ) -> Self {
            let sifr_generated_field_value_72897bf3bc91df5a_746d5f79656172: SifrInt =
                (*tm_year).clone();
            let sifr_generated_field_value_1029314d456c6adf_746d5f6d6f6e: SifrInt =
                (*tm_mon).clone();
            let sifr_generated_field_value_a505494cd43c9214_746d5f6d646179: SifrInt =
                (*tm_mday_argument_a505494cd43c9214).clone();
            let sifr_generated_field_value_129c5b76af381059_746d5f686f7572: SifrInt =
                (*tm_hour).clone();
            let sifr_generated_field_value_103d514d457d4a49_746d5f6d696e: SifrInt =
                (*tm_min_argument_103d514d457d4a49).clone();
            let sifr_generated_field_value_f3d84e4dc71632a0_746d5f736563: SifrInt =
                (*tm_sec).clone();
            let sifr_generated_field_value_d5143a059ed34c12_746d5f77646179: SifrInt =
                (*tm_wday_argument_d5143a059ed34c12).clone();
            let sifr_generated_field_value_6b9a41f3b9220250_746d5f79646179: SifrInt =
                (*tm_yday_argument_6b9a41f3b9220250).clone();
            let sifr_generated_field_value_d0ec16f562c1ee92_746d5f6973647374: SifrInt =
                (*tm_isdst).clone();
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
            let y: String = self.tm_year.to_string();
            let mut mo: String = self.tm_mon.to_string();
            if mo.chars().count() < SifrInt::from_i64(2) {
                mo = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize.saturating_add(mo.len()));
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(mo.as_str());
                    sifr_generated_concat
                };
            }
            let mut d: String = self.tm_mday.to_string();
            if d.chars().count() < SifrInt::from_i64(2) {
                d = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize.saturating_add(d.len()));
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(d.as_str());
                    sifr_generated_concat
                };
            }
            let mut h: String = self.tm_hour.to_string();
            if h.chars().count() < SifrInt::from_i64(2) {
                h = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize.saturating_add(h.len()));
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(h.as_str());
                    sifr_generated_concat
                };
            }
            let mut mi: String = self.tm_min.to_string();
            if mi.chars().count() < SifrInt::from_i64(2) {
                mi = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize.saturating_add(mi.len()));
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(mi.as_str());
                    sifr_generated_concat
                };
            }
            let mut s: String = self.tm_sec.to_string();
            if s.chars().count() < SifrInt::from_i64(2) {
                s = {
                    let mut sifr_generated_concat: String =
                        String::with_capacity(1usize.saturating_add(s.len()));
                    sifr_generated_concat.push('0');
                    sifr_generated_concat.push_str(s.as_str());
                    sifr_generated_concat
                };
            }
            {
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
            }
        }
    }
    impl PartialEq for SifrGeneratedStdlibSifrX2etimeX2estructTime {
        fn eq(&self, other: &Self) -> bool {
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
pub use sifr_generated_project_nominals::FloatOverflowError;
pub use sifr_generated_project_nominals::FloatPrecisionLossError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2etimeX2estructTime;
pub use sifr_generated_project_nominals::ValueError;
mod sifr_generated_project_unions {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum SifrGeneratedUnion8X3asequence5X3aunion1X3a336X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a044X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
            crate::sifr_generated_project_nominals::FloatOverflowError,
        ),
        SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
            crate::sifr_generated_project_nominals::FloatPrecisionLossError,
        ),
        SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0(
            crate::sifr_generated_project_nominals::ValueError,
        ),
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a336X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a044X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0 {
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
    #[derive(Debug, Clone)]
    pub enum SifrGeneratedUnion8X3asequence5X3aunion1X3a431X3a5X3aclass18X3asifrX2ebuiltinX2eError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a044X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass18X3asifrX2ebuiltinX2eError1X3a0(
            crate::sifr_generated_project_nominals::Error,
        ),
        SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
            crate::sifr_generated_project_nominals::FloatOverflowError,
        ),
        SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
            crate::sifr_generated_project_nominals::FloatPrecisionLossError,
        ),
        SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0(
            crate::sifr_generated_project_nominals::ValueError,
        ),
    }
    impl From<crate::sifr_generated_project_nominals::Error>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a431X3a5X3aclass18X3asifrX2ebuiltinX2eError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a044X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0 {
        fn from(value: crate::sifr_generated_project_nominals::Error) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass18X3asifrX2ebuiltinX2eError1X3a0(
                value,
            )
        }
    }
    impl From<crate::sifr_generated_project_nominals::FloatOverflowError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a431X3a5X3aclass18X3asifrX2ebuiltinX2eError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a044X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0 {
        fn from(
            value: crate::sifr_generated_project_nominals::FloatOverflowError,
        ) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
                value,
            )
        }
    }
    impl From<crate::sifr_generated_project_nominals::FloatPrecisionLossError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a431X3a5X3aclass18X3asifrX2ebuiltinX2eError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a044X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0 {
        fn from(
            value: crate::sifr_generated_project_nominals::FloatPrecisionLossError,
        ) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
                value,
            )
        }
    }
    impl From<crate::sifr_generated_project_nominals::ValueError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a431X3a5X3aclass18X3asifrX2ebuiltinX2eError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a044X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0 {
        fn from(value: crate::sifr_generated_project_nominals::ValueError) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a431X3a5X3aclass18X3asifrX2ebuiltinX2eError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a044X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                Self::SifrGeneratedUnionVariant5X3aclass18X3asifrX2ebuiltinX2eError1X3a0(
                    v,
                ) => write!(f, "{v}"),
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
}
use crate::sifr_generated_generated_support::{
    assert_bool_vector_eq, gmtime_struct, localtime_struct, mktime, monotonic, perf_counter, sleep,
    strftime, strptime, time,
};
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_unions::SifrGeneratedUnion8X3asequence5X3aunion1X3a336X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a044X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0;
pub use sifr_generated_project_unions::SifrGeneratedUnion8X3asequence5X3aunion1X3a431X3a5X3aclass18X3asifrX2ebuiltinX2eError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a044X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0;
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
    let mut actual: Vec<bool> =
        vec![strftime("%Y-%m-%d %H:%M:%S", 0.0_f64).as_str() == "1970-01-01 00:00:00"];
    let gmt: SifrGeneratedStdlibSifrX2etimeX2estructTime = gmtime_struct(0.0_f64);
    actual.push(
        gmt.tm_year == SifrInt::from_i64(1970)
            && gmt.tm_mon == SifrInt::from_i64(1)
            && gmt.tm_mday == SifrInt::from_i64(1)
            && gmt.tm_hour == SifrInt::from_i64(0)
            && gmt.tm_min == SifrInt::from_i64(0)
            && gmt.tm_sec == SifrInt::from_i64(0),
    );
    let local: SifrGeneratedStdlibSifrX2etimeX2estructTime = localtime_struct(0.0_f64);
    actual.push(local.tm_year > SifrInt::from_i64(0) && local.tm_yday >= SifrInt::from_i64(1));
    actual
}
fn collect_parse_and_safety_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let mut parsed_ok: bool = false;
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let parsed: String = strptime("2024-01-15 10:30:00", "%Y-%m-%d %H:%M:%S")?;
        parsed_ok = parsed == "2024-01-15T10:30:00";
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        let _ = e.message;
        parsed_ok = false;
    }
    actual.push(parsed_ok);
    let mut parse_error_ok: bool = false;
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let _bad: String = strptime("bad", "%Y-%m-%d %H:%M:%S")?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        let _ = e.message;
        parse_error_ok = true;
    }
    actual.push(parse_error_ok);
    sleep(-0.05_f64);
    actual.push(true);
    let epoch_tm: SifrGeneratedStdlibSifrX2etimeX2estructTime = gmtime_struct(0.0_f64);
    let sifr_generated_try_res: Result<
        (),
        SifrGeneratedUnion8X3asequence5X3aunion1X3a431X3a5X3aclass18X3asifrX2ebuiltinX2eError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a044X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0,
    > = (|| {
        let epoch_stamp: f64 = mktime(&epoch_tm)
            .map_err(|sifr_generated_e| match sifr_generated_e {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a336X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a044X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
                    sifr_generated_union_value,
                ) => {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a431X3a5X3aclass18X3asifrX2ebuiltinX2eError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a044X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
                        sifr_generated_union_value,
                    )
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a336X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a044X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
                    sifr_generated_union_value,
                ) => {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a431X3a5X3aclass18X3asifrX2ebuiltinX2eError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a044X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
                        sifr_generated_union_value,
                    )
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a336X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a044X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0(
                    sifr_generated_union_value,
                ) => {
                    SifrGeneratedUnion8X3asequence5X3aunion1X3a431X3a5X3aclass18X3asifrX2ebuiltinX2eError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a044X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0(
                        sifr_generated_union_value,
                    )
                }
            })?;
        actual.push(epoch_stamp == 0.0_f64);
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        match sifr_generated_try_err {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a431X3a5X3aclass18X3asifrX2ebuiltinX2eError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a044X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3asifrX2ebuiltinX2eError1X3a0(
                _try_variant_error,
            ) => {
                actual.push(false);
            }
            SifrGeneratedUnion8X3asequence5X3aunion1X3a431X3a5X3aclass18X3asifrX2ebuiltinX2eError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a044X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a0(
                _try_variant_error,
            ) => {
                actual.push(false);
            }
            SifrGeneratedUnion8X3asequence5X3aunion1X3a431X3a5X3aclass18X3asifrX2ebuiltinX2eError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a044X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0(
                _try_variant_error,
            ) => {
                actual.push(false);
            }
            SifrGeneratedUnion8X3asequence5X3aunion1X3a431X3a5X3aclass18X3asifrX2ebuiltinX2eError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a044X3a5X3aclass31X3asifrX2ebuiltinX2eFloatOverflowError1X3a049X3a5X3aclass36X3asifrX2ebuiltinX2eFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0(
                _try_variant_error,
            ) => {
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
