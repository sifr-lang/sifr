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
            "{}{}{}{}{}{}{}{}{}{}{}",
            y,
            "-".to_string(),
            mo,
            "-".to_string(),
            d,
            "T".to_string(),
            h,
            ":".to_string(),
            mi,
            ":".to_string(),
            s
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
        31 as i64, 28 as i64, 31 as i64, 30 as i64, 31 as i64, 30 as i64, 31 as i64, 31 as i64,
        30 as i64, 31 as i64, 30 as i64, 31 as i64,
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
            __sifr_index_str
                .chars()
                .nth(__sifr_index_norm)
                .map(|c| c.to_string())
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
        0 as i64, 0 as i64, 0 as i64, 0 as i64, 0 as i64, 0 as i64, 0 as i64, 0 as i64, 0 as i64,
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
    let year: i64 = _int_or_negative_one(_parse_decimal(&_substring(rendered, 0 as i64, 4 as i64)));
    let month: i64 =
        _int_or_negative_one(_parse_decimal(&_substring(rendered, 5 as i64, 7 as i64)));
    let day: i64 = _int_or_negative_one(_parse_decimal(&_substring(rendered, 8 as i64, 10 as i64)));
    let hour: i64 =
        _int_or_negative_one(_parse_decimal(&_substring(rendered, 11 as i64, 13 as i64)));
    let minute: i64 =
        _int_or_negative_one(_parse_decimal(&_substring(rendered, 14 as i64, 16 as i64)));
    let second: i64 =
        _int_or_negative_one(_parse_decimal(&_substring(rendered, 17 as i64, 19 as i64)));
    if (((((year < (0 as i64)) || (month < (0 as i64))) || (day < (0 as i64)))
        || (hour < (0 as i64)))
        || (minute < (0 as i64)))
        || (second < (0 as i64))
    {
        return _invalid_struct_time();
    }
    if !(_valid_date(year, month, day)) {
        return _invalid_struct_time();
    }
    let wday: i64 = _weekday(year, month, day);
    let yday: i64 = _day_of_year(year, month, day);
    return struct_time::new(year, month, day, hour, minute, second, wday, yday, 0 as i64);
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
                dt.with_timezone(&chrono::Local)
                    .format("%Y-%m-%dT%H:%M:%S")
                    .to_string()
            })
            .unwrap_or_default()
    };
    return _to_struct_time(&rendered);
}

// --- stdlib: sifr.bytes ---
fn decode_utf8(data: &Vec<u8>) -> Result<String, ParseError> {
    return String::from_utf8(data.iter().copied().collect::<Vec<u8>>()).map_err(|e| ParseError {
        message: e.to_string(),
    });
}
fn bytes_from_hex(s: &String) -> Result<Vec<u8>, ParseError> {
    return {
        let s: String = s.to_string();
        let mut cleaned = String::new();
        for ch in s.chars() {
            if ch.is_ascii_whitespace() {
                continue;
            }
            if !ch.is_ascii_hexdigit() {
                return Err(ParseError {
                    message: format!("invalid hex character: {}", ch),
                });
            }
            cleaned.push(ch);
        }
        if (cleaned.len() % 2) != 0 {
            return Err(ParseError {
                message: "fromhex() arg must contain an even number of hexadecimal digits"
                    .to_string()
                    .to_string(),
            });
        }
        let mut result = Vec::new();
        for pair in cleaned.as_bytes().chunks(2) {
            let pair_str = std::str::from_utf8(pair).map_err(|e| ParseError {
                message: e.to_string(),
            })?;
            result.push(u8::from_str_radix(pair_str, 16).map_err(|e| ParseError {
                message: e.to_string(),
            })?);
        }
        Ok(result)
    };
}
fn bytes_from_ints(values: &Vec<i64>) -> Result<Vec<u8>, ValueError> {
    return {
        let __vals = values;
        let mut __out = Vec::new();
        for __pair in __vals.iter().enumerate() {
            if (*__pair.1 < 0) || (*__pair.1 > 255) {
                return Err(ValueError {
                    message: format!("byte out of range at index {}: {}", __pair.0, *__pair.1),
                });
            }
            __out.push(*__pair.1 as u8);
        }
        Ok(__out)
    };
}
fn bytes_with_size(size: i64) -> Result<Vec<u8>, ValueError> {
    return {
        let __size = size;
        if __size < 0 {
            return Err(ValueError {
                message: "bytes(size) requires a non-negative size"
                    .to_string()
                    .to_string(),
            });
        }
        Ok((0..__size).map(|_| 0 as u8).collect::<Vec<u8>>())
    };
}
fn encode_utf8_result(s: &String) -> Result<Vec<u8>, ParseError> {
    return Ok({
        let __s = s;
        __s.as_bytes().to_vec()
    });
}
fn count_byte(data: &Vec<u8>, value: i64) -> i64 {
    let mut count: i64 = 0 as i64;
    for b in data.iter().map(|__byte| *__byte as i64) {
        if b == value {
            count = count + (1 as i64);
        }
    }
    return count;
}
fn find_byte(data: &Vec<u8>, value: i64) -> Option<i64> {
    let mut idx: i64 = 0 as i64;
    for b in data.iter().map(|__byte| *__byte as i64) {
        if b == value {
            return Some(idx);
        }
        idx = idx + (1 as i64);
    }
    return None;
}
fn starts_with(data: &Vec<u8>, prefix: &Vec<u8>) -> bool {
    if (prefix.len() as i64) > (data.len() as i64) {
        return false;
    }
    let mut i: i64 = 0 as i64;
    while i < (prefix.len() as i64) {
        let a: Option<i64> = data.get(i as usize).map(|__byte| *__byte as i64);
        let b: Option<i64> = prefix.get(i as usize).map(|__byte| *__byte as i64);
        let Some(a) = a else {
            return false;
        };
        let Some(b) = b else {
            return false;
        };
        if a != b {
            return false;
        }
        i = i + (1 as i64);
    }
    return true;
}
fn ends_with(data: &Vec<u8>, suffix: &Vec<u8>) -> bool {
    if (suffix.len() as i64) > (data.len() as i64) {
        return false;
    }
    let offset: i64 = (data.len() as i64) - (suffix.len() as i64);
    let mut i: i64 = 0 as i64;
    while i < (suffix.len() as i64) {
        let a: Option<i64> = data.get((offset + i) as usize).map(|__byte| *__byte as i64);
        let b: Option<i64> = suffix.get(i as usize).map(|__byte| *__byte as i64);
        let Some(a) = a else {
            return false;
        };
        let Some(b) = b else {
            return false;
        };
        if a != b {
            return false;
        }
        i = i + (1 as i64);
    }
    return true;
}

// --- stdlib: sifr.platform ---
fn system() -> String {
    return if cfg!(target_os = "windows") {
        "Windows".to_string().to_string()
    } else {
        if cfg!(target_os = "macos") {
            "Darwin".to_string().to_string()
        } else {
            if cfg!(target_os = "linux") {
                "Linux".to_string().to_string()
            } else {
                std::env::consts::OS.to_string()
            }
        }
    };
}
fn machine() -> String {
    return std::env::consts::ARCH.to_string();
}
fn processor() -> String {
    return std::env::consts::ARCH.to_string();
}

#[derive(Debug, Clone)]
struct IOError {
    message: String,
    kind: String,
}

impl IOError {
    fn new(message: String) -> Self {
        return Self {
            message: message,
            kind: "Other".to_string(),
        };
    }
}

impl std::fmt::Display for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for IOError {}

fn __io_err(e: std::io::Error) -> IOError {
    let msg = e.to_string();
    let kind = if e.kind() == std::io::ErrorKind::NotFound {
        "FileNotFound".to_string()
    } else {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            "PermissionDenied".to_string()
        } else {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                "FileExists".to_string()
            } else {
                "Other".to_string()
            }
        }
    };
    return IOError {
        message: msg,
        kind: kind,
    };
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

impl std::error::Error for ParseError {}

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

impl std::error::Error for ValueError {}

#[derive(Debug, Clone)]
struct JSONDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl JSONDecodeError {
    fn new(message: String) -> Self {
        return Self {
            message: message,
            line: 0,
            column: 0,
        };
    }
}

impl std::fmt::Display for JSONDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for JSONDecodeError {}

#[derive(Debug, Clone)]
struct TOMLDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl TOMLDecodeError {
    fn new(message: String) -> Self {
        return Self {
            message: message,
            line: 0,
            column: 0,
        };
    }
}

impl std::fmt::Display for TOMLDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for TOMLDecodeError {}

#[derive(Debug, Clone)]
struct RegexError {
    message: String,
    detail: String,
}

impl RegexError {
    fn new(message: String) -> Self {
        return Self {
            message: message,
            detail: String::new(),
        };
    }
}

impl std::fmt::Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for RegexError {}

fn demo_math() {
    println!("=== math new intrinsics ===");
    let e0: f64 = {
        let __x: f64 = (0.0 as f64) as f64;
        let __t: f64 = 1.0 / (1.0 + (0.3275911 * __x.abs()));
        let __poly: f64 = __t
            * (0.254829592
                + (__t
                    * (-0.284496736
                        + (__t * (1.421413741 + (__t * (-1.453152027 + (__t * 1.061405429))))))));
        let __r: f64 = 1.0 - (__poly * (-__x * __x).exp());
        if __x >= 0.0 {
            __r
        } else {
            -__r
        }
    };
    println!(
        "{}",
        format!(
            "{}{}",
            "erf near 0 = ".to_string(),
            format!("{}", (e0 < (0.001 as f64)) && (e0 > -(0.001 as f64)))
        )
    );
    let ec0: f64 = {
        let __x: f64 = (0.0 as f64) as f64;
        let __t: f64 = 1.0 / (1.0 + (0.3275911 * __x.abs()));
        let __poly: f64 = __t
            * (0.254829592
                + (__t
                    * (-0.284496736
                        + (__t * (1.421413741 + (__t * (-1.453152027 + (__t * 1.061405429))))))));
        let __r: f64 = __poly * (-__x * __x).exp();
        if __x >= 0.0 {
            __r
        } else {
            2.0 - __r
        }
    };
    println!(
        "{}",
        format!(
            "{}{}",
            "erfc near 1 = ".to_string(),
            format!("{}", (ec0 > (0.999 as f64)) && (ec0 < (1.001 as f64)))
        )
    );
    let g: f64 = {
        let __x: f64 = (5.0 as f64) as f64;
        let __g: usize = 7 as usize;
        let __c = vec![
            0.9999999999998099,
            676.5203681218851,
            -1259.1392167224028,
            771.3234287776531,
            -176.6150291621406,
            12.507343278686905,
            -0.13857109526572012,
            0.000009984369578019572,
            0.00000015056327351493116,
        ];
        if (__x <= 0.0) && (__x == __x.floor()) {
            f64::INFINITY
        } else {
            if __x < 0.5 {
                {
                    let __xn: f64 = 1.0 - __x;
                    let mut __s: f64 = __c[0];
                    for __i in 1..__g + 2 {
                        __s += __c[__i] / ((__xn + (__i as f64)) - 1.0);
                    }
                    let __t2: f64 = (__xn + (__g as f64)) - 0.5;
                    let __base: f64 = (((2.0 * std::f64::consts::PI).sqrt()
                        * __t2.powf(__xn - 0.5))
                        * (0.0 - __t2).exp())
                        * __s;
                    std::f64::consts::PI / ((__x * std::f64::consts::PI).sin() * __base)
                }
            } else {
                {
                    let __xm: f64 = __x - 1.0;
                    let mut __s: f64 = __c[0];
                    for __i in 1..__g + 2 {
                        __s += __c[__i] / (__xm + (__i as f64));
                    }
                    let __t2: f64 = (__xm + (__g as f64)) + 0.5;
                    (((2.0 * std::f64::consts::PI).sqrt() * __t2.powf(__xm + 0.5))
                        * (0.0 - __t2).exp())
                        * __s
                }
            }
        }
    };
    println!(
        "{}",
        format!(
            "{}{}",
            "gamma(5) > 23 = ".to_string(),
            format!("{}", g > (23.0 as f64))
        )
    );
    let lg: f64 = {
        let __x: f64 = (5.0 as f64) as f64;
        let __g: usize = 7;
        let __c = vec![
            0.9999999999998099,
            676.5203681218851,
            -1259.1392167224028,
            771.3234287776531,
            -176.6150291621406,
            12.507343278686905,
            -0.13857109526572012,
            0.000009984369578019572,
            0.00000015056327351493116,
        ];
        if (__x <= 0.0) && (__x == __x.floor()) {
            f64::INFINITY
        } else {
            {
                let __xm: f64 = if __x < 0.5 { 1.0 - __x } else { __x - 1.0 };
                let mut __s: f64 = __c[0];
                for __i in 1..__g + 2 {
                    __s += __c[__i] / (__xm + (__i as f64));
                }
                let __t2: f64 = (__xm + (__g as f64)) + 0.5;
                let __r: f64 = (((2.0 * std::f64::consts::PI).sqrt().ln()
                    + ((__xm + 0.5) * __t2.ln()))
                    - __t2)
                    + __s.ln();
                if __x < 0.5 {
                    (std::f64::consts::PI / ((__x * std::f64::consts::PI).sin() * __r.exp()))
                        .abs()
                        .ln()
                } else {
                    __r
                }
            }
        }
    };
    println!(
        "{}",
        format!(
            "{}{}",
            "lgamma(5) > 3 = ".to_string(),
            format!("{}", lg > (3.0 as f64))
        )
    );
    let fp: Vec<f64> = {
        let __x: f64 = (8.0 as f64) as f64;
        if __x == 0.0 {
            vec![__x, 0.0]
        } else {
            if !__x.is_finite() {
                vec![__x, 0.0]
            } else {
                {
                    let __bits: u64 = __x.to_bits();
                    let __sign_mask: u64 = (1 as u64) << 63;
                    let __frac_mask: u64 = ((1 as u64) << 52) - (1 as u64);
                    let __sign: u64 = __bits & __sign_mask;
                    let __exp: i32 = ((__bits >> 52) & (2047 as u64)) as i32;
                    let __frac: u64 = __bits & __frac_mask;
                    if __exp == 0 {
                        {
                            let __scaled: f64 = __x * (2.0 as f64).powi(54);
                            let __sbits: u64 = __scaled.to_bits();
                            let __sexp: i32 = ((__sbits >> 52) & (2047 as u64)) as i32;
                            let __sfrac: u64 = __sbits & __frac_mask;
                            let __mant: f64 =
                                f64::from_bits((__sign | ((1022 as u64) << 52)) | __sfrac);
                            let __e: i32 = (__sexp - 1022) - 54;
                            vec![__mant, __e as f64]
                        }
                    } else {
                        {
                            let __mant: f64 =
                                f64::from_bits((__sign | ((1022 as u64) << 52)) | __frac);
                            let __e: i32 = __exp - 1022;
                            vec![__mant, __e as f64]
                        }
                    }
                }
            }
        }
    };
    let mantissa: Option<f64> = {
        let __sifr_index_list = &fp;
        let __sifr_index_i = 0 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    if let Some(mantissa) = mantissa {
        println!(
            "{}",
            format!(
                "{}{}",
                "frexp(8.0) mantissa = ".to_string(),
                format!("{}", mantissa)
            )
        );
    }
    let ld: f64 = ((0.5 as f64) as f64) * (2.0 as f64).powi((4 as i64) as i32);
    println!(
        "{}",
        format!("{}{}", "ldexp(0.5, 4) = ".to_string(), format!("{}", ld))
    );
    let md: Vec<f64> = {
        let __x: f64 = (3.7 as f64) as f64;
        if __x.is_nan() {
            vec![f64::NAN, f64::NAN]
        } else {
            if __x.is_infinite() {
                vec![(0.0 as f64).copysign(__x), __x]
            } else {
                {
                    let __int = __x.trunc();
                    let mut __frac = __x - __int;
                    if __frac == 0.0 {
                        __frac = (0.0 as f64).copysign(__x);
                    }
                    vec![__frac, __int]
                }
            }
        }
    };
    let frac: Option<f64> = {
        let __sifr_index_list = &md;
        let __sifr_index_i = 0 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    if let Some(frac) = frac {
        println!(
            "{}",
            format!(
                "{}{}",
                "modf(3.7) frac > 0 = ".to_string(),
                format!("{}", frac > (0.0 as f64))
            )
        );
    }
    let na: f64 = {
        let __x: f64 = (1.0 as f64) as f64;
        let __y: f64 = (2.0 as f64) as f64;
        if __x.is_nan() || __y.is_nan() {
            f64::NAN
        } else {
            if __x == __y {
                __y
            } else {
                if __x == 0.0 {
                    {
                        let __sign: u64 = if __y.is_sign_negative() {
                            (1 as u64) << 63
                        } else {
                            0 as u64
                        };
                        f64::from_bits(__sign | (1 as u64))
                    }
                } else {
                    {
                        let mut __bits: u64 = __x.to_bits();
                        if (__x < __y) == (__x > 0.0) {
                            __bits += 1 as u64;
                        } else {
                            __bits -= 1 as u64;
                        }
                        f64::from_bits(__bits)
                    }
                }
            }
        }
    };
    println!(
        "{}",
        format!(
            "{}{}",
            "nextafter(1.0, 2.0) > 1.0 = ".to_string(),
            format!("{}", na > (1.0 as f64))
        )
    );
    let u: f64 = {
        let __x: f64 = (1.0 as f64) as f64;
        if __x.is_nan() {
            f64::NAN
        } else {
            if __x.is_infinite() {
                f64::INFINITY
            } else {
                {
                    let __a = __x.abs();
                    if __a == 0.0 {
                        f64::from_bits(1 as u64)
                    } else {
                        if __a == f64::MAX {
                            __a - f64::from_bits(__a.to_bits() - (1 as u64))
                        } else {
                            f64::from_bits(__a.to_bits() + (1 as u64)) - __a
                        }
                    }
                }
            }
        }
    };
    println!(
        "{}",
        format!(
            "{}{}",
            "ulp(1.0) > 0 = ".to_string(),
            format!("{}", u > (0.0 as f64))
        )
    );
}

fn demo_os() {
    println!("=== os new intrinsics ===");
    let pid: i64 = std::process::id() as i64;
    println!(
        "{}",
        format!(
            "{}{}",
            "pid > 0 = ".to_string(),
            format!("{}", pid > (0 as i64))
        )
    );
    let cpus: i64 = {
        let __n = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        __n as i64
    };
    println!(
        "{}",
        format!(
            "{}{}",
            "cpu_count >= 1 = ".to_string(),
            format!("{}", cpus >= (1 as i64))
        )
    );
}

fn demo_hashlib() {
    println!("=== hashlib new intrinsics ===");
    let s: String = "hello".to_string();
    println!(
        "{}",
        format!(
            "{}{}",
            "sha224 len = ".to_string(),
            format!(
                "{}",
                format!(
                    "{:x}",
                    (<sha2::Sha224 as sha2::Digest>::digest)((s).as_bytes())
                )
                .chars()
                .count() as i64
            )
        )
    );
    println!(
        "{}",
        format!(
            "{}{}",
            "sha384 len = ".to_string(),
            format!(
                "{}",
                format!(
                    "{:x}",
                    (<sha2::Sha384 as sha2::Digest>::digest)((s).as_bytes())
                )
                .chars()
                .count() as i64
            )
        )
    );
    println!(
        "{}",
        format!(
            "{}{}",
            "blake2b len = ".to_string(),
            format!(
                "{}",
                format!(
                    "{:x}",
                    (<blake2::Blake2b512 as blake2::Digest>::digest)((s).as_bytes())
                )
                .chars()
                .count() as i64
            )
        )
    );
    println!(
        "{}",
        format!(
            "{}{}",
            "blake2s len = ".to_string(),
            format!(
                "{}",
                format!(
                    "{:x}",
                    (<blake2::Blake2s256 as blake2::Digest>::digest)((s).as_bytes())
                )
                .chars()
                .count() as i64
            )
        )
    );
}

fn demo_platform() {
    println!("=== platform new intrinsics ===");
    println!(
        "{}",
        format!(
            "{}{}",
            "system len > 0 = ".to_string(),
            format!("{}", (system().chars().count() as i64) > (0 as i64))
        )
    );
    println!(
        "{}",
        format!(
            "{}{}",
            "machine len > 0 = ".to_string(),
            format!("{}", (machine().chars().count() as i64) > (0 as i64))
        )
    );
    println!(
        "{}",
        format!(
            "{}{}",
            "processor len > 0 = ".to_string(),
            format!("{}", (processor().chars().count() as i64) > (0 as i64))
        )
    );
}

fn demo_time() {
    println!("=== time new intrinsics ===");
    let gmt: struct_time = gmtime_struct(0.0 as f64);
    println!(
        "{}",
        format!(
            "{}{}",
            "gmtime year = ".to_string(),
            format!("{}", gmt.tm_year == (1970 as i64))
        )
    );
    let lt: struct_time = localtime_struct(0.0 as f64);
    println!(
        "{}",
        format!(
            "{}{}",
            "localtime yday >= 1 = ".to_string(),
            format!("{}", lt.tm_yday >= (1 as i64))
        )
    );
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let parsed: String = chrono::NaiveDateTime::parse_from_str(
            &"2024-01-15 10:30:00".to_string(),
            &"%Y-%m-%d %H:%M:%S".to_string(),
        )
        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S").to_string())
        .map_err(|e| ValueError {
            message: e.to_string(),
        })?;
        println!(
            "{}",
            format!(
                "{}{}",
                "strptime ok = ".to_string(),
                format!("{}", (parsed.chars().count() as i64) > (0 as i64))
            )
        );
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}",
            format!("{}{}", "strptime error: ".to_string(), e.message)
        );
    }
}

fn demo_base64() {
    println!("=== base64 new intrinsics ===");
    let encoded: String = {
        let __b32_alpha = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
        let __s = "hello world".to_string();
        let __data = __s.as_bytes();
        let mut __out = String::new();
        let mut __i = 0 as usize;
        while __i < __data.len() {
            let __b0 = __data[__i] as i64;
            let __b1 = if (__i + 1) < __data.len() {
                __data[__i + 1] as i64
            } else {
                0
            };
            let __b2 = if (__i + 2) < __data.len() {
                __data[__i + 2] as i64
            } else {
                0
            };
            let __b3 = if (__i + 3) < __data.len() {
                __data[__i + 3] as i64
            } else {
                0
            };
            let __b4 = if (__i + 4) < __data.len() {
                __data[__i + 4] as i64
            } else {
                0
            };
            let __buf = ((((__b0 << 32) | (__b1 << 24)) | (__b2 << 16)) | (__b3 << 8)) | __b4;
            let __remaining = __data.len() - __i;
            let __n = if __remaining < 5 { __remaining } else { 5 };
            for __j in 0..8 {
                if __j < (((__n * 8) + 4) / 5) {
                    __out.push(
                        __b32_alpha[((__buf >> ((35 - (__j * 5)) as usize)) & 31) as usize] as char,
                    );
                } else {
                    __out.push('=');
                }
            }
            __i += 5;
        }
        __out
    };
    println!(
        "{}",
        format!(
            "{}{}",
            "b32encode len > 0 = ".to_string(),
            format!("{}", (encoded.chars().count() as i64) > (0 as i64))
        )
    );
    let __sifr_try_res: Result<(), ParseError> = (|| {
        let decoded: String = ({
            let __b32_alpha = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
            let __s_val = encoded;
            let __s = __s_val.trim_end_matches('=');
            let mut __bits = 0;
            let mut __bit_count = 0;
            let mut __out: Vec<u8> = Vec::new();
            for __c in __s.chars() {
                let __val_opt = __b32_alpha
                    .iter()
                    .position(|b| (*b as char) == __c.to_ascii_uppercase());
                let mut __val = 0;
                if let Some(__v) = __val_opt {
                    __val = __v as i64;
                } else {
                    return Err(ParseError {
                        message: format!("invalid base32 char: {}", __c),
                    });
                }
                __bits = (__bits << 5) | __val;
                __bit_count += 5;
                if __bit_count >= 8 {
                    __bit_count -= 8;
                    __out.push(((__bits >> (__bit_count as usize)) & 255) as u8);
                }
            }
            String::from_utf8(__out).map_err(|e| ParseError {
                message: e.to_string(),
            })
        })?;
        println!("{}", format!("{}{}", "b32decode = ".to_string(), decoded));
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}",
            format!("{}{}", "b32decode error: ".to_string(), e.message)
        );
    }
}

fn demo_shutil() {
    println!("=== shutil new intrinsics ===");
    let usage: Vec<i64> = {
        let __path = "/".to_string();
        let __meta_ok = std::fs::metadata(&__path).is_ok();
        if __meta_ok {
            {
                let __out = std::process::Command::new("df".to_string())
                    .arg("-k".to_string())
                    .arg(&__path)
                    .output();
                {
                    let __s = __out.as_ref().map_or("".to_string(), |__o| {
                        String::from_utf8_lossy(&__o.stdout).to_string()
                    });
                    let __lines = __s.lines().collect::<Vec<&str>>();
                    if __lines.len() >= 2 {
                        {
                            let __parts = __lines[1].split_whitespace().collect::<Vec<&str>>();
                            if __parts.len() >= 4 {
                                {
                                    let __total = __parts[1].parse::<i64>().unwrap_or(0) * 1024;
                                    let __used = __parts[2].parse::<i64>().unwrap_or(0) * 1024;
                                    let __free = __parts[3].parse::<i64>().unwrap_or(0) * 1024;
                                    vec![__total, __used, __free]
                                }
                            } else {
                                vec![0, 0, 0]
                            }
                        }
                    } else {
                        vec![0, 0, 0]
                    }
                }
            }
        } else {
            vec![0, 0, 0]
        }
    };
    let total: Option<i64> = {
        let __sifr_index_list = &usage;
        let __sifr_index_i = 0 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    if let Some(total) = total {
        println!(
            "{}",
            format!(
                "{}{}",
                "disk_total > 0 = ".to_string(),
                format!("{}", total > (0 as i64))
            )
        );
    }
}

fn main() {
    demo_math();
    demo_os();
    demo_hashlib();
    demo_platform();
    demo_time();
    demo_base64();
    demo_shutil();
}
