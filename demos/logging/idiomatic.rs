use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const NOTSET: i64 = 0;
const DEBUG: i64 = 10;
const INFO: i64 = 20;
const WARNING: i64 = 30;
const ERROR: i64 = 40;

static TEMP_SUFFIX: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
struct Formatter {
    template: String,
}

impl Formatter {
    fn new(template: impl Into<String>) -> Self {
        Self {
            template: template.into(),
        }
    }

    fn render(&self, level: &str, name: &str, message: &str) -> String {
        self.template
            .replace("%(levelname)s", level)
            .replace("%(name)s", name)
            .replace("%(message)s", message)
    }
}

#[derive(Debug, Clone)]
struct FileHandler {
    path: PathBuf,
    level: i64,
    formatter: Formatter,
}

impl FileHandler {
    fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            level: NOTSET,
            formatter: Formatter::new("%(levelname)s:%(name)s:%(message)s"),
        }
    }

    fn set_level(&mut self, level: i64) {
        self.level = level;
    }

    fn set_formatter(&mut self, formatter: Formatter) {
        self.formatter = formatter;
    }

    fn emit(&self, level: &str, name: &str, message: &str) {
        if level_value(level) < self.level {
            return;
        }

        let line = format!("{}\n", self.formatter.render(level, name, message));
        // Logging failure should not take down the demo path being illustrated.
        let _ = append_line(&self.path, &line);
    }
}

#[derive(Debug, Clone)]
struct Logger {
    name: String,
    level: i64,
    output_path: Option<PathBuf>,
}

impl Logger {
    fn new(name: impl Into<String>, level: i64) -> Self {
        Self {
            name: name.into(),
            level,
            output_path: None,
        }
    }

    fn set_level(&mut self, level: i64) {
        self.level = level;
    }

    fn set_file(&mut self, path: impl Into<PathBuf>) {
        self.output_path = Some(path.into());
    }

    fn debug(&self, message: &str) {
        self.log("DEBUG", message);
    }

    fn info(&self, message: &str) {
        self.log("INFO", message);
    }

    fn warning(&self, message: &str) {
        self.log("WARNING", message);
    }

    fn error(&self, message: &str) {
        self.log("ERROR", message);
    }

    fn log(&self, level: &str, message: &str) {
        if level_value(level) < self.level {
            return;
        }

        let Some(path) = &self.output_path else {
            return;
        };

        let line = format!("[{level}] {}: {message}\n", self.name);
        // Match the demo's "best effort" logging behavior on write failures.
        let _ = append_line(path, &line);
    }
}

fn append_line(path: &Path, line: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new().append(true).create(true).open(path)?;
    file.write_all(line.as_bytes())
}

fn level_value(level: &str) -> i64 {
    match level {
        "DEBUG" => DEBUG,
        "INFO" => INFO,
        "WARNING" => WARNING,
        "ERROR" => ERROR,
        _ => NOTSET,
    }
}

fn get_logger(name: &str) -> Logger {
    Logger::new(name, INFO)
}

fn basic_config(level: i64) -> Logger {
    Logger::new("root", level)
}

fn mktemp_path(prefix: &str) -> PathBuf {
    let suffix = TEMP_SUFFIX.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}{}-{suffix}", std::process::id()))
}

fn collect_logger_actual(base: &Path) -> Vec<bool> {
    let app_log = base.join("app.log");
    let app_ok = (|| -> std::io::Result<bool> {
        fs::write(&app_log, "")?;
        let mut app = get_logger("demo");
        app.set_file(&app_log);
        app.set_level(INFO);
        app.info("start");
        app.warning("warn");
        app.debug("hidden");
        Ok(fs::read_to_string(&app_log)? == "[INFO] demo: start\n[WARNING] demo: warn\n")
    })()
    .unwrap_or(false);

    vec![app_ok]
}

fn collect_root_and_handler_actual(base: &Path) -> Vec<bool> {
    let root_log = base.join("root.log");
    let handler_log = base.join("handler.log");

    let (root_ok, handler_ok) = (|| -> std::io::Result<(bool, bool)> {
        fs::write(&root_log, "")?;
        fs::write(&handler_log, "")?;

        let mut root = basic_config(WARNING);
        root.set_file(&root_log);
        root.info("skip");
        root.error("boom");

        let mut handler = FileHandler::new(&handler_log);
        handler.set_level(INFO);
        handler.set_formatter(Formatter::new("%(levelname)s:%(message)s"));
        handler.emit("INFO", "demo", "formatted");

        Ok((
            fs::read_to_string(&root_log)? == "[ERROR] root: boom\n",
            fs::read_to_string(&handler_log)? == "INFO:formatted\n",
        ))
    })()
    .unwrap_or((false, false));

    vec![root_ok, handler_ok]
}

fn collect_safety_actual(base: &Path) -> Vec<bool> {
    let missing_log = base.join("missing").join("blocked.log");

    let mut bad = get_logger("bad");
    bad.set_file(&missing_log);
    // This corpus treats logging as best-effort: write failures stay non-fatal.
    bad.error("should fail");

    vec![!missing_log.exists(), INFO == 20 && WARNING == 30]
}

fn collect_cleanup_actual(base: &Path) -> Vec<bool> {
    let cleanup_ok = match fs::remove_dir_all(base) {
        Ok(()) => !base.exists(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
        Err(_) => false,
    };

    vec![cleanup_ok]
}

fn main() {
    let base = mktemp_path("sifr_logging_logging_demo_");
    let _ = fs::create_dir_all(&base);

    let mut actual = Vec::new();
    actual.extend(collect_logger_actual(&base));
    actual.extend(collect_root_and_handler_actual(&base));
    actual.extend(collect_safety_actual(&base));
    actual.extend(collect_cleanup_actual(&base));

    let expected = vec![true, true, true, true, true, true];
    assert_eq!(actual, expected);
    println!("logging logging parity demo: pass");
}
