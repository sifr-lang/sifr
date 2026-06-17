// --- stdlib: sifr.time ---
#[derive(Debug, Clone)]
struct struct_time {
    tm_year: i64,
    tm_mon: i64,
    tm_mday: i64,
    tm_hour: i64,
    tm_min: i64,
    tm_sec: i64,
    tm_wday: i64,
    tm_yday: i64,
    tm_isdst: i64,
}
impl struct_time {
    fn new(
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
        return Self {
            tm_year: tm_year,
            tm_mon: tm_mon,
            tm_mday: tm_mday,
            tm_hour: tm_hour,
            tm_min: tm_min,
            tm_sec: tm_sec,
            tm_wday: tm_wday,
            tm_yday: tm_yday,
            tm_isdst: tm_isdst,
        };
    }
    fn as_tuple(&self) -> (i64, i64, i64, i64, i64, i64, i64, i64, i64) {
        return (
            self.tm_year,
            self.tm_mon,
            self.tm_mday,
            self.tm_hour,
            self.tm_min,
            self.tm_sec,
            self.tm_wday,
            self.tm_yday,
            self.tm_isdst,
        );
    }
    fn isoformat(&self) -> String {
        let y: String = format!("{}", self.tm_year);
        let mut mo: String = format!("{}", self.tm_mon);
        if (mo.len() as i64) < (2 as i64) {
            mo = format!("{}{}", "0".to_string(), mo);
        }
        let mut d: String = format!("{}", self.tm_mday);
        if (d.len() as i64) < (2 as i64) {
            d = format!("{}{}", "0".to_string(), d);
        }
        let mut h: String = format!("{}", self.tm_hour);
        if (h.len() as i64) < (2 as i64) {
            h = format!("{}{}", "0".to_string(), h);
        }
        let mut mi: String = format!("{}", self.tm_min);
        if (mi.len() as i64) < (2 as i64) {
            mi = format!("{}{}", "0".to_string(), mi);
        }
        let mut s: String = format!("{}", self.tm_sec);
        if (s.len() as i64) < (2 as i64) {
            s = format!("{}{}", "0".to_string(), s);
        }
        return format!(
            "{}{}{}{}{}{}{}{}{}{}{}", y, "-".to_string(), mo, "-".to_string(), d, "T"
            .to_string(), h, ":".to_string(), mi, ":".to_string(), s
        );
    }
}
impl PartialEq for struct_time {
    fn eq(&self, other: &struct_time) -> bool {
        return self.as_tuple() == other.as_tuple();
    }
}
impl std::fmt::Display for struct_time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.isoformat());
    }
}
fn _is_leap_year(year: i64) -> bool {
    return (((year % (4 as i64)) == (0 as i64)) && ((year % (100 as i64)) != (0 as i64)))
        || ((year % (400 as i64)) == (0 as i64));
}
fn _days_in_year(year: i64) -> i64 {
    if _is_leap_year(year) {
        return 366 as i64;
    }
    return 365 as i64;
}
fn _days_in_month(year: i64, month: i64) -> i64 {
    let month_days: Vec<i64> = vec![
        31 as i64, 28 as i64, 31 as i64, 30 as i64, 31 as i64, 30 as i64, 31 as i64, 31
        as i64, 30 as i64, 31 as i64, 30 as i64, 31 as i64
    ];
    let idx: i64 = month - (1 as i64);
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
    if ((month == (2 as i64)) && (_is_leap_year(year))) {
        return 29 as i64;
    }
    if let Some(d) = d {
        return d;
    }
    return 0 as i64;
}
fn _substring(value: &String, start: i64, end: i64) -> String {
    let mut result: String = "".to_string();
    let mut i: i64 = start;
    while i < end {
        let ch: Option<String> = {
            let __sifr_index_str = &value;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        if let Some(ch) = ch {
            result = format!("{}{}", result, ch);
        }
        i = i + (1 as i64);
    }
    return result;
}
fn _digit_value(ch: &String) -> Option<i64> {
    if ch.clone() == "0".to_string() {
        return Some(0 as i64);
    }
    if ch.clone() == "1".to_string() {
        return Some(1 as i64);
    }
    if ch.clone() == "2".to_string() {
        return Some(2 as i64);
    }
    if ch.clone() == "3".to_string() {
        return Some(3 as i64);
    }
    if ch.clone() == "4".to_string() {
        return Some(4 as i64);
    }
    if ch.clone() == "5".to_string() {
        return Some(5 as i64);
    }
    if ch.clone() == "6".to_string() {
        return Some(6 as i64);
    }
    if ch.clone() == "7".to_string() {
        return Some(7 as i64);
    }
    if ch.clone() == "8".to_string() {
        return Some(8 as i64);
    }
    if ch.clone() == "9".to_string() {
        return Some(9 as i64);
    }
    return None;
}
fn _parse_decimal(text: &String) -> Option<i64> {
    if (text.len() as i64) == (0 as i64) {
        return None;
    }
    let mut out: i64 = 0 as i64;
    let mut i: i64 = 0 as i64;
    while i < (text.chars().count() as i64) {
        let ch_opt: Option<String> = Some({
            let Some(__indexed_char) = text.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
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
        out = (out * (10 as i64)) + digit;
        i = i + (1 as i64);
    }
    return Some(out);
}
fn _int_or_negative_one(value: Option<i64>) -> i64 {
    let Some(value) = value else {
        return -(1 as i64);
    };
    return value;
}
fn _day_of_year(year: i64, month: i64, day: i64) -> i64 {
    let mut yday: i64 = 0 as i64;
    let mut m: i64 = 1 as i64;
    while m < month {
        yday = yday + _days_in_month(year, m);
        m = m + (1 as i64);
    }
    return yday + day;
}
fn _weekday(year: i64, month: i64, day: i64) -> i64 {
    let mut days_since_epoch: i64 = 0 as i64;
    if year >= (1970 as i64) {
        let mut y: i64 = 1970 as i64;
        while y < year {
            days_since_epoch = days_since_epoch + _days_in_year(y);
            y = y + (1 as i64);
        }
    } else {
        let mut y: i64 = 1969 as i64;
        while y >= year {
            days_since_epoch = days_since_epoch - _days_in_year(y);
            y = y - (1 as i64);
        }
    }
    let mut m: i64 = 1 as i64;
    while m < month {
        days_since_epoch = days_since_epoch + _days_in_month(year, m);
        m = m + (1 as i64);
    }
    days_since_epoch = (days_since_epoch + day) - (1 as i64);
    let mut wd: i64 = ((3 as i64) + days_since_epoch) % (7 as i64);
    if wd < (0 as i64) {
        wd = wd + (7 as i64);
    }
    return wd;
}
fn _valid_date(year: i64, month: i64, day: i64) -> bool {
    if year <= (0 as i64) {
        return false;
    }
    if (month < (1 as i64)) || (month > (12 as i64)) {
        return false;
    }
    let max_day: i64 = _days_in_month(year, month);
    return (day >= (1 as i64)) && (day <= max_day);
}
fn _invalid_struct_time() -> struct_time {
    return struct_time::new(
        0 as i64,
        0 as i64,
        0 as i64,
        0 as i64,
        0 as i64,
        0 as i64,
        0 as i64,
        0 as i64,
        0 as i64,
    );
}
fn _to_struct_time(rendered: &String) -> struct_time {
    if (rendered.chars().count() as i64) < (19 as i64) {
        return _invalid_struct_time();
    }
    if (((((({
        let Some(__indexed_char) = rendered.chars().nth((4 as i64) as usize) else {
            unreachable!("compiler-verified string index should be in range");
        };
        __indexed_char.to_string()
    }) != "-".to_string())
        || (({
            let Some(__indexed_char) = rendered.chars().nth((7 as i64) as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        }) != "-".to_string()))
        || (({
            let Some(__indexed_char) = rendered.chars().nth((10 as i64) as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        }) != "T".to_string()))
        || (({
            let Some(__indexed_char) = rendered.chars().nth((13 as i64) as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        }) != ":".to_string()))
        || (({
            let Some(__indexed_char) = rendered.chars().nth((16 as i64) as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        }) != ":".to_string()))
    {
        return _invalid_struct_time();
    }
    let year: i64 = _int_or_negative_one(
        _parse_decimal(&_substring(rendered, 0 as i64, 4 as i64)),
    );
    let month: i64 = _int_or_negative_one(
        _parse_decimal(&_substring(rendered, 5 as i64, 7 as i64)),
    );
    let day: i64 = _int_or_negative_one(
        _parse_decimal(&_substring(rendered, 8 as i64, 10 as i64)),
    );
    let hour: i64 = _int_or_negative_one(
        _parse_decimal(&_substring(rendered, 11 as i64, 13 as i64)),
    );
    let minute: i64 = _int_or_negative_one(
        _parse_decimal(&_substring(rendered, 14 as i64, 16 as i64)),
    );
    let second: i64 = _int_or_negative_one(
        _parse_decimal(&_substring(rendered, 17 as i64, 19 as i64)),
    );
    if (((((year < (0 as i64)) || (month < (0 as i64))) || (day < (0 as i64)))
        || (hour < (0 as i64))) || (minute < (0 as i64))) || (second < (0 as i64))
    {
        return _invalid_struct_time();
    }
    if !(_valid_date(year, month, day)) {
        return _invalid_struct_time();
    }
    let wday: i64 = _weekday(year, month, day);
    let yday: i64 = _day_of_year(year, month, day);
    return struct_time::new(
        year,
        month,
        day,
        hour,
        minute,
        second,
        wday,
        yday,
        0 as i64,
    );
}
fn time() -> f64 {
    return std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
}
fn strftime(fmt: &String, epoch: f64) -> String {
    return {
        let secs = epoch as i64;
        let dt = chrono::DateTime::from_timestamp(secs, 0).unwrap_or_default();
        dt.format(&fmt).to_string()
    };
}
fn strptime(s: &String, fmt: &String) -> Result<String, ValueError> {
    return chrono::NaiveDateTime::parse_from_str(&s, &fmt)
        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S").to_string())
        .map_err(|e| ValueError {
            message: e.to_string(),
        });
}
fn gmtime_struct(epoch: f64) -> struct_time {
    let rendered: String = {
        let __ts = epoch as i64;
        chrono::DateTime::<chrono::Utc>::from_timestamp(__ts, 0)
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S").to_string())
            .unwrap_or_default()
    };
    return _to_struct_time(&rendered);
}
fn localtime_struct(epoch: f64) -> struct_time {
    let rendered: String = {
        let __ts = epoch as i64;
        chrono::DateTime::<chrono::Utc>::from_timestamp(__ts, 0)
            .map(|dt| {
                dt.with_timezone(&chrono::Local).format("%Y-%m-%dT%H:%M:%S").to_string()
            })
            .unwrap_or_default()
    };
    return _to_struct_time(&rendered);
}
fn mktime(t: &struct_time) -> f64 {
    if !(_valid_date(t.tm_year, t.tm_mon, t.tm_mday)) {
        return 0.0 as f64;
    }
    let mut days: i64 = 0 as i64;
    if t.tm_year >= (1970 as i64) {
        let mut y: i64 = 1970 as i64;
        while y < t.tm_year {
            days = days + _days_in_year(y);
            y = y + (1 as i64);
        }
    } else {
        let mut y: i64 = 1969 as i64;
        while y >= t.tm_year {
            days = days - _days_in_year(y);
            y = y - (1 as i64);
        }
    }
    let mut m: i64 = 1 as i64;
    while m < t.tm_mon {
        days = days + _days_in_month(t.tm_year, m);
        m = m + (1 as i64);
    }
    days = (days + t.tm_mday) - (1 as i64);
    let stamp: i64 = (((days * (86400 as i64)) + (t.tm_hour * (3600 as i64)))
        + (t.tm_min * (60 as i64))) + t.tm_sec;
    return stamp as f64;
}

// --- stdlib: sifr.test ---
fn assert_eq<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    actual: &T,
    expected: &T,
) {
    assert!(* actual == * expected);
}
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0 as i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize]) == expected.get(i as usize).copied());
        i = i + (1 as i64);
    }
}

#[derive(Debug, Clone)]
struct IOError {
    message: String,
    kind: String,
}

impl IOError {
    fn new(message: String) -> Self {
        return Self { message: message, kind: "Other".to_string() };
    }
}

impl std::fmt::Display for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for IOError {
}

fn __io_err(e: std::io::Error) -> IOError {
    let msg = e.to_string();
    let kind = if e.kind() == std::io::ErrorKind::NotFound { "FileNotFound".to_string() } else { if e.kind() == std::io::ErrorKind::PermissionDenied { "PermissionDenied".to_string() } else { if e.kind() == std::io::ErrorKind::AlreadyExists { "FileExists".to_string() } else { "Other".to_string() } } };
    return IOError { message: msg, kind: kind };
}

#[derive(Debug, Clone)]
struct ParseError {
    message: String,
}

impl ParseError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for ParseError {
}

#[derive(Debug, Clone)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for ValueError {
}

#[derive(Debug, Clone)]
struct JSONDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl JSONDecodeError {
    fn new(message: String) -> Self {
        return Self { message: message, line: 0, column: 0 };
    }
}

impl std::fmt::Display for JSONDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for JSONDecodeError {
}

#[derive(Debug, Clone)]
struct TOMLDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl TOMLDecodeError {
    fn new(message: String) -> Self {
        return Self { message: message, line: 0, column: 0 };
    }
}

impl std::fmt::Display for TOMLDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for TOMLDecodeError {
}

#[derive(Debug, Clone)]
struct RegexError {
    message: String,
    detail: String,
}

impl RegexError {
    fn new(message: String) -> Self {
        return Self { message: message, detail: String::new() };
    }
}

impl std::fmt::Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for RegexError {
}

fn collect_clock_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    actual.push((std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs_f64() > (0.0 as f64)) && (time() > (0.0 as f64)));
    let perf_before: f64 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs_f64();
    let mono_before: f64 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs_f64();
    {
    let __secs = 0.01 as f64;
    if __secs.is_finite() && (__secs > 0.0) { std::thread::sleep(std::time::Duration::from_nanos((__secs * 1000000000.0) as u64)) } else { () }
};
    let perf_after: f64 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs_f64();
    let mono_after: f64 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs_f64();
    actual.push((perf_after >= perf_before) && (mono_after >= mono_before));
    return actual;
}

fn collect_format_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    actual.push((strftime(&"%Y-%m-%d %H:%M:%S".to_string(), 0.0 as f64)).as_str() == ("1970-01-01 00:00:00".to_string()).as_str());
    let gmt: struct_time = gmtime_struct(0.0 as f64);
    actual.push((((((gmt.tm_year == (1970 as i64)) && (gmt.tm_mon == (1 as i64))) && (gmt.tm_mday == (1 as i64))) && (gmt.tm_hour == (0 as i64))) && (gmt.tm_min == (0 as i64))) && (gmt.tm_sec == (0 as i64)));
    let local: struct_time = localtime_struct(0.0 as f64);
    actual.push((local.tm_year > (0 as i64)) && (local.tm_yday >= (1 as i64)));
    return actual;
}

fn collect_parse_and_safety_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let mut parsed_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let parsed: String = chrono::NaiveDateTime::parse_from_str(&"2024-01-15 10:30:00".to_string(), &"%Y-%m-%d %H:%M:%S".to_string()).map(|dt| dt.format("%Y-%m-%dT%H:%M:%S").to_string()).map_err(|e| ValueError { message: e.to_string() })?;
    parsed_ok = parsed == "2024-01-15T10:30:00".to_string();
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
        parsed_ok = false;
    }
    actual.push(parsed_ok);
    let mut parse_error_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let _bad: String = chrono::NaiveDateTime::parse_from_str(&"bad".to_string(), &"%Y-%m-%d %H:%M:%S".to_string()).map(|dt| dt.format("%Y-%m-%dT%H:%M:%S").to_string()).map_err(|e| ValueError { message: e.to_string() })?;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
        parse_error_ok = true;
    }
    actual.push(parse_error_ok);
    {
    let __secs = -(0.05 as f64);
    if __secs.is_finite() && (__secs > 0.0) { std::thread::sleep(std::time::Duration::from_nanos((__secs * 1000000000.0) as u64)) } else { () }
};
    actual.push(true);
    let epoch_tm: struct_time = gmtime_struct(0.0 as f64);
    actual.push(mktime(&epoch_tm) == (0.0 as f64));
    return actual;
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
