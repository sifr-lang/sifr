// Reference: core_language
// Source issue: 3-core_language-core-language-compiler.md
fn factorial(n: i64) -> i64 {
    if n <= 1_i64 {
        return 1_i64;
    }
    return n * factorial(n - 1_i64);
}

fn fibonacci(n: i64) -> i64 {
    if n <= 0_i64 {
        return 0_i64;
    }
    if n == 1_i64 {
        return 1_i64;
    }
    return fibonacci(n - 1_i64) + fibonacci(n - 2_i64);
}

fn greet(name: String) -> String {
    return format!("{}{}{}", "Hello, ".to_string(), name, "!".to_string());
}

fn classify(x: i64) -> String {
    if x > 0_i64 {
        return "positive".to_string();
    }
    if x < 0_i64 {
        return "negative".to_string();
    }
    return "zero".to_string();
}

fn double(n: i64) -> i64 {
    return n * 2_i64;
}

fn is_even(n: i64) -> bool {
    if n == 0_i64 {
        return true;
    }
    if n == 1_i64 {
        return false;
    }
    return is_even(n - 2_i64);
}

fn main() {
    let x: i64 = 42_i64;
    let pi: f64 = 3.14_f64;
    let flag: bool = true;
    let name: String = "Sifr".to_string();
    let sum: i64 = x + 8_i64;
    let product: i64 = double(sum);
    println!("{}", product);
    let fact: i64 = factorial(5_i64);
    println!("{}", fact);
    let fib: i64 = fibonacci(10_i64);
    println!("{}", fib);
    let msg: String = greet(name);
    println!("{}", msg);
    let label: String = classify(x);
    println!("{}", label);
    let neg_label: String = classify(-7_i64);
    println!("{}", neg_label);
    let zero_label: String = classify(0_i64);
    println!("{}", zero_label);
    let even: bool = is_even(4_i64);
    println!("{}", even);
    let odd: bool = is_even(7_i64);
    println!("{}", odd);
}
