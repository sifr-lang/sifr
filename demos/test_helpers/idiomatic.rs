// --- stdlib: sifr.test ---
fn assert_almost_eq(actual: f64, expected: f64, tolerance: f64) {
    assert!(tolerance >= (0.0 as f64));
    if actual == expected {
        return;
    }
    let mut diff: f64 = actual - expected;
    if diff < (0.0 as f64) {
        diff = (0.0 as f64) - diff;
    }
    if diff != diff {
        assert!(false);
    }
    assert!(diff <= tolerance);
}
fn assert_gt<T: Clone + std::fmt::Display + PartialOrd + 'static>(a: &T, b: &T) {
    assert!(*a > *b);
}
fn assert_lt<T: Clone + std::fmt::Display + PartialOrd + 'static>(a: &T, b: &T) {
    assert!(*a < *b);
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
fn prod(data: &[i64]) -> i64 {
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
                            let __mant: f64 =
                                f64::from_bits((__sign | ((1022 as u64) << 52)) | __sfrac);
                            let __e: i32 = (__sexp - 1022) - 54;
                            vec![__mant, __e as f64]
                        }
                    } else {
                        {
                            let __mant: f64 =
                                f64::from_bits((__sign | ((1022 as u64) << 52)) | __frac);
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
                            let __mant: f64 =
                                f64::from_bits((__sign | ((1022 as u64) << 52)) | __sfrac);
                            let __e: i32 = (__sexp - 1022) - 54;
                            vec![__mant, __e as f64]
                        }
                    } else {
                        {
                            let __mant: f64 =
                                f64::from_bits((__sign | ((1022 as u64) << 52)) | __frac);
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
fn _sum(data: &[f64]) -> f64 {
    let mut total: f64 = 0.0 as f64;
    for val in data.iter().copied() {
        total = total + val;
    }
    return total;
}
fn mean(data: &[f64]) -> Result<f64, StatisticsError> {
    let count: i64 = data.len() as i64;
    if count == (0 as i64) {
        return Err(StatisticsError::new(
            "mean requires at least one data point".to_string(),
        ));
    }
    let total: f64 = _sum(data);
    return Ok(total / (count as f64));
}
fn variance(data: &[f64]) -> Result<f64, StatisticsError> {
    let n: i64 = data.len() as i64;
    if n < (2 as i64) {
        return Err(StatisticsError::new(
            "variance requires at least two data points".to_string(),
        ));
    }
    let avg: f64 = _sum(data) / (n as f64);
    let mut total: f64 = 0.0 as f64;
    for val in data.iter().copied() {
        let diff: f64 = val - avg;
        total = total + (diff * diff);
    }
    return Ok(total / ((n - (1 as i64)) as f64));
}
fn pvariance(data: &[f64]) -> Result<f64, StatisticsError> {
    let n: i64 = data.len() as i64;
    if n == (0 as i64) {
        return Err(StatisticsError::new(
            "pvariance requires at least one data point".to_string(),
        ));
    }
    let avg: f64 = _sum(data) / (n as f64);
    let mut total: f64 = 0.0 as f64;
    for val in data.iter().copied() {
        let diff: f64 = val - avg;
        total = total + (diff * diff);
    }
    return Ok(total / (n as f64));
}
fn stdev(data: &[f64]) -> Result<f64, StatisticsError> {
    let n: i64 = data.len() as i64;
    if n < (2 as i64) {
        return Err(StatisticsError::new(
            "stdev requires at least two data points".to_string(),
        ));
    }
    let avg: f64 = _sum(data) / (n as f64);
    let mut total: f64 = 0.0 as f64;
    for val in data.iter().copied() {
        let diff: f64 = val - avg;
        total = total + (diff * diff);
    }
    let v: f64 = total / ((n - (1 as i64)) as f64);
    return Ok((v).sqrt());
}
fn pstdev(data: &[f64]) -> Result<f64, StatisticsError> {
    let n: i64 = data.len() as i64;
    if n == (0 as i64) {
        return Err(StatisticsError::new(
            "pstdev requires at least one data point".to_string(),
        ));
    }
    let avg: f64 = _sum(data) / (n as f64);
    let mut total: f64 = 0.0 as f64;
    for val in data.iter().copied() {
        let diff: f64 = val - avg;
        total = total + (diff * diff);
    }
    let v: f64 = total / (n as f64);
    return Ok((v).sqrt());
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

impl std::error::Error for Error {}

fn main() {
    let data: Vec<f64> = vec![
        2.0 as f64, 4.0 as f64, 4.0 as f64, 4.0 as f64, 5.0 as f64, 5.0 as f64, 7.0 as f64,
        9.0 as f64,
    ];
    let __sifr_try_res: Result<(), StatisticsError> = (|| {
        let m: f64 = mean(&data)?;
        let sv: f64 = variance(&data)?;
        let pv: f64 = pvariance(&data)?;
        let sd: f64 = stdev(&data)?;
        let pd: f64 = pstdev(&data)?;
        println!("mean = {m}");
        assert_eq!(format!("mean = {m}"), "mean = 5");
        println!("sample variance = {sv}");
        assert_eq!(
            format!("sample variance = {sv}"),
            "sample variance = 4.571428571428571"
        );
        println!("population variance = {pv}");
        assert_eq!(
            format!("population variance = {pv}"),
            "population variance = 4"
        );
        println!("sample stdev = {sd}");
        assert_eq!(
            format!("sample stdev = {sd}"),
            "sample stdev = 2.138089935299395"
        );
        println!("population stdev = {pd}");
        assert_eq!(format!("population stdev = {pd}"), "population stdev = 2");
        {
            let __lhs = m;
            let __rhs = 5.0 as f64;
            let __tol = 0.001 as f64;
            assert!(
                (__lhs == __rhs) || ((__lhs - __rhs).abs() <= __tol),
                "assert_almost_eq failed: {} != {} (tolerance {})",
                __lhs,
                __rhs,
                __tol
            )
        };
        {
            let __lhs = sv;
            let __rhs = 4.571 as f64;
            let __tol = 0.01 as f64;
            assert!(
                (__lhs == __rhs) || ((__lhs - __rhs).abs() <= __tol),
                "assert_almost_eq failed: {} != {} (tolerance {})",
                __lhs,
                __rhs,
                __tol
            )
        };
        {
            let __lhs = pv;
            let __rhs = 4.0 as f64;
            let __tol = 0.001 as f64;
            assert!(
                (__lhs == __rhs) || ((__lhs - __rhs).abs() <= __tol),
                "assert_almost_eq failed: {} != {} (tolerance {})",
                __lhs,
                __rhs,
                __tol
            )
        };
        {
            let __lhs = sd;
            let __rhs = 2.138 as f64;
            let __tol = 0.01 as f64;
            assert!(
                (__lhs == __rhs) || ((__lhs - __rhs).abs() <= __tol),
                "assert_almost_eq failed: {} != {} (tolerance {})",
                __lhs,
                __rhs,
                __tol
            )
        };
        {
            let __lhs = pd;
            let __rhs = 2.0 as f64;
            let __tol = 0.001 as f64;
            assert!(
                (__lhs == __rhs) || ((__lhs - __rhs).abs() <= __tol),
                "assert_almost_eq failed: {} != {} (tolerance {})",
                __lhs,
                __rhs,
                __tol
            )
        };
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("statistics error: {}", e.message);
        assert_eq!(
            format!("statistics error: {}", e.message),
            "All assertions passed!"
        );
    }
    assert!(
        (10 as i64) > (5 as i64),
        "assert_gt failed: {} is not > {}",
        10 as i64,
        5 as i64
    );
    assert!(
        (3 as i64) < (7 as i64),
        "assert_lt failed: {} is not < {}",
        3 as i64,
        7 as i64
    );
    assert!(
        (100 as i64) > (0 as i64),
        "assert_gt failed: {} is not > {}",
        100 as i64,
        0 as i64
    );
    assert!(
        (0 as i64) < (1 as i64),
        "assert_lt failed: {} is not < {}",
        0 as i64,
        1 as i64
    );
    println!("All assertions passed!");
}
