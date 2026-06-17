// --- stdlib: sifr.shutil ---
fn copy(src: &String, dst: &String) -> Result<(), IOError> {
    return std::fs::copy(&src, &dst).map(|_| ()).map_err(__io_err);
}
fn move_file(src: &String, dst: &String) -> Result<(), IOError> {
    return std::fs::rename(&src, &dst).map(|_| ()).map_err(__io_err);
}
fn rmtree(path: &String) -> Result<(), IOError> {
    return std::fs::remove_dir_all(&path).map(|_| ()).map_err(__io_err);
}

// --- stdlib: sifr.test ---
fn assert_eq<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    actual: &T,
    expected: &T,
) {
    assert!(* actual == * expected);
}
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0 as i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize]) == expected.get(i as usize).copied());
        i = i + (1 as i64);
    }
}

// --- stdlib: sifr.tempfile ---
fn _random_suffix() -> String {
    let n: i64 = {
        let __start = 100000 as i64;
        let __end = 999999 as i64;
        __start + rand::RngExt::random_range(&mut rand::rng(), 0..(__end - __start) + 1)
    };
    return format!("{}", n);
}
fn mktemp_path(prefix: &String) -> String {
    let suffix: String = _random_suffix();
    let mut root: String = std::env::temp_dir().display().to_string();
    if (root.chars().count() as i64) == (0 as i64) {
        root = "/tmp".to_string();
    } else {
        let last: Option<String> = {
            let __sifr_index_str = &root;
            let __sifr_index_i = (root.chars().count() as i64) - (1 as i64);
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        if let Some(last) = last {
            if last == "/".to_string() {
                root = String::from_iter(
                    (root)
                        .chars()
                        .skip(0 as usize)
                        .take(
                            (((root.chars().count() as i64) - (1 as i64)).max(0) - 0)
                                .max(0) as usize,
                        ),
                );
            }
        }
    }
    return format!("{}{}{}{}", root, "/".to_string(), prefix, suffix);
}

#[derive(Debug, Clone)]
struct IOError {
    message: String,
    kind: String,
}

impl IOError {
    fn new(message: String) -> Self {
        return Self { message: message, kind: "Other".to_string() };
    }
}

impl std::fmt::Display for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for IOError {
}

fn __io_err(e: std::io::Error) -> IOError {
    let msg = e.to_string();
    let kind = if e.kind() == std::io::ErrorKind::NotFound { "FileNotFound".to_string() } else { if e.kind() == std::io::ErrorKind::PermissionDenied { "PermissionDenied".to_string() } else { if e.kind() == std::io::ErrorKind::AlreadyExists { "FileExists".to_string() } else { "Other".to_string() } } };
    return IOError { message: msg, kind: kind };
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

impl std::error::Error for ParseError {
}

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

impl std::error::Error for ValueError {
}

#[derive(Debug, Clone)]
struct JSONDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl JSONDecodeError {
    fn new(message: String) -> Self {
        return Self { message: message, line: 0, column: 0 };
    }
}

impl std::fmt::Display for JSONDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for JSONDecodeError {
}

#[derive(Debug, Clone)]
struct TOMLDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl TOMLDecodeError {
    fn new(message: String) -> Self {
        return Self { message: message, line: 0, column: 0 };
    }
}

impl std::fmt::Display for TOMLDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for TOMLDecodeError {
}

#[derive(Debug, Clone)]
struct RegexError {
    message: String,
    detail: String,
}

impl RegexError {
    fn new(message: String) -> Self {
        return Self { message: message, detail: String::new() };
    }
}

impl std::fmt::Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for RegexError {
}

fn collect_copy_move_tree_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let base: String = mktemp_path(&"sifr_shutil_shutil_demo_".to_string());
    let src: String = format!("{}{}", base, "/src.txt".to_string());
    let copied: String = format!("{}{}", base, "/copied.txt".to_string());
    let moved: String = format!("{}{}", base, "/moved.txt".to_string());
    let tree: String = format!("{}{}", base, "/tree".to_string());
    let nested: String = format!("{}{}", tree, "/nested.txt".to_string());
    let mut copy_ok: bool = false;
    let mut move_ok: bool = false;
    let mut rmtree_ok: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _mk: () = std::fs::create_dir_all(&base).map(|_| ()).map_err(__io_err)?;
    let _w: () = std::fs::write(&src, "demo".to_string().as_bytes()).map(|_| ()).map_err(__io_err)?;
    let _cp: () = copy(&src, &copied)?;
    let mut copied_content_ok: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
    let copied_content: String = std::fs::read_to_string(&copied).map_err(__io_err)?;
    copied_content_ok = copied_content == "demo".to_string();
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
    }
    copy_ok = (((std::path::Path::new(&src).exists()) && (std::path::Path::new(&copied).exists())) && (copied_content_ok));
    let _mv: () = move_file(&copied, &moved)?;
    move_ok = ((std::path::Path::new(&moved).exists()) && (!(std::path::Path::new(&copied).exists())));
    let _mk_tree: () = std::fs::create_dir_all(&tree).map(|_| ()).map_err(__io_err)?;
    let _w_nested: () = std::fs::write(&nested, "nested".to_string().as_bytes()).map(|_| ()).map_err(__io_err)?;
    let _rm_tree: () = rmtree(&tree)?;
    rmtree_ok = !(std::path::Path::new(&tree).exists());
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
    }
    actual.push(copy_ok);
    actual.push(move_ok);
    actual.push(rmtree_ok);
    return actual;
}

fn collect_tooling_and_cleanup_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let base: String = mktemp_path(&"sifr_shutil_shutil_demo_cleanup_".to_string());
    let mut base_ready: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _mk: () = std::fs::create_dir_all(&base).map(|_| ()).map_err(__io_err)?;
    base_ready = std::path::Path::new(&base).exists();
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
    }
    let mut which_ok: bool = false;
    let tool: Option<String> = std::env::var("PATH".to_string()).ok().and_then(|__path| __path.split(':').map(|d| std::path::Path::new(d).join(&"sh".to_string())).find(|p| p.is_file()).map(|p| p.to_string_lossy().to_string()));
    if let Some(tool) = tool {
        which_ok = (tool.chars().count() as i64) > (0 as i64);
    }
    actual.push(which_ok);
    let usage: Vec<i64> = {
    let __path = format!("{}{}", base, "".to_string());
    let __meta_ok = std::fs::metadata(&__path).is_ok();
    if __meta_ok { {
    let __out = std::process::Command::new("df".to_string()).arg("-k".to_string()).arg(&__path).output();
    {
    let __s = __out.as_ref().map_or("".to_string(), |__o| String::from_utf8_lossy(&__o.stdout).to_string());
    let __lines = __s.lines().collect::<Vec<&str>>();
    if __lines.len() >= 2 { {
    let __parts = __lines[1].split_whitespace().collect::<Vec<&str>>();
    if __parts.len() >= 4 { {
    let __total = __parts[1].parse::<i64>().unwrap_or(0) * 1024;
    let __used = __parts[2].parse::<i64>().unwrap_or(0) * 1024;
    let __free = __parts[3].parse::<i64>().unwrap_or(0) * 1024;
    vec![__total, __used, __free]
} } else { vec![0, 0, 0] }
} } else { vec![0, 0, 0] }
}
} } else { vec![0, 0, 0] }
};
    let mut usage_ok: bool = false;
    if (usage.len() as i64) == (3 as i64) {
        let total: Option<i64> = Some(usage[(0 as i64) as usize]);
        if let Some(total) = total {
            usage_ok = total > (0 as i64);
        }
    }
    usage_ok = usage_ok && base_ready;
    actual.push(usage_ok);
    let mut missing_copy_rejected: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _bad: () = copy(&format!("{}{}", base, "/missing_src.txt".to_string()), &format!("{}{}", base, "/missing_dst.txt".to_string()))?;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
        missing_copy_rejected = true;
    }
    actual.push(missing_copy_rejected);
    let mut cleanup_ok: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _cleanup: String = ({
    let __cmd = format!("{}{}", "rm -rf ".to_string(), base);
    let __output = std::process::Command::new("sh".to_string()).arg("-c".to_string()).arg(&__cmd).output().map_err(__io_err)?;
    Ok(String::from_utf8_lossy(&__output.stdout).trim().to_string())
})?;
    cleanup_ok = !(std::path::Path::new(&base).exists());
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
        cleanup_ok = !(std::path::Path::new(&base).exists());
    }
    actual.push(cleanup_ok);
    return actual;
}

fn append_all(target: &mut Vec<bool>, values: &Vec<bool>) {
    for value in values.iter().copied() {
        target.push(value);
    }
}

fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true, true];
    let mut actual: Vec<bool> = vec![];
    append_all(&mut actual, &collect_copy_move_tree_actual());
    append_all(&mut actual, &collect_tooling_and_cleanup_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("shutil shutil parity demo: pass");
}
