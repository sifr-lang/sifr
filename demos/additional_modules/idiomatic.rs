use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;

#[derive(Debug, Clone)]
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

fn zip_err<E: std::fmt::Display>(error: E) -> IOError {
    IOError::new(error.to_string())
}

#[derive(Debug, Clone)]
struct ParsingError {
    message: String,
}

impl ParsingError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
struct CompletedProcess {
    stdout: String,
}

fn add(a: i64, b: i64) -> i64 {
    a + b
}

fn sub(a: i64, b: i64) -> i64 {
    a - b
}

fn mul(a: i64, b: i64) -> i64 {
    a * b
}

fn floordiv(a: i64, b: i64) -> i64 {
    a.div_euclid(b)
}

fn mod_val(a: i64, b: i64) -> i64 {
    a.rem_euclid(b)
}

fn neg(value: i64) -> i64 {
    -value
}

fn lt(a: i64, b: i64) -> bool {
    a < b
}

fn eq(a: i64, b: i64) -> bool {
    a == b
}

fn itemgetter<T: Clone>(items: &[T], index: usize) -> T {
    items[index].clone()
}

fn isleap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn weekday(year: i64, month: i64, day: i64) -> i64 {
    let offsets = [0_i64, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let mut y = year;
    if month < 3 {
        y -= 1;
    }
    let sunday_zero = (y + y / 4 - y / 100 + y / 400 + offsets[(month - 1) as usize] + day) % 7;
    (sunday_zero + 6) % 7
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if isleap(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn monthrange(year: i64, month: i64) -> Vec<i64> {
    vec![weekday(year, month, 1), days_in_month(year, month)]
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

fn version() -> String {
    "sifr 0.1.0".to_string()
}

fn maxsize() -> i64 {
    i64::MAX
}

fn run(command: &str) -> Result<CompletedProcess, IOError> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .map_err(IOError::from)?;
    Ok(CompletedProcess {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
    })
}

#[derive(Default)]
struct ConfigParser {
    sections: BTreeMap<String, BTreeMap<String, String>>,
}

impl ConfigParser {
    fn new() -> Self {
        Self::default()
    }

    fn read_string(&mut self, text: &str) -> Result<(), ParsingError> {
        let mut current_section = None::<String>;
        for raw_line in text.lines() {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                let name = line
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .to_string();
                self.sections.entry(name.clone()).or_default();
                current_section = Some(name);
                continue;
            }

            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| ParsingError::new("invalid config line"))?;
            let section = current_section
                .clone()
                .ok_or_else(|| ParsingError::new("missing section header"))?;
            self.sections
                .entry(section)
                .or_default()
                .insert(key.trim().to_string(), value.trim().to_string());
        }
        Ok(())
    }

    fn get(&self, section: &str, key: &str) -> Option<String> {
        self.sections
            .get(section)
            .and_then(|items| items.get(key))
            .cloned()
    }

    fn has_option(&self, section: &str, key: &str) -> bool {
        self.sections
            .get(section)
            .is_some_and(|items| items.contains_key(key))
    }
}

fn compress(text: &str) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    let _ = encoder.write_all(text.as_bytes());
    encoder.finish().unwrap_or_default()
}

fn decompress(data: &[u8]) -> Result<String, IOError> {
    let mut decoder = GzDecoder::new(data);
    let mut output = String::new();
    decoder.read_to_string(&mut output).map_err(IOError::from)?;
    Ok(output)
}

fn rewrite_zip_entry(path: &str, name: &str, content: &[u8]) -> Result<(), IOError> {
    let existing_entries = if Path::new(path).exists() {
        let file = File::open(path).map_err(IOError::from)?;
        let mut archive = zip::ZipArchive::new(file).map_err(zip_err)?;
        let mut entries = Vec::new();
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).map_err(zip_err)?;
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).map_err(IOError::from)?;
            entries.push((entry.name().to_string(), bytes));
        }
        entries
    } else {
        Vec::new()
    };

    let temp_path = format!("{path}.tmp");
    let temp_file = File::create(&temp_path).map_err(IOError::from)?;
    let mut writer = zip::ZipWriter::new(temp_file);
    for (entry_name, entry_bytes) in existing_entries {
        writer
            .start_file(entry_name, zip::write::FileOptions::default())
            .map_err(zip_err)?;
        writer.write_all(&entry_bytes).map_err(IOError::from)?;
    }
    writer
        .start_file(name, zip::write::FileOptions::default())
        .map_err(zip_err)?;
    writer.write_all(content).map_err(IOError::from)?;
    writer.finish().map_err(zip_err)?;

    if Path::new(path).exists() {
        fs::remove_file(path).map_err(IOError::from)?;
    }
    fs::rename(temp_path, path).map_err(IOError::from)
}

struct ZipFile {
    path: String,
}

impl ZipFile {
    fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }

    fn create(&self) -> Result<(), IOError> {
        let file = File::create(&self.path).map_err(IOError::from)?;
        zip::ZipWriter::new(file).finish().map_err(zip_err)?;
        Ok(())
    }

    fn write(&self, name: &str, content: &str) -> Result<(), IOError> {
        rewrite_zip_entry(&self.path, name, content.as_bytes())
    }

    fn read(&self, name: &str) -> Result<String, IOError> {
        let file = File::open(&self.path).map_err(IOError::from)?;
        let mut archive = zip::ZipArchive::new(file).map_err(zip_err)?;
        let mut entry = archive.by_name(name).map_err(zip_err)?;
        let mut output = String::new();
        entry.read_to_string(&mut output).map_err(IOError::from)?;
        Ok(output)
    }

    fn namelist(&self) -> Result<Vec<String>, IOError> {
        let file = File::open(&self.path).map_err(IOError::from)?;
        let mut archive = zip::ZipArchive::new(file).map_err(zip_err)?;
        let mut names = Vec::new();
        for index in 0..archive.len() {
            if let Ok(entry) = archive.by_index(index) {
                names.push(entry.name().to_string());
            }
        }
        Ok(names)
    }
}

fn remove_file(path: &str) -> Result<(), IOError> {
    fs::remove_file(path).map_err(IOError::from)
}

fn main() {
    println!("=== operator ===");
    println!("add(10, 5) = {}", add(10, 5));
    println!("sub(10, 5) = {}", sub(10, 5));
    println!("mul(3, 4) = {}", mul(3, 4));
    println!("floordiv(7, 2) = {}", floordiv(7, 2));
    println!("mod_val(7, 2) = {}", mod_val(7, 2));
    println!("neg(42) = {}", neg(42));
    println!("lt(3, 5) = {}", lt(3, 5));
    println!("eq(5, 5) = {}", eq(5, 5));
    let items = vec![1_i64, 2, 3];
    println!("itemgetter([1,2,3], 1) = {}", itemgetter(&items, 1));

    println!("=== calendar ===");
    println!("isleap(2000) = {}", isleap(2000));
    println!("isleap(1900) = {}", isleap(1900));
    println!("isleap(2024) = {}", isleap(2024));
    println!("weekday(2024,1,1) = {}", weekday(2024, 1, 1));
    println!("monthrange(2024,2)[1] = {}", monthrange(2024, 2)[1]);

    println!("=== html ===");
    let html = "<b>Hi & Bye</b>";
    let escaped = escape(html);
    println!("escape(<b>Hi & Bye</b>) = {escaped}");
    println!(
        "unescape(&lt;b&gt;Hi &amp; Bye&lt;/b&gt;) = {}",
        unescape(&escaped)
    );

    println!("=== sys ===");
    println!("version = {}", version());
    println!("maxsize > 0 = {}", maxsize() > 0);

    println!("=== subprocess ===");
    match run("echo hello") {
        Ok(result) => println!("echo hello = {}", result.stdout),
        Err(error) => println!("error: {}", error.message),
    }

    println!("=== configparser ===");
    let mut config = ConfigParser::new();
    match config.read_string("[database]\nhost = db.example.com\nport = 5432\n") {
        Ok(()) => {
            println!(
                "host = {}",
                config
                    .get("database", "host")
                    .unwrap_or_else(|| "None".to_string())
            );
            println!(
                "port = {}",
                config
                    .get("database", "port")
                    .unwrap_or_else(|| "None".to_string())
            );
            println!("has_host = {}", config.has_option("database", "host"));
            println!("has_missing = {}", config.has_option("database", "missing"));
        }
        Err(error) => println!("{}", error.message),
    }

    println!("=== gzip ===");
    let data = "Sifr stdlib gzip compression!";
    let compressed = compress(data);
    println!("compressed len > 0 = {}", !compressed.is_empty());
    match decompress(&compressed) {
        Ok(text) => println!("decompressed = {text}"),
        Err(error) => println!("error: {}", error.message),
    }

    println!("=== zipfile ===");
    let zip_path = "/tmp/sifr_demo_zipfile.zip";
    let zip_file = ZipFile::new(zip_path);
    match zip_file.create() {
        Ok(()) => println!("zip created = true"),
        Err(error) => println!("create error: {}", error.message),
    }
    match zip_file.write("demo.txt", "Hello from ZipFile!") {
        Ok(()) => match (zip_file.read("demo.txt"), zip_file.namelist()) {
            (Ok(content), Ok(names)) => {
                println!("zip content = {content}");
                println!("zip namelist len = {}", names.len());
            }
            (Err(error), _) | (_, Err(error)) => println!("zip error: {}", error.message),
        },
        Err(error) => println!("zip error: {}", error.message),
    }
    let _ = remove_file(zip_path);
}
