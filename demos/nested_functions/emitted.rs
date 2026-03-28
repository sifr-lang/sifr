fn pattern_basic() -> i64 {
    let add = |a, b| {
    return a + b;
};
    return add(3 as i64, 7 as i64);
}

fn pattern_closure() -> i64 {
    let multiplier: i64 = 3 as i64;
    let multiply = |x| {
    return x * multiplier;
};
    return multiply(5 as i64);
}

fn pattern_recursive() -> i64 {
    fn factorial(n: i64) -> i64 {
        if n <= (1 as i64) {
            return 1 as i64;
        }
        return n * factorial(n - (1 as i64));
    }
    return factorial(6 as i64);
}

fn pattern_recursive_capture() -> i64 {
    let limit: i64 = 100 as i64;
    fn sum_up(i: i64, acc: i64, limit: i64) -> i64 {
        if i > limit {
            return acc;
        }
        return sum_up(i + (1 as i64), acc + i, limit);
    }
    return sum_up(1 as i64, 0 as i64, limit);
}

fn pattern_multiple() -> String {
    let greet = |name| {
    return format!("{}{}", "Hello, ".to_string(), name);
};
    let exclaim = |msg| {
    return format!("{}{}", msg, "!".to_string());
};
    return exclaim(greet("Sifr".to_string()));
}

fn pattern_params() -> i64 {
    fn power(base: i64, exp: i64) -> i64 {
        if exp <= (0 as i64) {
            return 1 as i64;
        }
        return base * power(base, exp - (1 as i64));
    }
    let a: i64 = power(2 as i64, 10 as i64);
    let b: i64 = power(3 as i64, 4 as i64);
    return a + b;
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
