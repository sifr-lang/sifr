fn pattern_basic() -> i64 {
    let add = |a: i64, b: i64| a + b;
    add(3, 7)
}

fn pattern_closure() -> i64 {
    let multiplier = 3;
    let multiply = |x: i64| x * multiplier;
    multiply(5)
}

fn pattern_recursive() -> i64 {
    fn factorial(n: i64) -> i64 {
        if n <= 1 {
            1
        } else {
            n * factorial(n - 1)
        }
    }

    factorial(6)
}

fn pattern_recursive_capture() -> i64 {
    let limit = 100;

    fn sum_up(i: i64, acc: i64, limit: i64) -> i64 {
        if i > limit {
            acc
        } else {
            sum_up(i + 1, acc + i, limit)
        }
    }

    sum_up(1, 0, limit)
}

fn pattern_multiple() -> String {
    let greet = |name: &str| format!("Hello, {name}");
    let exclaim = |message: String| format!("{message}!");
    exclaim(greet("Sifr"))
}

fn pattern_params() -> i64 {
    fn power(base: i64, exp: i64) -> i64 {
        if exp <= 0 {
            1
        } else {
            base * power(base, exp - 1)
        }
    }

    power(2, 10) + power(3, 4)
}

fn main() {
    println!("Pattern 1 - Basic nested function:");
    println!("{}", pattern_basic());

    println!("Pattern 2 - Closure (captures outer var):");
    println!("{}", pattern_closure());

    println!("Pattern 3 - Recursive nested function:");
    println!("{}", pattern_recursive());

    println!("Pattern 4 - Recursive with capture:");
    println!("{}", pattern_recursive_capture());

    println!("Pattern 5 - Multiple nested functions:");
    println!("{}", pattern_multiple());

    println!("Pattern 6 - Nested with params:");
    println!("{}", pattern_params());
}
