fn factorial(n: i64) -> i64 {
    if n <= 1 {
        return 1;
    }
    n * factorial(n - 1)
}

fn fibonacci(n: i64) -> i64 {
    if n <= 1 {
        return n;
    }

    let mut a = 0;
    let mut b = 1;
    for _ in 2..=n {
        let c = a + b;
        a = b;
        b = c;
    }
    b
}

fn main() {
    println!("=== Basic exact int ===");
    let x = 42;
    let y = 100;
    println!("{x}");
    println!("{y}");

    println!("=== Arithmetic ===");
    println!("{}", x + y);
    println!("{}", y - x);
    println!("{}", x * y);
    println!("{}", 10_i64.pow(9));

    println!("=== Exact integer constants ===");
    let exact_value = 2_i64.pow(30);
    println!("{exact_value}");

    println!("=== Comparison ===");
    let a = 100;
    let b = 200;
    println!("{}", a < b);
    println!("{}", a == 100);
    println!("{}", b > a);

    println!("=== Fixed-width checks ===");
    let byte_value: u8 = 255;
    println!("{byte_value}");
    let widened = i64::from(byte_value);
    println!("{}", widened + 1);

    println!("=== Factorial ===");
    println!("{}", factorial(10));
    println!("{}", factorial(12));

    println!("=== Fibonacci ===");
    println!("{}", fibonacci(20));
    println!("{}", fibonacci(40));
}
