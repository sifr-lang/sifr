// src/main.rs
mod __sifr_project_nominals {
    pub fn random_int(min: i64, max: i64) -> i64 {
        ::sifr_stdlib::random::random_int(
                ::sifr_runtime::interop::SifrIntBridge::from(min),
                ::sifr_runtime::interop::SifrIntBridge::from(max),
            )
            .to_i64_saturating()
    }
    pub fn random_float() -> f64 {
        ::sifr_stdlib::random::random_float()
    }
    pub fn random_uniform(min: f64, max: f64) -> f64 {
        ::sifr_stdlib::random::random_uniform(min, max)
    }
    pub fn random_randrange(start: i64, stop: i64, step: i64) -> Result<i64, ValueError> {
        ::sifr_stdlib::random::random_randrange(
                ::sifr_runtime::interop::SifrIntBridge::from(start),
                ::sifr_runtime::interop::SifrIntBridge::from(stop),
                ::sifr_runtime::interop::SifrIntBridge::from(step),
            )
            .map(|__sifr_bridge_ok| __sifr_bridge_ok.to_i64_saturating())
            .map_err(|__sifr_bridge_error| ValueError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn random_gauss(mu: f64, sigma: f64) -> f64 {
        ::sifr_stdlib::random::random_gauss(mu, sigma)
    }
    pub fn random_module_state_words() -> Vec<i64> {
        ::sifr_stdlib::random::random_module_state_words()
            .into_iter()
            .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
            .collect()
    }
    pub fn random_module_state_index() -> i64 {
        ::sifr_stdlib::random::random_module_state_index().to_i64_saturating()
    }
    pub fn random_module_state_gauss_next() -> Option<f64> {
        ::sifr_stdlib::random::random_module_state_gauss_next()
    }
    pub fn random_module_set_state(
        words: &Vec<i64>,
        index: i64,
        gauss_next: Option<f64>,
    ) -> Result<(), ValueError> {
        ::sifr_stdlib::random::random_module_set_state(
                &words
                    .iter()
                    .copied()
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
        wrapcol: i64,
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
    pub fn time_now() -> f64 {
        ::sifr_stdlib::time::time_now()
    }
    pub fn time_format(epoch: f64, fmt: &String) -> String {
        ::sifr_stdlib::time::time_format(epoch, fmt)
    }
    pub fn perf_counter() -> f64 {
        ::sifr_stdlib::time::perf_counter()
    }
    pub fn sleep(seconds: f64) {
        ::sifr_stdlib::time::sleep(seconds);
    }
    pub fn monotonic() -> f64 {
        ::sifr_stdlib::time::monotonic()
    }
    pub fn strptime(s: &String, fmt: &String) -> Result<String, ValueError> {
        ::sifr_stdlib::time::strptime(s, fmt)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ValueError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn _strptime_intrinsic(s: &String, fmt: &String) -> Result<String, ValueError> {
        ::sifr_stdlib::time::strptime(s, fmt)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ValueError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn gmtime(epoch: f64) -> String {
        ::sifr_stdlib::time::gmtime(epoch)
    }
    pub fn _gmtime_intrinsic(epoch: f64) -> String {
        ::sifr_stdlib::time::gmtime(epoch)
    }
    pub fn localtime(epoch: f64) -> String {
        ::sifr_stdlib::time::localtime(epoch)
    }
    pub fn _localtime_intrinsic(epoch: f64) -> String {
        ::sifr_stdlib::time::localtime(epoch)
    }
    pub fn time_strptime(s: &String, fmt: &String) -> Result<Vec<i64>, ValueError> {
        ::sifr_stdlib::time::time_strptime(s, fmt)
            .map(|__sifr_bridge_ok| {
                __sifr_bridge_ok
                    .into_iter()
                    .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
                    .collect()
            })
            .map_err(|__sifr_bridge_error| ValueError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn time_gmtime() -> Vec<i64> {
        ::sifr_stdlib::time::time_gmtime()
            .into_iter()
            .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
            .collect()
    }
    pub fn time_localtime() -> Vec<i64> {
        ::sifr_stdlib::time::time_localtime()
            .into_iter()
            .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
            .collect()
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
    pub const _MT_N: i64 = 624_i64;
    pub const _MT_M: i64 = 397_i64;
    pub const _MT_MATRIX_A: i64 = 2567483615_i64;
    pub const _MT_UPPER_MASK: i64 = 2147483648_i64;
    pub const _MT_LOWER_MASK: i64 = 2147483647_i64;
    pub const _MT_F: i64 = 1812433253_i64;
    pub const _MT_WORD_MASK: i64 = 4294967295_i64;
    #[derive(Debug, Clone, PartialEq)]
    pub struct __SifrStdlib_sifr_x2erandom_x2eRandomState {
        pub version: i64,
        pub state_words: Vec<i64>,
        pub index: i64,
        pub gauss_next: Option<f64>,
    }
    impl __SifrStdlib_sifr_x2erandom_x2eRandomState {
        pub fn new(
            version: i64,
            state_words: Vec<i64>,
            index: i64,
            gauss_next: Option<f64>,
        ) -> Self {
            let __sifr_field_init_0: i64 = version;
            let __sifr_field_init_1: Vec<i64> = state_words;
            let __sifr_field_init_2: i64 = index;
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
        pub _state_words: Vec<i64>,
        pub _index: i64,
        pub _gauss_next: Option<f64>,
    }
    impl __SifrStdlib_sifr_x2erandom_x2eRandom {
        pub fn new(seed_value: Option<i64>) -> Self {
            let normalized_seed: i64 = _normalize_seed_input(seed_value);
            let __sifr_field_init_0: Vec<i64> = _seed_words_from_seed(normalized_seed);
            let __sifr_field_init_1: i64 = _MT_N;
            let __sifr_field_init_2: Option<f64> = None;
            Self {
                _state_words: __sifr_field_init_0,
                _index: __sifr_field_init_1,
                _gauss_next: __sifr_field_init_2,
            }
        }
    }
    impl __SifrStdlib_sifr_x2erandom_x2eRandom {
        pub fn seed(&mut self, seed_value: Option<i64>) {
            let normalized_seed: i64 = _normalize_seed_input(seed_value);
            self._state_words = _seed_words_from_seed(normalized_seed);
            self._index = _MT_N;
            self._gauss_next = None;
        }
    }
    impl __SifrStdlib_sifr_x2erandom_x2eRandom {
        pub fn _twist(&mut self) {
            let mut i: i64 = 0_i64;
            while i < _MT_N {
                let y: i64 = (_state_word_at(&self._state_words, i) & _MT_UPPER_MASK)
                    + (_state_word_at(&self._state_words, (i + (1_i64)) % _MT_N)
                        & _MT_LOWER_MASK);
                let mut x_a: i64 = y >> (1_i64);
                if (y % (2_i64)) != (0_i64) {
                    x_a = x_a ^ _MT_MATRIX_A;
                }
                let new_word: i64 = _state_word_at(&self._state_words, (i + _MT_M) % _MT_N)
                    ^ x_a;
                {
                    let __idx_raw = i;
                    let __idx_norm = if __idx_raw < 0 {
                        (self._state_words.len() as i64) + __idx_raw
                    } else {
                        __idx_raw
                    };
                    if __idx_norm >= 0 {
                        if let Some(__elem) = self._state_words.get_mut(__idx_norm as usize)
                        {
                            *__elem = new_word & _MT_WORD_MASK;
                        }
                    }
                }
                i += 1_i64;
            }
            self._index = 0_i64;
        }
    }
    impl __SifrStdlib_sifr_x2erandom_x2eRandom {
        pub fn _next_u32(&mut self) -> i64 {
            if (self._index >= _MT_N) {
                self._twist();
            }
            let mut y: i64 = _state_word_at(&self._state_words, self._index);
            self._index += 1_i64;
            y = y ^ (y >> (11_i64));
            y = y ^ ((y << (7_i64)) & (2636928640_i64));
            y = y ^ ((y << (15_i64)) & (4022730752_i64));
            y = y ^ (y >> (18_i64));
            y & _MT_WORD_MASK
        }
    }
    impl __SifrStdlib_sifr_x2erandom_x2eRandom {
        pub fn random(&mut self) -> f64 {
            (self._next_u32() as f64) / (4294967296.0_f64)
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
            start: i64,
            stop: Option<i64>,
            step: i64,
        ) -> Result<i64, ValueError> {
            if step == (0_i64) {
                return Err(ValueError::new("randrange: step must not be zero".to_string()));
            }
            let mut actual_start: i64 = start;
            let mut actual_stop: i64 = start;
            if stop.is_none() {
                actual_start = 0_i64;
            } else {
                if let Some(stop) = stop {
                    actual_stop = stop;
                }
            }
            let width: i64 = actual_stop - actual_start;
            if step > (0_i64) {
                if width <= (0_i64) {
                    return Err(ValueError::new("randrange: empty range".to_string()));
                }
            } else {
                if width >= (0_i64) {
                    return Err(ValueError::new("randrange: empty range".to_string()));
                }
            }
            let mut abs_width: i64 = width;
            if abs_width < (0_i64) {
                abs_width = (0_i64) - abs_width;
            }
            let mut abs_step: i64 = step;
            if abs_step < (0_i64) {
                abs_step = (0_i64) - abs_step;
            }
            let count: i64 = ((abs_width + abs_step) - (1_i64)) / abs_step;
            if count <= (0_i64) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
            let pick: i64 = self._next_u32() % count;
            Ok(actual_start + (pick * step))
        }
    }
    impl __SifrStdlib_sifr_x2erandom_x2eRandom {
        pub fn randint(&mut self, minimum: i64, maximum: i64) -> Result<i64, ValueError> {
            if minimum > maximum {
                return Err(ValueError::new("randint: min must be <= max".to_string()));
            }
            self.randrange(minimum, Some(maximum + (1_i64)), 1_i64)
        }
    }
    impl __SifrStdlib_sifr_x2erandom_x2eRandom {
        pub fn getrandbits(&mut self, k: i64) -> Result<i64, ValueError> {
            if k < (0_i64) {
                return Err(
                    ValueError::new("getrandbits: number of bits must be >= 0".to_string()),
                );
            }
            let mut result: i64 = 0_i64;
            let mut bits_left: i64 = k;
            while bits_left > (0_i64) {
                let word: i64 = self._next_u32();
                let mut take: i64 = 32_i64;
                if bits_left < (32_i64) {
                    take = bits_left;
                }
                let mask: i64 = ((1_i64) << take) - (1_i64);
                result = (result << take) | (word & mask);
                bits_left -= take;
            }
            Ok(result)
        }
    }
    impl __SifrStdlib_sifr_x2erandom_x2eRandom {
        pub fn randbytes(&mut self, n: i64) -> Result<Vec<u8>, ValueError> {
            if n < (0_i64) {
                return Err(ValueError::new("randbytes: n must be >= 0".to_string()));
            }
            let mut values: Vec<i64> = vec![];
            let mut i: i64 = 0_i64;
            while i < n {
                let byte_value: i64 = self._next_u32() & (255_i64);
                values.push(byte_value);
                i += 1_i64;
            }
            {
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
                3_i64,
                _clone_words(&self._state_words),
                self._index,
                self._gauss_next,
            )
        }
    }
    impl __SifrStdlib_sifr_x2erandom_x2eRandom {
        pub fn setstate(
            &mut self,
            state: &__SifrStdlib_sifr_x2erandom_x2eRandomState,
        ) -> Result<(), ValueError> {
            if (state.version != (3_i64)) {
                return Err(ValueError::new("setstate: unsupported version".to_string()));
            }
            if ((state.state_words.len() as i64) != _MT_N) {
                return Err(
                    ValueError::new("setstate: state_words must have length 624".to_string()),
                );
            }
            if (state.index < (0_i64)) || (state.index > _MT_N) {
                return Err(
                    ValueError::new("setstate: index must be in range [0, 624]".to_string()),
                );
            }
            let mut normalized: Vec<i64> = vec![];
            for word in state.state_words.clone().iter().copied() {
                if (word < (0_i64)) || (word > _MT_WORD_MASK) {
                    return Err(ValueError::new("setstate: word out of range".to_string()));
                }
                normalized.push(word & _MT_WORD_MASK);
            }
            self._state_words = normalized;
            self._index = state.index;
            self._gauss_next = state.gauss_next;
            Ok(())
        }
    }
    pub fn _state_word_at(words: &Vec<i64>, index: i64) -> i64 {
        let value: Option<i64> = {
            let __sifr_index_list = &words;
            let __sifr_index_i = index;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        if let Some(value) = value {
            return value;
        }
        0_i64
    }
    pub fn _clone_words(words: &Vec<i64>) -> Vec<i64> {
        let mut copied: Vec<i64> = vec![];
        for word in words.iter().copied() {
            copied.push(word);
        }
        copied
    }
    pub fn _normalize_seed_input(seed_value: Option<i64>) -> i64 {
        if let Some(seed_value) = seed_value {
            return seed_value;
        }
        (time_now() * (1000000.0_f64)) as i64
    }
    pub fn _seed_words_from_seed(seed_value: i64) -> Vec<i64> {
        let mut words: Vec<i64> = vec![];
        words.push(seed_value & _MT_WORD_MASK);
        let mut i: i64 = 1_i64;
        while i < _MT_N {
            let prev: i64 = _state_word_at(&words, i - (1_i64));
            let next_word: i64 = ((_MT_F * (prev ^ (prev >> (30_i64)))) + i) & _MT_WORD_MASK;
            words.push(next_word);
            i += 1_i64;
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
fn random_int(min: i64, max: i64) -> i64 {
    ::sifr_stdlib::random::random_int(
            ::sifr_runtime::interop::SifrIntBridge::from(min),
            ::sifr_runtime::interop::SifrIntBridge::from(max),
        )
        .to_i64_saturating()
}
fn random_float() -> f64 {
    ::sifr_stdlib::random::random_float()
}
fn random_uniform(min: f64, max: f64) -> f64 {
    ::sifr_stdlib::random::random_uniform(min, max)
}
fn random_randrange(start: i64, stop: i64, step: i64) -> Result<i64, ValueError> {
    ::sifr_stdlib::random::random_randrange(
            ::sifr_runtime::interop::SifrIntBridge::from(start),
            ::sifr_runtime::interop::SifrIntBridge::from(stop),
            ::sifr_runtime::interop::SifrIntBridge::from(step),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.to_i64_saturating())
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn random_gauss(mu: f64, sigma: f64) -> f64 {
    ::sifr_stdlib::random::random_gauss(mu, sigma)
}
fn random_module_state_words() -> Vec<i64> {
    ::sifr_stdlib::random::random_module_state_words()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}
fn random_module_state_index() -> i64 {
    ::sifr_stdlib::random::random_module_state_index().to_i64_saturating()
}
fn random_module_state_gauss_next() -> Option<f64> {
    ::sifr_stdlib::random::random_module_state_gauss_next()
}
fn random_module_set_state(
    words: &Vec<i64>,
    index: i64,
    gauss_next: Option<f64>,
) -> Result<(), ValueError> {
    ::sifr_stdlib::random::random_module_set_state(
            &words
                .iter()
                .copied()
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
    wrapcol: i64,
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
fn time_now() -> f64 {
    ::sifr_stdlib::time::time_now()
}
fn time_format(epoch: f64, fmt: &String) -> String {
    ::sifr_stdlib::time::time_format(epoch, fmt)
}
fn perf_counter() -> f64 {
    ::sifr_stdlib::time::perf_counter()
}
fn sleep(seconds: f64) {
    ::sifr_stdlib::time::sleep(seconds);
}
fn monotonic() -> f64 {
    ::sifr_stdlib::time::monotonic()
}
fn strptime(s: &String, fmt: &String) -> Result<String, ValueError> {
    ::sifr_stdlib::time::strptime(s, fmt)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _strptime_intrinsic(s: &String, fmt: &String) -> Result<String, ValueError> {
    ::sifr_stdlib::time::strptime(s, fmt)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn gmtime(epoch: f64) -> String {
    ::sifr_stdlib::time::gmtime(epoch)
}
fn _gmtime_intrinsic(epoch: f64) -> String {
    ::sifr_stdlib::time::gmtime(epoch)
}
fn localtime(epoch: f64) -> String {
    ::sifr_stdlib::time::localtime(epoch)
}
fn _localtime_intrinsic(epoch: f64) -> String {
    ::sifr_stdlib::time::localtime(epoch)
}
fn time_strptime(s: &String, fmt: &String) -> Result<Vec<i64>, ValueError> {
    ::sifr_stdlib::time::time_strptime(s, fmt)
        .map(|__sifr_bridge_ok| {
            __sifr_bridge_ok
                .into_iter()
                .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
                .collect()
        })
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn time_gmtime() -> Vec<i64> {
    ::sifr_stdlib::time::time_gmtime()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}
fn time_localtime() -> Vec<i64> {
    ::sifr_stdlib::time::time_localtime()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
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
const _MT_N: i64 = 624_i64;
const _MT_M: i64 = 397_i64;
const _MT_MATRIX_A: i64 = 2567483615_i64;
const _MT_UPPER_MASK: i64 = 2147483648_i64;
const _MT_LOWER_MASK: i64 = 2147483647_i64;
const _MT_F: i64 = 1812433253_i64;
const _MT_WORD_MASK: i64 = 4294967295_i64;
fn _state_word_at(words: &Vec<i64>, index: i64) -> i64 {
    let value: Option<i64> = {
        let __sifr_index_list = &words;
        let __sifr_index_i = index;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    if let Some(value) = value {
        return value;
    }
    0_i64
}
fn _clone_words(words: &Vec<i64>) -> Vec<i64> {
    let mut copied: Vec<i64> = vec![];
    for word in words.iter().copied() {
        copied.push(word);
    }
    copied
}
fn _normalize_seed_input(seed_value: Option<i64>) -> i64 {
    if let Some(seed_value) = seed_value {
        return seed_value;
    }
    (time_now() * (1000000.0_f64)) as i64
}
fn _seed_words_from_seed(seed_value: i64) -> Vec<i64> {
    let mut words: Vec<i64> = vec![];
    words.push(seed_value & _MT_WORD_MASK);
    let mut i: i64 = 1_i64;
    while i < _MT_N {
        let prev: i64 = _state_word_at(&words, i - (1_i64));
        let next_word: i64 = ((_MT_F * (prev ^ (prev >> (30_i64)))) + i) & _MT_WORD_MASK;
        words.push(next_word);
        i += 1_i64;
    }
    words
}
fn _build_state_from_module_storage() -> __SifrStdlib_sifr_x2erandom_x2eRandomState {
    __SifrStdlib_sifr_x2erandom_x2eRandomState::new(
        3_i64,
        random_module_state_words(),
        random_module_state_index(),
        random_module_state_gauss_next(),
    )
}
fn _store_state_into_module_storage(state: &__SifrStdlib_sifr_x2erandom_x2eRandomState) {
    let _set_result: Result<(), ValueError> = random_module_set_state(
        &_clone_words(&state.state_words.clone()),
        state.index,
        state.gauss_next,
    );
    let _ = _set_result;
}
fn _ensure_module_state_initialized() {
    let words: Vec<i64> = random_module_state_words();
    if (words.len() as i64) == _MT_N {
        return;
    }
    let bootstrap: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        Some(5489_i64),
    );
    _store_state_into_module_storage(&bootstrap.getstate());
}
fn _module_random() -> __SifrStdlib_sifr_x2erandom_x2eRandom {
    _ensure_module_state_initialized();
    let mut r: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        Some(0_i64),
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
fn seed(seed_value: Option<i64>) {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        seed_value,
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
        Some(0_i64),
    );
    let result: Result<(), ValueError> = probe.setstate(state);
    _sync_module_random(&mut probe);
    result
}
fn randint(minimum: i64, maximum: i64) -> Result<i64, ValueError> {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: Result<i64, ValueError> = generator.randint(minimum, maximum);
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
        Some(77_i64),
    );
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let state_before: __SifrStdlib_sifr_x2erandom_x2eRandomState = rng.getstate();
        let next_one: i64 = rng.randint(0_i64, 100000_i64)?;
        let _rng_set_result: Result<(), ValueError> = rng.setstate(&state_before);
        let _ = _rng_set_result;
        let replay_one: i64 = rng.randint(0_i64, 100000_i64)?;
        assert!(next_one == replay_one);
        seed(Some(1234_i64));
        let first_module_random: f64 = random();
        let second_module_int: i64 = randint(0_i64, 100000_i64)?;
        let module_state: __SifrStdlib_sifr_x2erandom_x2eRandomState = getstate();
        let after_state_int: i64 = randint(0_i64, 100000_i64)?;
        let _module_set_result: Result<(), ValueError> = setstate(&module_state);
        let _ = _module_set_result;
        let replay_after_state_int: i64 = randint(0_i64, 100000_i64)?;
        assert!(after_state_int == replay_after_state_int);
        seed(Some(1234_i64));
        assert!((first_module_random == random()));
        let replay_second_module_int: i64 = randint(0_i64, 100000_i64)?;
        assert!(second_module_int == replay_second_module_int);
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
