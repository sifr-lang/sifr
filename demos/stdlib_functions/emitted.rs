// src/main.rs
mod __sifr_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueError {
        pub message: String,
    }
    impl ValueError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for ValueError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ValueError {}
}
pub use __sifr_project_nominals::ValueError;
use ::sifr_runtime::SifrInt;
fn bisect_left<T: Clone + 'static + PartialOrd>(
    a: &[T],
    x: &T,
    lo: SifrInt,
    hi: Option<SifrInt>,
) -> SifrInt {
    let mut left: SifrInt = lo.clone();
    if &left < &SifrInt::from_i64(0) {
        left = SifrInt::from_i64(0);
    }
    let mut right: SifrInt = SifrInt::from(a.len());
    if (hi.is_none()) {
        right = SifrInt::from(a.len());
    } else {
        if let Some(hi) = hi.clone() {
            if (&hi < &SifrInt::from_i64(0)) {
                right = SifrInt::from_i64(0);
            } else {
                if (&hi > &SifrInt::from(a.len())) {
                    right = SifrInt::from(a.len());
                } else {
                    right = hi;
                }
            }
        }
    }
    while (&left < &right) {
        let mid: SifrInt = (&left + &right)
            .floor_div_known_nonzero(&SifrInt::from_i64(2));
        let val: Option<T> = {
            let __sifr_checked_read_collection = &a;
            let __sifr_checked_read_index = mid.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(val) = val {
            if val < *x {
                left = &mid + &SifrInt::from_i64(1);
            } else {
                right = mid;
            }
        } else {
            left = &mid + &SifrInt::from_i64(1);
        }
    }
    left.clone()
}
pub trait __SifrAdd: Sized {
    fn __sifr_add(self, rhs: Self) -> Self;
}
impl __SifrAdd for ::sifr_runtime::SifrInt {
    fn __sifr_add(self, rhs: Self) -> Self {
        self + rhs
    }
}
impl __SifrAdd for f64 {
    fn __sifr_add(self, rhs: Self) -> Self {
        self + rhs
    }
}
impl __SifrAdd for String {
    fn __sifr_add(mut self, rhs: Self) -> Self {
        self.push_str(&rhs);
        self
    }
}
fn pairwise<T: Clone + 'static>(data: &[T]) -> Vec<Vec<T>> {
    let mut result: Vec<Vec<T>> = vec![];
    let mut prev_values: Vec<T> = vec![];
    for value in data.iter().cloned() {
        if (&SifrInt::from(prev_values.len()) > &SifrInt::from_i64(0)) {
            let mut pair: Vec<T> = vec![];
            let prev: Option<T> = {
                let __sifr_checked_read_collection = &prev_values;
                let __sifr_checked_read_index = SifrInt::from_i64(0);
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            if let Some(prev) = prev {
                pair.push(prev.clone());
            }
            pair.push(value.clone());
            result.push(pair.to_vec());
            let __sifr_try_res: Result<(), IndexError> = (|| {
                {
                    let __assign_value = value.clone();
                    {
                        let __index_raw = SifrInt::from_i64(0);
                        let __index_normalized = __index_raw
                            .normalize_index_or_len(prev_values.len());
                        if let Some(__elem) = prev_values.get_mut(__index_normalized) {
                            *__elem = __assign_value;
                        }
                    }
                }
                Ok(())
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let _e = __sifr_try_err.clone();
                return result;
            }
        } else {
            prev_values.push(value.clone());
        }
    }
    result
}
fn batched<T: Clone + 'static>(
    data: &[T],
    n: SifrInt,
) -> Result<Vec<Vec<T>>, ValueError> {
    if (&n <= &SifrInt::from_i64(0)) {
        return Err(ValueError::new("batched: n must be > 0".to_string()));
    }
    let mut result: Vec<Vec<T>> = vec![];
    let mut current_batch: Vec<T> = vec![];
    for value in data.iter().cloned() {
        current_batch.push(value.clone());
        if (&SifrInt::from(current_batch.len()) == &n) {
            result.push(current_batch.to_vec());
            current_batch = vec![];
        }
    }
    if (&SifrInt::from(current_batch.len()) > &SifrInt::from_i64(0)) {
        result.push(current_batch.to_vec());
    }
    Ok(result)
}
const PI: f64 = 3.141592653589793_f64;
const E: f64 = 2.718281828459045_f64;
const TAU: f64 = 6.283185307179586_f64;
const INF: f64 = f64::INFINITY;
const NAN: f64 = f64::NAN;
fn sqrt(x: f64) -> f64 {
    ::sifr_stdlib::math::sqrt(x)
}
fn floor(x: f64) -> SifrInt {
    ::sifr_stdlib::math::floor(x).into_sifr_int()
}
fn ceil(x: f64) -> SifrInt {
    ::sifr_stdlib::math::ceil(x).into_sifr_int()
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
fn round_val(x: f64) -> SifrInt {
    ::sifr_stdlib::math::round_val(x).into_sifr_int()
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
fn trunc(x: f64) -> SifrInt {
    ::sifr_stdlib::math::trunc(x).into_sifr_int()
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
fn isqrt(n: SifrInt) -> SifrInt {
    ::sifr_stdlib::math::isqrt(::sifr_runtime::interop::SifrIntBridge::from(n))
        .into_sifr_int()
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
fn ldexp(m: f64, e: SifrInt) -> f64 {
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
fn factorial(n: SifrInt) -> SifrInt {
    if &n < &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    let mut result: SifrInt = SifrInt::from_i64(1);
    let mut i: SifrInt = SifrInt::from_i64(2);
    while &i <= &n {
        result = &result * &i;
        i = &i + &SifrInt::from_i64(1);
    }
    result.clone()
}
fn gcd(a: SifrInt, b: SifrInt) -> SifrInt {
    let mut x: SifrInt = a.clone();
    let mut y: SifrInt = b.clone();
    if &x < &SifrInt::from_i64(0) {
        x = &SifrInt::from_i64(0) - &x;
    }
    if &y < &SifrInt::from_i64(0) {
        y = &SifrInt::from_i64(0) - &y;
    }
    while (&y != &SifrInt::from_i64(0)) {
        let temp: SifrInt = y.clone();
        y = x.floor_mod_known_nonzero(&y);
        x = temp;
    }
    x.clone()
}
fn lcm(a: SifrInt, b: SifrInt) -> SifrInt {
    if &a == &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    if &b == &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    let g: SifrInt = gcd(a.clone(), b.clone());
    if &g == &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    let mut x: SifrInt = a.clone();
    if &x < &SifrInt::from_i64(0) {
        x = &SifrInt::from_i64(0) - &x;
    }
    let mut y: SifrInt = b.clone();
    if &y < &SifrInt::from_i64(0) {
        y = &SifrInt::from_i64(0) - &y;
    }
    &x.floor_div_known_nonzero(&g) * &y
}
fn comb(n: SifrInt, k: SifrInt) -> SifrInt {
    if &k < &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    if &k > &n {
        return SifrInt::from_i64(0);
    }
    if &k == &SifrInt::from_i64(0) {
        return SifrInt::from_i64(1);
    }
    if &k == &n {
        return SifrInt::from_i64(1);
    }
    let mut r: SifrInt = k.clone();
    if &r > &(&n - &k) {
        r = &n - &k;
    }
    let mut result: SifrInt = SifrInt::from_i64(1);
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &r) {
        result = &result * &(&n - &i);
        let divisor: SifrInt = &i + &SifrInt::from_i64(1);
        if (&divisor == &SifrInt::from_i64(0)) {
            return SifrInt::from_i64(0);
        }
        result = result.floor_div_known_nonzero(&divisor);
        i = &i + &SifrInt::from_i64(1);
    }
    result.clone()
}
fn perm(n: SifrInt, k: SifrInt) -> SifrInt {
    if &k < &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    if &k > &n {
        return SifrInt::from_i64(0);
    }
    let mut result: SifrInt = SifrInt::from_i64(1);
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &k {
        result = &result * &(&n - &i);
        i = &i + &SifrInt::from_i64(1);
    }
    result.clone()
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
fn prod(data: &[SifrInt]) -> SifrInt {
    let mut result: SifrInt = SifrInt::from_i64(1);
    for val in data.iter().cloned() {
        result = &result * &val;
    }
    result.clone()
}
fn _copy_float_list(data: &[f64]) -> Vec<f64> {
    let mut out: Vec<f64> = vec![];
    for value in data.iter().copied() {
        out.push(value);
    }
    out
}
fn dist(p: &[f64], q: &[f64]) -> f64 {
    dist_impl(_copy_float_list(p), _copy_float_list(q))
}
fn fsum(data: &[f64]) -> f64 {
    fsum_impl(_copy_float_list(data))
}
fn sumprod(p: &[f64], q: &[f64]) -> f64 {
    sumprod_impl(_copy_float_list(p), _copy_float_list(q))
}
fn frexp_mantissa(x: f64) -> f64 {
    let parts: Vec<f64> = frexp(x);
    let m: Option<f64> = {
        let __sifr_checked_read_collection = &parts;
        let __sifr_checked_read_index = SifrInt::from_i64(0);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(m) = m else {
        return NAN;
    };
    m
}
fn frexp_exponent(x: f64) -> SifrInt {
    let parts: Vec<f64> = frexp(x);
    let exp_val: Option<f64> = {
        let __sifr_checked_read_collection = &parts;
        let __sifr_checked_read_index = SifrInt::from_i64(1);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(exp_val) = exp_val else {
        return SifrInt::from_i64(0);
    };
    trunc(exp_val)
}
fn modf_fractional(x: f64) -> f64 {
    let parts: Vec<f64> = modf(x);
    let f: Option<f64> = {
        let __sifr_checked_read_collection = &parts;
        let __sifr_checked_read_index = SifrInt::from_i64(0);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(f) = f else {
        return NAN;
    };
    f
}
fn modf_integral(x: f64) -> f64 {
    let parts: Vec<f64> = modf(x);
    let i: Option<f64> = {
        let __sifr_checked_read_collection = &parts;
        let __sifr_checked_read_index = SifrInt::from_i64(1);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(i) = i else {
        return NAN;
    };
    i
}
fn pow(x: f64, y: f64) -> f64 {
    pow_val(x, y)
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrIoNativeFileHandle {
    _id: String,
}
impl __SifrIoNativeFileHandle {
    fn new(id: String) -> Self {
        let __sifr_field_init_0: String = id;
        Self { _id: __sifr_field_init_0 }
    }
}
impl __SifrIoNativeFileHandle {}
impl ::std::fmt::Display for __SifrIoNativeFileHandle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "NativeFileHandle(_id={})", self._id)
    }
}
fn read_text(path: &str) -> Result<String, IOError> {
    ::sifr_stdlib::fs::read_text(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn write_text(path: &str, content: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::write_text(path, content)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn exists(path: &str) -> bool {
    ::sifr_stdlib::fs::exists(path)
}
fn read_lines(path: &str) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::read_lines(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn append_text(path: &str, content: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::append_text(path, content)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _open_file(path: &str, mode: &str) -> Result<String, IOError> {
    ::sifr_stdlib::fs::open_file(path, mode)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_read(handle: &str) -> Result<String, IOError> {
    ::sifr_stdlib::fs::file_read(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_write(handle: &str, data: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::file_write(handle, data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_readline(handle: &str) -> Result<Option<String>, IOError> {
    ::sifr_stdlib::fs::file_readline(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_readlines(handle: &str) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::file_readlines(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_close(handle: &str) {
    ::sifr_stdlib::fs::file_close(handle);
}
fn _file_read_bytes(handle: &str, size: Option<SifrInt>) -> Result<Vec<u8>, IOError> {
    ::sifr_stdlib::fs::file_read_bytes(
            handle,
            size.map(::sifr_runtime::interop::SifrIntBridge::from),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_write_bytes(handle: &str, data: &[u8]) -> Result<(), IOError> {
    ::sifr_stdlib::fs::file_write_bytes(handle, data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_flush(handle: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::file_flush(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_seek(
    handle: &str,
    offset: SifrInt,
    whence: SifrInt,
) -> Result<SifrInt, IOError> {
    ::sifr_stdlib::fs::file_seek(
            handle,
            ::sifr_runtime::interop::SifrIntBridge::from(offset),
            ::sifr_runtime::interop::SifrIntBridge::from(whence),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_tell(handle: &str) -> Result<SifrInt, IOError> {
    ::sifr_stdlib::fs::file_tell(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn open_file(path: &str, mode: &str) -> Result<__SifrIoNativeFileHandle, IOError> {
    let __sifr_try_res: Result<Result<__SifrIoNativeFileHandle, IOError>, IOError> = (|| {
        let handle_id: String = _open_file(path, mode)?;
        Ok(Ok(__SifrIoNativeFileHandle::new(handle_id)))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(e);
        }
    }
}
fn file_read(handle: &__SifrIoNativeFileHandle) -> Result<String, IOError> {
    _file_read(&handle._id.clone())
}
fn file_write(handle: &__SifrIoNativeFileHandle, data: &str) -> Result<(), IOError> {
    _file_write(&handle._id.clone(), data)
}
fn file_readline(handle: &__SifrIoNativeFileHandle) -> Result<Option<String>, IOError> {
    _file_readline(&handle._id.clone())
}
fn file_readlines(handle: &__SifrIoNativeFileHandle) -> Result<Vec<String>, IOError> {
    _file_readlines(&handle._id.clone())
}
fn file_close(handle: &__SifrIoNativeFileHandle) {
    _file_close(&handle._id.clone());
}
fn file_read_bytes(
    handle: &__SifrIoNativeFileHandle,
    size: Option<SifrInt>,
) -> Result<Vec<u8>, IOError> {
    _file_read_bytes(&handle._id.clone(), size.clone())
}
fn file_write_bytes(
    handle: &__SifrIoNativeFileHandle,
    data: &[u8],
) -> Result<(), IOError> {
    _file_write_bytes(&handle._id.clone(), data)
}
fn file_flush(handle: &__SifrIoNativeFileHandle) -> Result<(), IOError> {
    _file_flush(&handle._id.clone())
}
fn file_seek(
    handle: &__SifrIoNativeFileHandle,
    offset: SifrInt,
    whence: SifrInt,
) -> Result<SifrInt, IOError> {
    _file_seek(&handle._id.clone(), offset.clone(), whence.clone())
}
fn file_tell(handle: &__SifrIoNativeFileHandle) -> Result<SifrInt, IOError> {
    _file_tell(&handle._id.clone())
}
fn getcwd() -> Result<String, IOError> {
    ::sifr_stdlib::fs::getcwd()
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn listdir(path: &str) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::listdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn mkdir(path: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::mkdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn rmdir(path: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::rmdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn remove_file(path: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::remove_file(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn rename(src: &str, dst: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::rename(src, dst)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn chdir(path: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::chdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn stat_size(path: &str) -> Result<SifrInt, IOError> {
    ::sifr_stdlib::fs::stat_size(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn disk_usage(path: &str) -> Vec<SifrInt> {
    ::sifr_stdlib::fs::disk_usage(path)
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
        .collect()
}
fn is_file(path: &str) -> bool {
    ::sifr_stdlib::fs::is_file(path)
}
fn is_dir(path: &str) -> bool {
    ::sifr_stdlib::fs::is_dir(path)
}
fn copy_file(src: &str, dst: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::copy_file(src, dst)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn walk_dir(path: &str) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::walk_dir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn rmdir_all(path: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::rmdir_all(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn gettempdir() -> String {
    ::sifr_stdlib::fs::gettempdir()
}
fn makedirs(path: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::makedirs(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn touch(path: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::touch(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn resolve_path(path: &str) -> Result<String, IOError> {
    ::sifr_stdlib::fs::resolve_path(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn iterdir(path: &str) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::iterdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn glob_pattern(dir: &str, pattern: &str) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::glob_pattern(dir, pattern)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn rglob_pattern(dir: &str, pattern: &str) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::rglob_pattern(dir, pattern)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn basename(path: &str) -> String {
    let __sifr_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
    let mut i: SifrInt = &SifrInt::from(__sifr_chars_path.len()) - &SifrInt::from_i64(1);
    while (&i >= &SifrInt::from_i64(0)) {
        let ch: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_path.len());
            __sifr_chars_path.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(ch) = ch {
            if (ch == "/") {
                return {
                    let _slice_src = &__sifr_chars_path;
                    let _slice_len = _slice_src.len();
                    let _slice_start = (&i + &SifrInt::from_i64(1))
                        .clamp_slice_bound(_slice_len);
                    let _slice_stop = _slice_len;
                    String::from_iter(
                        _slice_src
                            .iter()
                            .skip(_slice_start)
                            .take(_slice_stop.saturating_sub(_slice_start))
                            .copied(),
                    )
                };
            }
        }
        i = &i - &SifrInt::from_i64(1);
    }
    {
        let mut __sifr_concat: String = String::with_capacity(path.len() + 0usize);
        __sifr_concat.push_str(path);
        __sifr_concat.push_str("");
        __sifr_concat
    }
}
fn stem(path: &str) -> String {
    let base: String = basename(path);
    let __sifr_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
    let mut i: SifrInt = &SifrInt::from(__sifr_chars_base.len()) - &SifrInt::from_i64(1);
    while (&i > &SifrInt::from_i64(0)) {
        let ch: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_base.len());
            __sifr_chars_base.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(ch) = ch {
            if (ch == ".") {
                return {
                    let _slice_src = &__sifr_chars_base;
                    let _slice_len = _slice_src.len();
                    let _slice_start = 0;
                    let _slice_stop = i.clamp_slice_bound(_slice_len);
                    String::from_iter(
                        _slice_src
                            .iter()
                            .skip(_slice_start)
                            .take(_slice_stop.saturating_sub(_slice_start))
                            .copied(),
                    )
                };
            }
        }
        i = &i - &SifrInt::from_i64(1);
    }
    {
        let mut __sifr_concat: String = String::with_capacity(base.len() + 0usize);
        __sifr_concat.push_str(base.as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    }
}
fn is_absolute(path: &str) -> bool {
    let __sifr_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
    if (&SifrInt::from(__sifr_chars_path.len()) == &SifrInt::from_i64(0)) {
        return false;
    }
    if (&SifrInt::from(__sifr_chars_path.len()) >= &SifrInt::from_i64(3)) {
        let colon: Option<String> = ({
            let __sifr_string_index = SifrInt::from_i64(1);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_path.len());
            __sifr_chars_path.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        let sep: Option<String> = ({
            let __sifr_string_index = SifrInt::from_i64(2);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_path.len());
            __sifr_chars_path.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(colon) = colon {
            if let Some(sep) = sep {
                if (colon == ":") && ((sep == "/") || (sep == "\\")) {
                    return true;
                }
            }
        }
    }
    let first: Option<String> = ({
        let __sifr_string_index = SifrInt::from_i64(0);
        let __sifr_string_index_normalized = __sifr_string_index
            .normalize_index_or_len(__sifr_chars_path.len());
        __sifr_chars_path.get(__sifr_string_index_normalized)
    })
        .map(|c| c.to_string());
    if let Some(first) = first {
        if (first == "/") || (first == "\\") {
            return true;
        }
    }
    false
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
    __SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(FloatOverflowError),
    __SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
        FloatPrecisionLossError,
    ),
}
impl From<FloatOverflowError>
for __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
    fn from(value: FloatOverflowError) -> Self {
        __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
            value,
        )
    }
}
impl ::std::fmt::Display
for __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
        }
    }
}
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
fn _float_int(
    value: SifrInt,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let __sifr_try_res: Result<
        Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError>,
        __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0,
    > = (|| {
        let converted: f64 = value
            .clone()
            .checked_to_f64()
            .map_err(|__sifr_float_error| match __sifr_float_error {
                ::sifr_runtime::IntegerFloatConversionError::Overflow => {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                        FloatOverflowError::new(
                            "exact integer is outside the finite float range".to_string(),
                        ),
                    )
                }
                ::sifr_runtime::IntegerFloatConversionError::PrecisionLoss => {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                        FloatPrecisionLossError::new(
                            "exact integer cannot be represented without float precision loss"
                                .to_string(),
                        ),
                    )
                }
            })?;
        Ok(Ok(converted))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            match __sifr_try_err {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let error = __sifr_try_variant_error.clone();
                    return Err(
                        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                            error.message.clone(),
                        ),
                    );
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let error = __sifr_try_variant_error.clone();
                    return Err(
                        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                            error.message.clone(),
                        ),
                    );
                }
            }
        }
    }
}
fn harmonic_mean(
    data: &[f64],
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: SifrInt = SifrInt::from(data.len());
    if (&n == &SifrInt::from_i64(0)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "harmonic_mean requires at least one data point".to_string(),
            ),
        );
    }
    let mut total: f64 = 0.0_f64;
    for val in data.iter().copied() {
        if (val <= (0.0_f64)) {
            return Err(
                __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                    "harmonic_mean requires positive values".to_string(),
                ),
            );
        }
        total += (1.0_f64) / val;
    }
    let __sifr_try_res: Result<
        Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError>,
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
    > = (|| {
        let numerator: f64 = _float_int(n.clone())?;
        Ok(Ok(numerator / total))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let error = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                    error.message.clone(),
                ),
            );
        }
    }
}
fn median_low(
    data: &[f64],
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: SifrInt = SifrInt::from(data.len());
    if (&n == &SifrInt::from_i64(0)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "median_low requires at least one data point".to_string(),
            ),
        );
    }
    let sorted_data: Vec<f64> = {
        let mut __sifr_sorted_v = (data).iter().copied().collect::<Vec<_>>();
        __sifr_sorted_v.sort_by(f64::total_cmp);
        __sifr_sorted_v
    };
    let mid: SifrInt = n.floor_div_known_nonzero(&SifrInt::from_i64(2));
    if (&n.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0)) {
        let val: Option<f64> = {
            let __sifr_checked_read_collection = &sorted_data;
            let __sifr_checked_read_index = &mid - &SifrInt::from_i64(1);
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(val) = val {
            return Ok(val);
        }
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "median_low: index error".to_string(),
            ),
        );
    } else {
        let val2: Option<f64> = {
            let __sifr_checked_read_collection = &sorted_data;
            let __sifr_checked_read_index = mid.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(val2) = val2 {
            return Ok(val2);
        }
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "median_low: index error".to_string(),
            ),
        );
    }
}
fn median_high(
    data: &[f64],
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: SifrInt = SifrInt::from(data.len());
    if (&n == &SifrInt::from_i64(0)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "median_high requires at least one data point".to_string(),
            ),
        );
    }
    let sorted_data: Vec<f64> = {
        let mut __sifr_sorted_v = (data).iter().copied().collect::<Vec<_>>();
        __sifr_sorted_v.sort_by(f64::total_cmp);
        __sifr_sorted_v
    };
    let mid: SifrInt = n.floor_div_known_nonzero(&SifrInt::from_i64(2));
    let val: Option<f64> = {
        let __sifr_checked_read_collection = &sorted_data;
        let __sifr_checked_read_index = mid.clone();
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    if let Some(val) = val {
        return Ok(val);
    }
    Err(
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
            "median_high: index error".to_string(),
        ),
    )
}
fn capwords(s: &str) -> String {
    let normalized: String = s
        .replace('\t', " ")
        .replace('\n', " ")
        .replace('\r', " ")
        .replace('\u{b}', " ")
        .replace('\u{c}', " ");
    let words: Vec<String> = normalized
        .split(' ')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let mut result: String = "".to_string();
    let mut first: bool = true;
    for word in words.iter().cloned() {
        let __sifr_chars_word: Vec<char> = word.chars().collect::<Vec<char>>();
        if (&SifrInt::from(__sifr_chars_word.len()) > &SifrInt::from_i64(0)) {
            if !first {
                result.push(' ');
            }
            first = false;
            let cap: String = {
                let _s = word.clone();
                let mut _c = _s.chars();
                _c.next()
                    .map(|f| f.to_uppercase().to_string() + &_c.as_str().to_lowercase())
                    .unwrap_or_default()
            };
            result.push_str(cap.as_str());
        }
    }
    result
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct IOError {
    message: String,
    kind: String,
}
impl IOError {
    fn new(message: String) -> Self {
        Self {
            message,
            kind: "Other".to_string(),
        }
    }
}
impl ::std::fmt::Display for IOError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for IOError {}
fn __io_err<E: ::std::fmt::Display + 'static>(e: E) -> IOError {
    let msg = e.to_string();
    let kind = {
        let __sifr_io_kind = (&e as &dyn ::std::any::Any)
            .downcast_ref::<std::io::Error>()
            .map(::std::io::Error::kind);
        match __sifr_io_kind {
            Some(::std::io::ErrorKind::NotFound) => "FileNotFound".to_string(),
            Some(::std::io::ErrorKind::PermissionDenied) => {
                "PermissionDenied".to_string()
            }
            Some(::std::io::ErrorKind::AlreadyExists) => "FileExists".to_string(),
            Some(::std::io::ErrorKind::IsADirectory) => "IsADirectory".to_string(),
            Some(::std::io::ErrorKind::NotADirectory) => "NotADirectory".to_string(),
            Some(::std::io::ErrorKind::DirectoryNotEmpty) => {
                "DirectoryNotEmpty".to_string()
            }
            _ => "Other".to_string(),
        }
    };
    IOError { message: msg, kind }
}
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
impl ::std::error::Error for Error {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FloatOverflowError {
    message: String,
}
impl FloatOverflowError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for FloatOverflowError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for FloatOverflowError {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FloatPrecisionLossError {
    message: String,
}
impl FloatPrecisionLossError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for FloatPrecisionLossError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for FloatPrecisionLossError {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct IndexError {
    message: String,
}
impl IndexError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for IndexError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for IndexError {}
impl From<IOError> for Error {
    fn from(err: IOError) -> Self {
        Self::new(err.message)
    }
}
impl From<ValueError> for Error {
    fn from(err: ValueError) -> Self {
        Self::new(err.message)
    }
}
impl From<FloatOverflowError> for Error {
    fn from(err: FloatOverflowError) -> Self {
        Self::new(err.message)
    }
}
impl From<FloatPrecisionLossError> for Error {
    fn from(err: FloatPrecisionLossError) -> Self {
        Self::new(err.message)
    }
}
impl From<IndexError> for Error {
    fn from(err: IndexError) -> Self {
        Self::new(err.message)
    }
}
fn main() {
    println!("=== Math Functions ===");
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(16usize + 0usize);
        __sifr_concat.push_str("factorial(10) = "); __sifr_concat.push_str(format!("{}",
        factorial(SifrInt::from_i64(10))) .as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(14usize + 0usize);
        __sifr_concat.push_str("gcd(48, 18) = "); __sifr_concat.push_str(format!("{}",
        gcd(SifrInt::from_i64(48), SifrInt::from_i64(18))) .as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(12usize + 0usize);
        __sifr_concat.push_str("lcm(4, 6) = "); __sifr_concat.push_str(format!("{}",
        lcm(SifrInt::from_i64(4), SifrInt::from_i64(6))) .as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(14usize + 0usize);
        __sifr_concat.push_str("comb(10, 3) = "); __sifr_concat.push_str(format!("{}",
        comb(SifrInt::from_i64(10), SifrInt::from_i64(3))) .as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(13usize + 0usize);
        __sifr_concat.push_str("perm(5, 3) = "); __sifr_concat.push_str(format!("{}",
        perm(SifrInt::from_i64(5), SifrInt::from_i64(3))) .as_str()); __sifr_concat }
    );
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3),
        SifrInt::from_i64(4), SifrInt::from_i64(5)
    ];
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(20usize + 0usize);
        __sifr_concat.push_str("prod([1,2,3,4,5]) = "); __sifr_concat
        .push_str(format!("{}", prod(& nums)) .as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(11usize + 0usize);
        __sifr_concat.push_str("exp(1.0) = "); __sifr_concat.push_str(format!("{}",
        exp(1.0_f64)) .as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(16usize + 0usize);
        __sifr_concat.push_str("isfinite(1.0) = "); __sifr_concat.push_str(format!("{}",
        isfinite(1.0_f64)) .as_str()); __sifr_concat }
    );
    println!("=== Statistics Functions ===");
    let data: Vec<f64> = vec![1.0_f64, 2.0_f64, 3.0_f64, 4.0_f64, 5.0_f64];
    let even: Vec<f64> = vec![1.0_f64, 2.0_f64, 3.0_f64, 4.0_f64];
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (||
    {
        let hm: f64 = harmonic_mean(&data)?;
        let ml: f64 = median_low(&even)?;
        let mh: f64 = median_high(&even)?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(16usize +
            0usize); __sifr_concat.push_str("harmonic_mean = "); __sifr_concat
            .push_str(format!("{}", hm) .as_str()); __sifr_concat }
        );
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(13usize +
            0usize); __sifr_concat.push_str("median_low = "); __sifr_concat
            .push_str(format!("{}", ml) .as_str()); __sifr_concat }
        );
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(14usize +
            0usize); __sifr_concat.push_str("median_high = "); __sifr_concat
            .push_str(format!("{}", mh) .as_str()); __sifr_concat }
        );
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let se = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(18usize +
            0usize); __sifr_concat.push_str("statistics error: "); __sifr_concat
            .push_str(se.message.clone().as_str()); __sifr_concat }
        );
    }
    println!("=== String Functions ===");
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(11usize + 0usize);
        __sifr_concat.push_str("capwords = "); __sifr_concat.push_str(capwords(&
        "hello world test".to_string()).as_str()); __sifr_concat }
    );
    println!("=== Path Functions ===");
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(7usize + 0usize);
        __sifr_concat.push_str("stem = "); __sifr_concat.push_str(stem(&
        "/docs/report.pdf".to_string()).as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(14usize + 0usize);
        __sifr_concat.push_str("is_absolute = "); __sifr_concat.push_str(format!("{}",
        is_absolute(& "/usr/bin".to_string())) .as_str()); __sifr_concat }
    );
    println!("=== Generic Bisect ===");
    let floats: Vec<f64> = vec![1.0_f64, 2.0_f64, 3.0_f64, 4.0_f64, 5.0_f64];
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(27usize + 0usize);
        __sifr_concat.push_str("bisect_left(floats, 2.5) = "); __sifr_concat
        .push_str(format!("{}", bisect_left(& floats, & (2.5_f64), SifrInt::from_i64(0),
        None)) .as_str()); __sifr_concat }
    );
    println!("=== Itertools ===");
    let items: Vec<SifrInt> = vec![
        SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3),
        SifrInt::from_i64(4)
    ];
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(11usize + 0usize);
        __sifr_concat.push_str("pairwise = "); __sifr_concat.push_str(format!("{:?}",
        pairwise(& (items).iter().cloned().collect::< Vec < _ >> ())) .as_str());
        __sifr_concat }
    );
    let items2: Vec<SifrInt> = vec![
        SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3),
        SifrInt::from_i64(4), SifrInt::from_i64(5), SifrInt::from_i64(6),
        SifrInt::from_i64(7), SifrInt::from_i64(8)
    ];
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let bat: Vec<Vec<SifrInt>> = batched(
            &(items2).iter().cloned().collect::<Vec<_>>(),
            SifrInt::from_i64(3),
        )?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(10usize +
            0usize); __sifr_concat.push_str("batched = "); __sifr_concat
            .push_str(format!("{:?}", bat) .as_str()); __sifr_concat }
        );
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("error: {}", e.message.clone());
    }
    println!("Done!");
}
