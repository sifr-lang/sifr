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
fn pow(x: f64, y: f64) -> f64 {
    pow_val(x, y)
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Container<T> {
    value: T,
}
impl<T> Container<T> {
    fn new(value: T) -> Self {
        let __sifr_field_init_0: T = value;
        Self { value: __sifr_field_init_0 }
    }
}
impl<T: Clone> Container<T> {
    fn get(&self) -> T {
        self.value.clone()
    }
}
impl<T: ::std::fmt::Display> ::std::fmt::Display for Container<T> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Container(value={})", self.value)
    }
}
pub trait Printable {
    fn display(&self) -> String;
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct User {
    name: String,
}
impl User {
    fn new(name: String) -> Self {
        let __sifr_field_init_0: String = name;
        Self { name: __sifr_field_init_0 }
    }
}
impl User {
    fn display(&self) -> String {
        format!("User({})", self.name.clone())
    }
}
impl ::std::fmt::Display for User {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "User(name={})", self.name)
    }
}
impl Printable for User {
    fn display(&self) -> String {
        User::display(self)
    }
}
#[derive(Debug, Clone, PartialEq)]
struct Product {
    title: String,
    price: f64,
}
impl Product {
    fn new(title: String, price: f64) -> Self {
        let __sifr_field_init_0: String = title;
        let __sifr_field_init_1: f64 = price;
        Self {
            title: __sifr_field_init_0,
            price: __sifr_field_init_1,
        }
    }
}
impl Product {
    fn display(&self) -> String {
        format!("Product({}, ${})", self.title.clone(), self.price)
    }
}
impl ::std::fmt::Display for Product {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Product(title={}, price={})", self.title, self.price)
    }
}
impl Printable for Product {
    fn display(&self) -> String {
        Product::display(self)
    }
}
fn identity<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(x: &T) -> T {
    x.clone()
}
fn repeat<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    x: &T,
    n: SifrInt,
) -> Vec<T> {
    let mut result: Vec<T> = vec![];
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &n) {
        result.push(x.clone());
        i = &i + &SifrInt::from_i64(1);
    }
    result
}
fn show(item: Box<dyn Printable>) {
    println!("{}", item.display());
}
fn main() {
    println!("=== PEP 695 Generic Functions ===");
    println!("{}", identity(& SifrInt::from_i64(42)));
    println!("{}", identity(& "hello".to_string()));
    println!("{:?}", repeat(& "x".to_string(), SifrInt::from_i64(3)));
    println!("=== PEP 695 Generic Classes ===");
    let c: Container<SifrInt> = Container::new(SifrInt::from_i64(99));
    println!("{}", c.get());
    let c2: Container<String> = Container::new("wrapped".to_string());
    println!("{}", c2.get());
    println!("=== Protocol Method Dispatch ===");
    let u: User = User::new("Alice".to_string());
    let pr: Product = Product::new("Widget".to_string(), 9.99_f64);
    show(Box::new(u));
    show(Box::new(pr));
    println!("=== Multi-Generator Comprehensions ===");
    let matrix: Vec<Vec<SifrInt>> = vec![
        vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)],
        vec![SifrInt::from_i64(4), SifrInt::from_i64(5), SifrInt::from_i64(6)],
        vec![SifrInt::from_i64(7), SifrInt::from_i64(8), SifrInt::from_i64(9)]
    ];
    let flat: Vec<SifrInt> = {
        let mut __sifr_list_comp = vec![];
        for row in matrix.iter().cloned() {
            for x in row.iter().cloned() {
                __sifr_list_comp.push(x);
            }
        }
        __sifr_list_comp
    };
    println!("{:?}", flat);
    println!("=== Stdlib Math Functions ===");
    println!("{}", log(1.0_f64));
    println!("{}", sin(0.0_f64));
    println!("{}", cos(0.0_f64));
    println!("{}", fabs(- (42.0_f64)));
    println!("{}", ((2.0_f64) as f64).powf((10.0_f64) as f64));
    println!(
        "{:?}", SifrInt::from_f64_trunc((3.14_f64).round_ties_even()).ok_or_else(||
        ValueError { message : "cannot round non-finite float to int".to_string() })
    );
}
