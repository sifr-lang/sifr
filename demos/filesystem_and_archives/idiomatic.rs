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

fn glob_txt(directory: &Path) -> io::Result<Vec<String>> {
    let mut names = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) == Some("txt") {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    names.sort();
    Ok(names)
}

fn gzip_compress(text: &str) -> io::Result<Vec<u8>> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(text.as_bytes())?;
    encoder
        .finish()
        .map_err(|error| io::Error::other(error.to_string()))
}

fn gzip_decompress(data: &[u8]) -> io::Result<String> {
    let mut decoder = flate2::read::GzDecoder::new(data);
    let mut out = String::new();
    decoder.read_to_string(&mut out)?;
    Ok(out)
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
            .map_err(|error| io::Error::other(error.to_string()))?;
        Ok(())
    }

    fn write(&self, name: &str, text: &str) -> io::Result<()> {
        rewrite_zip_with_entry(&self.path, name, text.as_bytes())
    }

    fn read(&self, name: &str) -> io::Result<String> {
        let file = File::open(&self.path)?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|error| io::Error::other(error.to_string()))?;
        let mut entry = archive
            .by_name(name)
            .map_err(|error| io::Error::other(error.to_string()))?;
        let mut out = String::new();
        entry.read_to_string(&mut out)?;
        Ok(out)
    }

    fn namelist(&self) -> io::Result<Vec<String>> {
        let file = File::open(&self.path)?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|error| io::Error::other(error.to_string()))?;
        let mut names = Vec::new();
        for index in 0..archive.len() {
            let file = archive
                .by_index(index)
                .map_err(|error| io::Error::other(error.to_string()))?;
            names.push(file.name().to_string());
        }
        Ok(names)
    }
}

fn rewrite_zip_with_entry(path: &Path, name: &str, content: &[u8]) -> io::Result<()> {
    let existing_entries = if path.exists() {
        let file = File::open(path)?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|error| io::Error::other(error.to_string()))?;
        let mut entries = Vec::new();
        for index in 0..archive.len() {
            let mut file = archive
                .by_index(index)
                .map_err(|error| io::Error::other(error.to_string()))?;
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
            .map_err(|error| io::Error::other(error.to_string()))?;
        writer.write_all(&existing_data)?;
    }

    writer
        .start_file(name.to_string(), zip::write::SimpleFileOptions::default())
        .map_err(|error| io::Error::other(error.to_string()))?;
    writer.write_all(content)?;
    writer
        .finish()
        .map_err(|error| io::Error::other(error.to_string()))?;

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
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

fn run_demo(base: &Path) -> io::Result<()> {
    fs::create_dir_all(base)?;

    let source = base.join("note.txt");
    fs::write(&source, "hello d1")?;
    let note_content = fs::read_to_string(&source)?;
    println!("io.read_text = {note_content}");

    let stem = source
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    println!("pathlib.stem = {stem}");
    println!("glob(\"*.txt\") = {:?}", glob_txt(base)?);

    let copied = base.join("copied.txt");
    let moved = base.join("moved.txt");
    fs::copy(&source, &copied)?;
    fs::rename(&copied, &moved)?;
    println!("shutil.move_file exists = {}", moved.exists());

    let temp_file = mkstemp("sifr_filesystem_archive_surface_demo_")?;
    let temp_dir = mkdtemp("sifr_filesystem_archive_surface_demo_")?;
    println!("tempfile.mkstemp = {}", temp_file.display());
    println!("tempfile.mkdtemp = {}", temp_dir.display());

    let compressed = gzip_compress("archive sample")?;
    println!("gzip roundtrip = {}", gzip_decompress(&compressed)?);

    let archive = ZipFile::new(base.join("demo.zip"));
    archive.create()?;
    archive.write("inside.txt", "inside-zip")?;
    println!("zipfile.read = {}", archive.read("inside.txt")?);
    println!(
        "zipfile.namelist = {:?}",
        Ok::<Vec<String>, io::Error>(archive.namelist()?)
    );

    remove_path(&temp_file)?;
    remove_path(&temp_dir)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let base = PathBuf::from(format!("/tmp/sifr_filesystem_archive_surface_demo_{}", std::process::id()));
    let result = run_demo(&base);
    let cleanup = remove_path(&base);
    result.and(cleanup)
}
