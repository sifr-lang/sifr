use chrono::{DateTime, Datelike, Local, NaiveDateTime, Timelike};
use sifr_runtime::interop::SifrIntBridge;

#[must_use]
pub const fn feature_name() -> &'static str {
    "time"
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

#[cfg(test)]
mod tests {
    use super::{datetime_format, datetime_from_timestamp, datetime_now_struct};

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
}
