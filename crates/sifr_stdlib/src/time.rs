use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Datelike, Local, NaiveDateTime, Timelike, Utc};
use sifr_runtime::interop::SifrIntBridge;

#[must_use]
pub const fn feature_name() -> &'static str {
    "time"
}

#[must_use]
pub fn time_now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

#[must_use]
pub fn perf_counter() -> f64 {
    time_now()
}

#[must_use]
pub fn time_format(epoch: f64, fmt: &str) -> String {
    DateTime::from_timestamp(epoch as i64, 0)
        .unwrap_or_default()
        .format(fmt)
        .to_string()
}

pub fn strptime(s: &str, fmt: &str) -> Result<String, String> {
    NaiveDateTime::parse_from_str(s, fmt)
        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S").to_string())
        .map_err(|err| err.to_string())
}

#[must_use]
pub fn gmtime(epoch: f64) -> String {
    DateTime::<Utc>::from_timestamp(epoch as i64, 0)
        .map(format_iso8601)
        .unwrap_or_default()
}

#[must_use]
pub fn localtime(epoch: f64) -> String {
    DateTime::<Utc>::from_timestamp(epoch as i64, 0)
        .map(|dt| format_iso8601(dt.with_timezone(&Local)))
        .unwrap_or_default()
}

pub fn time_strptime(s: &str, fmt: &str) -> Result<Vec<SifrIntBridge>, String> {
    NaiveDateTime::parse_from_str(s, fmt)
        .map(time_parts)
        .map_err(|err| err.to_string())
}

#[must_use]
pub fn time_gmtime() -> Vec<SifrIntBridge> {
    time_parts(Utc::now().naive_utc())
}

#[must_use]
pub fn time_localtime() -> Vec<SifrIntBridge> {
    time_parts(Local::now().naive_local())
}

#[must_use]
pub fn datetime_now() -> String {
    Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

#[must_use]
pub fn datetime_now_struct() -> Vec<SifrIntBridge> {
    let dt = Local::now();
    vec![
        SifrIntBridge::from(i64::from(dt.year())),
        SifrIntBridge::from(i64::from(dt.month())),
        SifrIntBridge::from(i64::from(dt.day())),
        SifrIntBridge::from(i64::from(dt.hour())),
        SifrIntBridge::from(i64::from(dt.minute())),
        SifrIntBridge::from(i64::from(dt.second())),
    ]
}

#[must_use]
pub fn datetime_format(dt: &str, fmt: &str) -> String {
    NaiveDateTime::parse_from_str(dt, fmt)
        .map(|parsed| parsed.format("%Y-%m-%dT%H:%M:%S").to_string())
        .unwrap_or_default()
}

pub fn datetime_from_timestamp(ts: f64) -> Result<String, std::io::Error> {
    let ts = ts as i64;
    DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S").to_string())
        .ok_or_else(|| std::io::Error::other("invalid timestamp"))
}

fn format_iso8601<Tz: chrono::TimeZone>(dt: DateTime<Tz>) -> String
where
    Tz::Offset: std::fmt::Display,
{
    dt.format("%Y-%m-%dT%H:%M:%S").to_string()
}

fn time_parts<T: Datelike + Timelike>(dt: T) -> Vec<SifrIntBridge> {
    vec![
        SifrIntBridge::from(i64::from(dt.year())),
        SifrIntBridge::from(i64::from(dt.month())),
        SifrIntBridge::from(i64::from(dt.day())),
        SifrIntBridge::from(i64::from(dt.hour())),
        SifrIntBridge::from(i64::from(dt.minute())),
        SifrIntBridge::from(i64::from(dt.second())),
        SifrIntBridge::from(i64::from(dt.weekday().num_days_from_monday())),
        SifrIntBridge::from(i64::from(dt.ordinal())),
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        datetime_format, datetime_from_timestamp, datetime_now_struct, gmtime, strptime,
        time_format, time_strptime,
    };

    #[test]
    fn datetime_adapter_formats_and_rejects_bad_format_input() {
        assert_eq!(
            datetime_format("2024-01-15 10:30:00", "%Y-%m-%d %H:%M:%S"),
            "2024-01-15T10:30:00"
        );
        assert_eq!(datetime_format("bad", "%Y-%m-%d"), "");
    }

    #[test]
    fn datetime_adapter_formats_timestamp_and_reports_invalid_range() {
        assert_eq!(
            datetime_from_timestamp(0.0).expect("epoch should format"),
            "1970-01-01T00:00:00"
        );
        let err =
            datetime_from_timestamp(100_000_000_000_000_000_000.0).expect_err("range should fail");
        assert_eq!(err.to_string(), "invalid timestamp");
    }

    #[test]
    fn datetime_now_struct_has_six_integer_fields() {
        assert_eq!(datetime_now_struct().len(), 6);
    }

    #[test]
    fn time_adapter_formats_and_parses_epoch_values() {
        assert_eq!(time_format(0.0, "%Y-%m-%d %H:%M:%S"), "1970-01-01 00:00:00");
        assert_eq!(gmtime(0.0), "1970-01-01T00:00:00");
        assert_eq!(
            strptime("2024-02-29 01:02:03", "%Y-%m-%d %H:%M:%S").expect("leap day parses"),
            "2024-02-29T01:02:03"
        );
        assert!(strptime("not a date", "%Y-%m-%d").is_err());

        let parts = time_strptime("2024-02-29 01:02:03", "%Y-%m-%d %H:%M:%S").expect("parts parse");
        let parts = parts
            .into_iter()
            .map(|part| part.to_i64_saturating())
            .collect::<Vec<_>>();
        assert_eq!(parts, vec![2024, 2, 29, 1, 2, 3, 3, 60]);
    }
}
