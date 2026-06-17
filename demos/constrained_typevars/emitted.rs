fn echo<T: Clone + std::fmt::Display + PartialOrd + 'static>(x: &T) -> T {
    return x.clone();
}

fn smallest<U: Clone + std::fmt::Display + PartialOrd + 'static>(a: &U, b: &U) -> U {
    if *a < *b {
        return a.clone();
    }
    return b.clone();
}

fn main() {
    println!("constrained_typevars typevar constraint enforcement demo:");
    println!("{}", echo(&(7 as i64)));
    println!("{}", echo(&"ok".to_string()));
    println!("{}", smallest(&(10 as i64), &(3 as i64)));
}
