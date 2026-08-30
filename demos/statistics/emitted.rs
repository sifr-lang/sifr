// src/main.rs
use ::std::collections::HashMap;

use ::sifr_runtime::SifrInt;

// --- stdlib: _sifr.math ---
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

// --- stdlib: sifr.math ---
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
    let g: SifrInt = gcd((a).clone(), (b).clone());
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
fn prod(data: &Vec<SifrInt>) -> SifrInt {
    let mut result: SifrInt = SifrInt::from_i64(1);
    for val in data.iter().cloned() {
        result = &result * &val;
    }
    result.clone()
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

// --- stdlib: sifr.statistics ---
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
fn _sum(data: &Vec<f64>) -> f64 {
    let mut total: f64 = 0.0_f64;
    for val in data.iter().copied() {
        total += val;
    }
    total
}
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
fn _divide_by_int(
    numerator: f64,
    denominator: SifrInt,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let __sifr_try_res: Result<
        Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError>,
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
    > = (|| {
        let divisor: f64 = _float_int((denominator).clone())?;
        Ok(Ok(numerator / divisor))
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
fn mean(
    data: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let count: SifrInt = SifrInt::from(data.len());
    if (&count == &SifrInt::from_i64(0)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "mean requires at least one data point".to_string(),
            ),
        );
    }
    let total: f64 = _sum(data);
    _divide_by_int(total, (count).clone())
}
fn median(
    data: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: SifrInt = SifrInt::from(data.len());
    if (&n == &SifrInt::from_i64(0)) {
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
    let mid: SifrInt = n.floor_div_known_nonzero(&SifrInt::from_i64(2));
    if (&n.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0)) {
        let a: Option<f64> = {
            let __sifr_checked_read_collection = &sorted_data;
            let __sifr_checked_read_index = &mid - &SifrInt::from_i64(1);
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        let b: Option<f64> = {
            let __sifr_checked_read_collection = &sorted_data;
            let __sifr_checked_read_index = mid.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
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
            let __sifr_checked_read_collection = &sorted_data;
            let __sifr_checked_read_index = mid.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
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
    let n: SifrInt = SifrInt::from(data.len());
    if (&n < &SifrInt::from_i64(2)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "variance requires at least two data points".to_string(),
            ),
        );
    }
    let __sifr_try_res: Result<
        (f64,),
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
    > = (|| {
        let avg: f64 = _divide_by_int(_sum(data), (n).clone())?;
        Ok((avg,))
    })();
    let (avg,) = match __sifr_try_res {
        Ok(__sifr_try_bindings) => __sifr_try_bindings,
        Err(__sifr_try_err) => {
            let error = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                    error.message.clone(),
                ),
            );
        }
    };
    let mut total: f64 = 0.0_f64;
    for val in data.iter().copied() {
        let diff: f64 = val - avg;
        total += diff * diff;
    }
    _divide_by_int(total, &n - &SifrInt::from_i64(1))
}
fn stdev(
    data: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: SifrInt = SifrInt::from(data.len());
    if (&n < &SifrInt::from_i64(2)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "stdev requires at least two data points".to_string(),
            ),
        );
    }
    let __sifr_try_res: Result<
        (f64,),
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
    > = (|| {
        let avg: f64 = _divide_by_int(_sum(data), (n).clone())?;
        Ok((avg,))
    })();
    let (avg,) = match __sifr_try_res {
        Ok(__sifr_try_bindings) => __sifr_try_bindings,
        Err(__sifr_try_err) => {
            let error = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                    error.message.clone(),
                ),
            );
        }
    };
    let mut total: f64 = 0.0_f64;
    for val in data.iter().copied() {
        let diff: f64 = val - avg;
        total += diff * diff;
    }
    let __sifr_try_res: Result<
        (f64,),
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
    > = (|| {
        let v: f64 = _divide_by_int(total, &n - &SifrInt::from_i64(1))?;
        Ok((v,))
    })();
    let (v,) = match __sifr_try_res {
        Ok(__sifr_try_bindings) => __sifr_try_bindings,
        Err(__sifr_try_err) => {
            let error = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                    error.message.clone(),
                ),
            );
        }
    };
    Ok(sqrt(v))
}
fn harmonic_mean(
    data: &Vec<f64>,
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
        let numerator: f64 = _float_int((n).clone())?;
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
fn geometric_mean(
    data: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: SifrInt = SifrInt::from(data.len());
    if (&n == &SifrInt::from_i64(0)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "geometric_mean requires at least one data point".to_string(),
            ),
        );
    }
    let mut log_sum: f64 = 0.0_f64;
    for val in data.iter().copied() {
        if (val <= (0.0_f64)) {
            return Err(
                __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                    "geometric_mean requires positive values".to_string(),
                ),
            );
        }
        log_sum += log(val);
    }
    let __sifr_try_res: Result<
        Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError>,
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
    > = (|| {
        let mean_log: f64 = _divide_by_int(log_sum, (n).clone())?;
        Ok(Ok(exp(mean_log)))
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
fn mode(
    data: &Vec<SifrInt>,
) -> Result<SifrInt, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    if (&SifrInt::from(data.len()) == &SifrInt::from_i64(0)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "mode requires at least one data point".to_string(),
            ),
        );
    }
    let mut counts: HashMap<SifrInt, SifrInt> = HashMap::from([]);
    for val in data.iter().cloned() {
        let existing: Option<SifrInt> = counts.get(&val).cloned();
        if let Some(existing) = existing.clone() {
            {
                let __assign_value = &existing + &SifrInt::from_i64(1);
                {
                    let __assign_key = val.clone();
                    counts.insert(__assign_key, __assign_value);
                }
            }
        } else {
            {
                let __assign_value = SifrInt::from_i64(1);
                {
                    let __assign_key = val.clone();
                    counts.insert(__assign_key, __assign_value);
                }
            }
        }
    }
    let mut best: SifrInt = SifrInt::from_i64(0);
    let mut best_set: bool = false;
    let mut best_count: SifrInt = SifrInt::from_i64(0);
    for val2 in data.iter().cloned() {
        let count2: Option<SifrInt> = counts.get(&val2).cloned();
        let mut count2_val: SifrInt = SifrInt::from_i64(0);
        if let Some(count2) = count2.clone() {
            count2_val = count2;
        }
        if (&count2_val > &best_count) {
            best_count = count2_val;
            best = val2;
            best_set = true;
        }
    }
    if best_set {
        return Ok(best.clone());
    }
    Err(
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
            "mode: no mode found".to_string(),
        ),
    )
}
fn multimode(
    data: &Vec<SifrInt>,
) -> Result<Vec<SifrInt>, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    if (&SifrInt::from(data.len()) == &SifrInt::from_i64(0)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "multimode requires at least one data point".to_string(),
            ),
        );
    }
    let mut counts: HashMap<SifrInt, SifrInt> = HashMap::from([]);
    for val in data.iter().cloned() {
        let existing: Option<SifrInt> = counts.get(&val).cloned();
        if let Some(existing) = existing.clone() {
            {
                let __assign_value = &existing + &SifrInt::from_i64(1);
                {
                    let __assign_key = val.clone();
                    counts.insert(__assign_key, __assign_value);
                }
            }
        } else {
            {
                let __assign_value = SifrInt::from_i64(1);
                {
                    let __assign_key = val.clone();
                    counts.insert(__assign_key, __assign_value);
                }
            }
        }
    }
    let mut max_count: SifrInt = SifrInt::from_i64(0);
    for val2 in data.iter().cloned() {
        let count2: Option<SifrInt> = counts.get(&val2).cloned();
        let mut count2_val: SifrInt = SifrInt::from_i64(0);
        if let Some(count2) = count2.clone() {
            count2_val = count2;
        }
        if (&count2_val > &max_count) {
            max_count = count2_val;
        }
    }
    let mut result: Vec<SifrInt> = vec![];
    let mut seen: HashMap<SifrInt, bool> = HashMap::from([]);
    for val3 in data.iter().cloned() {
        let already_opt: Option<bool> = seen.get(&val3).cloned();
        let mut already: bool = false;
        if let Some(already_opt) = already_opt {
            already = already_opt;
        }
        if !already {
            let count3: Option<SifrInt> = counts.get(&val3).cloned();
            let mut count3_val: SifrInt = SifrInt::from_i64(0);
            if let Some(count3) = count3.clone() {
                count3_val = count3;
            }
            if (&count3_val == &max_count) {
                result.push(val3.clone());
            }
            {
                let __assign_value = true;
                {
                    let __assign_key = val3.clone();
                    seen.insert(__assign_key, __assign_value);
                }
            }
        }
    }
    Ok(result)
}
fn quantiles(
    data: &Vec<f64>,
    n: SifrInt,
) -> Result<Vec<f64>, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    if (&SifrInt::from(data.len()) < &SifrInt::from_i64(2)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "quantiles requires at least two data points".to_string(),
            ),
        );
    }
    if (&n < &SifrInt::from_i64(1)) {
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
    let m: SifrInt = SifrInt::from(sorted_data.len());
    let mut result: Vec<f64> = vec![];
    let mut i: SifrInt = SifrInt::from_i64(1);
    while (&i < &n) {
        let __sifr_try_res: Result<
            (f64, f64, f64),
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
        > = (|| {
            let i_float: f64 = _float_int((i).clone())?;
            let m_float: f64 = _float_int((m).clone())?;
            let n_float: f64 = _float_int((n).clone())?;
            Ok((i_float, m_float, n_float))
        })();
        let (i_float, m_float, n_float) = match __sifr_try_res {
            Ok(__sifr_try_bindings) => __sifr_try_bindings,
            Err(__sifr_try_err) => {
                let error = __sifr_try_err.clone();
                return Err(
                    __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                        error.message.clone(),
                    ),
                );
            }
        };
        let idx_f: f64 = (i_float * m_float) / n_float;
        let mut idx: SifrInt = SifrInt::from_i64(0);
        let __sifr_try_res: Result<(), ValueError> = (|| {
            let converted_idx: SifrInt = SifrInt::from_f64_trunc(idx_f)
                .ok_or_else(|| ValueError {
                    message: "cannot convert non-finite float to int".to_string(),
                })?;
            idx = converted_idx;
            Ok(())
        })();
        if let Err(__sifr_try_err) = __sifr_try_res {
            let error = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                    error.message.clone(),
                ),
            );
        }
        let __sifr_try_res: Result<
            (f64,),
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
        > = (|| {
            let idx_float: f64 = _float_int((idx).clone())?;
            Ok((idx_float,))
        })();
        let (idx_float,) = match __sifr_try_res {
            Ok(__sifr_try_bindings) => __sifr_try_bindings,
            Err(__sifr_try_err) => {
                let error = __sifr_try_err.clone();
                return Err(
                    __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                        error.message.clone(),
                    ),
                );
            }
        };
        let frac: f64 = idx_f - idx_float;
        if (&idx >= &m) {
            idx = &m - &SifrInt::from_i64(1);
        }
        if (&idx < &SifrInt::from_i64(0)) {
            idx = SifrInt::from_i64(0);
        }
        let lo: Option<f64> = {
            let __sifr_checked_read_collection = &sorted_data;
            let __sifr_checked_read_index = idx.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        let mut lo_val: f64 = 0.0_f64;
        if let Some(lo) = lo {
            lo_val = lo;
        }
        if (frac > (0.0_f64)) {
            let hi_idx: SifrInt = &idx + &SifrInt::from_i64(1);
            if (&hi_idx < &m) {
                let hi: Option<f64> = {
                    let __sifr_checked_read_collection = &sorted_data;
                    let __sifr_checked_read_index = hi_idx.clone();
                    let __sifr_checked_read_normalized = __sifr_checked_read_index
                        .normalize_index_or_len(__sifr_checked_read_collection.len());
                    __sifr_checked_read_collection
                        .get(__sifr_checked_read_normalized)
                        .cloned()
                };
                if let Some(hi) = hi {
                    lo_val += frac * (hi - lo_val);
                }
            }
        }
        result.push(lo_val);
        i = &i + &SifrInt::from_i64(1);
    }
    Ok(result)
}
fn covariance(
    x: &Vec<f64>,
    y: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: SifrInt = SifrInt::from(x.len());
    if (&n < &SifrInt::from_i64(2)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "covariance requires at least two data points".to_string(),
            ),
        );
    }
    if (&SifrInt::from(y.len()) != &n) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "covariance: x and y must have the same length".to_string(),
            ),
        );
    }
    let __sifr_try_res: Result<
        (f64, f64),
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
    > = (|| {
        let mx: f64 = _divide_by_int(_sum(x), (n).clone())?;
        let my: f64 = _divide_by_int(_sum(y), (n).clone())?;
        Ok((mx, my))
    })();
    let (mx, my) = match __sifr_try_res {
        Ok(__sifr_try_bindings) => __sifr_try_bindings,
        Err(__sifr_try_err) => {
            let error = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                    error.message.clone(),
                ),
            );
        }
    };
    let mut total: f64 = 0.0_f64;
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &n) {
        let xi: Option<f64> = {
            let __sifr_checked_read_collection = &x;
            let __sifr_checked_read_index = i.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        let yi: Option<f64> = {
            let __sifr_checked_read_collection = &y;
            let __sifr_checked_read_index = i.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(xi) = xi {
            if let Some(yi) = yi {
                total += (xi - mx) * (yi - my);
            }
        }
        i = &i + &SifrInt::from_i64(1);
    }
    _divide_by_int(total, &n - &SifrInt::from_i64(1))
}
fn correlation(
    x: &Vec<f64>,
    y: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: SifrInt = SifrInt::from(x.len());
    if (&n < &SifrInt::from_i64(2)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "correlation requires at least two data points".to_string(),
            ),
        );
    }
    if (&SifrInt::from(y.len()) != &n) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "correlation: x and y must have the same length".to_string(),
            ),
        );
    }
    let __sifr_try_res: Result<
        (f64, f64),
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
    > = (|| {
        let mx: f64 = _divide_by_int(_sum(x), (n).clone())?;
        let my: f64 = _divide_by_int(_sum(y), (n).clone())?;
        Ok((mx, my))
    })();
    let (mx, my) = match __sifr_try_res {
        Ok(__sifr_try_bindings) => __sifr_try_bindings,
        Err(__sifr_try_err) => {
            let error = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                    error.message.clone(),
                ),
            );
        }
    };
    let mut cov_num: f64 = 0.0_f64;
    let mut sx_num: f64 = 0.0_f64;
    let mut sy_num: f64 = 0.0_f64;
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &n) {
        let xi: Option<f64> = {
            let __sifr_checked_read_collection = &x;
            let __sifr_checked_read_index = i.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        let yi: Option<f64> = {
            let __sifr_checked_read_collection = &y;
            let __sifr_checked_read_index = i.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(xi) = xi {
            if let Some(yi) = yi {
                cov_num += (xi - mx) * (yi - my);
                sx_num += (xi - mx) * (xi - mx);
                sy_num += (yi - my) * (yi - my);
            }
        }
        i = &i + &SifrInt::from_i64(1);
    }
    let __sifr_try_res: Result<
        (f64, f64),
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
    > = (|| {
        let sx_variance: f64 = _divide_by_int(sx_num, &n - &SifrInt::from_i64(1))?;
        let sy_variance: f64 = _divide_by_int(sy_num, &n - &SifrInt::from_i64(1))?;
        let sx: f64 = sqrt(sx_variance);
        let sy: f64 = sqrt(sy_variance);
        Ok((sx, sy))
    })();
    let (sx, sy) = match __sifr_try_res {
        Ok(__sifr_try_bindings) => __sifr_try_bindings,
        Err(__sifr_try_err) => {
            let error = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                    error.message.clone(),
                ),
            );
        }
    };
    if (sx == (0.0_f64)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "correlation: x has zero variance".to_string(),
            ),
        );
    }
    if (sy == (0.0_f64)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "correlation: y has zero variance".to_string(),
            ),
        );
    }
    let __sifr_try_res: Result<
        Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError>,
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
    > = (|| {
        let covariance_value: f64 = _divide_by_int(cov_num, &n - &SifrInt::from_i64(1))?;
        Ok(Ok(covariance_value / (sx * sy)))
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
fn linear_regression(
    x: &Vec<f64>,
    y: &Vec<f64>,
) -> Result<Vec<f64>, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: SifrInt = SifrInt::from(x.len());
    if (&n < &SifrInt::from_i64(2)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "linear_regression requires at least two data points".to_string(),
            ),
        );
    }
    if (&SifrInt::from(y.len()) != &n) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "linear_regression: x and y must have the same length".to_string(),
            ),
        );
    }
    let __sifr_try_res: Result<
        (f64, f64),
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
    > = (|| {
        let mx: f64 = _divide_by_int(_sum(x), (n).clone())?;
        let my: f64 = _divide_by_int(_sum(y), (n).clone())?;
        Ok((mx, my))
    })();
    let (mx, my) = match __sifr_try_res {
        Ok(__sifr_try_bindings) => __sifr_try_bindings,
        Err(__sifr_try_err) => {
            let error = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                    error.message.clone(),
                ),
            );
        }
    };
    let mut num: f64 = 0.0_f64;
    let mut den: f64 = 0.0_f64;
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &n) {
        let xi: Option<f64> = {
            let __sifr_checked_read_collection = &x;
            let __sifr_checked_read_index = i.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        let yi: Option<f64> = {
            let __sifr_checked_read_collection = &y;
            let __sifr_checked_read_index = i.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(xi) = xi {
            if let Some(yi) = yi {
                num += (xi - mx) * (yi - my);
                den += (xi - mx) * (xi - mx);
            }
        }
        i = &i + &SifrInt::from_i64(1);
    }
    if (den == (0.0_f64)) {
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
    assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &SifrInt::from(actual.len()) {
        assert!(
            ({ let __sifr_condition_list = & actual; let __sifr_condition_index = i
            .clone(); let __sifr_condition_normalized = __sifr_condition_index
            .normalize_index_or_len(__sifr_condition_list.len()); __sifr_condition_list
            .get(__sifr_condition_normalized).cloned() }) == ({ let __sifr_condition_list
            = & expected; let __sifr_condition_index = i.clone(); let
            __sifr_condition_normalized = __sifr_condition_index
            .normalize_index_or_len(__sifr_condition_list.len()); __sifr_condition_list
            .get(__sifr_condition_normalized).cloned() })
        );
        i = &i + &SifrInt::from_i64(1);
    }
}
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &SifrInt::from(actual.len()) {
        assert!(
            ({ let __sifr_condition_list = & actual; let __sifr_condition_index = i
            .clone(); let __sifr_condition_normalized = __sifr_condition_index
            .normalize_index_or_len(__sifr_condition_list.len()); __sifr_condition_list
            .get(__sifr_condition_normalized).copied() }) == ({ let __sifr_condition_list
            = & expected; let __sifr_condition_index = i.clone(); let
            __sifr_condition_normalized = __sifr_condition_index
            .normalize_index_or_len(__sifr_condition_list.len()); __sifr_condition_list
            .get(__sifr_condition_normalized).copied() })
        );
        i = &i + &SifrInt::from_i64(1);
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl ::std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for ValueError {
}

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

impl ::std::error::Error for FloatOverflowError {
}

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

impl ::std::error::Error for FloatPrecisionLossError {
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
    let mut mode_v: SifrInt = SifrInt::from_i64(0);
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (|| {
    let out_mode: SifrInt = mode(&vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(3), SifrInt::from_i64(3)])?;
    mode_v = out_mode;
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        mode_ok = false;
    }
    actual.push(format!("{}", mode_ok && (&mode_v == &SifrInt::from_i64(3))));
    let mut mm_ok: bool = true;
    let mut mm_v: Vec<SifrInt> = vec![];
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (|| {
    let out_mm: Vec<SifrInt> = multimode(&vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(3)])?;
    mm_v = out_mm;
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        mm_ok = false;
    }
    actual.push(format!("{}", mm_ok && (&SifrInt::from(mm_v.len()) == &SifrInt::from_i64(2))));
    let mut q_ok: bool = true;
    let mut q_v: Vec<f64> = vec![];
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (|| {
    let out_q: Vec<f64> = quantiles(&vec![1.0_f64, 2.0_f64, 3.0_f64, 4.0_f64, 5.0_f64, 6.0_f64, 7.0_f64, 8.0_f64], SifrInt::from_i64(4))?;
    q_v = out_q;
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        q_ok = false;
    }
    actual.push(format!("{}", q_ok && (&SifrInt::from(q_v.len()) == &SifrInt::from_i64(3))));
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
    let __sifr_checked_read_collection = &lr_v;
    let __sifr_checked_read_index = SifrInt::from_i64(0);
    let __sifr_checked_read_normalized = __sifr_checked_read_index.normalize_index_or_len(__sifr_checked_read_collection.len());
    __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
};
        let lr_intercept: Option<f64> = {
    let __sifr_checked_read_collection = &lr_v;
    let __sifr_checked_read_index = SifrInt::from_i64(1);
    let __sifr_checked_read_normalized = __sifr_checked_read_index.normalize_index_or_len(__sifr_checked_read_collection.len());
    __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
};
        if let Some(lr_slope) = lr_slope {
            lr_slope_ok = near(lr_slope, 2.0_f64, 0.0001_f64);
        }
        if let Some(lr_intercept) = lr_intercept {
            lr_intercept_ok = near(lr_intercept, 0.0_f64, 0.0001_f64);
        }
    }
    actual.push(format!("{}", ((lr_ok && (&SifrInt::from(lr_v.len()) == &SifrInt::from_i64(2))) && lr_slope_ok) && lr_intercept_ok));
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
