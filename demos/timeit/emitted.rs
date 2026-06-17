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

fn workload() {
    let mut total: i64 = 0 as i64;
    let mut i: i64 = 0 as i64;
    while i < (100 as i64) {
        total = total + i;
        i = i + (1 as i64);
    }
}

fn all_non_negative(values: &Vec<f64>) -> bool {
    let mut i: i64 = 0 as i64;
    while i < (values.len() as i64) {
        let current: Option<f64> = Some(values[i as usize]);
        let Some(current) = current else {
            return false;
        };
        if current < (0.0 as f64) {
            return false;
        }
        i = i + (1 as i64);
    }
    return true;
}

fn collect_timer_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let t1: f64 = default_timer();
    {
    let __secs = 0.01 as f64;
    if __secs.is_finite() && (__secs > 0.0) { std::thread::sleep(std::time::Duration::from_nanos((__secs * 1000000000.0) as u64)) } else { () }
};
    let t2: f64 = default_timer();
    actual.push(t2 >= t1);
    return actual;
}

fn collect_repeat_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let elapsed: f64 = timeit(workload, 10 as i64);
    actual.push(elapsed >= (0.0 as f64));
    let repeated: Vec<f64> = repeat(workload, 3 as i64, 10 as i64);
    actual.push((repeated.len() as i64) == (3 as i64));
    actual.push(all_non_negative(&repeated));
    return actual;
}

fn collect_edge_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    actual.push((repeat(workload, 0 as i64, 5 as i64).len() as i64) == (0 as i64));
    actual.push(timeit(workload, 0 as i64) >= (0.0 as f64));
    actual.push((repeat(workload, 2 as i64, 0 as i64).len() as i64) == (2 as i64));
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
    append_all(&mut actual, &collect_timer_actual());
    append_all(&mut actual, &collect_repeat_actual());
    append_all(&mut actual, &collect_edge_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("timeit timeit parity demo: pass");
}
