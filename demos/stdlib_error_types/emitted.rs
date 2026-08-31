// src/main.rs
mod __sifr_project_nominals {
    pub use ::sifr_runtime::SifrInt;
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
    pub fn floor(x: f64) -> SifrInt {
        ::sifr_stdlib::math::floor(x).into_sifr_int()
    }
    pub fn ceil(x: f64) -> SifrInt {
        ::sifr_stdlib::math::ceil(x).into_sifr_int()
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
    pub fn round_val(x: f64) -> SifrInt {
        ::sifr_stdlib::math::round_val(x).into_sifr_int()
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
    pub fn trunc(x: f64) -> SifrInt {
        ::sifr_stdlib::math::trunc(x).into_sifr_int()
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
    pub fn isqrt(n: SifrInt) -> SifrInt {
        ::sifr_stdlib::math::isqrt(::sifr_runtime::interop::SifrIntBridge::from(n))
            .into_sifr_int()
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
    pub fn ldexp(m: f64, e: SifrInt) -> f64 {
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
    pub fn factorial(n: SifrInt) -> SifrInt {
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
    pub fn gcd(a: SifrInt, b: SifrInt) -> SifrInt {
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
    pub fn lcm(a: SifrInt, b: SifrInt) -> SifrInt {
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
    pub fn comb(n: SifrInt, k: SifrInt) -> SifrInt {
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
    pub fn perm(n: SifrInt, k: SifrInt) -> SifrInt {
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
    pub fn prod(data: &[SifrInt]) -> SifrInt {
        let mut result: SifrInt = SifrInt::from_i64(1);
        for val in data.iter().cloned() {
            result = &result * &val;
        }
        result.clone()
    }
    pub fn _copy_float_list(data: &[f64]) -> Vec<f64> {
        let mut out: Vec<f64> = vec![];
        for value in data.iter().copied() {
            out.push(value);
        }
        out
    }
    pub fn dist(p: &[f64], q: &[f64]) -> f64 {
        dist_impl(_copy_float_list(p), _copy_float_list(q))
    }
    pub fn fsum(data: &[f64]) -> f64 {
        fsum_impl(_copy_float_list(data))
    }
    pub fn sumprod(p: &[f64], q: &[f64]) -> f64 {
        sumprod_impl(_copy_float_list(p), _copy_float_list(q))
    }
    pub fn frexp_mantissa(x: f64) -> f64 {
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
    pub fn frexp_exponent(x: f64) -> SifrInt {
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
    pub fn modf_fractional(x: f64) -> f64 {
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
    pub fn modf_integral(x: f64) -> f64 {
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct FloatOverflowError {
        pub message: String,
    }
    impl FloatOverflowError {
        pub fn new(message: String) -> Self {
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
    pub struct FloatPrecisionLossError {
        pub message: String,
    }
    impl FloatPrecisionLossError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for FloatPrecisionLossError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for FloatPrecisionLossError {}
    impl From<ParseError> for Error {
        fn from(err: ParseError) -> Self {
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
}
pub use __sifr_project_nominals::Error;
pub use __sifr_project_nominals::FloatOverflowError;
pub use __sifr_project_nominals::FloatPrecisionLossError;
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
        __SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
            crate::__sifr_project_nominals::FloatOverflowError,
        ),
        __SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
            crate::__sifr_project_nominals::FloatPrecisionLossError,
        ),
    }
    impl From<crate::__sifr_project_nominals::FloatOverflowError>
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
        fn from(value: crate::__sifr_project_nominals::FloatOverflowError) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                value,
            )
        }
    }
    impl From<crate::__sifr_project_nominals::FloatPrecisionLossError>
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
        fn from(value: crate::__sifr_project_nominals::FloatPrecisionLossError) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
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
}
pub use __sifr_project_unions::__SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a044_x3a5_x3aclass31_x3asifr_x2estatistics_x2eStatisticsError1_x3a0;
pub use __sifr_project_unions::__SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0;
use ::sifr_runtime::SifrInt;
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
fn compute_mean(
    data: &[f64],
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    if (&SifrInt::from(data.len()) == &SifrInt::from_i64(0)) {
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
    let __sifr_try_res: Result<
        Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError>,
        __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0,
    > = (|| {
        let count: f64 = SifrInt::from(data.len())
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
        Ok(Ok(total / count))
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
fn topo_sort(
    has_cycle: bool,
) -> Result<SifrInt, __SifrStdlib_sifr_x2egraphlib_x2eCycleError> {
    if has_cycle {
        return Err(
            __SifrStdlib_sifr_x2egraphlib_x2eCycleError::new(
                "graph contains a cycle".to_string(),
            ),
        );
    }
    Ok(SifrInt::from_i64(42))
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
        let order: SifrInt = topo_sort(false)?;
        println!("topo sort result = {}", order);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("cycle error: {}", e.message.clone());
    }
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2egraphlib_x2eCycleError> = (|| {
        let order2: SifrInt = topo_sort(true)?;
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
        let val: SifrInt = (SifrInt::parse_decimal(
                &("not_a_number".to_string()),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
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
