use std::fs;
use std::time::Instant;

const INFO: &str = "INFO";
const TIMEZONE: i64 = 0;
const TZNAME: [&str; 2] = ["UTC", "UTC"];

#[derive(Debug, Clone, PartialEq, Eq)]
struct IOError {
    message: String,
}

impl std::fmt::Display for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for IOError {}

impl From<std::io::Error> for IOError {
    fn from(error: std::io::Error) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

fn write_text(path: &str, content: &str) -> Result<(), IOError> {
    fs::write(path, content).map_err(IOError::from)
}

fn read_text(path: &str) -> Result<String, IOError> {
    fs::read_to_string(path).map_err(IOError::from)
}

fn exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}

fn remove_file(path: &str) -> Result<(), IOError> {
    fs::remove_file(path).map_err(IOError::from)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StructTime {
    tm_year: i32,
    tm_mon: u8,
    tm_mday: u8,
    tm_hour: u8,
    tm_min: u8,
    tm_sec: u8,
    tm_wday: u8,
    tm_yday: u16,
    tm_isdst: i8,
}

fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let year = year - i32::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let month = i32::from(month);
    let day = i32::from(day);
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    i64::from(era * 146097 + doe - 719_468)
}

fn civil_from_days(days: i64) -> (i32, u8, u8, u16) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let doe = days - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = year + i64::from(month <= 2);
    (year as i32, month as u8, day as u8, doy as u16 + 1)
}

fn gmtime_struct(timestamp: f64) -> StructTime {
    let total_seconds = timestamp.floor() as i64;
    let days = total_seconds.div_euclid(86_400);
    let day_seconds = total_seconds.rem_euclid(86_400);
    let (tm_year, tm_mon, tm_mday, tm_yday) = civil_from_days(days);
    let tm_hour = (day_seconds / 3_600) as u8;
    let tm_min = ((day_seconds % 3_600) / 60) as u8;
    let tm_sec = (day_seconds % 60) as u8;
    let tm_wday = ((days + 4).rem_euclid(7)) as u8;
    StructTime {
        tm_year,
        tm_mon,
        tm_mday,
        tm_hour,
        tm_min,
        tm_sec,
        tm_wday,
        tm_yday,
        tm_isdst: 0,
    }
}

fn mktime(value: StructTime) -> f64 {
    let days = days_from_civil(value.tm_year, value.tm_mon, value.tm_mday);
    let seconds = days * 86_400
        + i64::from(value.tm_hour) * 3_600
        + i64::from(value.tm_min) * 60
        + i64::from(value.tm_sec);
    seconds as f64
}

#[derive(Debug, Clone)]
struct Formatter;

#[derive(Debug, Clone)]
struct FileHandler {
    path: String,
    level: String,
}

impl FileHandler {
    fn new(path: &str, level: &str) -> Self {
        Self {
            path: path.to_string(),
            level: level.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
struct Logger {
    name: String,
    file: Option<String>,
    handlers: Vec<FileHandler>,
}

impl Logger {
    fn set_file(&mut self, path: &str) {
        self.file = Some(path.to_string());
    }

    fn add_handler(&mut self, handler: FileHandler) {
        self.handlers.push(handler);
    }

    fn clear_handler(&mut self) {
        self.handlers.clear();
    }

    fn info(&self, message: &str) -> Result<(), IOError> {
        let target = self
            .handlers
            .last()
            .map(|handler| handler.path.as_str())
            .or(self.file.as_deref())
            .ok_or_else(|| IOError {
                message: "logger has no file target".to_string(),
            })?;
        let level = self
            .handlers
            .last()
            .map(|handler| handler.level.as_str())
            .unwrap_or(INFO);
        write_text(target, &format!("{level}:{}:{message}\n", self.name))
    }
}

fn get_logger(name: &str) -> Logger {
    Logger {
        name: name.to_string(),
        file: None,
        handlers: Vec::new(),
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Timer;

impl Timer {
    fn call(&self, workload: impl Fn(), iterations: usize) -> f64 {
        let start = Instant::now();
        for _ in 0..iterations {
            workload();
        }
        start.elapsed().as_secs_f64()
    }
}

fn workload() {
    let mut total = 0;
    for value in 0..64 {
        total += value;
    }
    let _ = total;
}

fn main() {
    let mut demo_ok = false;
    let log_path = "/tmp/sifr_runtime_logging_and_timers.log";
    let _formatter = Formatter;

    if let Ok(()) = write_text(log_path, "") {
        let mut logger = get_logger("logging_and_timers-demo");
        logger.set_file(log_path);
        let file_handler = FileHandler::new(log_path, INFO);
        logger.add_handler(file_handler);

        if logger.info("hello").is_ok() {
            logger.clear_handler();
            let gmt = gmtime_struct(0.0);
            let epoch_tm = StructTime {
                tm_year: 1970,
                tm_mon: 1,
                tm_mday: 1,
                tm_hour: 0,
                tm_min: 0,
                tm_sec: 0,
                tm_wday: 3,
                tm_yday: 1,
                tm_isdst: 0,
            };
            let epoch_ok = mktime(epoch_tm) == 0.0;
            let elapsed = Timer.call(workload, 4);

            if let Ok(content) = read_text(log_path) {
                demo_ok = content == "INFO:logging_and_timers-demo:hello\n"
                    && gmt.tm_year == 1970
                    && epoch_ok
                    && elapsed >= 0.0
                    && TIMEZONE == 0
                    && TZNAME[0] == "UTC";
            }
        }
    }

    if exists(log_path) {
        let _ = remove_file(log_path);
    }

    assert!(demo_ok);
    println!("runtime_logging_and_timers_time_timeit_object_surface_demo: ok");
}
