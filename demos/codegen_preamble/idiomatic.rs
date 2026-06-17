use std::fs;
use std::io;

const INFO: i64 = 20;

struct Logger {
    name: String,
    level: i64,
}

impl Logger {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            level: INFO,
        }
    }

    fn set_level(&mut self, level: i64) {
        self.level = level;
    }

    fn info(&self, message: &str) {
        if self.level <= INFO {
            println!("[INFO] {}: {}", self.name, message);
        }
    }
}

fn write_text(path: &str, contents: &str) -> io::Result<()> {
    fs::write(path, contents)
}

fn main() {
    let path = "/tmp/sifr_codegen_preamble_demo.txt";

    match (|| -> io::Result<String> {
        write_text(path, "codegen preamble")?;
        fs::read_to_string(path)
    })() {
        Ok(text) => {
            println!("file = {text}");
            assert_eq!(format!("file = {text}"), "file = codegen preamble");
        }
        Err(err) => println!("ioerror = {err}"),
    }

    let mut log = Logger::new("codegen");
    log.set_level(INFO);
    log.info("preamble logging alive");
    println!("preamble demo complete");
}
