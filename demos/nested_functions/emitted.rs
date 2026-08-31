// src/main.rs
use ::sifr_runtime::SifrInt;

fn pattern_basic() -> SifrInt {
    let add = |a: SifrInt, b: SifrInt| {
    &a + &b
};
    add(SifrInt::from_i64(3), SifrInt::from_i64(7))
}

fn pattern_closure() -> SifrInt {
    let multiplier: SifrInt = SifrInt::from_i64(3);
    let multiply = |x: SifrInt| {
    &x * &multiplier
};
    multiply(SifrInt::from_i64(5))
}

fn pattern_recursive() -> SifrInt {
    fn factorial(n: SifrInt) -> SifrInt {
        if &n <= &SifrInt::from_i64(1) {
            return SifrInt::from_i64(1);
        }
        return &n * &factorial(&n - &SifrInt::from_i64(1));
    }
    factorial(SifrInt::from_i64(6))
}

fn pattern_recursive_capture() -> SifrInt {
    let limit: SifrInt = SifrInt::from_i64(100);
    fn sum_up(i: SifrInt, acc: SifrInt, limit: SifrInt) -> SifrInt {
        if &i > &limit {
            return acc.clone();
        }
        return sum_up(&i + &SifrInt::from_i64(1), &acc + &i, limit.clone());
    }
    sum_up(SifrInt::from_i64(1), SifrInt::from_i64(0), limit.clone())
}

fn pattern_multiple() -> String {
    let greet = |name: &str| {
    {
    let mut __sifr_concat: String = String::with_capacity(7usize + name.len());
    __sifr_concat.push_str("Hello, ");
    __sifr_concat.push_str(name);
    __sifr_concat
}
};
    let exclaim = |msg: &str| {
    {
    let mut __sifr_concat: String = String::with_capacity(msg.len() + 1usize);
    __sifr_concat.push_str(msg);
    __sifr_concat.push('!');
    __sifr_concat
}
};
    exclaim(&greet(&"Sifr".to_string()))
}

fn pattern_params() -> SifrInt {
    fn power(base: SifrInt, exp: SifrInt) -> SifrInt {
        if &exp <= &SifrInt::from_i64(0) {
            return SifrInt::from_i64(1);
        }
        return &base * &power(base.clone(), &exp - &SifrInt::from_i64(1));
    }
    let a: SifrInt = power(SifrInt::from_i64(2), SifrInt::from_i64(10));
    let b: SifrInt = power(SifrInt::from_i64(3), SifrInt::from_i64(4));
    &a + &b
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
