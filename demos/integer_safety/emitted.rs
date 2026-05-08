// src/main.rs
fn factorial(n: i64) -> i64 {
    if n <= (1 as i64) {
        return 1 as i64;
    }
    return n * factorial(n - (1 as i64));
}

fn fibonacci(n: i64) -> i64 {
    if n <= (1 as i64) {
        return n;
    }
    let mut a: i64 = 0 as i64;
    let mut b: i64 = 1 as i64;
    let mut i: i64 = 2 as i64;
    while i <= n {
        let c: i64 = a + b;
        a = b;
        b = c;
        i = i + (1 as i64);
    }
    return b;
}

fn main() {
    println!("=== Basic exact int ===");
    let x: i64 = 42 as i64;
    let y: i64 = 100 as i64;
    println!("{}", x);
    println!("{}", y);
    println!("=== Arithmetic ===");
    println!("{}", x + y);
    println!("{}", y - x);
    println!("{}", x * y);
    println!("{}", (10 as i64).pow((9 as i64) as u32));
    println!("=== Exact integer constants ===");
    let exact_value: i64 = (2 as i64).pow((30 as i64) as u32);
    println!("{}", exact_value);
    println!("=== Comparison ===");
    let a: i64 = 100 as i64;
    let b: i64 = 200 as i64;
    println!("{}", (a < b));
    println!("{}", (a == (100 as i64)));
    println!("{}", (b > a));
    println!("=== Fixed-width checks ===");
    let byte_value: u8 = 255u8;
    println!("{}", byte_value);
    let widened: i64 = byte_value as i64;
    println!("{}", widened + (1 as i64));
    println!("=== Factorial ===");
    println!("{}", factorial(10 as i64));
    println!("{}", factorial(12 as i64));
    println!("=== Fibonacci ===");
    println!("{}", fibonacci(20 as i64));
    println!("{}", fibonacci(40 as i64));
}
