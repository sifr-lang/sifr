use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
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

fn unique_temp_path(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{prefix}{}", unique_suffix()))
}

fn mkstemp(prefix: &str) -> io::Result<PathBuf> {
    for _ in 0..64 {
        let path = unique_temp_path(prefix);
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
        let path = unique_temp_path(prefix);
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

#[derive(Debug, Clone)]
struct ZipFile {
    path: PathBuf,
}

impl ZipFile {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn create(&self) -> io::Result<()> {
        let file = File::create(&self.path)?;
        zip::ZipWriter::new(file)
            .finish()
            .map_err(|err| io::Error::other(err.to_string()))?;
        Ok(())
    }

    fn write(&self, name: &str, text: &str) -> io::Result<()> {
        rewrite_zip_with_entry(&self.path, name, text.as_bytes())
    }

    fn namelist(&self) -> io::Result<Vec<String>> {
        let file = File::open(&self.path)?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|err| io::Error::other(err.to_string()))?;
        let mut names = Vec::new();
        for index in 0..archive.len() {
            let file = archive
                .by_index(index)
                .map_err(|err| io::Error::other(err.to_string()))?;
            names.push(file.name().to_string());
        }
        Ok(names)
    }

    fn read(&self, name: &str) -> io::Result<String> {
        let file = File::open(&self.path)?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|err| io::Error::other(err.to_string()))?;
        let mut entry = archive
            .by_name(name)
            .map_err(|err| io::Error::other(err.to_string()))?;
        let mut out = String::new();
        entry.read_to_string(&mut out)?;
        Ok(out)
    }
}

fn rewrite_zip_with_entry(path: &Path, name: &str, content: &[u8]) -> io::Result<()> {
    let existing_entries = if path.exists() {
        let file = File::open(path)?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|err| io::Error::other(err.to_string()))?;
        let mut entries = Vec::new();
        for index in 0..archive.len() {
            let mut file = archive
                .by_index(index)
                .map_err(|err| io::Error::other(err.to_string()))?;
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            entries.push((file.name().to_string(), data));
        }
        entries
    } else {
        Vec::new()
    };

    let temp_path = path.with_extension("tmp");
    let temp_file = File::create(&temp_path)?;
    let mut writer = zip::ZipWriter::new(temp_file);

    for (existing_name, existing_data) in existing_entries {
        writer
            .start_file(existing_name, zip::write::SimpleFileOptions::default())
            .map_err(|err| io::Error::other(err.to_string()))?;
        writer.write_all(&existing_data)?;
    }

    writer
        .start_file(name.to_string(), zip::write::SimpleFileOptions::default())
        .map_err(|err| io::Error::other(err.to_string()))?;
    writer.write_all(content)?;
    writer
        .finish()
        .map_err(|err| io::Error::other(err.to_string()))?;

    if path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(temp_path, path)
}

fn remove_path(path: &Path) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        fs::remove_dir(path)
    } else {
        fs::remove_file(path)
    }
}

fn main() {
    let mut temp_file = PathBuf::new();
    let mut temp_dir = PathBuf::new();
    let mut zip_path = PathBuf::new();

    let mut tempfile_ok = false;
    let mut zip_ok = false;

    if let Ok(created_file) = mkstemp("sifr_runtime_tempfiles_and_zip_") {
        if let Ok(created_dir) = mkdtemp("sifr_runtime_tempfiles_and_zip_") {
            temp_file = created_file;
            temp_dir = created_dir;
            tempfile_ok = temp_file.exists() && temp_dir.exists();

            if fs::write(&temp_file, "payload").is_ok() {
                zip_path = PathBuf::from(format!("{}.zip", temp_file.display()));
                let archive = ZipFile::new(zip_path.clone());
                if archive.create().is_ok() && archive.write("entry.txt", "payload").is_ok() {
                    if let (Ok(names), Ok(content)) =
                        (archive.namelist(), archive.read("entry.txt"))
                    {
                        zip_ok = names == vec!["entry.txt".to_string()] && content == "payload";
                    }
                }
            }
        }
    }

    let cleanup_ok = remove_path(&zip_path).is_ok()
        && remove_path(&temp_file).is_ok()
        && remove_path(&temp_dir).is_ok()
        && (temp_file.as_os_str().is_empty() || !temp_file.exists())
        && (temp_dir.as_os_str().is_empty() || !temp_dir.exists())
        && (zip_path.as_os_str().is_empty() || !zip_path.exists());

    assert!(tempfile_ok);
    assert!(zip_ok);
    assert!(cleanup_ok);
    println!("runtime_tempfiles_and_zip_zip_lifecycle_demo: ok");
}
