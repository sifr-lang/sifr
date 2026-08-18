// src/main.rs
fn echo<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(x: &T) -> T {
    x.clone()
}

fn smallest<U: Clone + ::std::fmt::Display + PartialOrd + 'static>(a: &U, b: &U) -> U {
    if *a < *b {
        return a.clone();
    }
    b.clone()
}

fn main() {
    println!("constrained_typevars typevar constraint enforcement demo:");
    println!("{}", echo(&(7_i64)));
    println!("{}", echo(&"ok".to_string()));
    println!("{}", smallest(&(10_i64), &(3_i64)));
}
