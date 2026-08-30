// src/main.rs
mod __sifr_project_nominals {
    pub use ::sifr_runtime::SifrInt;
    pub fn random_int(min: SifrInt, max: SifrInt) -> SifrInt {
        ::sifr_stdlib::random::random_int(
                ::sifr_runtime::interop::SifrIntBridge::from(min),
                ::sifr_runtime::interop::SifrIntBridge::from(max),
            )
            .into_sifr_int()
    }
    pub fn random_float() -> f64 {
        ::sifr_stdlib::random::random_float()
    }
    pub fn random_word_to_unit_float(value: SifrInt) -> f64 {
        ::sifr_stdlib::random::random_word_to_unit_float(
            ::sifr_runtime::interop::SifrIntBridge::from(value),
        )
    }
    pub fn random_seed() -> SifrInt {
        ::sifr_stdlib::random::random_seed().into_sifr_int()
    }
    pub fn random_uniform(min: f64, max: f64) -> f64 {
        ::sifr_stdlib::random::random_uniform(min, max)
    }
    pub fn random_randrange(
        start: SifrInt,
        stop: SifrInt,
        step: SifrInt,
    ) -> Result<SifrInt, ValueError> {
        ::sifr_stdlib::random::random_randrange(
                ::sifr_runtime::interop::SifrIntBridge::from(start),
                ::sifr_runtime::interop::SifrIntBridge::from(stop),
                ::sifr_runtime::interop::SifrIntBridge::from(step),
            )
            .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
            .map_err(|__sifr_bridge_error| ValueError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn random_gauss(mu: f64, sigma: f64) -> f64 {
        ::sifr_stdlib::random::random_gauss(mu, sigma)
    }
    pub fn random_module_state_words() -> Vec<SifrInt> {
        ::sifr_stdlib::random::random_module_state_words()
            .into_iter()
            .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
            .collect()
    }
    pub fn random_module_state_index() -> SifrInt {
        ::sifr_stdlib::random::random_module_state_index().into_sifr_int()
    }
    pub fn random_module_state_gauss_next() -> Option<f64> {
        ::sifr_stdlib::random::random_module_state_gauss_next()
    }
    pub fn random_module_set_state(
        words: &Vec<SifrInt>,
        index: SifrInt,
        gauss_next: Option<f64>,
    ) -> Result<(), ValueError> {
        ::sifr_stdlib::random::random_module_set_state(
                &words
                    .iter()
                    .cloned()
                    .map(::sifr_runtime::interop::SifrIntBridge::from)
                    .collect::<Vec<_>>(),
                ::sifr_runtime::interop::SifrIntBridge::from(index),
                gauss_next.map(|__sifr_bridge_item_0| __sifr_bridge_item_0),
            )
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ValueError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn base64_encode(s: &String) -> String {
        ::sifr_stdlib::base64::base64_encode(s)
    }
    pub fn base64_encode_bytes(data: &Vec<u8>) -> Vec<u8> {
        ::sifr_stdlib::base64::base64_encode_bytes(data)
    }
    pub fn base64_decode(s: &String) -> Result<String, ParseError> {
        ::sifr_stdlib::base64::base64_decode(s)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ParseError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn base64_decode_bytes(data: &Vec<u8>) -> Result<Vec<u8>, ParseError> {
        ::sifr_stdlib::base64::base64_decode_bytes(data)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ParseError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn base64_encode_opts(
        s: &String,
        altchars: &String,
        wrapcol: SifrInt,
    ) -> Result<String, ParseError> {
        ::sifr_stdlib::base64::base64_encode_opts(
                s,
                altchars,
                ::sifr_runtime::interop::SifrIntBridge::from(wrapcol),
            )
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ParseError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn base64_decode_opts(
        s: &String,
        altchars: &String,
        validate: bool,
        ignorechars: &String,
    ) -> Result<String, ParseError> {
        ::sifr_stdlib::base64::base64_decode_opts(s, altchars, validate, ignorechars)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ParseError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn urlsafe_b64encode(s: &String) -> String {
        ::sifr_stdlib::base64::urlsafe_b64encode(s)
    }
    pub fn urlsafe_b64encode_bytes(data: &Vec<u8>) -> Vec<u8> {
        ::sifr_stdlib::base64::urlsafe_b64encode_bytes(data)
    }
    pub fn urlsafe_b64decode(s: &String) -> Result<String, ParseError> {
        ::sifr_stdlib::base64::urlsafe_b64decode(s)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ParseError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn urlsafe_b64decode_bytes(data: &Vec<u8>) -> Result<Vec<u8>, ParseError> {
        ::sifr_stdlib::base64::urlsafe_b64decode_bytes(data)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ParseError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn b32encode(s: &String) -> String {
        ::sifr_stdlib::base64::b32encode(s)
    }
    pub fn b32decode(s: &String) -> Result<String, ParseError> {
        ::sifr_stdlib::base64::b32decode(s)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ParseError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn b32hexencode(s: &String) -> String {
        ::sifr_stdlib::base64::b32hexencode(s)
    }
    pub fn b32hexdecode(s: &String) -> Result<String, ParseError> {
        ::sifr_stdlib::base64::b32hexdecode(s)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ParseError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn sha256_bytes(data: &Vec<u8>) -> Vec<u8> {
        ::sifr_stdlib::hash::sha256_bytes(data)
    }
    pub fn md5_bytes(data: &Vec<u8>) -> Vec<u8> {
        ::sifr_stdlib::hash::md5_bytes(data)
    }
    pub fn sha1_bytes(data: &Vec<u8>) -> Vec<u8> {
        ::sifr_stdlib::hash::sha1_bytes(data)
    }
    pub fn sha224_bytes(data: &Vec<u8>) -> Vec<u8> {
        ::sifr_stdlib::hash::sha224_bytes(data)
    }
    pub fn sha384_bytes(data: &Vec<u8>) -> Vec<u8> {
        ::sifr_stdlib::hash::sha384_bytes(data)
    }
    pub fn sha512_bytes(data: &Vec<u8>) -> Vec<u8> {
        ::sifr_stdlib::hash::sha512_bytes(data)
    }
    pub fn blake2b_bytes(data: &Vec<u8>) -> Vec<u8> {
        ::sifr_stdlib::hash::blake2b_bytes(data)
    }
    pub fn blake2s_bytes(data: &Vec<u8>) -> Vec<u8> {
        ::sifr_stdlib::hash::blake2s_bytes(data)
    }
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
        while &y != &SifrInt::from_i64(0) {
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
        while &i < &r {
            result = &result * &(&n - &i);
            let divisor: SifrInt = &i + &SifrInt::from_i64(1);
            if &divisor == &SifrInt::from_i64(0) {
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
    pub fn prod(data: &Vec<SifrInt>) -> SifrInt {
        let mut result: SifrInt = SifrInt::from_i64(1);
        for val in data.iter().cloned() {
            result = &result * &val;
        }
        result.clone()
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
            let __sifr_index_i = SifrInt::from_i64(0);
            let __sifr_index_norm = __sifr_index_i
                .normalize_index_or_len(__sifr_index_list.len());
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        let Some(m) = m else {
            return NAN;
        };
        m
    }
    pub fn frexp_exponent(x: f64) -> SifrInt {
        let parts: Vec<f64> = frexp(x);
        let exp_val: Option<f64> = {
            let __sifr_index_list = &parts;
            let __sifr_index_i = SifrInt::from_i64(1);
            let __sifr_index_norm = __sifr_index_i
                .normalize_index_or_len(__sifr_index_list.len());
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        let Some(exp_val) = exp_val else {
            return SifrInt::from_i64(0);
        };
        trunc(exp_val)
    }
    pub fn modf_fractional(x: f64) -> f64 {
        let parts: Vec<f64> = modf(x);
        let f: Option<f64> = {
            let __sifr_index_list = &parts;
            let __sifr_index_i = SifrInt::from_i64(0);
            let __sifr_index_norm = __sifr_index_i
                .normalize_index_or_len(__sifr_index_list.len());
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
            let __sifr_index_i = SifrInt::from_i64(1);
            let __sifr_index_norm = __sifr_index_i
                .normalize_index_or_len(__sifr_index_list.len());
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
    pub fn __const__MT_N() -> SifrInt {
        SifrInt::from_i64(624)
    }
    pub fn __const__MT_M() -> SifrInt {
        SifrInt::from_i64(397)
    }
    pub fn __const__MT_MATRIX_A() -> SifrInt {
        SifrInt::from_i64(2567483615)
    }
    pub fn __const__MT_UPPER_MASK() -> SifrInt {
        SifrInt::from_i64(2147483648)
    }
    pub fn __const__MT_LOWER_MASK() -> SifrInt {
        SifrInt::from_i64(2147483647)
    }
    pub fn __const__MT_F() -> SifrInt {
        SifrInt::from_i64(1812433253)
    }
    pub fn __const__MT_WORD_MASK() -> SifrInt {
        SifrInt::from_i64(4294967295)
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct __SifrStdlib_sifr_x2erandom_x2eRandomState {
        pub version: SifrInt,
        pub state_words: Vec<SifrInt>,
        pub index: SifrInt,
        pub gauss_next: Option<f64>,
    }
    impl __SifrStdlib_sifr_x2erandom_x2eRandomState {
        pub fn new(
            version: SifrInt,
            state_words: Vec<SifrInt>,
            index: SifrInt,
            gauss_next: Option<f64>,
        ) -> Self {
            let __sifr_field_init_0: SifrInt = version.clone();
            let __sifr_field_init_1: Vec<SifrInt> = state_words;
            let __sifr_field_init_2: SifrInt = index.clone();
            let __sifr_field_init_3: Option<f64> = gauss_next;
            Self {
                version: __sifr_field_init_0,
                state_words: __sifr_field_init_1,
                index: __sifr_field_init_2,
                gauss_next: __sifr_field_init_3,
            }
        }
    }
    impl __SifrStdlib_sifr_x2erandom_x2eRandomState {}
    #[derive(Debug, Clone, PartialEq)]
    pub struct __SifrStdlib_sifr_x2erandom_x2eRandom {
        pub _state_words: Vec<SifrInt>,
        pub _index: SifrInt,
        pub _gauss_next: Option<f64>,
    }
    impl __SifrStdlib_sifr_x2erandom_x2eRandom {
        pub fn new(seed_value: Option<SifrInt>) -> Self {
            let normalized_seed: SifrInt = _normalize_seed_input((seed_value).clone());
            let __sifr_field_init_0: Vec<SifrInt> = _seed_words_from_seed(
                (normalized_seed).clone(),
            );
            let __sifr_field_init_1: SifrInt = __const__MT_N().clone();
            let __sifr_field_init_2: Option<f64> = None;
            Self {
                _state_words: __sifr_field_init_0,
                _index: __sifr_field_init_1,
                _gauss_next: __sifr_field_init_2,
            }
        }
    }
    impl __SifrStdlib_sifr_x2erandom_x2eRandom {
        pub fn seed(&mut self, seed_value: &Option<SifrInt>) {
            let normalized_seed: SifrInt = _normalize_seed_input(
                (seed_value.clone()).clone(),
            );
            self._state_words = _seed_words_from_seed((normalized_seed).clone());
            self._index = __const__MT_N().clone();
            self._gauss_next = None;
        }
    }
    impl __SifrStdlib_sifr_x2erandom_x2eRandom {
        pub fn _twist(&mut self) {
            let mut i: SifrInt = SifrInt::from_i64(0);
            while &i < &__const__MT_N() {
                let y: SifrInt = &(&_state_word_at(&self._state_words, (i).clone())
                    & &__const__MT_UPPER_MASK())
                    + &(&_state_word_at(
                        &self._state_words,
                        (&i + &SifrInt::from_i64(1))
                            .floor_mod_known_nonzero(&__const__MT_N()),
                    ) & &__const__MT_LOWER_MASK());
                let mut x_a: SifrInt = y.floor_div_known_nonzero(&SifrInt::from_i64(2));
                if (&y.floor_mod_known_nonzero(&SifrInt::from_i64(2))
                    != &SifrInt::from_i64(0))
                {
                    x_a = &x_a ^ &__const__MT_MATRIX_A();
                }
                let new_word: SifrInt = &_state_word_at(
                    &self._state_words,
                    (&i + &__const__MT_M()).floor_mod_known_nonzero(&__const__MT_N()),
                ) ^ &x_a;
                {
                    let __idx_raw = i.clone();
                    let __idx_norm = __idx_raw
                        .normalize_index_or_len(self._state_words.len());
                    if let Some(__elem) = self._state_words.get_mut(__idx_norm) {
                        *__elem = &new_word & &__const__MT_WORD_MASK();
                    }
                }
                i = &i + &SifrInt::from_i64(1);
            }
            self._index = SifrInt::from_i64(0);
        }
    }
    impl __SifrStdlib_sifr_x2erandom_x2eRandom {
        pub fn _next_u32(&mut self) -> SifrInt {
            if (&self._index.clone() >= &__const__MT_N()) {
                self._twist();
            }
            let mut y: SifrInt = _state_word_at(&self._state_words, self._index.clone());
            self._index = &self._index.clone() + &SifrInt::from_i64(1);
            y = &y ^ &y.floor_div_known_nonzero(&SifrInt::from_i64(2048));
            y = &y ^ &(&(&y * &SifrInt::from_i64(128)) & &SifrInt::from_i64(2636928640));
            y = &y ^ &(&(&y * &SifrInt::from_i64(32768)) & &SifrInt::from_i64(4022730752));
            y = &y ^ &y.floor_div_known_nonzero(&SifrInt::from_i64(262144));
            &y & &__const__MT_WORD_MASK()
        }
    }
    impl __SifrStdlib_sifr_x2erandom_x2eRandom {
        pub fn random(&mut self) -> f64 {
            random_word_to_unit_float(self._next_u32())
        }
    }
    impl __SifrStdlib_sifr_x2erandom_x2eRandom {
        pub fn uniform(&mut self, minimum: f64, maximum: f64) -> f64 {
            minimum + ((maximum - minimum) * self.random())
        }
    }
    impl __SifrStdlib_sifr_x2erandom_x2eRandom {
        pub fn randrange(
            &mut self,
            start: &SifrInt,
            stop: &Option<SifrInt>,
            step: &SifrInt,
        ) -> Result<SifrInt, ValueError> {
            if (&step.clone() == &SifrInt::from_i64(0)) {
                return Err(ValueError::new("randrange: step must not be zero".to_string()));
            }
            let mut actual_start: SifrInt = start.clone();
            let mut actual_stop: SifrInt = start.clone();
            if (stop.clone() == None) {
                actual_start = SifrInt::from_i64(0);
            } else {
                if let Some(stop) = stop.as_ref() {
                    actual_stop = stop.clone();
                }
            }
            let width: SifrInt = &actual_stop - &actual_start;
            if (&step.clone() > &SifrInt::from_i64(0)) {
                if &width <= &SifrInt::from_i64(0) {
                    return Err(ValueError::new("randrange: empty range".to_string()));
                }
            } else {
                if &width >= &SifrInt::from_i64(0) {
                    return Err(ValueError::new("randrange: empty range".to_string()));
                }
            }
            let mut abs_width: SifrInt = width.clone();
            if &abs_width < &SifrInt::from_i64(0) {
                abs_width = &SifrInt::from_i64(0) - &abs_width;
            }
            let mut abs_step: SifrInt = step.clone();
            if &abs_step < &SifrInt::from_i64(0) {
                abs_step = &SifrInt::from_i64(0) - &abs_step;
            }
            if &abs_step == &SifrInt::from_i64(0) {
                return Err(ValueError::new("randrange: step must not be zero".to_string()));
            }
            let count: SifrInt = (&(&abs_width + &abs_step) - &SifrInt::from_i64(1))
                .floor_div_known_nonzero(&abs_step);
            if &count <= &SifrInt::from_i64(0) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
            if &count == &SifrInt::from_i64(0) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
            let pick: SifrInt = self._next_u32().floor_mod_known_nonzero(&count);
            Ok(&actual_start + &(&pick * step))
        }
    }
    impl __SifrStdlib_sifr_x2erandom_x2eRandom {
        pub fn randint(
            &mut self,
            minimum: &SifrInt,
            maximum: &SifrInt,
        ) -> Result<SifrInt, ValueError> {
            if *minimum > *maximum {
                return Err(ValueError::new("randint: min must be <= max".to_string()));
            }
            self.randrange(
                minimum,
                &Some((maximum + &SifrInt::from_i64(1)).clone()),
                &SifrInt::from_i64(1),
            )
        }
    }
    impl __SifrStdlib_sifr_x2erandom_x2eRandom {
        pub fn getrandbits(&mut self, k: &SifrInt) -> Result<SifrInt, ValueError> {
            if (&k.clone() < &SifrInt::from_i64(0)) {
                return Err(
                    ValueError::new("getrandbits: number of bits must be >= 0".to_string()),
                );
            }
            let mut result: SifrInt = SifrInt::from_i64(0);
            let mut bits_left: SifrInt = k.clone();
            while &bits_left > &SifrInt::from_i64(0) {
                let word: SifrInt = self._next_u32();
                let mut take: SifrInt = SifrInt::from_i64(32);
                if &bits_left < &SifrInt::from_i64(32) {
                    take = bits_left.clone();
                }
                let mut mask: SifrInt = SifrInt::from_i64(0);
                let mut shifted_result: SifrInt = result;
                let mut shift_index: SifrInt = SifrInt::from_i64(0);
                while &shift_index < &take {
                    mask = &(&mask * &SifrInt::from_i64(2)) + &SifrInt::from_i64(1);
                    shifted_result = &shifted_result * &SifrInt::from_i64(2);
                    shift_index = &shift_index + &SifrInt::from_i64(1);
                }
                result = &shifted_result | &(&word & &mask);
                bits_left = &bits_left - &take;
            }
            Ok(result.clone())
        }
    }
    impl __SifrStdlib_sifr_x2erandom_x2eRandom {
        pub fn randbytes(&mut self, n: &SifrInt) -> Result<Vec<u8>, ValueError> {
            if (&n.clone() < &SifrInt::from_i64(0)) {
                return Err(ValueError::new("randbytes: n must be >= 0".to_string()));
            }
            let mut values: Vec<SifrInt> = vec![];
            let mut i: SifrInt = SifrInt::from_i64(0);
            while i < *n {
                let byte_value: SifrInt = &self._next_u32() & &SifrInt::from_i64(255);
                values.push(byte_value.clone());
                i = &i + &SifrInt::from_i64(1);
            }
            {
                let __vals = values;
                let mut __out = Vec::new();
                for __pair in __vals.iter().enumerate() {
                    __out
                        .push(
                            __pair
                                .1
                                .try_to_u8()
                                .map_err(|_error| Err(ValueError {
                                    message: format!(
                                        "byte out of range at index {}: {}", __pair.0, * __pair.1
                                    ),
                                }))?,
                        );
                }
                Ok::<Vec<u8>, ValueError>(__out)
            }
        }
    }
    impl __SifrStdlib_sifr_x2erandom_x2eRandom {
        pub fn gauss(&mut self, mu: f64, sigma: f64) -> f64 {
            let cached: Option<f64> = self._gauss_next;
            if let Some(cached) = cached {
                self._gauss_next = None;
                return mu + (sigma * cached);
            }
            let mut u1: f64 = self.random();
            if u1 <= (0.0_f64) {
                u1 = 0.000000000001_f64;
            }
            let u2: f64 = self.random();
            let radius: f64 = sqrt(-(2.0_f64) * log(u1));
            let theta: f64 = ((2.0_f64) * PI) * u2;
            let z0: f64 = radius * cos(theta);
            let z1: f64 = radius * sin(theta);
            let next_cached: Option<f64> = Some(z1);
            self._gauss_next = next_cached;
            mu + (sigma * z0)
        }
    }
    impl __SifrStdlib_sifr_x2erandom_x2eRandom {
        pub fn getstate(&self) -> __SifrStdlib_sifr_x2erandom_x2eRandomState {
            __SifrStdlib_sifr_x2erandom_x2eRandomState::new(
                SifrInt::from_i64(3),
                _clone_words(&self._state_words),
                self._index.clone(),
                self._gauss_next,
            )
        }
    }
    impl __SifrStdlib_sifr_x2erandom_x2eRandom {
        pub fn setstate(
            &mut self,
            state: &__SifrStdlib_sifr_x2erandom_x2eRandomState,
        ) -> Result<(), ValueError> {
            if (&state.version.clone() != &SifrInt::from_i64(3)) {
                return Err(ValueError::new("setstate: unsupported version".to_string()));
            }
            if (&SifrInt::from(state.state_words.len()) != &__const__MT_N()) {
                return Err(
                    ValueError::new("setstate: state_words must have length 624".to_string()),
                );
            }
            if (&state.index.clone() < &SifrInt::from_i64(0))
                || (&state.index.clone() > &__const__MT_N())
            {
                return Err(
                    ValueError::new("setstate: index must be in range [0, 624]".to_string()),
                );
            }
            let mut normalized: Vec<SifrInt> = vec![];
            for word in state.state_words.clone().iter().cloned() {
                if (&word < &SifrInt::from_i64(0)) || (&word > &__const__MT_WORD_MASK()) {
                    return Err(ValueError::new("setstate: word out of range".to_string()));
                }
                normalized.push(&word & &__const__MT_WORD_MASK());
            }
            self._state_words = normalized;
            self._index = state.index.clone();
            self._gauss_next = state.gauss_next;
            Ok(())
        }
    }
    pub fn _state_word_at(words: &Vec<SifrInt>, index: SifrInt) -> SifrInt {
        let value: Option<SifrInt> = {
            let __sifr_index_list = &words;
            let __sifr_index_i = index.clone();
            let __sifr_index_norm = __sifr_index_i
                .normalize_index_or_len(__sifr_index_list.len());
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        if let Some(value) = value.clone() {
            return value;
        }
        SifrInt::from_i64(0)
    }
    pub fn _clone_words(words: &Vec<SifrInt>) -> Vec<SifrInt> {
        let mut copied: Vec<SifrInt> = vec![];
        for word in words.iter().cloned() {
            copied.push(word.clone());
        }
        copied
    }
    pub fn _normalize_seed_input(seed_value: Option<SifrInt>) -> SifrInt {
        if let Some(seed_value) = seed_value.clone() {
            return seed_value.clone();
        }
        random_seed()
    }
    pub fn _seed_words_from_seed(seed_value: SifrInt) -> Vec<SifrInt> {
        let mut words: Vec<SifrInt> = vec![];
        words.push(&seed_value & &__const__MT_WORD_MASK());
        let mut i: SifrInt = SifrInt::from_i64(1);
        while &i < &__const__MT_N() {
            let prev: SifrInt = _state_word_at(&words, &i - &SifrInt::from_i64(1));
            let next_word: SifrInt = &(&(&__const__MT_F()
                * &(&prev ^ &prev.floor_div_known_nonzero(&SifrInt::from_i64(1073741824))))
                + &i) & &__const__MT_WORD_MASK();
            words.push(next_word.clone());
            i = &i + &SifrInt::from_i64(1);
        }
        words
    }
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
pub use __sifr_project_nominals::ParseError;
pub use __sifr_project_nominals::ValueError;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2erandom_x2eRandom;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2erandom_x2eRandomState;
use ::sifr_runtime::SifrInt;
fn random_int(min: SifrInt, max: SifrInt) -> SifrInt {
    ::sifr_stdlib::random::random_int(
            ::sifr_runtime::interop::SifrIntBridge::from(min),
            ::sifr_runtime::interop::SifrIntBridge::from(max),
        )
        .into_sifr_int()
}
fn random_float() -> f64 {
    ::sifr_stdlib::random::random_float()
}
fn random_word_to_unit_float(value: SifrInt) -> f64 {
    ::sifr_stdlib::random::random_word_to_unit_float(
        ::sifr_runtime::interop::SifrIntBridge::from(value),
    )
}
fn random_seed() -> SifrInt {
    ::sifr_stdlib::random::random_seed().into_sifr_int()
}
fn random_uniform(min: f64, max: f64) -> f64 {
    ::sifr_stdlib::random::random_uniform(min, max)
}
fn random_randrange(
    start: SifrInt,
    stop: SifrInt,
    step: SifrInt,
) -> Result<SifrInt, ValueError> {
    ::sifr_stdlib::random::random_randrange(
            ::sifr_runtime::interop::SifrIntBridge::from(start),
            ::sifr_runtime::interop::SifrIntBridge::from(stop),
            ::sifr_runtime::interop::SifrIntBridge::from(step),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn random_gauss(mu: f64, sigma: f64) -> f64 {
    ::sifr_stdlib::random::random_gauss(mu, sigma)
}
fn random_module_state_words() -> Vec<SifrInt> {
    ::sifr_stdlib::random::random_module_state_words()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
        .collect()
}
fn random_module_state_index() -> SifrInt {
    ::sifr_stdlib::random::random_module_state_index().into_sifr_int()
}
fn random_module_state_gauss_next() -> Option<f64> {
    ::sifr_stdlib::random::random_module_state_gauss_next()
}
fn random_module_set_state(
    words: &Vec<SifrInt>,
    index: SifrInt,
    gauss_next: Option<f64>,
) -> Result<(), ValueError> {
    ::sifr_stdlib::random::random_module_set_state(
            &words
                .iter()
                .cloned()
                .map(::sifr_runtime::interop::SifrIntBridge::from)
                .collect::<Vec<_>>(),
            ::sifr_runtime::interop::SifrIntBridge::from(index),
            gauss_next.map(|__sifr_bridge_item_0| __sifr_bridge_item_0),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_encode(s: &String) -> String {
    ::sifr_stdlib::base64::base64_encode(s)
}
fn base64_encode_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::base64::base64_encode_bytes(data)
}
fn base64_decode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_decode_bytes(data: &Vec<u8>) -> Result<Vec<u8>, ParseError> {
    ::sifr_stdlib::base64::base64_decode_bytes(data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_encode_opts(
    s: &String,
    altchars: &String,
    wrapcol: SifrInt,
) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_encode_opts(
            s,
            altchars,
            ::sifr_runtime::interop::SifrIntBridge::from(wrapcol),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_decode_opts(
    s: &String,
    altchars: &String,
    validate: bool,
    ignorechars: &String,
) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_decode_opts(s, altchars, validate, ignorechars)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn urlsafe_b64encode(s: &String) -> String {
    ::sifr_stdlib::base64::urlsafe_b64encode(s)
}
fn urlsafe_b64encode_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::base64::urlsafe_b64encode_bytes(data)
}
fn urlsafe_b64decode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::urlsafe_b64decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn urlsafe_b64decode_bytes(data: &Vec<u8>) -> Result<Vec<u8>, ParseError> {
    ::sifr_stdlib::base64::urlsafe_b64decode_bytes(data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn b32encode(s: &String) -> String {
    ::sifr_stdlib::base64::b32encode(s)
}
fn b32decode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::b32decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn b32hexencode(s: &String) -> String {
    ::sifr_stdlib::base64::b32hexencode(s)
}
fn b32hexdecode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::b32hexdecode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn sha256_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha256_bytes(data)
}
fn md5_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::md5_bytes(data)
}
fn sha1_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha1_bytes(data)
}
fn sha224_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha224_bytes(data)
}
fn sha384_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha384_bytes(data)
}
fn sha512_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha512_bytes(data)
}
fn blake2b_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::blake2b_bytes(data)
}
fn blake2s_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::blake2s_bytes(data)
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
    while &y != &SifrInt::from_i64(0) {
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
    while &i < &r {
        result = &result * &(&n - &i);
        let divisor: SifrInt = &i + &SifrInt::from_i64(1);
        if &divisor == &SifrInt::from_i64(0) {
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
        let __sifr_index_list = &parts;
        let __sifr_index_i = SifrInt::from_i64(0);
        let __sifr_index_norm = __sifr_index_i
            .normalize_index_or_len(__sifr_index_list.len());
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(m) = m else {
        return NAN;
    };
    m
}
fn frexp_exponent(x: f64) -> SifrInt {
    let parts: Vec<f64> = frexp(x);
    let exp_val: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = SifrInt::from_i64(1);
        let __sifr_index_norm = __sifr_index_i
            .normalize_index_or_len(__sifr_index_list.len());
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(exp_val) = exp_val else {
        return SifrInt::from_i64(0);
    };
    trunc(exp_val)
}
fn modf_fractional(x: f64) -> f64 {
    let parts: Vec<f64> = modf(x);
    let f: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = SifrInt::from_i64(0);
        let __sifr_index_norm = __sifr_index_i
            .normalize_index_or_len(__sifr_index_list.len());
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
        let __sifr_index_i = SifrInt::from_i64(1);
        let __sifr_index_norm = __sifr_index_i
            .normalize_index_or_len(__sifr_index_list.len());
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
fn __const__MT_N() -> SifrInt {
    SifrInt::from_i64(624)
}
fn __const__MT_M() -> SifrInt {
    SifrInt::from_i64(397)
}
fn __const__MT_MATRIX_A() -> SifrInt {
    SifrInt::from_i64(2567483615)
}
fn __const__MT_UPPER_MASK() -> SifrInt {
    SifrInt::from_i64(2147483648)
}
fn __const__MT_LOWER_MASK() -> SifrInt {
    SifrInt::from_i64(2147483647)
}
fn __const__MT_F() -> SifrInt {
    SifrInt::from_i64(1812433253)
}
fn __const__MT_WORD_MASK() -> SifrInt {
    SifrInt::from_i64(4294967295)
}
fn _state_word_at(words: &Vec<SifrInt>, index: SifrInt) -> SifrInt {
    let value: Option<SifrInt> = {
        let __sifr_index_list = &words;
        let __sifr_index_i = index.clone();
        let __sifr_index_norm = __sifr_index_i
            .normalize_index_or_len(__sifr_index_list.len());
        __sifr_index_list.get(__sifr_index_norm).cloned()
    };
    if let Some(value) = value.clone() {
        return value;
    }
    SifrInt::from_i64(0)
}
fn _clone_words(words: &Vec<SifrInt>) -> Vec<SifrInt> {
    let mut copied: Vec<SifrInt> = vec![];
    for word in words.iter().cloned() {
        copied.push(word.clone());
    }
    copied
}
fn _normalize_seed_input(seed_value: Option<SifrInt>) -> SifrInt {
    if let Some(seed_value) = seed_value.clone() {
        return seed_value.clone();
    }
    random_seed()
}
fn _seed_words_from_seed(seed_value: SifrInt) -> Vec<SifrInt> {
    let mut words: Vec<SifrInt> = vec![];
    words.push(&seed_value & &__const__MT_WORD_MASK());
    let mut i: SifrInt = SifrInt::from_i64(1);
    while &i < &__const__MT_N() {
        let prev: SifrInt = _state_word_at(&words, &i - &SifrInt::from_i64(1));
        let next_word: SifrInt = &(&(&__const__MT_F()
            * &(&prev ^ &prev.floor_div_known_nonzero(&SifrInt::from_i64(1073741824))))
            + &i) & &__const__MT_WORD_MASK();
        words.push(next_word.clone());
        i = &i + &SifrInt::from_i64(1);
    }
    words
}
fn _build_state_from_module_storage() -> __SifrStdlib_sifr_x2erandom_x2eRandomState {
    __SifrStdlib_sifr_x2erandom_x2eRandomState::new(
        SifrInt::from_i64(3),
        random_module_state_words(),
        random_module_state_index(),
        random_module_state_gauss_next(),
    )
}
fn _store_state_into_module_storage(state: &__SifrStdlib_sifr_x2erandom_x2eRandomState) {
    let _set_result: Result<(), ValueError> = random_module_set_state(
        &_clone_words(&state.state_words.clone()),
        state.index.clone(),
        state.gauss_next,
    );
    let _ = _set_result;
}
fn _ensure_module_state_initialized() {
    let words: Vec<SifrInt> = random_module_state_words();
    if &SifrInt::from(words.len()) == &__const__MT_N() {
        return;
    }
    let bootstrap: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        Some(SifrInt::from_i64(5489)),
    );
    _store_state_into_module_storage(&bootstrap.getstate());
}
fn _module_random() -> __SifrStdlib_sifr_x2erandom_x2eRandom {
    _ensure_module_state_initialized();
    let mut r: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        Some(SifrInt::from_i64(0)),
    );
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let _set_result: Result<(), ValueError> = r
            .setstate(&_build_state_from_module_storage());
        let _ = _set_result;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = e.message.clone();
    }
    r
}
fn _sync_module_random(generator: &mut __SifrStdlib_sifr_x2erandom_x2eRandom) {
    _store_state_into_module_storage(&generator.getstate());
}
fn seed(seed_value: Option<SifrInt>) {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        (seed_value).clone(),
    );
    _sync_module_random(&mut generator);
}
fn getstate() -> __SifrStdlib_sifr_x2erandom_x2eRandomState {
    _ensure_module_state_initialized();
    _build_state_from_module_storage()
}
fn setstate(
    state: &__SifrStdlib_sifr_x2erandom_x2eRandomState,
) -> Result<(), ValueError> {
    let mut probe: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        Some(SifrInt::from_i64(0)),
    );
    let result: Result<(), ValueError> = probe.setstate(state);
    _sync_module_random(&mut probe);
    result
}
fn randint(minimum: SifrInt, maximum: SifrInt) -> Result<SifrInt, ValueError> {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: Result<SifrInt, ValueError> = generator.randint(&minimum, &maximum);
    _sync_module_random(&mut generator);
    value
}
fn random() -> f64 {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: f64 = generator.random();
    _sync_module_random(&mut generator);
    value
}
fn main() {
    let mut rng: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        Some(SifrInt::from_i64(77)),
    );
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let state_before: __SifrStdlib_sifr_x2erandom_x2eRandomState = rng.getstate();
        let next_one: SifrInt = rng
            .randint(&SifrInt::from_i64(0), &SifrInt::from_i64(100000))?;
        let _rng_set_result: Result<(), ValueError> = rng.setstate(&state_before);
        let _ = _rng_set_result;
        let replay_one: SifrInt = rng
            .randint(&SifrInt::from_i64(0), &SifrInt::from_i64(100000))?;
        assert!(& next_one == & replay_one);
        seed(Some(SifrInt::from_i64(1234)));
        let first_module_random: f64 = random();
        let second_module_int: SifrInt = randint(
            SifrInt::from_i64(0),
            SifrInt::from_i64(100000),
        )?;
        let module_state: __SifrStdlib_sifr_x2erandom_x2eRandomState = getstate();
        let after_state_int: SifrInt = randint(
            SifrInt::from_i64(0),
            SifrInt::from_i64(100000),
        )?;
        let _module_set_result: Result<(), ValueError> = setstate(&module_state);
        let _ = _module_set_result;
        let replay_after_state_int: SifrInt = randint(
            SifrInt::from_i64(0),
            SifrInt::from_i64(100000),
        )?;
        assert!(& after_state_int == & replay_after_state_int);
        seed(Some(SifrInt::from_i64(1234)));
        assert!((first_module_random == random()));
        let replay_second_module_int: SifrInt = randint(
            SifrInt::from_i64(0),
            SifrInt::from_i64(100000),
        )?;
        assert!(& second_module_int == & replay_second_module_int);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        assert!(
            (format!("{}", format!("{}{}", "unexpected random error: ", e.message
            .clone())) == "rng_random_state_object_model_demo: pass")
        );
    }
    assert!(
        (format!("{}", "rng_random_state_object_model_demo: pass") ==
        "rng_random_state_object_model_demo: pass")
    );
}
