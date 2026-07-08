use std::{
    fs::{self, OpenOptions},
    io::Write as _,
    path::Path,
};

#[must_use]
pub const fn feature_name() -> &'static str {
    "fs"
}

pub fn read_text(path: &str) -> Result<String, std::io::Error> {
    fs::read_to_string(path)
}

pub fn write_text(path: &str, content: &str) -> Result<(), std::io::Error> {
    fs::write(path, content.as_bytes())
}

#[must_use]
pub fn exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn read_lines(path: &str) -> Result<Vec<String>, std::io::Error> {
    fs::read_to_string(path).map(|text| text.lines().map(str::to_string).collect())
}

pub fn append_text(path: &str, content: &str) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new().append(true).create(true).open(path)?;
    file.write_all(content.as_bytes())
}
