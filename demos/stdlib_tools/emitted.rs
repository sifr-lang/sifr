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

// --- stdlib: sifr.tomllib ---
#[derive(Debug, Clone, PartialEq)]
struct TomlValue {
    kind: String,
    bool_value: Option<bool>,
    int_value: Option<i64>,
    float_value: Option<f64>,
    str_value: Option<String>,
    datetime_value: Option<String>,
    array_items: Box<Vec<TomlValue>>,
    table_items: Box<Vec<(String, TomlValue)>>,
}
impl TomlValue {
    fn new(
        kind: String,
        bool_value: Option<bool>,
        int_value: Option<i64>,
        float_value: Option<f64>,
        str_value: Option<String>,
        datetime_value: Option<String>,
    ) -> Self {
        return Self {
            kind: kind,
            bool_value: bool_value,
            int_value: int_value,
            float_value: float_value,
            str_value: str_value,
            datetime_value: datetime_value,
            array_items: Box::new(vec![]),
            table_items: Box::new(vec![]),
        };
    }
    fn is_bool(&self) -> bool {
        return self.kind.clone() == "bool".to_string();
    }
    fn is_int(&self) -> bool {
        return self.kind.clone() == "int".to_string();
    }
    fn is_float(&self) -> bool {
        return self.kind.clone() == "float".to_string();
    }
    fn is_str(&self) -> bool {
        return self.kind.clone() == "str".to_string();
    }
    fn is_datetime(&self) -> bool {
        return self.kind.clone() == "datetime".to_string();
    }
    fn is_array(&self) -> bool {
        return self.kind.clone() == "array".to_string();
    }
    fn is_table(&self) -> bool {
        return self.kind.clone() == "table".to_string();
    }
    fn as_bool(&self) -> Option<bool> {
        return self.bool_value;
    }
    fn as_int(&self) -> Option<i64> {
        return self.int_value;
    }
    fn as_float(&self) -> Option<f64> {
        return self.float_value;
    }
    fn as_str(&self) -> Option<String> {
        return self.str_value.clone();
    }
    fn as_datetime(&self) -> Option<String> {
        return self.datetime_value.clone();
    }
    fn as_array(&self) -> Option<Vec<TomlValue>> {
        if !(self.is_array()) {
            return None;
        }
        let mut result: Vec<TomlValue> = vec![];
        for item in (self.array_items).as_ref().clone().iter().cloned() {
            result.push(item);
        }
        return Some(result);
    }
    fn as_table(&self) -> Option<Vec<(String, TomlValue)>> {
        if !(self.is_table()) {
            return None;
        }
        let mut result: Vec<(String, TomlValue)> = vec![];
        for (key, value) in (self.table_items).as_ref().clone().iter().cloned() {
            result.push((key, value));
        }
        return Some(result);
    }
    fn at(&self, index: i64) -> Option<TomlValue> {
        if !(self.is_array()) {
            return None;
        }
        if ((index < (0 as i64))
            || (index >= ((self.array_items).as_ref().clone().len() as i64)))
        {
            return None;
        }
        let value: Option<TomlValue> = {
            let __sifr_index_list = &self.array_items;
            let __sifr_index_i = index;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        return value;
    }
    fn get(&self, key: &String) -> Option<TomlValue> {
        if !(self.is_table()) {
            return None;
        }
        for (item_key, item_value) in (self.table_items).as_ref().clone().iter().cloned()
        {
            if item_key == *key {
                return Some(item_value);
            }
        }
        return None;
    }
    fn keys(&self) -> Vec<String> {
        let mut result: Vec<String> = vec![];
        if !(self.is_table()) {
            return result;
        }
        for (item_key, _item_value) in (self.table_items)
            .as_ref()
            .clone()
            .iter()
            .cloned()
        {
            result.push(item_key);
        }
        return result;
    }
    fn values(&self) -> Vec<TomlValue> {
        let mut result: Vec<TomlValue> = vec![];
        if !(self.is_table()) {
            return result;
        }
        for (_item_key, item_value) in (self.table_items)
            .as_ref()
            .clone()
            .iter()
            .cloned()
        {
            result.push(item_value);
        }
        return result;
    }
    fn items(&self) -> Vec<(String, TomlValue)> {
        if !(self.is_table()) {
            return vec![];
        }
        let mut result: Vec<(String, TomlValue)> = vec![];
        for (key, value) in (self.table_items).as_ref().clone().iter().cloned() {
            result.push((key, value));
        }
        return result;
    }
}
fn loads(text: &String) -> Result<TomlValue, TOMLDecodeError> {
    return {
        let __toml_input = &text;
        fn __sifr_toml_value_from_parsed(
            value: toml::Value,
        ) -> Result<TomlValue, TOMLDecodeError> {
            match value {
                toml::Value::Boolean(v) => {
                    return Ok(TomlValue {
                        kind: "bool".to_string().to_string(),
                        bool_value: Some(v),
                        int_value: None,
                        float_value: None,
                        str_value: None,
                        datetime_value: None,
                        array_items: Box::new(vec![]),
                        table_items: Box::new(vec![]),
                    });
                }
                toml::Value::Integer(v) => {
                    return Ok(TomlValue {
                        kind: "int".to_string().to_string(),
                        bool_value: None,
                        int_value: Some(v),
                        float_value: None,
                        str_value: None,
                        datetime_value: None,
                        array_items: Box::new(vec![]),
                        table_items: Box::new(vec![]),
                    });
                }
                toml::Value::Float(v) => {
                    return Ok(TomlValue {
                        kind: "float".to_string().to_string(),
                        bool_value: None,
                        int_value: None,
                        float_value: Some(v),
                        str_value: None,
                        datetime_value: None,
                        array_items: Box::new(vec![]),
                        table_items: Box::new(vec![]),
                    });
                }
                toml::Value::String(v) => {
                    return Ok(TomlValue {
                        kind: "str".to_string().to_string(),
                        bool_value: None,
                        int_value: None,
                        float_value: None,
                        str_value: Some(v),
                        datetime_value: None,
                        array_items: Box::new(vec![]),
                        table_items: Box::new(vec![]),
                    });
                }
                toml::Value::Datetime(v) => {
                    return Ok(TomlValue {
                        kind: "datetime".to_string().to_string(),
                        bool_value: None,
                        int_value: None,
                        float_value: None,
                        str_value: None,
                        datetime_value: Some(v.to_string()),
                        array_items: Box::new(vec![]),
                        table_items: Box::new(vec![]),
                    });
                }
                toml::Value::Array(items) => {
                    let mut converted = vec![];
                    for item in items {
                        converted.push(__sifr_toml_value_from_parsed(item)?);
                    }
                    return Ok(TomlValue {
                        kind: "array".to_string().to_string(),
                        bool_value: None,
                        int_value: None,
                        float_value: None,
                        str_value: None,
                        datetime_value: None,
                        array_items: Box::new(converted),
                        table_items: Box::new(vec![]),
                    });
                }
                toml::Value::Table(items) => {
                    let mut converted = vec![];
                    for entry in items {
                        let entry_key = entry.0;
                        let entry_value = entry.1;
                        let converted_value = __sifr_toml_value_from_parsed(
                            entry_value,
                        )?;
                        converted.push((entry_key, converted_value));
                    }
                    return Ok(TomlValue {
                        kind: "table".to_string().to_string(),
                        bool_value: None,
                        int_value: None,
                        float_value: None,
                        str_value: None,
                        datetime_value: None,
                        array_items: Box::new(vec![]),
                        table_items: Box::new(converted),
                    });
                }
            }
        }
        __toml_input
            .parse::<toml::Value>()
            .map_err(|e| TOMLDecodeError {
                message: e.to_string(),
                line: 0,
                column: 0,
            })
            .and_then(|parsed| __sifr_toml_value_from_parsed(parsed))
    };
}

// --- stdlib: sifr.timeit ---
fn default_timer() -> f64 {
    return std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
}
fn _elapsed_non_negative(start: f64, end: f64) -> f64 {
    let elapsed: f64 = end - start;
    if elapsed < (0.0 as f64) {
        return 0.0 as f64;
    }
    return elapsed;
}
fn timeit(stmt: impl Fn(), number: i64) -> f64 {
    let start: f64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    let mut i: i64 = 0 as i64;
    while i < number {
        stmt();
        i = i + (1 as i64);
    }
    let end: f64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    return _elapsed_non_negative(start, end);
}
fn repeat(stmt: impl Fn(), count: i64, number: i64) -> Vec<f64> {
    let mut results: Vec<f64> = vec![];
    let mut r: i64 = 0 as i64;
    while r < count {
        let start: f64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        let mut i: i64 = 0 as i64;
        while i < number {
            stmt();
            i = i + (1 as i64);
        }
        let end: f64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        let elapsed: f64 = _elapsed_non_negative(start, end);
        results.push(elapsed);
        r = r + (1 as i64);
    }
    return results;
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

fn do_work() {
    let mut total: i64 = 0 as i64;
    let mut i: i64 = 0 as i64;
    while i < (1000 as i64) {
        total = total + i;
        i = i + (1 as i64);
    }
}

fn main() {
    println!("=== Monotonic Clocks ===");
    let t1: f64 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs_f64();
    let t2: f64 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs_f64();
    println!("{}", t2 >= t1);
    let m1: f64 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs_f64();
    let m2: f64 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs_f64();
    println!("{}", m2 >= m1);
    println!("=== timeit (Callable API) ===");
    let dt: f64 = default_timer();
    println!("{}", dt >= (0.0 as f64));
    let elapsed: f64 = timeit(do_work, 100 as i64);
    println!("{}", elapsed >= (0.0 as f64));
    let results: Vec<f64> = repeat(do_work, 3 as i64, 50 as i64);
    println!("{}", results.len() as i64);
    println!("=== glob ===");
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _r1: String = ({
    let __cmd = "mkdir -p /tmp/sifr_polish_demo".to_string();
    let __output = std::process::Command::new("sh".to_string()).arg("-c".to_string()).arg(&__cmd).output().map_err(__io_err)?;
    Ok(String::from_utf8_lossy(&__output.stdout).trim().to_string())
})?;
    let _w1: () = std::fs::write(&"/tmp/sifr_polish_demo/a.txt".to_string(), "aaa".to_string().as_bytes()).map(|_| ()).map_err(__io_err)?;
    let _w2: () = std::fs::write(&"/tmp/sifr_polish_demo/b.txt".to_string(), "bbb".to_string().as_bytes()).map(|_| ()).map_err(__io_err)?;
    let _w3: () = std::fs::write(&"/tmp/sifr_polish_demo/c.csv".to_string(), "1,2".to_string().as_bytes()).map(|_| ()).map_err(__io_err)?;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("setup error: {}", err.message);
    }
    let matches: Vec<String> = glob(&"/tmp/sifr_polish_demo".to_string(), &"*.txt".to_string());
    println!("{}", matches.len() as i64);
    println!("=== shutil ===");
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _cp: () = copy(&"/tmp/sifr_polish_demo/a.txt".to_string(), &"/tmp/sifr_polish_demo/a_copy.txt".to_string())?;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("copy error: {}", err.message);
    }
    println!("{}", std::path::Path::new(&"/tmp/sifr_polish_demo/a_copy.txt".to_string()).exists());
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _mv: () = move_file(&"/tmp/sifr_polish_demo/a_copy.txt".to_string(), &"/tmp/sifr_polish_demo/a_moved.txt".to_string())?;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("move error: {}", err.message);
    }
    println!("{}", std::path::Path::new(&"/tmp/sifr_polish_demo/a_moved.txt".to_string()).exists());
    println!("{}", std::path::Path::new(&"/tmp/sifr_polish_demo/a_copy.txt".to_string()).exists());
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _r2: String = ({
    let __cmd = "mkdir -p /tmp/sifr_polish_demo/sub".to_string();
    let __output = std::process::Command::new("sh".to_string()).arg("-c".to_string()).arg(&__cmd).output().map_err(__io_err)?;
    Ok(String::from_utf8_lossy(&__output.stdout).trim().to_string())
})?;
    let _w4: () = std::fs::write(&"/tmp/sifr_polish_demo/sub/nested.txt".to_string(), "nested".to_string().as_bytes()).map(|_| ()).map_err(__io_err)?;
    let _rm: () = rmtree(&"/tmp/sifr_polish_demo/sub".to_string())?;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("rmtree error: {}", err.message);
    }
    println!("{}", std::path::Path::new(&"/tmp/sifr_polish_demo/sub".to_string()).exists());
    println!("=== tomllib ===");
    let __sifr_try_res: Result<(), TOMLDecodeError> = (|| {
    let mut inline: TomlValue = loads(&"key = \"value\"".to_string())?;
    let key_value: Option<TomlValue> = inline.get(&"key".to_string());
    println!("{}", key_value != None);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("toml loads error: {}", err.message);
    }
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _cleanup: String = ({
    let __cmd = "rm -rf /tmp/sifr_polish_demo".to_string();
    let __output = std::process::Command::new("sh".to_string()).arg("-c".to_string()).arg(&__cmd).output().map_err(__io_err)?;
    Ok(String::from_utf8_lossy(&__output.stdout).trim().to_string())
})?;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("cleanup error: {}", err.message);
    }
    println!("=== Done ===");
}
