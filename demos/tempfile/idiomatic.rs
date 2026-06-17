use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static UNIQUE_COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let counter = UNIQUE_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{nanos}_{counter}")
}

fn mktemp_path(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{prefix}{}", unique_suffix()))
}

fn mkstemp(prefix: &str) -> io::Result<PathBuf> {
    for _ in 0..64 {
        let path = mktemp_path(prefix);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(_) => return Ok(path),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "tempfile.mkstemp: failed to create unique path after 64 attempts",
    ))
}

fn mkdtemp(prefix: &str) -> io::Result<PathBuf> {
    for _ in 0..64 {
        let path = mktemp_path(prefix);
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "tempfile.mkdtemp: failed to create unique path after 64 attempts",
    ))
}

fn remove_path(path: &Path) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

fn has_prefix(path: &Path, prefix: &str) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with(prefix) && name.len() > prefix.len())
}

fn collect_tempfile_actual() -> Vec<bool> {
    let preview_path = mktemp_path("sifr_tempfile_preview_");

    let result: io::Result<Vec<bool>> = (|| {
        let file_path = mkstemp("sifr_tempfile_tmp_")?;
        let dir_path = mkdtemp("sifr_tempfile_tmpd_")?;

        let mut actual = vec![file_path.exists(), dir_path.exists()];
        actual.push(
            has_prefix(&preview_path, "sifr_tempfile_preview_")
                && has_prefix(&file_path, "sifr_tempfile_tmp_")
                && has_prefix(&dir_path, "sifr_tempfile_tmpd_"),
        );

        let temp_root = preview_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(std::env::temp_dir);
        let missing_parent_name = "__sifr_tempfile_missing_parent__";
        let missing_parent_path = temp_root.join(missing_parent_name);
        remove_path(&missing_parent_path)?;

        let missing_prefix = format!("{missing_parent_name}/bad_");
        actual.push(mkstemp(&missing_prefix).is_err());

        remove_path(&file_path)?;
        remove_path(&dir_path)?;
        remove_path(&missing_parent_path)?;
        actual.push(!file_path.exists() && !dir_path.exists());

        let next_path = mkstemp("sifr_tempfile_tmp_")?;
        actual.push(next_path != file_path);
        remove_path(&next_path)?;

        Ok(actual)
    })();

    result.unwrap_or_else(|_| vec![false; 6])
}

fn main() {
    let expected = vec![true, true, true, true, true, true];
    let actual = collect_tempfile_actual();

    assert_eq!(actual, expected);
    println!("tempfile tempfile parity demo: pass");
}
