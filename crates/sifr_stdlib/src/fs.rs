use std::{
    collections::HashMap,
    fs::{self, DirEntry, File, OpenOptions},
    io::{BufRead as _, BufReader, BufWriter, Read as _, Write as _},
    path::{Path, PathBuf},
    process::Command,
    sync::{
        atomic::{AtomicU64, Ordering},
        LazyLock, Mutex, MutexGuard,
    },
};

use sifr_runtime::interop::SifrIntBridge;

enum FileHandleEntry {
    TextRead(BufReader<File>),
    TextWrite(BufWriter<File>),
    BinaryRead(BufReader<File>),
    BinaryWrite(BufWriter<File>),
}

static FILE_HANDLES: LazyLock<Mutex<HashMap<String, FileHandleEntry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static NEXT_FILE_HANDLE_ID: AtomicU64 = AtomicU64::new(1);

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

pub fn open_file(path: &str, mode: &str) -> Result<String, std::io::Error> {
    let handle_id = next_handle_id();
    let handle = match mode {
        "r" | "rt" => FileHandleEntry::TextRead(BufReader::new(File::open(path)?)),
        "w" | "wt" => FileHandleEntry::TextWrite(BufWriter::new(File::create(path)?)),
        "a" | "at" => FileHandleEntry::TextWrite(BufWriter::new(append_file(path)?)),
        "rb" => FileHandleEntry::BinaryRead(BufReader::new(File::open(path)?)),
        "wb" => FileHandleEntry::BinaryWrite(BufWriter::new(File::create(path)?)),
        "ab" => FileHandleEntry::BinaryWrite(BufWriter::new(append_file(path)?)),
        _ => {
            return Err(std::io::Error::other(format!("invalid mode: {mode}")));
        }
    };
    file_handles().insert(handle_id.clone(), handle);
    Ok(handle_id)
}

pub fn file_read(handle: &str) -> Result<String, std::io::Error> {
    with_text_reader(handle, |reader| {
        let mut text = String::new();
        reader.read_to_string(&mut text)?;
        Ok(text)
    })
}

pub fn file_write(handle: &str, data: &str) -> Result<(), std::io::Error> {
    with_text_writer(handle, |writer| writer.write_all(data.as_bytes()))
}

pub fn file_readline(handle: &str) -> Result<Option<String>, std::io::Error> {
    with_text_reader(handle, |reader| {
        let mut line = String::new();
        let read = reader.read_line(&mut line)?;
        if read == 0 {
            return Ok(None);
        }
        trim_trailing_crlf(&mut line);
        Ok(Some(line))
    })
}

pub fn file_readlines(handle: &str) -> Result<Vec<String>, std::io::Error> {
    with_text_reader(handle, |reader| {
        let mut lines = Vec::new();
        loop {
            let mut line = String::new();
            let read = reader.read_line(&mut line)?;
            if read == 0 {
                break;
            }
            trim_trailing_crlf(&mut line);
            lines.push(line);
        }
        Ok(lines)
    })
}

pub fn file_close(handle: &str) {
    file_handles().remove(handle);
}

pub fn file_read_bytes(handle: &str) -> Result<Vec<u8>, std::io::Error> {
    with_binary_reader(handle, |reader| {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        Ok(bytes)
    })
}

pub fn file_write_bytes(handle: &str, data: &[u8]) -> Result<(), std::io::Error> {
    with_binary_writer(handle, |writer| writer.write_all(data))
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

pub fn chdir(path: &str) -> Result<(), std::io::Error> {
    std::env::set_current_dir(path)
}

pub fn stat_size(path: &str) -> Result<SifrIntBridge, std::io::Error> {
    fs::metadata(path).map(|metadata| SifrIntBridge::from(saturating_i64(metadata.len())))
}

#[must_use]
pub fn disk_usage(path: &str) -> Vec<SifrIntBridge> {
    if fs::metadata(path).is_err() {
        return zero_usage();
    }
    let Ok(output) = Command::new("df").arg("-k").arg(path).output() else {
        return zero_usage();
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let Some(line) = text.lines().nth(1) else {
        return zero_usage();
    };
    let parts = line.split_whitespace().collect::<Vec<_>>();
    if parts.len() < 4 {
        return zero_usage();
    }
    vec![
        bridge_kib_to_bytes(parts[1]),
        bridge_kib_to_bytes(parts[2]),
        bridge_kib_to_bytes(parts[3]),
    ]
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

fn append_file(path: &str) -> Result<File, std::io::Error> {
    OpenOptions::new().append(true).create(true).open(path)
}

fn next_handle_id() -> String {
    NEXT_FILE_HANDLE_ID
        .fetch_add(1, Ordering::SeqCst)
        .to_string()
}

fn file_handles() -> MutexGuard<'static, HashMap<String, FileHandleEntry>> {
    FILE_HANDLES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn with_text_reader<T>(
    handle: &str,
    read: impl FnOnce(&mut BufReader<File>) -> Result<T, std::io::Error>,
) -> Result<T, std::io::Error> {
    let mut handles = file_handles();
    match handles.get_mut(handle) {
        Some(FileHandleEntry::TextRead(reader)) => read(reader),
        _ => Err(std::io::Error::other("file not open for reading")),
    }
}

fn with_text_writer<T>(
    handle: &str,
    write: impl FnOnce(&mut BufWriter<File>) -> Result<T, std::io::Error>,
) -> Result<T, std::io::Error> {
    let mut handles = file_handles();
    match handles.get_mut(handle) {
        Some(FileHandleEntry::TextWrite(writer)) => write(writer),
        _ => Err(std::io::Error::other("file not open for writing")),
    }
}

fn with_binary_reader<T>(
    handle: &str,
    read: impl FnOnce(&mut BufReader<File>) -> Result<T, std::io::Error>,
) -> Result<T, std::io::Error> {
    let mut handles = file_handles();
    match handles.get_mut(handle) {
        Some(FileHandleEntry::BinaryRead(reader)) => read(reader),
        _ => Err(std::io::Error::other("file not open for binary reading")),
    }
}

fn with_binary_writer<T>(
    handle: &str,
    write: impl FnOnce(&mut BufWriter<File>) -> Result<T, std::io::Error>,
) -> Result<T, std::io::Error> {
    let mut handles = file_handles();
    match handles.get_mut(handle) {
        Some(FileHandleEntry::BinaryWrite(writer)) => write(writer),
        _ => Err(std::io::Error::other("file not open for binary writing")),
    }
}

fn trim_trailing_crlf(line: &mut String) {
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
}

fn bridge_kib_to_bytes(text: &str) -> SifrIntBridge {
    SifrIntBridge::from(text.parse::<i64>().unwrap_or(0).saturating_mul(1024))
}

fn zero_usage() -> Vec<SifrIntBridge> {
    vec![0_i64.into(), 0_i64.into(), 0_i64.into()]
}

fn saturating_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::{disk_usage, stat_size, write_text};
    use std::path::PathBuf;

    #[test]
    fn migrated_metadata_helpers_use_stdlib_boundary() {
        let path = temp_path("migrated_metadata_helpers_use_stdlib_boundary");
        write_text(&path.to_string_lossy(), "sifr").expect("test file should be written");

        assert_eq!(
            stat_size(&path.to_string_lossy())
                .expect("metadata should load")
                .to_i64_saturating(),
            4
        );
        assert_eq!(disk_usage("/definitely/missing/sifr/path").len(), 3);

        std::fs::remove_file(path).expect("test file should be removed");
    }

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("sifr_stdlib_fs_{name}_{}", std::process::id()))
    }
}
