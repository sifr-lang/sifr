// src/main.rs
fn pattern_basic() -> i64 {
    let add = |a: i64, b: i64| {
    a + b
};
    add(3_i64, 7_i64)
}

fn pattern_closure() -> i64 {
    let multiplier: i64 = 3_i64;
    let multiply = |x: i64| {
    x * multiplier
};
    multiply(5_i64)
}

fn pattern_recursive() -> i64 {
    fn factorial(n: i64) -> i64 {
        if n <= (1_i64) {
            return 1_i64;
        }
        return n * factorial(n - (1_i64));
    }
    factorial(6_i64)
}

fn pattern_recursive_capture() -> i64 {
    let limit: i64 = 100_i64;
    fn sum_up(i: i64, acc: i64, limit: i64) -> i64 {
        if i > limit {
            return acc;
        }
        return sum_up(i + (1_i64), acc + i, limit);
    }
    sum_up(1_i64, 0_i64, limit)
}

fn pattern_multiple() -> String {
    let greet = |name: &String| {
    {
    let mut __sifr_concat: String = String::with_capacity(7usize + name.len());
    __sifr_concat.push_str("Hello, ");
    __sifr_concat.push_str((name).as_str());
    __sifr_concat
}
};
    let exclaim = |msg: &String| {
    {
    let mut __sifr_concat: String = String::with_capacity(msg.len() + 1usize);
    __sifr_concat.push_str((msg).as_str());
    __sifr_concat.push('!');
    __sifr_concat
}
};
    exclaim(&greet(&"Sifr".to_string()))
}

fn pattern_params() -> i64 {
    fn power(base: i64, exp: i64) -> i64 {
        if exp <= (0_i64) {
            return 1_i64;
        }
        return base * power(base, exp - (1_i64));
    }
    let a: i64 = power(2_i64, 10_i64);
    let b: i64 = power(3_i64, 4_i64);
    a + b
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
