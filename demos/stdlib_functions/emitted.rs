// --- stdlib: sifr.math ---
fn factorial(n: i64) -> i64 {
    if n < (0 as i64) {
        return 0 as i64;
    }
    let mut result: i64 = 1 as i64;
    let mut i: i64 = 2 as i64;
    while i <= n {
        result = result * i;
        i = i + (1 as i64);
    }
    return result;
}
fn gcd(a: i64, b: i64) -> i64 {
    let mut x: i64 = a;
    let mut y: i64 = b;
    if x < (0 as i64) {
        x = (0 as i64) - x;
    }
    if y < (0 as i64) {
        y = (0 as i64) - y;
    }
    while y != (0 as i64) {
        let temp: i64 = y;
        y = x % y;
        x = temp;
    }
    return x;
}
fn lcm(a: i64, b: i64) -> i64 {
    if a == (0 as i64) {
        return 0 as i64;
    }
    if b == (0 as i64) {
        return 0 as i64;
    }
    let g: i64 = gcd(a, b);
    let mut x: i64 = a;
    if x < (0 as i64) {
        x = (0 as i64) - x;
    }
    let mut y: i64 = b;
    if y < (0 as i64) {
        y = (0 as i64) - y;
    }
    return (x / g) * y;
}
fn comb(n: i64, k: i64) -> i64 {
    if k < (0 as i64) {
        return 0 as i64;
    }
    if k > n {
        return 0 as i64;
    }
    if k == (0 as i64) {
        return 1 as i64;
    }
    if k == n {
        return 1 as i64;
    }
    let mut r: i64 = k;
    if r > (n - k) {
        r = n - k;
    }
    let mut result: i64 = 1 as i64;
    let mut i: i64 = 0 as i64;
    while i < r {
        result = result * (n - i);
        result = result / (i + (1 as i64));
        i = i + (1 as i64);
    }
    return result;
}
fn perm(n: i64, k: i64) -> i64 {
    if k < (0 as i64) {
        return 0 as i64;
    }
    if k > n {
        return 0 as i64;
    }
    let mut result: i64 = 1 as i64;
    let mut i: i64 = 0 as i64;
    while i < k {
        result = result * (n - i);
        i = i + (1 as i64);
    }
    return result;
}
fn prod(data: &Vec<i64>) -> i64 {
    let mut result: i64 = 1 as i64;
    for val in data.iter().copied() {
        result = result * val;
    }
    return result;
}

// --- stdlib: sifr.statistics ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct StatisticsError {
    message: String,
}
impl StatisticsError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}
impl std::fmt::Display for StatisticsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.message);
    }
}
impl std::error::Error for StatisticsError {}
fn harmonic_mean(data: &Vec<f64>) -> Result<f64, StatisticsError> {
    let n: i64 = data.len() as i64;
    if n == (0 as i64) {
        return Err(
            StatisticsError::new(
                "harmonic_mean requires at least one data point".to_string(),
            ),
        );
    }
    let mut total: f64 = 0.0 as f64;
    for val in data.iter().copied() {
        if val <= (0.0 as f64) {
            return Err(
                StatisticsError::new(
                    "harmonic_mean requires positive values".to_string(),
                ),
            );
        }
        total = total + ((1.0 as f64) / val);
    }
    return Ok((n as f64) / total);
}
fn median_low(data: &Vec<f64>) -> Result<f64, StatisticsError> {
    let n: i64 = data.len() as i64;
    if n == (0 as i64) {
        return Err(
            StatisticsError::new(
                "median_low requires at least one data point".to_string(),
            ),
        );
    }
    let sorted_data: Vec<f64> = {
        let mut __sifr_sorted_v = (data).iter().copied().collect::<Vec<_>>();
        __sifr_sorted_v.sort_by(f64::total_cmp);
        __sifr_sorted_v
    };
    let mid: i64 = n / (2 as i64);
    if (n % (2 as i64)) == (0 as i64) {
        let val: Option<f64> = {
            let __sifr_index_list = &sorted_data;
            let __sifr_index_i = mid - (1 as i64);
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        if let Some(val) = val {
            return Ok(val);
        }
        return Err(StatisticsError::new("median_low: index error".to_string()));
    } else {
        let val2: Option<f64> = {
            let __sifr_index_list = &sorted_data;
            let __sifr_index_i = mid;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        if let Some(val2) = val2 {
            return Ok(val2);
        }
        return Err(StatisticsError::new("median_low: index error".to_string()));
    }
}
fn median_high(data: &Vec<f64>) -> Result<f64, StatisticsError> {
    let n: i64 = data.len() as i64;
    if n == (0 as i64) {
        return Err(
            StatisticsError::new(
                "median_high requires at least one data point".to_string(),
            ),
        );
    }
    let sorted_data: Vec<f64> = {
        let mut __sifr_sorted_v = (data).iter().copied().collect::<Vec<_>>();
        __sifr_sorted_v.sort_by(f64::total_cmp);
        __sifr_sorted_v
    };
    let mid: i64 = n / (2 as i64);
    let val: Option<f64> = {
        let __sifr_index_list = &sorted_data;
        let __sifr_index_i = mid;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    if let Some(val) = val {
        return Ok(val);
    }
    return Err(StatisticsError::new("median_high: index error".to_string()));
}

// --- stdlib: sifr.string ---
fn capwords(s: &String) -> String {
    let normalized: String = s
        .replace(&"\t".to_string(), &" ".to_string())
        .replace(&"\n".to_string(), &" ".to_string())
        .replace(&"\r".to_string(), &" ".to_string())
        .replace(&"\u{b}".to_string(), &" ".to_string())
        .replace(&"\u{c}".to_string(), &" ".to_string());
    let words: Vec<String> = normalized
        .split(&" ".to_string())
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let mut result: String = "".to_string();
    let mut first: bool = true;
    for word in words.iter().cloned() {
        if (word.chars().count() as i64) > (0 as i64) {
            if !first {
                result = format!("{}{}", result, " ".to_string());
            }
            first = false;
            let cap: String = {
                let _s = word.clone();
                let mut _c = _s.chars();
                _c.next()
                    .map(|f| f.to_uppercase().to_string() + &_c.as_str().to_lowercase())
                    .unwrap_or_default()
            };
            result = format!("{}{}", result, cap);
        }
    }
    return result;
}

// --- stdlib: sifr.pathlib ---
fn basename(path: &String) -> String {
    let mut i: i64 = (path.chars().count() as i64) - (1 as i64);
    while i >= (0 as i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = path.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(ch) = ch {
            if ch == "/".to_string() {
                return String::from_iter(
                    (path).chars().skip((i + (1 as i64)).max(0) as usize),
                );
            }
        }
        i = i - (1 as i64);
    }
    return format!("{}{}", path, "".to_string());
}
fn stem(path: &String) -> String {
    let base: String = basename(path);
    let mut i: i64 = (base.chars().count() as i64) - (1 as i64);
    while i > (0 as i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = base.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(ch) = ch {
            if ch == ".".to_string() {
                return String::from_iter(
                    (base)
                        .chars()
                        .skip(0 as usize)
                        .take(((i).max(0) - 0).max(0) as usize),
                );
            }
        }
        i = i - (1 as i64);
    }
    return format!("{}{}", base, "".to_string());
}
fn is_absolute(path: &String) -> bool {
    if (path.len() as i64) == (0 as i64) {
        return false;
    }
    if (path.chars().count() as i64) >= (3 as i64) {
        let colon: Option<String> = {
            let __sifr_index_str = &path;
            let __sifr_index_i = 1 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        let sep: Option<String> = {
            let __sifr_index_str = &path;
            let __sifr_index_i = 2 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        if let Some(colon) = colon {
            if let Some(sep) = sep {
                if (colon == ":".to_string())
                    && ((sep == "/".to_string()) || (sep == "\\".to_string()))
                {
                    return true;
                }
            }
        }
    }
    let first: Option<String> = Some({
        let Some(__indexed_char) = path.chars().nth((0 as i64) as usize) else {
            unreachable!("compiler-verified string index should be in range");
        };
        __indexed_char.to_string()
    });
    if let Some(first) = first {
        if (first == "/".to_string()) || (first == "\\".to_string()) {
            return true;
        }
    }
    return false;
}

// --- stdlib: sifr.bisect ---
fn bisect_left<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    a: &Vec<T>,
    x: &T,
    lo: i64,
    hi: Option<i64>,
) -> i64 {
    let mut left: i64 = lo;
    if left < (0 as i64) {
        left = 0 as i64;
    }
    let mut right: i64 = a.len() as i64;
    if hi.is_none() {
        right = a.len() as i64;
    } else {
        if let Some(hi) = hi {
            if hi < (0 as i64) {
                right = 0 as i64;
            } else {
                if hi > (a.len() as i64) {
                    right = a.len() as i64;
                } else {
                    right = hi;
                }
            }
        }
    }
    while left < right {
        let mid: i64 = (left + right) / (2 as i64);
        let val: Option<T> = {
            let __sifr_index_list = &a;
            let __sifr_index_i = mid;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        if let Some(val) = val {
            if val < *x {
                left = mid + (1 as i64);
            } else {
                right = mid;
            }
        } else {
            left = mid + (1 as i64);
        }
    }
    return left;
}

// --- stdlib: sifr.itertools ---
fn pairwise<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
) -> Vec<Vec<T>> {
    let mut result: Vec<Vec<T>> = vec![];
    let mut prev_values: Vec<T> = vec![];
    for value in data.iter().cloned() {
        if (prev_values.len() as i64) > (0 as i64) {
            let mut pair: Vec<T> = vec![];
            let prev: Option<T> = Some(prev_values[(0 as i64) as usize].clone());
            if let Some(prev) = prev {
                pair.push(prev.clone());
            }
            pair.push(value.clone());
            result.push(pair);
            {
                let __idx_raw = 0 as i64;
                let __idx_norm = if __idx_raw < 0 {
                    (prev_values.len() as i64) + __idx_raw
                } else {
                    __idx_raw
                };
                if __idx_norm >= 0 {
                    if let Some(__elem) = prev_values.get_mut(__idx_norm as usize) {
                        *__elem = value;
                    }
                }
            }
        } else {
            prev_values.push(value.clone());
        }
    }
    return result;
}
fn batched<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    n: i64,
) -> Result<Vec<Vec<T>>, ValueError> {
    if n <= (0 as i64) {
        return Err(ValueError::new("batched: n must be > 0".to_string()));
    }
    let mut result: Vec<Vec<T>> = vec![];
    let mut current_batch: Vec<T> = vec![];
    for value in data.iter().cloned() {
        current_batch.push(value.clone());
        if (current_batch.len() as i64) == n {
            result.push(current_batch);
            current_batch = vec![];
        }
    }
    if (current_batch.len() as i64) > (0 as i64) {
        result.push(current_batch);
    }
    return Ok(result);
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
struct Error {
    message: String,
}

impl Error {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for Error {
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

fn main() {
    println!("=== Math Functions ===");
    println!("{}", format!("{}{}", "factorial(10) = ".to_string(), format!("{}", factorial(10 as i64))));
    println!("{}", format!("{}{}", "gcd(48, 18) = ".to_string(), format!("{}", gcd(48 as i64, 18 as i64))));
    println!("{}", format!("{}{}", "lcm(4, 6) = ".to_string(), format!("{}", lcm(4 as i64, 6 as i64))));
    println!("{}", format!("{}{}", "comb(10, 3) = ".to_string(), format!("{}", comb(10 as i64, 3 as i64))));
    println!("{}", format!("{}{}", "perm(5, 3) = ".to_string(), format!("{}", perm(5 as i64, 3 as i64))));
    let nums: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64, 4 as i64, 5 as i64];
    println!("{}", format!("{}{}", "prod([1,2,3,4,5]) = ".to_string(), format!("{}", prod(&nums))));
    println!("{}", format!("{}{}", "exp(1.0) = ".to_string(), format!("{}", (1.0 as f64).exp())));
    println!("{}", format!("{}{}", "isfinite(1.0) = ".to_string(), format!("{}", (1.0 as f64).is_finite())));
    println!("=== Statistics Functions ===");
    let data: Vec<f64> = vec![1.0 as f64, 2.0 as f64, 3.0 as f64, 4.0 as f64, 5.0 as f64];
    let even: Vec<f64> = vec![1.0 as f64, 2.0 as f64, 3.0 as f64, 4.0 as f64];
    let __sifr_try_res: Result<(), StatisticsError> = (|| {
    let hm: f64 = harmonic_mean(&data)?;
    let ml: f64 = median_low(&even)?;
    let mh: f64 = median_high(&even)?;
    println!("{}", format!("{}{}", "harmonic_mean = ".to_string(), format!("{}", hm)));
    println!("{}", format!("{}{}", "median_low = ".to_string(), format!("{}", ml)));
    println!("{}", format!("{}{}", "median_high = ".to_string(), format!("{}", mh)));
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let se = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "statistics error: ".to_string(), se.message));
    }
    println!("=== String Functions ===");
    println!("{}", format!("{}{}", "capwords = ".to_string(), capwords(&"hello world test".to_string())));
    println!("=== Path Functions ===");
    println!("{}", format!("{}{}", "stem = ".to_string(), stem(&"/docs/report.pdf".to_string())));
    println!("{}", format!("{}{}", "is_absolute = ".to_string(), format!("{}", is_absolute(&"/usr/bin".to_string()))));
    println!("=== Generic Bisect ===");
    let floats: Vec<f64> = vec![1.0 as f64, 2.0 as f64, 3.0 as f64, 4.0 as f64, 5.0 as f64];
    println!("{}", format!("{}{}", "bisect_left(floats, 2.5) = ".to_string(), format!("{}", bisect_left(&floats, &(2.5 as f64), 0 as i64, None))));
    println!("=== Itertools ===");
    let items: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64, 4 as i64];
    println!("{}", format!("{}{}", "pairwise = ".to_string(), format!("{:?}", pairwise(&(items).iter().copied().collect::<Vec<_>>()))));
    let items2: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64, 4 as i64, 5 as i64, 6 as i64, 7 as i64, 8 as i64];
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let bat: Vec<Vec<i64>> = batched(&(items2).iter().copied().collect::<Vec<_>>(), 3 as i64)?;
    println!("{}", format!("{}{}", "batched = ".to_string(), format!("{:?}", bat)));
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("error: {}", e.message);
    }
    println!("Done!");
}
