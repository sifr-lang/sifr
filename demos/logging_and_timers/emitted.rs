use std::sync::Mutex;

// --- stdlib: sifr.time ---
const TIMEZONE: i64 = 0 as i64;
fn __const_TZNAME() -> (String, String) {
    return ("UTC".to_string(), "UTC".to_string());
}
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
fn gmtime_struct(epoch: f64) -> struct_time {
    let rendered: String = {
        let __ts = epoch as i64;
        chrono::DateTime::<chrono::Utc>::from_timestamp(__ts, 0)
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S").to_string())
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

// --- stdlib: sifr.timeit ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Timer {}
impl Timer {
    fn new() -> Self {
        return Self {};
    }
    fn timeit(&self, stmt: impl Fn(), number: i64) -> f64 {
        return timeit(stmt, number);
    }
    fn repeat(&self, stmt: impl Fn(), count: i64, number: i64) -> Vec<f64> {
        return repeat(stmt, count, number);
    }
    fn __call__(&self, stmt: impl Fn(), number: i64) -> f64 {
        return self.timeit(stmt, number);
    }
}
fn _elapsed_non_negative(start: f64, end: f64) -> f64 {
    let elapsed: f64 = end - start;
    if elapsed < (0.0 as f64) {
        return 0.0 as f64;
    }
    return elapsed;
}
fn timeit(stmt: impl Fn(), number: i64) -> f64 {
    let start: f64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    let mut i: i64 = 0 as i64;
    while i < number {
        stmt();
        i = i + (1 as i64);
    }
    let end: f64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    return _elapsed_non_negative(start, end);
}
fn repeat(stmt: impl Fn(), count: i64, number: i64) -> Vec<f64> {
    let mut results: Vec<f64> = vec![];
    let mut r: i64 = 0 as i64;
    while r < count {
        let start: f64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        let mut i: i64 = 0 as i64;
        while i < number {
            stmt();
            i = i + (1 as i64);
        }
        let end: f64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        let elapsed: f64 = _elapsed_non_negative(start, end);
        results.push(elapsed);
        r = r + (1 as i64);
    }
    return results;
}

// --- stdlib: sifr.logging ---
#[derive(Debug, Clone)]
struct FileHandle {
    _handle: i64,
    _mode: String,
}
impl FileHandle {
    fn new(_handle: i64, _mode: String) -> Self {
        return Self {
            _handle: _handle,
            _mode: _mode,
        };
    }
    fn read(&self) -> Result<String, IOError> {
        let __hid = self._handle;
        let mut __handles = __SIFR_FILE_HANDLES
            .lock()
            .unwrap_or_else(|__err| __err.into_inner());
        match __handles.get_mut(&__hid) {
            Some(SifrFileHandle::TextRead(ref mut __r)) => {
                let mut __s = String::new();
                std::io::Read::read_to_string(__r, &mut __s).map_err(__io_err)?;
                return Ok(__s);
            }
            _ => {
                return Err(IOError {
                    message: "file not open for reading".to_string(),
                    kind: "Other".to_string(),
                });
            }
        }
    }
    fn write(&self, data: &String) -> Result<(), IOError> {
        let __hid = self._handle;
        let mut __handles = __SIFR_FILE_HANDLES
            .lock()
            .unwrap_or_else(|__err| __err.into_inner());
        match __handles.get_mut(&__hid) {
            Some(SifrFileHandle::TextWrite(ref mut __w)) => {
                std::io::Write::write_all(__w, data.as_bytes()).map_err(__io_err)?;
                return Ok(());
            }
            _ => {
                return Err(IOError {
                    message: "file not open for writing".to_string(),
                    kind: "Other".to_string(),
                });
            }
        }
    }
    fn readline(&self) -> Result<Option<String>, IOError> {
        let __hid = self._handle;
        let mut __handles = __SIFR_FILE_HANDLES
            .lock()
            .unwrap_or_else(|__err| __err.into_inner());
        match __handles.get_mut(&__hid) {
            Some(SifrFileHandle::TextRead(ref mut __r)) => {
                let mut __line = String::new();
                let __n = std::io::BufRead::read_line(__r, &mut __line)
                    .map_err(__io_err)?;
                if __n == 0 {
                    return Ok(None);
                }
                if __line.ends_with('\n') {
                    __line.pop();
                    if __line.ends_with('\r') {
                        __line.pop();
                    }
                }
                return Ok(Some(__line));
            }
            _ => {
                return Err(IOError {
                    message: "file not open for reading".to_string(),
                    kind: "Other".to_string(),
                });
            }
        }
    }
    fn readlines(&self) -> Result<Vec<String>, IOError> {
        let __hid = self._handle;
        let mut __handles = __SIFR_FILE_HANDLES
            .lock()
            .unwrap_or_else(|__err| __err.into_inner());
        match __handles.get_mut(&__hid) {
            Some(SifrFileHandle::TextRead(ref mut __r)) => {
                let mut __lines: Vec<String> = Vec::<String>::new();
                let mut __line = String::new();
                loop {
                    __line.clear();
                    let __n = std::io::BufRead::read_line(__r, &mut __line)
                        .map_err(__io_err)?;
                    if __n == 0 {
                        break;
                    }
                    let mut __l = __line.clone();
                    if __l.ends_with('\n') {
                        __l.pop();
                        if __l.ends_with('\r') {
                            __l.pop();
                        }
                    }
                    __lines.push(__l);
                }
                return Ok(__lines);
            }
            _ => {
                return Err(IOError {
                    message: "file not open for reading".to_string(),
                    kind: "Other".to_string(),
                });
            }
        }
    }
    fn close(&self) {
        let __hid = self._handle;
        __SIFR_FILE_HANDLES
            .lock()
            .unwrap_or_else(|__err| __err.into_inner())
            .remove(&__hid);
    }
    fn read_bytes(&self) -> Result<Vec<u8>, IOError> {
        let __hid = self._handle;
        let mut __handles = __SIFR_FILE_HANDLES
            .lock()
            .unwrap_or_else(|__err| __err.into_inner());
        match __handles.get_mut(&__hid) {
            Some(SifrFileHandle::BinaryRead(ref mut __r)) => {
                let mut __buf = Vec::<u8>::new();
                std::io::Read::read_to_end(__r, &mut __buf).map_err(__io_err)?;
                return Ok(__buf);
            }
            _ => {
                return Err(IOError {
                    message: "file not open for binary reading".to_string(),
                    kind: "Other".to_string(),
                });
            }
        }
    }
    fn write_bytes(&self, data: &Vec<u8>) -> Result<(), IOError> {
        let __hid = self._handle;
        let mut __handles = __SIFR_FILE_HANDLES
            .lock()
            .unwrap_or_else(|__err| __err.into_inner());
        match __handles.get_mut(&__hid) {
            Some(SifrFileHandle::BinaryWrite(ref mut __w)) => {
                std::io::Write::write_all(__w, &data).map_err(__io_err)?;
                return Ok(());
            }
            _ => {
                return Err(IOError {
                    message: "file not open for binary writing".to_string(),
                    kind: "Other".to_string(),
                });
            }
        }
    }
    fn __enter__(&self) -> &Self {
        return self;
    }
    fn __exit__(&self) {
        self.close();
    }
}
const DEBUG: i64 = 10 as i64;
const INFO: i64 = 20 as i64;
const WARNING: i64 = 30 as i64;
const ERROR: i64 = 40 as i64;
const CRITICAL: i64 = 50 as i64;
const NOTSET: i64 = 0 as i64;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Formatter {
    _fmt: String,
}
impl Formatter {
    fn new(fmt: String) -> Self {
        return Self { _fmt: fmt };
    }
    fn template(&self) -> String {
        return self._fmt.clone();
    }
    fn format(&self, level: &String, name: &String, msg: &String) -> String {
        let mut result: String = self._fmt.clone();
        result = result.replace(&"%(levelname)s".to_string(), &level);
        result = result.replace(&"%(name)s".to_string(), &name);
        result = result.replace(&"%(message)s".to_string(), &msg);
        return result;
    }
}
impl std::fmt::Display for Formatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Formatter(_fmt={})", self._fmt);
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct StreamHandler {
    _level: i64,
    _formatter: Formatter,
}
impl StreamHandler {
    fn new(level: i64) -> Self {
        return Self {
            _level: level,
            _formatter: Formatter::new("%(levelname)s:%(name)s:%(message)s".to_string()),
        };
    }
    fn set_level(&mut self, level: i64) {
        self._level = level;
    }
    fn level(&self) -> i64 {
        return self._level;
    }
    fn set_formatter(&mut self, fmt: &Formatter) {
        self._formatter = Formatter::new(format!("{}{}", fmt._fmt, "".to_string()));
    }
    fn format_template(&mut self) -> String {
        return self._formatter.clone().template();
    }
    fn _allows(&self, level_num: i64) -> bool {
        if self._level == NOTSET {
            return true;
        }
        return level_num >= self._level;
    }
    fn emit(&mut self, level: &String, name: &String, msg: &String) {
        let level_num: i64 = _level_name_to_num(level);
        if !(self._allows(level_num)) {
            return;
        }
        let line: String = self._formatter.clone().format(level, name, msg);
        println!("{}", line);
    }
}
impl std::fmt::Display for StreamHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f, "StreamHandler(_level={}, _formatter={})", self._level, self._formatter
        );
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FileHandler {
    _path: String,
    _level: i64,
    _formatter: Formatter,
}
impl FileHandler {
    fn new(path: String, level: i64) -> Self {
        return Self {
            _path: format!("{}{}", path, "".to_string()),
            _level: level,
            _formatter: Formatter::new("%(levelname)s:%(name)s:%(message)s".to_string()),
        };
    }
    fn path(&self) -> String {
        return self._path.clone();
    }
    fn set_level(&mut self, level: i64) {
        self._level = level;
    }
    fn level(&self) -> i64 {
        return self._level;
    }
    fn set_formatter(&mut self, fmt: &Formatter) {
        self._formatter = Formatter::new(format!("{}{}", fmt._fmt, "".to_string()));
    }
    fn format_template(&mut self) -> String {
        return self._formatter.clone().template();
    }
    fn _allows(&self, level_num: i64) -> bool {
        if self._level == NOTSET {
            return true;
        }
        return level_num >= self._level;
    }
    fn emit(&mut self, level: &String, name: &String, msg: &String) {
        let level_num: i64 = _level_name_to_num(level);
        if !(self._allows(level_num)) {
            return;
        }
        let line: String = format!(
            "{}{}", self._formatter.clone().format(level, name, msg), "\n".to_string()
        );
        let __sifr_try_res: Result<(), IOError> = (|| {
            let mut fh: FileHandle = (|| {
                let __path = self._path.clone().to_string();
                let __mode = "a".to_string().to_string();
                let __handle_id = __sifr_next_file_handle_id();
                match __mode.as_str() {
                    "r" | "rt" => {
                        let __f = std::fs::File::open(__path.as_str())
                            .map_err(__io_err)?;
                        let __reader = std::io::BufReader::new(__f);
                        __SIFR_FILE_HANDLES
                            .lock()
                            .unwrap_or_else(|__err| __err.into_inner())
                            .insert(__handle_id, SifrFileHandle::TextRead(__reader));
                        return Ok(FileHandle {
                            _handle: __handle_id,
                            _mode: __mode.to_string(),
                        });
                    }
                    "w" | "wt" => {
                        let __f = std::fs::File::create(__path.as_str())
                            .map_err(__io_err)?;
                        let __writer = std::io::BufWriter::new(__f);
                        __SIFR_FILE_HANDLES
                            .lock()
                            .unwrap_or_else(|__err| __err.into_inner())
                            .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                        return Ok(FileHandle {
                            _handle: __handle_id,
                            _mode: __mode.to_string(),
                        });
                    }
                    "a" | "at" => {
                        let __f = std::fs::OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open(__path.as_str())
                            .map_err(__io_err)?;
                        let __writer = std::io::BufWriter::new(__f);
                        __SIFR_FILE_HANDLES
                            .lock()
                            .unwrap_or_else(|__err| __err.into_inner())
                            .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                        return Ok(FileHandle {
                            _handle: __handle_id,
                            _mode: __mode.to_string(),
                        });
                    }
                    "rb" => {
                        let __f = std::fs::File::open(__path.as_str())
                            .map_err(__io_err)?;
                        let __reader = std::io::BufReader::new(__f);
                        __SIFR_FILE_HANDLES
                            .lock()
                            .unwrap_or_else(|__err| __err.into_inner())
                            .insert(__handle_id, SifrFileHandle::BinaryRead(__reader));
                        return Ok(FileHandle {
                            _handle: __handle_id,
                            _mode: __mode.to_string(),
                        });
                    }
                    "wb" => {
                        let __f = std::fs::File::create(__path.as_str())
                            .map_err(__io_err)?;
                        let __writer = std::io::BufWriter::new(__f);
                        __SIFR_FILE_HANDLES
                            .lock()
                            .unwrap_or_else(|__err| __err.into_inner())
                            .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                        return Ok(FileHandle {
                            _handle: __handle_id,
                            _mode: __mode.to_string(),
                        });
                    }
                    "ab" => {
                        let __f = std::fs::OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open(__path.as_str())
                            .map_err(__io_err)?;
                        let __writer = std::io::BufWriter::new(__f);
                        __SIFR_FILE_HANDLES
                            .lock()
                            .unwrap_or_else(|__err| __err.into_inner())
                            .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                        return Ok(FileHandle {
                            _handle: __handle_id,
                            _mode: __mode.to_string(),
                        });
                    }
                    _ => {
                        return Err(IOError {
                            message: format!("invalid mode: {}", __mode),
                            kind: "Other".to_string(),
                        });
                    }
                }
            })()?;
            let __sifr_try_res: Result<(), IOError> = (|| {
                let _: () = fh.write(&line)?;
                return Ok(());
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e2 = __sifr_try_err.clone();
                let _: String = e2.message;
            }
            fh.close();
            return Ok(());
        })();
        if let Err(__sifr_try_err) = __sifr_try_res {
            let e = __sifr_try_err.clone();
            let _: String = e.message;
        }
    }
}
impl std::fmt::Display for FileHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f, "FileHandler(_path={}, _level={}, _formatter={})", self._path, self
            ._level, self._formatter
        );
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct NullHandler {
    _level: i64,
    _formatter: Formatter,
}
impl NullHandler {
    fn new(level: i64) -> Self {
        return Self {
            _level: level,
            _formatter: Formatter::new("%(levelname)s:%(name)s:%(message)s".to_string()),
        };
    }
    fn set_level(&mut self, level: i64) {
        self._level = level;
    }
    fn level(&self) -> i64 {
        return self._level;
    }
    fn set_formatter(&mut self, fmt: &Formatter) {
        self._formatter = Formatter::new(format!("{}{}", fmt._fmt, "".to_string()));
    }
    fn format_template(&mut self) -> String {
        return self._formatter.clone().template();
    }
    fn emit(&self, level: &String, name: &String, msg: &String) {
        let _: String = (level).clone();
        let _: String = (name).clone();
        let _: String = (msg).clone();
    }
}
impl std::fmt::Display for NullHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f, "NullHandler(_level={}, _formatter={})", self._level, self._formatter
        );
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Logger {
    _name: String,
    _level: i64,
    _log_path: String,
    _handler_kind: String,
    _handler_path: String,
    _handler_level: i64,
    _handler_fmt: String,
}
impl Logger {
    fn new(name: String, level: i64) -> Self {
        return Self {
            _name: name,
            _level: level,
            _log_path: "".to_string(),
            _handler_kind: "".to_string(),
            _handler_path: "".to_string(),
            _handler_level: NOTSET,
            _handler_fmt: "%(levelname)s:%(name)s:%(message)s".to_string(),
        };
    }
    fn set_level(&mut self, level: i64) {
        self._level = level;
    }
    fn set_file(&mut self, path: &String) {
        self._log_path = format!("{}{}", path, "".to_string());
    }
    fn add_handler(&mut self, handler: &FileHandler) {
        self._handler_kind = "file".to_string();
        self._handler_path = handler.path();
        self._handler_level = handler.level();
        self._handler_fmt = "%(levelname)s:%(name)s:%(message)s".to_string();
    }
    fn set_stream_handler(&mut self, handler: &StreamHandler) {
        self._handler_kind = "stream".to_string();
        self._handler_path = "".to_string();
        self._handler_level = handler.level();
        self._handler_fmt = "%(levelname)s:%(name)s:%(message)s".to_string();
    }
    fn set_null_handler(&mut self, handler: &NullHandler) {
        self._handler_kind = "null".to_string();
        self._handler_path = "".to_string();
        self._handler_level = handler.level();
        self._handler_fmt = "%(levelname)s:%(name)s:%(message)s".to_string();
    }
    fn clear_handler(&mut self) {
        self._handler_kind = "".to_string();
        self._handler_path = "".to_string();
        self._handler_level = NOTSET;
        self._handler_fmt = "%(levelname)s:%(name)s:%(message)s".to_string();
    }
    fn _handler_allows(&self, level_num: i64) -> bool {
        if self._handler_level == NOTSET {
            return true;
        }
        return level_num >= self._handler_level;
    }
    fn _handler_line(&self, level: &String, msg: &String) -> String {
        let mut formatter: Formatter = Formatter::new(self._handler_fmt.clone());
        return formatter.format(level, &self._name.clone(), msg);
    }
    fn _emit(&self, level: &String, level_num: i64, msg: &String) {
        if self._level > level_num {
            return;
        }
        if self._handler_kind.clone() == "null".to_string() {
            return;
        }
        if self._handler_kind.clone() == "stream".to_string() {
            if self._handler_allows(level_num) {
                println!("{}", self._handler_line(level, msg));
            }
            return;
        }
        if self._handler_kind.clone() == "file".to_string() {
            if ((self._handler_allows(level_num))
                && (self._handler_path.clone() != "".to_string()))
            {
                let line: String = format!(
                    "{}{}", self._handler_line(level, msg), "\n".to_string()
                );
                let __sifr_try_res: Result<(), IOError> = (|| {
                    let mut fh: FileHandle = (|| {
                        let __path = self._handler_path.clone().to_string();
                        let __mode = "a".to_string().to_string();
                        let __handle_id = __sifr_next_file_handle_id();
                        match __mode.as_str() {
                            "r" | "rt" => {
                                let __f = std::fs::File::open(__path.as_str())
                                    .map_err(__io_err)?;
                                let __reader = std::io::BufReader::new(__f);
                                __SIFR_FILE_HANDLES
                                    .lock()
                                    .unwrap_or_else(|__err| __err.into_inner())
                                    .insert(__handle_id, SifrFileHandle::TextRead(__reader));
                                return Ok(FileHandle {
                                    _handle: __handle_id,
                                    _mode: __mode.to_string(),
                                });
                            }
                            "w" | "wt" => {
                                let __f = std::fs::File::create(__path.as_str())
                                    .map_err(__io_err)?;
                                let __writer = std::io::BufWriter::new(__f);
                                __SIFR_FILE_HANDLES
                                    .lock()
                                    .unwrap_or_else(|__err| __err.into_inner())
                                    .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                                return Ok(FileHandle {
                                    _handle: __handle_id,
                                    _mode: __mode.to_string(),
                                });
                            }
                            "a" | "at" => {
                                let __f = std::fs::OpenOptions::new()
                                    .append(true)
                                    .create(true)
                                    .open(__path.as_str())
                                    .map_err(__io_err)?;
                                let __writer = std::io::BufWriter::new(__f);
                                __SIFR_FILE_HANDLES
                                    .lock()
                                    .unwrap_or_else(|__err| __err.into_inner())
                                    .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                                return Ok(FileHandle {
                                    _handle: __handle_id,
                                    _mode: __mode.to_string(),
                                });
                            }
                            "rb" => {
                                let __f = std::fs::File::open(__path.as_str())
                                    .map_err(__io_err)?;
                                let __reader = std::io::BufReader::new(__f);
                                __SIFR_FILE_HANDLES
                                    .lock()
                                    .unwrap_or_else(|__err| __err.into_inner())
                                    .insert(__handle_id, SifrFileHandle::BinaryRead(__reader));
                                return Ok(FileHandle {
                                    _handle: __handle_id,
                                    _mode: __mode.to_string(),
                                });
                            }
                            "wb" => {
                                let __f = std::fs::File::create(__path.as_str())
                                    .map_err(__io_err)?;
                                let __writer = std::io::BufWriter::new(__f);
                                __SIFR_FILE_HANDLES
                                    .lock()
                                    .unwrap_or_else(|__err| __err.into_inner())
                                    .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                                return Ok(FileHandle {
                                    _handle: __handle_id,
                                    _mode: __mode.to_string(),
                                });
                            }
                            "ab" => {
                                let __f = std::fs::OpenOptions::new()
                                    .append(true)
                                    .create(true)
                                    .open(__path.as_str())
                                    .map_err(__io_err)?;
                                let __writer = std::io::BufWriter::new(__f);
                                __SIFR_FILE_HANDLES
                                    .lock()
                                    .unwrap_or_else(|__err| __err.into_inner())
                                    .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                                return Ok(FileHandle {
                                    _handle: __handle_id,
                                    _mode: __mode.to_string(),
                                });
                            }
                            _ => {
                                return Err(IOError {
                                    message: format!("invalid mode: {}", __mode),
                                    kind: "Other".to_string(),
                                });
                            }
                        }
                    })()?;
                    let __sifr_try_res: Result<(), IOError> = (|| {
                        let _: () = fh.write(&line)?;
                        return Ok(());
                    })();
                    if let Err(__sifr_try_err) = __sifr_try_res {
                        let e2 = __sifr_try_err.clone();
                        let _: String = e2.message;
                    }
                    fh.close();
                    return Ok(());
                })();
                if let Err(__sifr_try_err) = __sifr_try_res {
                    let e = __sifr_try_err.clone();
                    let _: String = e.message;
                }
            }
            return;
        }
        let line: String = format!(
            "{}{}{}{}{}{}", "[".to_string(), level, "] ".to_string(), self._name.clone(),
            ": ".to_string(), msg
        );
        println!("{}", line);
        if self._log_path.clone() != "".to_string() {
            let __sifr_try_res: Result<(), IOError> = (|| {
                let mut fh: FileHandle = (|| {
                    let __path = self._log_path.clone().to_string();
                    let __mode = "a".to_string().to_string();
                    let __handle_id = __sifr_next_file_handle_id();
                    match __mode.as_str() {
                        "r" | "rt" => {
                            let __f = std::fs::File::open(__path.as_str())
                                .map_err(__io_err)?;
                            let __reader = std::io::BufReader::new(__f);
                            __SIFR_FILE_HANDLES
                                .lock()
                                .unwrap_or_else(|__err| __err.into_inner())
                                .insert(__handle_id, SifrFileHandle::TextRead(__reader));
                            return Ok(FileHandle {
                                _handle: __handle_id,
                                _mode: __mode.to_string(),
                            });
                        }
                        "w" | "wt" => {
                            let __f = std::fs::File::create(__path.as_str())
                                .map_err(__io_err)?;
                            let __writer = std::io::BufWriter::new(__f);
                            __SIFR_FILE_HANDLES
                                .lock()
                                .unwrap_or_else(|__err| __err.into_inner())
                                .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                            return Ok(FileHandle {
                                _handle: __handle_id,
                                _mode: __mode.to_string(),
                            });
                        }
                        "a" | "at" => {
                            let __f = std::fs::OpenOptions::new()
                                .append(true)
                                .create(true)
                                .open(__path.as_str())
                                .map_err(__io_err)?;
                            let __writer = std::io::BufWriter::new(__f);
                            __SIFR_FILE_HANDLES
                                .lock()
                                .unwrap_or_else(|__err| __err.into_inner())
                                .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                            return Ok(FileHandle {
                                _handle: __handle_id,
                                _mode: __mode.to_string(),
                            });
                        }
                        "rb" => {
                            let __f = std::fs::File::open(__path.as_str())
                                .map_err(__io_err)?;
                            let __reader = std::io::BufReader::new(__f);
                            __SIFR_FILE_HANDLES
                                .lock()
                                .unwrap_or_else(|__err| __err.into_inner())
                                .insert(__handle_id, SifrFileHandle::BinaryRead(__reader));
                            return Ok(FileHandle {
                                _handle: __handle_id,
                                _mode: __mode.to_string(),
                            });
                        }
                        "wb" => {
                            let __f = std::fs::File::create(__path.as_str())
                                .map_err(__io_err)?;
                            let __writer = std::io::BufWriter::new(__f);
                            __SIFR_FILE_HANDLES
                                .lock()
                                .unwrap_or_else(|__err| __err.into_inner())
                                .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                            return Ok(FileHandle {
                                _handle: __handle_id,
                                _mode: __mode.to_string(),
                            });
                        }
                        "ab" => {
                            let __f = std::fs::OpenOptions::new()
                                .append(true)
                                .create(true)
                                .open(__path.as_str())
                                .map_err(__io_err)?;
                            let __writer = std::io::BufWriter::new(__f);
                            __SIFR_FILE_HANDLES
                                .lock()
                                .unwrap_or_else(|__err| __err.into_inner())
                                .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                            return Ok(FileHandle {
                                _handle: __handle_id,
                                _mode: __mode.to_string(),
                            });
                        }
                        _ => {
                            return Err(IOError {
                                message: format!("invalid mode: {}", __mode),
                                kind: "Other".to_string(),
                            });
                        }
                    }
                })()?;
                let __sifr_try_res: Result<(), IOError> = (|| {
                    let _: () = fh.write(&format!("{}{}", line, "\n".to_string()))?;
                    return Ok(());
                })();
                if let Err(__sifr_try_err) = __sifr_try_res {
                    let e2 = __sifr_try_err.clone();
                    let _: String = e2.message;
                }
                fh.close();
                return Ok(());
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e = __sifr_try_err.clone();
                let _: String = e.message;
            }
        }
    }
    fn debug(&self, msg: &String) {
        self._emit(&"DEBUG".to_string(), DEBUG, msg);
    }
    fn info(&self, msg: &String) {
        self._emit(&"INFO".to_string(), INFO, msg);
    }
    fn warning(&self, msg: &String) {
        self._emit(&"WARNING".to_string(), WARNING, msg);
    }
    fn error(&self, msg: &String) {
        self._emit(&"ERROR".to_string(), ERROR, msg);
    }
    fn critical(&self, msg: &String) {
        self._emit(&"CRITICAL".to_string(), CRITICAL, msg);
    }
}
impl std::fmt::Display for Logger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "Logger(_name={}, _level={}, _log_path={}, _handler_kind={}, _handler_path={}, _handler_level={}, _handler_fmt={})",
            self._name, self._level, self._log_path, self._handler_kind, self
            ._handler_path, self._handler_level, self._handler_fmt
        );
    }
}
fn _level_name_to_num(level: &String) -> i64 {
    if level.clone() == "DEBUG".to_string() {
        return DEBUG;
    }
    if level.clone() == "INFO".to_string() {
        return INFO;
    }
    if level.clone() == "WARNING".to_string() {
        return WARNING;
    }
    if level.clone() == "ERROR".to_string() {
        return ERROR;
    }
    if level.clone() == "CRITICAL".to_string() {
        return CRITICAL;
    }
    return NOTSET;
}
fn getLogger(name: &String) -> Logger {
    let level: i64 = *__SIFR_GLOBAL_LOG_LEVEL
        .lock()
        .unwrap_or_else(|__err| __err.into_inner());
    return Logger::new((name).clone(), level);
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

enum SifrFileHandle {
    TextRead(std::io::BufReader<std::fs::File>),
    TextWrite(std::io::BufWriter<std::fs::File>),
    BinaryRead(std::io::BufReader<std::fs::File>),
    BinaryWrite(std::io::BufWriter<std::fs::File>),
}

static __SIFR_FILE_HANDLES: std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<i64, SifrFileHandle>>> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

static __SIFR_NEXT_FILE_HANDLE_ID: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(1);

fn __sifr_next_file_handle_id() -> i64 {
    return __SIFR_NEXT_FILE_HANDLE_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}

static __SIFR_GLOBAL_LOG_LEVEL: std::sync::LazyLock<std::sync::Mutex<i64>> = std::sync::LazyLock::new(|| std::sync::Mutex::new(20));

fn workload() {
    let mut i: i64 = 0 as i64;
    let mut total: i64 = 0 as i64;
    while i < (64 as i64) {
        total = total + i;
        i = i + (1 as i64);
    }
}

fn main() {
    let mut demo_ok: bool = false;
    let log_path: String = "/tmp/sifr_runtime_logging_and_timers.log".to_string();
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _w: () = std::fs::write(&log_path, "".to_string().as_bytes()).map(|_| ()).map_err(__io_err)?;
    let mut logger: Logger = getLogger(&"logging_and_timers-demo".to_string());
    logger.set_file(&log_path);
    let fh: FileHandler = FileHandler::new(format!("{}{}", log_path, "".to_string()), INFO);
    logger.add_handler(&fh);
    logger.info(&"hello".to_string());
    logger.clear_handler();
    let gmt: struct_time = gmtime_struct(0.0 as f64);
    let epoch_tm: struct_time = struct_time::new(1970 as i64, 1 as i64, 1 as i64, 0 as i64, 0 as i64, 0 as i64, 3 as i64, 1 as i64, 0 as i64);
    let epoch_ok: bool = mktime(&epoch_tm) == (0.0 as f64);
    let mut timer: Timer = Timer::new();
    let elapsed: f64 = timer.__call__(workload, 4 as i64);
    let content: String = std::fs::read_to_string(&log_path).map_err(__io_err)?;
    demo_ok = ((((((content == "INFO:logging_and_timers-demo:hello\n".to_string()) && (gmt.tm_year == (1970 as i64))) && (epoch_ok)) && (elapsed >= (0.0 as f64))) && (TIMEZONE == (0 as i64))) && ((__const_TZNAME()).0 == "UTC".to_string()));
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = e.message;
    }
    let __sifr_try_res: Result<(), IOError> = (|| {
    if std::path::Path::new(&log_path).exists() {
        let _rm: () = std::fs::remove_file(&log_path).map(|_| ()).map_err(__io_err)?;
    }
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = e.message;
    }
    assert!(demo_ok);
    println!("runtime_logging_and_timers_time_timeit_object_surface_demo: ok");
}
