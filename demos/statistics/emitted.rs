// src/main.rs
use ::std::collections::HashMap;

// --- stdlib: _sifr.math ---
const PI: f64 = 3.141592653589793_f64;
const E: f64 = 2.718281828459045_f64;
const TAU: f64 = 6.283185307179586_f64;
const INF: f64 = f64::INFINITY;
const NAN: f64 = f64::NAN;
fn sqrt(x: f64) -> f64 {
    ::sifr_stdlib::math::sqrt(x)
}
fn floor(x: f64) -> i64 {
    ::sifr_stdlib::math::floor(x).to_i64_saturating()
}
fn ceil(x: f64) -> i64 {
    ::sifr_stdlib::math::ceil(x).to_i64_saturating()
}
fn log(x: f64) -> f64 {
    ::sifr_stdlib::math::log(x)
}
fn cbrt(x: f64) -> f64 {
    ::sifr_stdlib::math::cbrt(x)
}
fn sin(x: f64) -> f64 {
    ::sifr_stdlib::math::sin(x)
}
fn cos(x: f64) -> f64 {
    ::sifr_stdlib::math::cos(x)
}
fn tan(x: f64) -> f64 {
    ::sifr_stdlib::math::tan(x)
}
fn pow_val(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::pow_val(x, y)
}
fn min_val(a: f64, b: f64) -> f64 {
    ::sifr_stdlib::math::min_val(a, b)
}
fn max_val(a: f64, b: f64) -> f64 {
    ::sifr_stdlib::math::max_val(a, b)
}
fn round_val(x: f64) -> i64 {
    ::sifr_stdlib::math::round_val(x).to_i64_saturating()
}
fn asin(x: f64) -> f64 {
    ::sifr_stdlib::math::asin(x)
}
fn acos(x: f64) -> f64 {
    ::sifr_stdlib::math::acos(x)
}
fn atan(x: f64) -> f64 {
    ::sifr_stdlib::math::atan(x)
}
fn atan2(y: f64, x: f64) -> f64 {
    ::sifr_stdlib::math::atan2(y, x)
}
fn sinh(x: f64) -> f64 {
    ::sifr_stdlib::math::sinh(x)
}
fn cosh(x: f64) -> f64 {
    ::sifr_stdlib::math::cosh(x)
}
fn tanh(x: f64) -> f64 {
    ::sifr_stdlib::math::tanh(x)
}
fn log10(x: f64) -> f64 {
    ::sifr_stdlib::math::log10(x)
}
fn log2(x: f64) -> f64 {
    ::sifr_stdlib::math::log2(x)
}
fn exp2(x: f64) -> f64 {
    ::sifr_stdlib::math::exp2(x)
}
fn degrees(x: f64) -> f64 {
    ::sifr_stdlib::math::degrees(x)
}
fn radians(x: f64) -> f64 {
    ::sifr_stdlib::math::radians(x)
}
fn isnan(x: f64) -> bool {
    ::sifr_stdlib::math::isnan(x)
}
fn isinf(x: f64) -> bool {
    ::sifr_stdlib::math::isinf(x)
}
fn trunc(x: f64) -> i64 {
    ::sifr_stdlib::math::trunc(x).to_i64_saturating()
}
fn copysign(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::copysign(x, y)
}
fn signbit(x: f64) -> bool {
    ::sifr_stdlib::math::signbit(x)
}
fn fmod(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::fmod(x, y)
}
fn remainder(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::remainder(x, y)
}
fn hypot(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::hypot(x, y)
}
fn fma(x: f64, y: f64, z: f64) -> f64 {
    ::sifr_stdlib::math::fma(x, y, z)
}
fn fmax(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::fmax(x, y)
}
fn fmin(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::fmin(x, y)
}
fn exp(x: f64) -> f64 {
    ::sifr_stdlib::math::exp(x)
}
fn expm1(x: f64) -> f64 {
    ::sifr_stdlib::math::expm1(x)
}
fn log1p(x: f64) -> f64 {
    ::sifr_stdlib::math::log1p(x)
}
fn fabs(x: f64) -> f64 {
    ::sifr_stdlib::math::fabs(x)
}
fn isfinite(x: f64) -> bool {
    ::sifr_stdlib::math::isfinite(x)
}
fn isnormal(x: f64) -> bool {
    ::sifr_stdlib::math::isnormal(x)
}
fn issubnormal(x: f64) -> bool {
    ::sifr_stdlib::math::issubnormal(x)
}
fn acosh(x: f64) -> f64 {
    ::sifr_stdlib::math::acosh(x)
}
fn asinh(x: f64) -> f64 {
    ::sifr_stdlib::math::asinh(x)
}
fn atanh(x: f64) -> f64 {
    ::sifr_stdlib::math::atanh(x)
}
fn isqrt(n: i64) -> i64 {
    ::sifr_stdlib::math::isqrt(::sifr_runtime::interop::SifrIntBridge::from(n))
        .to_i64_saturating()
}
fn dist_impl(p: Vec<f64>, q: Vec<f64>) -> f64 {
    ::sifr_stdlib::math::dist(p, q)
}
fn fsum_impl(data: Vec<f64>) -> f64 {
    ::sifr_stdlib::math::fsum(data)
}
fn sumprod_impl(p: Vec<f64>, q: Vec<f64>) -> f64 {
    ::sifr_stdlib::math::sumprod(p, q)
}
fn erf(x: f64) -> f64 {
    ::sifr_stdlib::math::erf(x)
}
fn erfc(x: f64) -> f64 {
    ::sifr_stdlib::math::erfc(x)
}
fn gamma(x: f64) -> f64 {
    ::sifr_stdlib::math::gamma(x)
}
fn lgamma(x: f64) -> f64 {
    ::sifr_stdlib::math::lgamma(x)
}
fn frexp(x: f64) -> Vec<f64> {
    ::sifr_stdlib::math::frexp(x)
}
fn ldexp(m: f64, e: i64) -> f64 {
    ::sifr_stdlib::math::ldexp(m, ::sifr_runtime::interop::SifrIntBridge::from(e))
}
fn modf(x: f64) -> Vec<f64> {
    ::sifr_stdlib::math::modf(x)
}
fn nextafter(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::nextafter(x, y)
}
fn ulp(x: f64) -> f64 {
    ::sifr_stdlib::math::ulp(x)
}

// --- stdlib: sifr.math ---
fn factorial(n: i64) -> i64 {
    if n < (0_i64) {
        return 0_i64;
    }
    let mut result: i64 = 1_i64;
    let mut i: i64 = 2_i64;
    while i <= n {
        result *= i;
        i += 1_i64;
    }
    result
}
fn gcd(a: i64, b: i64) -> i64 {
    let mut x: i64 = a;
    let mut y: i64 = b;
    if x < (0_i64) {
        x = (0_i64) - x;
    }
    if y < (0_i64) {
        y = (0_i64) - y;
    }
    while y != (0_i64) {
        let temp: i64 = y;
        y = x % y;
        x = temp;
    }
    x
}
fn lcm(a: i64, b: i64) -> i64 {
    if a == (0_i64) {
        return 0_i64;
    }
    if b == (0_i64) {
        return 0_i64;
    }
    let g: i64 = gcd(a, b);
    let mut x: i64 = a;
    if x < (0_i64) {
        x = (0_i64) - x;
    }
    let mut y: i64 = b;
    if y < (0_i64) {
        y = (0_i64) - y;
    }
    (x / g) * y
}
fn comb(n: i64, k: i64) -> i64 {
    if k < (0_i64) {
        return 0_i64;
    }
    if k > n {
        return 0_i64;
    }
    if k == (0_i64) {
        return 1_i64;
    }
    if k == n {
        return 1_i64;
    }
    let mut r: i64 = k;
    if r > (n - k) {
        r = n - k;
    }
    let mut result: i64 = 1_i64;
    let mut i: i64 = 0_i64;
    while i < r {
        result *= n - i;
        result /= i + (1_i64);
        i += 1_i64;
    }
    result
}
fn perm(n: i64, k: i64) -> i64 {
    if k < (0_i64) {
        return 0_i64;
    }
    if k > n {
        return 0_i64;
    }
    let mut result: i64 = 1_i64;
    let mut i: i64 = 0_i64;
    while i < k {
        result *= n - i;
        i += 1_i64;
    }
    result
}
fn log_base(x: f64, base: f64) -> f64 {
    log(x) / log(base)
}
fn isclose(a: f64, b: f64, rel_tol: f64, abs_tol: f64) -> bool {
    if rel_tol < (0.0_f64) {
        return false;
    }
    if abs_tol < (0.0_f64) {
        return false;
    }
    if a == b {
        return true;
    }
    if isnan(a) || isnan(b) {
        return false;
    }
    if isinf(a) || isinf(b) {
        return false;
    }
    let mut diff: f64 = a - b;
    if diff < (0.0_f64) {
        diff = (0.0_f64) - diff;
    }
    let mut a_abs: f64 = a;
    if a_abs < (0.0_f64) {
        a_abs = (0.0_f64) - a_abs;
    }
    let mut b_abs: f64 = b;
    if b_abs < (0.0_f64) {
        b_abs = (0.0_f64) - b_abs;
    }
    let mut larger_abs: f64 = a_abs;
    if b_abs > larger_abs {
        larger_abs = b_abs;
    }
    let mut rel_bound: f64 = rel_tol * larger_abs;
    if abs_tol > rel_bound {
        rel_bound = abs_tol;
    }
    diff <= rel_bound
}
fn prod(data: &Vec<i64>) -> i64 {
    let mut result: i64 = 1_i64;
    for val in data.iter().copied() {
        result *= val;
    }
    result
}
fn _copy_float_list(data: &Vec<f64>) -> Vec<f64> {
    let mut out: Vec<f64> = vec![];
    for value in data.iter().copied() {
        out.push(value);
    }
    out
}
fn dist(p: &Vec<f64>, q: &Vec<f64>) -> f64 {
    dist_impl(_copy_float_list(p), _copy_float_list(q))
}
fn fsum(data: &Vec<f64>) -> f64 {
    fsum_impl(_copy_float_list(data))
}
fn sumprod(p: &Vec<f64>, q: &Vec<f64>) -> f64 {
    sumprod_impl(_copy_float_list(p), _copy_float_list(q))
}
fn frexp_mantissa(x: f64) -> f64 {
    let parts: Vec<f64> = frexp(x);
    let m: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 0_i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(m) = m else {
        return NAN;
    };
    m
}
fn frexp_exponent(x: f64) -> i64 {
    let parts: Vec<f64> = frexp(x);
    let exp_val: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 1_i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(exp_val) = exp_val else {
        return 0_i64;
    };
    trunc(exp_val)
}
fn modf_fractional(x: f64) -> f64 {
    let parts: Vec<f64> = modf(x);
    let f: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 0_i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(f) = f else {
        return NAN;
    };
    f
}
fn modf_integral(x: f64) -> f64 {
    let parts: Vec<f64> = modf(x);
    let i: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 1_i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(i) = i else {
        return NAN;
    };
    i
}
fn pow(x: f64, y: f64) -> f64 {
    pow_val(x, y)
}

// --- stdlib: sifr.statistics ---
#[derive(Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {
    message: String,
}
impl __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {}
impl ::std::fmt::Debug for __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct("StatisticsError").field("message", &self.message).finish()
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl ::std::error::Error for __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {}
fn _sum(data: &Vec<f64>) -> f64 {
    let mut total: f64 = 0.0_f64;
    for val in data.iter().copied() {
        total += val;
    }
    total
}
fn mean(
    data: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let count: i64 = data.len() as i64;
    if count == (0_i64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "mean requires at least one data point".to_string(),
            ),
        );
    }
    let total: f64 = _sum(data);
    Ok(total / (count as f64))
}
fn median(
    data: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: i64 = data.len() as i64;
    if n == (0_i64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "median requires at least one data point".to_string(),
            ),
        );
    }
    let sorted_data: Vec<f64> = {
        let mut __sifr_sorted_v = (data).iter().copied().collect::<Vec<_>>();
        __sifr_sorted_v.sort_by(f64::total_cmp);
        __sifr_sorted_v
    };
    let mid: i64 = n / (2_i64);
    if (n % (2_i64)) == (0_i64) {
        let a: Option<f64> = {
            let __sifr_index_list = &sorted_data;
            let __sifr_index_i = mid - (1_i64);
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
                return Ok((a + b) / (2.0_f64));
            }
        }
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "median: index error".to_string(),
            ),
        );
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
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "median: index error".to_string(),
            ),
        );
    }
}
fn variance(
    data: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: i64 = data.len() as i64;
    if n < (2_i64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "variance requires at least two data points".to_string(),
            ),
        );
    }
    let avg: f64 = _sum(data) / (n as f64);
    let mut total: f64 = 0.0_f64;
    for val in data.iter().copied() {
        let diff: f64 = val - avg;
        total += diff * diff;
    }
    Ok(total / ((n - (1_i64)) as f64))
}
fn stdev(
    data: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: i64 = data.len() as i64;
    if n < (2_i64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "stdev requires at least two data points".to_string(),
            ),
        );
    }
    let avg: f64 = _sum(data) / (n as f64);
    let mut total: f64 = 0.0_f64;
    for val in data.iter().copied() {
        let diff: f64 = val - avg;
        total += diff * diff;
    }
    let v: f64 = total / ((n - (1_i64)) as f64);
    Ok(sqrt(v))
}
fn harmonic_mean(
    data: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: i64 = data.len() as i64;
    if n == (0_i64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "harmonic_mean requires at least one data point".to_string(),
            ),
        );
    }
    let mut total: f64 = 0.0_f64;
    for val in data.iter().copied() {
        if val <= (0.0_f64) {
            return Err(
                __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                    "harmonic_mean requires positive values".to_string(),
                ),
            );
        }
        total += (1.0_f64) / val;
    }
    Ok((n as f64) / total)
}
fn geometric_mean(
    data: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: i64 = data.len() as i64;
    if n == (0_i64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "geometric_mean requires at least one data point".to_string(),
            ),
        );
    }
    let mut log_sum: f64 = 0.0_f64;
    for val in data.iter().copied() {
        if val <= (0.0_f64) {
            return Err(
                __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                    "geometric_mean requires positive values".to_string(),
                ),
            );
        }
        log_sum += log(val);
    }
    Ok(exp(log_sum / (n as f64)))
}
fn mode(
    data: &Vec<i64>,
) -> Result<i64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    if ((data.len() as i64) == (0_i64)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "mode requires at least one data point".to_string(),
            ),
        );
    }
    let mut counts: HashMap<i64, i64> = HashMap::from([]);
    for val in data.iter().copied() {
        let existing: Option<i64> = counts.get(&val).copied();
        if let Some(existing) = existing {
            counts.insert(val, existing + (1_i64));
        } else {
            counts.insert(val, 1_i64);
        }
    }
    let mut best: i64 = 0_i64;
    let mut best_set: bool = false;
    let mut best_count: i64 = 0_i64;
    for val2 in data.iter().copied() {
        let count2: Option<i64> = counts.get(&val2).copied();
        let mut count2_val: i64 = 0_i64;
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
    Err(
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
            "mode: no mode found".to_string(),
        ),
    )
}
fn multimode(
    data: &Vec<i64>,
) -> Result<Vec<i64>, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    if ((data.len() as i64) == (0_i64)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "multimode requires at least one data point".to_string(),
            ),
        );
    }
    let mut counts: HashMap<i64, i64> = HashMap::from([]);
    for val in data.iter().copied() {
        let existing: Option<i64> = counts.get(&val).copied();
        if let Some(existing) = existing {
            counts.insert(val, existing + (1_i64));
        } else {
            counts.insert(val, 1_i64);
        }
    }
    let mut max_count: i64 = 0_i64;
    for val2 in data.iter().copied() {
        let count2: Option<i64> = counts.get(&val2).copied();
        let mut count2_val: i64 = 0_i64;
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
            let mut count3_val: i64 = 0_i64;
            if let Some(count3) = count3 {
                count3_val = count3;
            }
            if count3_val == max_count {
                result.push(val3);
            }
            seen.insert(val3, true);
        }
    }
    Ok(result)
}
fn quantiles(
    data: &Vec<f64>,
    n: i64,
) -> Result<Vec<f64>, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    if ((data.len() as i64) < (2_i64)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "quantiles requires at least two data points".to_string(),
            ),
        );
    }
    if n < (1_i64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "quantiles: n must be at least 1".to_string(),
            ),
        );
    }
    let sorted_data: Vec<f64> = {
        let mut __sifr_sorted_v = (data).iter().copied().collect::<Vec<_>>();
        __sifr_sorted_v.sort_by(f64::total_cmp);
        __sifr_sorted_v
    };
    let m: i64 = sorted_data.len() as i64;
    let mut result: Vec<f64> = vec![];
    let mut i: i64 = 1_i64;
    while i < n {
        let idx_f: f64 = ((i as f64) * (m as f64)) / (n as f64);
        let mut idx: i64 = idx_f as i64;
        let frac: f64 = idx_f - (idx as f64);
        if idx >= m {
            idx = m - (1_i64);
        }
        if idx < (0_i64) {
            idx = 0_i64;
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
        let mut lo_val: f64 = 0.0_f64;
        if let Some(lo) = lo {
            lo_val = lo;
        }
        if frac > (0.0_f64) {
            let hi_idx: i64 = idx + (1_i64);
            if hi_idx < m {
                let hi: Option<f64> = Some(sorted_data[hi_idx as usize]);
                if let Some(hi) = hi {
                    lo_val += frac * (hi - lo_val);
                }
            }
        }
        result.push(lo_val);
        i += 1_i64;
    }
    Ok(result)
}
fn covariance(
    x: &Vec<f64>,
    y: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: i64 = x.len() as i64;
    if n < (2_i64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "covariance requires at least two data points".to_string(),
            ),
        );
    }
    if ((y.len() as i64) != n) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "covariance: x and y must have the same length".to_string(),
            ),
        );
    }
    let mx: f64 = _sum(x) / (n as f64);
    let my: f64 = _sum(y) / (n as f64);
    let mut total: f64 = 0.0_f64;
    let mut i: i64 = 0_i64;
    while i < n {
        let xi: Option<f64> = Some(x[i as usize]);
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
                total += (xi - mx) * (yi - my);
            }
        }
        i += 1_i64;
    }
    Ok(total / ((n - (1_i64)) as f64))
}
fn correlation(
    x: &Vec<f64>,
    y: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: i64 = x.len() as i64;
    if n < (2_i64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "correlation requires at least two data points".to_string(),
            ),
        );
    }
    if ((y.len() as i64) != n) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "correlation: x and y must have the same length".to_string(),
            ),
        );
    }
    let mx: f64 = _sum(x) / (n as f64);
    let my: f64 = _sum(y) / (n as f64);
    let mut cov_num: f64 = 0.0_f64;
    let mut sx_num: f64 = 0.0_f64;
    let mut sy_num: f64 = 0.0_f64;
    let mut i: i64 = 0_i64;
    while i < n {
        let xi: Option<f64> = Some(x[i as usize]);
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
                cov_num += (xi - mx) * (yi - my);
                sx_num += (xi - mx) * (xi - mx);
                sy_num += (yi - my) * (yi - my);
            }
        }
        i += 1_i64;
    }
    let sx: f64 = sqrt(sx_num / ((n - (1_i64)) as f64));
    let sy: f64 = sqrt(sy_num / ((n - (1_i64)) as f64));
    if sx == (0.0_f64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "correlation: x has zero variance".to_string(),
            ),
        );
    }
    if sy == (0.0_f64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "correlation: y has zero variance".to_string(),
            ),
        );
    }
    Ok((cov_num / ((n - (1_i64)) as f64)) / (sx * sy))
}
fn linear_regression(
    x: &Vec<f64>,
    y: &Vec<f64>,
) -> Result<Vec<f64>, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: i64 = x.len() as i64;
    if n < (2_i64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "linear_regression requires at least two data points".to_string(),
            ),
        );
    }
    if ((y.len() as i64) != n) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "linear_regression: x and y must have the same length".to_string(),
            ),
        );
    }
    let mx: f64 = _sum(x) / (n as f64);
    let my: f64 = _sum(y) / (n as f64);
    let mut num: f64 = 0.0_f64;
    let mut den: f64 = 0.0_f64;
    let mut i: i64 = 0_i64;
    while i < n {
        let xi: Option<f64> = Some(x[i as usize]);
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
                num += (xi - mx) * (yi - my);
                den += (xi - mx) * (xi - mx);
            }
        }
        i += 1_i64;
    }
    if den == (0.0_f64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "linear_regression: x has zero variance".to_string(),
            ),
        );
    }
    let slope: f64 = num / den;
    let intercept: f64 = my - (slope * mx);
    let mut result: Vec<f64> = vec![];
    result.push(slope);
    result.push(intercept);
    Ok(result)
}

// --- stdlib: sifr.test ---
fn assert_vector_eq(actual: &Vec<String>, expected: &Vec<String>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0_i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize].clone()) == expected.get(i as usize).cloned());
        i += 1_i64;
    }
}
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0_i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize]) == expected.get(i as usize).copied());
        i += 1_i64;
    }
}
// --- end stdlib ---

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Error {
    message: String,
}

impl Error {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for Error {
}

fn near(v: f64, target: f64, tol: f64) -> bool {
    if v < (target - tol) {
        return false;
    }
    if v > (target + tol) {
        return false;
    }
    true
}

fn collect_positive_actual() -> Vec<String> {
    let mut actual: Vec<String> = vec![];
    let data: Vec<f64> = vec![1.0_f64, 2.0_f64, 3.0_f64, 4.0_f64, 5.0_f64];
    let mut mean_ok: bool = true;
    let mut mean_v: f64 = 0.0_f64;
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (|| {
    let out_mean: f64 = mean(&data)?;
    mean_v = out_mean;
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        mean_ok = false;
    }
    actual.push(format!("{}", mean_ok && near(mean_v, 3.0_f64, 0.0001_f64)));
    let mut median_ok: bool = true;
    let mut median_v: f64 = 0.0_f64;
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (|| {
    let out_median: f64 = median(&data)?;
    median_v = out_median;
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        median_ok = false;
    }
    actual.push(format!("{}", median_ok && near(median_v, 3.0_f64, 0.0001_f64)));
    let mut variance_ok: bool = true;
    let mut variance_v: f64 = 0.0_f64;
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (|| {
    let out_variance: f64 = variance(&data)?;
    variance_v = out_variance;
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        variance_ok = false;
    }
    actual.push(format!("{}", variance_ok && near(variance_v, 2.5_f64, 0.0001_f64)));
    let mut stdev_ok: bool = true;
    let mut stdev_v: f64 = 0.0_f64;
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (|| {
    let out_stdev: f64 = stdev(&data)?;
    stdev_v = out_stdev;
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        stdev_ok = false;
    }
    actual.push(format!("{}", stdev_ok && near(stdev_v, 1.5811_f64, 0.001_f64)));
    let mut mode_ok: bool = true;
    let mut mode_v: i64 = 0_i64;
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (|| {
    let out_mode: i64 = mode(&vec![1_i64, 2_i64, 2_i64, 3_i64, 3_i64, 3_i64])?;
    mode_v = out_mode;
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        mode_ok = false;
    }
    actual.push(format!("{}", mode_ok && (mode_v == (3_i64))));
    let mut mm_ok: bool = true;
    let mut mm_v: Vec<i64> = vec![];
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (|| {
    let out_mm: Vec<i64> = multimode(&vec![1_i64, 2_i64, 2_i64, 3_i64, 3_i64])?;
    mm_v = out_mm;
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        mm_ok = false;
    }
    actual.push(format!("{}", mm_ok && ((mm_v.len() as i64) == (2_i64))));
    let mut q_ok: bool = true;
    let mut q_v: Vec<f64> = vec![];
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (|| {
    let out_q: Vec<f64> = quantiles(&vec![1.0_f64, 2.0_f64, 3.0_f64, 4.0_f64, 5.0_f64, 6.0_f64, 7.0_f64, 8.0_f64], 4_i64)?;
    q_v = out_q;
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        q_ok = false;
    }
    actual.push(format!("{}", q_ok && ((q_v.len() as i64) == (3_i64))));
    let x: Vec<f64> = vec![1.0_f64, 2.0_f64, 3.0_f64, 4.0_f64, 5.0_f64];
    let y: Vec<f64> = vec![2.0_f64, 4.0_f64, 6.0_f64, 8.0_f64, 10.0_f64];
    let mut cov_ok: bool = true;
    let mut cov_v: f64 = 0.0_f64;
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (|| {
    let out_cov: f64 = covariance(&x, &y)?;
    cov_v = out_cov;
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        cov_ok = false;
    }
    actual.push(format!("{}", cov_ok && near(cov_v, 5.0_f64, 0.0001_f64)));
    let mut corr_ok: bool = true;
    let mut corr_v: f64 = 0.0_f64;
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (|| {
    let out_corr: f64 = correlation(&x, &y)?;
    corr_v = out_corr;
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        corr_ok = false;
    }
    actual.push(format!("{}", corr_ok && near(corr_v, 1.0_f64, 0.0001_f64)));
    let mut lr_ok: bool = true;
    let mut lr_v: Vec<f64> = vec![];
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (|| {
    let out_lr: Vec<f64> = linear_regression(&x, &y)?;
    lr_v = out_lr;
    Ok(())
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
    let __sifr_index_i = 0_i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
};
        let lr_intercept: Option<f64> = {
    let __sifr_index_list = &lr_v;
    let __sifr_index_i = 1_i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
};
        if let Some(lr_slope) = lr_slope {
            lr_slope_ok = near(lr_slope, 2.0_f64, 0.0001_f64);
        }
        if let Some(lr_intercept) = lr_intercept {
            lr_intercept_ok = near(lr_intercept, 0.0_f64, 0.0001_f64);
        }
    }
    actual.push(format!("{}", ((lr_ok && ((lr_v.len() as i64) == (2_i64))) && lr_slope_ok) && lr_intercept_ok));
    let mut hmean_ok: bool = true;
    let mut hmean_v: f64 = 0.0_f64;
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (|| {
    let out_hmean: f64 = harmonic_mean(&vec![2.0_f64, 4.0_f64, 4.0_f64, 8.0_f64])?;
    hmean_v = out_hmean;
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        hmean_ok = false;
    }
    actual.push(format!("{}", hmean_ok && near(hmean_v, 3.5555555556_f64, 0.0001_f64)));
    let mut gmean_ok: bool = true;
    let mut gmean_v: f64 = 0.0_f64;
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (|| {
    let out_gmean: f64 = geometric_mean(&vec![4.0_f64, 9.0_f64])?;
    gmean_v = out_gmean;
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        gmean_ok = false;
    }
    actual.push(format!("{}", gmean_ok && near(gmean_v, 6.0_f64, 0.0001_f64)));
    actual
}

fn collect_error_actual_ok() -> Vec<bool> {
    let mut actual_ok: Vec<bool> = vec![];
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (|| {
    let bad_mean: f64 = mean(&vec![])?;
    let _ = format!("{}", bad_mean);
    actual_ok.push(true);
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        actual_ok.push(false);
    }
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (|| {
    let bad_hmean: f64 = harmonic_mean(&vec![0.0_f64, 1.0_f64])?;
    let _ = format!("{}", bad_hmean);
    actual_ok.push(true);
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        actual_ok.push(false);
    }
    actual_ok
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
