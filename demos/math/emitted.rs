// --- stdlib: sifr.math ---
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

// --- stdlib: sifr.test ---
fn assert_eq<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    actual: &T,
    expected: &T,
) {
    assert!(* actual == * expected);
}
fn assert_vector_eq(actual: &Vec<String>, expected: &Vec<String>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0 as i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize].clone()) == expected.get(i as usize).cloned());
        i = i + (1 as i64);
    }
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

fn collect_positive_actual() -> Vec<String> {
    let mut actual: Vec<String> = vec![];
    actual.push(format!("{}", (27.0 as f64).cbrt() == (3.0 as f64)));
    actual.push(format!("{}", (3.0 as f64).exp2() == (8.0 as f64)));
    actual.push(format!("{}", (((2.0 as f64) * (3.0 as f64)) + (4.0 as f64)) == (10.0 as f64)));
    actual.push(format!("{}", log_base(32.0 as f64, 2.0 as f64) == (5.0 as f64)));
    actual.push(format!("{}", isclose(0.000000001 as f64, 0.0 as f64, 0.9 as f64, 0.00000001 as f64)));
    let p: Vec<f64> = vec![1.0 as f64, 2.0 as f64, 3.0 as f64];
    let q: Vec<f64> = vec![4.0 as f64, 5.0 as f64, 6.0 as f64];
    actual.push(format!("{}", ({
    let __p = &p;
    let __q = &q;
    let __len = __p.len().min(__q.len());
    let mut __sum: f64 = 0.0;
    for __i in 0..__len {
        __sum += __p[__i] * __q[__i];
    }
    __sum
}) == (32.0 as f64)));
    let tiny_subnormal: f64 = (0.000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000022250738585072014 as f64) / (2.0 as f64);
    actual.push(format!("{}", (1.0 as f64).is_normal()));
    actual.push(format!("{}", (tiny_subnormal).is_finite() && !(tiny_subnormal).is_normal()));
    actual.push(format!("{}", ({
    let __x: f64 = (5.5 as f64) as f64;
    let __y: f64 = (2.0 as f64) as f64;
    if __x.is_nan() || __y.is_nan() { f64::NAN } else { if (__y == 0.0) || __x.is_infinite() { f64::NAN } else { if __y.is_infinite() { __x } else { {
    let __q: f64 = __x / __y;
    let __n0: f64 = __q.trunc();
    let __frac: f64 = __q - __n0;
    let __abs_frac: f64 = __frac.abs();
    let __n: f64 = if __abs_frac < 0.5 { __n0 } else { if __abs_frac > 0.5 { __n0 + __q.signum() } else { if ((__n0 as i64) % 2) == 0 { __n0 } else { __n0 + __q.signum() } } };
    let __r: f64 = __x - (__n * __y);
    if __r == 0.0 { (0.0 as f64).copysign(__x) } else { __r }
} } } }
}) < (0.0 as f64)));
    actual.push(format!("{}", ({
    let __p = &vec![1.0 as f64, 2.0 as f64];
    let __q = &vec![1.0 as f64];
    if __p.len() != __q.len() { f64::NAN } else { if __p.is_empty() { 0.0 } else { {
    let mut __scale: f64 = 0.0;
    let mut __ssq: f64 = 1.0;
    for __i in 0..__p.len() {
        let __d: f64 = (__p[__i] - __q[__i]).abs();
        if __d != 0.0 {
            if __scale < __d {
                let __r: f64 = __scale / __d;
                __ssq = 1.0 + ((__ssq * __r) * __r);
                __scale = __d;
            } else {
                let __r: f64 = __d / __scale;
                __ssq += __r * __r;
            }
        }
    }
    if __scale == 0.0 { 0.0 } else { __scale * __ssq.sqrt() }
} } }
}).is_nan()));
    actual.push(format!("{}", ({
    let __data = &vec![10000000000000000159028911097599180468360808563945281389781327557747838772170381060813469985856815104.0 as f64, 1.0 as f64, -(10000000000000000159028911097599180468360808563945281389781327557747838772170381060813469985856815104.0 as f64)];
    let mut __sum: f64 = 0.0;
    let mut __comp: f64 = 0.0;
    let mut __pos_inf: bool = false;
    let mut __neg_inf: bool = false;
    let mut __has_nan: bool = false;
    for __x in __data.iter() {
        let __v: f64 = *__x;
        if __v.is_nan() {
            __has_nan = true;
            continue;
        }
        if __v.is_infinite() {
            if __v.is_sign_positive() {
                __pos_inf = true;
            } else {
                __neg_inf = true;
            }
            continue;
        }
        let __t: f64 = __sum + __v;
        if __sum.abs() >= __v.abs() {
            __comp += (__sum - __t) + __v;
        } else {
            __comp += (__v - __t) + __sum;
        }
        __sum = __t;
    }
    if __has_nan || (__pos_inf && __neg_inf) { f64::NAN } else { if __pos_inf { f64::INFINITY } else { if __neg_inf { f64::NEG_INFINITY } else { __sum + __comp } } }
}) == (1.0 as f64)));
    actual.push(format!("{}", ({
    let __x: f64 = (1.0 as f64) as f64;
    let __y: f64 = f64::INFINITY as f64;
    if __x.is_nan() || __y.is_nan() { f64::NAN } else { if __x == __y { __y } else { if __x == 0.0 { {
    let __sign: u64 = if __y.is_sign_negative() { (1 as u64) << 63 } else { 0 as u64 };
    f64::from_bits(__sign | (1 as u64))
} } else { {
    let mut __bits: u64 = __x.to_bits();
    if (__x < __y) == (__x > 0.0) {
        __bits += 1 as u64;
    } else {
        __bits -= 1 as u64;
    }
    f64::from_bits(__bits)
} } } }
}) > (1.0 as f64)));
    actual.push(format!("{}", ({
    let __x: f64 = (1.0 as f64) as f64;
    if __x.is_nan() { f64::NAN } else { if __x.is_infinite() { f64::INFINITY } else { {
    let __a = __x.abs();
    if __a == 0.0 { f64::from_bits(1 as u64) } else { if __a == f64::MAX { __a - f64::from_bits(__a.to_bits() - (1 as u64)) } else { f64::from_bits(__a.to_bits() + (1 as u64)) - __a } }
} } }
}) > (0.0 as f64)));
    return actual;
}

fn collect_negative_actual_false() -> Vec<bool> {
    let actual_false: Vec<bool> = vec![isclose(1.0 as f64, 1.0 as f64, -(0.1 as f64), 0.0 as f64), isclose(1.0 as f64, 1.0 as f64, 0.1 as f64, -(0.1 as f64))];
    return actual_false;
}

fn main() {
    let expected: Vec<String> = vec!["true".to_string(), "true".to_string(), "true".to_string(), "true".to_string(), "true".to_string(), "true".to_string(), "true".to_string(), "true".to_string(), "true".to_string(), "true".to_string(), "true".to_string(), "true".to_string(), "true".to_string()];
    let actual: Vec<String> = collect_positive_actual();
    assert_vector_eq(&actual, &expected);
    let expected_false: Vec<bool> = vec![false, false];
    let actual_false: Vec<bool> = collect_negative_actual_false();
    assert_bool_vector_eq(&actual_false, &expected_false);
    println!("math math parity demo: pass");
}
