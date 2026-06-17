use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime, TimeZone, Utc};
use std::fmt;

#[derive(Debug, Clone)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for ValueError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct UtcOffset {
    offset_seconds: i32,
}

impl UtcOffset {
    fn new(offset_seconds: i32) -> Self {
        Self { offset_seconds }
    }
}

impl fmt::Display for UtcOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.offset_seconds < 0 { '-' } else { '+' };
        let offset = self.offset_seconds.unsigned_abs();
        let hours = offset / 3600;
        let minutes = (offset % 3600) / 60;

        write!(f, "UTC{sign}{hours:02}:{minutes:02}")
    }
}

fn datetime(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> NaiveDateTime {
    NaiveDate::from_ymd_opt(year, month, day)
        .and_then(|date| date.and_hms_opt(hour, minute, second))
        .expect("hardcoded demo datetime must be valid")
}

fn isoformat(value: &NaiveDateTime) -> String {
    value.format("%Y-%m-%dT%H:%M:%S").to_string()
}

fn now() -> chrono::DateTime<Local> {
    Local::now()
}

fn from_timestamp(timestamp: f64) -> Result<NaiveDateTime, ValueError> {
    if !timestamp.is_finite() || timestamp < i64::MIN as f64 || timestamp > i64::MAX as f64 {
        return Err(ValueError::new("invalid timestamp"));
    }

    Utc.timestamp_opt(timestamp as i64, 0)
        .single()
        .map(|value| value.naive_utc())
        .ok_or_else(|| ValueError::new("invalid timestamp"))
}

fn timezone(offset_seconds: i32) -> UtcOffset {
    UtcOffset::new(offset_seconds)
}

fn collect_positive_actual() -> Vec<bool> {
    let dt = datetime(2024, 1, 15, 10, 30, 0);
    let base_delta = Duration::seconds(3600);
    let extra_delta = Duration::seconds(1800);
    let current = now();

    vec![
        isoformat(&dt) == "2024-01-15T10:30:00",
        (base_delta + extra_delta).num_seconds() == 5400,
        current.year() > 2020 && (1..=12).contains(&current.month()),
        from_timestamp(0.0)
            .map(|value| isoformat(&value) == "1970-01-01T00:00:00")
            .unwrap_or(false),
        timezone(-19_800).to_string() == "UTC-05:30",
    ]
}

fn collect_negative_actual() -> Vec<bool> {
    vec![from_timestamp(100000000000000000000.0)
        .err()
        .is_some_and(|error| !error.message.is_empty())]
}

fn main() {
    let mut actual = Vec::new();
    actual.extend(collect_positive_actual());
    actual.extend(collect_negative_actual());

    let expected = vec![true, true, true, true, true, true];
    assert_eq!(actual, expected);
    println!("datetime datetime parity demo: pass");
}
