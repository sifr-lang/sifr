use num_bigint::BigInt;

fn factorial(n: i64) -> BigInt {
    let mut result = BigInt::from(1);

    for value in 2..=n {
        result *= value;
    }

    result
}

fn fibonacci(n: i64) -> BigInt {
    if n <= 1 {
        return BigInt::from(n);
    }

    let mut a = BigInt::from(0);
    let mut b = BigInt::from(1);

    for _ in 2..=n {
        let next = &a + &b;
        a = b;
        b = next;
    }

    b
}

fn main() {
    println!("=== Basic BigInt ===");
    let x = BigInt::from(42);
    let y = BigInt::from(100);
    println!("{x}");
    println!("{y}");

    println!("=== Arithmetic ===");
    println!("{}", &x + &y);
    println!("{}", &y - &x);
    println!("{}", &x * &y);
    println!("{}", BigInt::from(10).pow(20));

    println!("=== Large Values ===");
    let huge = BigInt::from(2).pow(100);
    println!("{huge}");

    println!("=== Comparison ===");
    let a = BigInt::from(100);
    let b = BigInt::from(200);
    println!("{}", a < b);
    println!("{}", a == BigInt::from(100));
    println!("{}", b > a);

    println!("=== int to bigint ===");
    let n = 999_i64;
    let big_n = BigInt::from(n);
    println!("{big_n}");

    println!("=== Factorial ===");
    println!("{}", factorial(10));
    println!("{}", factorial(20));
    println!("{}", factorial(30));

    println!("=== Fibonacci ===");
    println!("{}", fibonacci(50));
    println!("{}", fibonacci(100));

    println!("=== Overflow Warnings (check stderr) ===");
    let base = 2_i64;
    let exp = 10_u32;
    let risky_pow = base.pow(exp);
    println!("{risky_pow}");

    let a2 = 1_000_000_i64;
    let b2 = 1_000_000_i64;
    let risky_mul = a2 * b2;
    println!("{risky_mul}");

    let safe_mul = 5_i64 * 10_i64;
    println!("{safe_mul}");
}
