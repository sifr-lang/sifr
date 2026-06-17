use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Lines, Write};

type IOError = io::Error;

fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(actual, expected);
}

struct FileHandle {
    lines: Lines<BufReader<File>>,
}

impl FileHandle {
    fn readline(&mut self) -> Result<Option<String>, IOError> {
        self.lines.next().transpose()
    }
}

fn read_text(path: &str) -> Result<String, IOError> {
    fs::read_to_string(path)
}

fn write_text(path: &str, content: &str) -> Result<(), IOError> {
    fs::write(path, content)
}

fn append_text(path: &str, content: &str) -> Result<(), IOError> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(content.as_bytes())
}

fn exists(path: &str) -> bool {
    std::path::Path::new(path).exists()
}

fn open(path: &str, mode: &str) -> Result<FileHandle, IOError> {
    match mode {
        "r" | "rt" => Ok(FileHandle {
            lines: BufReader::new(File::open(path)?).lines(),
        }),
        other => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unsupported mode: {other}"),
        )),
    }
}

fn collect_io_roundtrip_actual() -> Vec<bool> {
    let path = "/tmp/sifr_io_io_demo.txt";

    let text_roundtrip_ok = (|| -> Result<bool, IOError> {
        write_text(path, "hello")?;
        append_text(path, "\nworld")?;
        Ok(read_text(path)? == "hello\nworld")
    })()
    .unwrap_or(false);

    vec![text_roundtrip_ok, exists(path)]
}

fn collect_open_actual() -> Vec<bool> {
    let path = "/tmp/sifr_io_io_demo.txt";

    let (first_ok, second_ok, eof_ok) = (|| -> Result<(bool, bool, bool), IOError> {
        let mut file = open(path, "r")?;
        let first = file.readline()?;
        let second = file.readline()?;
        let third = file.readline()?;
        drop(file);

        Ok((
            first.as_deref() == Some("hello"),
            second.as_deref() == Some("world"),
            third.is_none(),
        ))
    })()
    .unwrap_or((false, false, false));

    let missing_rejected = open("/tmp/sifr_io_io_demo_missing.txt", "r").is_err();

    vec![first_ok, second_ok, eof_ok, missing_rejected]
}

fn main() {
    let mut actual = Vec::new();
    actual.extend(collect_io_roundtrip_actual());
    actual.extend(collect_open_actual());

    assert_bool_vector_eq(&actual, &[true, true, true, true, true, true]);
    println!("io io parity demo: pass");
}
