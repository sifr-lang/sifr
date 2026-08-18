use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, TimeZone, Timelike, Utc};
use std::fmt;
use std::sync::LazyLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
struct StructTime {
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

impl StructTime {
    fn from_utc(value: chrono::DateTime<Utc>) -> Self {
        Self {
            tm_year: i64::from(value.year()),
            tm_mon: i64::from(value.month()),
            tm_mday: i64::from(value.day()),
            tm_hour: i64::from(value.hour()),
            tm_min: i64::from(value.minute()),
            tm_sec: i64::from(value.second()),
            tm_wday: i64::from(value.weekday().num_days_from_monday()),
            tm_yday: i64::from(value.ordinal()),
            tm_isdst: 0,
        }
    }

    fn from_local(value: chrono::DateTime<Local>) -> Self {
        Self {
            tm_year: i64::from(value.year()),
            tm_mon: i64::from(value.month()),
            tm_mday: i64::from(value.day()),
            tm_hour: i64::from(value.hour()),
            tm_min: i64::from(value.minute()),
            tm_sec: i64::from(value.second()),
            tm_wday: i64::from(value.weekday().num_days_from_monday()),
            tm_yday: i64::from(value.ordinal()),
            tm_isdst: 0,
        }
    }
}

#[derive(Debug, Clone)]
struct ValueError {
    message: String,
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for ValueError {}

static MONOTONIC_ZERO: LazyLock<Instant> = LazyLock::new(Instant::now);

fn invalid_time(message: &str) -> ValueError {
    ValueError {
        message: message.to_string(),
    }
}

fn epoch_utc(seconds: f64) -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(seconds as i64, 0)
        .single()
        .unwrap_or_else(|| {
            Utc.timestamp_opt(0, 0)
                .single()
                .expect("unix epoch must exist")
        })
}

fn unix_seconds_now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

fn time() -> f64 {
    unix_seconds_now()
}

fn sleep(seconds: f64) {
    if seconds.is_finite() && seconds > 0.0 {
        std::thread::sleep(Duration::from_secs_f64(seconds));
    }
}

fn perf_counter() -> f64 {
    MONOTONIC_ZERO.elapsed().as_secs_f64()
}

fn monotonic() -> f64 {
    perf_counter()
}

fn strftime(format: &str, epoch: f64) -> String {
    epoch_utc(epoch).format(format).to_string()
}

fn strptime(text: &str, format: &str) -> Result<String, ValueError> {
    NaiveDateTime::parse_from_str(text, format)
        .map(|parsed| parsed.format("%Y-%m-%dT%H:%M:%S").to_string())
        .map_err(|error| ValueError {
            message: error.to_string(),
        })
}

fn gmtime_struct(epoch: f64) -> StructTime {
    StructTime::from_utc(epoch_utc(epoch))
}

fn localtime_struct(epoch: f64) -> StructTime {
    StructTime::from_local(epoch_utc(epoch).with_timezone(&Local))
}

fn mktime(value: &StructTime) -> Result<f64, ValueError> {
    let Some(date) = NaiveDate::from_ymd_opt(
        value.tm_year as i32,
        value.tm_mon as u32,
        value.tm_mday as u32,
    ) else {
        return Err(invalid_time("invalid date components"));
    };
    let Some(datetime) = date.and_hms_opt(
        value.tm_hour as u32,
        value.tm_min as u32,
        value.tm_sec as u32,
    ) else {
        return Err(invalid_time("invalid time components"));
    };
    Ok(datetime.and_utc().timestamp() as f64)
}

fn collect_clock_actual() -> Vec<bool> {
    let before_perf = perf_counter();
    let before_mono = monotonic();
    sleep(0.01);
    vec![
        time() > 0.0,
        perf_counter() >= before_perf && monotonic() >= before_mono,
    ]
}

fn collect_format_actual() -> Vec<bool> {
    let utc_epoch = gmtime_struct(0.0);
    let local_epoch = localtime_struct(0.0);

    vec![
        strftime("%Y-%m-%d %H:%M:%S", 0.0) == "1970-01-01 00:00:00",
        utc_epoch.tm_year == 1970
            && utc_epoch.tm_mon == 1
            && utc_epoch.tm_mday == 1
            && utc_epoch.tm_hour == 0
            && utc_epoch.tm_min == 0
            && utc_epoch.tm_sec == 0,
        local_epoch.tm_year > 0 && local_epoch.tm_yday >= 1,
    ]
}

fn collect_parse_and_safety_actual() -> Vec<bool> {
    let parsed_ok = strptime("2024-01-15 10:30:00", "%Y-%m-%d %H:%M:%S")
        .map(|parsed| parsed == "2024-01-15T10:30:00")
        .unwrap_or(false);

    let parse_error_ok = strptime("bad", "%Y-%m-%d %H:%M:%S").is_err();

    sleep(-0.05);
    let epoch_tm = gmtime_struct(0.0);

    vec![
        parsed_ok,
        parse_error_ok,
        true,
        matches!(mktime(&epoch_tm), Ok(value) if value == 0.0),
    ]
}

fn main() {
    let mut actual = Vec::new();
    actual.extend(collect_clock_actual());
    actual.extend(collect_format_actual());
    actual.extend(collect_parse_and_safety_actual());

    let expected = vec![true, true, true, true, true, true, true, true, true];
    assert_eq!(actual, expected);
    println!("time time parity demo: pass");
}
