use sifr_runtime::interop::SifrIntBridge;

#[must_use]
pub const fn feature_name() -> &'static str {
    "math"
}

macro_rules! unary_float {
    ($name:ident, $method:ident) => {
        #[must_use]
        pub fn $name(x: f64) -> f64 {
            x.$method()
        }
    };
}

macro_rules! binary_float {
    ($name:ident, $method:ident) => {
        #[must_use]
        pub fn $name(x: f64, y: f64) -> f64 {
            x.$method(y)
        }
    };
}

unary_float!(sqrt, sqrt);
unary_float!(abs_val, abs);
unary_float!(log, ln);
unary_float!(cbrt, cbrt);
unary_float!(exp2, exp2);
unary_float!(sin, sin);
unary_float!(cos, cos);
unary_float!(tan, tan);
unary_float!(asin, asin);
unary_float!(acos, acos);
unary_float!(atan, atan);
unary_float!(sinh, sinh);
unary_float!(cosh, cosh);
unary_float!(tanh, tanh);
unary_float!(log10, log10);
unary_float!(log2, log2);
unary_float!(degrees, to_degrees);
unary_float!(radians, to_radians);
unary_float!(exp, exp);
unary_float!(expm1, exp_m1);
unary_float!(log1p, ln_1p);
unary_float!(fabs, abs);
unary_float!(acosh, acosh);
unary_float!(asinh, asinh);
unary_float!(atanh, atanh);
binary_float!(copysign, copysign);
binary_float!(hypot, hypot);
binary_float!(pow_val, powf);
binary_float!(min_val, min);
binary_float!(max_val, max);
binary_float!(fmax, max);
binary_float!(fmin, min);
binary_float!(atan2, atan2);

#[must_use]
pub fn floor(x: f64) -> SifrIntBridge {
    SifrIntBridge::from(x.floor() as i64)
}

#[must_use]
pub fn ceil(x: f64) -> SifrIntBridge {
    SifrIntBridge::from(x.ceil() as i64)
}

#[must_use]
pub fn round_val(x: f64) -> SifrIntBridge {
    SifrIntBridge::from(x.round() as i64)
}

#[must_use]
pub fn trunc(x: f64) -> SifrIntBridge {
    SifrIntBridge::from(x.trunc() as i64)
}

#[must_use]
pub const fn isnan(x: f64) -> bool {
    x.is_nan()
}

#[must_use]
pub const fn isinf(x: f64) -> bool {
    x.is_infinite()
}

#[must_use]
pub const fn isfinite(x: f64) -> bool {
    x.is_finite()
}

#[must_use]
pub const fn isnormal(x: f64) -> bool {
    x.is_normal()
}

#[must_use]
pub const fn signbit(x: f64) -> bool {
    x.is_sign_negative()
}

#[must_use]
pub fn issubnormal(x: f64) -> bool {
    x.is_finite() && !x.is_normal()
}

#[must_use]
pub fn fmod(x: f64, y: f64) -> f64 {
    x % y
}

#[must_use]
pub fn fma(x: f64, y: f64, z: f64) -> f64 {
    (x * y) + z
}

#[must_use]
pub fn isqrt(n: SifrIntBridge) -> SifrIntBridge {
    SifrIntBridge::from((n.to_i64_saturating() as f64).sqrt() as i64)
}

#[must_use]
pub fn remainder(x: f64, y: f64) -> f64 {
    if x.is_nan() || y.is_nan() || y == 0.0 || x.is_infinite() {
        return f64::NAN;
    }
    if y.is_infinite() {
        return x;
    }
    let q = x / y;
    let n0 = q.trunc();
    let frac = q - n0;
    let abs_frac = frac.abs();
    let n = if abs_frac < 0.5 {
        n0
    } else if abs_frac > 0.5 || (n0 as i64) % 2 != 0 {
        n0 + q.signum()
    } else {
        n0
    };
    let r = x - (n * y);
    if r == 0.0 {
        0.0_f64.copysign(x)
    } else {
        r
    }
}

#[must_use]
pub fn dist(p: Vec<f64>, q: Vec<f64>) -> f64 {
    if p.len() != q.len() {
        return f64::NAN;
    }
    if p.is_empty() {
        return 0.0;
    }
    let mut scale = 0.0_f64;
    let mut ssq = 1.0_f64;
    for (left, right) in p.into_iter().zip(q) {
        let d = (left - right).abs();
        if d != 0.0 {
            if scale < d {
                let r = scale / d;
                ssq = 1.0 + (ssq * r * r);
                scale = d;
            } else {
                let r = d / scale;
                ssq += r * r;
            }
        }
    }
    if scale == 0.0 {
        0.0
    } else {
        scale * ssq.sqrt()
    }
}

#[must_use]
pub fn fsum(data: Vec<f64>) -> f64 {
    let mut sum = 0.0_f64;
    let mut comp = 0.0_f64;
    let mut pos_inf = false;
    let mut neg_inf = false;
    let mut has_nan = false;
    for v in data {
        if v.is_nan() {
            has_nan = true;
            continue;
        }
        if v.is_infinite() {
            if v.is_sign_positive() {
                pos_inf = true;
            } else {
                neg_inf = true;
            }
            continue;
        }
        let t = sum + v;
        if sum.abs() >= v.abs() {
            comp += (sum - t) + v;
        } else {
            comp += (v - t) + sum;
        }
        sum = t;
    }
    if has_nan || (pos_inf && neg_inf) {
        f64::NAN
    } else if pos_inf {
        f64::INFINITY
    } else if neg_inf {
        f64::NEG_INFINITY
    } else {
        sum + comp
    }
}

#[must_use]
pub fn sumprod(p: Vec<f64>, q: Vec<f64>) -> f64 {
    p.into_iter().zip(q).map(|(left, right)| left * right).sum()
}

#[must_use]
pub fn erf(x: f64) -> f64 {
    let t = 1.0 / (1.0 + (0.327_591_1 * x.abs()));
    let poly = t
        * (0.254_829_592
            + t * (-0.284_496_736
                + t * (1.421_413_741 + t * (-1.453_152_027 + t * 1.061_405_429))));
    let r = 1.0 - (poly * (-x * x).exp());
    if x >= 0.0 {
        r
    } else {
        -r
    }
}

#[must_use]
pub fn erfc(x: f64) -> f64 {
    let t = 1.0 / (1.0 + (0.327_591_1 * x.abs()));
    let poly = t
        * (0.254_829_592
            + t * (-0.284_496_736
                + t * (1.421_413_741 + t * (-1.453_152_027 + t * 1.061_405_429))));
    let r = poly * (-x * x).exp();
    if x >= 0.0 {
        r
    } else {
        2.0 - r
    }
}

const LANCZOS_G: f64 = 7.0;
const LANCZOS: [f64; 9] = [
    0.999_999_999_999_809_9,
    676.520_368_121_885_1,
    -1_259.139_216_722_402_8,
    771.323_428_777_653_1,
    -176.615_029_162_140_6,
    12.507_343_278_686_905,
    -0.138_571_095_265_720_12,
    0.000_009_984_369_578_019_572,
    0.000_000_150_563_273_514_931_16,
];

fn lanczos_sum(x: f64) -> f64 {
    let mut sum = LANCZOS[0];
    for (idx, coeff) in LANCZOS.iter().enumerate().skip(1) {
        sum += coeff / (x + idx as f64);
    }
    sum
}

fn gamma_positive(x: f64) -> f64 {
    let xm = x - 1.0;
    let sum = lanczos_sum(xm);
    let t = xm + LANCZOS_G + 0.5;
    ((2.0 * std::f64::consts::PI).sqrt() * t.powf(xm + 0.5) * (-t).exp()) * sum
}

#[must_use]
pub fn gamma(x: f64) -> f64 {
    if x <= 0.0 && x == x.floor() {
        return f64::INFINITY;
    }
    if x < 0.5 {
        std::f64::consts::PI / ((x * std::f64::consts::PI).sin() * gamma_positive(1.0 - x))
    } else {
        gamma_positive(x)
    }
}

#[must_use]
pub fn lgamma(x: f64) -> f64 {
    if x <= 0.0 && x == x.floor() {
        return f64::INFINITY;
    }
    let xm = if x < 0.5 { 1.0 - x } else { x - 1.0 };
    let sum = lanczos_sum(xm);
    let t = xm + LANCZOS_G + 0.5;
    let r = (2.0 * std::f64::consts::PI).sqrt().ln() + ((xm + 0.5) * t.ln()) - t + sum.ln();
    if x < 0.5 {
        (std::f64::consts::PI / ((x * std::f64::consts::PI).sin() * r.exp()))
            .abs()
            .ln()
    } else {
        r
    }
}

#[must_use]
pub fn frexp(x: f64) -> Vec<f64> {
    if x == 0.0 || !x.is_finite() {
        return vec![x, 0.0];
    }
    let bits = x.to_bits();
    let sign_mask = 1_u64 << 63;
    let frac_mask = (1_u64 << 52) - 1;
    let sign = bits & sign_mask;
    let exp = ((bits >> 52) & 2047) as i32;
    let frac = bits & frac_mask;
    if exp == 0 {
        let scaled = x * 2.0_f64.powi(54);
        let sbits = scaled.to_bits();
        let sexp = ((sbits >> 52) & 2047) as i32;
        let sfrac = sbits & frac_mask;
        let mant = f64::from_bits(sign | (1022_u64 << 52) | sfrac);
        vec![mant, f64::from(sexp - 1022 - 54)]
    } else {
        let mant = f64::from_bits(sign | (1022_u64 << 52) | frac);
        vec![mant, f64::from(exp - 1022)]
    }
}

#[must_use]
pub fn ldexp(m: f64, e: SifrIntBridge) -> f64 {
    m * 2.0_f64.powi(e.to_i64_saturating() as i32)
}

#[must_use]
pub fn modf(x: f64) -> Vec<f64> {
    if x.is_nan() {
        return vec![f64::NAN, f64::NAN];
    }
    if x.is_infinite() {
        return vec![0.0_f64.copysign(x), x];
    }
    let int = x.trunc();
    let mut frac = x - int;
    if frac == 0.0 {
        frac = 0.0_f64.copysign(x);
    }
    vec![frac, int]
}

#[must_use]
pub fn nextafter(x: f64, y: f64) -> f64 {
    if x.is_nan() || y.is_nan() {
        return f64::NAN;
    }
    if x == y {
        return y;
    }
    if x == 0.0 {
        let sign = if y.is_sign_negative() { 1_u64 << 63 } else { 0 };
        return f64::from_bits(sign | 1);
    }
    let mut bits = x.to_bits();
    if (x < y) == (x > 0.0) {
        bits += 1;
    } else {
        bits -= 1;
    }
    f64::from_bits(bits)
}

#[must_use]
pub fn ulp(x: f64) -> f64 {
    if x.is_nan() {
        f64::NAN
    } else if x.is_infinite() {
        f64::INFINITY
    } else if x == 0.0 {
        f64::from_bits(1)
    } else if x.abs() == f64::MAX {
        x.abs() - f64::from_bits(x.abs().to_bits() - 1)
    } else {
        (nextafter(x.abs(), f64::INFINITY) - x.abs()).abs()
    }
}
