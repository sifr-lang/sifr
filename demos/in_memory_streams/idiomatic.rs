use std::fs::{self, File};
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct IOError(String);

impl std::fmt::Display for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for IOError {}

fn negative_seek_error(offset: i64) -> IOError {
    IOError(format!("negative seek position: {offset}"))
}

struct StringIO {
    buffer: Cursor<Vec<u8>>,
}

impl StringIO {
    fn new(initial: &str) -> Self {
        Self {
            buffer: Cursor::new(initial.as_bytes().to_vec()),
        }
    }

    fn write(&mut self, text: &str) -> io::Result<()> {
        self.buffer.write_all(text.as_bytes())
    }

    fn seek(&mut self, offset: i64) -> Result<i64, IOError> {
        if offset < 0 {
            return Err(negative_seek_error(offset));
        }
        self.buffer
            .seek(SeekFrom::Start(offset as u64))
            .map(|pos| pos as i64)
            .map_err(|error| IOError(error.to_string()))
    }

    fn read(&mut self) -> Result<String, IOError> {
        let mut out = String::new();
        self.buffer
            .read_to_string(&mut out)
            .map_err(|error| IOError(error.to_string()))?;
        Ok(out)
    }
}

struct BytesIO {
    buffer: Cursor<Vec<u8>>,
}

impl BytesIO {
    fn new(initial: &[u8]) -> Self {
        Self {
            buffer: Cursor::new(initial.to_vec()),
        }
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.buffer.write_all(bytes)
    }

    fn seek(&mut self, offset: i64) -> Result<i64, IOError> {
        if offset < 0 {
            return Err(negative_seek_error(offset));
        }
        self.buffer
            .seek(SeekFrom::Start(offset as u64))
            .map(|pos| pos as i64)
            .map_err(|error| IOError(error.to_string()))
    }

    fn read_bytes(&mut self) -> io::Result<Vec<u8>> {
        let mut out = Vec::new();
        self.buffer.read_to_end(&mut out)?;
        Ok(out)
    }
}

struct BinaryFileHandle {
    file: Option<File>,
}

impl BinaryFileHandle {
    fn open(path: &Path, mode: &str) -> io::Result<Self> {
        let file = match mode {
            "wb" => File::create(path)?,
            "rb" => File::open(path)?,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "unsupported mode",
                ))
            }
        };
        Ok(Self { file: Some(file) })
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        match self.file.as_mut() {
            Some(file) => file.write_all(bytes),
            None => Err(io::Error::other("stream is closed")),
        }
    }

    fn read_bytes(&mut self) -> io::Result<Vec<u8>> {
        let mut out = Vec::new();
        match self.file.as_mut() {
            Some(file) => {
                file.read_to_end(&mut out)?;
                Ok(out)
            }
            None => Err(io::Error::other("stream is closed")),
        }
    }

    fn close(&mut self) {
        self.file = None;
    }
}

fn cleanup(path: &Path) -> bool {
    !path.exists() || fs::remove_file(path).is_ok()
}

fn main() {
    let path = PathBuf::from("/tmp/sifr_runtime_in_memory_streams.bin");

    let mut stringio = StringIO::new("sample");
    let stringio_ok = stringio.write("1").is_ok()
        && stringio.seek(0).ok() == Some(0)
        && stringio.read().ok().as_deref() == Some("1ample");
    let stringio_negative_seek_ok = stringio.seek(-1).is_err();

    let mut bytesio = BytesIO::new(b"abc");
    let bytesio_ok = bytesio.seek(3).ok() == Some(3)
        && bytesio.write_bytes(b"d").is_ok()
        && bytesio.seek(0).ok() == Some(0)
        && bytesio.read_bytes().is_ok_and(|bytes| bytes == b"abcd");
    let bytesio_negative_seek_ok = bytesio.seek(-1).is_err();

    let mut binary_file_ok = false;
    if let Ok(mut writer) = BinaryFileHandle::open(&path, "wb") {
        if writer.write_bytes(b"runtime-in_memory_streams").is_ok() {
            writer.close();
            if let Ok(mut reader) = BinaryFileHandle::open(&path, "rb") {
                binary_file_ok = reader
                    .read_bytes()
                    .is_ok_and(|loaded| loaded == b"runtime-in_memory_streams");
                reader.close();
            }
        }
    }

    let cleanup_ok = cleanup(&path) && !path.exists();

    assert!(stringio_ok);
    assert!(stringio_negative_seek_ok);
    assert!(bytesio_ok);
    assert!(bytesio_negative_seek_ok);
    assert!(binary_file_ok);
    assert!(cleanup_ok);
    println!("runtime_in_memory_streams_in_memory_hierarchy_demo: ok");
}
