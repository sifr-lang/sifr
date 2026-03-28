// --- stdlib: sifr.shutil ---
fn copy(src: &String, dst: &String) -> Result<(), IOError> {
    return std::fs::copy(&src, &dst).map(|_| ()).map_err(__io_err);
}
fn rmtree(path: &String) -> Result<(), IOError> {
    return std::fs::remove_dir_all(&path).map(|_| ()).map_err(__io_err);
}

#[derive(Debug, Clone)]
struct IOError {
    message: String,
    kind: String,
}

impl IOError {
    fn new(message: String) -> Self {
        return Self {
            message: message,
            kind: "Other".to_string(),
        };
    }
}

impl std::fmt::Display for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for IOError {}

fn __io_err(e: std::io::Error) -> IOError {
    let msg = e.to_string();
    let kind = if e.kind() == std::io::ErrorKind::NotFound {
        "FileNotFound".to_string()
    } else {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            "PermissionDenied".to_string()
        } else {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                "FileExists".to_string()
            } else {
                "Other".to_string()
            }
        }
    };
    return IOError {
        message: msg,
        kind: kind,
    };
}

#[derive(Debug, Clone)]
struct ParseError {
    message: String,
}

impl ParseError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for ValueError {}

#[derive(Debug, Clone)]
struct JSONDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl JSONDecodeError {
    fn new(message: String) -> Self {
        return Self {
            message: message,
            line: 0,
            column: 0,
        };
    }
}

impl std::fmt::Display for JSONDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for JSONDecodeError {}

#[derive(Debug, Clone)]
struct TOMLDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl TOMLDecodeError {
    fn new(message: String) -> Self {
        return Self {
            message: message,
            line: 0,
            column: 0,
        };
    }
}

impl std::fmt::Display for TOMLDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for TOMLDecodeError {}

#[derive(Debug, Clone)]
struct RegexError {
    message: String,
    detail: String,
}

impl RegexError {
    fn new(message: String) -> Self {
        return Self {
            message: message,
            detail: String::new(),
        };
    }
}

impl std::fmt::Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for RegexError {}

fn demo_safe_read_write() {
    println!("=== Safe File Read/Write ===");
    let __sifr_try_res: Result<(), IOError> = (|| {
        let w: () = std::fs::write(
            &"/tmp/sifr_io_demo.txt".to_string(),
            "hello from sifr".to_string().as_bytes(),
        )
        .map(|_| ())
        .map_err(__io_err)?;
        let content: String =
            std::fs::read_to_string(&"/tmp/sifr_io_demo.txt".to_string()).map_err(__io_err)?;
        println!("read: {}", content);
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("error: {}", e.message);
    }
}

fn demo_file_not_found() {
    println!("=== File Not Found (no panic) ===");
    let __sifr_try_res: Result<(), IOError> = (|| {
        let data: String =
            std::fs::read_to_string(&"/tmp/sifr_io_demo_missing_file.txt".to_string())
                .map_err(__io_err)?;
        println!("should not reach here");
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("caught IOError: {}", e.message);
    }
}

fn demo_directory_ops() {
    println!("=== Directory Operations ===");
    let __sifr_try_res: Result<(), IOError> = (|| {
        let m: () = std::fs::create_dir_all(&"/tmp/sifr_io_demo_dir".to_string())
            .map(|_| ())
            .map_err(__io_err)?;
        let w: () = std::fs::write(
            &"/tmp/sifr_io_demo_dir/test.txt".to_string(),
            "inside dir".to_string().as_bytes(),
        )
        .map(|_| ())
        .map_err(__io_err)?;
        let entries: Vec<String> = std::fs::read_dir(&"/tmp/sifr_io_demo_dir".to_string())
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect::<Vec<String>>()
            })
            .map_err(__io_err)?;
        println!("entries: {}", entries.len() as i64);
        let content: String =
            std::fs::read_to_string(&"/tmp/sifr_io_demo_dir/test.txt".to_string())
                .map_err(__io_err)?;
        println!("file in dir: {}", content);
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("error: {}", e.message);
    }
    let __sifr_try_res: Result<(), IOError> = (|| {
        let bad_entries: Vec<String> =
            std::fs::read_dir(&"/tmp/sifr_io_demo_nonexistent_xyz".to_string())
                .map(|rd| {
                    rd.filter_map(|e| e.ok())
                        .map(|e| e.file_name().to_string_lossy().to_string())
                        .collect::<Vec<String>>()
                })
                .map_err(__io_err)?;
        println!("should not reach here");
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("caught listdir IOError: {}", e.message);
    }
}

fn demo_copy_and_cleanup() {
    println!("=== Copy and Cleanup ===");
    let __sifr_try_res: Result<(), IOError> = (|| {
        let c: () = copy(
            &"/tmp/sifr_io_demo.txt".to_string(),
            &"/tmp/sifr_io_demo_copy.txt".to_string(),
        )?;
        let copy_content: String =
            std::fs::read_to_string(&"/tmp/sifr_io_demo_copy.txt".to_string()).map_err(__io_err)?;
        println!("copy: {}", copy_content);
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("error: {}", e.message);
    }
    let __sifr_try_res: Result<(), IOError> = (|| {
        let r1: () = std::fs::remove_file(&"/tmp/sifr_io_demo.txt".to_string())
            .map(|_| ())
            .map_err(__io_err)?;
        let r2: () = std::fs::remove_file(&"/tmp/sifr_io_demo_copy.txt".to_string())
            .map(|_| ())
            .map_err(__io_err)?;
        let r3: () = rmtree(&"/tmp/sifr_io_demo_dir".to_string())?;
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("cleanup error: {}", e.message);
    }
}

fn demo_read_lines() {
    println!("=== Read Lines ===");
    let __sifr_try_res: Result<(), IOError> = (|| {
        let w: () = std::fs::write(
            &"/tmp/sifr_io_demo_lines.txt".to_string(),
            "line1\nline2\nline3".to_string().as_bytes(),
        )
        .map(|_| ())
        .map_err(__io_err)?;
        let lines: Vec<String> =
            std::fs::read_to_string(&"/tmp/sifr_io_demo_lines.txt".to_string())
                .map(|s| s.lines().map(|l| l.to_string()).collect::<Vec<String>>())
                .map_err(__io_err)?;
        println!("line count: {}", lines.len() as i64);
        let r: () = std::fs::remove_file(&"/tmp/sifr_io_demo_lines.txt".to_string())
            .map(|_| ())
            .map_err(__io_err)?;
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("error: {}", e.message);
    }
}

fn demo_append() {
    println!("=== Append Text ===");
    let __sifr_try_res: Result<(), IOError> = (|| {
        let w1: () = std::fs::write(
            &"/tmp/sifr_io_demo_append.txt".to_string(),
            "first".to_string().as_bytes(),
        )
        .map(|_| ())
        .map_err(__io_err)?;
        let a: () = ({
            let mut _f = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(&"/tmp/sifr_io_demo_append.txt".to_string())
                .map_err(__io_err)?;
            std::io::Write::write_all(&mut _f, " second".to_string().as_bytes())
                .map_err(__io_err)?;
            Ok(())
        })?;
        let content: String = std::fs::read_to_string(&"/tmp/sifr_io_demo_append.txt".to_string())
            .map_err(__io_err)?;
        println!("appended: {}", content);
        let r: () = std::fs::remove_file(&"/tmp/sifr_io_demo_append.txt".to_string())
            .map(|_| ())
            .map_err(__io_err)?;
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("error: {}", e.message);
    }
}

fn demo_getcwd() {
    println!("=== Get Current Directory ===");
    let __sifr_try_res: Result<(), IOError> = (|| {
        let cwd: String = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .map_err(__io_err)?;
        println!("getcwd succeeded");
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("getcwd error: {}", e.message);
    }
}

fn main() {
    demo_safe_read_write();
    demo_file_not_found();
    demo_directory_ops();
    demo_copy_and_cleanup();
    demo_read_lines();
    demo_append();
    demo_getcwd();
    println!("=== All I/O safety demos passed ===");
}
