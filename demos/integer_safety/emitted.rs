// src/main.rs
use ::sifr_runtime::SifrInt;
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn factorial(n: SifrInt) -> SifrInt {
    if n <= SifrInt::from_i64(1) {
        return SifrInt::from_i64(1);
    }
    ::std::ops::Mul::mul(
        &n,
        &factorial(::std::ops::Sub::sub(&n, &SifrInt::from_i64(1))),
    )
}
#[expect(
    clippy::many_single_char_names,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn fibonacci(n: SifrInt) -> SifrInt {
    if n <= SifrInt::from_i64(1) {
        return n;
    }
    let mut a: SifrInt = SifrInt::from_i64(0);
    let mut b: SifrInt = SifrInt::from_i64(1);
    let mut i: SifrInt = SifrInt::from_i64(2);
    while i <= n {
        let c: SifrInt = ::std::ops::Add::add(&a, &b);
        a.clone_from(&b);
        b.clone_from(&c);
        i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
    }
    b
}
fn main() {
    println!("=== Basic exact int ===");
    let x: SifrInt = SifrInt::from_i64(42);
    let y: SifrInt = SifrInt::from_i64(100);
    println!("{x}");
    println!("{y}");
    println!("=== Arithmetic ===");
    println!("{}", ::std::ops::Add::add(&x, &y));
    println!("{}", ::std::ops::Sub::sub(&y, &x));
    println!("{}", ::std::ops::Mul::mul(&x, &y));
    println!("{}", SifrInt::from_i64(10).pow_known_valid(9_u32));
    println!("=== Exact integer constants ===");
    let exact_value: SifrInt = SifrInt::from_i64(2).pow_known_valid(30_u32);
    println!("{exact_value}");
    println!("=== Comparison ===");
    let a: SifrInt = SifrInt::from_i64(100);
    let b: SifrInt = SifrInt::from_i64(200);
    println!("{}", a < b);
    println!("{}", a == SifrInt::from_i64(100));
    println!("{}", b > a);
    println!("=== Fixed-width checks ===");
    let byte_value: u8 = 255u8;
    println!("{byte_value}");
    let widened: SifrInt = SifrInt::from(byte_value);
    println!("{}", ::std::ops::Add::add(&widened, &SifrInt::from_i64(1)));
    println!("=== Factorial ===");
    println!("{}", factorial(SifrInt::from_i64(10)));
    println!("{}", factorial(SifrInt::from_i64(12)));
    println!("=== Fibonacci ===");
    println!("{}", fibonacci(SifrInt::from_i64(20)));
    println!("{}", fibonacci(SifrInt::from_i64(40)));
}
