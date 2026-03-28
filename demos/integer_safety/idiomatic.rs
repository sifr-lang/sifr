use num_bigint::BigInt;

fn factorial(n: i64) -> BigInt {
    if n <= (1 as i64) {
        return BigInt::from(1 as i64);
    }
    return BigInt::from(n).clone() * factorial(n - (1 as i64)).clone();
}

fn fibonacci(n: i64) -> BigInt {
    if n <= (1 as i64) {
        return BigInt::from(n);
    }
    let mut a: BigInt = BigInt::from(0 as i64);
    let mut b: BigInt = BigInt::from(1 as i64);
    let mut i: i64 = 2 as i64;
    while i <= n {
        let c: BigInt = a.clone() + b.clone();
        a = b.clone();
        b = c.clone();
        i = i + (1 as i64);
    }
    return b;
}

fn main() {
    println!("=== Basic BigInt ===");
    let x: BigInt = BigInt::from(42 as i64);
    let y: BigInt = BigInt::from(100 as i64);
    println!("{}", x);
    println!("{}", y);
    println!("=== Arithmetic ===");
    println!("{}", x.clone() + y.clone());
    println!("{}", y.clone() - x.clone());
    println!("{}", x.clone() * y.clone());
    println!(
        "{}",
        (BigInt::from(10 as i64).clone()).pow((20 as i64) as u32)
    );
    println!("=== Large Values ===");
    let huge: BigInt = (BigInt::from(2 as i64).clone()).pow((100 as i64) as u32);
    println!("{}", huge);
    println!("=== Comparison ===");
    let a: BigInt = BigInt::from(100 as i64);
    let b: BigInt = BigInt::from(200 as i64);
    println!("{}", a < b);
    println!("{}", a == BigInt::from(100 as i64));
    println!("{}", b > a);
    println!("=== int to bigint ===");
    let n: i64 = 999 as i64;
    let big_n: BigInt = BigInt::from(n);
    println!("{}", big_n);
    println!("=== Factorial ===");
    println!("{}", factorial(10 as i64));
    println!("{}", factorial(20 as i64));
    println!("{}", factorial(30 as i64));
    println!("=== Fibonacci ===");
    println!("{}", fibonacci(50 as i64));
    println!("{}", fibonacci(100 as i64));
    println!("=== Overflow Warnings (check stderr) ===");
    let base: i64 = 2 as i64;
    let exp: i64 = 10 as i64;
    let risky_pow: i64 = (base).pow(exp as u32);
    println!("{}", risky_pow);
    let a2: i64 = 1000000 as i64;
    let b2: i64 = 1000000 as i64;
    let risky_mul: i64 = a2 * b2;
    println!("{}", risky_mul);
    let safe_mul: i64 = (5 as i64) * (10 as i64);
    println!("{}", safe_mul);
}
