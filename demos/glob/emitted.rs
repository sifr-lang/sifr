// --- stdlib: sifr.fnmatch ---
fn fnmatch(name: &String, pattern: &String) -> bool {
    return _match(name, 0 as i64, pattern, 0 as i64);
}
fn _match(name: &String, mut ni: i64, pattern: &String, mut pi: i64) -> bool {
    while pi < (pattern.chars().count() as i64) {
        let pc: Option<String> = Some({
            let Some(__indexed_char) = pattern.chars().nth(pi as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(pc) = pc {
            if pc == "*".to_string() {
                pi = pi + (1 as i64);
                if pi == (pattern.len() as i64) {
                    return true;
                }
                let mut j: i64 = ni;
                while j <= (name.chars().count() as i64) {
                    if _match(name, j, pattern, pi) {
                        return true;
                    }
                    j = j + (1 as i64);
                }
                return false;
            } else {
                if pc == "?".to_string() {
                    if ni >= (name.len() as i64) {
                        return false;
                    }
                    ni = ni + (1 as i64);
                    pi = pi + (1 as i64);
                } else {
                    if ni >= (name.len() as i64) {
                        return false;
                    }
                    let nc: Option<String> = Some({
                        let Some(__indexed_char) = name.chars().nth(ni as usize) else {
                            unreachable!(
                                "compiler-verified string index should be in range"
                            );
                        };
                        __indexed_char.to_string()
                    });
                    if let Some(nc) = nc {
                        if nc != pc {
                            return false;
                        }
                    } else {
                        return false;
                    }
                    ni = ni + (1 as i64);
                    pi = pi + (1 as i64);
                }
            }
        } else {
            return false;
        }
    }
    return ni == (name.chars().count() as i64);
}
fn fnmatch_filter(names: &Vec<String>, pattern: &String) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    for name in names.iter().cloned() {
        if fnmatch(&name, pattern) {
            result.push(name);
        }
    }
    return result;
}
fn filterfalse(names: &Vec<String>, pattern: &String) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    for name in names.iter().cloned() {
        if !(fnmatch(&name, pattern)) {
            result.push(name);
        }
    }
    return result;
}
fn fnmatchcase(name: &String, pattern: &String) -> bool {
    return _match(name, 0 as i64, pattern, 0 as i64);
}
fn _translate_literal(ch: &String) -> String {
    if ch.clone() == ".".to_string() {
        return "\\.".to_string();
    }
    if ch.clone() == "^".to_string() {
        return "\\^".to_string();
    }
    if ch.clone() == "$".to_string() {
        return "\\$".to_string();
    }
    if ch.clone() == "+".to_string() {
        return "\\+".to_string();
    }
    if ch.clone() == "(".to_string() {
        return "\\(".to_string();
    }
    if ch.clone() == ")".to_string() {
        return "\\)".to_string();
    }
    if ch.clone() == "{".to_string() {
        return "\\{".to_string();
    }
    if ch.clone() == "}".to_string() {
        return "\\}".to_string();
    }
    if ch.clone() == "[".to_string() {
        return "\\[".to_string();
    }
    if ch.clone() == "]".to_string() {
        return "\\]".to_string();
    }
    if ch.clone() == "|".to_string() {
        return "\\|".to_string();
    }
    if ch.clone() == "\\".to_string() {
        return "\\\\".to_string();
    }
    return format!("{}{}", ch, "".to_string());
}
fn translate(pattern: &String) -> String {
    let mut body: String = "".to_string();
    let mut i: i64 = 0 as i64;
    while i < (pattern.chars().count() as i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = pattern.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(ch) = ch {
            if ch == "*".to_string() {
                body = format!("{}{}", body, ".*".to_string());
            } else {
                if ch == "?".to_string() {
                    body = format!("{}{}", body, ".".to_string());
                } else {
                    body = format!("{}{}", body, _translate_literal(& ch));
                }
            }
        }
        i = i + (1 as i64);
    }
    return format!("{}{}{}", "(?s:".to_string(), body, ")\\z".to_string());
}
fn filter(names: &Vec<String>, pattern: &String) -> Vec<String> {
    return fnmatch_filter(names, pattern);
}

// --- stdlib: sifr.glob ---
fn glob(directory: &String, pattern: &String) -> Vec<String> {
    let include_hidden: bool = (((pattern.chars().count() as i64) > (0 as i64))
        && (({
            let __sifr_index_str = &pattern;
            let __sifr_index_i = 0 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        }) == Some(".".to_string())));
    let mut matches: Vec<String> = vec![];
    let __sifr_try_res: Result<(), IOError> = (|| {
        let entries: Vec<String> = std::fs::read_dir(&directory)
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect::<Vec<String>>()
            })
            .map_err(__io_err)?;
        for entry in entries.iter().cloned() {
            if (entry.len() as i64) == (0 as i64) {
                continue;
            }
            if ((!(include_hidden))
                && (({
                    let __sifr_index_str = &entry;
                    let __sifr_index_i = 0 as i64;
                    let __sifr_index_norm = if __sifr_index_i < 0 {
                        ((__sifr_index_str.chars().count() as i64) + __sifr_index_i)
                            as usize
                    } else {
                        __sifr_index_i as usize
                    };
                    __sifr_index_str
                        .chars()
                        .nth(__sifr_index_norm)
                        .map(|c| c.to_string())
                }) == Some(".".to_string())))
            {
                continue;
            }
            if fnmatch(&entry, pattern) {
                matches.push(entry);
            }
        }
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
        return vec![];
    }
    return {
        let mut __sifr_sorted_v = (matches).iter().cloned().collect::<Vec<_>>();
        __sifr_sorted_v.sort();
        __sifr_sorted_v
    };
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

fn collect_glob_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let base: String = format!("{}{}", "/tmp/sifr_glob_glob_demo_".to_string(), format!("{}", std::process::id() as i64));
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _mk: String = ({
    let __cmd = format!("{}{}", "mkdir -p ".to_string(), base);
    let __output = std::process::Command::new("sh".to_string()).arg("-c".to_string()).arg(&__cmd).output().map_err(__io_err)?;
    Ok(String::from_utf8_lossy(&__output.stdout).trim().to_string())
})?;
    let _w1: () = std::fs::write(&format!("{}{}", base, "/a.txt".to_string()), "a".to_string().as_bytes()).map(|_| ()).map_err(__io_err)?;
    let _w2: () = std::fs::write(&format!("{}{}", base, "/b.txt".to_string()), "b".to_string().as_bytes()).map(|_| ()).map_err(__io_err)?;
    let _w3: () = std::fs::write(&format!("{}{}", base, "/.hidden.txt".to_string()), "h".to_string().as_bytes()).map(|_| ()).map_err(__io_err)?;
    let txt: Vec<String> = glob(&base, &"*.txt".to_string());
    let txt_ok: bool = ((((txt.len() as i64) == (2 as i64)) && (({
    let __sifr_index_list = &txt;
    let __sifr_index_i = 0 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).cloned()
}) == Some("a.txt".to_string()))) && (({
    let __sifr_index_list = &txt;
    let __sifr_index_i = 1 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).cloned()
}) == Some("b.txt".to_string())));
    actual.push(txt_ok);
    let hidden: Vec<String> = glob(&base, &".*.txt".to_string());
    let hidden_ok: bool = (((hidden.len() as i64) == (1 as i64)) && (({
    let __sifr_index_list = &hidden;
    let __sifr_index_i = 0 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).cloned()
}) == Some(".hidden.txt".to_string())));
    actual.push(hidden_ok);
    let wildcard_q: Vec<String> = glob(&base, &"?.txt".to_string());
    let wildcard_q_ok: bool = ((((wildcard_q.len() as i64) == (2 as i64)) && (({
    let __sifr_index_list = &wildcard_q;
    let __sifr_index_i = 0 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).cloned()
}) == Some("a.txt".to_string()))) && (({
    let __sifr_index_list = &wildcard_q;
    let __sifr_index_i = 1 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).cloned()
}) == Some("b.txt".to_string())));
    actual.push(wildcard_q_ok);
    let none: Vec<String> = glob(&base, &"*.csv".to_string());
    actual.push((none.len() as i64) == (0 as i64));
    let missing: Vec<String> = glob(&format!("{}{}", base, "_missing".to_string()), &"*.txt".to_string());
    actual.push((missing.len() as i64) == (0 as i64));
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
        actual = vec![false, false, false, false, false];
    }
    return actual;
}

fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true];
    let actual: Vec<bool> = collect_glob_actual();
    assert_bool_vector_eq(&actual, &expected);
    println!("glob glob parity demo: pass");
}
