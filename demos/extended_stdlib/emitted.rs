// --- stdlib: sifr.bytes ---
fn decode_utf8(data: &Vec<u8>) -> Result<String, ParseError> {
    return String::from_utf8(data.iter().copied().collect::<Vec<u8>>())
        .map_err(|e| ParseError {
            message: e.to_string(),
        });
}
fn bytes_from_hex(s: &String) -> Result<Vec<u8>, ParseError> {
    return {
        let s: String = s.to_string();
        let mut cleaned = String::new();
        for ch in s.chars() {
            if ch.is_ascii_whitespace() {
                continue;
            }
            if !ch.is_ascii_hexdigit() {
                return Err(ParseError {
                    message: format!("invalid hex character: {}", ch),
                });
            }
            cleaned.push(ch);
        }
        if (cleaned.len() % 2) != 0 {
            return Err(ParseError {
                message: "fromhex() arg must contain an even number of hexadecimal digits"
                    .to_string()
                    .to_string(),
            });
        }
        let mut result = Vec::new();
        for pair in cleaned.as_bytes().chunks(2) {
            let pair_str = std::str::from_utf8(pair)
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            result
                .push(
                    u8::from_str_radix(pair_str, 16)
                        .map_err(|e| ParseError {
                            message: e.to_string(),
                        })?,
                );
        }
        Ok(result)
    };
}
fn bytes_from_ints(values: &Vec<i64>) -> Result<Vec<u8>, ValueError> {
    return {
        let __vals = values;
        let mut __out = Vec::new();
        for __pair in __vals.iter().enumerate() {
            if (*__pair.1 < 0) || (*__pair.1 > 255) {
                return Err(ValueError {
                    message: format!(
                        "byte out of range at index {}: {}", __pair.0, * __pair.1
                    ),
                });
            }
            __out.push(*__pair.1 as u8);
        }
        Ok(__out)
    };
}
fn bytes_with_size(size: i64) -> Result<Vec<u8>, ValueError> {
    return {
        let __size = size;
        if __size < 0 {
            return Err(ValueError {
                message: "bytes(size) requires a non-negative size"
                    .to_string()
                    .to_string(),
            });
        }
        Ok((0..__size).map(|_| 0 as u8).collect::<Vec<u8>>())
    };
}
fn encode_utf8_result(s: &String) -> Result<Vec<u8>, ParseError> {
    return Ok({
        let __s = s;
        __s.as_bytes().to_vec()
    });
}
fn count_byte(data: &Vec<u8>, value: i64) -> i64 {
    let mut count: i64 = 0 as i64;
    for b in data.iter().map(|__byte| *__byte as i64) {
        if b == value {
            count = count + (1 as i64);
        }
    }
    return count;
}
fn find_byte(data: &Vec<u8>, value: i64) -> Option<i64> {
    let mut idx: i64 = 0 as i64;
    for b in data.iter().map(|__byte| *__byte as i64) {
        if b == value {
            return Some(idx);
        }
        idx = idx + (1 as i64);
    }
    return None;
}
fn starts_with(data: &Vec<u8>, prefix: &Vec<u8>) -> bool {
    if (prefix.len() as i64) > (data.len() as i64) {
        return false;
    }
    let mut i: i64 = 0 as i64;
    while i < (prefix.len() as i64) {
        let a: Option<i64> = data.get(i as usize).map(|__byte| *__byte as i64);
        let b: Option<i64> = prefix.get(i as usize).map(|__byte| *__byte as i64);
        let Some(a) = a else {
            return false;
        };
        let Some(b) = b else {
            return false;
        };
        if a != b {
            return false;
        }
        i = i + (1 as i64);
    }
    return true;
}
fn ends_with(data: &Vec<u8>, suffix: &Vec<u8>) -> bool {
    if (suffix.len() as i64) > (data.len() as i64) {
        return false;
    }
    let offset: i64 = (data.len() as i64) - (suffix.len() as i64);
    let mut i: i64 = 0 as i64;
    while i < (suffix.len() as i64) {
        let a: Option<i64> = data
            .get((offset + i) as usize)
            .map(|__byte| *__byte as i64);
        let b: Option<i64> = suffix.get(i as usize).map(|__byte| *__byte as i64);
        let Some(a) = a else {
            return false;
        };
        let Some(b) = b else {
            return false;
        };
        if a != b {
            return false;
        }
        i = i + (1 as i64);
    }
    return true;
}

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
fn log_base(x: f64, base: f64) -> f64 {
    return (x).ln() / (base).ln();
}
fn isclose(a: f64, b: f64, rel_tol: f64, abs_tol: f64) -> bool {
    if rel_tol < (0.0 as f64) {
        return false;
    }
    if abs_tol < (0.0 as f64) {
        return false;
    }
    if a == b {
        return true;
    }
    if (((a).is_nan()) || ((b).is_nan())) {
        return false;
    }
    if (((a).is_infinite()) || ((b).is_infinite())) {
        return false;
    }
    let mut diff: f64 = a - b;
    if diff < (0.0 as f64) {
        diff = (0.0 as f64) - diff;
    }
    let mut a_abs: f64 = a;
    if a_abs < (0.0 as f64) {
        a_abs = (0.0 as f64) - a_abs;
    }
    let mut b_abs: f64 = b;
    if b_abs < (0.0 as f64) {
        b_abs = (0.0 as f64) - b_abs;
    }
    let mut rel_bound: f64 = rel_tol * (a_abs).max(b_abs);
    if abs_tol > rel_bound {
        rel_bound = abs_tol;
    }
    return diff <= rel_bound;
}
fn prod(data: &Vec<i64>) -> i64 {
    let mut result: i64 = 1 as i64;
    for val in data.iter().copied() {
        result = result * val;
    }
    return result;
}
fn frexp_mantissa(x: f64) -> f64 {
    let parts: Vec<f64> = {
        let __x: f64 = x as f64;
        if __x == 0.0 {
            vec![__x, 0.0]
        } else {
            if !__x.is_finite() {
                vec![__x, 0.0]
            } else {
                {
                    let __bits: u64 = __x.to_bits();
                    let __sign_mask: u64 = (1 as u64) << 63;
                    let __frac_mask: u64 = ((1 as u64) << 52) - (1 as u64);
                    let __sign: u64 = __bits & __sign_mask;
                    let __exp: i32 = ((__bits >> 52) & (2047 as u64)) as i32;
                    let __frac: u64 = __bits & __frac_mask;
                    if __exp == 0 {
                        {
                            let __scaled: f64 = __x * (2.0 as f64).powi(54);
                            let __sbits: u64 = __scaled.to_bits();
                            let __sexp: i32 = ((__sbits >> 52) & (2047 as u64)) as i32;
                            let __sfrac: u64 = __sbits & __frac_mask;
                            let __mant: f64 = f64::from_bits(
                                (__sign | ((1022 as u64) << 52)) | __sfrac,
                            );
                            let __e: i32 = (__sexp - 1022) - 54;
                            vec![__mant, __e as f64]
                        }
                    } else {
                        {
                            let __mant: f64 = f64::from_bits(
                                (__sign | ((1022 as u64) << 52)) | __frac,
                            );
                            let __e: i32 = __exp - 1022;
                            vec![__mant, __e as f64]
                        }
                    }
                }
            }
        }
    };
    let m: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 0 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(m) = m else {
        return f64::NAN;
    };
    return m;
}
fn frexp_exponent(x: f64) -> i64 {
    let parts: Vec<f64> = {
        let __x: f64 = x as f64;
        if __x == 0.0 {
            vec![__x, 0.0]
        } else {
            if !__x.is_finite() {
                vec![__x, 0.0]
            } else {
                {
                    let __bits: u64 = __x.to_bits();
                    let __sign_mask: u64 = (1 as u64) << 63;
                    let __frac_mask: u64 = ((1 as u64) << 52) - (1 as u64);
                    let __sign: u64 = __bits & __sign_mask;
                    let __exp: i32 = ((__bits >> 52) & (2047 as u64)) as i32;
                    let __frac: u64 = __bits & __frac_mask;
                    if __exp == 0 {
                        {
                            let __scaled: f64 = __x * (2.0 as f64).powi(54);
                            let __sbits: u64 = __scaled.to_bits();
                            let __sexp: i32 = ((__sbits >> 52) & (2047 as u64)) as i32;
                            let __sfrac: u64 = __sbits & __frac_mask;
                            let __mant: f64 = f64::from_bits(
                                (__sign | ((1022 as u64) << 52)) | __sfrac,
                            );
                            let __e: i32 = (__sexp - 1022) - 54;
                            vec![__mant, __e as f64]
                        }
                    } else {
                        {
                            let __mant: f64 = f64::from_bits(
                                (__sign | ((1022 as u64) << 52)) | __frac,
                            );
                            let __e: i32 = __exp - 1022;
                            vec![__mant, __e as f64]
                        }
                    }
                }
            }
        }
    };
    let exp_val: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 1 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(exp_val) = exp_val else {
        return 0 as i64;
    };
    return (exp_val).trunc() as i64;
}
fn modf_fractional(x: f64) -> f64 {
    let parts: Vec<f64> = {
        let __x: f64 = x as f64;
        if __x.is_nan() {
            vec![f64::NAN, f64::NAN]
        } else {
            if __x.is_infinite() {
                vec![(0.0 as f64).copysign(__x), __x]
            } else {
                {
                    let __int = __x.trunc();
                    let mut __frac = __x - __int;
                    if __frac == 0.0 {
                        __frac = (0.0 as f64).copysign(__x);
                    }
                    vec![__frac, __int]
                }
            }
        }
    };
    let f: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 0 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(f) = f else {
        return f64::NAN;
    };
    return f;
}
fn modf_integral(x: f64) -> f64 {
    let parts: Vec<f64> = {
        let __x: f64 = x as f64;
        if __x.is_nan() {
            vec![f64::NAN, f64::NAN]
        } else {
            if __x.is_infinite() {
                vec![(0.0 as f64).copysign(__x), __x]
            } else {
                {
                    let __int = __x.trunc();
                    let mut __frac = __x - __int;
                    if __frac == 0.0 {
                        __frac = (0.0 as f64).copysign(__x);
                    }
                    vec![__frac, __int]
                }
            }
        }
    };
    let i: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 1 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(i) = i else {
        return f64::NAN;
    };
    return i;
}
fn pow(x: f64, y: f64) -> f64 {
    return (x).powf(y);
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

fn main() {
    println!("=== sifr.time ===");
    let t1: f64 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs_f64();
    println!("Current epoch: {}", t1);
    {
    let __secs = 0.01 as f64;
    if __secs.is_finite() && (__secs > 0.0) { std::thread::sleep(std::time::Duration::from_nanos((__secs * 1000000000.0) as u64)) } else { () }
};
    let t2: f64 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs_f64();
    println!("Time advanced: {}", t2 > t1);
    println!("=== sifr.random ===");
    let r: i64 = {
    let __start = 1 as i64;
    let __end = 100 as i64;
    __start + rand::Rng::gen_range(&mut rand::thread_rng(), 0..(__end - __start) + 1)
};
    println!("Random int [1,100]: {}", r);
    let f: f64 = rand::random::<f64>();
    println!("Random float [0,1): {}", f);
    println!("=== sifr.re ===");
    let __sifr_try_res: Result<(), RegexError> = (|| {
    let matched: bool = regex::Regex::new(&"[0-9]+".to_string()).map(|re| re.is_match(&"hello 42".to_string())).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })?;
    println!("Match digits in \'hello 42\': {}", matched);
    let found: Option<String> = regex::Regex::new(&"[0-9]+".to_string()).map(|re| re.find(&"price is $42.99".to_string()).map(|m| m.as_str().to_string())).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })?;
    if let Some(found) = found {
        println!("Found: {}", found);
    }
    let replaced: String = regex::Regex::new(&"[0-9]+".to_string()).map(|re| re.replace_all(&"a1b2c3".to_string(), &*"N".to_string()).to_string()).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })?;
    println!("Replace: {}", replaced);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("regex error: {}", err.message);
    }
    println!("=== sifr.hashlib ===");
    println!("SHA-256(\'sifr\'): {}", format!("{:x}", (<sha2::Sha256 as sha2::Digest>::digest)(("sifr".to_string()).as_bytes())));
    println!("MD5(\'sifr\'): {}", format!("{:x}", md5::compute(("sifr".to_string()).as_bytes())));
    println!("=== sifr.base64 ===");
    let encoded: String = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &"Hello, Sifr!".to_string().as_bytes());
    println!("Base64 encode: {}", encoded);
    let __sifr_try_res: Result<(), ParseError> = (|| {
    let decoded_b64: String = ({
    let __bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &encoded.as_bytes()).map_err(|e| ParseError { message: e.to_string() })?;
    String::from_utf8(__bytes).map_err(|e| ParseError { message: e.to_string() })
})?;
    println!("Base64 decode: {}", decoded_b64);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("base64 error: {}", err.message);
    }
    println!("=== Demo complete ===");
}
