// src/main.rs
use ::sifr_runtime::SifrInt;
fn factorial(n: &SifrInt) -> SifrInt {
    if n <= &SifrInt::from_i64(1) {
        return SifrInt::from_i64(1);
    }
    ::std::ops::Mul::mul(
        n,
        &factorial(&::std::ops::Sub::sub(n, &SifrInt::from_i64(1))),
    )
}
fn fibonacci(n: &SifrInt) -> SifrInt {
    if n <= &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    if n == &SifrInt::from_i64(1) {
        return SifrInt::from_i64(1);
    }
    ::std::ops::Add::add(
        &fibonacci(&::std::ops::Sub::sub(n, &SifrInt::from_i64(1))),
        &fibonacci(&::std::ops::Sub::sub(n, &SifrInt::from_i64(2))),
    )
}
fn greet(name: &str) -> String {
    {
        let mut sifr_generated_concat: String =
            String::with_capacity(7usize.saturating_add(name.len()).saturating_add(1usize));
        sifr_generated_concat.push_str("Hello, ");
        sifr_generated_concat.push_str(name);
        sifr_generated_concat.push('!');
        sifr_generated_concat
    }
}
fn classify(x: &SifrInt) -> String {
    if x > &SifrInt::from_i64(0) {
        return "positive".to_string();
    }
    if x < &SifrInt::from_i64(0) {
        return "negative".to_string();
    }
    "zero".to_string()
}
fn double(n: &SifrInt) -> SifrInt {
    ::std::ops::Mul::mul(n, &SifrInt::from_i64(2))
}
fn is_even(n: &SifrInt) -> bool {
    if n == &SifrInt::from_i64(0) {
        return true;
    }
    if n == &SifrInt::from_i64(1) {
        return false;
    }
    is_even(&::std::ops::Sub::sub(n, &SifrInt::from_i64(2)))
}
fn main() {
    let x: SifrInt = SifrInt::from_i64(42);
    let name: String = "Sifr".to_string();
    let sum: SifrInt = ::std::ops::Add::add(&x, &SifrInt::from_i64(8));
    let product: SifrInt = double(&sum);
    println!("{product}");
    let fact: SifrInt = factorial(&SifrInt::from_i64(5));
    println!("{fact}");
    let fib: SifrInt = fibonacci(&SifrInt::from_i64(10));
    println!("{fib}");
    let msg: String = greet(name.as_str());
    println!("{msg}");
    let label: String = classify(&x);
    println!("{label}");
    let neg_label: String = classify(&::std::ops::Neg::neg(&SifrInt::from_i64(7)));
    println!("{neg_label}");
    let zero_label: String = classify(&SifrInt::from_i64(0));
    println!("{zero_label}");
    let even: bool = is_even(&SifrInt::from_i64(4));
    println!("{even}");
    let odd: bool = is_even(&SifrInt::from_i64(7));
    println!("{odd}");
}
