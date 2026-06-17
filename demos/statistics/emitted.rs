use std::collections::HashMap;

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
fn _sum(data: &Vec<f64>) -> f64 {
    let mut total: f64 = 0.0 as f64;
    for val in data.iter().copied() {
        total = total + val;
    }
    return total;
}
fn mean(data: &Vec<f64>) -> Result<f64, StatisticsError> {
    let count: i64 = data.len() as i64;
    if count == (0 as i64) {
        return Err(
            StatisticsError::new("mean requires at least one data point".to_string()),
        );
    }
    let total: f64 = _sum(data);
    return Ok(total / (count as f64));
}
fn median(data: &Vec<f64>) -> Result<f64, StatisticsError> {
    let n: i64 = data.len() as i64;
    if n == (0 as i64) {
        return Err(
            StatisticsError::new("median requires at least one data point".to_string()),
        );
    }
    let sorted_data: Vec<f64> = {
        let mut __sifr_sorted_v = (data).iter().copied().collect::<Vec<_>>();
        __sifr_sorted_v.sort_by(f64::total_cmp);
        __sifr_sorted_v
    };
    let mid: i64 = n / (2 as i64);
    if (n % (2 as i64)) == (0 as i64) {
        let a: Option<f64> = {
            let __sifr_index_list = &sorted_data;
            let __sifr_index_i = mid - (1 as i64);
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        let b: Option<f64> = {
            let __sifr_index_list = &sorted_data;
            let __sifr_index_i = mid;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        if let Some(a) = a {
            if let Some(b) = b {
                return Ok((a + b) / (2.0 as f64));
            }
        }
        return Err(StatisticsError::new("median: index error".to_string()));
    } else {
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
        return Err(StatisticsError::new("median: index error".to_string()));
    }
}
fn variance(data: &Vec<f64>) -> Result<f64, StatisticsError> {
    let n: i64 = data.len() as i64;
    if n < (2 as i64) {
        return Err(
            StatisticsError::new(
                "variance requires at least two data points".to_string(),
            ),
        );
    }
    let avg: f64 = _sum(data) / (n as f64);
    let mut total: f64 = 0.0 as f64;
    for val in data.iter().copied() {
        let diff: f64 = val - avg;
        total = total + (diff * diff);
    }
    return Ok(total / ((n - (1 as i64)) as f64));
}
fn stdev(data: &Vec<f64>) -> Result<f64, StatisticsError> {
    let n: i64 = data.len() as i64;
    if n < (2 as i64) {
        return Err(
            StatisticsError::new("stdev requires at least two data points".to_string()),
        );
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
fn geometric_mean(data: &Vec<f64>) -> Result<f64, StatisticsError> {
    let n: i64 = data.len() as i64;
    if n == (0 as i64) {
        return Err(
            StatisticsError::new(
                "geometric_mean requires at least one data point".to_string(),
            ),
        );
    }
    let mut log_sum: f64 = 0.0 as f64;
    for val in data.iter().copied() {
        if val <= (0.0 as f64) {
            return Err(
                StatisticsError::new(
                    "geometric_mean requires positive values".to_string(),
                ),
            );
        }
        log_sum = log_sum + (val).ln();
    }
    return Ok((log_sum / (n as f64)).exp());
}
fn mode(data: &Vec<i64>) -> Result<i64, StatisticsError> {
    if (data.len() as i64) == (0 as i64) {
        return Err(
            StatisticsError::new("mode requires at least one data point".to_string()),
        );
    }
    let mut counts: HashMap<i64, i64> = HashMap::from([]);
    for val in data.iter().copied() {
        let existing: Option<i64> = counts.get(&val).copied();
        if let Some(existing) = existing {
            counts.insert(val, existing + (1 as i64));
        } else {
            counts.insert(val, 1 as i64);
        }
    }
    let mut best: i64 = 0 as i64;
    let mut best_set: bool = false;
    let mut best_count: i64 = 0 as i64;
    for val2 in data.iter().copied() {
        let count2: Option<i64> = counts.get(&val2).copied();
        let mut count2_val: i64 = 0 as i64;
        if let Some(count2) = count2 {
            count2_val = count2;
        }
        if count2_val > best_count {
            best_count = count2_val;
            best = val2;
            best_set = true;
        }
    }
    if best_set {
        return Ok(best);
    }
    return Err(StatisticsError::new("mode: no mode found".to_string()));
}
fn multimode(data: &Vec<i64>) -> Result<Vec<i64>, StatisticsError> {
    if (data.len() as i64) == (0 as i64) {
        return Err(
            StatisticsError::new(
                "multimode requires at least one data point".to_string(),
            ),
        );
    }
    let mut counts: HashMap<i64, i64> = HashMap::from([]);
    for val in data.iter().copied() {
        let existing: Option<i64> = counts.get(&val).copied();
        if let Some(existing) = existing {
            counts.insert(val, existing + (1 as i64));
        } else {
            counts.insert(val, 1 as i64);
        }
    }
    let mut max_count: i64 = 0 as i64;
    for val2 in data.iter().copied() {
        let count2: Option<i64> = counts.get(&val2).copied();
        let mut count2_val: i64 = 0 as i64;
        if let Some(count2) = count2 {
            count2_val = count2;
        }
        if count2_val > max_count {
            max_count = count2_val;
        }
    }
    let mut result: Vec<i64> = vec![];
    let mut seen: HashMap<i64, bool> = HashMap::from([]);
    for val3 in data.iter().copied() {
        let already_opt: Option<bool> = seen.get(&val3).copied();
        let mut already: bool = false;
        if let Some(already_opt) = already_opt {
            already = already_opt;
        }
        if !already {
            let count3: Option<i64> = counts.get(&val3).copied();
            let mut count3_val: i64 = 0 as i64;
            if let Some(count3) = count3 {
                count3_val = count3;
            }
            if count3_val == max_count {
                result.push(val3);
            }
            seen.insert(val3, true);
        }
    }
    return Ok(result);
}
fn quantiles(data: &Vec<f64>, n: i64) -> Result<Vec<f64>, StatisticsError> {
    if (data.len() as i64) < (2 as i64) {
        return Err(
            StatisticsError::new(
                "quantiles requires at least two data points".to_string(),
            ),
        );
    }
    if n < (1 as i64) {
        return Err(StatisticsError::new("quantiles: n must be at least 1".to_string()));
    }
    let sorted_data: Vec<f64> = {
        let mut __sifr_sorted_v = (data).iter().copied().collect::<Vec<_>>();
        __sifr_sorted_v.sort_by(f64::total_cmp);
        __sifr_sorted_v
    };
    let m: i64 = sorted_data.len() as i64;
    let mut result: Vec<f64> = vec![];
    let mut i: i64 = 1 as i64;
    while i < n {
        let idx_f: f64 = ((i as f64) * (m as f64)) / (n as f64);
        let mut idx: i64 = idx_f as i64;
        let frac: f64 = idx_f - (idx as f64);
        if idx >= m {
            idx = m - (1 as i64);
        }
        if idx < (0 as i64) {
            idx = 0 as i64;
        }
        let lo: Option<f64> = {
            let __sifr_index_list = &sorted_data;
            let __sifr_index_i = idx;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        let mut lo_val: f64 = 0.0 as f64;
        if let Some(lo) = lo {
            lo_val = lo;
        }
        if frac > (0.0 as f64) {
            let hi_idx: i64 = idx + (1 as i64);
            if hi_idx < m {
                let hi: Option<f64> = {
                    let __sifr_index_list = &sorted_data;
                    let __sifr_index_i = hi_idx;
                    let __sifr_index_norm = if __sifr_index_i < 0 {
                        ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                    } else {
                        __sifr_index_i as usize
                    };
                    __sifr_index_list.get(__sifr_index_norm).copied()
                };
                if let Some(hi) = hi {
                    lo_val = lo_val + (frac * (hi - lo_val));
                }
            }
        }
        result.push(lo_val);
        i = i + (1 as i64);
    }
    return Ok(result);
}
fn covariance(x: &Vec<f64>, y: &Vec<f64>) -> Result<f64, StatisticsError> {
    let n: i64 = x.len() as i64;
    if n < (2 as i64) {
        return Err(
            StatisticsError::new(
                "covariance requires at least two data points".to_string(),
            ),
        );
    }
    if (y.len() as i64) != n {
        return Err(
            StatisticsError::new(
                "covariance: x and y must have the same length".to_string(),
            ),
        );
    }
    let mx: f64 = _sum(x) / (n as f64);
    let my: f64 = _sum(y) / (n as f64);
    let mut total: f64 = 0.0 as f64;
    let mut i: i64 = 0 as i64;
    while i < n {
        let xi: Option<f64> = {
            let __sifr_index_list = &x;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        let yi: Option<f64> = {
            let __sifr_index_list = &y;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        if let Some(xi) = xi {
            if let Some(yi) = yi {
                total = total + ((xi - mx) * (yi - my));
            }
        }
        i = i + (1 as i64);
    }
    return Ok(total / ((n - (1 as i64)) as f64));
}
fn correlation(x: &Vec<f64>, y: &Vec<f64>) -> Result<f64, StatisticsError> {
    let n: i64 = x.len() as i64;
    if n < (2 as i64) {
        return Err(
            StatisticsError::new(
                "correlation requires at least two data points".to_string(),
            ),
        );
    }
    if (y.len() as i64) != n {
        return Err(
            StatisticsError::new(
                "correlation: x and y must have the same length".to_string(),
            ),
        );
    }
    let mx: f64 = _sum(x) / (n as f64);
    let my: f64 = _sum(y) / (n as f64);
    let mut cov_num: f64 = 0.0 as f64;
    let mut sx_num: f64 = 0.0 as f64;
    let mut sy_num: f64 = 0.0 as f64;
    let mut i: i64 = 0 as i64;
    while i < n {
        let xi: Option<f64> = {
            let __sifr_index_list = &x;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        let yi: Option<f64> = {
            let __sifr_index_list = &y;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        if let Some(xi) = xi {
            if let Some(yi) = yi {
                cov_num = cov_num + ((xi - mx) * (yi - my));
                sx_num = sx_num + ((xi - mx) * (xi - mx));
                sy_num = sy_num + ((yi - my) * (yi - my));
            }
        }
        i = i + (1 as i64);
    }
    let sx: f64 = (sx_num / ((n - (1 as i64)) as f64)).sqrt();
    let sy: f64 = (sy_num / ((n - (1 as i64)) as f64)).sqrt();
    if sx == (0.0 as f64) {
        return Err(StatisticsError::new("correlation: x has zero variance".to_string()));
    }
    if sy == (0.0 as f64) {
        return Err(StatisticsError::new("correlation: y has zero variance".to_string()));
    }
    return Ok((cov_num / ((n - (1 as i64)) as f64)) / (sx * sy));
}
fn linear_regression(x: &Vec<f64>, y: &Vec<f64>) -> Result<Vec<f64>, StatisticsError> {
    let n: i64 = x.len() as i64;
    if n < (2 as i64) {
        return Err(
            StatisticsError::new(
                "linear_regression requires at least two data points".to_string(),
            ),
        );
    }
    if (y.len() as i64) != n {
        return Err(
            StatisticsError::new(
                "linear_regression: x and y must have the same length".to_string(),
            ),
        );
    }
    let mx: f64 = _sum(x) / (n as f64);
    let my: f64 = _sum(y) / (n as f64);
    let mut num: f64 = 0.0 as f64;
    let mut den: f64 = 0.0 as f64;
    let mut i: i64 = 0 as i64;
    while i < n {
        let xi: Option<f64> = {
            let __sifr_index_list = &x;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        let yi: Option<f64> = {
            let __sifr_index_list = &y;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        if let Some(xi) = xi {
            if let Some(yi) = yi {
                num = num + ((xi - mx) * (yi - my));
                den = den + ((xi - mx) * (xi - mx));
            }
        }
        i = i + (1 as i64);
    }
    if den == (0.0 as f64) {
        return Err(
            StatisticsError::new("linear_regression: x has zero variance".to_string()),
        );
    }
    let slope: f64 = num / den;
    let intercept: f64 = my - (slope * mx);
    let mut result: Vec<f64> = vec![];
    result.push(slope);
    result.push(intercept);
    return Ok(result);
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

fn near(v: f64, target: f64, tol: f64) -> bool {
    if v < (target - tol) {
        return false;
    }
    if v > (target + tol) {
        return false;
    }
    return true;
}

fn collect_positive_actual() -> Vec<String> {
    let mut actual: Vec<String> = vec![];
    let data: Vec<f64> = vec![1.0 as f64, 2.0 as f64, 3.0 as f64, 4.0 as f64, 5.0 as f64];
    let mut mean_ok: bool = true;
    let mut mean_v: f64 = 0.0 as f64;
    let __sifr_try_res: Result<(), StatisticsError> = (|| {
    let out_mean: f64 = mean(&data)?;
    mean_v = out_mean;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        mean_ok = false;
    }
    actual.push(format!("{}", mean_ok && near(mean_v, 3.0 as f64, 0.0001 as f64)));
    let mut median_ok: bool = true;
    let mut median_v: f64 = 0.0 as f64;
    let __sifr_try_res: Result<(), StatisticsError> = (|| {
    let out_median: f64 = median(&data)?;
    median_v = out_median;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        median_ok = false;
    }
    actual.push(format!("{}", median_ok && near(median_v, 3.0 as f64, 0.0001 as f64)));
    let mut variance_ok: bool = true;
    let mut variance_v: f64 = 0.0 as f64;
    let __sifr_try_res: Result<(), StatisticsError> = (|| {
    let out_variance: f64 = variance(&data)?;
    variance_v = out_variance;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        variance_ok = false;
    }
    actual.push(format!("{}", variance_ok && near(variance_v, 2.5 as f64, 0.0001 as f64)));
    let mut stdev_ok: bool = true;
    let mut stdev_v: f64 = 0.0 as f64;
    let __sifr_try_res: Result<(), StatisticsError> = (|| {
    let out_stdev: f64 = stdev(&data)?;
    stdev_v = out_stdev;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        stdev_ok = false;
    }
    actual.push(format!("{}", stdev_ok && near(stdev_v, 1.5811 as f64, 0.001 as f64)));
    let mut mode_ok: bool = true;
    let mut mode_v: i64 = 0 as i64;
    let __sifr_try_res: Result<(), StatisticsError> = (|| {
    let out_mode: i64 = mode(&vec![1 as i64, 2 as i64, 2 as i64, 3 as i64, 3 as i64, 3 as i64])?;
    mode_v = out_mode;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        mode_ok = false;
    }
    actual.push(format!("{}", mode_ok && (mode_v == (3 as i64))));
    let mut mm_ok: bool = true;
    let mut mm_v: Vec<i64> = vec![];
    let __sifr_try_res: Result<(), StatisticsError> = (|| {
    let out_mm: Vec<i64> = multimode(&vec![1 as i64, 2 as i64, 2 as i64, 3 as i64, 3 as i64])?;
    mm_v = out_mm;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        mm_ok = false;
    }
    actual.push(format!("{}", mm_ok && ((mm_v.len() as i64) == (2 as i64))));
    let mut q_ok: bool = true;
    let mut q_v: Vec<f64> = vec![];
    let __sifr_try_res: Result<(), StatisticsError> = (|| {
    let out_q: Vec<f64> = quantiles(&vec![1.0 as f64, 2.0 as f64, 3.0 as f64, 4.0 as f64, 5.0 as f64, 6.0 as f64, 7.0 as f64, 8.0 as f64], 4 as i64)?;
    q_v = out_q;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        q_ok = false;
    }
    actual.push(format!("{}", q_ok && ((q_v.len() as i64) == (3 as i64))));
    let x: Vec<f64> = vec![1.0 as f64, 2.0 as f64, 3.0 as f64, 4.0 as f64, 5.0 as f64];
    let y: Vec<f64> = vec![2.0 as f64, 4.0 as f64, 6.0 as f64, 8.0 as f64, 10.0 as f64];
    let mut cov_ok: bool = true;
    let mut cov_v: f64 = 0.0 as f64;
    let __sifr_try_res: Result<(), StatisticsError> = (|| {
    let out_cov: f64 = covariance(&x, &y)?;
    cov_v = out_cov;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        cov_ok = false;
    }
    actual.push(format!("{}", cov_ok && near(cov_v, 5.0 as f64, 0.0001 as f64)));
    let mut corr_ok: bool = true;
    let mut corr_v: f64 = 0.0 as f64;
    let __sifr_try_res: Result<(), StatisticsError> = (|| {
    let out_corr: f64 = correlation(&x, &y)?;
    corr_v = out_corr;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        corr_ok = false;
    }
    actual.push(format!("{}", corr_ok && near(corr_v, 1.0 as f64, 0.0001 as f64)));
    let mut lr_ok: bool = true;
    let mut lr_v: Vec<f64> = vec![];
    let __sifr_try_res: Result<(), StatisticsError> = (|| {
    let out_lr: Vec<f64> = linear_regression(&x, &y)?;
    lr_v = out_lr;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        lr_ok = false;
    }
    let mut lr_slope_ok: bool = false;
    let mut lr_intercept_ok: bool = false;
    if lr_ok {
        let lr_slope: Option<f64> = {
    let __sifr_index_list = &lr_v;
    let __sifr_index_i = 0 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
};
        let lr_intercept: Option<f64> = {
    let __sifr_index_list = &lr_v;
    let __sifr_index_i = 1 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
};
        if let Some(lr_slope) = lr_slope {
            lr_slope_ok = near(lr_slope, 2.0 as f64, 0.0001 as f64);
        }
        if let Some(lr_intercept) = lr_intercept {
            lr_intercept_ok = near(lr_intercept, 0.0 as f64, 0.0001 as f64);
        }
    }
    actual.push(format!("{}", ((lr_ok && ((lr_v.len() as i64) == (2 as i64))) && lr_slope_ok) && lr_intercept_ok));
    let mut hmean_ok: bool = true;
    let mut hmean_v: f64 = 0.0 as f64;
    let __sifr_try_res: Result<(), StatisticsError> = (|| {
    let out_hmean: f64 = harmonic_mean(&vec![2.0 as f64, 4.0 as f64, 4.0 as f64, 8.0 as f64])?;
    hmean_v = out_hmean;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        hmean_ok = false;
    }
    actual.push(format!("{}", hmean_ok && near(hmean_v, 3.5555555556 as f64, 0.0001 as f64)));
    let mut gmean_ok: bool = true;
    let mut gmean_v: f64 = 0.0 as f64;
    let __sifr_try_res: Result<(), StatisticsError> = (|| {
    let out_gmean: f64 = geometric_mean(&vec![4.0 as f64, 9.0 as f64])?;
    gmean_v = out_gmean;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        gmean_ok = false;
    }
    actual.push(format!("{}", gmean_ok && near(gmean_v, 6.0 as f64, 0.0001 as f64)));
    return actual;
}

fn collect_error_actual_ok() -> Vec<bool> {
    let mut actual_ok: Vec<bool> = vec![];
    let __sifr_try_res: Result<(), StatisticsError> = (|| {
    let bad_mean: f64 = mean(&vec![])?;
    let _: String = format!("{}", bad_mean);
    actual_ok.push(true);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        actual_ok.push(false);
    }
    let __sifr_try_res: Result<(), StatisticsError> = (|| {
    let bad_hmean: f64 = harmonic_mean(&vec![0.0 as f64, 1.0 as f64])?;
    let _: String = format!("{}", bad_hmean);
    actual_ok.push(true);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        actual_ok.push(false);
    }
    return actual_ok;
}

fn main() {
    let expected: Vec<String> = vec!["true".to_string(), "true".to_string(), "true".to_string(), "true".to_string(), "true".to_string(), "true".to_string(), "true".to_string(), "true".to_string(), "true".to_string(), "true".to_string(), "true".to_string(), "true".to_string()];
    let actual: Vec<String> = collect_positive_actual();
    assert_vector_eq(&actual, &expected);
    let expected_ok: Vec<bool> = vec![false, false];
    let actual_ok: Vec<bool> = collect_error_actual_ok();
    assert_bool_vector_eq(&actual_ok, &expected_ok);
    println!("statistics parity demo: pass");
}
