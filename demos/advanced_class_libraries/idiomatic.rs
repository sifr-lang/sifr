use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
struct IOError {
    message: String,
}

impl IOError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl From<std::io::Error> for IOError {
    fn from(error: std::io::Error) -> Self {
        Self::new(error.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
struct Deque<T> {
    values: VecDeque<T>,
    maxlen: Option<usize>,
}

impl<T> Deque<T> {
    fn new(maxlen: Option<usize>) -> Self {
        Self {
            values: VecDeque::new(),
            maxlen,
        }
    }

    fn append(&mut self, value: T) {
        if let Some(limit) = self.maxlen {
            if self.values.len() == limit {
                self.values.pop_front();
            }
        }
        self.values.push_back(value);
    }

    fn popleft(&mut self) -> T {
        self.values.pop_front().expect("demo deque is non-empty")
    }

    fn len(&self) -> i64 {
        self.values.len() as i64
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DateTime {
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
}

impl DateTime {
    fn new(year: i64, month: i64, day: i64, hour: i64, minute: i64, second: i64) -> Self {
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct Date {
    year: i64,
    month: i64,
    day: i64,
}

impl Date {
    fn new(year: i64, month: i64, day: i64) -> Self {
        Self { year, month, day }
    }

    fn isoformat(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
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

    fn touch(&self) -> Result<(), IOError> {
        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&self.inner)
            .map(|_| ())
            .map_err(IOError::from)
    }

    fn exists(&self) -> bool {
        self.inner.exists()
    }

    fn unlink(&self) -> Result<(), IOError> {
        fs::remove_file(&self.inner).map_err(IOError::from)
    }

    fn with_suffix(&self, suffix: &str) -> Self {
        let mut next = self.inner.clone();
        next.set_extension(suffix.trim_start_matches('.'));
        Self::new(next)
    }

    fn with_name(&self, name: &str) -> Self {
        Self::new(self.inner.with_file_name(name))
    }

    fn to_str(&self) -> String {
        self.inner.to_string_lossy().into_owned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Pattern {
    raw: String,
}

impl Pattern {
    fn is_match(&self, text: &str) -> bool {
        self.search(text).is_some()
    }

    fn search(&self, text: &str) -> Option<String> {
        if self.raw != "\\d+" {
            return None;
        }
        let mut current = String::new();
        for ch in text.chars() {
            if ch.is_ascii_digit() {
                current.push(ch);
            } else if !current.is_empty() {
                return Some(current);
            }
        }
        (!current.is_empty()).then_some(current)
    }

    fn findall(&self, text: &str) -> Vec<String> {
        if self.raw != "\\d+" {
            return Vec::new();
        }
        let mut matches = Vec::new();
        let mut current = String::new();
        for ch in text.chars() {
            if ch.is_ascii_digit() {
                current.push(ch);
            } else if !current.is_empty() {
                matches.push(std::mem::take(&mut current));
            }
        }
        if !current.is_empty() {
            matches.push(current);
        }
        matches
    }
}

fn compile(pattern: &str) -> Result<Pattern, RegexError> {
    if pattern == "\\d+" {
        Ok(Pattern {
            raw: pattern.to_string(),
        })
    } else {
        Err(RegexError::new("unsupported regex pattern"))
    }
}

fn fullmatch(pattern: &str, text: &str) -> Result<bool, RegexError> {
    let compiled = compile(pattern)?;
    Ok(compiled.search(text).as_deref() == Some(text))
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

    fn debug(&self, message: &str) {
        self.log(DEBUG, "DEBUG", message);
    }

    fn info(&self, message: &str) {
        self.log(INFO, "INFO", message);
    }

    fn warning(&self, message: &str) {
        self.log(WARNING, "WARNING", message);
    }
}

fn get_logger(name: &str) -> Logger {
    Logger::new(name)
}

struct Reader {
    rows: Vec<Vec<String>>,
}

impl Reader {
    fn new(text: &str) -> Self {
        Self {
            rows: text
                .lines()
                .map(|line| line.split(',').map(|part| part.to_string()).collect())
                .collect(),
        }
    }

    fn rows(&self) -> Vec<Vec<String>> {
        self.rows.clone()
    }
}

#[derive(Default)]
struct Writer {
    rows: Vec<Vec<String>>,
}

impl Writer {
    fn new() -> Self {
        Self::default()
    }

    fn writerow(&mut self, row: Vec<String>) {
        self.rows.push(row);
    }

    fn getvalue(&self) -> String {
        self.rows
            .iter()
            .map(|row| row.join(","))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

struct DictReader {
    headers: Vec<String>,
    rows: Vec<BTreeMap<String, String>>,
}

impl DictReader {
    fn new(text: &str) -> Self {
        let mut lines = text.lines();
        let headers = lines
            .next()
            .unwrap_or_default()
            .split(',')
            .map(|part| part.to_string())
            .collect::<Vec<_>>();
        let rows = lines
            .map(|line| {
                headers
                    .iter()
                    .cloned()
                    .zip(line.split(',').map(|part| part.to_string()))
                    .collect::<BTreeMap<_, _>>()
            })
            .collect::<Vec<_>>();
        Self { headers, rows }
    }

    fn fieldnames(&self) -> Vec<String> {
        self.headers.clone()
    }

    fn rows(&self) -> Vec<BTreeMap<String, String>> {
        self.rows.clone()
    }
}

struct DictWriter {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl DictWriter {
    fn new(headers: Vec<String>) -> Self {
        Self {
            headers,
            rows: Vec::new(),
        }
    }

    fn writeheader(&mut self) {
        self.rows.push(self.headers.clone());
    }

    fn writerow(&mut self, row: BTreeMap<String, String>) {
        self.rows.push(
            self.headers
                .iter()
                .map(|header| row.get(header).cloned().unwrap_or_default())
                .collect(),
        );
    }

    fn getvalue(&self) -> String {
        self.rows
            .iter()
            .map(|row| row.join(","))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn main() {
    let mut d = Deque::new(Some(3));
    d.append(1_i64);
    d.append(2_i64);
    d.append(3_i64);
    d.append(4_i64);
    println!("deque len (maxlen=3) = {}", d.len());
    println!("deque popleft = {}", d.popleft());

    let dt = DateTime::new(2024, 6, 15, 9, 30, 0);
    println!("datetime isoformat = {}", dt.isoformat());
    println!("datetime year = {}", dt.year);

    let today = Date::new(2024, 6, 15);
    println!("date isoformat = {}", today.isoformat());

    let path = Path::new("/tmp/demo_file.txt");
    match path.touch() {
        Ok(()) => {
            println!("path touch ok = true");
            println!("path exists = {}", path.exists());
            match path.unlink() {
                Ok(()) => println!("path unlink ok = true"),
                Err(error) => println!("path error: {}", error.message),
            }
        }
        Err(error) => println!("path error: {}", error.message),
    }

    let file_path = Path::new("/tmp/myfile.txt");
    println!("with_suffix = {}", file_path.with_suffix(".csv").to_str());
    println!("with_name = {}", file_path.with_name("other.txt").to_str());

    match compile("\\d+") {
        Ok(pattern) => {
            println!("pattern is_match = {}", pattern.is_match("abc123"));
            if let Some(found) = pattern.search("hello 42 world") {
                println!("pattern search found = {}", !found.is_empty());
            }
            let nums = pattern.findall("1 plus 2 equals 3");
            println!("pattern findall count = {}", nums.len());
            match fullmatch("\\d+", "12345") {
                Ok(value) => println!("fullmatch digits = {value}"),
                Err(error) => println!("fullmatch error: {}", error.message),
            }
        }
        Err(error) => println!("regex error: {}", error.message),
    }

    let mut log = get_logger("demo");
    log.set_level(DEBUG);
    log.debug("debug message");
    log.info("info message");
    log.warning("warning message");

    let csv_text = "name,age\nalice,30\nbob,25";
    let reader = Reader::new(csv_text);
    let all_rows = reader.rows();
    println!("csv rows = {}", all_rows.len());

    let mut writer = Writer::new();
    writer.writerow(vec!["x".to_string(), "y".to_string()]);
    writer.writerow(vec!["1".to_string(), "2".to_string()]);
    println!("csv writer output = {}", writer.getvalue());

    let dict_reader = DictReader::new("name,score\nalice,95\nbob,87");
    let headers = dict_reader.fieldnames();
    if let Some(first_header) = headers.first() {
        println!("dictreader headers = {first_header}");
    }
    let dict_rows = dict_reader.rows();
    println!("dictreader row count = {}", dict_rows.len());

    let mut dict_writer = DictWriter::new(vec!["name".to_string(), "score".to_string()]);
    dict_writer.writeheader();
    dict_writer.writerow(BTreeMap::from([
        ("name".to_string(), "charlie".to_string()),
        ("score".to_string(), "91".to_string()),
    ]));
    println!("dictwriter output = {}", dict_writer.getvalue());
}
