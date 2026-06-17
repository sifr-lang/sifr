use regex::Regex;
use sha2::{Digest, Sha256};

#[derive(Clone, Debug)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug)]
struct RegexError {
    message: String,
}

impl RegexError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug)]
struct StatisticsError {
    message: String,
}

impl StatisticsError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug)]
struct DateTime {
    year: i64,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
}

impl DateTime {
    fn new(year: i64, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    }

    fn isoformat(&self) -> String {
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}

fn from_timestamp(timestamp: f64) -> Result<DateTime, ValueError> {
    if !timestamp.is_finite() {
        return Err(ValueError::new("timestamp must be finite"));
    }
    if timestamp.fract() != 0.0 {
        return Err(ValueError::new(
            "timestamp must be a whole number of seconds",
        ));
    }

    let seconds = timestamp as i64;
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = (seconds_of_day / 3_600) as u32;
    let minute = ((seconds_of_day % 3_600) / 60) as u32;
    let second = (seconds_of_day % 60) as u32;
    Ok(DateTime::new(year, month, day, hour, minute, second))
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i64, u32, u32) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    if month <= 2 {
        year += 1;
    }
    (year, month as u32, day as u32)
}

fn search(pattern: &str, text: &str) -> Result<Option<String>, RegexError> {
    let regex = Regex::new(pattern).map_err(|error| RegexError::new(error.to_string()))?;
    Ok(regex.find(text).map(|matched| matched.as_str().to_string()))
}

fn comb(n: i64, k: i64) -> i64 {
    if k < 0 || k > n {
        return 0;
    }
    let k = k.min(n - k);
    (0..k).fold(1, |acc, step| acc * (n - step) / (step + 1))
}

fn isclose(a: f64, b: f64, rel_tol: f64, abs_tol: f64) -> bool {
    let diff = (a - b).abs();
    diff <= abs_tol.max(rel_tol * a.abs().max(b.abs()))
}

fn mean(values: &[f64]) -> Result<f64, StatisticsError> {
    if values.is_empty() {
        return Err(StatisticsError::new(
            "mean requires at least one data point",
        ));
    }
    Ok(values.iter().sum::<f64>() / values.len() as f64)
}

#[derive(Clone, Debug)]
struct HashObject {
    digest: [u8; 32],
}

impl HashObject {
    fn hexdigest(&self) -> String {
        self.digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
}

fn new(name: &str, data: &str) -> Result<HashObject, ValueError> {
    match name {
        "sha256" => {
            let digest: [u8; 32] = Sha256::digest(data.as_bytes()).into();
            Ok(HashObject { digest })
        }
        _ => Err(ValueError::new(format!("unsupported hash: {name}"))),
    }
}

fn main() {
    let dt = DateTime::new(2026, 3, 16, 12, 0, 0);
    println!("datetime.isoformat = {}", dt.isoformat());

    match from_timestamp(0.0) {
        Ok(epoch) => println!("datetime.from_timestamp(0) = {}", epoch.isoformat()),
        Err(error) => println!("datetime error: {}", error.message),
    }

    match search(r"\d+", "sifr-1200") {
        Ok(found) => println!(
            "re.search = {}",
            found.unwrap_or_else(|| "None".to_string())
        ),
        Err(error) => println!("re error: {}", error.message),
    }

    println!("math.comb(8, 3) = {}", comb(8, 3));
    println!(
        "math.isclose(0.1+0.2, 0.3) = {}",
        isclose(0.30000000000000004, 0.3, 1e-9, 0.0)
    );

    match mean(&[2.0, 4.0, 6.0, 8.0]) {
        Ok(avg) => println!("statistics.mean = {}", avg),
        Err(error) => println!("statistics error: {}", error.message),
    }

    match new("sha256", "hashlib-sample") {
        Ok(hash) => println!("hashlib.sha256 len = {}", hash.hexdigest().len()),
        Err(error) => println!("hashlib error: {}", error.message),
    }
}
