// src/main.rs
mod helper;

use crate::helper::area_like;

fn main() {
    println!("project_check project-aware check parity demo:");
    println!("{}", area_like(3.0_f64));
}

// src/helper.rs
pub use ::sifr_runtime::SifrInt;
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
pub fn area_like(r: f64) -> f64 {
    PI * r
}
