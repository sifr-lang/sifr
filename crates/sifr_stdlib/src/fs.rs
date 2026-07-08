use std::{
    fs::{self, DirEntry, OpenOptions},
    io::Write as _,
    path::{Path, PathBuf},
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

pub fn getcwd() -> Result<String, std::io::Error> {
    std::env::current_dir().map(|path| path.to_string_lossy().to_string())
}

pub fn listdir(path: &str) -> Result<Vec<String>, std::io::Error> {
    fs::read_dir(path).map(|entries| {
        entries
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .collect()
    })
}

pub fn mkdir(path: &str) -> Result<(), std::io::Error> {
    fs::create_dir_all(path)
}

pub fn rmdir(path: &str) -> Result<(), std::io::Error> {
    fs::remove_dir(path)
}

pub fn remove_file(path: &str) -> Result<(), std::io::Error> {
    fs::remove_file(path)
}

pub fn rename(src: &str, dst: &str) -> Result<(), std::io::Error> {
    fs::rename(src, dst)
}

#[must_use]
pub fn is_file(path: &str) -> bool {
    Path::new(path).is_file()
}

#[must_use]
pub fn is_dir(path: &str) -> bool {
    Path::new(path).is_dir()
}

pub fn copy_file(src: &str, dst: &str) -> Result<(), std::io::Error> {
    fs::copy(src, dst).map(|_| ())
}

pub fn walk_dir(path: &str) -> Result<Vec<String>, std::io::Error> {
    let mut stack = vec![PathBuf::from(path)];
    let mut result = Vec::new();
    while let Some(current) = stack.pop() {
        let entries = fs::read_dir(&current)?;
        for entry_result in entries {
            let entry = entry_result?;
            let path = entry.path();
            result.push(path.display().to_string());
            if path.is_dir() {
                stack.push(path);
            }
        }
    }
    Ok(result)
}

pub fn rmdir_all(path: &str) -> Result<(), std::io::Error> {
    fs::remove_dir_all(path)
}

#[must_use]
pub fn gettempdir() -> String {
    std::env::temp_dir().display().to_string()
}

pub fn makedirs(path: &str) -> Result<(), std::io::Error> {
    mkdir(path)
}

pub fn touch(path: &str) -> Result<(), std::io::Error> {
    OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(path)
        .map(|_| ())
}

pub fn resolve_path(path: &str) -> Result<String, std::io::Error> {
    fs::canonicalize(path).map(|path| path.to_string_lossy().to_string())
}

pub fn iterdir(path: &str) -> Result<Vec<String>, std::io::Error> {
    fs::read_dir(path).map(|entries| {
        entries
            .filter_map(Result::ok)
            .map(|entry| entry.path().to_string_lossy().to_string())
            .collect()
    })
}

pub fn glob_pattern(dir: &str, pattern: &str) -> Result<Vec<String>, std::io::Error> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Ok(Vec::new());
    };
    let include_hidden = pattern.starts_with('.');
    let mut results = entries
        .filter_map(Result::ok)
        .filter(|entry| entry_matches_pattern(entry, pattern, include_hidden))
        .map(|entry| entry.path().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    results.sort();
    Ok(results)
}

pub fn rglob_pattern(dir: &str, pattern: &str) -> Result<Vec<String>, std::io::Error> {
    let include_hidden = pattern.starts_with('.');
    let mut stack = vec![dir.to_string()];
    let mut results = Vec::new();
    while let Some(current) = stack.pop() {
        let Ok(entries) = fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.filter_map(Result::ok) {
            if !entry_matches_hidden_policy(&entry, include_hidden) {
                continue;
            }
            let path = entry.path();
            if path.is_dir() {
                stack.push(path.to_string_lossy().to_string());
            }
            if wildcard_match(pattern, &entry.file_name().to_string_lossy()) {
                results.push(path.to_string_lossy().to_string());
            }
        }
    }
    results.sort();
    Ok(results)
}

fn entry_matches_pattern(entry: &DirEntry, pattern: &str, include_hidden: bool) -> bool {
    entry_matches_hidden_policy(entry, include_hidden)
        && wildcard_match(pattern, &entry.file_name().to_string_lossy())
}

fn entry_matches_hidden_policy(entry: &DirEntry, include_hidden: bool) -> bool {
    include_hidden || !entry.file_name().to_string_lossy().starts_with('.')
}

fn wildcard_match(pattern: &str, value: &str) -> bool {
    let pattern = pattern.chars().collect::<Vec<_>>();
    let value = value.chars().collect::<Vec<_>>();
    let (mut p, mut v) = (0, 0);
    let mut star = None;
    let mut retry_value = 0;
    while v < value.len() {
        if p < pattern.len() && (pattern[p] == '?' || pattern[p] == value[v]) {
            p += 1;
            v += 1;
        } else if p < pattern.len() && pattern[p] == '*' {
            star = Some(p);
            p += 1;
            retry_value = v;
        } else if let Some(star_pos) = star {
            p = star_pos + 1;
            retry_value += 1;
            v = retry_value;
        } else {
            return false;
        }
    }
    while p < pattern.len() && pattern[p] == '*' {
        p += 1;
    }
    p == pattern.len()
}
