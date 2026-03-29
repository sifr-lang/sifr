use std::collections::{BTreeMap, BTreeSet};
use std::ops::{Add, Sub};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
struct CycleError {
    message: String,
}

impl CycleError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Default)]
struct TopologicalSorter {
    predecessors: BTreeMap<i64, BTreeSet<i64>>,
}

impl TopologicalSorter {
    fn new() -> Self {
        Self::default()
    }

    fn add(&mut self, node: i64, predecessor: i64) {
        self.predecessors
            .entry(node)
            .or_default()
            .insert(predecessor);
        self.predecessors.entry(predecessor).or_default();
    }

    fn static_order(&self) -> Result<Vec<i64>, CycleError> {
        let mut remaining = self.predecessors.clone();
        let mut order = Vec::new();

        loop {
            let ready = remaining
                .iter()
                .filter_map(|(node, deps)| deps.is_empty().then_some(*node))
                .collect::<Vec<_>>();

            if ready.is_empty() {
                break;
            }

            for node in ready {
                remaining.remove(&node);
                for deps in remaining.values_mut() {
                    deps.remove(&node);
                }
                order.push(node);
            }
        }

        if remaining.is_empty() {
            Ok(order)
        } else {
            Err(CycleError::new("cycle detected"))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Path {
    inner: PathBuf,
}

impl Path {
    fn new(path: impl Into<PathBuf>) -> Self {
        Self { inner: path.into() }
    }

    fn name(&self) -> String {
        self.inner
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string()
    }

    fn parent(&self) -> Self {
        Self::new(
            self.inner
                .parent()
                .unwrap_or(self.inner.as_path())
                .to_path_buf(),
        )
    }

    fn to_str(&self) -> String {
        self.inner.to_string_lossy().into_owned()
    }

    fn suffix(&self) -> String {
        self.inner
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| format!(".{ext}"))
            .unwrap_or_default()
    }

    fn stem(&self) -> String {
        self.inner
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_default()
            .to_string()
    }

    fn is_absolute(&self) -> bool {
        self.inner.is_absolute()
    }
}

const DEBUG: i64 = 10;
const INFO: i64 = 20;
const WARNING: i64 = 30;

struct Logger {
    name: String,
    level: i64,
}

impl Logger {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            level: INFO,
        }
    }

    fn set_level(&mut self, level: i64) {
        self.level = level;
    }

    fn log(&self, level: i64, label: &str, message: &str) {
        if level >= self.level {
            println!("[{label}] {}: {message}", self.name);
        }
    }

    fn info(&self, message: &str) {
        self.log(INFO, "INFO", message);
    }

    fn warning(&self, message: &str) {
        self.log(WARNING, "WARNING", message);
    }

    fn debug(&self, message: &str) {
        self.log(DEBUG, "DEBUG", message);
    }
}

fn get_logger(name: &str) -> Logger {
    Logger::new(name)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Match {
    value: String,
    start: i64,
    end: i64,
}

impl Match {
    fn new(value: impl Into<String>, start: i64, end: i64) -> Self {
        Self {
            value: value.into(),
            start,
            end,
        }
    }

    fn group(&self) -> &str {
        &self.value
    }

    fn start(&self) -> i64 {
        self.start
    }

    fn end(&self) -> i64 {
        self.end
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Uuid {
    raw: String,
}

impl Uuid {
    fn new(raw: impl Into<String>) -> Self {
        Self { raw: raw.into() }
    }

    fn hex(&self) -> String {
        self.raw.chars().filter(|ch| *ch != '-').collect()
    }

    fn version(&self) -> i64 {
        self.raw
            .split('-')
            .nth(2)
            .and_then(|segment| segment.chars().next())
            .and_then(|digit| digit.to_digit(16))
            .map(i64::from)
            .unwrap_or(-1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Timedelta {
    days: i64,
    seconds: i64,
}

impl Timedelta {
    fn new(days: i64, seconds: i64) -> Self {
        let total_seconds = days * 86_400 + seconds;
        Self {
            days: total_seconds.div_euclid(86_400),
            seconds: total_seconds.rem_euclid(86_400),
        }
    }

    fn total_seconds(&self) -> i64 {
        self.days * 86_400 + self.seconds
    }

    fn days(&self) -> i64 {
        self.days
    }
}

impl Add for Timedelta {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.days + rhs.days, self.seconds + rhs.seconds)
    }
}

impl Sub for Timedelta {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.days - rhs.days, self.seconds - rhs.seconds)
    }
}

fn main() {
    println!("=== TopologicalSorter ===");
    let mut sorter = TopologicalSorter::new();
    sorter.add(1, 0);
    sorter.add(2, 1);
    match sorter.static_order() {
        Ok(order) => {
            if let Some(first) = order.first() {
                println!("{first}");
            }
            if let Some(second) = order.get(1) {
                println!("{second}");
            }
            if let Some(third) = order.get(2) {
                println!("{third}");
            }
        }
        Err(error) => println!("cycle error: {}", error.message),
    }

    println!("=== Path ===");
    let path = Path::new("/home/user/docs/report.pdf");
    println!("{}", path.name());
    println!("{}", path.parent().to_str());
    println!("{}", path.suffix());
    println!("{}", path.stem());
    println!("{}", path.is_absolute());

    println!("=== Logger ===");
    let mut log = get_logger("demo");
    log.info("application started");
    log.warning("disk space low");
    log.debug("this should not appear at INFO level");
    log.set_level(DEBUG);
    log.debug("now visible after level change");

    println!("=== Match ===");
    let matched = Match::new("world", 6, 11);
    println!("{}", matched.group());
    println!("{}", matched.start());
    println!("{}", matched.end());

    println!("=== UUID ===");
    let uuid = Uuid::new("550e8400-e29b-41d4-a716-446655440000");
    println!("{}", uuid.hex());
    println!("{}", uuid.version());

    println!("=== timedelta ===");
    let one_day = Timedelta::new(1, 0);
    let two_hours = Timedelta::new(0, 7_200);
    let combined = one_day + two_hours;
    println!("{}", combined.total_seconds());
    println!("{}", combined.days());
    let diff = one_day - two_hours;
    println!("{}", diff.total_seconds());
    println!("{}", one_day == Timedelta::new(1, 0));
}
