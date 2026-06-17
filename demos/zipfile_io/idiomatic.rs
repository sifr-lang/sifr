use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const ZIP_STORED: i64 = 0;

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

fn create_unique_file(prefix: &str) -> io::Result<PathBuf> {
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
        "failed to create unique temporary file after 64 attempts",
    ))
}

#[derive(Debug, Clone)]
struct NamedTemporaryFile {
    path: PathBuf,
    delete: bool,
    closed: bool,
    cleaned: bool,
}

impl NamedTemporaryFile {
    fn new(_mode: &str, delete: bool, prefix: &str) -> io::Result<Self> {
        Ok(Self {
            path: create_unique_file(prefix)?,
            delete,
            closed: false,
            cleaned: false,
        })
    }

    fn name(&self) -> String {
        self.path.display().to_string()
    }

    fn close(&mut self) -> io::Result<()> {
        self.closed = true;
        if self.delete {
            self.cleanup()?;
        }
        Ok(())
    }

    fn cleanup(&mut self) -> io::Result<()> {
        self.closed = true;
        if !self.cleaned && self.path.exists() {
            fs::remove_file(&self.path)?;
        }
        self.cleaned = true;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ZipReadHandle {
    data: Vec<u8>,
    cursor: usize,
}

impl ZipReadHandle {
    fn new(data: Vec<u8>) -> Self {
        Self { data, cursor: 0 }
    }

    fn read_bytes(&mut self, size: i64) -> Vec<u8> {
        let end = if size < 0 {
            self.data.len()
        } else {
            self.cursor
                .saturating_add(size as usize)
                .min(self.data.len())
        };
        let out = self.data[self.cursor..end].to_vec();
        self.cursor = end;
        out
    }
}

#[derive(Debug, Clone)]
struct ZipFile {
    path: PathBuf,
    mode: String,
    compression: i64,
}

impl ZipFile {
    fn new(path: &str, mode: &str, compression: i64) -> Self {
        Self {
            path: PathBuf::from(path),
            mode: mode.to_string(),
            compression,
        }
    }

    fn writable_mode(&self) -> bool {
        matches!(self.mode.as_str(), "w" | "a" | "wb" | "ab")
    }

    fn create(&self) -> io::Result<()> {
        if !self.writable_mode() {
            return Err(io::Error::other("zip file is not writable"));
        }
        let _compression = self.compression;
        let file = File::create(&self.path)?;
        zip::ZipWriter::new(file)
            .finish()
            .map_err(|err| io::Error::other(err.to_string()))?;
        Ok(())
    }

    fn write(&self, name: &str, text: &str) -> io::Result<()> {
        self.write_bytes(name, text.as_bytes())
    }

    fn write_bytes(&self, name: &str, bytes: &[u8]) -> io::Result<()> {
        if !self.writable_mode() {
            return Err(io::Error::other("zip file is not writable"));
        }
        rewrite_zip_with_entry(&self.path, name, bytes)
    }

    fn read_bytes(&self, name: &str) -> io::Result<Vec<u8>> {
        let file = File::open(&self.path)?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|err| io::Error::other(err.to_string()))?;
        let mut entry = archive
            .by_name(name)
            .map_err(|err| io::Error::other(err.to_string()))?;
        let mut out = Vec::new();
        entry.read_to_end(&mut out)?;
        Ok(out)
    }

    fn open(&self, name: &str, mode: &str) -> io::Result<ZipReadHandle> {
        if mode != "r" {
            return Err(io::Error::other(
                "ZipFile.open supports only text read mode",
            ));
        }
        Ok(ZipReadHandle::new(self.read_bytes(name)?))
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

fn is_zipfile(path: &str) -> bool {
    File::open(path)
        .ok()
        .and_then(|file| zip::ZipArchive::new(file).ok())
        .is_some()
}

fn main() {
    let zip_path = PathBuf::from("/tmp/sifr_runtime_zipfile_io.zip");

    let demo_ok = (|| -> io::Result<bool> {
        let mut temp_file = NamedTemporaryFile::new("wb", false, "sifr_runtime_zipfile_io_")?;
        let tmp_path = temp_file.name();
        temp_file.close()?;
        temp_file.cleanup()?;
        let tempfile_ok = !Path::new(&tmp_path).exists();

        if zip_path.exists() {
            fs::remove_file(&zip_path)?;
        }

        let writer = ZipFile::new(&zip_path.display().to_string(), "w", ZIP_STORED);
        writer.create()?;
        writer.write("note.txt", "runtime-zipfile_io")?;
        writer.write_bytes("bin/raw.bin", b"\x00\x01\x02")?;

        let reader = ZipFile::new(&zip_path.display().to_string(), "r", ZIP_STORED);
        let payload = reader.read_bytes("bin/raw.bin")?;

        let mut handle = ZipReadHandle::new(b"abc".to_vec());
        let handle_negative_ok = handle.read_bytes(-1) == b"abc";

        let open_rejected = reader.open("bin/raw.bin", "rb").is_err();

        let bad_mode_writer = ZipFile::new(&zip_path.display().to_string(), "rw", ZIP_STORED);
        let bad_mode_rejected = bad_mode_writer.write("bad.txt", "bad-mode").is_err();

        Ok(tempfile_ok
            && is_zipfile(&zip_path.display().to_string())
            && payload == b"\x00\x01\x02"
            && handle_negative_ok
            && open_rejected
            && bad_mode_rejected)
    })()
    .unwrap_or(false);

    if zip_path.exists() {
        let _ = fs::remove_file(&zip_path);
    }

    assert!(demo_ok);
    println!("runtime_zipfile_io_zipfile_lifecycle_demo: ok");
}
