// src/main.rs
use ::sifr_runtime::SifrInt;

fn factorial(n: i64) -> i64 {
    if n <= (1_i64) {
        return 1_i64;
    }
    n * factorial(n - (1_i64))
}

fn fibonacci(n: i64) -> i64 {
    if n <= (1_i64) {
        return n;
    }
    let mut a: i64 = 0_i64;
    let mut b: i64 = 1_i64;
    let mut i: i64 = 2_i64;
    while i <= n {
        let c: i64 = a + b;
        a = b;
        b = c;
        i += 1_i64;
    }
    b
}

fn main() {
    println!("=== Basic exact int ===");
    let x: i64 = 42_i64;
    let y: i64 = 100_i64;
    println!("{}", x);
    println!("{}", y);
    println!("=== Arithmetic ===");
    println!("{}", x + y);
    println!("{}", y - x);
    println!("{}", x * y);
    println!("{}", SifrInt::from_i64(10).pow((9_i64) as u32));
    println!("=== Exact integer constants ===");
    let exact_value: SifrInt = SifrInt::from_i64(2).pow((30_i64) as u32);
    println!("{}", exact_value);
    println!("=== Comparison ===");
    let a: i64 = 100_i64;
    let b: i64 = 200_i64;
    println!("{}", (a < b));
    println!("{}", (a == (100_i64)));
    println!("{}", (b > a));
    println!("=== Fixed-width checks ===");
    let byte_value: u8 = 255u8;
    println!("{}", byte_value);
    let widened: i64 = byte_value as i64;
    println!("{}", widened + (1_i64));
    println!("=== Factorial ===");
    println!("{}", factorial(10_i64));
    println!("{}", factorial(12_i64));
    println!("=== Fibonacci ===");
    println!("{}", fibonacci(20_i64));
    println!("{}", fibonacci(40_i64));
}
