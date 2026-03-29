use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy)]
struct DateTime {
    year: i64,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
}

impl DateTime {
    fn isoformat(self) -> String {
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}

#[derive(Clone, Copy)]
struct Date {
    year: i64,
    month: u32,
    day: u32,
}

impl Date {
    fn isoformat(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

fn now() -> DateTime {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    from_timestamp(seconds).expect("current time should be representable")
}

fn today() -> Date {
    let current = now();
    Date {
        year: current.year,
        month: current.month,
        day: current.day,
    }
}

fn from_timestamp(timestamp: f64) -> Result<DateTime, &'static str> {
    if !timestamp.is_finite() || !(0.0..=253_402_300_799.0).contains(&timestamp) {
        return Err("invalid timestamp");
    }

    let whole_seconds = timestamp.trunc() as i64;
    let days = whole_seconds.div_euclid(86_400);
    let seconds = whole_seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);

    Ok(DateTime {
        year,
        month,
        day,
        hour: (seconds / 3_600) as u32,
        minute: ((seconds % 3_600) / 60) as u32,
        second: (seconds % 60) as u32,
    })
}

// Convert Unix-epoch day counts to a proleptic Gregorian date.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };

    (year, month as u32, day as u32)
}

fn main() {
    let current_iso = now().isoformat();
    let current_has_t = current_iso.contains('T');
    println!("current_has_t = {current_has_t}");
    assert_eq!(
        format!("current_has_t = {current_has_t}"),
        "current_has_t = true"
    );

    let today_iso = today().isoformat();
    let today_has_dash = today_iso.contains('-');
    println!("today_has_dash = {today_has_dash}");
    assert_eq!(
        format!("today_has_dash = {today_has_dash}"),
        "today_has_dash = true"
    );

    match from_timestamp(0.0) {
        Ok(epoch) => {
            let epoch_text = epoch.isoformat();
            println!("from_timestamp_ok = {epoch_text}");
            assert_eq!(
                format!("from_timestamp_ok = {epoch_text}"),
                "from_timestamp_ok = 1970-01-01T00:00:00"
            );
        }
        Err(message) => {
            println!("unexpected_error = {message}");
            assert_eq!(
                format!("unexpected_error = {message}"),
                "from_timestamp_invalid = invalid timestamp"
            );
        }
    }

    match from_timestamp(-99_999_999_999_999.0) {
        Ok(bad) => println!("from_timestamp_invalid_unexpected = {}", bad.isoformat()),
        Err(message) => println!("from_timestamp_invalid = {message}"),
    }
}
