// --- stdlib: sifr.datetime ---
#[derive(Debug, Clone)]
struct timedelta {
    _days: i64,
    _seconds: i64,
}
impl timedelta {
    fn new(days: i64, seconds: i64) -> Self {
        return Self {
            _days: days,
            _seconds: seconds,
        };
    }
    fn total_seconds(&self) -> i64 {
        return (self._days * (86400 as i64)) + self._seconds;
    }
    fn days(&self) -> i64 {
        return self._days;
    }
    fn seconds(&self) -> i64 {
        return self._seconds;
    }
}
impl std::ops::Add<&timedelta> for &timedelta {
    type Output = timedelta;
    fn add(self, other: &timedelta) -> Self::Output {
        let total: i64 = self.total_seconds() + other.total_seconds();
        let d: i64 = total / (86400 as i64);
        let s: i64 = total % (86400 as i64);
        return timedelta::new(d, s);
    }
}
impl std::ops::Sub<&timedelta> for &timedelta {
    type Output = timedelta;
    fn sub(self, other: &timedelta) -> Self::Output {
        let total: i64 = self.total_seconds() - other.total_seconds();
        let d: i64 = total / (86400 as i64);
        let s: i64 = total % (86400 as i64);
        return timedelta::new(d, s);
    }
}
impl PartialEq for timedelta {
    fn eq(&self, other: &timedelta) -> bool {
        return self.total_seconds() == other.total_seconds();
    }
}
impl std::fmt::Display for timedelta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "timedelta(_days={}, _seconds={})", self._days, self._seconds);
    }
}
#[derive(Debug, Clone)]
struct timezone {
    _offset: i64,
}
impl timezone {
    fn new(offset: i64) -> Self {
        return Self { _offset: offset };
    }
    fn offset(&self) -> i64 {
        return self._offset;
    }
    fn iso_suffix(&self) -> String {
        let mut sign: String = "+".to_string();
        if self._offset < (0 as i64) {
            sign = "-".to_string();
        }
        let mut abs_offset: i64 = self._offset;
        if abs_offset < (0 as i64) {
            abs_offset = -abs_offset;
        }
        let h: i64 = abs_offset / (3600 as i64);
        let m: i64 = (abs_offset % (3600 as i64)) / (60 as i64);
        let mut hs: String = format!("{}", h);
        if (hs.len() as i64) < (2 as i64) {
            hs = format!("{}{}", "0".to_string(), hs);
        }
        let mut ms: String = format!("{}", m);
        if (ms.len() as i64) < (2 as i64) {
            ms = format!("{}{}", "0".to_string(), ms);
        }
        return format!("{}{}{}{}", sign, hs, ":".to_string(), ms);
    }
}
impl PartialEq for timezone {
    fn eq(&self, other: &timezone) -> bool {
        return self._offset == other._offset;
    }
}
impl std::fmt::Display for timezone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self._offset == (0 as i64) {
            return write!(f, "{}", "UTC".to_string());
        }
        return write!(f, "{}", format!("{}{}", "UTC".to_string(), self.iso_suffix()));
    }
}
#[derive(Debug, Clone)]
struct datetime {
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
    _tz_offset: Option<i64>,
}
impl datetime {
    fn new(
        year: i64,
        month: i64,
        day: i64,
        hour: i64,
        minute: i64,
        second: i64,
        tz_offset: Option<i64>,
    ) -> Self {
        return Self {
            year: year,
            month: month,
            day: day,
            hour: hour,
            minute: minute,
            second: second,
            _tz_offset: tz_offset,
        };
    }
    fn isoformat(&self) -> String {
        let y: String = format!("{}", self.year);
        let mut mo: String = format!("{}", self.month);
        if (mo.len() as i64) < (2 as i64) {
            mo = format!("{}{}", "0".to_string(), mo);
        }
        let mut d: String = format!("{}", self.day);
        if (d.len() as i64) < (2 as i64) {
            d = format!("{}{}", "0".to_string(), d);
        }
        let mut h: String = format!("{}", self.hour);
        if (h.len() as i64) < (2 as i64) {
            h = format!("{}{}", "0".to_string(), h);
        }
        let mut mi: String = format!("{}", self.minute);
        if (mi.len() as i64) < (2 as i64) {
            mi = format!("{}{}", "0".to_string(), mi);
        }
        let mut s: String = format!("{}", self.second);
        if (s.len() as i64) < (2 as i64) {
            s = format!("{}{}", "0".to_string(), s);
        }
        let base: String = format!(
            "{}{}{}{}{}{}{}{}{}{}{}", y, "-".to_string(), mo, "-".to_string(), d, "T"
            .to_string(), h, ":".to_string(), mi, ":".to_string(), s
        );
        let tz_offset_opt: Option<i64> = self._tz_offset;
        if let Some(tz_offset_opt) = tz_offset_opt {
            let offset: i64 = tz_offset_opt;
            let mut sign: String = "+".to_string();
            let mut abs_offset: i64 = offset;
            if abs_offset < (0 as i64) {
                sign = "-".to_string();
                abs_offset = -abs_offset;
            }
            let h_off: i64 = abs_offset / (3600 as i64);
            let m_off: i64 = (abs_offset % (3600 as i64)) / (60 as i64);
            let mut hs_off: String = format!("{}", h_off);
            if (hs_off.len() as i64) < (2 as i64) {
                hs_off = format!("{}{}", "0".to_string(), hs_off);
            }
            let mut ms_off: String = format!("{}", m_off);
            if (ms_off.len() as i64) < (2 as i64) {
                ms_off = format!("{}{}", "0".to_string(), ms_off);
            }
            return format!("{}{}{}{}{}", base, sign, hs_off, ":".to_string(), ms_off);
        }
        return base;
    }
    fn timestamp(&self) -> i64 {
        let mut days: i64 = 0 as i64;
        if self.year >= (1970 as i64) {
            let mut y: i64 = 1970 as i64;
            while y < self.year {
                days = days + _days_in_year(y);
                y = y + (1 as i64);
            }
        } else {
            let mut y: i64 = 1969 as i64;
            while y >= self.year {
                days = days - _days_in_year(y);
                y = y - (1 as i64);
            }
        }
        let mut m: i64 = 1 as i64;
        while m < self.month {
            days = days + _days_in_month(self.year, m);
            m = m + (1 as i64);
        }
        days = (days + self.day) - (1 as i64);
        let naive_timestamp: i64 = (((days * (86400 as i64))
            + (self.hour * (3600 as i64))) + (self.minute * (60 as i64))) + self.second;
        let tz_offset_opt: Option<i64> = self._tz_offset;
        if let Some(tz_offset_opt) = tz_offset_opt {
            let offset: i64 = tz_offset_opt;
            return naive_timestamp - offset;
        }
        return naive_timestamp;
    }
    fn astimezone(&self, tz: &Option<timezone>) -> Result<datetime, ValueError> {
        let mut target: timezone = timezone::new(0 as i64);
        if let Some(tz) = tz.as_ref() {
            let __sifr_try_res: Result<(), ValueError> = (|| {
                let tz_text: String = format!("{}", tz);
                let target_offset: i64 = _timezone_offset_from_text(&tz_text)?;
                target = timezone::new(target_offset);
                return Ok(());
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e = __sifr_try_err.clone();
                return Err(ValueError::new(e.message));
            }
        }
        return from_timestamp(self.timestamp() as f64, &Some(target));
    }
}
impl PartialEq for datetime {
    fn eq(&self, other: &datetime) -> bool {
        let same_tz: bool = self._tz_offset == other._tz_offset;
        return (((((((self.year == other.year) && (self.month == other.month))
            && (self.day == other.day)) && (self.hour == other.hour))
            && (self.minute == other.minute)) && (self.second == other.second))
            && (same_tz));
    }
}
impl std::fmt::Display for datetime {
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
fn _parse_datetime_iso(
    value: &String,
) -> Result<(i64, i64, i64, i64, i64, i64), ValueError> {
    if (value.chars().count() as i64) < (19 as i64) {
        return Err(ValueError::new("invalid datetime string".to_string()));
    }
    if (((((({
        let Some(__indexed_char) = value.chars().nth((4 as i64) as usize) else {
            unreachable!("compiler-verified string index should be in range");
        };
        __indexed_char.to_string()
    }) != "-".to_string())
        || (({
            let Some(__indexed_char) = value.chars().nth((7 as i64) as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        }) != "-".to_string()))
        || (({
            let Some(__indexed_char) = value.chars().nth((10 as i64) as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        }) != "T".to_string()))
        || (({
            let Some(__indexed_char) = value.chars().nth((13 as i64) as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        }) != ":".to_string()))
        || (({
            let Some(__indexed_char) = value.chars().nth((16 as i64) as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        }) != ":".to_string()))
    {
        return Err(ValueError::new("invalid datetime string".to_string()));
    }
    let __sifr_try_res: Result<
        Result<(i64, i64, i64, i64, i64, i64), ValueError>,
        ParseError,
    > = (|| {
        let year: i64 = (_substring(value, 0 as i64, 4 as i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let month: i64 = (_substring(value, 5 as i64, 7 as i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let day: i64 = (_substring(value, 8 as i64, 10 as i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let hour: i64 = (_substring(value, 11 as i64, 13 as i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let minute: i64 = (_substring(value, 14 as i64, 16 as i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let second: i64 = (_substring(value, 17 as i64, 19 as i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        return Ok(Ok((year, month, day, hour, minute, second)));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let _e = __sifr_try_err.clone();
            return Err(ValueError::new("invalid datetime string".to_string()));
        }
    }
}
fn _timezone_offset_from_text(text: &String) -> Result<i64, ValueError> {
    if text.clone() == "UTC".to_string() {
        return Ok(0 as i64);
    }
    if (text.chars().count() as i64) != (9 as i64) {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    if _substring(text, 0 as i64, 3 as i64) != "UTC".to_string() {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    let sign_value: String = _substring(text, 3 as i64, 4 as i64);
    if (sign_value != "+".to_string()) && (sign_value != "-".to_string()) {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    if ({
        let __sifr_index_str = &text;
        let __sifr_index_i = 6 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
    }) != Some(":".to_string())
    {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    let __sifr_try_res: Result<Result<i64, ValueError>, ParseError> = (|| {
        let hours: i64 = (_substring(text, 4 as i64, 6 as i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let minutes: i64 = (_substring(text, 7 as i64, 9 as i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let mut offset: i64 = (hours * (3600 as i64)) + (minutes * (60 as i64));
        if sign_value == "-".to_string() {
            offset = -offset;
        }
        return Ok(Ok(offset));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let _e = __sifr_try_err.clone();
            return Err(ValueError::new("invalid timezone string".to_string()));
        }
    }
}
fn _from_timestamp_with_tz(
    ts: f64,
    tz: &Option<timezone>,
) -> Result<datetime, ValueError> {
    let __sifr_try_res: Result<Result<datetime, ValueError>, ValueError> = (|| {
        let whole_seconds: i64 = ts as i64;
        let mut adjusted_seconds: i64 = whole_seconds;
        let mut tz_offset_value: i64 = 0 as i64;
        let mut tz_has_offset: bool = false;
        if let Some(tz) = tz.as_ref() {
            let tz_text: String = format!("{}", tz);
            let tz_offset: i64 = _timezone_offset_from_text(&tz_text)?;
            adjusted_seconds = whole_seconds + tz_offset;
            tz_offset_value = tz_offset;
            tz_has_offset = true;
        }
        let rendered: String = ({
            let __ts = (adjusted_seconds as f64) as i64;
            chrono::DateTime::from_timestamp(__ts, 0)
                .map(|dt| dt.format(&"%Y-%m-%dT%H:%M:%S".to_string()).to_string())
                .ok_or_else(|| ValueError {
                    message: "invalid timestamp".to_string(),
                })
        })?;
        let parts: (i64, i64, i64, i64, i64, i64) = _parse_datetime_iso(&rendered)?;
        let year_part: Option<i64> = Some((parts).0);
        let month_part: Option<i64> = Some((parts).1);
        let day_part: Option<i64> = Some((parts).2);
        let hour_part: Option<i64> = Some((parts).3);
        let minute_part: Option<i64> = Some((parts).4);
        let second_part: Option<i64> = Some((parts).5);
        let mut year: i64 = 0 as i64;
        let mut month: i64 = 1 as i64;
        let mut day: i64 = 1 as i64;
        let mut hour: i64 = 0 as i64;
        let mut minute: i64 = 0 as i64;
        let mut second: i64 = 0 as i64;
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
                    datetime::new(
                        year,
                        month,
                        day,
                        hour,
                        minute,
                        second,
                        Some(tz_offset_value),
                    ),
                ),
            );
        }
        return Ok(Ok(datetime::new(year, month, day, hour, minute, second, None)));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(ValueError::new(e.message));
        }
    }
}
fn now(tz: &Option<timezone>) -> datetime {
    let current_epoch: f64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    let __sifr_try_res: Result<datetime, ValueError> = (|| {
        let current: datetime = _from_timestamp_with_tz(current_epoch, tz)?;
        return Ok(current);
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let _e = __sifr_try_err.clone();
            let parts: Vec<i64> = {
                let __dt = chrono::Local::now();
                vec![
                    chrono::Datelike::year(& __dt) as i64, chrono::Datelike::month(&
                    __dt) as i64, chrono::Datelike::day(& __dt) as i64,
                    chrono::Timelike::hour(& __dt) as i64, chrono::Timelike::minute(&
                    __dt) as i64, chrono::Timelike::second(& __dt) as i64
                ]
            };
            let mut yr: i64 = 0 as i64;
            let mut mo: i64 = 1 as i64;
            let mut dy: i64 = 1 as i64;
            let mut hr: i64 = 0 as i64;
            let mut mn: i64 = 0 as i64;
            let mut sc: i64 = 0 as i64;
            for (i, v) in Box::new(
                (parts)
                    .iter()
                    .copied()
                    .enumerate()
                    .map(|__pair| ((__pair.0 as i64) + 0, __pair.1)),
            ) {
                if i == (0 as i64) {
                    yr = v;
                }
                if i == (1 as i64) {
                    mo = v;
                }
                if i == (2 as i64) {
                    dy = v;
                }
                if i == (3 as i64) {
                    hr = v;
                }
                if i == (4 as i64) {
                    mn = v;
                }
                if i == (5 as i64) {
                    sc = v;
                }
            }
            if let Some(tz) = tz.as_ref() {
                let __sifr_try_res: Result<datetime, ValueError> = (|| {
                    let parsed_offset: i64 = _timezone_offset_from_text(
                        &format!("{}", tz),
                    )?;
                    return Ok(
                        datetime::new(yr, mo, dy, hr, mn, sc, Some(parsed_offset)),
                    );
                    unreachable!("sifr try/except return capture fell through");
                })();
                match __sifr_try_res {
                    Ok(__sifr_ret_val) => {
                        return __sifr_ret_val;
                    }
                    Err(__sifr_try_err) => {
                        let _e = __sifr_try_err.clone();
                        return datetime::new(yr, mo, dy, hr, mn, sc, None);
                    }
                }
            }
            return datetime::new(yr, mo, dy, hr, mn, sc, None);
        }
    }
}
fn from_timestamp(ts: f64, tz: &Option<timezone>) -> Result<datetime, ValueError> {
    return _from_timestamp_with_tz(ts, tz);
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

fn collect_positive_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let mut dt: datetime = datetime::new(2024 as i64, 1 as i64, 15 as i64, 10 as i64, 30 as i64, 0 as i64, None);
    actual.push((dt.isoformat()).as_str() == ("2024-01-15T10:30:00".to_string()).as_str());
    let base_td: timedelta = timedelta::new(0 as i64, 3600 as i64);
    let extra_td: timedelta = timedelta::new(0 as i64, 1800 as i64);
    let mut td: timedelta = &base_td + &extra_td;
    actual.push(td.total_seconds() == (5400 as i64));
    let current: datetime = now(&None);
    actual.push(((current.year > (2020 as i64)) && (current.month >= (1 as i64))) && (current.month <= (12 as i64)));
    let mut epoch_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let mut epoch: datetime = from_timestamp(0.0 as f64, &None)?;
    epoch_ok = epoch.isoformat() == "1970-01-01T00:00:00".to_string();
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
    }
    actual.push(epoch_ok);
    actual.push((format!("{}", timezone::new(-(19800 as i64)))).as_str() == ("UTC-05:30".to_string()).as_str());
    return actual;
}

fn collect_negative_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let mut invalid_rejected: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let _bad: datetime = from_timestamp(100000000000000000000.0 as f64, &None)?;
    let _: datetime = _bad;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        invalid_rejected = (e.message.chars().count() as i64) > (0 as i64);
    }
    actual.push(invalid_rejected);
    return actual;
}

fn append_all(target: &mut Vec<bool>, values: &Vec<bool>) {
    for value in values.iter().copied() {
        target.push(value);
    }
}

fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true];
    let mut actual: Vec<bool> = vec![];
    append_all(&mut actual, &collect_positive_actual());
    append_all(&mut actual, &collect_negative_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("datetime datetime parity demo: pass");
}
