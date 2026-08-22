// src/main.rs
mod __sifr_project_nominals {
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2egraphlib_x2eCycleError {
        pub message: String,
    }
    impl __SifrStdlib_sifr_x2egraphlib_x2eCycleError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl __SifrStdlib_sifr_x2egraphlib_x2eCycleError {}
    impl ::std::fmt::Debug for __SifrStdlib_sifr_x2egraphlib_x2eCycleError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            f.debug_struct("CycleError").field("message", &self.message).finish()
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2egraphlib_x2eCycleError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }
    impl ::std::error::Error for __SifrStdlib_sifr_x2egraphlib_x2eCycleError {}
    pub const PI: f64 = 3.141592653589793_f64;
    pub const E: f64 = 2.718281828459045_f64;
    pub const TAU: f64 = 6.283185307179586_f64;
    pub const INF: f64 = f64::INFINITY;
    pub const NAN: f64 = f64::NAN;
    pub fn sqrt(x: f64) -> f64 {
        ::sifr_stdlib::math::sqrt(x)
    }
    pub fn floor(x: f64) -> i64 {
        ::sifr_stdlib::math::floor(x).to_i64_saturating()
    }
    pub fn ceil(x: f64) -> i64 {
        ::sifr_stdlib::math::ceil(x).to_i64_saturating()
    }
    pub fn log(x: f64) -> f64 {
        ::sifr_stdlib::math::log(x)
    }
    pub fn cbrt(x: f64) -> f64 {
        ::sifr_stdlib::math::cbrt(x)
    }
    pub fn sin(x: f64) -> f64 {
        ::sifr_stdlib::math::sin(x)
    }
    pub fn cos(x: f64) -> f64 {
        ::sifr_stdlib::math::cos(x)
    }
    pub fn tan(x: f64) -> f64 {
        ::sifr_stdlib::math::tan(x)
    }
    pub fn pow_val(x: f64, y: f64) -> f64 {
        ::sifr_stdlib::math::pow_val(x, y)
    }
    pub fn min_val(a: f64, b: f64) -> f64 {
        ::sifr_stdlib::math::min_val(a, b)
    }
    pub fn max_val(a: f64, b: f64) -> f64 {
        ::sifr_stdlib::math::max_val(a, b)
    }
    pub fn round_val(x: f64) -> i64 {
        ::sifr_stdlib::math::round_val(x).to_i64_saturating()
    }
    pub fn asin(x: f64) -> f64 {
        ::sifr_stdlib::math::asin(x)
    }
    pub fn acos(x: f64) -> f64 {
        ::sifr_stdlib::math::acos(x)
    }
    pub fn atan(x: f64) -> f64 {
        ::sifr_stdlib::math::atan(x)
    }
    pub fn atan2(y: f64, x: f64) -> f64 {
        ::sifr_stdlib::math::atan2(y, x)
    }
    pub fn sinh(x: f64) -> f64 {
        ::sifr_stdlib::math::sinh(x)
    }
    pub fn cosh(x: f64) -> f64 {
        ::sifr_stdlib::math::cosh(x)
    }
    pub fn tanh(x: f64) -> f64 {
        ::sifr_stdlib::math::tanh(x)
    }
    pub fn log10(x: f64) -> f64 {
        ::sifr_stdlib::math::log10(x)
    }
    pub fn log2(x: f64) -> f64 {
        ::sifr_stdlib::math::log2(x)
    }
    pub fn exp2(x: f64) -> f64 {
        ::sifr_stdlib::math::exp2(x)
    }
    pub fn degrees(x: f64) -> f64 {
        ::sifr_stdlib::math::degrees(x)
    }
    pub fn radians(x: f64) -> f64 {
        ::sifr_stdlib::math::radians(x)
    }
    pub fn isnan(x: f64) -> bool {
        ::sifr_stdlib::math::isnan(x)
    }
    pub fn isinf(x: f64) -> bool {
        ::sifr_stdlib::math::isinf(x)
    }
    pub fn trunc(x: f64) -> i64 {
        ::sifr_stdlib::math::trunc(x).to_i64_saturating()
    }
    pub fn copysign(x: f64, y: f64) -> f64 {
        ::sifr_stdlib::math::copysign(x, y)
    }
    pub fn signbit(x: f64) -> bool {
        ::sifr_stdlib::math::signbit(x)
    }
    pub fn fmod(x: f64, y: f64) -> f64 {
        ::sifr_stdlib::math::fmod(x, y)
    }
    pub fn remainder(x: f64, y: f64) -> f64 {
        ::sifr_stdlib::math::remainder(x, y)
    }
    pub fn hypot(x: f64, y: f64) -> f64 {
        ::sifr_stdlib::math::hypot(x, y)
    }
    pub fn fma(x: f64, y: f64, z: f64) -> f64 {
        ::sifr_stdlib::math::fma(x, y, z)
    }
    pub fn fmax(x: f64, y: f64) -> f64 {
        ::sifr_stdlib::math::fmax(x, y)
    }
    pub fn fmin(x: f64, y: f64) -> f64 {
        ::sifr_stdlib::math::fmin(x, y)
    }
    pub fn exp(x: f64) -> f64 {
        ::sifr_stdlib::math::exp(x)
    }
    pub fn expm1(x: f64) -> f64 {
        ::sifr_stdlib::math::expm1(x)
    }
    pub fn log1p(x: f64) -> f64 {
        ::sifr_stdlib::math::log1p(x)
    }
    pub fn fabs(x: f64) -> f64 {
        ::sifr_stdlib::math::fabs(x)
    }
    pub fn isfinite(x: f64) -> bool {
        ::sifr_stdlib::math::isfinite(x)
    }
    pub fn isnormal(x: f64) -> bool {
        ::sifr_stdlib::math::isnormal(x)
    }
    pub fn issubnormal(x: f64) -> bool {
        ::sifr_stdlib::math::issubnormal(x)
    }
    pub fn acosh(x: f64) -> f64 {
        ::sifr_stdlib::math::acosh(x)
    }
    pub fn asinh(x: f64) -> f64 {
        ::sifr_stdlib::math::asinh(x)
    }
    pub fn atanh(x: f64) -> f64 {
        ::sifr_stdlib::math::atanh(x)
    }
    pub fn isqrt(n: i64) -> i64 {
        ::sifr_stdlib::math::isqrt(::sifr_runtime::interop::SifrIntBridge::from(n))
            .to_i64_saturating()
    }
    pub fn dist_impl(p: Vec<f64>, q: Vec<f64>) -> f64 {
        ::sifr_stdlib::math::dist(p, q)
    }
    pub fn fsum_impl(data: Vec<f64>) -> f64 {
        ::sifr_stdlib::math::fsum(data)
    }
    pub fn sumprod_impl(p: Vec<f64>, q: Vec<f64>) -> f64 {
        ::sifr_stdlib::math::sumprod(p, q)
    }
    pub fn erf(x: f64) -> f64 {
        ::sifr_stdlib::math::erf(x)
    }
    pub fn erfc(x: f64) -> f64 {
        ::sifr_stdlib::math::erfc(x)
    }
    pub fn gamma(x: f64) -> f64 {
        ::sifr_stdlib::math::gamma(x)
    }
    pub fn lgamma(x: f64) -> f64 {
        ::sifr_stdlib::math::lgamma(x)
    }
    pub fn frexp(x: f64) -> Vec<f64> {
        ::sifr_stdlib::math::frexp(x)
    }
    pub fn ldexp(m: f64, e: i64) -> f64 {
        ::sifr_stdlib::math::ldexp(m, ::sifr_runtime::interop::SifrIntBridge::from(e))
    }
    pub fn modf(x: f64) -> Vec<f64> {
        ::sifr_stdlib::math::modf(x)
    }
    pub fn nextafter(x: f64, y: f64) -> f64 {
        ::sifr_stdlib::math::nextafter(x, y)
    }
    pub fn ulp(x: f64) -> f64 {
        ::sifr_stdlib::math::ulp(x)
    }
    pub fn factorial(n: i64) -> i64 {
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
    pub fn gcd(a: i64, b: i64) -> i64 {
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
    pub fn lcm(a: i64, b: i64) -> i64 {
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
    pub fn comb(n: i64, k: i64) -> i64 {
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
    pub fn perm(n: i64, k: i64) -> i64 {
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
    pub fn log_base(x: f64, base: f64) -> f64 {
        log(x) / log(base)
    }
    pub fn isclose(a: f64, b: f64, rel_tol: f64, abs_tol: f64) -> bool {
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
    pub fn prod(data: &Vec<i64>) -> i64 {
        let mut result: i64 = 1_i64;
        for val in data.iter().copied() {
            result *= val;
        }
        result
    }
    pub fn _copy_float_list(data: &Vec<f64>) -> Vec<f64> {
        let mut out: Vec<f64> = vec![];
        for value in data.iter().copied() {
            out.push(value);
        }
        out
    }
    pub fn dist(p: &Vec<f64>, q: &Vec<f64>) -> f64 {
        dist_impl(_copy_float_list(p), _copy_float_list(q))
    }
    pub fn fsum(data: &Vec<f64>) -> f64 {
        fsum_impl(_copy_float_list(data))
    }
    pub fn sumprod(p: &Vec<f64>, q: &Vec<f64>) -> f64 {
        sumprod_impl(_copy_float_list(p), _copy_float_list(q))
    }
    pub fn frexp_mantissa(x: f64) -> f64 {
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
    pub fn frexp_exponent(x: f64) -> i64 {
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
    pub fn modf_fractional(x: f64) -> f64 {
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
    pub fn modf_integral(x: f64) -> f64 {
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
    pub fn pow(x: f64, y: f64) -> f64 {
        pow_val(x, y)
    }
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {
        pub message: String,
    }
    impl __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {
        pub fn new(message: String) -> Self {
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Error {
        pub message: String,
    }
    impl Error {
        pub fn new(message: String) -> Self {
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
    pub struct ParseError {
        pub message: String,
    }
    impl ParseError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for ParseError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ParseError {}
    impl From<ParseError> for Error {
        fn from(err: ParseError) -> Self {
            Self::new(err.message)
        }
    }
}
pub use __sifr_project_nominals::Error;
pub use __sifr_project_nominals::ParseError;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2egraphlib_x2eCycleError;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2estatistics_x2eStatisticsError;

mod __sifr_project_unions {
    #[derive(Debug, Clone)]
    pub enum __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a044_x3a5_x3aclass31_x3asifr_x2estatistics_x2eStatisticsError1_x3a0 {
        __SifrUnionVariant_5_x3aclass10_x3aParseError1_x3a0(
            crate::__sifr_project_nominals::ParseError,
        ),
        __SifrUnionVariant_5_x3aclass31_x3asifr_x2estatistics_x2eStatisticsError1_x3a0(
            crate::__sifr_project_nominals::__SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
        ),
    }
    impl From<crate::__sifr_project_nominals::ParseError>
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a044_x3a5_x3aclass31_x3asifr_x2estatistics_x2eStatisticsError1_x3a0 {
        fn from(value: crate::__sifr_project_nominals::ParseError) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a044_x3a5_x3aclass31_x3asifr_x2estatistics_x2eStatisticsError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aParseError1_x3a0(
                value,
            )
        }
    }
    impl From<
        crate::__sifr_project_nominals::__SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
    >
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a044_x3a5_x3aclass31_x3asifr_x2estatistics_x2eStatisticsError1_x3a0 {
        fn from(
            value: crate::__sifr_project_nominals::__SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
        ) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a044_x3a5_x3aclass31_x3asifr_x2estatistics_x2eStatisticsError1_x3a0::__SifrUnionVariant_5_x3aclass31_x3asifr_x2estatistics_x2eStatisticsError1_x3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a044_x3a5_x3aclass31_x3asifr_x2estatistics_x2eStatisticsError1_x3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a044_x3a5_x3aclass31_x3asifr_x2estatistics_x2eStatisticsError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aParseError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a044_x3a5_x3aclass31_x3asifr_x2estatistics_x2eStatisticsError1_x3a0::__SifrUnionVariant_5_x3aclass31_x3asifr_x2estatistics_x2eStatisticsError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
            }
        }
    }
}
pub use __sifr_project_unions::__SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a044_x3a5_x3aclass31_x3asifr_x2estatistics_x2eStatisticsError1_x3a0;
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
fn compute_mean(
    data: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    if ((data.len() as i64) == (0_i64)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "cannot compute mean of empty dataset".to_string(),
            ),
        );
    }
    let mut total: f64 = 0.0_f64;
    for val in data.iter().copied() {
        total += val;
    }
    Ok(total / ((data.len() as i64) as f64))
}
fn topo_sort(
    has_cycle: bool,
) -> Result<i64, __SifrStdlib_sifr_x2egraphlib_x2eCycleError> {
    if has_cycle {
        return Err(
            __SifrStdlib_sifr_x2egraphlib_x2eCycleError::new(
                "graph contains a cycle".to_string(),
            ),
        );
    }
    Ok(42_i64)
}
fn main() {
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (||
    {
        let mean: f64 = compute_mean(&vec![1.0_f64, 2.0_f64, 3.0_f64])?;
        println!("mean = {}", mean);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("stats error: {}", e.message.clone());
    }
    let empty: Vec<f64> = vec![];
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (||
    {
        let mean2: f64 = compute_mean(&empty)?;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("caught StatisticsError: {}", e.message.clone());
    }
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2egraphlib_x2eCycleError> = (|| {
        let order: i64 = topo_sort(false)?;
        println!("topo sort result = {}", order);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("cycle error: {}", e.message.clone());
    }
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2egraphlib_x2eCycleError> = (|| {
        let order2: i64 = topo_sort(true)?;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("caught CycleError: {}", e.message.clone());
    }
    let __sifr_try_res: Result<
        (),
        __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a044_x3a5_x3aclass31_x3asifr_x2estatistics_x2eStatisticsError1_x3a0,
    > = (|| {
        let val: i64 = (("not_a_number".to_string())
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            }))
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a044_x3a5_x3aclass31_x3asifr_x2estatistics_x2eStatisticsError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aParseError1_x3a0(
                __e,
            ))?;
        let mean3: f64 = (compute_mean(&empty))
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a044_x3a5_x3aclass31_x3asifr_x2estatistics_x2eStatisticsError1_x3a0::__SifrUnionVariant_5_x3aclass31_x3asifr_x2estatistics_x2eStatisticsError1_x3a0(
                __e,
            ))?;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        match __sifr_try_err {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a044_x3a5_x3aclass31_x3asifr_x2estatistics_x2eStatisticsError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aParseError1_x3a0(
                __sifr_try_variant_error,
            ) => {
                let e = __sifr_try_variant_error.clone();
                println!("caught ParseError: {}", e.message.clone());
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a044_x3a5_x3aclass31_x3asifr_x2estatistics_x2eStatisticsError1_x3a0::__SifrUnionVariant_5_x3aclass31_x3asifr_x2estatistics_x2eStatisticsError1_x3a0(
                __sifr_try_variant_error,
            ) => {
                let e = __sifr_try_variant_error.clone();
                println!("caught StatisticsError: {}", e.message.clone());
            }
        }
    }
    println!("all module-specific error types work correctly");
}
