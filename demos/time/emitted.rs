// src/main.rs
mod __sifr_project_nominals {
    pub fn time_now() -> f64 {
        ::sifr_stdlib::time::time_now()
    }
    pub fn time_format(epoch: f64, fmt: &String) -> String {
        ::sifr_stdlib::time::time_format(epoch, fmt)
    }
    pub fn perf_counter() -> f64 {
        ::sifr_stdlib::time::perf_counter()
    }
    pub fn sleep(seconds: f64) {
        ::sifr_stdlib::time::sleep(seconds);
    }
    pub fn monotonic() -> f64 {
        ::sifr_stdlib::time::monotonic()
    }
    pub fn strptime(s: &String, fmt: &String) -> Result<String, ValueError> {
        ::sifr_stdlib::time::strptime(s, fmt)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ValueError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn _strptime_intrinsic(s: &String, fmt: &String) -> Result<String, ValueError> {
        ::sifr_stdlib::time::strptime(s, fmt)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ValueError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn gmtime(epoch: f64) -> String {
        ::sifr_stdlib::time::gmtime(epoch)
    }
    pub fn _gmtime_intrinsic(epoch: f64) -> String {
        ::sifr_stdlib::time::gmtime(epoch)
    }
    pub fn localtime(epoch: f64) -> String {
        ::sifr_stdlib::time::localtime(epoch)
    }
    pub fn _localtime_intrinsic(epoch: f64) -> String {
        ::sifr_stdlib::time::localtime(epoch)
    }
    pub fn time_strptime(s: &String, fmt: &String) -> Result<Vec<i64>, ValueError> {
        ::sifr_stdlib::time::time_strptime(s, fmt)
            .map(|__sifr_bridge_ok| {
                __sifr_bridge_ok
                    .into_iter()
                    .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
                    .collect()
            })
            .map_err(|__sifr_bridge_error| ValueError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn time_gmtime() -> Vec<i64> {
        ::sifr_stdlib::time::time_gmtime()
            .into_iter()
            .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
            .collect()
    }
    pub fn time_localtime() -> Vec<i64> {
        ::sifr_stdlib::time::time_localtime()
            .into_iter()
            .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
            .collect()
    }
    #[derive(Debug, Clone)]
    pub struct __SifrStdlib_sifr_x2etime_x2estruct__time {
        pub tm_year: i64,
        pub tm_mon: i64,
        pub tm_mday: i64,
        pub tm_hour: i64,
        pub tm_min: i64,
        pub tm_sec: i64,
        pub tm_wday: i64,
        pub tm_yday: i64,
        pub tm_isdst: i64,
    }
    impl __SifrStdlib_sifr_x2etime_x2estruct__time {
        pub fn new(
            tm_year: i64,
            tm_mon: i64,
            tm_mday: i64,
            tm_hour: i64,
            tm_min: i64,
            tm_sec: i64,
            tm_wday: i64,
            tm_yday: i64,
            tm_isdst: i64,
        ) -> Self {
            let __sifr_field_init_0: i64 = tm_year;
            let __sifr_field_init_1: i64 = tm_mon;
            let __sifr_field_init_2: i64 = tm_mday;
            let __sifr_field_init_3: i64 = tm_hour;
            let __sifr_field_init_4: i64 = tm_min;
            let __sifr_field_init_5: i64 = tm_sec;
            let __sifr_field_init_6: i64 = tm_wday;
            let __sifr_field_init_7: i64 = tm_yday;
            let __sifr_field_init_8: i64 = tm_isdst;
            Self {
                tm_year: __sifr_field_init_0,
                tm_mon: __sifr_field_init_1,
                tm_mday: __sifr_field_init_2,
                tm_hour: __sifr_field_init_3,
                tm_min: __sifr_field_init_4,
                tm_sec: __sifr_field_init_5,
                tm_wday: __sifr_field_init_6,
                tm_yday: __sifr_field_init_7,
                tm_isdst: __sifr_field_init_8,
            }
        }
    }
    impl __SifrStdlib_sifr_x2etime_x2estruct__time {
        pub fn as_tuple(&self) -> (i64, i64, i64, i64, i64, i64, i64, i64, i64) {
            (
                self.tm_year,
                self.tm_mon,
                self.tm_mday,
                self.tm_hour,
                self.tm_min,
                self.tm_sec,
                self.tm_wday,
                self.tm_yday,
                self.tm_isdst,
            )
        }
    }
    impl __SifrStdlib_sifr_x2etime_x2estruct__time {
        pub fn isoformat(&self) -> String {
            let y: String = format!("{}", self.tm_year);
            let mut mo: String = format!("{}", self.tm_mon);
            if ((mo.chars().count() as i64) < (2_i64)) {
                mo = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + mo.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((mo).as_str());
                    __sifr_concat
                };
            }
            let mut d: String = format!("{}", self.tm_mday);
            if ((d.chars().count() as i64) < (2_i64)) {
                d = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + d.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((d).as_str());
                    __sifr_concat
                };
            }
            let mut h: String = format!("{}", self.tm_hour);
            if ((h.chars().count() as i64) < (2_i64)) {
                h = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + h.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((h).as_str());
                    __sifr_concat
                };
            }
            let mut mi: String = format!("{}", self.tm_min);
            if ((mi.chars().count() as i64) < (2_i64)) {
                mi = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + mi.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((mi).as_str());
                    __sifr_concat
                };
            }
            let mut s: String = format!("{}", self.tm_sec);
            if ((s.chars().count() as i64) < (2_i64)) {
                s = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + s.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((s).as_str());
                    __sifr_concat
                };
            }
            {
                let mut __sifr_concat: String = String::with_capacity(
                    (((((((((y.len() + 1usize) + mo.len()) + 1usize) + d.len()) + 1usize)
                        + h.len()) + 1usize) + mi.len()) + 1usize) + s.len(),
                );
                __sifr_concat.push_str((y).as_str());
                __sifr_concat.push('-');
                __sifr_concat.push_str((mo).as_str());
                __sifr_concat.push('-');
                __sifr_concat.push_str((d).as_str());
                __sifr_concat.push('T');
                __sifr_concat.push_str((h).as_str());
                __sifr_concat.push(':');
                __sifr_concat.push_str((mi).as_str());
                __sifr_concat.push(':');
                __sifr_concat.push_str((s).as_str());
                __sifr_concat
            }
        }
    }
    impl PartialEq for __SifrStdlib_sifr_x2etime_x2estruct__time {
        fn eq(&self, other: &__SifrStdlib_sifr_x2etime_x2estruct__time) -> bool {
            self.as_tuple() == other.as_tuple()
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2etime_x2estruct__time {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.isoformat())
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueError {
        pub message: String,
    }
    impl ValueError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for ValueError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ValueError {}
}
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2etime_x2estruct__time;
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0_i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize]) == expected.get(i as usize).copied());
        i += 1_i64;
    }
}
fn time_now() -> f64 {
    ::sifr_stdlib::time::time_now()
}
fn time_format(epoch: f64, fmt: &String) -> String {
    ::sifr_stdlib::time::time_format(epoch, fmt)
}
fn perf_counter() -> f64 {
    ::sifr_stdlib::time::perf_counter()
}
fn sleep(seconds: f64) {
    ::sifr_stdlib::time::sleep(seconds);
}
fn monotonic() -> f64 {
    ::sifr_stdlib::time::monotonic()
}
fn strptime(s: &String, fmt: &String) -> Result<String, ValueError> {
    ::sifr_stdlib::time::strptime(s, fmt)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _strptime_intrinsic(s: &String, fmt: &String) -> Result<String, ValueError> {
    ::sifr_stdlib::time::strptime(s, fmt)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn gmtime(epoch: f64) -> String {
    ::sifr_stdlib::time::gmtime(epoch)
}
fn _gmtime_intrinsic(epoch: f64) -> String {
    ::sifr_stdlib::time::gmtime(epoch)
}
fn localtime(epoch: f64) -> String {
    ::sifr_stdlib::time::localtime(epoch)
}
fn _localtime_intrinsic(epoch: f64) -> String {
    ::sifr_stdlib::time::localtime(epoch)
}
fn time_strptime(s: &String, fmt: &String) -> Result<Vec<i64>, ValueError> {
    ::sifr_stdlib::time::time_strptime(s, fmt)
        .map(|__sifr_bridge_ok| {
            __sifr_bridge_ok
                .into_iter()
                .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
                .collect()
        })
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn time_gmtime() -> Vec<i64> {
    ::sifr_stdlib::time::time_gmtime()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}
fn time_localtime() -> Vec<i64> {
    ::sifr_stdlib::time::time_localtime()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}
fn _is_leap_year(year: i64) -> bool {
    (((year % (4_i64)) == (0_i64)) && ((year % (100_i64)) != (0_i64)))
        || ((year % (400_i64)) == (0_i64))
}
fn _days_in_year(year: i64) -> i64 {
    if _is_leap_year(year) {
        return 366_i64;
    }
    365_i64
}
fn _days_in_month(year: i64, month: i64) -> i64 {
    let month_days: Vec<i64> = vec![
        31_i64, 28_i64, 31_i64, 30_i64, 31_i64, 30_i64, 31_i64, 31_i64, 30_i64, 31_i64,
        30_i64, 31_i64
    ];
    let idx: i64 = month - (1_i64);
    let d: Option<i64> = {
        let __sifr_index_list = &month_days;
        let __sifr_index_i = idx;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    if (month == (2_i64)) && _is_leap_year(year) {
        return 29_i64;
    }
    if let Some(d) = d {
        return d;
    }
    0_i64
}
fn _substring(value: &String, start: i64, end: i64) -> String {
    let __sifr_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
    let mut result: String = "".to_string();
    let mut i: i64 = start;
    while i < end {
        let ch: Option<String> = __sifr_chars_value
            .get(i as usize)
            .map(|c| c.to_string());
        if let Some(ch) = ch {
            result.push_str((ch).as_str());
        }
        i += 1_i64;
    }
    result
}
fn _digit_value(ch: &String) -> Option<i64> {
    if (ch).as_str() == "0" {
        return Some(0_i64);
    }
    if (ch).as_str() == "1" {
        return Some(1_i64);
    }
    if (ch).as_str() == "2" {
        return Some(2_i64);
    }
    if (ch).as_str() == "3" {
        return Some(3_i64);
    }
    if (ch).as_str() == "4" {
        return Some(4_i64);
    }
    if (ch).as_str() == "5" {
        return Some(5_i64);
    }
    if (ch).as_str() == "6" {
        return Some(6_i64);
    }
    if (ch).as_str() == "7" {
        return Some(7_i64);
    }
    if (ch).as_str() == "8" {
        return Some(8_i64);
    }
    if (ch).as_str() == "9" {
        return Some(9_i64);
    }
    None
}
fn _parse_decimal(text: &String) -> Option<i64> {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    if ((__sifr_chars_text.len() as i64) == (0_i64)) {
        return None;
    }
    let mut out: i64 = 0_i64;
    let mut i: i64 = 0_i64;
    while (i < (__sifr_chars_text.len() as i64)) {
        let ch_opt: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_text
                .get(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        let Some(ch_opt) = ch_opt else {
            return None;
        };
        let ch: String = ch_opt;
        let digit_opt: Option<i64> = _digit_value(&ch);
        let Some(digit_opt) = digit_opt else {
            return None;
        };
        let digit: i64 = digit_opt;
        out = (out * (10_i64)) + digit;
        i += 1_i64;
    }
    Some(out)
}
fn _int_or_negative_one(value: Option<i64>) -> i64 {
    let Some(value) = value else {
        return -(1_i64);
    };
    value
}
fn _day_of_year(year: i64, month: i64, day: i64) -> i64 {
    let mut yday: i64 = 0_i64;
    let mut m: i64 = 1_i64;
    while m < month {
        yday += _days_in_month(year, m);
        m += 1_i64;
    }
    yday + day
}
fn _weekday(year: i64, month: i64, day: i64) -> i64 {
    let mut days_since_epoch: i64 = 0_i64;
    if year >= (1970_i64) {
        let mut y: i64 = 1970_i64;
        while y < year {
            days_since_epoch += _days_in_year(y);
            y += 1_i64;
        }
    } else {
        let mut y: i64 = 1969_i64;
        while y >= year {
            days_since_epoch -= _days_in_year(y);
            y -= 1_i64;
        }
    }
    let mut m: i64 = 1_i64;
    while m < month {
        days_since_epoch += _days_in_month(year, m);
        m += 1_i64;
    }
    days_since_epoch = (days_since_epoch + day) - (1_i64);
    let mut wd: i64 = ((3_i64) + days_since_epoch) % (7_i64);
    if wd < (0_i64) {
        wd += 7_i64;
    }
    wd
}
fn _valid_date(year: i64, month: i64, day: i64) -> bool {
    if year <= (0_i64) {
        return false;
    }
    if (month < (1_i64)) || (month > (12_i64)) {
        return false;
    }
    let max_day: i64 = _days_in_month(year, month);
    (day >= (1_i64)) && (day <= max_day)
}
fn _invalid_struct_time() -> __SifrStdlib_sifr_x2etime_x2estruct__time {
    __SifrStdlib_sifr_x2etime_x2estruct__time::new(
        0_i64,
        0_i64,
        0_i64,
        0_i64,
        0_i64,
        0_i64,
        0_i64,
        0_i64,
        0_i64,
    )
}
fn _to_struct_time(rendered: &String) -> __SifrStdlib_sifr_x2etime_x2estruct__time {
    let __sifr_chars_rendered: Vec<char> = rendered.chars().collect::<Vec<char>>();
    if ((__sifr_chars_rendered.len() as i64) < (19_i64)) {
        return _invalid_struct_time();
    }
    if ((((({
        let Some(__indexed_char) = __sifr_chars_rendered
            .get((4_i64) as usize)
            .map(|c| c.to_string()) else {
            unreachable!("compiler-verified string index should be in range");
        };
        __indexed_char
    }) != "-")
        || (({
            let Some(__indexed_char) = __sifr_chars_rendered
                .get((7_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) != "-"))
        || (({
            let Some(__indexed_char) = __sifr_chars_rendered
                .get((10_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) != "T"))
        || (({
            let Some(__indexed_char) = __sifr_chars_rendered
                .get((13_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) != ":"))
        || (({
            let Some(__indexed_char) = __sifr_chars_rendered
                .get((16_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) != ":")
    {
        return _invalid_struct_time();
    }
    let year: i64 = _int_or_negative_one(
        _parse_decimal(&_substring(rendered, 0_i64, 4_i64)),
    );
    let month: i64 = _int_or_negative_one(
        _parse_decimal(&_substring(rendered, 5_i64, 7_i64)),
    );
    let day: i64 = _int_or_negative_one(
        _parse_decimal(&_substring(rendered, 8_i64, 10_i64)),
    );
    let hour: i64 = _int_or_negative_one(
        _parse_decimal(&_substring(rendered, 11_i64, 13_i64)),
    );
    let minute: i64 = _int_or_negative_one(
        _parse_decimal(&_substring(rendered, 14_i64, 16_i64)),
    );
    let second: i64 = _int_or_negative_one(
        _parse_decimal(&_substring(rendered, 17_i64, 19_i64)),
    );
    if (((((year < (0_i64)) || (month < (0_i64))) || (day < (0_i64)))
        || (hour < (0_i64))) || (minute < (0_i64))) || (second < (0_i64))
    {
        return _invalid_struct_time();
    }
    if !(_valid_date(year, month, day)) {
        return _invalid_struct_time();
    }
    let wday: i64 = _weekday(year, month, day);
    let yday: i64 = _day_of_year(year, month, day);
    __SifrStdlib_sifr_x2etime_x2estruct__time::new(
        year,
        month,
        day,
        hour,
        minute,
        second,
        wday,
        yday,
        0_i64,
    )
}
fn time() -> f64 {
    time_now()
}
fn strftime(fmt: &String, epoch: f64) -> String {
    time_format(epoch, fmt)
}
fn gmtime_struct(epoch: f64) -> __SifrStdlib_sifr_x2etime_x2estruct__time {
    let rendered: String = _gmtime_intrinsic(epoch);
    _to_struct_time(&rendered)
}
fn localtime_struct(epoch: f64) -> __SifrStdlib_sifr_x2etime_x2estruct__time {
    let rendered: String = _localtime_intrinsic(epoch);
    _to_struct_time(&rendered)
}
fn mktime(t: &__SifrStdlib_sifr_x2etime_x2estruct__time) -> f64 {
    if !(_valid_date(t.tm_year, t.tm_mon, t.tm_mday)) {
        return 0.0_f64;
    }
    let mut days: i64 = 0_i64;
    if (t.tm_year >= (1970_i64)) {
        let mut y: i64 = 1970_i64;
        while (y < t.tm_year) {
            days += _days_in_year(y);
            y += 1_i64;
        }
    } else {
        let mut y: i64 = 1969_i64;
        while (y >= t.tm_year) {
            days -= _days_in_year(y);
            y -= 1_i64;
        }
    }
    let mut m: i64 = 1_i64;
    while (m < t.tm_mon) {
        days += _days_in_month(t.tm_year, m);
        m += 1_i64;
    }
    days = (days + t.tm_mday) - (1_i64);
    let stamp: i64 = (((days * (86400_i64)) + (t.tm_hour * (3600_i64)))
        + (t.tm_min * (60_i64))) + t.tm_sec;
    stamp as f64
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ValueError {
    message: String,
}
impl ValueError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for ValueError {}
fn collect_clock_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    actual.push(time() > (0.0_f64));
    let perf_before: f64 = perf_counter();
    let mono_before: f64 = monotonic();
    sleep(0.01_f64);
    let perf_after: f64 = perf_counter();
    let mono_after: f64 = monotonic();
    actual.push((perf_after >= perf_before) && (mono_after >= mono_before));
    actual
}
fn collect_format_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    actual
        .push(
            (strftime(&"%Y-%m-%d %H:%M:%S".to_string(), 0.0_f64)).as_str()
                == ("1970-01-01 00:00:00".to_string()).as_str(),
        );
    let gmt: __SifrStdlib_sifr_x2etime_x2estruct__time = gmtime_struct(0.0_f64);
    actual
        .push(
            (((((gmt.tm_year == (1970_i64)) && (gmt.tm_mon == (1_i64)))
                && (gmt.tm_mday == (1_i64))) && (gmt.tm_hour == (0_i64)))
                && (gmt.tm_min == (0_i64))) && (gmt.tm_sec == (0_i64)),
        );
    let local: __SifrStdlib_sifr_x2etime_x2estruct__time = localtime_struct(0.0_f64);
    actual.push((local.tm_year > (0_i64)) && (local.tm_yday >= (1_i64)));
    actual
}
fn collect_parse_and_safety_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let mut parsed_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let parsed: String = strptime(
            &"2024-01-15 10:30:00".to_string(),
            &"%Y-%m-%d %H:%M:%S".to_string(),
        )?;
        parsed_ok = parsed == "2024-01-15T10:30:00";
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message.clone());
        parsed_ok = false;
    }
    actual.push(parsed_ok);
    let mut parse_error_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let _bad: String = strptime(
            &"bad".to_string(),
            &"%Y-%m-%d %H:%M:%S".to_string(),
        )?;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message.clone());
        parse_error_ok = true;
    }
    actual.push(parse_error_ok);
    sleep(-(0.05_f64));
    actual.push(true);
    let epoch_tm: __SifrStdlib_sifr_x2etime_x2estruct__time = gmtime_struct(0.0_f64);
    actual.push(mktime(&epoch_tm) == (0.0_f64));
    actual
}
fn append_all(target: &mut Vec<bool>, values: &Vec<bool>) {
    for value in values.iter().copied() {
        target.push(value);
    }
}
fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true, true, true, true];
    let mut actual: Vec<bool> = vec![];
    append_all(&mut actual, &collect_clock_actual());
    append_all(&mut actual, &collect_format_actual());
    append_all(&mut actual, &collect_parse_and_safety_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("time time parity demo: pass");
}
